//! Cursor images from the xcursor theme, for outputs on which no host
//! compositor draws a cursor.
//!
//! The theme loading follows `cursor.rs` of Smithay's anvil, MIT License
//! (see `LICENSES/MIT-Smithay.txt`).

use std::collections::HashMap;

use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            ImportAll, ImportMem, Renderer,
            element::{
                Kind,
                memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
                surface::render_elements_from_surface_tree,
            },
        },
    },
    input::pointer::{CursorImageStatus, CursorImageSurfaceData},
    reexports::wayland_server::Resource,
    utils::{Logical, Physical, Point, Transform},
    wayland::compositor::with_states,
};
use xcursor::{CursorTheme, parser::parse_xcursor};

use crate::render::SceneElement;

/// The images of the cursor theme, each loaded on first use.
pub struct Cursors {
    theme: CursorTheme,
    size: u32,
    images: HashMap<String, Option<CursorImage>>,
}

/// One cursor image and its hotspot.
#[derive(Clone)]
pub struct CursorImage {
    pub buffer: MemoryRenderBuffer,
    pub hotspot: Point<i32, Physical>,
}

impl Cursors {
    /// The theme in `XCURSOR_THEME` with the size in `XCURSOR_SIZE`, the
    /// variables that toolkits read too.
    pub fn from_env() -> Self {
        let name = std::env::var("XCURSOR_THEME").unwrap_or_else(|_| "default".into());
        let size = std::env::var("XCURSOR_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24);
        Self {
            theme: CursorTheme::load(&name),
            size,
            images: HashMap::new(),
        }
    }

    /// The image of a named cursor, or of the default cursor if the theme
    /// lacks the name.
    pub fn get(&mut self, name: &str) -> Option<CursorImage> {
        let (theme, size) = (&self.theme, self.size);
        let image = self
            .images
            .entry(name.into())
            .or_insert_with(|| load(theme, name, size))
            .clone();
        image.or_else(|| (name != "default").then(|| self.get("default"))?)
    }
}

/// The first image of the nearest size; animations show their first frame.
fn load(theme: &CursorTheme, name: &str, size: u32) -> Option<CursorImage> {
    let data = std::fs::read(theme.load_icon(name)?).ok()?;
    let images = parse_xcursor(&data)?;
    let image = images.iter().min_by_key(|i| i.size.abs_diff(size))?;
    let int = |v: u32| i32::try_from(v).ok();
    let buffer = MemoryRenderBuffer::from_slice(
        &image.pixels_rgba,
        Fourcc::Abgr8888,
        (int(image.width)?, int(image.height)?),
        1,
        Transform::Normal,
        None,
    );
    Some(CursorImage {
        buffer,
        hotspot: (int(image.xhot)?, int(image.yhot)?).into(),
    })
}

/// The render elements of the cursor at a position in logical output
/// coordinates: the surface of the client under the pointer, or a named
/// cursor of the theme.
pub fn elements<R>(
    renderer: &mut R,
    status: &CursorImageStatus,
    cursors: &mut Cursors,
    position: Point<f64, Logical>,
    scale: f64,
) -> Vec<SceneElement<R>>
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + Send + 'static,
{
    let position = position.to_physical(scale);
    match status {
        CursorImageStatus::Hidden => Vec::new(),
        CursorImageStatus::Surface(surface) if surface.is_alive() => {
            let hotspot = with_states(surface, |states| {
                states
                    .data_map
                    .get::<CursorImageSurfaceData>()
                    .map(|d| d.lock().expect("not poisoned").hotspot)
            })
            .unwrap_or_default();
            let location = position - hotspot.to_f64().to_physical(scale);
            render_elements_from_surface_tree(
                renderer,
                surface,
                location.to_i32_round(),
                scale,
                1.0,
                Kind::Cursor,
            )
            .into_iter()
            .map(SceneElement::Surface)
            .collect()
        }
        CursorImageStatus::Surface(_) => named(renderer, cursors, "default", position),
        CursorImageStatus::Named(icon) => named(renderer, cursors, icon.name(), position),
    }
}

fn named<R>(
    renderer: &mut R,
    cursors: &mut Cursors,
    name: &str,
    position: Point<f64, Physical>,
) -> Vec<SceneElement<R>>
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + Send + 'static,
{
    let Some(image) = cursors.get(name) else {
        return Vec::new();
    };
    let location = position - image.hotspot.to_f64();
    MemoryRenderBufferRenderElement::from_buffer(
        renderer,
        location,
        &image.buffer,
        None,
        None,
        None,
        Kind::Cursor,
    )
    .map(SceneElement::Cursor)
    .into_iter()
    .collect()
}
