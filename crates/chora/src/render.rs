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
    utils::{Logical, Physical, Point, Scale},
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
                            to_physical(loc),
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
        to_physical(offset),
        Scale::from(scale),
        alpha,
        Kind::Unspecified,
    )))
}

/// The translation and the scale of a 2D transform. Other transforms come
/// with the effects milestone.
fn location_and_scale(t: &Transform) -> Option<(Point<f64, Logical>, (f64, f64))> {
    let scale = t.scale_2d()?;
    Some((t.translation_2d().into(), scale))
}

#[allow(clippy::cast_possible_truncation)]
fn to_physical(p: Point<f64, Logical>) -> Point<i32, Physical> {
    (p.x.round() as i32, p.y.round() as i32).into()
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
/// in the scene.
pub fn surface_under(
    state: &State,
    pos: Point<f64, Logical>,
) -> Option<(WlSurface, Point<f64, Logical>)> {
    for placed in state.scene.draw_list().iter().rev() {
        match placed.input {
            InputMode::Pass => continue,
            InputMode::Block => return None,
            InputMode::Accept => {}
        }
        let (offset, (sx, sy)) = location_and_scale(&placed.transform)?;
        let local = Point::<f64, Logical>::from(((pos.x - offset.x) / sx, (pos.y - offset.y) / sy));
        match placed.node.kind {
            NodeKind::Surface => {
                let Some(window) = state.windows.get(&placed.node.id) else {
                    continue;
                };
                let geo = window.geometry().loc.to_f64();
                if let Some((surface, loc)) =
                    window.surface_under(local + geo, WindowSurfaceType::ALL)
                {
                    let loc = loc.to_f64() - geo;
                    return Some((surface, offset + Point::from((loc.x * sx, loc.y * sy))));
                }
            }
            NodeKind::Rect => {
                let [w, h] = placed.node.props.size.unwrap_or_default();
                if (0.0..w).contains(&local.x) && (0.0..h).contains(&local.y) {
                    return None;
                }
            }
            NodeKind::Group => {}
        }
    }
    None
}
