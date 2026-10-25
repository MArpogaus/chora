//! Text nodes: the core lays out and draws the text itself.
//!
//! cosmic-text shapes the text with the system fonts and with the fonts
//! of every directory of `CHORA_FONT_DIRS`. One entry per node caches
//! the layout size and the raster of one physical scale. A changed prop
//! or a new scale lays out and rasterizes again.

use std::{collections::HashMap, sync::Mutex};

use chora_scene::Node;
use cosmic_text::{
    Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache, SwashContent, SwashImage,
};
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::element::{
            Kind,
            memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
            utils::RescaleRenderElement,
        },
    },
    utils::{Point, Rectangle, Size, Transform},
};

use crate::{images::Placement, mesh::MeshRenderer, render::SceneElement};

/// The font size of a text node without `font_size`, in logical pixels.
const FONT_SIZE: f64 = 16.0;
/// The line height as a multiple of the font size.
const LINE_HEIGHT: f32 = 1.2;
/// The longest side of a text raster, in physical pixels.
const MAX_SIDE: f32 = 8192.0;

/// The props a layout depends on and the physical scale of its raster.
type Key = (
    String,
    Option<String>,
    f64,
    Option<f64>,
    Option<[f64; 4]>,
    f64,
);

/// The layouts and rasters of the text nodes, by node id.
#[derive(Default)]
pub struct Texts {
    cache: Mutex<Cache>,
}

#[derive(Default)]
struct Cache {
    fonts: Option<Fonts>,
    nodes: HashMap<String, Entry>,
}

/// The font system of the core and its glyph raster.
struct Fonts {
    system: FontSystem,
    swash: SwashCache,
}

/// One laid out node with its raster.
struct Entry {
    key: Key,
    /// The layout size in logical pixels.
    size: [f64; 2],
    /// The raster size in physical pixels.
    pixels: [f32; 2],
    buffer: MemoryRenderBuffer,
}

impl Texts {
    /// The layout size of a text node in logical pixels; `None` without
    /// text or beyond the raster limits. A text node reports it in `rects`
    /// like an image without a size.
    pub fn size(&self, node: &Node) -> Option<[f64; 2]> {
        self.laid_out(node, 1.0).map(|entry| entry.1)
    }

    /// Drops the entries of gone nodes and of other kinds.
    pub fn retain(&self, keep: impl Fn(&str) -> bool) {
        self.cache
            .lock()
            .expect("the text cache lock")
            .nodes
            .retain(|id, _| keep(id));
    }

    /// The raster and the layout size of a node at a physical scale,
    /// laid out and rasterized again when the props or the scale changed.
    fn laid_out(
        &self,
        node: &Node,
        scale: f64,
    ) -> Option<(MemoryRenderBuffer, [f64; 2], [f32; 2])> {
        let props = &node.props;
        let key = (
            props.text.clone()?,
            props.family.clone(),
            props.font_size.unwrap_or(FONT_SIZE),
            props.max_width,
            props.color,
            scale,
        );
        let mut cache = self.cache.lock().expect("the text cache lock");
        if let Some(entry) = cache.nodes.get(&node.id)
            && entry.key == key
        {
            return Some((entry.buffer.clone(), entry.size, entry.pixels));
        }
        let fonts = cache.fonts.get_or_insert_with(Fonts::new);
        let (size, pixels, buffer) = rasterize(fonts, &key)?;
        cache.nodes.insert(
            node.id.clone(),
            Entry {
                key,
                size,
                pixels,
                buffer: buffer.clone(),
            },
        );
        Some((buffer, size, pixels))
    }
}

impl Fonts {
    /// The system fonts plus every directory of `CHORA_FONT_DIRS`.
    fn new() -> Self {
        let mut system = FontSystem::new();
        for dir in std::env::var("CHORA_FONT_DIRS")
            .unwrap_or_default()
            .split(':')
        {
            if !dir.is_empty() {
                system.db_mut().load_fonts_dir(dir);
            }
        }
        Self {
            system,
            swash: SwashCache::new(),
        }
    }
}

/// Lays out and rasterizes one key into premultiplied RGBA, and returns
/// the layout size in logical pixels next to the raster size.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn rasterize(fonts: &mut Fonts, key: &Key) -> Option<([f64; 2], [f32; 2], MemoryRenderBuffer)> {
    let (text, family, font_size, max_width, color, scale) = key;
    let scale = *scale as f32;
    let size = *font_size as f32 * scale;
    let attrs = Attrs::new().family(family.as_deref().map_or(Family::SansSerif, Family::Name));
    let mut buffer = Buffer::new(&mut fonts.system, Metrics::relative(size, LINE_HEIGHT));
    buffer.set_size(max_width.map(|w| w as f32 * scale), None);
    buffer.set_text(text, &attrs, Shaping::Advanced, None);
    buffer.shape_until_scroll(&mut fonts.system, false);
    let mut width = 0.0f32;
    let mut height = 0.0f32;
    for run in buffer.layout_runs() {
        width = width.max(run.line_w);
        height = height.max(run.line_top + run.line_height);
    }
    if width <= 0.0 || height <= 0.0 || width > MAX_SIDE || height > MAX_SIDE {
        return None;
    }
    let (w, h) = (width.ceil() as usize, height.ceil() as usize);
    let mut rgba = vec![0; w * h * 4];
    for run in buffer.layout_runs() {
        for glyph in run.glyphs {
            let at = glyph.physical((0.0, run.line_y), 1.0);
            let Some(image) = fonts.swash.get_image(&mut fonts.system, at.cache_key) else {
                continue;
            };
            blit(
                &mut rgba,
                (w, h),
                (at.x, at.y),
                image,
                straight(color.as_ref()),
            );
        }
    }
    #[allow(clippy::cast_possible_wrap)] // at most MAX_SIDE
    let raster = MemoryRenderBuffer::from_slice(
        &rgba,
        Fourcc::Abgr8888,
        (w as i32, h as i32),
        1,
        Transform::Normal,
        None,
    );
    Some((
        [f64::from(width / scale), f64::from(height / scale)],
        [width, height],
        raster,
    ))
}

/// The node color as straight-alpha RGBA bytes, white by default.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn straight(color: Option<&[f64; 4]>) -> [u8; 4] {
    let [r, g, b, a] = color.copied().unwrap_or([1.0; 4]);
    let byte = |c: f64| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
    [byte(r), byte(g), byte(b), byte(a)]
}

/// Draws one glyph image into the premultiplied RGBA buffer at `x`, `y`.
/// Glyphs land on distinct pixels, so they overwrite instead of blending.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)] // glyph boxes are small, see MAX_SIDE
fn blit(
    rgba: &mut [u8],
    (width, height): (usize, usize),
    at: (i32, i32),
    image: &SwashImage,
    color: [u8; 4],
) {
    let (left, top) = (at.0 + image.placement.left, at.1 - image.placement.top);
    let (gw, gh) = (image.placement.width, image.placement.height);
    for row in 0..gh {
        let row_y = top + row as i32;
        if row_y < 0 || row_y as usize >= height {
            continue;
        }
        for col in 0..gw {
            let col_x = left + col as i32;
            if col_x < 0 || col_x as usize >= width {
                continue;
            }
            let bpp = image.data.len() / (gw as usize * gh as usize).max(1);
            let off = (row as usize * gw as usize + col as usize) * bpp;
            let value = match image.content {
                SwashContent::Color => [
                    mul(image.data[off], color[3]),
                    mul(image.data[off + 1], color[3]),
                    mul(image.data[off + 2], color[3]),
                    mul(image.data[off + 3], color[3]),
                ],
                // subpixel masks land as plain coverage.
                SwashContent::Mask | SwashContent::SubpixelMask => {
                    let cov = mul(image.data[off], color[3]);
                    [
                        mul(color[0], cov),
                        mul(color[1], cov),
                        mul(color[2], cov),
                        cov,
                    ]
                }
            };
            rgba[(row_y as usize * width + col_x as usize) * 4..][..4].copy_from_slice(&value);
        }
    }
}

/// Scales one byte by an alpha.
fn mul(c: u8, a: u8) -> u8 {
    u8::try_from((u16::from(c) * u16::from(a) + 127) / 255).unwrap_or(c)
}

/// The render element of a text node: the raster at the output scale,
/// drawn at the layout size times the node scale. `None` for a node
/// without text, without color or beyond the raster limits.
pub fn element<R>(
    texts: &Texts,
    renderer: &mut R,
    node: &Node,
    placement: &Placement,
) -> Option<SceneElement<R>>
where
    R: MeshRenderer,
{
    node.props.color?;
    let (buffer, size, pixels) = texts.laid_out(node, placement.output_scale)?;
    let src = Rectangle::from_size(Size::from((f64::from(pixels[0]), f64::from(pixels[1]))));
    #[allow(clippy::cast_possible_truncation)]
    let size = Size::from((
        (size[0] * placement.scale.0).max(1.0).round() as i32,
        (size[1] * placement.scale.1).max(1.0).round() as i32,
    ));
    MemoryRenderBufferRenderElement::from_buffer(
        renderer,
        placement.origin.to_physical(1.0),
        &buffer,
        Some(placement.alpha),
        Some(src),
        Some(size),
        Kind::Unspecified,
    )
    .ok()
    // the wrap keeps the variant apart from the cursor's element.
    .map(|element| {
        SceneElement::Text(RescaleRenderElement::from_element(
            element,
            Point::from((0, 0)),
            1.0,
        ))
    })
}

/// Whether any font folder holds a font file; tests answer `SKIP`
/// without fonts.
#[cfg(test)]
pub(crate) fn fonts_present() -> bool {
    let mut dirs: Vec<std::path::PathBuf> = std::env::var("CHORA_FONT_DIRS")
        .unwrap_or_default()
        .split(':')
        .filter(|dir| !dir.is_empty())
        .map(std::path::PathBuf::from)
        .collect();
    dirs.extend(["/usr/share/fonts".into(), "/usr/local/share/fonts".into()]);
    dirs.iter().any(|dir| has_fonts(dir, 3))
}

#[cfg(test)]
fn has_fonts(dir: &std::path::Path, depth: u8) -> bool {
    let font_file = |entry: &std::fs::DirEntry| {
        entry
            .path()
            .extension()
            .is_some_and(|ext| ext == "ttf" || ext == "otf" || ext == "ttc" || ext == "otc")
    };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|entry| {
        font_file(&entry)
            || (depth > 1 && entry.path().is_dir() && has_fonts(&entry.path(), depth - 1))
    })
}

#[cfg(test)]
mod tests {
    use chora_scene::{NodeKind, Props};

    use super::{Texts, fonts_present};

    fn text_node(props: Props) -> chora_scene::Node {
        chora_scene::Node {
            id: "t".into(),
            kind: NodeKind::Text,
            owner: None,
            parent: None,
            children: Vec::new(),
            props,
            surface: None,
            to: None,
        }
    }

    #[test]
    fn a_text_node_without_text_has_no_size() {
        let texts = Texts::default();
        assert!(texts.size(&text_node(Props::default())).is_none());
    }

    #[test]
    fn the_layout_grows_with_the_font_size() {
        if !fonts_present() {
            println!("SKIP: no fonts for the layout");
            return;
        }
        let texts = Texts::default();
        let small = text_node(Props {
            text: Some("Hi".into()),
            font_size: Some(10.0),
            ..Props::default()
        });
        let [w1, h1] = texts.size(&small).expect("a layout");
        let big = text_node(Props {
            font_size: Some(20.0),
            ..small.props.clone()
        });
        let [w2, h2] = texts.size(&big).expect("a layout again");
        assert!(w2 > w1 && h2 > h1, "{w1}x{h1} then {w2}x{h2}");
    }

    #[test]
    fn max_width_only_wraps() {
        if !fonts_present() {
            println!("SKIP: no fonts for the layout");
            return;
        }
        let words = "alpha beta gamma delta epsilon zeta eta theta iota kappa";
        let wide = text_node(Props {
            text: Some(words.into()),
            font_size: Some(20.0),
            ..Props::default()
        });
        let [w1, h1] = layout_of(&wide);
        let narrow = text_node(Props {
            max_width: Some(w1 / 2.0),
            ..wide.props.clone()
        });
        let [w2, h2] = layout_of(&narrow);
        assert!(w2 <= w1 / 2.0 + 1.0, "wrapped to {w2} of {}", w1 / 2.0);
        assert!(h2 > h1 * 1.5, "{h1} to {h2}");
    }

    fn layout_of(node: &chora_scene::Node) -> [f64; 2] {
        Texts::default().size(node).expect("a layout")
    }
}
