//! Turns the scene into render elements, and finds the surface under a
//! point.

use std::collections::HashMap;

use chora_scene::{InputMode, NodeKind, Placed, Scene, Transform};
use smithay::{
    backend::renderer::{
        ImportAll, ImportMem, Renderer,
        element::{
            AsRenderElements, Kind,
            solid::{SolidColorBuffer, SolidColorRenderElement},
            surface::WaylandSurfaceRenderElement,
        },
    },
    desktop::{Window, WindowSurfaceType},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    render_elements,
    utils::{Logical, Point, Scale},
};

use crate::state::State;

render_elements! {
    pub SceneElement<R> where R: ImportAll + ImportMem;
    Surface = WaylandSurfaceRenderElement<R>,
    Solid = SolidColorRenderElement,
}

/// The render elements of the scene, front first. `rects` keeps one buffer
/// per rect node, so that damage tracking sees unchanged rects as unchanged.
pub fn elements<R>(
    scene: &Scene,
    windows: &HashMap<String, Window>,
    rects: &mut HashMap<String, SolidColorBuffer>,
    renderer: &mut R,
) -> Vec<SceneElement<R>>
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + 'static,
{
    let list = scene.draw_list();
    rects.retain(|id, _| scene.node(id).is_some_and(|n| n.kind == NodeKind::Rect));
    let mut out = Vec::new();
    for placed in list.iter().rev() {
        let Some((offset, scale)) = location_and_scale(&placed.transform) else {
            continue;
        };
        #[allow(clippy::cast_possible_truncation)]
        let alpha = placed.opacity as f32;
        match placed.node.kind {
            NodeKind::Surface => {
                let Some(window) = windows.get(&placed.node.id) else {
                    continue;
                };
                let loc = offset - window.geometry().loc.to_f64().upscale(scale);
                out.extend(
                    window
                        .render_elements::<WaylandSurfaceRenderElement<R>>(
                            renderer,
                            loc.to_physical_precise_round(1.0),
                            scale.into(),
                            alpha,
                        )
                        .into_iter()
                        .map(SceneElement::Surface),
                );
            }
            NodeKind::Rect => out.extend(rect(placed, rects, offset, scale, alpha)),
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
    R::TextureId: Clone + 'static,
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

/// The surface under a point of the scene and the position of that surface
/// in the scene. A node with `block` that the point hits takes the input and
/// gives it to nobody; a node with `pass` lets it through.
pub fn surface_under(state: &State, pos: Point<f64, Logical>) -> Option<Under> {
    let draw_list = state.scene.draw_list();
    let (placed, hit) = draw_list
        .iter()
        .rev()
        .filter(|p| p.input != InputMode::Pass)
        .find_map(|p| Some((p, hit(state, p, pos)?)))?;
    match (placed.input, hit) {
        (InputMode::Accept, Hit::Surface(surface, location)) => Some(Under {
            surface,
            location,
            node: placed.node.id.clone(),
        }),
        _ => None,
    }
}

/// The surface under the pointer.
pub struct Under {
    /// The surface, possibly a subsurface or a popup.
    pub surface: WlSurface,
    /// The position of the surface in the scene.
    pub location: Point<f64, Logical>,
    /// The surface node.
    pub node: String,
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
