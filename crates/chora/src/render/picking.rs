//! Finding the surface or node under a point of an output.

use super::{
    InputMode, Layer, Logical, Node, NodeKind, Output, Placed, Placing, Point, ROOT, Scene, Spot,
    State, Transform, WindowSurfaceType, WlSurface, content_size, flat, layers, location_and_scale,
    mesh, node_size, overlay_id, space,
};

/// The position of the window geometry of a surface node in the scene.
pub fn window_origin(state: &State, id: &str) -> Option<Point<i32, Logical>> {
    let list = state.scene.draw_list();
    let placed = list.iter().find(|p| p.node.id == id)?;
    let (x, y) = placed.transform.translation_2d();
    #[allow(clippy::cast_possible_truncation)]
    Some((x.round() as i32, y.round() as i32).into())
}

/// Whether the transform of a node in the scene is not a 2D translation
/// with a positive scale, so that a plane carries it.
pub fn under_plane(state: &State, id: &str) -> bool {
    let list = state.scene.draw_list();
    list.iter()
        .find(|p| p.node.id == id)
        .is_some_and(|p| !p.transform.is_plain())
}

/// The surface under the pointer: a layer surface above the scene, a
/// surface of the scene, or a layer surface below the scene.
/// Layer and lock surfaces sit in output coordinates; this offset moves
/// them into scene coordinates, exactly for cameras without zoom.
/// The overlay of the output sits in its coordinates, as layer surfaces.
pub fn surface_under(state: &State, spot: &Spot) -> Option<Under> {
    let offset = spot.scene - spot.local;
    if let Some(lock) = &state.lock {
        return lock.surface(&spot.output).map(|surface| Under {
            surface: surface.clone(),
            location: offset,
            node: None,
            plane: None,
        });
    }
    let layer = |layers: &[Layer]| {
        layers::surface_under(&spot.output, layers, spot.local).map(|(surface, loc, _)| Under {
            surface,
            location: loc.to_f64() + offset,
            node: None,
            plane: None,
        })
    };
    if let Some(under) = layer(&layers::ABOVE) {
        return Some(under);
    }
    let overlay = state.scene.hit_list(&overlay_id(&spot.output.name()));
    match scene_under(state, spot, &overlay, spot.local) {
        Target::Surface(under) => {
            return Some(Under {
                location: under.location + offset,
                ..under
            });
        }
        Target::Blocked => return None,
        Target::Nothing => {}
    }
    match scene_under(state, spot, &state.scene.hit_list(ROOT), spot.scene) {
        Target::Surface(under) => Some(under),
        Target::Blocked => None,
        Target::Nothing => layer(&layers::BELOW),
    }
}

/// The node of the scene under the pointer that takes pointer input: a
/// surface, a rect or an image. `None` over the lock screen, over a layer
/// surface above the scene, and over nothing.
pub fn node_under(state: &State, spot: &Spot) -> Option<String> {
    node_hit(state, spot).map(|(node, _)| node)
}

/// As [`node_under`], with the point in the coordinates of the node: the
/// point that its plane maps to the pointer.
/// The overlay of the output first, in its coordinates (D43).
pub fn node_hit(state: &State, spot: &Spot) -> Option<(String, Point<f64, Logical>)> {
    if state.lock.is_some()
        || layers::surface_under(&spot.output, &layers::ABOVE, spot.local).is_some()
    {
        return None;
    }
    let overlay = state.scene.hit_list(&overlay_id(&spot.output.name()));
    let scene = state.scene.hit_list(ROOT);
    let found = [(&overlay, spot.local), (&scene, spot.scene)]
        .into_iter()
        .find_map(|(list, pos)| found_shown(state, spot, list, pos))?;
    (found.input == InputMode::Accept).then(|| (found.node.id.clone(), found.local))
}

/// What a point of the scene hits.
pub(super) enum Target {
    Surface(Under),
    /// A rect, or a node with `block`, which takes the input and gives it to
    /// nobody.
    Blocked,
    Nothing,
}

/// The surface of the scene under a point. A node with `pass` lets the
/// input through.
pub(super) fn scene_under(
    state: &State,
    spot: &Spot,
    hit_list: &[Placed<'_>],
    pos: Point<f64, Logical>,
) -> Target {
    match found_shown(state, spot, hit_list, pos) {
        Some(Found {
            node,
            input: InputMode::Accept,
            hit: Hit::Surface(surface, on_surface),
            local,
            planar,
        }) => Target::Surface(Under {
            surface,
            location: pos - on_surface,
            node: Some(node.id.clone()),
            plane: (planar || spot.via.is_some()).then(|| local - on_surface),
        }),
        Some(_) => Target::Blocked,
        None => Target::Nothing,
    }
}

/// The surface under the pointer.
pub struct Under {
    /// The surface, possibly a subsurface or a popup.
    pub surface: WlSurface,
    /// The point of the scene minus the point on the surface in its own
    /// units, so that the point minus `location` is the position on the
    /// surface that its client gets. It is the origin of the surface in the
    /// scene only on a node without scale.
    pub location: Point<f64, Logical>,
    /// The surface node; `None` for a layer surface.
    pub node: Option<String>,
    /// The origin of the surface in the coordinates of its node, when a
    /// plane carries the node or a pane forwards the pointer to it; a
    /// press then maps every motion through that plane or pane.
    pub plane: Option<Point<f64, Logical>>,
}

/// What a point hits.
pub(super) enum Hit {
    /// A surface, and the point on it in its own units.
    Surface(WlSurface, Point<f64, Logical>),
    /// A rect: an area without a client.
    Area,
}

/// The top node of a list under a point, with what it hit.
pub(super) struct Found<'a> {
    node: &'a Node,
    /// The input mode of the node after those of its ancestors.
    input: InputMode,
    hit: Hit,
    /// The point in the coordinates of the node.
    local: Point<f64, Logical>,
    /// Whether the node sits under a plane.
    planar: bool,
}

/// The first node of a list of an output, from the top, that a point
/// hits, as [`found`] finds it. A node that shows nothing because the
/// layer budget left it, or a layer, plane or piece that holds it, out
/// of the last frame of that output takes the input where it would take
/// it, and gives it to nobody: the one place of that rule for every
/// layer, plane, mesh and piece. The keyboard stays where it is; the
/// owner of the node learns of it from `layers.dropped`.
pub(super) fn found_shown<'a>(
    state: &'a State,
    spot: &Spot,
    list: &[Placed<'a>],
    pos: Point<f64, Logical>,
) -> Option<Found<'a>> {
    let found = found(state, list, InputMode::Accept, pos)?;
    Some(if left_out(state, &spot.output, &found.node.id) {
        Found {
            input: InputMode::Block,
            hit: Hit::Area,
            ..found
        }
    } else {
        found
    })
}

/// Whether a node shows nothing on an output because the layer budget
/// left it, or a layer, plane or piece that holds it, out of the last
/// frame of that output.
pub fn left_out(state: &State, output: &Output, id: &str) -> bool {
    state
        .assets
        .frame_dark
        .get(&output.name())
        .is_some_and(|ids| ids.iter().any(|out| state.scene.is_within(id, out)))
}

/// The first node of a list, from the top, that a point hits. `outer` is
/// the input mode above the list: a mode other than `accept` rules all of
/// it. A node with `pass` lets the point through.
pub(super) fn found<'a>(
    state: &'a State,
    list: &[Placed<'a>],
    outer: InputMode,
    pos: Point<f64, Logical>,
) -> Option<Found<'a>> {
    list.iter().rev().find_map(|placed| {
        let input = if outer == InputMode::Accept {
            placed.input
        } else {
            outer
        };
        (input != InputMode::Pass)
            .then(|| hit(state, placed, input, pos))
            .flatten()
    })
}

pub(super) fn hit<'a>(
    state: &'a State,
    placed: &Placed<'a>,
    input: InputMode,
    pos: Point<f64, Logical>,
) -> Option<Found<'a>> {
    hit_cut(state, placed, input, (pos, mesh::NEAR))
}

/// As [`hit`], for a node that its list cuts at `near`: a 3D context
/// cuts at [`space::NEAR_W`].
/// The point on the surface in its own units: Smithay gives the
/// client `pos - location`, which a scaled node must not scale.
/// A group is empty and text takes no pointer input.
pub(super) fn hit_cut<'a>(
    state: &'a State,
    placed: &Placed<'a>,
    input: InputMode,
    (pos, near): (Point<f64, Logical>, f64),
) -> Option<Found<'a>> {
    if !placed
        .clips
        .iter()
        .all(|(transform, size)| inside(transform, *size, pos))
    {
        return None;
    }
    if placed.node.is_context() && placed.transform.is_plain() {
        return context_hit(state, placed, input, pos);
    }
    let Some((offset, (sx, sy))) = location_and_scale(&placed.transform) else {
        return plane_hit(state, placed, input, (pos, near));
    };
    let local = Point::<f64, Logical>::from(((pos.x - offset.x) / sx, (pos.y - offset.y) / sy));
    let hit = match placed.node.kind {
        NodeKind::Surface => {
            let window = state.windows.get(&placed.node.id)?;
            let geo = window.geometry().loc.to_f64();
            let (surface, loc) = window.surface_under(local + geo, WindowSurfaceType::ALL)?;
            Hit::Surface(surface, local - (loc.to_f64() - geo))
        }
        NodeKind::Rect | NodeKind::Image | NodeKind::Shader => {
            let [w, h] = node_size(state, placed.node);
            if !((0.0..w).contains(&local.x) && (0.0..h).contains(&local.y)) {
                return None;
            }
            Hit::Area
        }
        NodeKind::Group | NodeKind::Text => return None,
    };
    Some(Found {
        node: placed.node,
        input,
        hit,
        local,
        planar: false,
    })
}

/// What a point hits in a 3D context: the ray from the eye through the
/// point takes the nearest piece of a plane that shows there and accepts
/// the input, in the order that the context draws them. The empty area of
/// the context lets the point through.
pub(super) fn context_hit<'a>(
    state: &'a State,
    context: &Placed<'a>,
    outer: InputMode,
    pos: Point<f64, Logical>,
) -> Option<Found<'a>> {
    let list = state.scene.context_list(context);
    let (windows, assets) = (&state.windows, &state.assets);
    let computed = &assets.stats.orders;
    let pieces = assets
        .orders
        .get(computed, context, &list, |p| flat(windows, assets, p));
    pieces.iter().rev().find_map(|piece| {
        let entry = &list[piece.plane];
        let input = if outer == InputMode::Accept {
            entry.input
        } else {
            outer
        };
        if input == InputMode::Pass {
            return None;
        }
        if let Some(part) = &piece.part
            && !space::contains(&part.shown, [pos.x, pos.y])
        {
            return None;
        }
        entry_hit(state, entry, input, pos)
    })
}

/// What a point hits of an entry of a context. A layer that draws flat
/// in the context, as a group with a clip does, holds its descendants:
/// the hit test walks its own list.
pub(super) fn entry_hit<'a>(
    state: &'a State,
    entry: &Placed<'a>,
    input: InputMode,
    pos: Point<f64, Logical>,
) -> Option<Found<'a>> {
    if !(entry.node.is_layer() && entry.transform.is_plain()) || entry.node.is_context() {
        return hit_cut(state, entry, input, (pos, space::NEAR_W));
    }
    if !entry
        .clips
        .iter()
        .all(|(transform, size)| inside(transform, *size, pos))
    {
        return None;
    }
    let local = node_point(&entry.transform, pos)?;
    let list = state.scene.hit_list(&entry.node.id);
    found(state, &list, input, local)
}

/// The largest coordinate of a point that the inverse of a plane gives; a
/// plane maps a point near its horizon farther.
pub(super) const MAX_LOCAL: f64 = 1e6;

/// A point of a node with its coordinates cut to `MAX_LOCAL`.
pub(super) fn clamp_local(point: Point<f64, Logical>) -> Point<f64, Logical> {
    let cut = |v: f64| v.clamp(-MAX_LOCAL, MAX_LOCAL);
    (cut(point.x), cut(point.y)).into()
}

/// What a point hits of a node under a transform that is not a 2D
/// translation with a positive scale. The inverse of the homography of
/// the plane maps the point into the node; a point that the plane shows
/// behind the eye or outside its content hits nothing. A group is a
/// layer: its own list answers inside its size. A node with a mesh takes
/// the input over the bounding box of its shape, at the point of the flat
/// quad of its size, as a drag of it tells; where that quad shows no
/// point, at the point of the screen.
pub(super) fn plane_hit<'a>(
    state: &'a State,
    placed: &Placed<'a>,
    input: InputMode,
    (pos, near): (Point<f64, Logical>, f64),
) -> Option<Found<'a>> {
    let node = placed.node;
    let take = |hit, local| Found {
        node,
        input,
        hit,
        local,
        planar: true,
    };
    if node.props.mesh.is_some() {
        let local = plane_point(&placed.transform, pos).unwrap_or(pos);
        return layer_hit(placed, near, pos).map(|hit| take(hit, local));
    }
    let [w, h] = content_size(&state.windows, &state.assets, node)?;
    let local = plane_point(&placed.transform, pos)?;
    if !((0.0..w).contains(&local.x) && (0.0..h).contains(&local.y)) {
        return None;
    }
    match node.kind {
        NodeKind::Surface => {
            let window = state.windows.get(&node.id)?;
            let geo = window.geometry().loc.to_f64();
            let (surface, loc) = window.surface_under(local + geo, WindowSurfaceType::ALL)?;
            let on_surface = local - (loc.to_f64() - geo);
            Some(take(Hit::Surface(surface, on_surface), local))
        }
        NodeKind::Group => {
            let list = state.scene.hit_list(&node.id);
            let inner = if node.is_context() {
                context_hit(state, list.first()?, input, local)
            } else {
                found(state, &list, input, local)
            };
            inner.map(|inner| Found {
                planar: true,
                ..inner
            })
        }
        NodeKind::Rect | NodeKind::Image | NodeKind::Shader => Some(take(Hit::Area, local)),
        NodeKind::Text => None,
    }
}

/// The point of the plane z = 0 that a transform shows at a point of the
/// screen: the inverse of its homography, each coordinate cut to
/// `MAX_LOCAL` near the horizon of the plane. `None` when the plane is
/// edge on, or shows the point behind the eye.
pub(crate) fn plane_point(
    transform: &Transform,
    pos: Point<f64, Logical>,
) -> Option<Point<f64, Logical>> {
    let inverse = transform.plane_inverse()?;
    let scale = inverse[2] * pos.x + inverse[5] * pos.y + inverse[8];
    let u = (inverse[0] * pos.x + inverse[3] * pos.y + inverse[6]) / scale;
    let v = (inverse[1] * pos.x + inverse[4] * pos.y + inverse[7]) / scale;
    let matrix = transform.matrix();
    let depth = matrix[3] * u + matrix[7] * v + matrix[15];
    (u.is_finite() && v.is_finite() && depth > mesh::NEAR).then(|| clamp_local((u, v).into()))
}

/// The screen point that a transform shows for a point of the plane
/// z = 0. `None` when the point lies at or behind the eye.
pub(crate) fn project_point(transform: &Transform, x: f64, y: f64) -> Option<(f64, f64)> {
    let m = transform.matrix();
    let w = m[3] * x + m[7] * y + m[15];
    (w > mesh::NEAR).then(|| {
        (
            (m[0] * x + m[4] * y + m[12]) / w,
            (m[1] * x + m[5] * y + m[13]) / w,
        )
    })
}

/// The point of a node's own coordinates that a transform shows at a
/// point of the screen, without any test of the inside.
pub(super) fn node_point(
    transform: &Transform,
    pos: Point<f64, Logical>,
) -> Option<Point<f64, Logical>> {
    match location_and_scale(transform) {
        Some((offset, (sx, sy))) => Some(((pos.x - offset.x) / sx, (pos.y - offset.y) / sy).into()),
        None => plane_point(transform, pos),
    }
}

/// The point of a node, in its own coordinates, that a point of the list
/// of `start` (the root, an overlay root or a plane) shows, also outside
/// the node: a press that drags a node maps every motion this way, so that
/// the node needs no hit. `None` when a plane on the way shows the point
/// behind the eye, or the node is not under `start`.
pub fn local_in(
    scene: &Scene,
    start: &str,
    pos: Point<f64, Logical>,
    id: &str,
) -> Option<Point<f64, Logical>> {
    local_in_list(scene, &scene.hit_list(start), pos, id)
}

/// As [`local_in`], with the list that holds the node or an ancestor of
/// it: a plane, a layer or a 3D context.
pub(super) fn local_in_list(
    scene: &Scene,
    list: &[Placed<'_>],
    pos: Point<f64, Logical>,
    id: &str,
) -> Option<Point<f64, Logical>> {
    if let Some(placed) = list.iter().find(|p| p.node.id == id) {
        return node_point(&placed.transform, pos);
    }
    let holder = list.iter().find(|p| {
        (is_plane(p) || p.node.is_context() || p.node.is_layer())
            && p.node.id != id
            && scene.is_within(id, &p.node.id)
    })?;
    if holder.node.is_context() && !is_plane(holder) {
        return local_in_list(scene, &scene.context_list(holder), pos, id);
    }
    let inner = scene.hit_list(&holder.node.id);
    let at = node_point(&holder.transform, pos)?;
    local_in_list(scene, inner.get(1..)?, at, id)
}

/// Whether a listed node is a plane: it has a size and its transform is
/// not a 2D translation with a positive scale.
pub(super) fn is_plane(placed: &Placed<'_>) -> bool {
    placed.node.props.size.is_some() && !placed.transform.is_plain()
}

/// Whether a point of the scene is inside the clip of a node with a
/// transform and a size.
pub(super) fn inside(transform: &Transform, [w, h]: [f64; 2], pos: Point<f64, Logical>) -> bool {
    node_point(transform, pos).is_some_and(|p| (0.0..w).contains(&p.x) && (0.0..h).contains(&p.y))
}

/// What a point hits of a node with a mesh under a transform that is not
/// a plain 2D one: the triangles of its shape take the input where they
/// draw, cut at `near` as the draw cuts them.
pub(super) fn layer_hit(placed: &Placed<'_>, near: f64, pos: Point<f64, Logical>) -> Option<Hit> {
    if !placed.node.is_layer() {
        return None;
    }
    let at = Placing {
        transform: &placed.transform,
        scale: 1.0,
        near,
    };
    mesh::shape(placed.node, &at)?
        .covers([pos.x, pos.y])
        .then_some(Hit::Area)
}
