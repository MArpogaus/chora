//! Layer surfaces of `wlr-layer-shell`: panels, launchers, wallpapers and
//! notifications.
//!
//! The core places layer surfaces by the protocol, in the logical
//! coordinates of their output and not through its camera, so a panel stays
//! in place when a plugin pans the scene. Panels reserve exclusive zones;
//! the rest of an output is its work area.

use smithay::{
    desktop::{LayerSurface, WindowSurfaceType, layer_map_for_output},
    output::Output,
    reexports::wayland_server::protocol::{wl_output::WlOutput, wl_surface::WlSurface},
    utils::{Logical, Point, Rectangle},
    wayland::{
        compositor::with_states,
        shell::{
            wlr_layer::{
                KeyboardInteractivity, Layer, LayerSurface as WlrLayerSurface, LayerSurfaceData,
                WlrLayerShellHandler, WlrLayerShellState,
            },
            xdg::PopupSurface,
        },
    },
};

use crate::state::State;

/// The layers above the scene, front first.
pub const ABOVE: [Layer; 2] = [Layer::Overlay, Layer::Top];
/// The layers below the scene, front first.
pub const BELOW: [Layer; 2] = [Layer::Bottom, Layer::Background];

impl WlrLayerShellHandler for State {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.layer_shell_state
    }

    fn new_layer_surface(
        &mut self,
        surface: WlrLayerSurface,
        output: Option<WlOutput>,
        _layer: Layer,
        namespace: String,
    ) {
        let output = output
            .as_ref()
            .and_then(Output::from_resource)
            .or_else(|| self.outputs.first().cloned())
            .filter(|o| self.outputs.contains(o));
        let Some(output) = output else {
            surface.send_close();
            return;
        };
        let mapped =
            layer_map_for_output(&output).map_layer(&LayerSurface::new(surface, namespace));
        if let Err(err) = mapped {
            tracing::warn!(?err, "a layer surface failed");
        }
    }

    fn new_popup(&mut self, _parent: WlrLayerSurface, popup: PopupSurface) {
        self.unconstrain_popup(&popup);
    }

    fn layer_destroyed(&mut self, surface: WlrLayerSurface) {
        for output in self.outputs.clone() {
            let mut map = layer_map_for_output(&output);
            let layer = map
                .layers()
                .find(|l| l.layer_surface() == &surface)
                .cloned();
            if let Some(layer) = layer {
                // The unmap places the other layers again, so the zone
                // before it decides whether the work area changed.
                let before = map.non_exclusive_zone();
                map.unmap_layer(&layer);
                drop(map);
                self.announce_zone(&output, before);
            }
        }
    }
}

impl State {
    /// Handles a commit of a layer surface: the initial configure and the
    /// new placement. Returns `false` for other surfaces.
    pub fn layer_committed(&mut self, surface: &WlSurface) -> bool {
        let Some(output) = self.output_of_layer(surface) else {
            return false;
        };
        let configured = with_states(surface, |states| {
            states
                .data_map
                .get::<LayerSurfaceData>()
                .is_some_and(|d| d.lock().expect("not poisoned").initial_configure_sent)
        });
        // The placement comes first, so that the initial configure has a
        // size.
        self.layers_changed(&output);
        if !configured
            && let Some(layer) = layer_map_for_output(&output)
                .layer_for_surface(surface, WindowSurfaceType::TOPLEVEL)
        {
            layer.layer_surface().send_configure();
        }
        true
    }

    /// The output that shows a layer surface.
    pub fn output_of_layer(&self, surface: &WlSurface) -> Option<Output> {
        self.outputs
            .iter()
            .find(|o| {
                layer_map_for_output(o)
                    .layer_for_surface(surface, WindowSurfaceType::ALL)
                    .is_some()
            })
            .cloned()
    }

    /// Places the layers of an output again, for example after a mode
    /// change, and announces a new work area.
    pub fn layers_changed(&mut self, output: &Output) {
        let before = layer_map_for_output(output).non_exclusive_zone();
        self.announce_zone(output, before);
    }

    /// Places the layers of an output and announces a work area that
    /// differs from the zone before.
    fn announce_zone(&mut self, output: &Output, before: Rectangle<i32, Logical>) {
        let mut map = layer_map_for_output(output);
        map.arrange();
        let changed = map.non_exclusive_zone() != before;
        drop(map);
        if changed {
            self.output_changed();
        }
        self.update_focus();
    }

    /// The layer surface of a surface, or of the root of a popup, if it
    /// accepts the keyboard.
    pub fn keyboard_layer(&self, surface: &WlSurface) -> Option<WlSurface> {
        self.layer_keyboard(surface)
            .filter(|(_, k)| *k != KeyboardInteractivity::None)
            .map(|(l, _)| l)
    }

    /// Whether a surface is a layer surface that takes the keyboard on
    /// demand, as the search field of a panel does.
    pub fn on_demand_layer(&self, surface: &WlSurface) -> bool {
        self.layer_keyboard(surface)
            .is_some_and(|(_, k)| k == KeyboardInteractivity::OnDemand)
    }

    /// The layer surface of a surface, or of the root of a popup, and how
    /// it takes the keyboard.
    fn layer_keyboard(&self, surface: &WlSurface) -> Option<(WlSurface, KeyboardInteractivity)> {
        self.outputs.iter().find_map(|output| {
            let map = layer_map_for_output(output);
            map.layer_for_surface(surface, WindowSurfaceType::ALL)
                .map(|l| {
                    (
                        l.wl_surface().clone(),
                        l.cached_state().keyboard_interactivity,
                    )
                })
        })
    }

    /// The topmost layer surface that takes the keyboard as long as it is
    /// mapped, as a launcher does.
    pub fn exclusive_layer(&self) -> Option<WlSurface> {
        self.outputs.iter().find_map(|output| {
            let map = layer_map_for_output(output);
            ABOVE.iter().find_map(|layer| {
                map.layers_on(*layer)
                    .rev()
                    .find(|l| {
                        l.cached_state().keyboard_interactivity == KeyboardInteractivity::Exclusive
                    })
                    .map(|l| l.wl_surface().clone())
            })
        })
    }
}

/// The layer surface of an output under a point in logical output
/// coordinates, on one of the given layers: the surface, possibly a popup,
/// and its position in output coordinates.
pub fn surface_under(
    output: &Output,
    layers: &[Layer],
    point: Point<f64, Logical>,
) -> Option<(WlSurface, Point<i32, Logical>, LayerSurface)> {
    let map = layer_map_for_output(output);
    layers.iter().find_map(|layer| {
        let found = map.layer_under(*layer, point)?;
        let geo = map.layer_geometry(found)?;
        let (surface, loc) =
            found.surface_under(point - geo.loc.to_f64(), WindowSurfaceType::ALL)?;
        Some((surface, geo.loc + loc, found.clone()))
    })
}
