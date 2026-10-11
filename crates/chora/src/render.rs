//! Turns the scene into render elements, and finds the surface under a
//! point.

use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use chora_scene::{InputMode, NodeKind, Placed, Scene, Transform};
use smithay::{
    backend::renderer::{
        ImportAll, ImportMem, Renderer,
        element::{
            AsRenderElements, Kind,
            memory::MemoryRenderBufferRenderElement,
            solid::{SolidColorBuffer, SolidColorRenderElement},
            surface::{WaylandSurfaceRenderElement, render_elements_from_surface_tree},
            utils::RescaleRenderElement,
        },
    },
    desktop::{
        Window, WindowSurfaceType, layer_map_for_output, space::SpaceElement,
        utils::send_frames_surface_tree,
    },
    input::pointer::CursorImageStatus,
    output::Output,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    render_elements,
    utils::{Logical, Point, Rectangle, Scale},
    wayland::shell::wlr_layer::Layer,
};

use crate::{
    layers,
    lock::Lock,
    output::{Spot, output_area, scene_rect},
    state::State,
};

render_elements! {
    pub SceneElement<R> where R: ImportAll + ImportMem;
    Surface = WaylandSurfaceRenderElement<R>,
    Scaled = RescaleRenderElement<WaylandSurfaceRenderElement<R>>,
    Solid = SolidColorRenderElement,
    Cursor = MemoryRenderBufferRenderElement<R>,
}

/// How an output shows the scene: its camera and its scale factor.
pub struct View {
    /// The transform from scene coordinates to logical output coordinates.
    pub camera: Transform,
    /// Physical pixels per logical pixel.
    pub scale: f64,
}

/// The parts of the state that rendering reads. A struct of borrows, so
/// that a backend can borrow its own state at the same time.
pub struct Parts<'a> {
    pub scene: &'a Scene,
    pub windows: &'a HashMap<String, Window>,
    /// One color buffer per rect node.
    pub rects: &'a mut HashMap<String, SolidColorBuffer>,
    pub lock: Option<&'a Lock>,
}

/// The render elements of one output, front first: the layer surfaces above
/// the scene, the scene, and the layer surfaces below it. A locked session
/// shows only the lock surface.
pub fn output_elements<R>(
    parts: &mut Parts<'_>,
    renderer: &mut R,
    output: &Output,
    view: &View,
) -> Vec<SceneElement<R>>
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + Send + 'static,
{
    if let Some(lock) = parts.lock {
        return lock.surface(output).map_or_else(Vec::new, |surface| {
            render_elements_from_surface_tree(
                renderer,
                surface,
                (0, 0),
                view.scale,
                1.0,
                Kind::Unspecified,
            )
            .into_iter()
            .map(SceneElement::Surface)
            .collect()
        });
    }
    let mut out = layer_elements(renderer, output, &layers::ABOVE, view.scale);
    out.extend(elements(parts, renderer, view));
    out.extend(layer_elements(renderer, output, &layers::BELOW, view.scale));
    out
}

/// The layer surfaces of an output on some layers, front first.
fn layer_elements<R>(
    renderer: &mut R,
    output: &Output,
    layers: &[Layer],
    scale: f64,
) -> Vec<SceneElement<R>>
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + Send + 'static,
{
    let map = layer_map_for_output(output);
    let mut out = Vec::new();
    for layer in layers {
        for surface in map.layers_on(*layer).rev() {
            let Some(geo) = map.layer_geometry(surface) else {
                continue;
            };
            let elements = surface.render_elements::<WaylandSurfaceRenderElement<R>>(
                renderer,
                geo.loc.to_physical_precise_round(scale),
                Scale::from(scale),
                1.0,
            );
            out.extend(elements.into_iter().map(SceneElement::Surface));
        }
    }
    out
}

/// The render elements of the scene for one output, front first. The rects
/// keep one buffer per rect node, so that damage tracking sees unchanged
/// rects as unchanged.
fn elements<R>(parts: &mut Parts<'_>, renderer: &mut R, view: &View) -> Vec<SceneElement<R>>
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + Send + 'static,
{
    let list = parts.scene.draw_list();
    let scene = parts.scene;
    parts
        .rects
        .retain(|id, _| scene.node(id).is_some_and(|n| n.kind == NodeKind::Rect));
    let mut out = Vec::new();
    for placed in list.iter().rev() {
        let transform = view.camera.then(&placed.transform);
        let Some((offset, (sx, sy))) = location_and_scale(&transform) else {
            continue;
        };
        // Physical pixels from here on.
        let offset = offset.upscale(view.scale);
        let scale = (sx * view.scale, sy * view.scale);
        #[allow(clippy::cast_possible_truncation)]
        let alpha = placed.opacity as f32;
        match placed.node.kind {
            NodeKind::Surface => {
                let Some(window) = parts.windows.get(&placed.node.id) else {
                    continue;
                };
                // Smithay sizes a surface by the output scale only, so the
                // node scale applies afterwards, around the node position.
                let origin = offset.to_physical_precise_round(1.0);
                let loc = offset - window.geometry().loc.to_f64().upscale(view.scale);
                out.extend(
                    window
                        .render_elements::<WaylandSurfaceRenderElement<R>>(
                            renderer,
                            loc.to_physical_precise_round(1.0),
                            view.scale.into(),
                            alpha,
                        )
                        .into_iter()
                        .map(|e| {
                            SceneElement::Scaled(RescaleRenderElement::from_element(
                                e,
                                origin,
                                (sx, sy),
                            ))
                        }),
                );
            }
            NodeKind::Rect => out.extend(rect(placed, parts.rects, offset, scale, alpha)),
            NodeKind::Group => {}
        }
    }
    out
}

fn rect<R>(
    placed: &Placed<'_>,
    rects: &mut HashMap<String, SolidColorBuffer>,
    offset: Point<f64, Logical>,
    scale: (f64, f64),
    alpha: f32,
) -> Option<SceneElement<R>>
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + Send + 'static,
{
    let props = &placed.node.props;
    let [width, height] = props.size?;
    #[allow(clippy::cast_possible_truncation)]
    let (size, color) = (
        (width.round() as i32, height.round() as i32),
        props.color?.map(|c| c as f32),
    );
    let buffer = rects.entry(placed.node.id.clone()).or_default();
    buffer.update(size, color);
    Some(SceneElement::Solid(SolidColorRenderElement::from_buffer(
        buffer,
        offset.to_physical_precise_round(1.0),
        Scale::from(scale),
        alpha,
        Kind::Unspecified,
    )))
}

/// The translation and the scale of a 2D transform with a positive scale.
/// Other transforms come with the effects milestone.
fn location_and_scale(t: &Transform) -> Option<(Point<f64, Logical>, (f64, f64))> {
    let (sx, sy) = t.scale_2d().filter(|(sx, sy)| *sx > 0.0 && *sy > 0.0)?;
    Some((t.translation_2d().into(), (sx, sy)))
}

/// The position of the window geometry of a surface node in the scene.
pub fn window_origin(state: &State, id: &str) -> Option<Point<i32, Logical>> {
    let list = state.scene.draw_list();
    let placed = list.iter().find(|p| p.node.id == id)?;
    let (x, y) = placed.transform.translation_2d();
    #[allow(clippy::cast_possible_truncation)]
    Some((x.round() as i32, y.round() as i32).into())
}

/// The surface under the pointer: a layer surface above the scene, a
/// surface of the scene, or a layer surface below the scene.
pub fn surface_under(state: &State, spot: &Spot) -> Option<Under> {
    // Layer and lock surfaces sit in output coordinates; this offset moves
    // them into scene coordinates, exactly for cameras without zoom.
    let offset = spot.scene - spot.local;
    if let Some(lock) = &state.lock {
        return lock.surface(&spot.output).map(|surface| Under {
            surface: surface.clone(),
            location: offset,
            node: None,
        });
    }
    let layer = |layers: &[Layer]| {
        layers::surface_under(&spot.output, layers, spot.local).map(|(surface, loc, _)| Under {
            surface,
            location: loc.to_f64() + offset,
            node: None,
        })
    };
    if let Some(under) = layer(&layers::ABOVE) {
        return Some(under);
    }
    match scene_under(state, spot.scene) {
        Target::Surface(under) => Some(under),
        Target::Blocked => None,
        Target::Nothing => layer(&layers::BELOW),
    }
}

/// What a point of the scene hits.
enum Target {
    Surface(Under),
    /// A rect, or a node with `block`, which takes the input and gives it to
    /// nobody.
    Blocked,
    Nothing,
}

/// The surface of the scene under a point. A node with `pass` lets the
/// input through.
fn scene_under(state: &State, pos: Point<f64, Logical>) -> Target {
    let draw_list = state.scene.draw_list();
    let found = draw_list
        .iter()
        .rev()
        .filter(|p| p.input != InputMode::Pass)
        .find_map(|p| Some((p, hit(state, p, pos)?)));
    match found {
        Some((placed, Hit::Surface(surface, location))) if placed.input == InputMode::Accept => {
            Target::Surface(Under {
                surface,
                location,
                node: Some(placed.node.id.clone()),
            })
        }
        Some(_) => Target::Blocked,
        None => Target::Nothing,
    }
}

/// The surface under the pointer.
pub struct Under {
    /// The surface, possibly a subsurface or a popup.
    pub surface: WlSurface,
    /// The position of the surface in the scene.
    pub location: Point<f64, Logical>,
    /// The surface node; `None` for a layer surface.
    pub node: Option<String>,
}

/// What a point hits.
enum Hit {
    /// A surface, and its position in the scene.
    Surface(WlSurface, Point<f64, Logical>),
    /// A rect: an area without a client.
    Area,
}

fn hit(state: &State, placed: &Placed<'_>, pos: Point<f64, Logical>) -> Option<Hit> {
    let (offset, (sx, sy)) = location_and_scale(&placed.transform)?;
    let local = Point::<f64, Logical>::from(((pos.x - offset.x) / sx, (pos.y - offset.y) / sy));
    match placed.node.kind {
        NodeKind::Surface => {
            let window = state.windows.get(&placed.node.id)?;
            let geo = window.geometry().loc.to_f64();
            let (surface, loc) = window.surface_under(local + geo, WindowSurfaceType::ALL)?;
            let loc = loc.to_f64() - geo;
            Some(Hit::Surface(
                surface,
                offset + Point::from((loc.x * sx, loc.y * sy)),
            ))
        }
        NodeKind::Rect => {
            let [w, h] = placed.node.props.size.unwrap_or_default();
            ((0.0..w).contains(&local.x) && (0.0..h).contains(&local.y)).then_some(Hit::Area)
        }
        NodeKind::Group => None,
    }
}

/// Sends frame callbacks for one output: every frame to the toplevels that
/// it shows, once per second to the others, so that hidden clients do not
/// draw for nothing. Taskbars learn the toplevels on the output too.
pub fn send_frames(state: &mut State, output: &Output) {
    let visible = shown_surfaces(state, output);
    let time = state.start_time.elapsed();
    for (id, window) in &state.windows {
        let on_output = visible.contains(id);
        if on_output {
            window.output_enter(output, Rectangle::from_size(window.bbox().size));
        } else {
            window.output_leave(output);
        }
        let throttle = if on_output {
            Duration::ZERO
        } else {
            Duration::from_secs(1)
        };
        window.send_frame(output, time, Some(throttle), |_, _| Some(output.clone()));
    }
    for layer in layer_map_for_output(output).layers() {
        layer.send_frame(output, time, Some(Duration::ZERO), |_, _| {
            Some(output.clone())
        });
    }
    if let CursorImageStatus::Surface(surface) = &state.cursor_status {
        send_frames_surface_tree(surface, output, time, Some(Duration::ZERO), |_, _| {
            Some(output.clone())
        });
    }
    if let Some(surface) = state.lock.as_ref().and_then(|l| l.surface(output)) {
        send_frames_surface_tree(surface, output, time, Some(Duration::ZERO), |_, _| {
            Some(output.clone())
        });
    }
    state
        .foreign_toplevel_manager_state
        .set_shown(output, &visible);
}

/// The surface nodes that an output shows: those whose scene rectangle
/// overlaps the scene area of the output.
fn shown_surfaces(state: &State, output: &Output) -> HashSet<String> {
    let area = output_area(output);
    let shown = scene_rect(&state.camera(output), Rectangle::from_size(area.size));
    state
        .scene
        .draw_list()
        .iter()
        .filter(|p| p.node.kind == NodeKind::Surface)
        .filter(|p| {
            let window = state.windows.get(&p.node.id);
            window.is_some_and(|w| shown.overlaps(window_rect(w, &p.transform)))
        })
        .map(|p| p.node.id.clone())
        .collect()
}

/// The scene rectangle of a window with its subsurfaces and popups. The
/// transform places its geometry origin and scales it.
fn window_rect(window: &Window, transform: &Transform) -> Rectangle<f64, Logical> {
    let (x, y) = transform.translation_2d();
    let (sx, sy) = transform.scale_2d().unwrap_or((1.0, 1.0));
    let bbox = window.bbox().to_f64();
    let geo = window.geometry().loc.to_f64();
    Rectangle::new(
        (x + (bbox.loc.x - geo.x) * sx, y + (bbox.loc.y - geo.y) * sy).into(),
        (bbox.size.w * sx, bbox.size.h * sy).into(),
    )
}

#[cfg(test)]
mod tests {
    use chora_scene::Transform;

    use super::location_and_scale;

    #[test]
    fn mirrored_and_collapsed_nodes_are_not_drawn() {
        assert!(location_and_scale(&Transform::scale(-1.0, 1.0, 1.0)).is_none());
        assert!(location_and_scale(&Transform::scale(1.0, 0.0, 1.0)).is_none());
        assert!(location_and_scale(&Transform::rotate_z(30.0)).is_none());
        assert!(location_and_scale(&Transform::scale(2.0, 2.0, 1.0)).is_some());
    }
}
