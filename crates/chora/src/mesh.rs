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
    collections::{BTreeMap, HashMap, hash_map::Entry},
    time::{Duration, Instant},
};

use crate::{gl, render::RenderStats};

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

/// The largest side of a layer texture, in pixels, when the GL takes it:
/// two textures of a larger side would pass the budget of a frame.
const MAX_SIDE: f64 = 4096.0;
/// The most pixels of layer textures in one frame of an output.
const FRAME_PIXELS: u64 = 64 << 20;
/// The slot of the texture of a node as the shaders of other nodes read it.
pub const SOURCE_SLOT: usize = 3;
/// The most bytes of layer textures that stay between frames.
const MAX_BYTES: u64 = 512 << 20;
/// The most steps of a power of two that the texture of a plane shrinks.
const MAX_STEPS: i32 = 4;
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

/// A texture of a layer with the damage tracker of its content, and the
/// time of its last use.
struct Slot {
    texture: GlesTexture,
    used: Instant,
    /// The tracker of the renders into the texture, with the scale that it
    /// renders at; the texture keeps what the last render drew.
    tracker: Option<(u64, OutputDamageTracker)>,
    /// How many renders drew into the texture.
    version: u64,
}

/// How a mesh element looked in the last frame, so that its commit counts
/// only what changed.
struct Seen {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    geometry: Rectangle<i32, Physical>,
    alpha: f32,
    texture: u32,
    version: u64,
}

/// The id of a mesh element and what it showed last.
struct Tracked {
    id: Id,
    commit: CommitCounter,
    seen: Option<Seen>,
}

/// The last result of the passes of a layer.
struct Passed {
    /// The size of the texture that the passes ran over.
    size: (i32, i32),
    /// A hash of the inputs of the passes.
    signature: u64,
    /// The slot of the texture of the result.
    slot: usize,
    /// The version of the result: it changes with each run of the passes.
    version: u64,
    /// How many times the passes ran.
    runs: u64,
    /// Whether the same inputs give the same result again.
    reusable: bool,
}

/// The textures and the ids of the layers by node, kept between frames:
/// slot 0 holds the content of a layer, slots 1 and 2 the passes of its
/// effects, slot 3 the node texture of the node for the shaders that read
/// it.
#[derive(Default)]
pub struct Layers {
    /// By node, slot and size, so that outputs of other scales keep their
    /// own.
    textures: HashMap<(String, usize, i32, i32), Slot>,
    /// By node and piece: 0 for a whole layer, the others for the pieces
    /// of a plane of a 3D context.
    ids: HashMap<(String, String, usize), Tracked>,
    /// By node: what made the last result of the passes of a layer.
    passed: HashMap<(String, String), Passed>,
    /// The most bytes of textures that [`Self::retain`] keeps.
    cap: Option<u64>,
    /// By node: the step of the texture scale of a plane, as a power of
    /// two below the output scale.
    steps: HashMap<(String, String), i32>,
    /// The view that renders now: an output, or the capture of one. The
    /// ids, the results of passes and the steps are kept per view, so that
    /// two outputs of other scales or cameras do not flip them.
    view: String,
    /// The pixels of layer textures in the current frame.
    pixels: u64,
    /// The budget ran out in this frame, and the log tells it once.
    told: bool,
    /// The nodes whose layers the budget left out of the current frame:
    /// the node that draws, also for a node texture that a shader node
    /// reads or a layer inside it, so that the source node itself, which
    /// draws, keeps its input.
    pub left_out: Vec<String>,
    /// Those of `left_out` that draw nothing: not a shader node that only
    /// misses a node texture.
    pub dark: Vec<String>,
    /// The shader node whose node texture renders now, which pays for the
    /// layers inside it.
    pub reader: Option<String>,
    /// The largest side of a layer texture on this GL, once known.
    max_side: Option<f64>,
    /// The last version that a render gave a texture. One counter for all
    /// slots, so that a texture that the cap evicted and made again never
    /// repeats the version of the texture before it.
    versions: u64,
}

impl Layers {
    /// Drops the layers of nodes that are gone, and textures that no frame
    /// used for a while; starts the budget of a new frame.
    pub fn retain(&mut self, mut keep: impl FnMut(&str) -> bool) {
        let now = Instant::now();
        self.textures
            .retain(|(id, ..), slot| keep(id) && now - slot.used < UNUSED_FOR);
        self.evict_over_cap();
        self.ids.retain(|(_, id, _), _| keep(id));
        self.passed.retain(|(_, id), _| keep(id));
        self.steps.retain(|(_, id), _| keep(id));
        self.new_budget();
    }

    /// Drops the textures that no frame used for the longest time until
    /// the rest fits the cap.
    fn evict_over_cap(&mut self) {
        let cost = |key: &(String, usize, i32, i32)| u64::try_from(key.2 * key.3).unwrap_or(0) * 4;
        let mut total: u64 = self.textures.keys().map(cost).sum();
        let cap = self.cap.unwrap_or(MAX_BYTES);
        if total <= cap {
            return;
        }
        let mut oldest: Vec<_> = self
            .textures
            .iter()
            .map(|(key, slot)| (slot.used, key.clone()))
            .collect();
        oldest.sort();
        for (_, key) in oldest {
            if total <= cap {
                break;
            }
            total -= cost(&key);
            self.textures.remove(&key);
        }
    }

    /// Sets the cap of [`Self::retain`] on the bytes of textures.
    #[cfg(test)]
    pub fn set_cap(&mut self, bytes: u64) {
        self.cap = Some(bytes);
    }

    /// The sizes of the textures of a node.
    #[cfg(test)]
    pub fn sizes(&self, node: &str) -> Vec<(i32, i32)> {
        let mut sizes: Vec<_> = self
            .textures
            .keys()
            .filter(|key| key.0 == node)
            .map(|key| (key.2, key.3))
            .collect();
        sizes.sort_unstable();
        sizes.dedup();
        sizes
    }

    /// How many results of passes and textures the layers keep.
    #[cfg(test)]
    pub fn kept(&self) -> (usize, usize) {
        (self.passed.len(), self.textures.len())
    }

    /// The largest side of a layer texture: `MAX_SIDE`, or less where
    /// `GL_MAX_TEXTURE_SIZE` is less.
    pub fn max_side(&mut self, gles: &mut GlesRenderer) -> f64 {
        *self
            .max_side
            .get_or_insert_with(|| side_limit(gl::max_texture_size(gles)))
    }

    /// Names the view that renders next: an output, or the capture of one.
    pub fn set_view(&mut self, view: String) {
        self.view = view;
    }

    /// Starts the budget of a new frame or capture.
    pub fn new_budget(&mut self) {
        self.pixels = 0;
        self.left_out.clear();
        self.dark.clear();
    }

    /// Takes pixels of the budget of the frame for the textures of the layer
    /// of a node. Returns `false` when they do not fit, and notes the node,
    /// or the shader node that reads it, which still draws.
    pub fn spend(&mut self, id: &str, size: Size<i32, BufferCoords>, textures: usize) -> bool {
        let area = u64::try_from(size.w * size.h).unwrap_or(u64::MAX);
        let pixels = area.saturating_mul(textures as u64);
        if self.pixels.saturating_add(pixels) > FRAME_PIXELS {
            if let Some(reader) = self.reader.clone() {
                self.leave_out(&reader);
            } else {
                self.leave_out(id);
                self.dark.push(id.to_owned());
            }
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

    /// The factor of the texture scale of a plane that shows `ratio` times
    /// its size: the least power of two of at least the ratio, at most 1
    /// and at least 2 to the power of `-MAX_STEPS`. A plane keeps its step
    /// while the wanted step is at most one below it and shrinks when it
    /// is two below, so that a walk back and forth around a limit
    /// allocates nothing. `None`, for a plane without a ratio, gives 1 and
    /// forgets the step.
    pub fn step(&mut self, node: &str, ratio: Option<f64>) -> f64 {
        let Some(ratio) = ratio.filter(|r| *r > 0.0) else {
            self.steps.remove(&(self.view.clone(), node.to_owned()));
            return 1.0;
        };
        #[allow(clippy::cast_possible_truncation)]
        let wanted = (ratio.log2().ceil() as i32).clamp(-MAX_STEPS, 0);
        let key = (self.view.clone(), node.to_owned());
        let kept = self.steps.get(&key).copied();
        let step = kept.map_or(wanted, |kept| {
            if wanted >= kept - 1 {
                wanted.max(kept)
            } else {
                wanted
            }
        });
        self.steps.insert(key, step);
        2.0_f64.powi(step)
    }

    /// The slot of a layer of a size, a new one when no frame made it yet.
    fn slot(
        &mut self,
        (gles, stats): (&mut GlesRenderer, &mut RenderStats),
        key: (&str, usize, Size<i32, BufferCoords>),
    ) -> Result<&mut Slot, GlesError> {
        let (node, slot, size) = key;
        let key = (node.to_string(), slot, size.w, size.h);
        let slot = match self.textures.entry(key) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let texture: GlesTexture = gles.create_buffer(Fourcc::Abgr8888, size)?;
                stats.textures_allocated += 1;
                entry.insert(Slot {
                    texture,
                    used: Instant::now(),
                    tracker: None,
                    version: 0,
                })
            }
        };
        slot.used = Instant::now();
        Ok(slot)
    }

    /// A texture of a slot of a layer of a size, the one of an earlier
    /// frame if there is one.
    pub fn texture(
        &mut self,
        gpu: (&mut GlesRenderer, &mut RenderStats),
        node: &str,
        slot: usize,
        size: Size<i32, BufferCoords>,
    ) -> Result<GlesTexture, GlesError> {
        Ok(self.slot(gpu, (node, slot, size))?.texture.clone())
    }

    /// The texture of a slot that an earlier frame made, without making
    /// one.
    fn existing(
        &mut self,
        node: &str,
        slot: usize,
        size: Size<i32, BufferCoords>,
    ) -> Option<GlesTexture> {
        let slot = self
            .textures
            .get_mut(&(node.to_string(), slot, size.w, size.h))?;
        slot.used = Instant::now();
        Some(slot.texture.clone())
    }

    /// Renders elements into the texture of a slot of a layer, which
    /// keeps what the last render drew: the damage tracker of the slot
    /// draws only what changed since. Returns the texture and its version,
    /// which counts the renders that drew into it.
    pub fn draw<E>(
        &mut self,
        (gles, stats): (&mut GlesRenderer, &mut RenderStats),
        (node, slot): (&str, usize),
        (size, scale): (Size<i32, BufferCoords>, f64),
        elements: &[E],
    ) -> Result<(GlesTexture, u64), String>
    where
        E: RenderElement<GlesRenderer>,
    {
        let next = self.versions + 1;
        let entry = self
            .slot((gles, stats), (node, slot, size))
            .map_err(|e| e.to_string())?;
        let mut texture = entry.texture.clone();
        let bits = scale.to_bits();
        let known = entry.tracker.take().filter(|(kept, _)| *kept == bits);
        let age = usize::from(known.is_some());
        let (_, mut tracker) = known.unwrap_or_else(|| {
            let pixels = (size.w, size.h);
            let fresh = OutputDamageTracker::new(pixels, scale, smithay::utils::Transform::Normal);
            (bits, fresh)
        });
        let drew = render_into(gles, &mut texture, (&mut tracker, age), scale, elements)?;
        entry.tracker = Some((bits, tracker));
        stats.textures_rendered += u64::from(drew);
        if drew {
            entry.version = next;
        }
        let version = entry.version;
        self.versions = self.versions.max(version);
        Ok((texture, version))
    }

    /// The id of the element of a layer, and its commit: a new one when
    /// the element shows something else than it did, or its texture has
    /// another version.
    fn next(
        &mut self,
        (node, piece): (&str, usize),
        (shape, alpha, texture, version): (&Shape, f32, u32, u64),
    ) -> (Id, CommitCounter) {
        let entry = self
            .ids
            .entry((self.view.clone(), node.into(), piece))
            .or_insert_with(|| Tracked {
                id: Id::new(),
                commit: CommitCounter::default(),
                seen: None,
            });
        let same = entry.seen.as_ref().is_some_and(|seen| {
            seen.version == version
                && seen.texture == texture
                && seen.alpha.to_bits() == alpha.to_bits()
                && seen.geometry == shape.bounds
                && seen.indices == shape.indices
                && seen.vertices == shape.vertices
        });
        if !same {
            entry.commit.increment();
            entry.seen = Some(Seen {
                vertices: shape.vertices.clone(),
                indices: shape.indices.clone(),
                geometry: shape.bounds,
                alpha,
                texture,
                version,
            });
        }
        (entry.id.clone(), entry.commit)
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
    let own = match texture_size(node, scale, MAX_SIDE) {
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

/// The pixel size of the texture of a layer node at an output scale, with
/// at most `max` pixels a side.
pub fn texture_size(node: &Node, scale: f64, max: f64) -> Option<Size<i32, BufferCoords>> {
    let size = node.props.size?;
    pixel_size(size, fit_scale(size, scale, max), max)
}

/// The largest side of a layer texture for the `GL_MAX_TEXTURE_SIZE` of a
/// GL: at most `MAX_SIDE`.
fn side_limit(gl_max: Option<i32>) -> f64 {
    gl_max.map_or(MAX_SIDE, |side| f64::from(side).min(MAX_SIDE))
}

/// The scale at which a texture of a logical size holds all of its
/// content: `scale`, or less when a side would pass `max` pixels, so that
/// a large plane shows whole and takes input where it shows.
pub fn fit_scale([w, h]: [f64; 2], scale: f64, max: f64) -> f64 {
    scale.min(max / w.max(h))
}

/// The pixel size of a texture of a logical size, at most `max` a side.
pub fn pixel_size([w, h]: [f64; 2], scale: f64, max: f64) -> Option<Size<i32, BufferCoords>> {
    #[allow(clippy::cast_possible_truncation)] // clamped
    let side = |v: f64| (v * scale).round().clamp(1.0, max) as i32;
    (w > 0.0 && h > 0.0).then(|| Size::from((side(w), side(h))))
}

/// Renders elements into a layer texture through the tracker of the
/// texture, transparent where they leave it empty, and opaque where an
/// element says it is opaque. `age` is 0 for a texture without content and
/// 1 when it holds the last render of the tracker. Returns whether
/// anything drew.
fn render_into<E>(
    gles: &mut GlesRenderer,
    texture: &mut GlesTexture,
    (tracker, age): (&mut OutputDamageTracker, usize),
    scale: f64,
    elements: &[E],
) -> Result<bool, String>
where
    E: RenderElement<GlesRenderer>,
{
    let size = Texture::size(texture);
    let mut target = gles.bind(texture).map_err(|e| e.to_string())?;
    let drew = tracker
        .render_output(gles, &mut target, age, elements, [0.0; 4])
        .map_err(|e| format!("{e:?}"))?
        .damage
        .is_some();
    if !drew {
        return Ok(false);
    }
    // An output shows no alpha, so a window that says it is opaque shows
    // there whatever alpha it draws, for example a GL client with the 2
    // bits of alpha of ARGB2101010. A texture keeps that alpha; alpha 1
    // makes it show what an output shows.
    let scale = Scale::from(scale);
    let opaque: Vec<Rectangle<i32, Physical>> = elements
        .iter()
        .flat_map(|e| {
            let at = e.geometry(scale).loc;
            e.opaque_regions(scale)
                .into_iter()
                .map(move |r| Rectangle::new(r.loc + at, r.size))
        })
        .collect();
    if opaque.is_empty() {
        return Ok(true);
    }
    let mut frame = gles
        .render(
            &mut target,
            (size.w, size.h).into(),
            smithay::utils::Transform::Normal,
        )
        .map_err(|e| e.to_string())?;
    gl::make_opaque(&mut frame, &opaque).map_err(|e| e.to_string())?;
    frame.finish().map(drop).map_err(|e| e.to_string())?;
    Ok(true)
}

/// A vertex of a mesh: the position in physical output coordinates in
/// homogeneous form (x, y, w), and the texture coordinates.
type Vertex = ([f32; 3], [f32; 2]);

/// Where a layer shows: the transform to logical output coordinates and
/// the output scale.
pub struct Placing<'a> {
    pub transform: &'a Transform,
    pub scale: f64,
    /// The least w of a point that shows; a triangle that reaches below it
    /// is cut there.
    pub near: f64,
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
/// logical output coordinates. `None` when no triangle is in front of the
/// viewer, or when a quad is edge on or collapsed.
pub fn shape(node: &Node, at: &Placing<'_>) -> Option<Shape> {
    let size = node.props.size?;
    let flat;
    let mesh = if let Some(mesh) = &node.props.mesh {
        mesh
    } else {
        if !at.transform.is_plain() {
            at.transform.plane_inverse()?;
        }
        flat = Mesh::flat(1, 1, size);
        &flat
    };
    project(mesh, at)
}

impl MeshElement {
    /// The element of a layer node: its texture through its shape.
    pub fn new(
        layers: &mut Layers,
        node: &Node,
        drawn: (GlesTexture, u64),
        shape: Shape,
        alpha: f32,
    ) -> Self {
        Self::piece(layers, (node, 0), drawn, shape, alpha)
    }

    /// The element of a piece of a plane of a 3D context: piece 0 is the
    /// whole layer, each other piece has an id of its own. The version says how often
    /// the texture drew anew.
    pub fn piece(
        layers: &mut Layers,
        (node, piece): (&Node, usize),
        (texture, version): (GlesTexture, u64),
        shape: Shape,
        alpha: f32,
    ) -> Self {
        let seen = (&shape, alpha, texture.tex_id(), version);
        let (id, commit) = layers.next((&node.id, piece), seen);
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

/// A point of a triangle in homogeneous screen coordinates: x, y, w, and
/// the texture coordinates u and v.
type Corner = [f64; 5];

/// The part of a convex polygon of corners at `w >= near`; the corners
/// that the cut adds sit at `w = near`, with the texture coordinates of
/// their edge.
fn cut_at(polygon: &[Corner], near: f64) -> Vec<Corner> {
    let mut out = Vec::with_capacity(polygon.len() + 1);
    for (index, from) in polygon.iter().enumerate() {
        let to = &polygon[(index + 1) % polygon.len()];
        let (from_in, to_in) = (from[2] >= near, to[2] >= near);
        if from_in {
            out.push(*from);
        }
        if from_in != to_in {
            let t = (near - from[2]) / (to[2] - from[2]);
            let mut corner: Corner = std::array::from_fn(|i| from[i] + (to[i] - from[i]) * t);
            corner[2] = near;
            out.push(corner);
        }
    }
    out
}

/// The vertices of a mesh in physical output coordinates, with texture
/// coordinates, the triangles that show, and their bounding box. A
/// triangle with all points behind the viewer does not show. One that
/// reaches behind `at.near` keeps only the part in front: its box comes
/// from that part, and where `near` is above the cut of the GPU, the part
/// becomes new triangles. The transform projects the points with their
/// depth, and the clip position keeps the w of each point, so the texture
/// maps perspective-correct. `None` when no triangle shows, or when the
/// mesh has more points than 16-bit indices reach.
#[allow(clippy::cast_possible_truncation)] // GL takes 32-bit floats
fn project(mesh: &Mesh, at: &Placing<'_>) -> Option<Shape> {
    let (m, scale) = (at.transform.matrix(), at.scale);
    let (cols, rows) = (f64::from(mesh.cols), f64::from(mesh.rows));
    let stride = mesh.cols as usize + 1;
    let mut vertices = Vec::with_capacity(mesh.points.len());
    let mut corners: Vec<Corner> = Vec::with_capacity(mesh.points.len());
    for (index, [px, py]) in mesh.points.iter().enumerate() {
        let pz = mesh.z(index);
        let hx = m[0] * px + m[4] * py + m[8] * pz + m[12];
        let hy = m[1] * px + m[5] * py + m[9] * pz + m[13];
        let hw = m[3] * px + m[7] * py + m[11] * pz + m[15];
        #[allow(clippy::cast_precision_loss)] // at most 65 points per side
        let uv = [
            ((index % stride) as f64 / cols) as f32,
            ((index / stride) as f64 / rows) as f32,
        ];
        corners.push([
            hx * scale,
            hy * scale,
            hw,
            f64::from(uv[0]),
            f64::from(uv[1]),
        ]);
        vertices.push(([(hx * scale) as f32, (hy * scale) as f32, hw as f32], uv));
    }
    u16::try_from(vertices.len()).ok()?;
    let mut bounds = Bounds::default();
    let mut kept = Vec::new();
    for triangle in triangles(mesh.cols, mesh.rows).chunks(3) {
        let own: Vec<Corner> = triangle.iter().map(|i| corners[usize::from(*i)]).collect();
        if own.iter().all(|c| c[2] > at.near) {
            bounds.add_points(&own);
            kept.extend_from_slice(triangle);
            continue;
        }
        let part = cut_at(&own, at.near.max(NEAR));
        if part.len() < 3 {
            continue;
        }
        bounds.add_points(&part);
        if at.near > NEAR {
            fan(&part, &mut vertices, &mut kept)?;
        } else {
            kept.extend_from_slice(triangle);
        }
    }
    if kept.is_empty() {
        return None;
    }
    Some(Shape {
        vertices,
        indices: kept,
        bounds: bounds.pixels(),
    })
}

/// Adds the vertices of a convex polygon and the triangles of its fan.
/// `None` when the vertices outgrow 16-bit indices.
#[allow(clippy::cast_possible_truncation)]
fn fan(polygon: &[Corner], vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>) -> Option<()> {
    let first = u16::try_from(vertices.len()).ok()?;
    u16::try_from(vertices.len() + polygon.len()).ok()?;
    vertices.extend(
        polygon
            .iter()
            .map(|[x, y, w, u, v]| ([*x as f32, *y as f32, *w as f32], [*u as f32, *v as f32])),
    );
    for next in 1..polygon.len() as u16 - 1 {
        indices.extend([first, first + next, first + next + 1]);
    }
    Some(())
}

impl Shape {
    /// Whether the shape draws at a point of the output, in the logical
    /// pixels that it was projected in: the point lies in a triangle, or
    /// in the part of one in front of the eye.
    pub fn covers(&self, point: [f64; 2]) -> bool {
        self.indices.chunks(3).any(|triangle| {
            let own: Vec<Corner> = triangle
                .iter()
                .map(|i| {
                    let (at, uv) = self.vertices[usize::from(*i)];
                    [at[0], at[1], at[2], uv[0], uv[1]].map(f64::from)
                })
                .collect();
            let shown: Vec<[f64; 2]> = cut_at(&own, NEAR)
                .iter()
                .map(|[x, y, w, ..]| [x / w, y / w])
                .collect();
            shown.len() >= 3 && crate::space::contains(&shown, point)
        })
    }
}

/// The shape of a piece of a plane of a size: a convex quad, possibly with
/// a repeated corner, of points of the plane, which the texture of the
/// plane covers from `(0, 0)` to the size. `None` when no part of it lies
/// in front of the eye. The vertices are 32-bit floats, as GL takes them.
#[allow(clippy::cast_possible_truncation)]
pub fn piece_shape(
    corners: &[[f64; 2]; 4],
    [w, h]: [f64; 2],
    transform: &Transform,
    scale: f64,
) -> Option<Shape> {
    let m = transform.matrix();
    let points = corners.map(|[u, v]| {
        [
            (m[0] * u + m[4] * v + m[12]) * scale,
            (m[1] * u + m[5] * v + m[13]) * scale,
            m[3] * u + m[7] * v + m[15],
        ]
    });
    let mut bounds = Bounds::default();
    if !bounds.add_clipped(&points) {
        return None;
    }
    let vertices = points
        .iter()
        .zip(corners)
        .map(|([x, y, hw], [u, v])| {
            (
                [*x as f32, *y as f32, *hw as f32],
                [(u / w) as f32, (v / h) as f32],
            )
        })
        .collect();
    Some(Shape {
        vertices,
        indices: vec![0, 1, 2, 0, 2, 3],
        bounds: bounds.pixels(),
    })
}

/// The box of the screen points of polygons.
pub(crate) struct Bounds {
    min: [f64; 2],
    max: [f64; 2],
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            min: [f64::MAX; 2],
            max: [f64::MIN; 2],
        }
    }
}

impl Bounds {
    /// The box of what was added; a box of nothing overlaps nothing.
    pub(crate) fn rect(&self) -> smithay::utils::Rectangle<f64, smithay::utils::Logical> {
        smithay::utils::Rectangle::new(
            (self.min[0], self.min[1]).into(),
            (self.max[0] - self.min[0], self.max[1] - self.min[1]).into(),
        )
    }

    /// The box in whole pixels, at least one pixel a side. A point close to
    /// the eye lands far away; the box stays in i32, so the casts are
    /// clamped.
    #[allow(clippy::cast_possible_truncation)]
    fn pixels(&self) -> Rectangle<i32, Physical> {
        let clamp = |v: f64| v.clamp(-FAR, FAR) as i32;
        let loc = Point::from((clamp(self.min[0].floor()), clamp(self.min[1].floor())));
        let end =
            Point::<i32, Physical>::from((clamp(self.max[0].ceil()), clamp(self.max[1].ceil())));
        let size = Size::from(((end.x - loc.x).max(1), (end.y - loc.y).max(1)));
        Rectangle::new(loc, size)
    }

    /// Adds the screen points of corners with a w in front of the eye.
    pub(crate) fn add_points(&mut self, corners: &[Corner]) {
        for [x, y, w, ..] in corners {
            for (axis, v) in [x / w, y / w].into_iter().enumerate() {
                self.min[axis] = self.min[axis].min(v);
                self.max[axis] = self.max[axis].max(v);
            }
        }
    }

    /// Adds the part of a polygon of (x, y, w) points that lies at
    /// `w >= NEAR`. Returns whether any of it does.
    pub(crate) fn add_clipped(&mut self, polygon: &[[f64; 3]]) -> bool {
        self.add_clipped_at(polygon, NEAR)
    }

    /// As [`Self::add_clipped`], at `w >= near`.
    pub(crate) fn add_clipped_at(&mut self, polygon: &[[f64; 3]], near: f64) -> bool {
        let mut shown = false;
        let mut add = |[x, y, w]: [f64; 3]| {
            shown = true;
            for (axis, v) in [x / w, y / w].into_iter().enumerate() {
                self.min[axis] = self.min[axis].min(v);
                self.max[axis] = self.max[axis].max(v);
            }
        };
        for (index, from) in polygon.iter().enumerate() {
            let to = &polygon[(index + 1) % polygon.len()];
            let (from_in, to_in) = (from[2] > near, to[2] > near);
            if from_in {
                add(*from);
            }
            if from_in != to_in {
                let t = (near - from[2]) / (to[2] - from[2]);
                add([
                    from[0] + (to[0] - from[0]) * t,
                    from[1] + (to[1] - from[1]) * t,
                    near,
                ]);
            }
        }
        shown
    }
}

/// The least w of a point that shows.
pub(crate) const NEAR: f64 = 1e-6;
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
        commit: Option<CommitCounter>,
    ) -> DamageSet<i32, Physical> {
        if commit.is_some_and(|seen| seen == self.commit) {
            return DamageSet::default();
        }
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
#[derive(Debug)]
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

/// A node texture that a shader reads: the name of its sampler, the
/// texture and its version.
pub type Source = (String, GlesTexture, u64);

/// Runs the passes of a layer node over its texture and returns the
/// texture of the last pass with its version. `version` is that of the
/// texture; passes that read no clock and see the same texture, the same
/// node textures and the same uniforms as before keep their last result
/// and its version. A pass whose shader fails is left out; its error goes
/// to `errors`.
pub fn apply_passes(
    gles: &mut GlesRenderer,
    (layers, errors, stats): (&mut Layers, &mut Vec<String>, &mut RenderStats),
    node: &Node,
    (texture, version): (GlesTexture, u64),
    (scale, sources): (f64, &[Source]),
) -> (GlesTexture, u64) {
    let wanted = passes(node, scale);
    if wanted.is_empty() {
        return (texture, version);
    }
    let all = wanted.len();
    let compiled: Vec<(Pass<'_>, Compiled)> = wanted
        .into_iter()
        .filter_map(|pass| {
            let compiled = program_of(gles, &pass, (scale, sources), errors)?;
            Some((pass, compiled))
        })
        .collect();
    let size = Texture::size(&texture);
    let size_key = (size.w, size.h);
    let timed = compiled
        .iter()
        .any(|(_, (program, ..))| program.reads_time());
    let signature = signature((scale, version, sources), &compiled);
    let key = (layers.view.clone(), node.id.clone());
    let last = layers.passed.get(&key).map(|last| {
        (
            last.reusable && last.signature == signature && last.size == size_key,
            last.slot,
            last.version,
            last.runs,
        )
    });
    if let Some((true, slot, version, _)) = last
        && !timed
        && let Some(result) = layers.existing(&node.id, slot, size)
    {
        return (result, version);
    }
    let runs = last.map_or(0, |(.., runs)| runs) + 1;
    let errors_before = errors.len();
    let (texture, slot, completed) =
        run_passes((gles, layers, errors, stats), &node.id, texture, &compiled);
    let result = hash_of(&(signature, runs));
    let reusable = !timed && errors.len() == errors_before && completed == all;
    let passed = Passed {
        size: size_key,
        signature,
        slot,
        version: result,
        runs,
        reusable,
    };
    layers.passed.insert(key, passed);
    (texture, result)
}

/// Draws each pass over the result of the one before; returns the last
/// texture, its slot and the number of passes that ran.
fn run_passes(
    (gles, layers, errors, stats): (
        &mut GlesRenderer,
        &mut Layers,
        &mut Vec<String>,
        &mut RenderStats,
    ),
    node: &str,
    mut texture: GlesTexture,
    compiled: &[(Pass<'_>, Compiled)],
) -> (GlesTexture, usize, usize) {
    let mut slot = 0;
    let mut completed = 0;
    for (_, (program, uniforms, extra)) in compiled {
        let size = Texture::size(&texture);
        let next = if slot == 1 { 2 } else { 1 };
        let Ok(mut target) = layers.texture((gles, stats), node, next, size) else {
            continue;
        };
        match draw_pass(gles, program, &texture, &mut target, (uniforms, extra)) {
            Ok(()) => {
                stats.passes += 1;
                texture = target;
                slot = next;
                completed += 1;
            }
            Err(err) => errors.push(err),
        }
    }
    (texture, slot, completed)
}

fn hash_of(value: &impl std::hash::Hash) -> u64 {
    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// A hash of what the passes draw: the passes with their uniforms and
/// programs, the version of the texture and those of the node textures.
fn signature(
    (scale, version, sources): (f64, u64, &[Source]),
    compiled: &[(Pass<'_>, Compiled)],
) -> u64 {
    use std::{
        fmt::Write,
        hash::{Hash, Hasher},
    };

    struct Feed(std::collections::hash_map::DefaultHasher);
    impl Write for Feed {
        fn write_str(&mut self, text: &str) -> std::fmt::Result {
            self.0.write(text.as_bytes());
            Ok(())
        }
    }
    let mut feed = Feed(std::collections::hash_map::DefaultHasher::new());
    (scale.to_bits(), version).hash(&mut feed.0);
    for (_, texture, version) in sources {
        (texture.tex_id(), version).hash(&mut feed.0);
    }
    for (pass, (program, uniforms, extra)) in compiled {
        let _ = write!(feed, "{pass:?}{uniforms:?}");
        program.id().hash(&mut feed.0);
        extra.hash(&mut feed.0);
    }
    feed.0.finish()
}

/// A program with its uniforms and its node textures.
type Compiled = (gl::Program, BTreeMap<String, Uniform>, Vec<(String, u32)>);

/// The program of a pass, its uniforms and its node textures.
fn program_of(
    gles: &mut GlesRenderer,
    pass: &Pass<'_>,
    (scale, sources): (f64, &[Source]),
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
                sources
                    .iter()
                    .map(|(n, t, _)| (n.clone(), t.tex_id()))
                    .collect()
            } else {
                Vec::new()
            };
            let mut uniforms = (*uniforms).clone();
            uniforms.insert("scale".into(), Uniform::Float(scale));
            Some((program?, uniforms, extra))
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
    let at = Placing {
        transform: &Transform::IDENTITY,
        scale: 1.0,
        near: NEAR,
    };
    let vertices = project(&quad, &at).ok_or("an empty pass")?.vertices;
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
    fn a_layer_texture_side_follows_a_smaller_gl_limit() {
        assert!((side_limit(Some(2048)) - 2048.0).abs() < f64::EPSILON);
        assert!((side_limit(Some(16384)) - 4096.0).abs() < f64::EPSILON);
        assert!((side_limit(None) - 4096.0).abs() < f64::EPSILON);
        assert!((fit_scale([5000.0, 100.0], 1.0, 2048.0) - 0.4096).abs() < 1e-9);
    }

    fn projected(mesh: &Mesh, transform: &Transform, scale: f64) -> Option<Shape> {
        project(
            mesh,
            &Placing {
                transform,
                scale,
                near: NEAR,
            },
        )
    }

    #[test]
    fn a_grid_has_two_triangles_per_cell() {
        let indices = triangles(2, 1);
        assert_eq!(indices.len(), 12);
        assert_eq!(&indices[..6], &[0, 1, 3, 1, 4, 3]);
    }

    #[test]
    fn a_point_behind_the_viewer_clips_its_triangles_and_all_behind_hide_them() {
        let mesh = Mesh::flat(1, 1, [10.0, 10.0]);
        let mut m = *Transform::IDENTITY.matrix();
        m[15] = -1.0;
        assert!(projected(&mesh, &Transform::from_matrix(m), 1.0).is_none());
        let shape = projected(&mesh, &Transform::IDENTITY, 2.0).unwrap();
        assert_eq!((shape.vertices.len(), shape.indices.len()), (4, 6));
        assert_eq!(shape.bounds.size, (20, 20).into());
        let mut mesh = Mesh::flat(2, 1, [20.0, 10.0]);
        mesh.depth = vec![0.0, 0.0, 2.0, 0.0, 0.0, 2.0];
        let mut m = *Transform::IDENTITY.matrix();
        m[11] = -1.0;
        let shape = projected(&mesh, &Transform::from_matrix(m), 1.0).unwrap();
        assert_eq!(shape.indices, [0, 1, 3, 1, 4, 3, 1, 2, 4, 2, 5, 4]);
        assert_eq!(shape.bounds.loc, (0, 0).into());
        assert!(shape.bounds.size.w > 1_000_000, "{:?}", shape.bounds);
        mesh.depth = vec![2.0; 6];
        assert!(projected(&mesh, &Transform::from_matrix(m), 1.0).is_none());
    }
}
