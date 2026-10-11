//! Protocols that need little policy: idle notification and inhibition,
//! text input and input methods, fractional scale, cursor shapes, the
//! foreign toplevel list, activation requests and the requests of
//! toplevels.
//!
//! Requests that change a window, for example fullscreen or activation, go
//! to plugins as `surface.requested`; the core changes nothing by itself.

use chora_scene::{Event, SurfaceRequest};
use smithay::{
    desktop::{PopupKind, PopupManager, Window, layer_map_for_output},
    input::tablet::TabletSeatHandler,
    reexports::wayland_server::{Resource, protocol::wl_surface::WlSurface},
    utils::{Logical, Rectangle},
    wayland::{
        compositor::{SurfaceData, with_states},
        foreign_toplevel_list::{ForeignToplevelListHandler, ForeignToplevelListState},
        fractional_scale::{FractionalScaleHandler, with_fractional_scale},
        idle_inhibit::IdleInhibitHandler,
        idle_notify::{IdleNotifierHandler, IdleNotifierState},
        input_method::{InputMethodHandler, PopupSurface as ImePopupSurface},
        seat::WaylandFocus,
        shell::xdg::ToplevelSurface,
        text_input::TextInputActivation,
        xdg_activation::{
            XdgActivationHandler, XdgActivationState, XdgActivationToken, XdgActivationTokenData,
        },
    },
};

use crate::state::State;

impl State {
    /// Tells plugins what a client asks for. A request before the first
    /// commit waits for `surface.created`.
    pub fn request(&mut self, toplevel: &ToplevelSurface, request: SurfaceRequest) {
        let Some(surface) = self.surface_id(toplevel.wl_surface()) else {
            return;
        };
        if self.scene.node(&surface).is_some() {
            self.emit(&Event::SurfaceRequested { surface, request });
        } else {
            self.early_requests
                .entry(surface)
                .or_default()
                .push(request);
        }
    }

    /// Sends the requests that a client made before `surface.created`.
    pub fn send_early_requests(&mut self, surface: &str) {
        for request in self.early_requests.remove(surface).unwrap_or_default() {
            self.emit(&Event::SurfaceRequested {
                surface: surface.into(),
                request,
            });
        }
    }

    /// Answers a request that the protocol wants a configure for. Before
    /// the initial configure, the first commit sends it.
    pub fn answer(toplevel: &ToplevelSurface) {
        if toplevel.is_initial_configure_sent() {
            toplevel.send_configure();
        }
    }

    /// Updates the inhibition of idleness: any live surface that asks for
    /// it inhibits, for example a video player.
    fn update_idle_inhibit(&mut self) {
        self.idle_inhibitors.retain(Resource::is_alive);
        let inhibited = !self.idle_inhibitors.is_empty();
        self.idle_notifier_state.set_is_inhibited(inhibited);
    }

    /// Adds a toplevel to the foreign toplevel list and to
    /// `wlr-foreign-toplevel-management`, for taskbars.
    pub fn list_toplevel(&mut self, id: &str) {
        let info = self.surface_info(id);
        let handle = self.foreign_toplevel_list_state.new_toplevel::<Self>(
            info.title.clone().unwrap_or_default(),
            info.app_id.clone().unwrap_or_default(),
        );
        self.foreign_toplevels.insert(id.into(), handle);
        let states = self.taskbar_states(id);
        self.foreign_toplevel_manager_state.add(id, info, states);
    }

    /// Tells taskbars a new title or application id.
    pub fn relist_toplevel(&mut self, id: &str) {
        let info = self.surface_info(id);
        if let Some(handle) = self.foreign_toplevels.get(id) {
            handle.send_title(info.title.as_deref().unwrap_or_default());
            handle.send_app_id(info.app_id.as_deref().unwrap_or_default());
            handle.send_done();
        }
        self.foreign_toplevel_manager_state.set_info(id, info);
    }

    /// Removes a destroyed toplevel from the lists of the taskbars.
    pub fn unlist_toplevel(&mut self, id: &str) {
        if let Some(handle) = self.foreign_toplevels.remove(id) {
            self.foreign_toplevel_list_state.remove_toplevel(&handle);
        }
        self.foreign_toplevel_manager_state.remove(id);
    }
}

impl ForeignToplevelListHandler for State {
    fn foreign_toplevel_list_state(&mut self) -> &mut ForeignToplevelListState {
        &mut self.foreign_toplevel_list_state
    }
}

impl IdleNotifierHandler for State {
    fn idle_notifier_state(&mut self) -> &mut IdleNotifierState<Self> {
        &mut self.idle_notifier_state
    }
}

impl IdleInhibitHandler for State {
    fn inhibit(&mut self, surface: WlSurface) {
        self.idle_inhibitors.push(surface);
        self.update_idle_inhibit();
    }

    fn uninhibit(&mut self, surface: WlSurface) {
        self.idle_inhibitors.retain(|s| *s != surface);
        self.update_idle_inhibit();
    }
}

impl TextInputActivation for State {}

/// Cursor shapes need it; tablets come later.
impl TabletSeatHandler for State {
    type ToolFocus = WlSurface;
}

impl InputMethodHandler for State {
    fn new_popup(&mut self, surface: ImePopupSurface) {
        if let Err(err) = self.popups.track_popup(PopupKind::from(surface)) {
            tracing::warn!(?err, "an input method popup failed");
        }
    }

    fn dismiss_popup(&mut self, surface: ImePopupSurface) {
        if let Some(parent) = surface.get_parent().map(|p| p.surface.clone()) {
            let _ = PopupManager::dismiss_popup(&parent, &PopupKind::from(surface));
        }
    }

    fn popup_repositioned(&mut self, _surface: ImePopupSurface) {}

    fn parent_geometry(&self, parent: &WlSurface) -> Rectangle<i32, Logical> {
        self.windows
            .values()
            .find(|w| w.wl_surface().as_deref() == Some(parent))
            .map(Window::geometry)
            .unwrap_or_default()
    }
}

impl FractionalScaleHandler for State {
    /// The preferred scale of a surface is the scale of the first output.
    fn new_fractional_scale(&mut self, surface: WlSurface) {
        with_states(&surface, |states| prefer_scale(states, self.first_scale()));
    }
}

impl State {
    /// The scale of the first output, or 1.
    fn first_scale(&self) -> f64 {
        self.outputs
            .first()
            .map_or(1.0, |o| o.current_scale().fractional_scale())
    }

    /// Tells the surfaces their preferred scale after a scale changed: the
    /// scale of the first output to toplevels, and the scale of their output
    /// to layer surfaces.
    pub fn update_preferred_scales(&self) {
        let scale = self.first_scale();
        for window in self.windows.values() {
            window.with_surfaces(|_, states| prefer_scale(states, scale));
        }
        for output in &self.outputs {
            let scale = output.current_scale().fractional_scale();
            for layer in layer_map_for_output(output).layers() {
                layer.with_surfaces(|_, states| prefer_scale(states, scale));
            }
        }
    }
}

/// Sets the preferred fractional scale of a surface. The client hears of
/// a change only.
fn prefer_scale(states: &SurfaceData, scale: f64) {
    with_fractional_scale(states, |fractional| {
        fractional.set_preferred_scale(scale);
    });
}

impl XdgActivationHandler for State {
    fn activation_state(&mut self) -> &mut XdgActivationState {
        &mut self.activation_state
    }

    /// Passes the request to plugins; a token works once.
    fn request_activation(
        &mut self,
        token: XdgActivationToken,
        _token_data: XdgActivationTokenData,
        surface: WlSurface,
    ) {
        self.activation_state.remove_token(&token);
        let toplevel = self
            .windows
            .values()
            .filter_map(Window::toplevel)
            .find(|t| *t.wl_surface() == surface)
            .cloned();
        if let Some(toplevel) = toplevel {
            self.request(&toplevel, SurfaceRequest::Activate);
        }
    }
}
