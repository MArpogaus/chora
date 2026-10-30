//! Layers and meshes (D19).
//!
//! A node with a size and a mesh, or with a transform that is not a plain
//! 2D translation and scale, is a layer: it renders with its descendants
//! into a texture of its size first. The core then draws the texture
//! through the mesh of the node, or a plain quad, with the full 4x4
//! transform of the node and the camera. So perspective and deformations,
//! for example of wobbly windows, a magic lamp minimize or a desktop cube,
//! need nothing from the clients.
//!
//! The vertices go to the GPU in homogeneous coordinates, so that the
//! texture maps correctly under perspective. Nodes in a layer have no depth
//! test; the order of the siblings decides what is in front.
//!
//! A layer goes through its passes first: the built-in shape of a rounded
//! rect with a border, the shader of a shader node, one pass per effect,
//! and the built-in shape of a clip. The raw GL calls are in [`crate::gl`].

use std::{
    collections::{BTreeMap, HashMap},
    time::{Duration, Instant},
};

use crate::gl;

use chora_scene::{Mesh, Node, NodeKind, Scene, Transform, Uniform};
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            Bind, Color32F, Frame, ImportAll, ImportMem, Offscreen, Renderer, RendererSuper,
            Texture,
            damage::OutputDamageTracker,
            element::{Element, Id, Kind, RenderElement},
            gles::{GlesError, GlesFrame, GlesRenderer, GlesTexture},
            utils::{CommitCounter, DamageSet, OpaqueRegions},
        },
    },
    utils::{Buffer as BufferCoords, Physical, Point, Rectangle, Scale, Size},
};

/// The largest side of a layer texture, in pixels.
const MAX_SIDE: f64 = 4096.0;
/// The most pixels of layer textures in one frame of an output.
const FRAME_PIXELS: u64 = 64 << 20;
/// How long an unused layer texture stays.
const UNUSED_FOR: Duration = Duration::from_secs(2);

/// A renderer that can draw meshes: the GLES renderer of the nested
/// backend, and the multi-GPU renderer of the tty backend, which renders
/// on the GLES renderer of its render GPU.
pub trait MeshRenderer:
    Renderer + ImportAll + ImportMem + RendererSuper<TextureId: Clone + Send + 'static>
{
    /// The GLES renderer that renders the layers.
    fn gles(&mut self) -> &mut GlesRenderer;
    /// The GLES frame of a frame.
    fn gles_frame<'a, 'f, 'b>(frame: &'a mut Self::Frame<'f, 'b>) -> &'a mut GlesFrame<'f, 'b>
    where
        'b: 'f,
        Self: 'f;
    /// An error of the GLES renderer as an error of this renderer.
    fn from_gles(err: GlesError) -> Self::Error;
}

impl MeshRenderer for GlesRenderer {
    fn gles(&mut self) -> &mut GlesRenderer {
        self
    }

    fn gles_frame<'a, 'f, 'b>(frame: &'a mut GlesFrame<'f, 'b>) -> &'a mut GlesFrame<'f, 'b>
    where
        'b: 'f,
    {
        frame
    }

    fn from_gles(err: GlesError) -> GlesError {
        err
    }
}

/// The textures and the ids of the layers by node, kept between frames:
/// slot 0 holds the content of a layer, slots 1 and 2 the passes of its
/// effects.
#[derive(Default)]
pub struct Layers {
    /// By node, slot and size, so that outputs of other scales keep their
    /// own; with the time of the last use.
    textures: HashMap<(String, usize, i32, i32), (GlesTexture, Instant)>,
    ids: HashMap<String, (Id, CommitCounter)>,
    /// The pixels of layer textures in the current frame.
    pixels: u64,
    /// The budget ran out in this frame, and the log tells it once.
    told: bool,
    /// The nodes whose layers the budget left out of the current frame.
    pub left_out: Vec<String>,
}

impl Layers {
    /// Drops the layers of nodes that are gone or no layers any more, and
    /// textures that no frame used for a while; starts the budget of a new
    /// frame.
    pub fn retain(&mut self, mut keep: impl FnMut(&str) -> bool) {
        let now = Instant::now();
        self.textures
            .retain(|(id, ..), (_, used)| keep(id) && now - *used < UNUSED_FOR);
        self.ids.retain(|id, _| keep(id));
        self.new_budget();
    }

    /// Starts the budget of a new frame or capture.
    pub fn new_budget(&mut self) {
        self.pixels = 0;
        self.left_out.clear();
    }

    /// Takes pixels of the budget of the frame for the textures of the layer
    /// of a node. Returns `false` when they do not fit, and notes the node.
    pub fn spend(&mut self, id: &str, size: Size<i32, BufferCoords>, textures: usize) -> bool {
        let area = u64::try_from(size.w * size.h).unwrap_or(u64::MAX);
        let pixels = area.saturating_mul(textures as u64);
        if self.pixels.saturating_add(pixels) > FRAME_PIXELS {
            self.leave_out(id);
            return false;
        }
        self.pixels += pixels;
        true
    }

    /// Notes a node whose layer stays out of the frame; the log tells it
    /// once.
    fn leave_out(&mut self, id: &str) {
        if !std::mem::replace(&mut self.told, true) {
            tracing::warn!("layers exceed {FRAME_PIXELS} pixels in a frame; some stay out");
        }
        if !self.left_out.iter().any(|n| n == id) {
            self.left_out.push(id.to_owned());
        }
    }

    /// A texture of a slot of a layer of a size, the one of an earlier
    /// frame if there is one.
    pub fn texture(
        &mut self,
        gles: &mut GlesRenderer,
        node: &str,
        slot: usize,
        size: Size<i32, BufferCoords>,
    ) -> Result<GlesTexture, GlesError> {
        let key = (node.to_string(), slot, size.w, size.h);
        let now = Instant::now();
        if let Some((texture, used)) = self.textures.get_mut(&key) {
            *used = now;
            return Ok(texture.clone());
        }
        let texture: GlesTexture = gles.create_buffer(Fourcc::Abgr8888, size)?;
        self.textures.insert(key, (texture.clone(), now));
        Ok(texture)
    }

    /// The id of the element of a layer, and a new commit: a layer counts
    /// as changed in each frame.
    fn next(&mut self, node: &str) -> (Id, CommitCounter) {
        let entry = self
            .ids
            .entry(node.into())
            .or_insert_with(|| (Id::new(), CommitCounter::default()));
        entry.1.increment();
        entry.clone()
    }
}

/// Whether the layers of a node and its descendants need more pixels at a
/// scale than the budget of a frame, as [`Layers::spend`] counts them.
pub fn over_budget(scene: &Scene, id: &str, scale: f64) -> bool {
    subtree_pixels(scene, id, scale) > FRAME_PIXELS
}

fn subtree_pixels(scene: &Scene, id: &str, scale: f64) -> u64 {
    let Some(node) = scene.node(id) else {
        return 0;
    };
    let own = match texture_size(node, scale) {
        Some(size) if node.is_layer() => {
            let textures = 1 + passes(node, scale).len().min(2);
            u64::try_from(size.w * size.h).unwrap_or(u64::MAX) * textures as u64
        }
        _ => 0,
    };
    let children = node
        .children
        .iter()
        .map(|c| subtree_pixels(scene, c, scale));
    own + children.sum::<u64>()
}

/// The pixel size of the texture of a layer node at an output scale.
pub fn texture_size(node: &Node, scale: f64) -> Option<Size<i32, BufferCoords>> {
    pixel_size(node.props.size?, scale)
}

/// The pixel size of a texture of a logical size, at most 4096 a side.
pub fn pixel_size([w, h]: [f64; 2], scale: f64) -> Option<Size<i32, BufferCoords>> {
    #[allow(clippy::cast_possible_truncation)] // clamped
    let side = |v: f64| (v * scale).round().clamp(1.0, MAX_SIDE) as i32;
    (w > 0.0 && h > 0.0).then(|| Size::from((side(w), side(h))))
}

/// Renders elements into a layer texture, transparent where they leave it
/// empty.
pub fn render_into<E>(
    gles: &mut GlesRenderer,
    texture: &mut GlesTexture,
    scale: f64,
    elements: &[E],
) -> Result<(), String>
where
    E: RenderElement<GlesRenderer>,
{
    let size = Texture::size(texture);
    let mut target = gles.bind(texture).map_err(|e| e.to_string())?;
    let mut tracker =
        OutputDamageTracker::new((size.w, size.h), scale, smithay::utils::Transform::Normal);
    tracker
        .render_output(gles, &mut target, 0, elements, [0.0; 4])
        .map(drop)
        .map_err(|e| format!("{e:?}"))
}

/// A vertex of a mesh: the position in physical output coordinates in
/// homogeneous form (x, y, w), and the texture coordinates.
type Vertex = ([f32; 3], [f32; 2]);

/// Where a layer shows: the transform to logical output coordinates and
/// the output scale.
pub struct Placing<'a> {
    pub transform: &'a Transform,
    pub scale: f64,
}

/// A layer texture drawn through a mesh.
pub struct MeshElement {
    id: Id,
    commit: CommitCounter,
    texture: GlesTexture,
    vertices: Vec<Vertex>,
    /// Three vertex indices per triangle.
    indices: Vec<u16>,
    geometry: Rectangle<i32, Physical>,
    alpha: f32,
}

/// Where the mesh of a layer lands on an output.
pub struct Shape {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    /// The bounding box in physical output coordinates.
    pub bounds: Rectangle<i32, Physical>,
}

/// The shape of a layer node: its mesh, or a quad, with a transform to
/// logical output coordinates. `None` when no triangle is in front of the viewer.
pub fn shape(node: &Node, at: &Placing<'_>) -> Option<Shape> {
    let size = node.props.size?;
    let flat;
    let mesh = if let Some(mesh) = &node.props.mesh {
        mesh
    } else {
        flat = Mesh::flat(1, 1, size);
        &flat
    };
    project(mesh, at.transform, at.scale)
}

impl MeshElement {
    /// The element of a layer node: its texture through its shape.
    pub fn new(
        layers: &mut Layers,
        node: &Node,
        texture: GlesTexture,
        shape: Shape,
        alpha: f32,
    ) -> Self {
        let (id, commit) = layers.next(&node.id);
        Self {
            id,
            commit,
            texture,
            vertices: shape.vertices,
            indices: shape.indices,
            geometry: shape.bounds,
            alpha,
        }
    }
}

/// The vertices of a mesh in physical output coordinates, with texture
/// coordinates, the triangles that show, and their bounding box. A
/// triangle with a point behind the viewer does not show. The
/// transform projects the points with their depth, and the clip position
/// keeps the w of each point, so the texture maps perspective-correct.
/// `None` when no triangle shows, or when the mesh has more points
/// than 16-bit indices reach.
#[allow(clippy::cast_possible_truncation)] // GL takes 32-bit floats
fn project(mesh: &Mesh, transform: &Transform, scale: f64) -> Option<Shape> {
    let m = transform.matrix();
    let (cols, rows) = (f64::from(mesh.cols), f64::from(mesh.rows));
    let stride = mesh.cols as usize + 1;
    let mut vertices = Vec::with_capacity(mesh.points.len());
    let mut screen = Vec::with_capacity(mesh.points.len());
    for (index, [px, py]) in mesh.points.iter().enumerate() {
        let pz = mesh.z(index);
        let hx = m[0] * px + m[4] * py + m[8] * pz + m[12];
        let hy = m[1] * px + m[5] * py + m[9] * pz + m[13];
        let hw = m[3] * px + m[7] * py + m[11] * pz + m[15];
        screen.push((hw > NEAR).then(|| [hx * scale / hw, hy * scale / hw]));
        #[allow(clippy::cast_precision_loss)] // at most 65 points per side
        let uv = [
            ((index % stride) as f64 / cols) as f32,
            ((index / stride) as f64 / rows) as f32,
        ];
        vertices.push(([(hx * scale) as f32, (hy * scale) as f32, hw as f32], uv));
    }
    u16::try_from(vertices.len()).ok()?;
    let mut indices = triangles(mesh.cols, mesh.rows);
    let shows = |i: &u16| screen.get(usize::from(*i)).copied().flatten();
    let kept: Vec<u16> = indices
        .chunks(3)
        .filter(|t| t.iter().all(|i| shows(i).is_some()))
        .flatten()
        .copied()
        .collect();
    if kept.is_empty() {
        return None;
    }
    let (mut min, mut max) = ([f64::MAX; 2], [f64::MIN; 2]);
    for [x, y] in kept.iter().filter_map(shows) {
        min = [min[0].min(x), min[1].min(y)];
        max = [max[0].max(x), max[1].max(y)];
    }
    indices = kept;
    // A point close to the eye lands far away; the box stays in i32.
    let clamp = |v: f64| v.clamp(-FAR, FAR) as i32;
    let loc = Point::from((clamp(min[0].floor()), clamp(min[1].floor())));
    let end = Point::<i32, Physical>::from((clamp(max[0].ceil()), clamp(max[1].ceil())));
    let size = Size::from(((end.x - loc.x).max(1), (end.y - loc.y).max(1)));
    Some(Shape {
        vertices,
        indices,
        bounds: Rectangle::new(loc, size),
    })
}

/// The least w of a point that shows.
const NEAR: f64 = 1e-6;
/// The farthest physical coordinate of the box of a mesh.
const FAR: f64 = (1 << 24) as f64;

/// Two triangles per cell of a grid.
fn triangles(cols: u32, rows: u32) -> Vec<u16> {
    let stride = cols + 1;
    let mut out = Vec::with_capacity((cols * rows * 6) as usize);
    for r in 0..rows {
        for c in 0..cols {
            let a = r * stride + c;
            let (b, d, e) = (a + 1, a + stride, a + stride + 1);
            for i in [a, b, d, b, e, d] {
                out.push(u16::try_from(i).unwrap_or(u16::MAX));
            }
        }
    }
    out
}

impl Element for MeshElement {
    fn id(&self) -> &Id {
        &self.id
    }

    fn current_commit(&self) -> CommitCounter {
        self.commit
    }

    fn src(&self) -> Rectangle<f64, BufferCoords> {
        let size = Texture::size(&self.texture);
        Rectangle::from_size(size.to_f64())
    }

    fn geometry(&self, _scale: Scale<f64>) -> Rectangle<i32, Physical> {
        self.geometry
    }

    fn damage_since(
        &self,
        scale: Scale<f64>,
        _commit: Option<CommitCounter>,
    ) -> DamageSet<i32, Physical> {
        DamageSet::from_slice(&[Rectangle::from_size(self.geometry(scale).size)])
    }

    fn opaque_regions(&self, _scale: Scale<f64>) -> OpaqueRegions<i32, Physical> {
        OpaqueRegions::default()
    }

    fn alpha(&self) -> f32 {
        self.alpha
    }

    fn kind(&self) -> Kind {
        Kind::Unspecified
    }
}

impl<R: MeshRenderer> RenderElement<R> for MeshElement {
    fn draw(
        &self,
        frame: &mut R::Frame<'_, '_>,
        _src: Rectangle<f64, BufferCoords>,
        _dst: Rectangle<i32, Physical>,
        _damage: &[Rectangle<i32, Physical>],
        _opaque_regions: &[Rectangle<i32, Physical>],
        _cache: Option<&smithay::utils::user_data::UserDataMap>,
    ) -> Result<(), R::Error> {
        let frame = R::gles_frame(frame);
        let program = gl::plain_program(frame).map_err(R::from_gles)?;
        let (clip, uv) = clip_space(&self.vertices, frame.projection());
        let call = gl::Draw {
            clip: &clip,
            uv: &uv,
            indices: &self.indices,
            texture: self.texture.tex_id(),
            alpha: self.alpha,
            size: float_size(&self.texture),
            uniforms: None,
            textures: &[],
        };
        gl::draw(frame, &program, &call).map_err(R::from_gles)
    }
}

/// The clip positions and the texture coordinates of vertices for the
/// projection of a frame. The 3x3 projection maps physical (x, y, 1) to
/// clip space; for (x, y, w) it gives the clip position before the divide.
fn clip_space(vertices: &[Vertex], p: &[f32; 9]) -> (Vec<f32>, Vec<f32>) {
    let clip = vertices
        .iter()
        .flat_map(|([x, y, w], _)| {
            [
                p[0] * x + p[3] * y + p[6] * w,
                p[1] * x + p[4] * y + p[7] * w,
                0.0,
                *w,
            ]
        })
        .collect();
    let uv = vertices.iter().flat_map(|(_, uv)| *uv).collect();
    (clip, uv)
}

/// The size of a texture as 32-bit floats.
#[allow(clippy::cast_precision_loss)] // at most 8192
fn float_size(texture: &GlesTexture) -> [f32; 2] {
    let size = Texture::size(texture);
    [size.w as f32, size.h as f32]
}

/// One pass of a layer texture through a shader.
pub enum Pass<'a> {
    /// The built-in shape: rounded corners and a border, in pixels of the
    /// texture, the border color premultiplied.
    Shape {
        radius: f64,
        border: Option<(f64, [f64; 4])>,
    },
    /// A shader file with its uniforms; `textures` tells whether the pass
    /// reads the node textures, as the shader of a shader node does.
    File {
        shader: &'a str,
        uniforms: &'a BTreeMap<String, Uniform>,
        textures: bool,
    },
}

/// The passes of a layer, in order: the shape of a rounded rect, the
/// shader of a shader node, the effects, and the clip, so that nothing
/// shows outside the clip. `scale` is the number of texture pixels per
/// logical pixel.
pub fn passes(node: &Node, scale: f64) -> Vec<Pass<'_>> {
    let props = &node.props;
    let mut out = Vec::new();
    if node.kind == NodeKind::Rect && (props.radius > 0.0 || props.border.is_some()) {
        let border = props
            .border
            .map(|b| (b.width * scale, crate::render::premultiplied(b.color)));
        out.push(Pass::Shape {
            radius: props.radius * scale,
            border,
        });
    }
    if node.kind == NodeKind::Shader
        && let Some(shader) = &props.shader
    {
        out.push(Pass::File {
            shader,
            uniforms: &props.uniforms,
            textures: true,
        });
    }
    out.extend(props.effects.iter().map(|e| Pass::File {
        shader: &e.shader,
        uniforms: &e.uniforms,
        textures: false,
    }));
    if let Some(clip) = props.clip {
        out.push(Pass::Shape {
            radius: clip.radius * scale,
            border: None,
        });
    }
    out
}

/// Runs the passes of a layer node over its texture and returns the
/// texture of the last pass. A pass whose shader fails is left out; its
/// error goes to `errors`. `textures` holds the node textures of a shader
/// node by the names of their samplers.
pub fn apply_passes(
    gles: &mut GlesRenderer,
    (layers, errors): (&mut Layers, &mut Vec<String>),
    node: &Node,
    mut texture: GlesTexture,
    (scale, textures): (f64, &[(String, GlesTexture)]),
) -> GlesTexture {
    // The slot of the current texture: 0 for the content, then the two
    // textures of passes take turns, so that no pass reads its own target.
    let mut slot = 0;
    for pass in passes(node, scale) {
        let Some((program, uniforms, extra)) = program_of(gles, &pass, textures, errors) else {
            continue;
        };
        let size = Texture::size(&texture);
        let next = if slot == 1 { 2 } else { 1 };
        let Ok(mut target) = layers.texture(gles, &node.id, next, size) else {
            continue;
        };
        match draw_pass(gles, &program, &texture, &mut target, (&uniforms, &extra)) {
            Ok(()) => {
                texture = target;
                slot = next;
            }
            Err(err) => errors.push(err),
        }
    }
    texture
}

/// A program with its uniforms and its node textures.
type Compiled = (gl::Program, BTreeMap<String, Uniform>, Vec<(String, u32)>);

/// The program of a pass, its uniforms and its node textures.
fn program_of(
    gles: &mut GlesRenderer,
    pass: &Pass<'_>,
    textures: &[(String, GlesTexture)],
    errors: &mut Vec<String>,
) -> Option<Compiled> {
    match pass {
        Pass::Shape { radius, border } => {
            let program = gl::shape_program(gles).map_err(|e| errors.push(e)).ok()?;
            let (width, color) = border.unwrap_or((0.0, [0.0; 4]));
            let uniforms = BTreeMap::from([
                ("radius".to_string(), Uniform::Float(*radius)),
                ("border_width".to_string(), Uniform::Float(width)),
                ("border_color".to_string(), Uniform::Vector(color.to_vec())),
            ]);
            Some((program, uniforms, Vec::new()))
        }
        Pass::File {
            shader,
            uniforms,
            textures: reads,
        } => {
            let (program, error) = gl::effect_program(gles, shader);
            errors.extend(error);
            let extra = if *reads {
                textures
                    .iter()
                    .map(|(n, t)| (n.clone(), t.tex_id()))
                    .collect()
            } else {
                Vec::new()
            };
            Some((program?, (*uniforms).clone(), extra))
        }
    }
}

/// Draws a texture through the shader of a pass into another one.
fn draw_pass(
    gles: &mut GlesRenderer,
    program: &gl::Program,
    source: &GlesTexture,
    target: &mut GlesTexture,
    (uniforms, textures): (&BTreeMap<String, Uniform>, &[(String, u32)]),
) -> Result<(), String> {
    let size = Texture::size(target);
    let mut framebuffer = gles.bind(target).map_err(|e| e.to_string())?;
    let mut frame = gles
        .render(
            &mut framebuffer,
            (size.w, size.h).into(),
            smithay::utils::Transform::Normal,
        )
        .map_err(|e| e.to_string())?;
    let all = Rectangle::from_size((size.w, size.h).into());
    frame
        .clear(Color32F::TRANSPARENT, &[all])
        .map_err(|e| e.to_string())?;
    let quad = Mesh::flat(1, 1, [f64::from(size.w), f64::from(size.h)]);
    let vertices = project(&quad, &Transform::IDENTITY, 1.0)
        .ok_or("an empty pass")?
        .vertices;
    let (clip, uv) = clip_space(&vertices, frame.projection());
    let call = gl::Draw {
        clip: &clip,
        uv: &uv,
        indices: &triangles(1, 1),
        texture: source.tex_id(),
        alpha: 1.0,
        size: float_size(source),
        uniforms: Some(uniforms),
        textures,
    };
    gl::draw(&mut frame, program, &call).map_err(|e| e.to_string())?;
    frame.finish().map(drop).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_grid_has_two_triangles_per_cell() {
        let indices = triangles(2, 1);
        assert_eq!(indices.len(), 12);
        assert_eq!(&indices[..6], &[0, 1, 3, 1, 4, 3]);
    }

    #[test]
    fn a_point_behind_the_viewer_hides_only_its_triangles() {
        let mesh = Mesh::flat(1, 1, [10.0, 10.0]);
        let mut m = *Transform::IDENTITY.matrix();
        m[15] = -1.0;
        assert!(project(&mesh, &Transform::from_matrix(m), 1.0).is_none());
        let shape = project(&mesh, &Transform::IDENTITY, 2.0).unwrap();
        assert_eq!((shape.vertices.len(), shape.indices.len()), (4, 6));
        assert_eq!(shape.bounds.size, (20, 20).into());
        // Two cells; the depth puts the right column behind the eye.
        let mut mesh = Mesh::flat(2, 1, [20.0, 10.0]);
        mesh.depth = vec![0.0, 0.0, 2.0, 0.0, 0.0, 2.0];
        let mut m = *Transform::IDENTITY.matrix();
        m[11] = -1.0;
        let shape = project(&mesh, &Transform::from_matrix(m), 1.0).unwrap();
        assert_eq!(shape.indices, [0, 1, 3, 1, 4, 3]);
        assert_eq!(shape.bounds.size, (10, 10).into());
    }
}
