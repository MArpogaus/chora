//! Turns the scene into render elements, and finds the surface under a
//! point.

use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use chora_scene::{InputMode, Node, NodeKind, Placed, ROOT, Scene, Transform, overlay_id};
use smithay::{
    backend::allocator::Fourcc,
    backend::renderer::{
        ImportMem,
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
    backdrop::{BackdropElement, Backdrops},
    images::{self, CellElement, Images, Placement},
    layers,
    lock::Lock,
    mesh::{self, Layers, MeshElement, MeshRenderer, Placing},
    output::{Spot, default_camera, output_area, scene_rect},
    space,
    state::State,
    text::{self, Texts},
};

render_elements! {
    pub SceneElement<R> where R: MeshRenderer;
    Surface = WaylandSurfaceRenderElement<R>,
    Scaled = RescaleRenderElement<WaylandSurfaceRenderElement<R>>,
    Solid = SolidColorRenderElement,
    Cursor = MemoryRenderBufferRenderElement<R>,
    Image = CellElement<R>,
    Mesh = MeshElement,
    Text = RescaleRenderElement<MemoryRenderBufferRenderElement<R>>,
    Backdrop = BackdropElement,
}

/// The kind of a [`SceneElement`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    Surface,
    Scaled,
    Solid,
    Cursor,
    Image,
    Mesh,
    Text,
    Backdrop,
}

impl<R: MeshRenderer> SceneElement<R> {
    /// The kind of the element.
    pub fn variant(&self) -> Variant {
        match self {
            Self::Surface(_) => Variant::Surface,
            Self::Scaled(_) => Variant::Scaled,
            Self::Solid(_) => Variant::Solid,
            Self::Cursor(_) => Variant::Cursor,
            Self::Image(_) => Variant::Image,
            Self::Mesh(_) => Variant::Mesh,
            Self::Text(_) => Variant::Text,
            Self::Backdrop(_) => Variant::Backdrop,
            Self::_GenericCatcher((_, never)) => match *never {},
        }
    }
}

/// One counter per [`Variant`]; a new variant fails to compile in `slot`.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ElementCounts {
    surface: u64,
    scaled: u64,
    solid: u64,
    cursor: u64,
    image: u64,
    mesh: u64,
    text: u64,
    backdrop: u64,
}

impl ElementCounts {
    fn slot(&mut self, variant: Variant) -> &mut u64 {
        match variant {
            Variant::Surface => &mut self.surface,
            Variant::Scaled => &mut self.scaled,
            Variant::Solid => &mut self.solid,
            Variant::Cursor => &mut self.cursor,
            Variant::Image => &mut self.image,
            Variant::Mesh => &mut self.mesh,
            Variant::Text => &mut self.text,
            Variant::Backdrop => &mut self.backdrop,
        }
    }
}

/// What rendering did since the start of the core, for the tests that
/// guard the cost of a scene. The counters only grow.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct RenderStats {
    /// The elements that [`output_elements`] returned, by variant.
    pub elements: ElementCounts,
    /// The node contents that rendered into a texture of their own.
    pub textures_rendered: u64,
    /// The shader passes over such textures: shapes, shaders and effects.
    pub passes: u64,
    /// The counters by output, for `render.stats`.
    pub by_output: HashMap<String, OutputStats>,
    /// The frames that a backend rendered.
    pub frames: u64,
    /// The frames of those that had damage.
    pub damaged_frames: u64,
    /// The nodes that drew as planes: nodes whose transform is not a 2D
    /// translation with a positive scale.
    pub planes: u64,
    /// The pieces of planes that a split or the near clip of a 3D context
    /// left, drawn on their own.
    pub splits: u64,
    /// The textures that the layers allocated on the GPU.
    pub textures_allocated: u64,
    /// The drawing orders of 3D contexts that the splitter computed; a
    /// frame and a hit test of an unchanged context reuse one.
    pub orders: std::cell::Cell<u64>,
}

/// What rendering did for one output since it came.
#[derive(Default, Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct OutputStats {
    /// The frames that rendered.
    pub frames: u64,
    /// The frames of those that had damage.
    pub damaged_frames: u64,
    /// The offscreen renders of its frames: node contents and shader
    /// passes. A capture does not count.
    pub passes: u64,
}

impl RenderStats {
    /// Counts the render of a frame of an output.
    pub fn frame(&mut self, output: &str, damaged: bool) {
        self.frames += 1;
        self.damaged_frames += u64::from(damaged);
        let of = self.by_output.entry(output.to_owned()).or_default();
        of.frames += 1;
        of.damaged_frames += u64::from(damaged);
    }

    /// The offscreen renders: node contents and shader passes.
    pub fn offscreen(&self) -> u64 {
        self.textures_rendered + self.passes
    }

    fn count<R: MeshRenderer>(&mut self, elements: &[SceneElement<R>]) {
        for element in elements {
            *self.elements.slot(element.variant()) += 1;
        }
    }
}

#[cfg(test)]
impl RenderStats {
    /// How many elements of a variant [`output_elements`] returned.
    pub fn elements_of(&mut self, variant: Variant) -> u64 {
        *self.elements.slot(variant)
    }
}

/// What the scene draws besides surfaces: a color buffer per rect node,
/// the images, the ids of the image cells of each image node, and the
/// textures of the layers.
#[derive(Default)]
pub struct Assets {
    pub rects: HashMap<String, SolidColorBuffer>,
    pub images: Images,
    /// The layouts and rasters of the text nodes.
    pub texts: Texts,
    /// The version of the image and the ids of the cells by node.
    pub cells: HashMap<String, (Option<Id>, Vec<Id>)>,
    pub layers: Layers,
    /// The blurs of the backdrops of rects.
    pub backdrops: Backdrops,
    /// The drawing orders of the 3D contexts, which the draw and the hit
    /// test share.
    pub orders: space::Orders,
    /// Errors of effect shaders since the last report.
    pub effect_errors: Vec<String>,
    /// The time of each shader file that compiled in a check.
    pub checked_effects: HashMap<String, Option<std::time::SystemTime>>,
    /// The nodes that the layer budget left out of the last frame of each
    /// output, as `layers.dropped` told them.
    pub left_out: HashMap<String, Vec<String>>,
    /// The nodes that the last render of each output left out, before the
    /// frame shows.
    pub frame_left_out: HashMap<String, Vec<String>>,
    /// Those of `frame_left_out` that draw nothing, for the input.
    pub frame_dark: HashMap<String, Vec<String>>,
    /// What rendering counted since the start of the core.
    pub stats: RenderStats,
    /// The virtual outputs, which render into textures.
    pub virtuals: Vec<crate::offscreen::Virtual>,
    /// The virtual outputs that rendered since the last frame callbacks,
    /// and whether they had damage.
    pub rendered_virtuals: Vec<(Output, bool)>,
    /// One transparent pixel, for a shader node whose output is gone.
    blank: Option<GlesTexture>,
}

impl Assets {
    /// The transparent pixel.
    fn blank(&mut self, gles: &mut GlesRenderer) -> Option<GlesTexture> {
        if self.blank.is_none() {
            let pixel = gles.import_memory(&[0; 4], Fourcc::Abgr8888, (1, 1).into(), false);
            self.blank = pixel.ok();
        }
        self.blank.clone()
    }
}

/// How an output shows the scene: its camera, its scale factor and its size.
pub struct View {
    /// The transform from scene coordinates to logical output coordinates.
    pub camera: Transform,
    /// Physical pixels per logical pixel.
    pub scale: f64,
    /// The size of the target in physical pixels.
    pub size: smithay::utils::Size<i32, smithay::utils::Physical>,
    /// The least w of a point of a layer that shows: the GPU cuts at 0, a
    /// 3D context cuts its planes at [`space::NEAR_W`].
    pub near: f64,
    /// Whether planes step the scale of their textures with their size on
    /// screen; a capture renders them at the full scale.
    pub stepped: bool,
}

impl View {
    /// The view of an output.
    pub fn of(state: &State, output: &Output) -> Self {
        Self::in_scene(&state.scene, output)
    }

    /// The view of an output with its camera in a scene.
    pub fn in_scene(scene: &Scene, output: &Output) -> Self {
        let camera = scene.current_camera(&output.name());
        Self {
            camera: camera.unwrap_or_else(|| default_camera(output)),
            scale: output.current_scale().fractional_scale(),
            size: physical_size(output),
            near: mesh::NEAR,
            stepped: true,
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

/// The render elements of one output without a cursor, as
/// [`screen_elements`] gives them.
pub fn output_elements<R>(
    parts: &mut Parts<'_>,
    renderer: &mut R,
    output: &Output,
    view: &View,
) -> Vec<SceneElement<R>>
where
    R: MeshRenderer,
{
    screen_elements(parts, renderer, output, view, Vec::new())
}

/// The render elements of a screen, front first: the injection mark,
/// the elements of the `cursor`, the layer surfaces above the scene, the
/// overlay, the scene, and the layer surfaces below it. No cursor and no
/// node of a client covers the mark (D23). A locked session shows only
/// the cursor and the lock surface.
pub fn screen_elements<R>(
    parts: &mut Parts<'_>,
    renderer: &mut R,
    output: &Output,
    view: &View,
    cursor: Vec<SceneElement<R>>,
) -> Vec<SceneElement<R>>
where
    R: MeshRenderer,
{
    if let Some(lock) = parts.lock {
        let elements = lock.surface(output).map_or_else(Vec::new, |surface| {
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
        parts.assets.stats.count(&elements);
        return cursor.into_iter().chain(elements).collect();
    }
    crate::offscreen::render_virtuals(parts, renderer.gles(), output);
    let passes_before = parts.assets.stats.offscreen();
    prune_assets(parts);
    begin_view(parts.assets, output, view);
    let scene = parts.scene;
    // The overlay of the output, above the scene and without its camera
    // (D43).
    let screen = View {
        camera: Transform::IDENTITY,
        scale: view.scale,
        size: view.size,
        near: mesh::NEAR,
        stepped: true,
    };
    let (mark, overlay): (Vec<_>, Vec<_>) = scene
        .overlay_layer_list(&output.name())
        .into_iter()
        .partition(|placed| placed.node.id == chora_scene::INJECTION);
    let mut out = elements(parts, renderer, &mark, &screen);
    let marks = out.len();
    out.extend(layer_elements(renderer, output, &layers::ABOVE, view.scale));
    out.extend(elements(parts, renderer, &overlay, &screen));
    let list = scene.layer_list(chora_scene::ROOT);
    out.extend(elements(parts, renderer, &list, view));
    out.extend(layer_elements(renderer, output, &layers::BELOW, view.scale));
    settle_backdrops(parts.assets, &mut out, view.scale);
    // The layers that this frame of the output leaves out, for
    // `layers.dropped` once the frame shows.
    let assets = &mut *parts.assets;
    assets
        .frame_left_out
        .insert(output.name(), assets.layers.left_out.clone());
    assets
        .frame_dark
        .insert(output.name(), assets.layers.dark.clone());
    assets.stats.count(&out);
    if view.stepped {
        let passes = assets.stats.offscreen() - passes_before;
        assets
            .stats
            .by_output
            .entry(output.name())
            .or_default()
            .passes += passes;
    }
    out.splice(marks..marks, cursor);
    out
}

/// Names the view that renders next, the output or a capture of it, to
/// the layers and the backdrops.
fn begin_view(assets: &mut Assets, output: &Output, view: &View) {
    let shown = if view.stepped {
        ""
    } else {
        crate::backdrop::CAPTURE
    };
    let name = format!("{}{shown}", output.name());
    assets.backdrops.begin(&name);
    assets.layers.set_view(name);
}

/// Settles the backdrops of a list of elements, front first. The draws
/// count their passes apart, so that `render.stats` does not tell when
/// the frame behind a backdrop changed.
fn settle_backdrops<R: MeshRenderer>(
    assets: &mut Assets,
    list: &mut [SceneElement<R>],
    scale: f64,
) {
    assets
        .backdrops
        .settle(list, backdrop_of, Scale::from(scale));
}

fn backdrop_of<R: MeshRenderer>(element: &mut SceneElement<R>) -> Option<&mut BackdropElement> {
    match element {
        SceneElement::Backdrop(backdrop) => Some(backdrop),
        _ => None,
    }
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
        out.extend(entry_elements(parts, renderer, placed, view));
    }
    out
}

/// The render elements of one entry of a layer list, front first.
fn entry_elements<R: MeshRenderer>(
    parts: &mut Parts<'_>,
    renderer: &mut R,
    placed: &Placed<'_>,
    view: &View,
) -> Vec<SceneElement<R>> {
    let mut out = if placed.node.is_layer() {
        layer(parts, renderer, placed, view).into_iter().collect()
    } else if placed.node.is_context() && view.camera.then(&placed.transform).is_plain() {
        context_elements(parts, renderer, placed, view)
    } else if let Some(size) = plane_size(parts, placed, view) {
        plane(parts, renderer, placed, size, view)
            .into_iter()
            .collect()
    } else {
        node_elements(parts, renderer, placed, view)
    };
    if placed.node.props.backdrop.is_some() {
        let transform = view.camera.then(&placed.transform);
        let at = Placing {
            transform: &transform,
            scale: view.scale,
            near: view.near,
        };
        let shape = mesh::shape(placed.node, &at);
        out.extend(backdrop(parts, placed, (shape, 0), view));
    }
    out
}

/// The element of the backdrop of a rect behind its shape, or a piece of
/// it. A node texture shows no backdrop, so that no shader reads what
/// lies behind a node.
fn backdrop<R: MeshRenderer>(
    parts: &mut Parts<'_>,
    placed: &Placed<'_>,
    (shape, piece): (Option<mesh::Shape>, usize),
    view: &View,
) -> Option<SceneElement<R>> {
    let shape = shape.filter(|s| s.bounds.overlaps(Rectangle::from_size(view.size)))?;
    if parts.depth > 0 || placed.opacity <= 0.0 {
        return None;
    }
    let size = (shape.bounds.size.w, shape.bounds.size.h).into();
    if !parts.assets.layers.afford(size, 2) {
        return None;
    }
    #[allow(clippy::cast_possible_truncation)]
    let alpha = placed.opacity as f32;
    let assets = &mut *parts.assets;
    let element = assets.backdrops.element(
        assets.layers.view(),
        (placed.node, piece),
        shape,
        (alpha, view.scale),
    )?;
    Some(SceneElement::Backdrop(element))
}

/// The render elements of the planes of a 3D context, front first: sorted,
/// split and clipped at the near plane (D75). A whole plane draws as it
/// would outside the context; a piece draws the texture of its plane
/// through its part of the quad, and the pieces of a plane share one
/// texture.
fn context_elements<R: MeshRenderer>(
    parts: &mut Parts<'_>,
    renderer: &mut R,
    context: &Placed<'_>,
    view: &View,
) -> Vec<SceneElement<R>> {
    let scene = parts.scene;
    let list = scene.context_list(context);
    let pieces = {
        let (windows, assets) = (parts.windows, &*parts.assets);
        let computed = &assets.stats.orders;
        assets
            .orders
            .get(computed, context, &list, |p| flat(windows, assets, p))
    };
    let view = &View {
        near: space::NEAR_W,
        ..*view
    };
    let mut textures = HashMap::new();
    let mut out = Vec::new();
    for (index, piece) in pieces.iter().enumerate().rev() {
        let entry = &list[piece.plane];
        let size = content_size(parts.windows, parts.assets, entry.node);
        let viewed = viewed(context, entry, size);
        let entry = Placed {
            node: viewed.as_ref().unwrap_or(entry.node),
            ..entry.clone()
        };
        match (&piece.part, size) {
            (None, _) => out.extend(entry_elements(parts, renderer, &entry, view)),
            (Some(part), Some(size)) => {
                let at = (part, index + 1, size, &mut textures);
                out.extend(piece_element(parts, renderer, &entry, at, view));
                if entry.node.props.backdrop.is_some() {
                    let transform = view.camera.then(&entry.transform);
                    let shape = mesh::piece_shape(&part.corners, size, &transform, view.scale);
                    out.extend(backdrop(parts, &entry, (shape, index + 1), view));
                }
            }
            (Some(_), None) => {}
        }
    }
    out
}

/// What the order of a context needs of an entry of its list: the content
/// size of a node that draws, and whether it draws as one unit, as a mesh
/// does. A group draws only as a layer, a plane or a
/// context; otherwise its children draw on their own.
fn flat(
    windows: &HashMap<String, Window>,
    assets: &Assets,
    placed: &Placed<'_>,
) -> Option<space::Flat> {
    let node = placed.node;
    let draws = node.kind != NodeKind::Group
        || node.is_layer()
        || node.is_context()
        || !placed.transform.is_plain();
    draws.then_some(())?;
    Some(space::Flat {
        size: content_size(windows, assets, node)?,
        unit: node.props.mesh.is_some(),
    })
}

/// A copy of a node of a 3D context whose shader or effects get the
/// uniforms `depth` and `facing` of its plane; `None` for a node without
/// them.
fn viewed(context: &Placed<'_>, entry: &Placed<'_>, size: Option<[f64; 2]>) -> Option<Node> {
    let node = entry.node;
    if node.kind != NodeKind::Shader && node.props.effects.is_empty() {
        return None;
    }
    let (depth, facing) = space::view(context, entry, size?)?;
    let mut node = node.clone();
    let set = |uniforms: &mut std::collections::BTreeMap<String, chora_scene::Uniform>| {
        uniforms.insert("depth".into(), chora_scene::Uniform::Float(depth));
        uniforms.insert("facing".into(), chora_scene::Uniform::Float(facing));
    };
    if node.kind == NodeKind::Shader {
        set(&mut node.props.uniforms);
    }
    for effect in &mut node.props.effects {
        set(&mut effect.uniforms);
    }
    Some(node)
}

/// A piece of a plane of a context as [`piece_element`] draws it: its
/// part, its number in the context, the content size of its plane and the
/// textures of the planes of the context in this frame.
type Piece<'a> = (
    &'a space::Part,
    usize,
    [f64; 2],
    &'a mut HashMap<String, Option<(GlesTexture, u64)>>,
);

/// The element of a piece of a plane of a context: the texture of the
/// plane, rendered once per frame for all its pieces, through the part of
/// the quad that the piece covers.
fn piece_element<R: MeshRenderer>(
    parts: &mut Parts<'_>,
    renderer: &mut R,
    placed: &Placed<'_>,
    (part, piece, size, textures): Piece<'_>,
    view: &View,
) -> Option<SceneElement<R>> {
    let mut node = placed.node.clone();
    node.props.size = Some(size);
    let transform = view.camera.then(&placed.transform);
    let shape = mesh::piece_shape(&part.corners, size, &transform, view.scale)?;
    if !shape.bounds.overlaps(Rectangle::from_size(view.size)) {
        return None;
    }
    let drawn = if let Some(drawn) = textures.get(&node.id) {
        drawn.clone()
    } else {
        let scale = texture_scale(&mut parts.assets.layers, &node, &transform, view);
        let drawn = node_texture(parts, renderer.gles(), &node, scale);
        textures.insert(node.id.clone(), drawn.clone());
        drawn
    }?;
    parts.assets.stats.splits += 1;
    #[allow(clippy::cast_possible_truncation)]
    let alpha = placed.opacity as f32;
    let element = MeshElement::piece(
        &mut parts.assets.layers,
        (&node, piece),
        drawn,
        shape,
        alpha,
    );
    Some(SceneElement::Mesh(element))
}

/// The size of a node that is a plane without being a layer: a node with
/// a content size whose transform below its layer is not a 2D translation
/// with a positive scale. The content size is the size, the window
/// geometry of a surface, the file of an image or the layout of a text.
fn plane_size(parts: &Parts<'_>, placed: &Placed<'_>, view: &View) -> Option<[f64; 2]> {
    if view.camera.then(&placed.transform).is_plain() {
        return None;
    }
    content_size(parts.windows, parts.assets, placed.node)
}

/// The size of the content of a node that draws or takes input as a
/// plane: its size, the window geometry of a surface, the file of an
/// image without a size, the layout of a text. `None` when it has none.
pub(crate) fn content_size(
    windows: &HashMap<String, Window>,
    assets: &Assets,
    node: &Node,
) -> Option<[f64; 2]> {
    let props = &node.props;
    let size = match node.kind {
        NodeKind::Surface => {
            let size = windows.get(&node.id)?.geometry().size;
            [f64::from(size.w), f64::from(size.h)]
        }
        NodeKind::Text => assets.texts.size(node)?,
        NodeKind::Image => props.size.or_else(|| {
            let image = assets.images.get(props.image.as_deref()?)?;
            Some([f64::from(image.size[0]), f64::from(image.size[1])])
        })?,
        NodeKind::Rect | NodeKind::Shader | NodeKind::Group => props.size?,
    };
    (size[0] > 0.0 && size[1] > 0.0).then_some(size)
}

/// The element of a plane: its content renders into a texture of its
/// size, which draws through the homography of the plane.
fn plane<R: MeshRenderer>(
    parts: &mut Parts<'_>,
    renderer: &mut R,
    placed: &Placed<'_>,
    size: [f64; 2],
    view: &View,
) -> Option<SceneElement<R>> {
    let mut node = placed.node.clone();
    node.props.size = Some(size);
    let sized = Placed {
        node: &node,
        ..placed.clone()
    };
    let element = layer(parts, renderer, &sized, view)?;
    parts.assets.stats.planes += 1;
    Some(element)
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
        near: view.near,
    };
    // A layer outside of the output renders nothing.
    let shape = mesh::shape(node, &at)?;
    if !shape.bounds.overlaps(Rectangle::from_size(view.size)) {
        return None;
    }
    let scale = texture_scale(&mut parts.assets.layers, node, &transform, view);
    let drawn = node_texture(parts, renderer.gles(), node, scale)?;
    #[allow(clippy::cast_possible_truncation)]
    let alpha = placed.opacity as f32;
    let element = MeshElement::new(&mut parts.assets.layers, node, drawn, shape, alpha);
    Some(SceneElement::Mesh(element))
}

/// The scale of the texture of a layer node, in texture pixels per logical
/// pixel: the output scale, and for a plane without a mesh a power of two
/// step below it that follows the size on screen, so that a far plane
/// renders fewer pixels and a walk crosses a step rarely.
fn texture_scale(layers: &mut Layers, node: &Node, transform: &Transform, view: &View) -> f64 {
    let scale = view.scale;
    if !view.stepped {
        return scale;
    }
    let ratio = (!transform.is_plain() && node.props.mesh.is_none())
        .then_some(node.props.size)
        .flatten()
        .and_then(|size| projected_ratio(transform, size));
    scale * layers.step(&node.id, ratio)
}

/// The most that a plane on screen is larger than its size: the longest
/// edge of its projection over the edge of its size. `None` when a corner
/// lies behind the eye.
fn projected_ratio(transform: &Transform, [width, height]: [f64; 2]) -> Option<f64> {
    let at = |across, down| project_point(transform, across, down);
    let corners = [
        at(0.0, 0.0)?,
        at(width, 0.0)?,
        at(width, height)?,
        at(0.0, height)?,
    ];
    let edge = |from: usize, to: usize| {
        let (p, q) = (corners[from], corners[to]);
        (p.0 - q.0).hypot(p.1 - q.1)
    };
    let ratio = (edge(0, 1).max(edge(3, 2)) / width).max(edge(0, 3).max(edge(1, 2)) / height);
    ratio.is_finite().then_some(ratio)
}

/// The texture of a layer node with its passes and its version, if the
/// budget of the frame has room for it.
fn node_texture(
    parts: &mut Parts<'_>,
    gles: &mut GlesRenderer,
    node: &Node,
    scale: f64,
) -> Option<(GlesTexture, u64)> {
    let max = parts.assets.layers.max_side(gles);
    let scale = mesh::fit_scale(node.props.size?, scale, max);
    let size = mesh::texture_size(node, scale, max)?;
    let textures = 1 + mesh::passes(node, scale).len().min(2);
    if !parts.assets.layers.spend(&node.id, size, textures) {
        return None;
    }
    layer_texture(parts, gles, node, size, scale)
}

/// Renders a layer node with its descendants into its texture, and applies
/// its effects. The texture keeps its content, so that only what changed
/// since the last frame draws; the version counts the changes.
fn layer_texture(
    parts: &mut Parts<'_>,
    gles: &mut GlesRenderer,
    node: &chora_scene::Node,
    size: smithay::utils::Size<i32, smithay::utils::Buffer>,
    scale: f64,
) -> Option<(GlesTexture, u64)> {
    let inner = View {
        camera: Transform::IDENTITY,
        scale,
        size: (size.w, size.h).into(),
        near: mesh::NEAR,
        stepped: true,
    };
    let mut content = flat_elements(parts, gles, &node.id, &inner)?;
    settle_backdrops(parts.assets, &mut content, scale);
    let assets = &mut *parts.assets;
    let drawn = assets.layers.draw(
        (gles, &mut assets.stats),
        (&node.id, 0),
        (size, scale),
        &content,
    );
    let drawn = drawn
        .inspect_err(|err| tracing::warn!(%err, "a layer failed"))
        .ok()?;
    let pixels = node
        .props
        .size
        .map_or(scale, |[w, _]| f64::from(size.w) / w.max(f64::MIN_POSITIVE));
    let sources = node_textures(parts, gles, node, pixels);
    let assets = &mut *parts.assets;
    Some(mesh::apply_passes(
        gles,
        (
            &mut assets.layers,
            &mut assets.effect_errors,
            &mut assets.stats,
        ),
        node,
        drawn,
        (pixels, &sources),
    ))
}

/// The node textures of a shader node by the names of their uniforms: each
/// node that a uniform names, rendered on its own at its size, once for
/// all the nodes that read it. Inside a node texture, shader nodes get
/// none, so that shader nodes that read each other do not recurse.
fn node_textures(
    parts: &mut Parts<'_>,
    gles: &mut GlesRenderer,
    node: &chora_scene::Node,
    scale: f64,
) -> Vec<mesh::Source> {
    let mut out = Vec::new();
    if node.kind != NodeKind::Shader || parts.depth > 0 {
        return out;
    }
    for (name, uniform) in &node.props.uniforms {
        let source = match uniform {
            chora_scene::Uniform::Texture(source) => source,
            chora_scene::Uniform::Output(read) => {
                let reader = node.owner.as_deref();
                let faces = parts.assets.virtuals.iter();
                let frame = faces
                    .filter(|v| v.output().name() == read.output)
                    .find_map(|v| v.frame(reader))
                    .or_else(|| Some((parts.assets.blank(gles)?, 0)));
                out.extend(frame.map(|(texture, version)| (name.clone(), texture, version)));
                continue;
            }
            _ => continue,
        };
        let Some(logical) = source_size(parts, &source.node) else {
            continue;
        };
        let max = parts.assets.layers.max_side(gles);
        let scale = mesh::fit_scale(logical, scale, max);
        let Some(size) = mesh::pixel_size(logical, scale, max) else {
            continue;
        };
        parts.assets.layers.reader = Some(node.id.clone());
        let fits = parts.assets.layers.spend(&source.node, size, 1);
        if !fits {
            parts.assets.layers.reader = None;
            continue;
        }
        let view = View {
            camera: Transform::IDENTITY,
            scale,
            size: (size.w, size.h).into(),
            near: mesh::NEAR,
            stepped: true,
        };
        parts.depth += 1;
        let content = flat_elements(parts, gles, &source.node, &view);
        parts.depth -= 1;
        parts.assets.layers.reader = None;
        let Some(content) = content else { continue };
        let assets = &mut *parts.assets;
        let drawn = assets.layers.draw(
            (gles, &mut assets.stats),
            (&source.node, mesh::SOURCE_SLOT),
            (size, scale),
            &content,
        );
        if let Ok((texture, version)) = drawn {
            out.push((name.clone(), texture, version));
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
/// that is gone or hidden. The planes of a 3D context show sorted.
pub fn flat_elements<R: MeshRenderer>(
    parts: &mut Parts<'_>,
    renderer: &mut R,
    id: &str,
    view: &View,
) -> Option<Vec<SceneElement<R>>> {
    let scene = parts.scene;
    let list = scene.layer_list(id);
    let (first, rest) = list.split_first()?;
    let mut content = if first.node.is_context() {
        context_elements(parts, renderer, first, view)
    } else {
        elements(parts, renderer, rest, view)
    };
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
        // A shader node draws as a layer; outside of one it shows
        // nothing.
        NodeKind::Group | NodeKind::Shader => Vec::new(),
        NodeKind::Text => text::element(
            &parts.assets.texts,
            renderer,
            placed.node,
            &Placement {
                origin: offset,
                scale: (sx, sy),
                output_scale: view.scale,
                alpha,
            },
        )
        .into_iter()
        .collect(),
    }
}

/// Drops the buffers of nodes that are gone or of another kind.
fn prune_assets(parts: &mut Parts<'_>) {
    let scene = parts.scene;
    let kind_is = |id: &str, kind| scene.node(id).is_some_and(|n| n.kind == kind);
    let assets = &mut *parts.assets;
    assets.rects.retain(|id, _| kind_is(id, NodeKind::Rect));
    assets.cells.retain(|id, _| kind_is(id, NodeKind::Image));
    assets.texts.retain(|id| kind_is(id, NodeKind::Text));
    assets.layers.retain(|id| scene.node(id).is_some());
    assets
        .backdrops
        .retain(|id| scene.node(id).is_some_and(|n| n.props.backdrop.is_some()));
    assets.orders.retain(|id| scene.node(id).is_some());
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
        premultiplied(props.color?).map(|c| c as f32),
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

/// A color of the scene, which has straight alpha, as the renderer takes
/// it: with the red, green and blue multiplied by the alpha.
pub fn premultiplied([r, g, b, a]: [f64; 4]) -> [f64; 4] {
    [r * a, g * a, b * a, a]
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
pub(crate) fn location_and_scale(t: &Transform) -> Option<(Point<f64, Logical>, (f64, f64))> {
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
pub fn surface_under(state: &State, spot: &Spot) -> Option<Under> {
    // Layer and lock surfaces sit in output coordinates; this offset moves
    // them into scene coordinates, exactly for cameras without zoom.
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
    // The overlay of the output sits in its coordinates, as layer surfaces.
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
pub fn node_hit(state: &State, spot: &Spot) -> Option<(String, Point<f64, Logical>)> {
    if state.lock.is_some()
        || layers::surface_under(&spot.output, &layers::ABOVE, spot.local).is_some()
    {
        return None;
    }
    // The overlay of the output first, in its coordinates (D43).
    let overlay = state.scene.hit_list(&overlay_id(&spot.output.name()));
    let scene = state.scene.hit_list(ROOT);
    let found = [(&overlay, spot.local), (&scene, spot.scene)]
        .into_iter()
        .find_map(|(list, pos)| found_shown(state, spot, list, pos))?;
    (found.input == InputMode::Accept).then(|| (found.node.id.clone(), found.local))
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
fn scene_under(
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
enum Hit {
    /// A surface, and the point on it in its own units.
    Surface(WlSurface, Point<f64, Logical>),
    /// A rect: an area without a client.
    Area,
}

/// The top node of a list under a point, with what it hit.
struct Found<'a> {
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
fn found_shown<'a>(
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
fn found<'a>(
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

fn hit<'a>(
    state: &'a State,
    placed: &Placed<'a>,
    input: InputMode,
    pos: Point<f64, Logical>,
) -> Option<Found<'a>> {
    hit_cut(state, placed, input, (pos, mesh::NEAR))
}

/// As [`hit`], for a node that its list cuts at `near`: a 3D context
/// cuts at [`space::NEAR_W`].
fn hit_cut<'a>(
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
            // The point on the surface in its own units: Smithay gives the
            // client `pos - location`, which a scaled node must not scale.
            Hit::Surface(surface, local - (loc.to_f64() - geo))
        }
        NodeKind::Rect | NodeKind::Image | NodeKind::Shader => {
            let [w, h] = node_size(state, placed.node);
            if !((0.0..w).contains(&local.x) && (0.0..h).contains(&local.y)) {
                return None;
            }
            Hit::Area
        }
        // A group is empty and text takes no pointer input.
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
fn context_hit<'a>(
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
fn entry_hit<'a>(
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
const MAX_LOCAL: f64 = 1e6;

/// A point of a node with its coordinates cut to `MAX_LOCAL`.
fn clamp_local(point: Point<f64, Logical>) -> Point<f64, Logical> {
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
fn plane_hit<'a>(
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
fn node_point(transform: &Transform, pos: Point<f64, Logical>) -> Option<Point<f64, Logical>> {
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
fn local_in_list(
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
fn is_plane(placed: &Placed<'_>) -> bool {
    placed.node.props.size.is_some() && !placed.transform.is_plain()
}

/// Whether a point of the scene is inside the clip of a node with a
/// transform and a size.
fn inside(transform: &Transform, [w, h]: [f64; 2], pos: Point<f64, Logical>) -> bool {
    node_point(transform, pos).is_some_and(|p| (0.0..w).contains(&p.x) && (0.0..h).contains(&p.y))
}

/// What a point hits of a node with a mesh under a transform that is not
/// a plain 2D one: the triangles of its shape take the input where they
/// draw, cut at `near` as the draw cuts them.
fn layer_hit(placed: &Placed<'_>, near: f64, pos: Point<f64, Logical>) -> Option<Hit> {
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

/// The size of a rect or an image node; an image without a size has the
/// size of its file.
pub(crate) fn node_size(state: &State, node: &Node) -> [f64; 2] {
    let props = &node.props;
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
    let time = state.time.now().saturating_duration_since(state.start_time);
    for (id, window) in &state.windows {
        let on_output = visible.contains(id);
        if on_output {
            window.output_enter(output, Rectangle::from_size(window.bbox().size));
        } else {
            window.output_leave(output);
        }
        // Smithay sends every frame to a surface whose primary output is
        // this one, and applies the throttle only to the others.
        let primary = on_output.then(|| output.clone());
        window.send_frame(output, time, Some(Duration::from_secs(1)), |_, _| {
            primary.clone()
        });
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
    crate::offscreen::present_virtuals(state);
}

/// The surface nodes that an output shows: those whose scene rectangle
/// overlaps the scene area of the output.
pub(crate) fn shown_surfaces(state: &State, output: &Output) -> HashSet<String> {
    let screen = Rectangle::from_size(output_area(output).size);
    let shown = scene_rect(&state.camera(output), screen);
    let overlay = state.scene.overlay_draw_list(&output.name());
    // The overlay sits in output coordinates.
    let draw_list = state.scene.draw_list();
    let placed = draw_list.iter().map(|p| (p, shown));
    placed
        .chain(overlay.iter().map(|p| (p, screen.to_f64())))
        .filter(|(p, _)| p.node.kind == NodeKind::Surface)
        .filter(|(p, within)| {
            let window = state.windows.get(&p.node.id);
            window.is_some_and(|w| within.overlaps(window_rect(w, &p.transform)))
        })
        .map(|(p, _)| p.node.id.clone())
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
/// transform places its geometry origin and scales it; under a plane it is
/// the box of the projected corners of the geometry, which its texture
/// holds.
pub(crate) fn window_rect(window: &Window, transform: &Transform) -> Rectangle<f64, Logical> {
    let bbox = window.bbox().to_f64();
    let geo = window.geometry();
    if !transform.is_plain() {
        let size = [f64::from(geo.size.w), f64::from(geo.size.h)];
        return bounding_box(transform, size);
    }
    let geo = geo.loc.to_f64();
    let (x, y) = transform.translation_2d();
    let (sx, sy) = transform.scale_2d().unwrap_or((1.0, 1.0));
    Rectangle::new(
        (x + (bbox.loc.x - geo.x) * sx, y + (bbox.loc.y - geo.y) * sy).into(),
        (bbox.size.w * sx, bbox.size.h * sy).into(),
    )
}

/// The smallest axis-aligned rectangle that holds the box from `(0, 0)` to
/// `(w, h)` after the transform.
pub(crate) fn bounding_box(transform: &Transform, size: [f64; 2]) -> Rectangle<f64, Logical> {
    bounding_box_from(transform, size, mesh::NEAR)
}

/// As [`bounding_box`], with the box clipped at `w = near`.
pub(crate) fn bounding_box_from(
    transform: &Transform,
    [w, h]: [f64; 2],
    near: f64,
) -> Rectangle<f64, Logical> {
    polygon_box(transform, [[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]], near)
}

/// The smallest axis-aligned rectangle that holds the part of a quad in
/// front of the near plane `w = near` after the transform, as the renderer
/// clips it; a 3D context clips its planes at 1/64. The
/// corners go around the quad. A quad with no corner in front overlaps
/// nothing.
fn polygon_box(
    transform: &Transform,
    corners: [[f64; 2]; 4],
    near: f64,
) -> Rectangle<f64, Logical> {
    let m = transform.matrix();
    let polygon = corners.map(|[px, py]| {
        [
            m[0] * px + m[4] * py + m[12],
            m[1] * px + m[5] * py + m[13],
            m[3] * px + m[7] * py + m[15],
        ]
    });
    let mut bounds = mesh::Bounds::default();
    bounds.add_clipped_at(&polygon, near);
    bounds.rect()
}

/// The smallest axis-aligned rectangle that holds the points after the
/// transform. The points go through the projection of the renderer, so
/// that a matrix which divides by w moves them as the frame does; the
/// depth of mesh points stays out, as in `Transform::apply`. A point at
/// or behind `w = near`, the near plane of a context or the eye, is left
/// out, and a box of no point in front overlaps nothing.
pub(crate) fn projected_box(
    transform: &Transform,
    points: impl Iterator<Item = [f64; 2]>,
    near: f64,
) -> Rectangle<f64, Logical> {
    let m = transform.matrix();
    let mut bounds = mesh::Bounds::default();
    for [px, py] in points {
        bounds.add_point_at(
            [
                m[0] * px + m[4] * py + m[12],
                m[1] * px + m[5] * py + m[13],
                m[3] * px + m[7] * py + m[15],
            ],
            near,
        );
    }
    bounds.rect()
}

#[cfg(test)]
mod blur;
#[cfg(test)]
mod budget;
#[cfg(test)]
mod context;
#[cfg(test)]
mod cost;
#[cfg(test)]
mod hit_guard;
#[cfg(test)]
mod marble;
#[cfg(test)]
mod planes;
#[cfg(test)]
mod popups;
#[cfg(test)]
mod sampling;

#[cfg(test)]
mod tests {
    use chora_scene::Transform;

    use serde_json::json;

    use super::{location_and_scale, premultiplied, send_frames, surface_under};
    use crate::testing::{Core, wayland::Wayland};

    /// The mesh points of a layer wholly behind the eye, or of a context
    /// between the eye and its near plane, give an empty box.
    #[test]
    fn mesh_points_behind_the_eye_give_an_empty_box() {
        let mut behind = [0.0; 16];
        behind[0] = 1.0;
        behind[5] = 1.0;
        behind[10] = 1.0;
        behind[15] = -1.0;
        let points = [[0.0, 0.0], [100.0, 0.0], [0.0, 80.0]];
        let rect = super::projected_box(
            &Transform::from_matrix(behind),
            points.into_iter(),
            crate::mesh::NEAR,
        );
        assert_eq!(rect, smithay::utils::Rectangle::default());
        let mut near = behind;
        near[15] = 0.01;
        let rect = super::projected_box(
            &Transform::from_matrix(near),
            points.into_iter(),
            1.0 / 64.0,
        );
        assert_eq!(
            rect,
            smithay::utils::Rectangle::default(),
            "inside the near plane of a context"
        );
    }

    /// A window that no output shows gets a frame callback at most once a
    /// second, so that a hidden client stops drawing (#60).
    #[test]
    fn a_hidden_window_gets_no_frame_callback_each_frame() {
        let mut core = Core::new();
        let output = core.add_output("HDMI-1", (800, 600));
        let mut wayland = Wayland::connect(&mut core);
        let shown = wayland.toplevel(&mut core, "shown");
        let hidden = wayland.toplevel(&mut core, "hidden");
        wayland.attach(&mut core, &shown, (100, 100));
        wayland.attach(&mut core, &hidden, (100, 100));
        let mut wm = core.named("wm");
        let commands = json!({"commands": [{"attach": {"surface": "s1", "parent": "root"}}]});
        wm.ok(&mut core, "scene.apply", commands);
        for _ in 0..3 {
            wayland.frame(&mut core, &shown);
            wayland.frame(&mut core, &hidden);
            send_frames(&mut core.state, &output);
            wayland.roundtrip(&mut core);
        }
        assert_eq!(wayland.configures(&shown).frames, 3);
        assert!(wayland.configures(&hidden).frames <= 1);
    }

    #[test]
    fn a_client_gets_the_pointer_in_its_own_units_on_a_scaled_node() {
        let mut core = Core::new();
        core.add_output("HDMI-1", (800, 600));
        let mut wayland = Wayland::connect(&mut core);
        let window = wayland.toplevel(&mut core, "window");
        wayland.attach(&mut core, &window, (200, 200));
        let mut wm = core.named("wm");
        let commands = json!({"commands": [
            {"create": {"id": "thumb", "kind": "group", "parent": "root",
                        "props": {"transform": {"translate": [100, 100], "scale": 0.5}}}},
            {"attach": {"surface": "s1", "parent": "thumb"}},
        ]});
        wm.ok(&mut core, "scene.apply", commands);
        let spot = core.state.spot((150.0, 150.0).into()).expect("a spot");
        let under = surface_under(&core.state, &spot).expect("the window");
        assert_eq!(spot.scene - under.location, (100.0, 100.0).into());
    }

    #[test]
    fn a_color_with_alpha_is_premultiplied_for_the_renderer() {
        assert_eq!(premultiplied([1.0, 1.0, 1.0, 0.5]), [0.5, 0.5, 0.5, 0.5]);
        assert_eq!(premultiplied([1.0, 0.0, 0.5, 0.0]), [0.0; 4]);
        assert_eq!(premultiplied([0.2, 0.4, 0.8, 1.0]), [0.2, 0.4, 0.8, 1.0]);
    }

    #[test]
    fn mirrored_and_collapsed_nodes_are_not_drawn() {
        assert!(location_and_scale(&Transform::scale(-1.0, 1.0, 1.0)).is_none());
        assert!(location_and_scale(&Transform::scale(1.0, 0.0, 1.0)).is_none());
        assert!(location_and_scale(&Transform::rotate_z(30.0)).is_none());
        assert!(location_and_scale(&Transform::scale(2.0, 2.0, 1.0)).is_some());
    }
}
