//! Text nodes: the core lays out and draws the text itself.
//!
//! cosmic-text shapes the text with the system fonts and with the fonts
//! of every directory of `CHORA_FONT_DIRS`. One entry per node caches
//! the layout size and, per physical scale, the raster. A changed prop
//! lays out again; a new scale rasterizes again.

use std::{collections::HashMap, sync::Mutex};

use chora_scene::Node;
use cosmic_text::{
    Attrs, Buffer, CacheKey, CacheKeyFlags, Family, FontSystem, Metrics, Shaping, SwashCache,
    SwashContent, SwashImage, fallback::FontFallbackIter,
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
/// The largest raster of a text node, in bytes, per physical scale;
/// a raster beyond it yields no size and no draw at that scale.
const MAX_RASTER_BYTES: usize = 16 * 1024 * 1024;

/// The props a layout depends on.
type Key = (String, Option<String>, f64, Option<f64>, Option<[f64; 4]>);

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

/// The layout and the rasters of one node.
struct Entry {
    /// The props the layout below was made from.
    key: Key,
    /// The layout size in logical pixels; `None` when the raster at
    /// scale 1 would be beyond [`MAX_RASTER_BYTES`].
    size: Option<[f64; 2]>,
    /// The raster per physical scale (its bits): size in physical
    /// pixels and buffer.
    rasters: HashMap<u64, ([f32; 2], MemoryRenderBuffer)>,
}

impl Texts {
    /// The layout size of a text node in logical pixels; `None` without
    /// text or beyond the raster limits. A text node reports it in `rects`
    /// like an image without a size.
    pub fn size(&self, node: &Node) -> Option<[f64; 2]> {
        let key = props_key(node)?;
        let mut cache = self.cache.lock().expect("the text cache lock");
        if let Some(entry) = cache.nodes.get(&node.id)
            && entry.key == key
        {
            return entry.size;
        }
        let fonts = cache.fonts.get_or_insert_with(Fonts::new);
        let size = layout(fonts, &key, 1.0).map(|(_, size, _)| size);
        cache.nodes.insert(
            node.id.clone(),
            Entry {
                key,
                size,
                rasters: HashMap::new(),
            },
        );
        size
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
        let key = props_key(node)?;
        let scale_key = scale.to_bits();
        let mut cache = self.cache.lock().expect("the text cache lock");
        if let Some(entry) = cache.nodes.get(&node.id)
            && entry.key == key
            && let Some((pixels, buffer)) = entry.rasters.get(&scale_key)
            && let Some(size) = entry.size
        {
            return Some((buffer.clone(), size, *pixels));
        }
        let fonts = cache.fonts.get_or_insert_with(Fonts::new);
        let (size, pixels, buffer) = rasterize(fonts, &key, scale)?;
        match cache.nodes.get_mut(&node.id) {
            Some(entry) if entry.key == key => {
                let drawn = entry.size.unwrap_or(size);
                entry.rasters.insert(scale_key, (pixels, buffer.clone()));
                Some((buffer, drawn, pixels))
            }
            _ => {
                let mut rasters = HashMap::new();
                rasters.insert(scale_key, (pixels, buffer.clone()));
                cache.nodes.insert(
                    node.id.clone(),
                    Entry {
                        key,
                        size: Some(size),
                        rasters,
                    },
                );
                Some((buffer, size, pixels))
            }
        }
    }
}

/// The props a layout depends on; `None` for a node without text.
fn props_key(node: &Node) -> Option<Key> {
    let props = &node.props;
    Some((
        props.text.clone()?,
        props.family.clone(),
        props.font_size.unwrap_or(FONT_SIZE),
        props.max_width,
        props.color,
    ))
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

/// Whether a raster of `w` by `h` RGBA pixels stays within the limit.
fn within_limit(w: usize, h: usize) -> bool {
    w.checked_mul(h)
        .is_some_and(|px| px <= MAX_RASTER_BYTES / 4)
}

/// Lays out one key at a physical scale and measures it; `None` for
/// empty text or a raster beyond [`MAX_RASTER_BYTES`]. Returns the
/// buffer, the layout size in logical pixels and the raster size.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn layout(fonts: &mut Fonts, key: &Key, scale: f32) -> Option<(Buffer, [f64; 2], [f32; 2])> {
    let (text, family, font_size, max_width, _) = key;
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
    if width <= 0.0 || height <= 0.0 || !within_limit(width.ceil() as usize, height.ceil() as usize)
    {
        return None;
    }
    Some((
        buffer,
        [f64::from(width / scale), f64::from(height / scale)],
        [width, height],
    ))
}

/// Rasterizes one key at a physical scale into premultiplied RGBA and
/// wraps it in a render buffer.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)] // at most the raster limit
fn rasterize(
    fonts: &mut Fonts,
    key: &Key,
    scale: f64,
) -> Option<([f64; 2], [f32; 2], MemoryRenderBuffer)> {
    let (size, [width, height], rgba) = paint(fonts, key, scale)?;
    let raster = MemoryRenderBuffer::from_slice(
        &rgba,
        Fourcc::Abgr8888,
        (width as i32, height as i32),
        1,
        Transform::Normal,
        None,
    );
    Some((size, [width, height], raster))
}

/// Lays out one key at a physical scale and paints its glyphs into
/// premultiplied RGBA, next to the layout size in logical pixels and
/// the raster size.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn paint(fonts: &mut Fonts, key: &Key, scale: f64) -> Option<([f64; 2], [f32; 2], Vec<u8>)> {
    let color = straight(key.4.as_ref());
    let (buffer, size, [width, height]) = layout(fonts, key, scale as f32)?;
    let (w, h) = (width.ceil() as usize, height.ceil() as usize);
    let mut rgba = vec![0; w * h * 4];
    let blank = paint_runs(fonts, &mut rgba, &buffer, (w, h), key.1.as_deref(), color);
    if blank > 0 {
        tracing::warn!(text = %key.0, blank, "glyphs laid out but drawn by no font");
    }
    Some((size, [width, height], rgba))
}

/// Paints all layout runs into the premultiplied RGBA buffer; the
/// number of glyphs that no font drew.
#[allow(clippy::too_many_arguments)]
fn paint_runs(
    fonts: &mut Fonts,
    rgba: &mut [u8],
    buffer: &Buffer,
    raster: (usize, usize),
    family: Option<&str>,
    color: [u8; 4],
) -> u32 {
    let mut blank = 0;
    for run in buffer.layout_runs() {
        for glyph in run.glyphs {
            let at = glyph.physical((0.0, run.line_y), 1.0);
            let word = run.text.get(glyph.start..glyph.end).unwrap_or("");
            let drawn = paint_glyph(
                fonts,
                rgba,
                raster,
                (at.x, at.y),
                at.cache_key,
                family,
                word,
                color,
            );
            blank += u32::from(!drawn);
        }
    }
    blank
}

/// Draws one glyph into the raster; when the laid out font draws
/// nothing for it, for example a `COLRv1` glyph, asks the other fonts
/// of the database. Whether any pixel landed.
#[allow(clippy::too_many_arguments)]
fn paint_glyph(
    fonts: &mut Fonts,
    rgba: &mut [u8],
    raster: (usize, usize),
    at: (i32, i32),
    cache_key: CacheKey,
    family: Option<&str>,
    word: &str,
    color: [u8; 4],
) -> bool {
    for key in std::iter::once(cache_key).chain(redraw(fonts, family, word, cache_key)) {
        if let Some(image) = inked(fonts.swash.get_image(&mut fonts.system, key).as_ref()) {
            blit(rgba, raster, at, image, color);
            return true;
        }
    }
    false
}

/// An image with at least one pixel; swash answers an empty mask for
/// a glyph it lays out but cannot draw.
fn inked(image: Option<&SwashImage>) -> Option<&SwashImage> {
    let image = image?;
    (image.placement.width > 0 && image.placement.height > 0).then_some(image)
}

/// The same word in the other fonts of the database, in fallback
/// order, as cache keys; empty when no other font has the first
/// character of the word.
fn redraw(fonts: &mut Fonts, family: Option<&str>, word: &str, from: CacheKey) -> Vec<CacheKey> {
    let Some(first) = word.chars().next() else {
        return Vec::new();
    };
    let attrs = Attrs::new().family(family.map_or(Family::SansSerif, Family::Name));
    let matches = fonts.system.get_font_matches(&attrs);
    let iter = FontFallbackIter::new(
        &mut fonts.system,
        &matches,
        &[],
        &[],
        word,
        from.font_weight,
    );
    let mut keys = Vec::new();
    for font in iter {
        if font.id() == from.font_id {
            continue;
        }
        let glyph = font.as_swash().charmap().map(u32::from(first));
        if glyph != 0 {
            keys.push(CacheKey {
                font_id: font.id(),
                glyph_id: glyph,
                font_size_bits: from.font_size_bits,
                x_bin: from.x_bin,
                y_bin: from.y_bin,
                font_weight: from.font_weight,
                flags: CacheKeyFlags::empty(),
            });
        }
    }
    keys
}

/// The node color as straight-alpha RGBA bytes, white by default.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn straight(color: Option<&[f64; 4]>) -> [u8; 4] {
    let [r, g, b, a] = color.copied().unwrap_or([1.0; 4]);
    let byte = |c: f64| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
    [byte(r), byte(g), byte(b), byte(a)]
}

/// Draws one glyph image into the premultiplied RGBA buffer at `x`, `y`.
/// Overlapping pixels keep the larger value per channel, so a combining
/// mark keeps the pixels of the glyph below it.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)] // glyph boxes are small, see MAX_RASTER_BYTES
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
                // color data is straight alpha; the buffer is premultiplied.
                SwashContent::Color => {
                    let alpha = mul(image.data[off + 3], color[3]);
                    [
                        mul(image.data[off], alpha),
                        mul(image.data[off + 1], alpha),
                        mul(image.data[off + 2], alpha),
                        alpha,
                    ]
                }
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
            blend(
                &mut *rgba,
                (row_y as usize * width + col_x as usize) * 4,
                value,
            );
        }
    }
}

/// Composites one RGBA pixel onto the pixel of `rgba` at `at`.
fn blend(rgba: &mut [u8], at: usize, over: [u8; 4]) {
    for (here, new) in rgba[at..at + 4].iter_mut().zip(over) {
        *here = (*here).max(new);
    }
}

/// Scales one byte by an alpha.
fn mul(c: u8, a: u8) -> u8 {
    u8::try_from((u16::from(c) * u16::from(a) + 127) / 255).unwrap_or(c)
}

/// The render element of a text node: the raster at the output scale,
/// drawn at the layout size times the node scale. `None` for a node
/// without text or beyond the raster limits.
pub fn element<R>(
    texts: &Texts,
    renderer: &mut R,
    node: &Node,
    placement: &Placement,
) -> Option<SceneElement<R>>
where
    R: MeshRenderer,
{
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
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(std::path::PathBuf::from(home).join(".local/share/fonts"));
    }
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
    use cosmic_text::{Placement, SwashContent, SwashImage};

    use super::{Texts, blend, blit, fonts_present};

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

    #[test]
    fn an_overlapping_glyph_keeps_the_pixel_below() {
        let mut rgba = [255, 0, 0, 255];
        blend(&mut rgba, 0, [0, 255, 0, 128]);
        assert_eq!(rgba, [255, 255, 0, 255]);
    }

    #[test]
    fn a_color_glyph_lands_premultiplied() {
        let mut rgba = [0; 4];
        let image = SwashImage {
            content: SwashContent::Color,
            placement: Placement {
                width: 1,
                height: 1,
                ..Placement::default()
            },
            data: vec![255, 0, 0, 128],
            ..SwashImage::default()
        };
        blit(&mut rgba, (1, 1), (0, 0), &image, [255, 255, 255, 255]);
        assert_eq!(rgba, [128, 0, 0, 128], "straight alpha at half coverage");
    }

    #[test]
    fn reading_the_size_keeps_the_raster_of_a_scale() {
        if !fonts_present() {
            println!("SKIP: no fonts for the layout");
            return;
        }
        let texts = Texts::default();
        let node = text_node(Props {
            text: Some("Hi".into()),
            ..Props::default()
        });
        texts.laid_out(&node, 2.0).expect("a raster at scale 2");
        texts.size(&node).expect("a size");
        let cache = texts.cache.lock().expect("the text cache lock");
        let entry = cache.nodes.get("t").expect("an entry for the node");
        assert!(
            entry.rasters.contains_key(&2.0f64.to_bits()),
            "reading the size evicted the raster of scale 2"
        );
    }

    #[test]
    fn an_emoji_node_inks_its_pixels() {
        if !fonts_present() {
            println!("SKIP: no fonts for the layout");
            return;
        }
        let node = text_node(Props {
            text: Some("🙂".into()),
            font_size: Some(40.0),
            ..Props::default()
        });
        let key = super::props_key(&node).expect("the props of a text node");
        let mut fonts = super::Fonts::new();
        let (_, _, rgba) = super::paint(&mut fonts, &key, 1.0).expect("a raster of the emoji");
        let inked = rgba.chunks(4).filter(|px| px[3] > 0).count();
        assert!(inked > 100, "the emoji node painted only {inked} pixels");
    }

    #[test]
    fn a_raster_beyond_the_byte_limit_has_no_size() {
        if !fonts_present() {
            println!("SKIP: no fonts for the layout");
            return;
        }
        let fits = text_node(Props {
            text: Some("M".into()),
            font_size: Some(1200.0),
            ..Props::default()
        });
        assert!(
            Texts::default().size(&fits).is_some(),
            "1200 px still rasterizes"
        );
        let huge = text_node(Props {
            font_size: Some(3000.0),
            ..fits.props.clone()
        });
        assert!(
            Texts::default().size(&huge).is_none(),
            "3000 px passes 16 MiB"
        );
    }
}
