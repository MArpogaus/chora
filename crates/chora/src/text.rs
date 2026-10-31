//! Text nodes: the core lays out and draws the text itself.
//!
//! cosmic-text shapes the text with the system fonts and with the fonts
//! of every directory of `CHORA_FONT_DIRS`. One entry per node caches
//! the layout size and, per physical scale, the raster. A changed prop
//! lays out again; a new scale rasterizes again.

use std::{
    collections::{BTreeMap, HashMap},
    sync::{Mutex, PoisonError},
    thread::JoinHandle,
};

use chora_scene::{FontStyle, Node};
use cosmic_text::{
    Attrs, Buffer, CacheKey, CacheKeyFlags, Family, FeatureTag, FontFeatures, FontSystem, Metrics,
    Shaping, Style, SwashCache, SwashContent, SwashImage, Weight, fallback::FontFallbackIter,
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
const LINE_HEIGHT: f64 = 1.2;
/// The largest raster of a text node, in bytes, per physical scale;
/// a raster beyond it yields no size and no draw at that scale.
const MAX_RASTER_BYTES: usize = 16 * 1024 * 1024;
/// The rasters of all text nodes together, in bytes; a raster beyond it
/// draws nothing until its node changes.
const MAX_TOTAL_BYTES: usize = 256 * 1024 * 1024;
/// The physical scales a node keeps rasters for.
const MAX_SCALES: usize = 4;

/// The props a layout depends on.
#[derive(Clone, PartialEq)]
struct Key {
    text: String,
    family: Option<String>,
    font_size: f64,
    max_width: Option<f64>,
    color: Option<[f64; 4]>,
    line_height: f64,
    weight: Option<u16>,
    style: Option<FontStyle>,
    letter_spacing: Option<f64>,
    features: BTreeMap<String, u32>,
}

/// The layouts and rasters of the text nodes, by node id.
#[derive(Default)]
pub struct Texts {
    cache: Mutex<Cache>,
}

struct Cache {
    /// The font database, loaded on its own thread from the start, so
    /// that the first text node does not stall the event loop as long.
    loading: Option<JoinHandle<Fonts>>,
    fonts: Option<Fonts>,
    nodes: HashMap<String, Entry>,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            loading: Some(std::thread::spawn(Fonts::new)),
            fonts: None,
            nodes: HashMap::new(),
        }
    }
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
    /// pixels and buffer; `None` for a raster that failed, so that it is
    /// not tried again each frame.
    rasters: HashMap<u64, Option<Raster>>,
}

type Raster = ([f32; 2], MemoryRenderBuffer);

impl Cache {
    /// The fonts and the entry of a node for its current props.
    fn entry(&mut self, id: &str, key: Key) -> (&mut Fonts, &mut Entry) {
        let loading = self.loading.take();
        let fonts = self.fonts.get_or_insert_with(|| {
            loading
                .and_then(|thread| thread.join().ok())
                .unwrap_or_else(Fonts::new)
        });
        let fresh = |fonts: &mut Fonts, key: Key| Entry {
            size: layout(fonts, &key, 1.0).map(|(_, size, _)| size),
            key,
            rasters: HashMap::new(),
        };
        let entry = self
            .nodes
            .entry(id.to_owned())
            .or_insert_with(|| fresh(fonts, key.clone()));
        if entry.key != key {
            *entry = fresh(fonts, key);
        }
        (fonts, entry)
    }

    /// The bytes all rasters hold.
    fn used(&self) -> usize {
        let rasters = self
            .nodes
            .values()
            .flat_map(|e| e.rasters.values().flatten());
        rasters.map(|(pixels, _)| bytes(*pixels)).sum()
    }
}

/// The bytes of a raster of this size.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn bytes([w, h]: [f32; 2]) -> usize {
    w as usize * h as usize * 4
}

impl Texts {
    /// The layout size of a text node in logical pixels; `None` without
    /// text or beyond the raster limits. A text node reports it in `rects`
    /// like an image without a size.
    pub fn size(&self, node: &Node) -> Option<[f64; 2]> {
        let key = props_key(node)?;
        let mut cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
        cache.entry(&node.id, key).1.size
    }

    /// Drops the entries of gone nodes and of other kinds.
    pub fn retain(&self, keep: impl Fn(&str) -> bool) {
        self.cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
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
        let mut cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
        let (fonts, entry) = cache.entry(&node.id, key);
        let size = entry.size?;
        if let Some(raster) = entry.rasters.get(&scale_key) {
            let (pixels, buffer) = raster.clone()?;
            return Some((buffer, size, pixels));
        }
        let raster =
            rasterize(fonts, &entry.key, scale).map(|(_, pixels, buffer)| (pixels, buffer));
        let used = cache.used();
        let raster = raster.filter(|(pixels, _)| {
            let fits = used + bytes(*pixels) <= MAX_TOTAL_BYTES;
            if !fits {
                tracing::warn!(node = node.id, "the text rasters of all nodes pass 256 MiB");
            }
            fits
        });
        let entry = cache.nodes.get_mut(&node.id)?;
        // An output that changed its scale leaves its old raster.
        if entry.rasters.len() >= MAX_SCALES {
            entry.rasters.clear();
        }
        entry.rasters.insert(scale_key, raster.clone());
        let (pixels, buffer) = raster?;
        Some((buffer, size, pixels))
    }
}

/// The props a layout depends on; `None` for a node without text.
fn props_key(node: &Node) -> Option<Key> {
    let props = &node.props;
    Some(Key {
        text: props.text.clone()?,
        family: props.family.clone(),
        font_size: props.font_size.unwrap_or(FONT_SIZE),
        max_width: props.max_width,
        color: props.color,
        line_height: props.line_height.unwrap_or(LINE_HEIGHT),
        weight: props.weight,
        style: props.style,
        letter_spacing: props.letter_spacing,
        features: props.features.clone(),
    })
}

/// The attributes cosmic-text shapes a key with.
#[allow(clippy::cast_possible_truncation)] // a letter spacing within 1 em
fn attrs(key: &Key) -> Attrs<'_> {
    let mut features = FontFeatures::new();
    for (tag, value) in &key.features {
        // The scene checks for four bytes.
        if let Ok(tag) = <&[u8; 4]>::try_from(tag.as_bytes()) {
            features.set(FeatureTag::new(tag), *value);
        }
    }
    let style = match key.style {
        Some(FontStyle::Italic) => Style::Italic,
        _ => Style::Normal,
    };
    let family = key
        .family
        .as_deref()
        .map_or(Family::SansSerif, Family::Name);
    let attrs = Attrs::new()
        .family(family)
        .weight(key.weight.map_or(Weight::NORMAL, Weight))
        .style(style)
        .font_features(features);
    match key.letter_spacing {
        Some(spacing) => attrs.letter_spacing(spacing as f32),
        None => attrs,
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
    // Without any font, cosmic-text panics in the shaping.
    if fonts.system.db().is_empty() {
        return None;
    }
    let size = key.font_size as f32 * scale;
    let metrics = Metrics::relative(size, key.line_height as f32);
    let mut buffer = Buffer::new(&mut fonts.system, metrics);
    buffer.set_size(key.max_width.map(|w| w as f32 * scale), None);
    buffer.set_text(&key.text, &attrs(key), Shaping::Advanced, None);
    buffer.shape_until_scroll(&mut fonts.system, false);
    let mut width = 0.0f32;
    let mut height = 0.0f32;
    for run in buffer.layout_runs() {
        width = width.max(run.line_w);
        height = height.max(run.line_top + run.line_height);
    }
    // Whole pixels: the raster and the render buffer must agree on its
    // row length, or each row starts off by a part of a pixel.
    let (width, height) = (width.ceil(), height.ceil());
    if width <= 0.0 || height <= 0.0 || !within_limit(width as usize, height as usize) {
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
    let color = straight(key.color.as_ref());
    let (buffer, size, [width, height]) = layout(fonts, key, scale as f32)?;
    let (w, h) = (width as usize, height as usize);
    let mut rgba = vec![0; w * h * 4];
    let blank = paint_runs(
        fonts,
        &mut rgba,
        &buffer,
        (w, h),
        key.family.as_deref(),
        color,
    );
    if blank > 0 {
        tracing::warn!(text = %key.text, blank, "glyphs laid out but drawn by no font");
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
            let word = run.text.get(glyph.start..glyph.end).unwrap_or("");
            if word.trim().is_empty() {
                continue; // a space lays out with no image and draws none
            }
            let at = glyph.physical((0.0, run.line_y), 1.0);
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
    // Uncached: the node keeps its raster, and a cache of each glyph per
    // size would grow with every font_size a client tries.
    let mut draw = |fonts: &mut Fonts, key| {
        let image = fonts.swash.get_image_uncached(&mut fonts.system, key);
        inked(image.as_ref()).map(|image| blit(rgba, raster, at, image, color))
    };
    if draw(fonts, cache_key).is_some() {
        return true;
    }
    // The fallback search walks the whole database, so only after the
    // laid out font drew nothing.
    redraw(fonts, family, word, cache_key)
        .into_iter()
        .any(|key| draw(fonts, key).is_some())
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
#[allow(clippy::print_stdout, reason = "a test without fonts prints SKIP")]
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

    /// A machine without fonts draws no text and does not crash.
    #[test]
    fn no_font_gives_no_layout() {
        let mut fonts = super::Fonts {
            system: cosmic_text::FontSystem::new_with_locale_and_db(
                "en-US".into(),
                cosmic_text::fontdb::Database::new(),
            ),
            swash: cosmic_text::SwashCache::new(),
        };
        let node = text_node(Props {
            text: Some("Hi".into()),
            ..Props::default()
        });
        let key = super::props_key(&node).expect("the props of a text node");
        assert!(super::layout(&mut fonts, &key, 1.0).is_none());
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
    fn the_type_props_reach_the_shaping() {
        let node = text_node(Props {
            text: Some("Hi".into()),
            weight: Some(700),
            style: Some(chora_scene::FontStyle::Italic),
            letter_spacing: Some(0.1),
            features: [("smcp".into(), 1)].into(),
            ..Props::default()
        });
        let key = super::props_key(&node).expect("the props of a text node");
        let attrs = super::attrs(&key);
        assert_eq!(attrs.weight, cosmic_text::Weight(700));
        assert_eq!(attrs.style, cosmic_text::Style::Italic);
        assert_eq!(
            attrs.letter_spacing_opt,
            Some(cosmic_text::LetterSpacing(0.1))
        );
        let smcp = cosmic_text::Feature {
            tag: cosmic_text::FeatureTag::SMALL_CAPS,
            value: 1,
        };
        assert_eq!(attrs.font_features.features, vec![smcp]);
    }

    #[test]
    fn letter_spacing_and_line_height_grow_the_layout() {
        if !fonts_present() {
            println!("SKIP: no fonts for the layout");
            return;
        }
        let plain = text_node(Props {
            text: Some("one\ntwo".into()),
            font_size: Some(20.0),
            ..Props::default()
        });
        let [w1, h1] = layout_of(&plain);
        let spaced = text_node(Props {
            letter_spacing: Some(0.2),
            line_height: Some(2.4),
            ..plain.props.clone()
        });
        let [w2, h2] = layout_of(&spaced);
        // Three letters of 0.2 em of 20 px each.
        assert!(w2 >= w1 + 11.0, "{w1} to {w2}");
        assert!((h2 - 2.0 * h1).abs() <= 1.0, "{h1} to {h2}");
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
            text: Some("🙂 🙂".into()),
            font_size: Some(40.0),
            ..Props::default()
        });
        let key = super::props_key(&node).expect("the props of a text node");
        let mut fonts = super::Fonts::new();
        let (_, _, rgba) = super::paint(&mut fonts, &key, 1.0).expect("a raster of the emoji");
        let inked = rgba.chunks(4).filter(|px| px[3] > 0).count();
        assert!(inked > 100, "the emoji node painted only {inked} pixels");
    }

    /// The render buffer takes the raster size as whole pixels; a part
    /// of a pixel in it shifts each row and slants the text.
    #[test]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn the_raster_has_whole_pixels() {
        if !fonts_present() {
            println!("SKIP: no fonts for the layout");
            return;
        }
        let node = text_node(Props {
            text: Some("Hi there".into()),
            font_size: Some(13.0),
            ..Props::default()
        });
        let key = super::props_key(&node).expect("the props of a text node");
        let mut fonts = super::Fonts::new();
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let (_, [w, h], rgba) = super::paint(&mut fonts, &key, scale).expect("a raster");
            assert!(w.fract() == 0.0 && h.fract() == 0.0, "{w}x{h} at {scale}");
            assert_eq!(rgba.len(), w as usize * h as usize * 4);
        }
    }

    /// A text that failed at a scale keeps its failure, so that the next
    /// frame does not lay it out again.
    #[test]
    fn a_failed_raster_stays_cached() {
        if !fonts_present() {
            println!("SKIP: no fonts for the layout");
            return;
        }
        let texts = Texts::default();
        let node = text_node(Props {
            text: Some("MMM".into()),
            font_size: Some(1000.0),
            ..Props::default()
        });
        texts.size(&node).expect("fits at scale 1");
        assert!(texts.laid_out(&node, 3.0).is_none(), "passes 16 MiB at 3");
        let cache = texts.cache.lock().expect("the text cache lock");
        assert!(matches!(
            cache.nodes["t"].rasters.get(&3.0f64.to_bits()),
            Some(None)
        ));
    }

    #[test]
    fn a_raster_beyond_the_byte_limit_has_no_size() {
        if !fonts_present() {
            println!("SKIP: no fonts for the layout");
            return;
        }
        let fits = text_node(Props {
            text: Some("M".into()),
            font_size: Some(1000.0),
            ..Props::default()
        });
        assert!(
            Texts::default().size(&fits).is_some(),
            "one M of 1000 px still rasterizes"
        );
        let huge = text_node(Props {
            text: Some("MMMMMMMM".into()),
            ..fits.props.clone()
        });
        assert!(
            Texts::default().size(&huge).is_none(),
            "eight of them pass 16 MiB"
        );
    }
}
