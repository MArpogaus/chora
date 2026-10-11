//! Layers and meshes (D19, M3).
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
//! Drawing a mesh needs raw GL calls, which are `unsafe`; this module is the
//! one place with them.
#![allow(unsafe_code)]

use std::collections::HashMap;

use chora_scene::{Mesh, Node, Transform};
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            Bind, ImportAll, ImportMem, Offscreen, Renderer, RendererSuper,
            damage::OutputDamageTracker,
            element::{Element, Id, Kind, RenderElement},
            gles::{Capability, GlesError, GlesFrame, GlesRenderer, GlesTexture, ffi},
            utils::{CommitCounter, DamageSet, OpaqueRegions},
        },
    },
    utils::{Buffer as BufferCoords, Physical, Point, Rectangle, Scale, Size},
};

/// The largest side of a layer texture, in pixels.
const MAX_SIDE: f64 = 8192.0;

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

/// The textures and the ids of the layers by node, kept between frames.
#[derive(Default)]
pub struct Layers {
    textures: HashMap<String, GlesTexture>,
    ids: HashMap<String, (Id, CommitCounter)>,
}

impl Layers {
    /// Drops the layers of nodes that are gone or no layers any more.
    pub fn retain(&mut self, mut keep: impl FnMut(&str) -> bool) {
        self.textures.retain(|id, _| keep(id));
        self.ids.retain(|id, _| keep(id));
    }

    /// A texture for a layer of a size, the one of the last frame if it
    /// fits.
    pub fn texture(
        &mut self,
        gles: &mut GlesRenderer,
        node: &str,
        size: Size<i32, BufferCoords>,
    ) -> Result<GlesTexture, GlesError> {
        if let Some(texture) = self.textures.get(node)
            && smithay::backend::renderer::Texture::size(texture) == size
        {
            return Ok(texture.clone());
        }
        let texture: GlesTexture = gles.create_buffer(Fourcc::Abgr8888, size)?;
        self.textures.insert(node.into(), texture.clone());
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

/// The pixel size of the texture of a layer node at an output scale.
pub fn texture_size(node: &Node, scale: f64) -> Option<Size<i32, BufferCoords>> {
    let [w, h] = node.props.size?;
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
    let size = smithay::backend::renderer::Texture::size(texture);
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

/// Where a layer shows: the transform to logical output coordinates, the
/// output scale and the opacity.
pub struct Placing<'a> {
    pub transform: &'a Transform,
    pub scale: f64,
    pub alpha: f32,
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

impl MeshElement {
    /// The element of a layer node: its texture through its mesh, or a
    /// quad, with a transform to logical output coordinates. `None` when a
    /// point is behind the viewer.
    pub fn new(
        layers: &mut Layers,
        node: &Node,
        texture: GlesTexture,
        at: &Placing<'_>,
    ) -> Option<Self> {
        let size = node.props.size?;
        let mesh = node
            .props
            .mesh
            .clone()
            .unwrap_or_else(|| Mesh::flat(1, 1, size));
        let (vertices, bounds) = project(&mesh, at.transform, at.scale)?;
        let (id, commit) = layers.next(&node.id);
        Some(Self {
            id,
            commit,
            texture,
            vertices,
            indices: triangles(mesh.cols, mesh.rows),
            geometry: bounds,
            alpha: at.alpha,
        })
    }
}

/// The vertices of a mesh in physical output coordinates, with texture
/// coordinates, and their bounding box. `None` when a point is behind the
/// viewer, or when the mesh has more points than 16-bit indices reach.
#[allow(clippy::cast_possible_truncation)] // GL takes 32-bit floats
fn project(
    mesh: &Mesh,
    transform: &Transform,
    scale: f64,
) -> Option<(Vec<Vertex>, Rectangle<i32, Physical>)> {
    let m = transform.matrix();
    let (cols, rows) = (f64::from(mesh.cols), f64::from(mesh.rows));
    let mut vertices = Vec::with_capacity(mesh.points.len());
    let (mut min, mut max) = ([f64::MAX; 2], [f64::MIN; 2]);
    for (index, [px, py]) in mesh.points.iter().enumerate() {
        let hx = m[0] * px + m[4] * py + m[12];
        let hy = m[1] * px + m[5] * py + m[13];
        let hw = m[3] * px + m[7] * py + m[15];
        if hw <= 1e-6 {
            return None;
        }
        let screen = [hx * scale / hw, hy * scale / hw];
        min = [min[0].min(screen[0]), min[1].min(screen[1])];
        max = [max[0].max(screen[0]), max[1].max(screen[1])];
        let stride = mesh.cols as usize + 1;
        #[allow(clippy::cast_precision_loss)] // at most 65 points per side
        let uv = [
            ((index % stride) as f64 / cols) as f32,
            ((index / stride) as f64 / rows) as f32,
        ];
        vertices.push(([(hx * scale) as f32, (hy * scale) as f32, hw as f32], uv));
    }
    u16::try_from(vertices.len()).ok()?;
    let loc = Point::from((min[0].floor() as i32, min[1].floor() as i32));
    let end = Point::<i32, Physical>::from((max[0].ceil() as i32, max[1].ceil() as i32));
    let size = Size::from(((end.x - loc.x).max(1), (end.y - loc.y).max(1)));
    Some((vertices, Rectangle::new(loc, size)))
}

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
        let size = smithay::backend::renderer::Texture::size(&self.texture);
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
        draw(frame, self).map_err(R::from_gles)
    }
}

/// The GL program of meshes, once per EGL context.
#[derive(Clone, Copy)]
struct Program {
    id: ffi::types::GLuint,
    position: ffi::types::GLint,
    uv: ffi::types::GLint,
    tex: ffi::types::GLint,
    alpha: ffi::types::GLint,
}

const VERTEX: &str = "
attribute vec4 position;
attribute vec2 uv;
varying vec2 v_uv;
void main() {
    gl_Position = position;
    v_uv = uv;
}
";

const FRAGMENT: &str = "
precision mediump float;
uniform sampler2D tex;
uniform float alpha;
varying vec2 v_uv;
void main() {
    gl_FragColor = texture2D(tex, v_uv) * alpha;
}
";

/// Draws a mesh element: maps its vertices through the projection of the
/// frame into clip space and draws its triangles with the texture.
fn draw(frame: &mut GlesFrame<'_, '_>, mesh: &MeshElement) -> Result<(), GlesError> {
    let program = program(frame)?;
    let p = *frame.projection();
    // Column-major 3x3: the projection maps physical (x, y, 1) to clip
    // space; for (x, y, w) it gives the clip position before the divide.
    let clip: Vec<f32> = mesh
        .vertices
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
    let uv: Vec<f32> = mesh.vertices.iter().flat_map(|(_, uv)| *uv).collect();
    let texture = mesh.texture.tex_id();
    let count = i32::try_from(mesh.indices.len()).unwrap_or(0);
    // Smithay's instanced draws leave divisors on attribute indices.
    let instancing = frame.capabilities().contains(&Capability::Instancing);
    frame.with_context(|gl| unsafe {
        // SAFETY: the arrays outlive the draw call, and the state that
        // changes here goes back to what Smithay sets before its own draws:
        // its draw calls bind their program, texture and arrays each time.
        gl.UseProgram(program.id);
        gl.ActiveTexture(ffi::TEXTURE0);
        gl.BindTexture(ffi::TEXTURE_2D, texture);
        gl.TexParameteri(
            ffi::TEXTURE_2D,
            ffi::TEXTURE_MIN_FILTER,
            ffi::LINEAR.cast_signed(),
        );
        gl.TexParameteri(
            ffi::TEXTURE_2D,
            ffi::TEXTURE_MAG_FILTER,
            ffi::LINEAR.cast_signed(),
        );
        gl.Uniform1i(program.tex, 0);
        gl.Uniform1f(program.alpha, mesh.alpha);
        gl.BindBuffer(ffi::ARRAY_BUFFER, 0);
        let attributes = [(program.position, 4, &clip), (program.uv, 2, &uv)];
        for (attribute, size, data) in attributes {
            #[allow(clippy::cast_sign_loss)] // a found attribute is not negative
            let index = attribute as u32;
            gl.EnableVertexAttribArray(index);
            if instancing {
                gl.VertexAttribDivisor(index, 0);
            }
            gl.VertexAttribPointer(index, size, ffi::FLOAT, ffi::FALSE, 0, data.as_ptr().cast());
        }
        gl.DrawElements(
            ffi::TRIANGLES,
            count,
            ffi::UNSIGNED_SHORT,
            mesh.indices.as_ptr().cast(),
        );
        for attribute in [program.position, program.uv] {
            #[allow(clippy::cast_sign_loss)]
            gl.DisableVertexAttribArray(attribute as u32);
        }
        gl.BindTexture(ffi::TEXTURE_2D, 0);
    })
}

/// The mesh program of the EGL context of a frame, compiled on first use.
fn program(frame: &mut GlesFrame<'_, '_>) -> Result<Program, GlesError> {
    if let Some(program) = frame.egl_context().user_data().get::<Program>() {
        return Ok(*program);
    }
    let program = frame
        .with_context(|gl| unsafe { compile(gl) })?
        .ok_or(GlesError::ShaderCompileError)?;
    frame
        .egl_context()
        .user_data()
        .insert_if_missing_threadsafe(|| program);
    Ok(program)
}

/// Compiles and links the mesh program.
unsafe fn compile(gl: &ffi::Gles2) -> Option<Program> {
    // SAFETY: plain GL calls on the current context with valid strings.
    unsafe {
        let shader = |kind, source: &str| {
            let shader = gl.CreateShader(kind);
            let source = std::ffi::CString::new(source).ok()?;
            gl.ShaderSource(shader, 1, &source.as_ptr(), std::ptr::null());
            gl.CompileShader(shader);
            let mut ok = 0;
            gl.GetShaderiv(shader, ffi::COMPILE_STATUS, &raw mut ok);
            (ok != 0).then_some(shader)
        };
        let vertex = shader(ffi::VERTEX_SHADER, VERTEX)?;
        let fragment = shader(ffi::FRAGMENT_SHADER, FRAGMENT)?;
        let program = gl.CreateProgram();
        gl.AttachShader(program, vertex);
        gl.AttachShader(program, fragment);
        gl.LinkProgram(program);
        let mut ok = 0;
        gl.GetProgramiv(program, ffi::LINK_STATUS, &raw mut ok);
        gl.DeleteShader(vertex);
        gl.DeleteShader(fragment);
        if ok == 0 {
            return None;
        }
        let location = |name: &str, uniform: bool| {
            let name = std::ffi::CString::new(name).expect("no nul");
            if uniform {
                gl.GetUniformLocation(program, name.as_ptr())
            } else {
                gl.GetAttribLocation(program, name.as_ptr())
            }
        };
        Some(Program {
            id: program,
            position: location("position", false),
            uv: location("uv", false),
            tex: location("tex", true),
            alpha: location("alpha", true),
        })
    }
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
    fn a_point_behind_the_viewer_hides_the_layer() {
        let mesh = Mesh::flat(1, 1, [10.0, 10.0]);
        let mut m = *Transform::IDENTITY.matrix();
        m[15] = -1.0;
        assert!(project(&mesh, &Transform::from_matrix(m), 1.0).is_none());
        let (vertices, bounds) = project(&mesh, &Transform::IDENTITY, 2.0).unwrap();
        assert_eq!(vertices.len(), 4);
        assert_eq!(bounds.size, (20, 20).into());
    }
}
