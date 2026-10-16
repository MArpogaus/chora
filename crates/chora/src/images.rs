//! Image nodes: PNG files that plugins name by an absolute path, for
//! example the parts of a window border.
//!
//! The core loads a file when a transaction names it, and again when the
//! file changed since. A file that fails to load rejects the transaction.
//! An image with a slice draws as up to nine cells: the corners keep their
//! size, the edges stretch in one direction and the center in both.

use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufReader,
    os::unix::fs::OpenOptionsExt,
    time::SystemTime,
};

use chora_scene::{Command, NodeKind, Scene};
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            ImportMem, Renderer,
            element::{
                Element, Id, Kind, RenderElement, UnderlyingStorage,
                memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
            },
            utils::{CommitCounter, DamageSet, OpaqueRegions},
        },
    },
    utils::{Buffer as BufferCoords, Logical, Physical, Point, Rectangle, Scale, Size, Transform},
};

/// The longest side of an image, in pixels.
const MAX_SIDE: u32 = 8192;
/// The largest image file.
const MAX_FILE: u64 = 32 << 20;
/// The most images in the cache at once.
const MAX_IMAGES: usize = 1024;
/// The most bytes of decoded pixels of all images.
const MAX_DECODED: u64 = 512 << 20;

/// A loaded image.
pub struct Image {
    buffer: MemoryRenderBuffer,
    /// Width and height in pixels.
    pub size: [u32; 2],
    modified: Option<SystemTime>,
    /// New with each load, so that the cells of a reloaded image get new
    /// ids and show again.
    pub version: Id,
}

/// The loaded images by path.
#[derive(Default)]
pub struct Images {
    cache: HashMap<String, Image>,
}

impl Images {
    /// Loads the images that the commands name, unless the cache holds them
    /// unchanged.
    pub fn load_for(&mut self, commands: &[Command]) -> Result<(), String> {
        for command in commands {
            let (Command::Create { props, .. }
            | Command::Update { props, .. }
            | Command::Animate { props, .. }) = command
            else {
                continue;
            };
            if let Some(path) = &props.image {
                self.load(path)?;
            }
        }
        Ok(())
    }

    fn load(&mut self, path: &str) -> Result<(), String> {
        let (file, meta) = open_file(path, MAX_FILE)?;
        let modified = meta.modified().ok();
        if modified.is_some() && self.cache.get(path).is_some_and(|i| i.modified == modified) {
            return Ok(());
        }
        if !self.cache.contains_key(path) && self.cache.len() >= MAX_IMAGES {
            return Err(format!("{path}: the core holds {MAX_IMAGES} images"));
        }
        let others: u64 = self
            .cache
            .iter()
            .filter(|(p, _)| p.as_str() != path)
            .map(|(_, i)| u64::from(i.size[0]) * u64::from(i.size[1]) * 4)
            .sum();
        let budget = MAX_DECODED.saturating_sub(others);
        let (pixels, size) = decode(file, budget).map_err(|e| format!("{path}: {e}"))?;
        #[allow(clippy::cast_possible_wrap)] // at most MAX_SIDE
        let buffer = MemoryRenderBuffer::from_slice(
            &pixels,
            Fourcc::Abgr8888,
            (size[0] as i32, size[1] as i32),
            1,
            Transform::Normal,
            None,
        );
        let image = Image {
            buffer,
            size,
            modified,
            version: Id::new(),
        };
        self.cache.insert(path.into(), image);
        Ok(())
    }

    /// Drops the images that no node shows.
    pub fn prune(&mut self, scene: &Scene) {
        let used: HashSet<&str> = scene
            .nodes()
            .filter(|n| n.kind == NodeKind::Image)
            .filter_map(|n| n.props.image.as_deref())
            .collect();
        self.cache.retain(|path, _| used.contains(path.as_str()));
    }

    /// The image of a path, if it is loaded.
    pub fn get(&self, path: &str) -> Option<&Image> {
        self.cache.get(path)
    }
}

/// Opens a regular file of at most `max` bytes that a client named. A FIFO
/// or a device would block the event loop, or never end: the open does not
/// wait for a writer, and the check runs on the open file, so a swap of
/// the path in between changes nothing.
pub fn open_file(path: &str, max: u64) -> Result<(File, std::fs::Metadata), String> {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(
            smithay::reexports::rustix::fs::OFlags::NONBLOCK
                .bits()
                .cast_signed(),
        )
        .open(path)
        .map_err(|e| format!("{path}: {e}"))?;
    let meta = file.metadata().map_err(|e| format!("{path}: {e}"))?;
    if !meta.is_file() {
        return Err(format!("{path}: not a regular file"));
    }
    if meta.len() > max {
        return Err(format!("{path}: larger than {max} bytes"));
    }
    Ok((file, meta))
}

/// Decodes a PNG file into premultiplied RGBA pixels, if they fit into a
/// budget of bytes.
fn decode(file: File, budget: u64) -> Result<(Vec<u8>, [u32; 2]), String> {
    let mut decoder = png::Decoder::new(BufReader::new(file));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let (w, h) = reader.info().size();
    if w == 0 || h == 0 || w > MAX_SIDE || h > MAX_SIDE {
        return Err(format!("{w}x{h} pixels; at most {MAX_SIDE} on a side"));
    }
    if u64::from(w) * u64::from(h) * 4 > budget {
        return Err(format!(
            "{w}x{h} pixels exceed the {MAX_DECODED} bytes of all images"
        ));
    }
    let len = reader.output_buffer_size().ok_or("too large")?;
    let mut data = vec![0; len];
    let info = reader.next_frame(&mut data).map_err(|e| e.to_string())?;
    let rows = data
        .chunks(info.line_size)
        .take(h as usize)
        .map(|row| &row[..row.len().min(w as usize * info.color_type.samples())]);
    let mut pixels = Vec::with_capacity(w as usize * h as usize * 4);
    for row in rows {
        for px in row.chunks(info.color_type.samples()) {
            pixels.extend(rgba(info.color_type, px));
        }
    }
    Ok((pixels, [w, h]))
}

/// One pixel as premultiplied RGBA.
fn rgba(color: png::ColorType, px: &[u8]) -> [u8; 4] {
    let [r, g, b, a] = match (color, px) {
        (png::ColorType::Rgba, [r, g, b, a]) => [*r, *g, *b, *a],
        (png::ColorType::Rgb, [r, g, b]) => [*r, *g, *b, 255],
        (png::ColorType::GrayscaleAlpha, [v, a]) => [*v, *v, *v, *a],
        (png::ColorType::Grayscale, [v]) => [*v, *v, *v, 255],
        _ => [0, 0, 0, 0],
    };
    let premultiply = |c: u8| u8::try_from((u16::from(c) * u16::from(a) + 127) / 255).unwrap_or(c);
    [premultiply(r), premultiply(g), premultiply(b), a]
}

/// A cell of an image: the source in pixels of the file and the
/// destination in node coordinates.
pub type Cell = (Rectangle<f64, Logical>, Rectangle<f64, Logical>);

/// The cells of an image: the source in pixels of the file and the
/// destination in node coordinates. Without a slice the image is one cell.
pub fn cells(file: [u32; 2], size: [f64; 2], slice: Option<[f64; 4]>) -> Vec<Cell> {
    let file = [f64::from(file[0]), f64::from(file[1])];
    let [left, top, right, bottom] = slice.unwrap_or_default();
    let axis = |len: f64, size: f64, a: f64, b: f64| {
        // The insets shrink together when they do not fit.
        let fit = (len.min(size) / (a + b)).min(1.0);
        let (a, b) = if a + b > 0.0 {
            (a * fit, b * fit)
        } else {
            (0.0, 0.0)
        };
        ([0.0, a, len - b, len], [0.0, a, size - b, size])
    };
    let (sx, dx) = axis(file[0], size[0], left, right);
    let (sy, dy) = axis(file[1], size[1], top, bottom);
    let span = |e: [f64; 4], i: usize| (e[i], e[i + 1] - e[i]);
    let mut out = Vec::new();
    for row in 0..3 {
        for col in 0..3 {
            let (src, dst) = (
                (span(sx, col), span(sy, row)),
                (span(dx, col), span(dy, row)),
            );
            if src.0.1 > 0.0 && src.1.1 > 0.0 && dst.0.1 > 0.0 && dst.1.1 > 0.0 {
                out.push((
                    Rectangle::new((src.0.0, src.1.0).into(), (src.0.1, src.1.1).into()),
                    Rectangle::new((dst.0.0, dst.1.0).into(), (dst.0.1, dst.1.1).into()),
                ));
            }
        }
    }
    out
}

/// Where an image draws: the node origin in physical pixels, the scale
/// from node coordinates to logical output coordinates, the output scale
/// and the opacity.
pub struct Placement {
    pub origin: Point<f64, Logical>,
    pub scale: (f64, f64),
    pub output_scale: f64,
    pub alpha: f32,
}

/// The render elements of an image node, one per cell. Each cell of each
/// node keeps its own id, so that damage tracking tells them apart.
pub fn elements<R>(
    renderer: &mut R,
    image: &Image,
    cells: &[Cell],
    ids: &mut Vec<Id>,
    at: &Placement,
) -> Vec<CellElement<R>>
where
    R: Renderer + ImportMem,
    R::TextureId: Send + Clone + 'static,
{
    ids.resize_with(cells.len(), Id::new);
    let mut out = Vec::new();
    for ((src, dst), id) in cells.iter().zip(ids.iter()) {
        let Some((location, size)) = cell_area(dst, at) else {
            continue;
        };
        let element = MemoryRenderBufferRenderElement::from_buffer(
            renderer,
            location.to_physical(1.0),
            &image.buffer,
            Some(at.alpha),
            Some(*src),
            Some(size),
            Kind::Unspecified,
        );
        match element {
            Ok(inner) => out.push(CellElement {
                id: id.clone(),
                inner,
            }),
            Err(err) => tracing::warn!(?err, "an image failed to upload"),
        }
    }
    out
}

/// The physical location and the logical size of a cell on the output, or
/// `None` for a cell without pixels. The edges are rounded, so that
/// neighbor cells leave no gap.
fn cell_area(
    dst: &Rectangle<f64, Logical>,
    at: &Placement,
) -> Option<(Point<f64, Logical>, Size<i32, Logical>)> {
    let (sx, sy) = at.scale;
    #[allow(clippy::cast_possible_truncation)]
    let round = |v: f64| v.round() as i32;
    let (x0, y0) = (round(dst.loc.x * sx), round(dst.loc.y * sy));
    let (x1, y1) = (
        round((dst.loc.x + dst.size.w) * sx),
        round((dst.loc.y + dst.size.h) * sy),
    );
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    let offset = Point::from((f64::from(x0), f64::from(y0))).upscale(at.output_scale);
    Some((at.origin + offset, Size::from((x1 - x0, y1 - y0))))
}

/// An image cell with the id of its node and cell, in place of the id of
/// the shared buffer.
pub struct CellElement<R: Renderer> {
    id: Id,
    inner: MemoryRenderBufferRenderElement<R>,
}

impl<R: Renderer> Element for CellElement<R> {
    fn id(&self) -> &Id {
        &self.id
    }

    fn current_commit(&self) -> CommitCounter {
        self.inner.current_commit()
    }

    fn location(&self, scale: Scale<f64>) -> Point<i32, Physical> {
        self.inner.location(scale)
    }

    fn src(&self) -> Rectangle<f64, BufferCoords> {
        self.inner.src()
    }

    fn transform(&self) -> Transform {
        self.inner.transform()
    }

    fn geometry(&self, scale: Scale<f64>) -> Rectangle<i32, Physical> {
        self.inner.geometry(scale)
    }

    fn damage_since(
        &self,
        scale: Scale<f64>,
        commit: Option<CommitCounter>,
    ) -> DamageSet<i32, Physical> {
        self.inner.damage_since(scale, commit)
    }

    fn opaque_regions(&self, scale: Scale<f64>) -> OpaqueRegions<i32, Physical> {
        self.inner.opaque_regions(scale)
    }

    fn alpha(&self) -> f32 {
        self.inner.alpha()
    }

    fn kind(&self) -> Kind {
        self.inner.kind()
    }
}

impl<R> RenderElement<R> for CellElement<R>
where
    R: Renderer + ImportMem,
    R::TextureId: 'static,
{
    fn draw(
        &self,
        frame: &mut R::Frame<'_, '_>,
        src: Rectangle<f64, BufferCoords>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        opaque_regions: &[Rectangle<i32, Physical>],
        cache: Option<&smithay::utils::user_data::UserDataMap>,
    ) -> Result<(), R::Error> {
        self.inner
            .draw(frame, src, dst, damage, opaque_regions, cache)
    }

    fn underlying_storage(&self, renderer: &mut R) -> Option<UnderlyingStorage<'_>> {
        self.inner.underlying_storage(renderer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_regular_file_opens() {
        let dir = std::env::temp_dir().join(format!("chora-open-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let fifo = dir.join("fifo");
        let made = std::process::Command::new("mkfifo").arg(&fifo).status();
        assert!(made.unwrap().success());
        let err = open_file(fifo.to_str().unwrap(), 1).unwrap_err();
        assert!(err.ends_with("not a regular file"), "{err}");
        assert!(open_file("/dev/zero", 1).is_err());
        let file = dir.join("big");
        std::fs::write(&file, b"ab").unwrap();
        assert!(open_file(file.to_str().unwrap(), 1).is_err());
        assert!(open_file(file.to_str().unwrap(), 2).is_ok());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_image_without_a_slice_is_one_cell() {
        let cells = cells([10, 20], [30.0, 40.0], None);
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].1.size, (30.0, 40.0).into());
    }

    #[test]
    fn a_slice_keeps_the_corners_and_stretches_the_rest() {
        let cells = cells([12, 12], [100.0, 50.0], Some([4.0, 4.0, 4.0, 4.0]));
        assert_eq!(cells.len(), 9);
        let (src, dst) = cells[4];
        assert_eq!((src.loc, src.size), ((4.0, 4.0).into(), (4.0, 4.0).into()));
        assert_eq!(
            (dst.loc, dst.size),
            ((4.0, 4.0).into(), (92.0, 42.0).into())
        );
        assert_eq!(cells[8].1.loc, (96.0, 46.0).into());
    }

    #[test]
    fn insets_that_do_not_fit_shrink() {
        let cells = cells([12, 12], [6.0, 50.0], Some([4.0, 0.0, 4.0, 0.0]));
        assert!(cells.iter().all(|(_, d)| d.loc.x + d.size.w <= 6.0));
    }

    #[test]
    fn premultiplied_alpha() {
        assert_eq!(
            rgba(png::ColorType::Rgba, &[255, 128, 0, 128]),
            [128, 64, 0, 128]
        );
        assert_eq!(rgba(png::ColorType::Grayscale, &[7]), [7, 7, 7, 255]);
    }
}
