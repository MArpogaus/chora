//! The import of client buffers at a commit. The frames import the buffer
//! of a surface lazily, with the damage of all commits since the last
//! import, from the newest buffer. That is right while a surface commits
//! one buffer again and again. Two cases need the commit itself:
//!
//! - A new buffer replaces one whose damage no renderer has imported yet:
//!   the old buffer goes into the texture before the commit replaces it,
//!   with its own damage, so that the new buffer brings only its own.
//! - A damage-less attach: a commit attaches an shm buffer of the same
//!   size and format as the buffer before, and its damage, cut to the
//!   buffer, is empty. The texture stays as it is and takes no pixel of
//!   the new buffer. The protocol makes the new buffer the content of the
//!   surface; keeping the texture is a deliberate leniency, as in
//!   wlroots, that foot relies on: it commits its second buffer, which
//!   still holds the frame before, without damage when the picture did
//!   not change.
//!
//! The renderers of smithay cannot be told to keep a texture: a new
//! buffer drops the textures of the surface, and the GLES renderer
//! uploads an shm buffer in full when the damage is empty. So a
//! damage-less attach imports through [`KeepTexture`], which hands the
//! renderer damage outside of the buffer. The renderer then reuses the
//! texture of the surface and uploads nothing.

use smithay::{
    backend::{
        allocator::{Fourcc, dmabuf::Dmabuf},
        renderer::{
            ContextId, DebugFlags, ImportDma, ImportDmaWl, ImportMem, ImportMemWl, Renderer,
            RendererSuper, TextureFilter,
            sync::SyncPoint,
            utils::{RendererSurfaceStateUserData, import_surface},
        },
    },
    reexports::wayland_server::protocol::{wl_buffer::WlBuffer, wl_shm, wl_surface::WlSurface},
    utils::{Buffer, Physical, Rectangle, Size, Transform},
    wayland::compositor::{
        BufferAssignment, Damage, SurfaceAttributes, SurfaceData, TraversalAction, with_states,
        with_surface_tree_downward,
    },
};

/// What a commit asks of the textures of a surface tree, found before
/// smithay takes the committed buffers.
#[derive(Default)]
pub struct CommitImports {
    /// The surfaces whose new buffer replaces another one.
    replaced: Vec<WlSurface>,
    /// The surfaces with a damage-less attach.
    kept: Vec<WlSurface>,
}

impl CommitImports {
    /// Reads the committed state of a surface tree.
    pub fn of(surface: &WlSurface) -> Self {
        let mut imports = Self::default();
        with_surface_tree_downward(
            surface,
            (),
            |_, _, ()| TraversalAction::DoChildren(()),
            |surface, states, ()| imports.read(surface, states),
            |_, _, ()| true,
        );
        imports
    }

    fn read(&mut self, surface: &WlSurface, states: &SurfaceData) {
        let mut guard = states.cached_state.get::<SurfaceAttributes>();
        let attrs = guard.current();
        let Some(BufferAssignment::NewBuffer(new)) = &attrs.buffer else {
            return;
        };
        let Some(data) = states.data_map.get::<RendererSurfaceStateUserData>() else {
            return;
        };
        let data = data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(old) = data.buffer() else {
            return;
        };
        if *old != *new {
            self.replaced.push(surface.clone());
        }
        let surface_size = data.surface_size().unwrap_or_default();
        if same_shm_format(old, new) && !damages(&attrs.damage, new, surface_size) {
            self.kept.push(surface.clone());
        }
    }

    /// Imports the replaced buffers whose damage no renderer has imported
    /// yet, before smithay takes the new buffers.
    pub fn before(&self, backend: &mut super::Backend) {
        let mut unseen = vec![true; self.replaced.len()];
        backend.importers(&mut |importer: &mut dyn Importer| {
            for (surface, unseen) in self.replaced.iter().zip(&mut unseen) {
                *unseen &= !with_states(surface, |states| importer.has_texture(states));
            }
        });
        if !unseen.contains(&true) {
            return;
        }
        backend.importers(&mut |importer: &mut dyn Importer| {
            for (surface, _) in self.replaced.iter().zip(&unseen).filter(|(_, u)| **u) {
                with_states(surface, |states| importer.import(states));
            }
        });
    }

    /// Keeps the textures of the damage-less attaches, after smithay took
    /// the new buffers.
    pub fn after(&self, backend: &mut super::Backend) {
        if self.kept.is_empty() {
            return;
        }
        backend.importers(&mut |importer: &mut dyn Importer| {
            for surface in &self.kept {
                with_states(surface, |states| importer.keep(states));
            }
        });
    }
}

/// Whether two buffers are shm buffers of the same size and format.
fn same_shm_format(old: &WlBuffer, new: &WlBuffer) -> bool {
    let shape =
        |buffer| crate::shm::buffer_data(buffer).map(|data| (data.width, data.height, data.format));
    shape(new).is_some_and(|new| shape(old) == Some(new))
}

/// Whether the damage of a commit, cut to the buffer or to the surface,
/// covers a pixel.
fn damages(
    damage: &[Damage],
    buffer: &WlBuffer,
    surface_size: Size<i32, smithay::utils::Logical>,
) -> bool {
    let buffer_size = crate::shm::buffer_data(buffer)
        .map(|data| Size::<i32, Buffer>::from((data.width, data.height)))
        .unwrap_or_default();
    damage.iter().any(|damage| match damage {
        Damage::Buffer(rect) => cut(*rect, Rectangle::from_size(buffer_size)),
        Damage::Surface(rect) => cut(*rect, Rectangle::from_size(surface_size)),
    })
}

fn cut<Kind>(rect: Rectangle<i32, Kind>, bounds: Rectangle<i32, Kind>) -> bool {
    rect.intersection(bounds)
        .is_some_and(|rect| !rect.is_empty())
}

/// A renderer that imports the buffers of clients.
pub trait Importer {
    /// Whether the renderer has a texture of the current buffer of a
    /// surface, that is, whether it imported all its damage.
    fn has_texture(&self, states: &SurfaceData) -> bool;
    /// Imports the current buffer of a surface with its damage.
    fn import(&mut self, states: &SurfaceData);
    /// Takes the current buffer of a surface as its texture, which stays
    /// as it is.
    fn keep(&mut self, states: &SurfaceData);
}

impl<R> Importer for R
where
    R: ImportMemWl + ImportDmaWl,
    R::TextureId: 'static,
{
    fn has_texture(&self, states: &SurfaceData) -> bool {
        states
            .data_map
            .get::<RendererSurfaceStateUserData>()
            .is_some_and(|data| {
                data.lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .texture(self.context_id())
                    .is_some()
            })
    }

    fn import(&mut self, states: &SurfaceData) {
        if let Err(err) = import_surface(self, states) {
            tracing::debug!(?err, "a replaced buffer is not imported");
        }
    }

    fn keep(&mut self, states: &SurfaceData) {
        if let Err(err) = import_surface(&mut KeepTexture(self), states) {
            tracing::debug!(?err, "a damage-less buffer is not imported");
        }
    }
}

/// A renderer whose shm imports upload nothing: it hands its inner
/// renderer damage outside of the buffer, so the inner renderer reuses
/// the texture of the surface as it is.
#[derive(Debug)]
struct KeepTexture<'r, R>(&'r mut R);

/// Damage that no buffer covers.
fn no_pixel() -> Rectangle<i32, Buffer> {
    Rectangle::new((-2, -2).into(), (1, 1).into())
}

impl<R: Renderer> RendererSuper for KeepTexture<'_, R> {
    type Error = R::Error;
    type TextureId = R::TextureId;
    type Framebuffer<'buffer> = R::Framebuffer<'buffer>;
    type Frame<'frame, 'buffer>
        = R::Frame<'frame, 'buffer>
    where
        'buffer: 'frame,
        Self: 'frame;
}

impl<R: Renderer> Renderer for KeepTexture<'_, R> {
    fn context_id(&self) -> ContextId<Self::TextureId> {
        self.0.context_id()
    }

    fn downscale_filter(&mut self, filter: TextureFilter) -> Result<(), Self::Error> {
        self.0.downscale_filter(filter)
    }

    fn upscale_filter(&mut self, filter: TextureFilter) -> Result<(), Self::Error> {
        self.0.upscale_filter(filter)
    }

    fn set_debug_flags(&mut self, flags: DebugFlags) {
        self.0.set_debug_flags(flags);
    }

    fn debug_flags(&self) -> DebugFlags {
        self.0.debug_flags()
    }

    fn render<'frame, 'buffer>(
        &'frame mut self,
        framebuffer: &'frame mut Self::Framebuffer<'buffer>,
        output_size: Size<i32, Physical>,
        dst_transform: Transform,
    ) -> Result<Self::Frame<'frame, 'buffer>, Self::Error>
    where
        'buffer: 'frame,
    {
        self.0.render(framebuffer, output_size, dst_transform)
    }

    fn wait(&mut self, sync: &SyncPoint) -> Result<(), Self::Error> {
        self.0.wait(sync)
    }
}

impl<R: ImportMem> ImportMem for KeepTexture<'_, R> {
    fn import_memory(
        &mut self,
        data: &[u8],
        format: Fourcc,
        size: Size<i32, Buffer>,
        flipped: bool,
    ) -> Result<Self::TextureId, Self::Error> {
        self.0.import_memory(data, format, size, flipped)
    }

    fn update_memory(
        &mut self,
        texture: &Self::TextureId,
        data: &[u8],
        region: Rectangle<i32, Buffer>,
    ) -> Result<(), Self::Error> {
        self.0.update_memory(texture, data, region)
    }

    fn mem_formats(&self) -> Box<dyn Iterator<Item = Fourcc>> {
        self.0.mem_formats()
    }
}

impl<R: ImportMemWl> ImportMemWl for KeepTexture<'_, R> {
    fn import_shm_buffer(
        &mut self,
        buffer: &WlBuffer,
        surface: Option<&SurfaceData>,
        _damage: &[Rectangle<i32, Buffer>],
    ) -> Result<Self::TextureId, Self::Error> {
        self.0.import_shm_buffer(buffer, surface, &[no_pixel()])
    }

    fn shm_formats(&self) -> Box<dyn Iterator<Item = wl_shm::Format>> {
        self.0.shm_formats()
    }
}

impl<R: ImportDma> ImportDma for KeepTexture<'_, R> {
    fn import_dmabuf(
        &mut self,
        dmabuf: &Dmabuf,
        damage: Option<&[Rectangle<i32, Buffer>]>,
    ) -> Result<Self::TextureId, Self::Error> {
        self.0.import_dmabuf(dmabuf, damage)
    }
}

impl<R: ImportDmaWl> ImportDmaWl for KeepTexture<'_, R> {}
