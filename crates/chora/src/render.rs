//! Turns the scene into render elements, and finds the surface under a
//! point.

use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use chora_scene::{InputMode, NodeKind, Placed, Scene, Transform};
use smithay::{
    backend::renderer::{
        element::{
            AsRenderElements, Id, Kind,
            memory::MemoryRenderBufferRenderElement,
            solid::{SolidColorBuffer, SolidColorRenderElement},
            surface::{WaylandSurfaceRenderElement, render_elements_from_surface_tree},
            utils::RescaleRenderElement,
        },
        gles::{GlesRenderer, GlesTexture},
    },
    desktop::{
        Window, WindowSurfaceType, layer_map_for_output,
        space::SpaceElement,
        utils::{
            OutputPresentationFeedback, send_frames_surface_tree,
            take_presentation_feedback_surface_tree,
        },
    },
    input::pointer::CursorImageStatus,
    output::Output,
    reexports::{
        wayland_protocols::wp::presentation_time::server::wp_presentation_feedback,
        wayland_server::protocol::wl_surface::WlSurface,
    },
    render_elements,
    utils::{Logical, Point, Rectangle, Scale},
    wayland::{compositor::SurfaceData, shell::wlr_layer::Layer},
};

use crate::{
    images::{self, CellElement, Images, Placement},
    layers,
    lock::Lock,
    mesh::{self, Layers, MeshElement, MeshRenderer, Placing},
    output::{Spot, output_area, scene_rect},
    state::State,
};

render_elements! {
    pub SceneElement<R> where R: MeshRenderer;
    Surface = WaylandSurfaceRenderElement<R>,
    Scaled = RescaleRenderElement<WaylandSurfaceRenderElement<R>>,
    Solid = SolidColorRenderElement,
    Cursor = MemoryRenderBufferRenderElement<R>,
    Image = CellElement<R>,
    Mesh = MeshElement,
}

/// What the scene draws besides surfaces: a color buffer per rect node,
/// the images, the ids of the image cells of each image node, and the
/// textures of the layers.
#[derive(Default)]
pub struct Assets {
    pub rects: HashMap<String, SolidColorBuffer>,
    pub images: Images,
    /// The version of the image and the ids of the cells by node.
    pub cells: HashMap<String, (Option<Id>, Vec<Id>)>,
    pub layers: Layers,
    /// Errors of effect shaders since the last report.
    pub effect_errors: Vec<String>,
    /// The time of each shader file that compiled in a check.
    pub checked_effects: HashMap<String, Option<std::time::SystemTime>>,
    /// The nodes that the layer budget left out of the last frame of each
    /// output, as `layers.dropped` told them (#17, #24).
    pub left_out: HashMap<String, Vec<String>>,
    /// The nodes that the last render of each output left out, before the
    /// frame shows.
    pub frame_left_out: HashMap<String, Vec<String>>,
}

/// How an output shows the scene: its camera, its scale factor and its size.
pub struct View {
    /// The transform from scene coordinates to logical output coordinates.
    pub camera: Transform,
    /// Physical pixels per logical pixel.
    pub scale: f64,
    /// The size of the target in physical pixels.
    pub size: smithay::utils::Size<i32, smithay::utils::Physical>,
}

impl View {
    /// The view of an output.
    pub fn of(state: &State, output: &Output) -> Self {
        Self {
            camera: state.camera(output),
            scale: output.current_scale().fractional_scale(),
            size: physical_size(output),
        }
    }
}

/// The size of an output in physical pixels, after its transform.
pub fn physical_size(output: &Output) -> smithay::utils::Size<i32, smithay::utils::Physical> {
    let scale = output.current_scale().fractional_scale();
    output_area(output)
        .size
        .to_f64()
        .to_physical(scale)
        .to_i32_ceil()
}

/// The parts of the state that rendering reads. A struct of borrows, so
/// that a backend can borrow its own state at the same time.
pub struct Parts<'a> {
    pub scene: &'a Scene,
    pub windows: &'a HashMap<String, Window>,
    pub assets: &'a mut Assets,
    pub lock: Option<&'a Lock>,
    /// How deep inside node textures the rendering is; 0 at the top.
    pub depth: u8,
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
    R: MeshRenderer,
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
    prune_assets(parts);
    let mut out = layer_elements(renderer, output, &layers::ABOVE, view.scale);
    let scene = parts.scene;
    let list = scene.layer_list(chora_scene::ROOT);
    out.extend(elements(parts, renderer, &list, view));
    out.extend(layer_elements(renderer, output, &layers::BELOW, view.scale));
    // The layers that this frame of the output leaves out, for
    // `layers.dropped` once the frame shows.
    let assets = &mut *parts.assets;
    assets
        .frame_left_out
        .insert(output.name(), assets.layers.left_out.clone());
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
    R: MeshRenderer,
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

/// The render elements of nodes of a layer list, front first. A layer in
/// the list renders into a texture and shows through its mesh.
fn elements<R: MeshRenderer>(
    parts: &mut Parts<'_>,
    renderer: &mut R,
    list: &[Placed<'_>],
    view: &View,
) -> Vec<SceneElement<R>> {
    let mut out = Vec::new();
    for placed in list.iter().rev() {
        if placed.node.is_layer() {
            out.extend(layer(parts, renderer, placed, view));
        } else {
            out.extend(node_elements(parts, renderer, placed, view));
        }
    }
    out
}

/// The element of a layer node: renders the node and its descendants into
/// its texture and draws that with the transform of the node.
fn layer<R: MeshRenderer>(
    parts: &mut Parts<'_>,
    renderer: &mut R,
    placed: &Placed<'_>,
    view: &View,
) -> Option<SceneElement<R>> {
    let node = placed.node;
    let transform = view.camera.then(&placed.transform);
    let at = Placing {
        transform: &transform,
        scale: view.scale,
    };
    // A layer outside of the output renders nothing.
    let shape = mesh::shape(node, &at)?;
    if !shape.bounds.overlaps(Rectangle::from_size(view.size)) {
        return None;
    }
    let size = mesh::texture_size(node, view.scale)?;
    let textures = 1 + mesh::passes(node, view.scale).len().min(2);
    if !parts.assets.layers.spend(&node.id, size, textures) {
        return None;
    }
    let texture = layer_texture(parts, renderer.gles(), node, size, view.scale)?;
    #[allow(clippy::cast_possible_truncation)]
    let alpha = placed.opacity as f32;
    let element = MeshElement::new(&mut parts.assets.layers, node, texture, shape, alpha);
    Some(SceneElement::Mesh(element))
}

/// Renders a layer node with its descendants into its texture, and applies
/// its effects.
fn layer_texture(
    parts: &mut Parts<'_>,
    gles: &mut GlesRenderer,
    node: &chora_scene::Node,
    size: smithay::utils::Size<i32, smithay::utils::Buffer>,
    scale: f64,
) -> Option<GlesTexture> {
    let inner = View {
        camera: Transform::IDENTITY,
        scale,
        size: (size.w, size.h).into(),
    };
    let content = flat_elements(parts, gles, &node.id, &inner)?;
    let mut texture = parts.assets.layers.texture(gles, &node.id, 0, size).ok()?;
    if let Err(err) = mesh::render_into(gles, &mut texture, scale, &content) {
        tracing::warn!(%err, "a layer failed");
        return None;
    }
    // Texture pixels per logical pixel, also when the texture hit its
    // largest size.
    let pixels = node
        .props
        .size
        .map_or(scale, |[w, _]| f64::from(size.w) / w.max(f64::MIN_POSITIVE));
    let textures = node_textures(parts, gles, node, pixels);
    let assets = &mut *parts.assets;
    Some(mesh::apply_passes(
        gles,
        (&mut assets.layers, &mut assets.effect_errors),
        node,
        texture,
        (pixels, &textures),
    ))
}

/// The node textures of a shader node by the names of their uniforms: each
/// node that a uniform names, rendered on its own at its size. Inside a node
/// texture, shader nodes get none, so that shader nodes that read each
/// other do not recurse.
fn node_textures(
    parts: &mut Parts<'_>,
    gles: &mut GlesRenderer,
    node: &chora_scene::Node,
    scale: f64,
) -> Vec<(String, GlesTexture)> {
    if node.kind != NodeKind::Shader || parts.depth > 0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (slot, (name, uniform)) in (3..).zip(&node.props.uniforms) {
        let chora_scene::Uniform::Texture(source) = uniform else {
            continue;
        };
        let Some(size) = source_size(parts, &source.node).and_then(|s| mesh::pixel_size(s, scale))
        else {
            continue;
        };
        if !parts.assets.layers.spend(&source.node, size, 1) {
            continue;
        }
        let view = View {
            camera: Transform::IDENTITY,
            scale,
            size: (size.w, size.h).into(),
        };
        parts.depth += 1;
        let content = flat_elements(parts, gles, &source.node, &view);
        parts.depth -= 1;
        let Some(content) = content else { continue };
        let Ok(mut texture) = parts.assets.layers.texture(gles, &node.id, slot, size) else {
            continue;
        };
        if mesh::render_into(gles, &mut texture, scale, &content).is_ok() {
            out.push((name.clone(), texture));
        }
    }
    out
}

/// The logical size of a node as a node texture: its size, or the window
/// geometry of a surface.
fn source_size(parts: &Parts<'_>, id: &str) -> Option<[f64; 2]> {
    let node = parts.scene.node(id)?;
    if node.kind == NodeKind::Surface {
        let size = parts.windows.get(id)?.geometry().size;
        return Some([f64::from(size.w), f64::from(size.h)]);
    }
    node.props.size
}

/// The render elements of a node and its descendants, front first, in node
/// coordinates: the node shows without its own transform, opacity, mesh and
/// effects. Nested layers show through their meshes. `None` for a node
/// that is gone or hidden.
pub fn flat_elements<R: MeshRenderer>(
    parts: &mut Parts<'_>,
    renderer: &mut R,
    id: &str,
    view: &View,
) -> Option<Vec<SceneElement<R>>> {
    let scene = parts.scene;
    let list = scene.layer_list(id);
    let (first, rest) = list.split_first()?;
    let mut content = elements(parts, renderer, rest, view);
    content.extend(node_elements(parts, renderer, first, view));
    Some(content)
}

/// The render elements of a node with its descendants for a capture of the
/// node: as [`flat_elements`], but with the shape, the shader and the clip
/// of the node itself. Its mesh and its effects stay out, so that a
/// capture shows the content of a window.
pub fn shown_elements<R: MeshRenderer>(
    parts: &mut Parts<'_>,
    renderer: &mut R,
    id: &str,
    view: &View,
) -> Option<Vec<SceneElement<R>>> {
    // A capture of a node has a pixel budget of its own, not the rest of
    // the last frame.
    parts.assets.layers.new_budget();
    let scene = parts.scene;
    let list = scene.layer_list(id);
    let first = list.first()?;
    let mut node = first.node.clone();
    node.props.mesh = None;
    node.props.effects.clear();
    if !node.is_layer() {
        return flat_elements(parts, renderer, id, view);
    }
    let placed = Placed {
        node: &node,
        ..first.clone()
    };
    Some(layer(parts, renderer, &placed, view).into_iter().collect())
}

/// The render elements of one node that is not a layer, front first. The
/// rects keep one buffer per rect node, so that damage tracking sees
/// unchanged rects as unchanged.
fn node_elements<R: MeshRenderer>(
    parts: &mut Parts<'_>,
    renderer: &mut R,
    placed: &Placed<'_>,
    view: &View,
) -> Vec<SceneElement<R>> {
    let transform = view.camera.then(&placed.transform);
    let Some((offset, (sx, sy))) = location_and_scale(&transform) else {
        return Vec::new();
    };
    // Physical pixels from here on.
    let offset = offset.upscale(view.scale);
    let scale = (sx * view.scale, sy * view.scale);
    #[allow(clippy::cast_possible_truncation)]
    let alpha = placed.opacity as f32;
    match placed.node.kind {
        NodeKind::Surface => {
            let Some(window) = parts.windows.get(&placed.node.id) else {
                return Vec::new();
            };
            // Smithay sizes a surface by the output scale only, so the node
            // scale applies afterwards, around the node position.
            let origin = offset.to_physical_precise_round(1.0);
            let loc = offset - window.geometry().loc.to_f64().upscale(view.scale);
            window
                .render_elements::<WaylandSurfaceRenderElement<R>>(
                    renderer,
                    loc.to_physical_precise_round(1.0),
                    view.scale.into(),
                    alpha,
                )
                .into_iter()
                .map(|e| {
                    SceneElement::Scaled(RescaleRenderElement::from_element(e, origin, (sx, sy)))
                })
                .collect()
        }
        NodeKind::Rect => rect(placed, &mut parts.assets.rects, offset, scale, alpha)
            .into_iter()
            .collect(),
        NodeKind::Image => {
            let at = Placement {
                origin: offset,
                scale: (sx, sy),
                output_scale: view.scale,
                alpha,
            };
            let Assets { images, cells, .. } = &mut *parts.assets;
            image(placed, images, cells, renderer, &at)
        }
        // A shader node draws as a layer; outside of one it shows nothing.
        NodeKind::Group | NodeKind::Shader => Vec::new(),
    }
}

/// Drops the buffers of nodes that are gone or of another kind.
fn prune_assets(parts: &mut Parts<'_>) {
    let scene = parts.scene;
    let kind_is = |id: &str, kind| scene.node(id).is_some_and(|n| n.kind == kind);
    let assets = &mut *parts.assets;
    assets.rects.retain(|id, _| kind_is(id, NodeKind::Rect));
    assets.cells.retain(|id, _| kind_is(id, NodeKind::Image));
    assets
        .layers
        .retain(|id| scene.node(id).is_some_and(chora_scene::Node::is_layer));
}

fn rect<R>(
    placed: &Placed<'_>,
    rects: &mut HashMap<String, SolidColorBuffer>,
    offset: Point<f64, Logical>,
    scale: (f64, f64),
    alpha: f32,
) -> Option<SceneElement<R>>
where
    R: MeshRenderer,
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

/// The render elements of an image node, if its file is loaded.
fn image<R>(
    placed: &Placed<'_>,
    images: &Images,
    cells: &mut HashMap<String, (Option<Id>, Vec<Id>)>,
    renderer: &mut R,
    at: &Placement,
) -> Vec<SceneElement<R>>
where
    R: MeshRenderer,
{
    let props = &placed.node.props;
    let Some(image) = props.image.as_deref().and_then(|p| images.get(p)) else {
        return Vec::new();
    };
    let size = props
        .size
        .unwrap_or([f64::from(image.size[0]), f64::from(image.size[1])]);
    let parts = images::cells(image.size, size, props.slice);
    let (version, ids) = cells.entry(placed.node.id.clone()).or_default();
    if version.as_ref() != Some(&image.version) {
        *version = Some(image.version.clone());
        ids.clear();
    }
    images::elements(renderer, image, &parts, ids, at)
        .into_iter()
        .map(SceneElement::Image)
        .collect()
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

/// The node of the scene under the pointer that takes pointer input: a
/// surface, a rect or an image. `None` over the lock screen, over a layer
/// surface above the scene, and over nothing.
pub fn node_under(state: &State, spot: &Spot) -> Option<String> {
    if state.lock.is_some()
        || layers::surface_under(&spot.output, &layers::ABOVE, spot.local).is_some()
    {
        return None;
    }
    let list = state.scene.draw_list();
    let (placed, _) = list
        .iter()
        .rev()
        .filter(|p| p.input != InputMode::Pass)
        .find_map(|p| Some((p, hit(state, p, spot.scene)?)))?;
    (placed.input == InputMode::Accept).then(|| placed.node.id.clone())
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
    if !placed
        .clips
        .iter()
        .all(|(transform, size)| inside(transform, *size, pos))
    {
        return None;
    }
    let Some((offset, (sx, sy))) = location_and_scale(&placed.transform) else {
        return layer_hit(placed, pos);
    };
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
        NodeKind::Rect | NodeKind::Image | NodeKind::Shader => {
            let [w, h] = node_size(state, placed);
            ((0.0..w).contains(&local.x) && (0.0..h).contains(&local.y)).then_some(Hit::Area)
        }
        NodeKind::Group => None,
    }
}

/// Whether a point of the scene is inside the clip of a node with a
/// transform and a size. A clip under a transform that is not a plain 2D
/// one takes the input over its bounding box, as a layer does.
fn inside(transform: &Transform, [w, h]: [f64; 2], pos: Point<f64, Logical>) -> bool {
    let Some((offset, (sx, sy))) = location_and_scale(transform) else {
        return true;
    };
    let (x, y) = ((pos.x - offset.x) / sx, (pos.y - offset.y) / sy);
    (0.0..w).contains(&x) && (0.0..h).contains(&y)
}

/// What a point hits of a node under a transform that is not a plain 2D
/// one: the bounding box of a layer takes the input, so that it does not
/// fall through to the nodes behind.
fn layer_hit(placed: &Placed<'_>, pos: Point<f64, Logical>) -> Option<Hit> {
    if !placed.node.is_layer() {
        return None;
    }
    let at = Placing {
        transform: &placed.transform,
        scale: 1.0,
    };
    let bounds = mesh::shape(placed.node, &at)?.bounds;
    let pos = pos.to_physical(1.0).to_i32_round();
    bounds.contains(pos).then_some(Hit::Area)
}

/// The size of a rect or an image node; an image without a size has the
/// size of its file.
fn node_size(state: &State, placed: &Placed<'_>) -> [f64; 2] {
    let props = &placed.node.props;
    props.size.unwrap_or_else(|| {
        props
            .image
            .as_deref()
            .and_then(|p| state.assets.images.get(p))
            .map_or([0.0; 2], |i| [f64::from(i.size[0]), f64::from(i.size[1])])
    })
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

/// Takes the presentation feedback of the surfaces that an output shows,
/// for a frame with damage. A locked output shows its lock surface only.
pub fn take_feedback(state: &State, output: &Output) -> OutputPresentationFeedback {
    let mut feedback = OutputPresentationFeedback::new(output);
    let primary = |_: &WlSurface, _: &SurfaceData| Some(output.clone());
    let flags = |_: &WlSurface, _: &SurfaceData| wp_presentation_feedback::Kind::empty();
    if let Some(lock) = &state.lock {
        if let Some(surface) = lock.surface(output) {
            take_presentation_feedback_surface_tree(surface, &mut feedback, primary, flags);
        }
        return feedback;
    }
    let visible = shown_surfaces(state, output);
    for (id, window) in &state.windows {
        if visible.contains(id) {
            window.take_presentation_feedback(&mut feedback, primary, flags);
        }
    }
    for layer in layer_map_for_output(output).layers() {
        layer.take_presentation_feedback(&mut feedback, primary, flags);
    }
    feedback
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
