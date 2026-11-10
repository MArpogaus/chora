//! Image nodes: PNG files that plugins name by an absolute path, for
//! example the parts of a window border.
//!
//! The core loads a file when a transaction names it, and again when the
//! file changed since. A worker thread decodes the files, so that a large
//! image does not stop the event loop; the transaction waits for them. A
//! file that fails to load rejects the transaction.
//! An image with a slice draws as up to nine cells: the corners keep their
//! size, the edges stretch in one direction and the center in both.

use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufReader,
    os::unix::fs::OpenOptionsExt,
    sync::mpsc,
    time::SystemTime,
};

use smithay::reexports::calloop::channel;

use chora_scene::{Command, NodeKind, Scene};
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            ImportMem, Renderer,
            element::{
                Element, Id, Kind, RenderElement, UnderlyingStorage,
                memory::{MemoryBuffer, MemoryRenderBuffer, MemoryRenderBufferRenderElement},
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
/// The ticket of a background reload: no transaction waits for it.
pub(crate) const RELOAD_TICKET: u64 = u64::MAX;

/// The most halvings of an image that it keeps: down to 1/16 a side.
const MAX_STEPS: usize = 4;

/// The most images in the cache at once.
const MAX_IMAGES: usize = 1024;
/// The most bytes of decoded pixels of all images.
const MAX_DECODED: u64 = 512 << 20;

/// A loaded image.
pub struct Image {
    /// The file pixels first, then each box-filtered halving.
    steps: Vec<Step>,
    /// Width and height in pixels.
    pub size: [u32; 2],
    modified: Option<SystemTime>,
    /// New with each load, so that the cells of a reloaded image get new
    /// ids and show again.
    pub version: Id,
}

/// One size of an image: the file pixels or a halving of the step before.
struct Step {
    buffer: MemoryRenderBuffer,
    size: [u32; 2],
}

/// The loaded images by path, and the worker that decodes the files.
#[derive(Default)]
pub struct Images {
    cache: HashMap<String, Image>,
    worker: Option<mpsc::Sender<Job>>,
}

/// A file for the worker to decode, for the transaction of a ticket.
struct Job {
    ticket: u64,
    path: String,
    file: File,
    modified: Option<SystemTime>,
    budget: u64,
}

/// A file that the worker decoded, or why it failed.
pub struct Decoded {
    pub ticket: u64,
    path: String,
    modified: Option<SystemTime>,
    result: Result<Vec<(MemoryBuffer, [u32; 2])>, Error>,
}

/// Why an image did not load. `limit` is true when a limit of the image
/// cache is the reason, so that the transaction answers code 6.
#[derive(Debug)]
pub struct Error {
    /// Whether a limit of the image cache is the reason.
    pub limit: bool,
    /// The message for the client.
    pub message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Error {
            limit: false,
            message,
        }
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Error {
            limit: false,
            message: message.into(),
        }
    }
}

impl Images {
    /// Starts the worker thread, which sends each file it decoded to the
    /// event loop. It ends when `Images` goes.
    pub fn start_worker(&mut self, done: channel::Sender<Decoded>) {
        let (jobs, todo) = mpsc::channel::<Job>();
        let worker = std::thread::Builder::new()
            .name("chora-images".into())
            .spawn(move || {
                // This thread starts before the loop owns SIGUSR1, so it
                // blocks the signal here: the kernel wakes one thread that
                // does not, and the default action ends the core.
                block_usr1();
                for job in todo {
                    // The copy into the buffer runs here too: for a large
                    // image it takes a while.
                    let result = decode(job.file, job.budget).map(|(pixels, size)| {
                        halvings(pixels, size)
                            .into_iter()
                            .map(|(pixels, size)| (memory(&pixels, size), size))
                            .collect()
                    });
                    let decoded = Decoded {
                        ticket: job.ticket,
                        path: job.path,
                        modified: job.modified,
                        result,
                    };
                    if done.send(decoded).is_err() {
                        break;
                    }
                }
            });
        match worker {
            Ok(_) => self.worker = Some(jobs),
            Err(err) => tracing::error!(?err, "no thread to decode images"),
        }
    }

    /// Sends the files that the commands name to the worker, unless the
    /// cache holds them unchanged. Returns how many it sent; each comes
    /// back as a `Decoded` with the ticket.
    pub fn decode_for(&mut self, ticket: u64, commands: &[Command]) -> Result<usize, Error> {
        let mut sent = HashSet::new();
        for command in commands {
            let (Command::Create { props, .. }
            | Command::Update { props, .. }
            | Command::Animate { props, .. }) = command
            else {
                continue;
            };
            let Some(path) = &props.image else {
                continue;
            };
            if sent.contains(path.as_str()) {
                continue;
            }
            let (file, meta) = open_file(path, MAX_FILE)?;
            let modified = meta.modified().ok();
            if modified.is_some() && self.cache.get(path).is_some_and(|i| i.modified == modified) {
                continue;
            }
            self.room_for(path)?;
            let worker = self.worker.as_ref().ok_or("no thread decodes images")?;
            let job = Job {
                ticket,
                path: path.clone(),
                file,
                modified,
                budget: self.budget(path),
            };
            worker
                .send(job)
                .map_err(|_| "the thread that decodes images is gone")?;
            sent.insert(path.as_str());
        }
        Ok(sent.len())
    }

    /// Puts a decoded file into the cache, if it still fits: other files
    /// may have come in since the worker started.
    pub fn insert(&mut self, decoded: Decoded) -> Result<(), Error> {
        let path = decoded.path;
        let steps = decoded.result.map_err(|mut e| {
            e.message = format!("{path}: {}", e.message);
            e
        })?;
        let size = steps[0].1;
        self.room_for(&path)?;
        let bytes = cost(size);
        if bytes > self.budget(&path) {
            return Err(Error {
                limit: true,
                message: format!(
                    "{path}: {}x{} pixels exceed the {MAX_DECODED} bytes of all images",
                    size[0], size[1]
                ),
            });
        }
        let steps = steps
            .into_iter()
            .map(|(memory, size)| Step {
                buffer: MemoryRenderBuffer::from_memory(memory, 1, Transform::Normal, None),
                size,
            })
            .collect();
        let image = Image {
            steps,
            size,
            modified: decoded.modified,
            version: Id::new(),
        };
        self.cache.insert(path, image);
        Ok(())
    }

    /// Fails if the cache is full and does not hold the path.
    fn room_for(&self, path: &str) -> Result<(), Error> {
        if !self.cache.contains_key(path) && self.cache.len() >= MAX_IMAGES {
            return Err(Error {
                limit: true,
                message: format!("{path}: the core holds {MAX_IMAGES} images"),
            });
        }
        Ok(())
    }

    /// The bytes of decoded pixels that a file may take besides the other
    /// files.
    fn budget(&self, path: &str) -> u64 {
        let others: u64 = self
            .cache
            .iter()
            .filter(|(p, _)| p.as_str() != path)
            .map(|(_, i)| cost(i.size))
            .sum();
        MAX_DECODED.saturating_sub(others)
    }

    /// Loads a file again in the background, so that the nodes that use
    /// it draw the new bytes from the next frame on. Only paths that the
    /// cache already holds are reloaded; a file that no node shows loads
    /// when a transaction names it. A file that fails to load keeps the
    /// old image.
    pub fn reload(&mut self, path: &str) {
        if !self.cache.contains_key(path) {
            return;
        }
        let Ok((file, meta)) = open_file(path, MAX_FILE) else {
            return;
        };
        let modified = meta.modified().ok();
        if modified.is_some() && self.cache.get(path).is_some_and(|i| i.modified == modified) {
            return;
        }
        let job = Job {
            ticket: RELOAD_TICKET,
            path: path.to_owned(),
            file,
            modified,
            budget: self.budget(path),
        };
        if let Some(worker) = &self.worker {
            let _ = worker.send(job);
        }
    }

    /// Drops the images that no node shows and that no waiting
    /// transaction names.
    pub fn prune(&mut self, scene: &Scene, named: &HashSet<&str>) {
        let used: HashSet<&str> = scene
            .nodes()
            .filter(|n| n.kind == NodeKind::Image)
            .filter_map(|n| n.props.image.as_deref())
            .collect();
        self.cache
            .retain(|path, _| used.contains(path.as_str()) || named.contains(path.as_str()));
    }

    /// Takes a loaded file out of the cache; for tests of nodes whose
    /// asset is gone.
    #[cfg(test)]
    pub(crate) fn forget(&mut self, path: &str) {
        self.cache.remove(path);
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
fn decode(file: File, budget: u64) -> Result<(Vec<u8>, [u32; 2]), Error> {
    let mut decoder = png::Decoder::new(BufReader::new(file));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let (w, h) = reader.info().size();
    if w == 0 || h == 0 || w > MAX_SIDE || h > MAX_SIDE {
        return Err(format!("{w}x{h} pixels; at most {MAX_SIDE} on a side").into());
    }
    if cost([w, h]) > budget {
        return Err(Error {
            limit: true,
            message: format!("{w}x{h} pixels exceed the {MAX_DECODED} bytes of all images"),
        });
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

/// The buffer of RGBA pixels; a side has at most `MAX_SIDE` pixels.
fn memory(pixels: &[u8], size: [u32; 2]) -> MemoryBuffer {
    #[allow(clippy::cast_possible_wrap)]
    let buffer_size = (size[0] as i32, size[1] as i32);
    MemoryBuffer::from_slice(pixels, Fourcc::Abgr8888, buffer_size)
}

/// The sizes of an image and of its halvings: each side halves, rounded
/// up, until the step limit or one pixel.
fn step_sizes(size: [u32; 2]) -> Vec<[u32; 2]> {
    let mut sizes = vec![size];
    while sizes.len() <= MAX_STEPS {
        let [w, h] = sizes[sizes.len() - 1];
        if w == 1 && h == 1 {
            break;
        }
        sizes.push([w.div_ceil(2), h.div_ceil(2)]);
    }
    sizes
}

/// The bytes of an image of a size with its halvings.
fn cost(size: [u32; 2]) -> u64 {
    step_sizes(size)
        .iter()
        .map(|[w, h]| u64::from(*w) * u64::from(*h) * 4)
        .sum()
}

/// The pixels of an image and of its halvings, each the box filter of the
/// one before.
fn halvings(pixels: Vec<u8>, size: [u32; 2]) -> Vec<(Vec<u8>, [u32; 2])> {
    let mut steps = vec![(pixels, size)];
    for next in step_sizes(size).into_iter().skip(1) {
        let (pixels, size) = &steps[steps.len() - 1];
        steps.push((halved(pixels, *size, next), next));
    }
    steps
}

/// The mean of each 2 by 2 block of premultiplied RGBA pixels; a block
/// at an odd edge has the pixels that exist.
fn halved(pixels: &[u8], [w, h]: [u32; 2], [nw, nh]: [u32; 2]) -> Vec<u8> {
    let row = |y: u32| &pixels[y as usize * w as usize * 4..(y as usize + 1) * w as usize * 4];
    let mut out = Vec::with_capacity(nw as usize * nh as usize * 4);
    for y in 0..nh {
        let rows: Vec<&[u8]> = (y * 2..(y * 2 + 2).min(h)).map(row).collect();
        for x in 0..nw {
            let (from, to) = (x as usize * 8, ((x as usize * 2 + 2).min(w as usize)) * 4);
            let mut sum = [0u32; 4];
            let mut count = 0;
            for pixel in rows.iter().flat_map(|r| r[from..to].chunks(4)) {
                for (total, value) in sum.iter_mut().zip(pixel) {
                    *total += u32::from(*value);
                }
                count += 1;
            }
            out.extend(sum.map(|total| u8::try_from((total + count / 2) / count).unwrap_or(255)));
        }
    }
    out
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

/// The left, top, right and bottom of a destination in node coordinates.
/// Neighbor cells copy the shared edge from one array, so that their edges
/// are equal bit for bit.
pub type Edges = [f64; 4];

/// A cell of an image: the source in pixels of the file and the
/// destination in node coordinates.
pub type Cell = (Rectangle<f64, Logical>, Edges);

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
                    [dx[col], dy[row], dx[col + 1], dy[row + 1]],
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
        let Some(area) = cell_area(dst, at) else {
            continue;
        };
        let index = step_index(src, image.size, area.size, image.steps.len());
        let step = &image.steps[index];
        let factor = |side: usize| f64::from(step.size[side]) / f64::from(image.size[side]);
        let scaled = Rectangle::<f64, Logical>::new(
            (src.loc.x * factor(0), src.loc.y * factor(1)).into(),
            (src.size.w * factor(0), src.size.h * factor(1)).into(),
        );
        #[allow(clippy::cast_possible_truncation)]
        let logical = |side: i32| ((f64::from(side) / at.output_scale).round() as i32).max(1);
        let element = MemoryRenderBufferRenderElement::from_buffer(
            renderer,
            area.loc.to_f64(),
            &step.buffer,
            Some(at.alpha),
            Some(scaled),
            Some(Size::from((logical(area.size.w), logical(area.size.h)))),
            Kind::Unspecified,
        );
        match element {
            Ok(inner) => out.push(CellElement {
                id: id.clone(),
                inner,
                area,
                commit: CommitCounter::from(index),
            }),
            Err(err) => tracing::warn!(?err, "an image failed to upload"),
        }
    }
    out
}

/// The area of a cell on the output in whole physical pixels, or `None`
/// for a cell without pixels. Each edge rounds to the nearest physical
/// pixel on its own, so that neighbor cells leave no gap, and a node at
/// x = 0.4 with a width of 10.4 covers the pixels 0 to 10 and not 0.4 to
/// 10.8. An image sized to whole physical pixels is not resampled.
fn cell_area(dst: &Edges, at: &Placement) -> Option<Rectangle<i32, Physical>> {
    let (sx, sy) = at.scale;
    let scale = at.output_scale;
    #[allow(clippy::cast_possible_truncation)]
    let round = |v: f64| v.round() as i32;
    let x = |edge: f64| round(at.origin.x + edge * sx * scale);
    let y = |edge: f64| round(at.origin.y + edge * sy * scale);
    let [x0, y0, x1, y1] = [x(dst[0]), y(dst[1]), x(dst[2]), y(dst[3])];
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    Some(Rectangle::new((x0, y0).into(), (x1 - x0, y1 - y0).into()))
}

/// The halvings that a cell may use: the most that still has one pixel
/// of the file for each pixel on the output. A cell takes the most
/// reduced axis, so that a cell that is much smaller on one axis shows no
/// moire there and is softer on the other. A halving never crosses a
/// cell edge: each edge of the source is a multiple of the step, or the
/// end of the file, so that a pixel of the step holds the color of one
/// cell only.
fn step_index(
    src: &Rectangle<f64, Logical>,
    file: [u32; 2],
    area: Size<i32, Physical>,
    steps: usize,
) -> usize {
    let ratio = (f64::from(area.w) / src.size.w).min(f64::from(area.h) / src.size.h);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let wanted = if ratio < 1.0 {
        (-ratio.log2()).floor() as usize
    } else {
        0
    };
    let edges = [
        (src.loc.x, file[0]),
        (src.loc.x + src.size.w, file[0]),
        (src.loc.y, file[1]),
        (src.loc.y + src.size.h, file[1]),
    ];
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let aligned = |k: usize| {
        edges.iter().all(|(edge, end)| {
            let at_end = (edge - f64::from(*end)).abs() < f64::EPSILON;
            let on_step = edge.fract() == 0.0 && (*edge as u64).is_multiple_of(1 << k);
            at_end || on_step
        })
    };
    let limit = wanted.min(steps - 1);
    (0..=limit).rev().find(|k| aligned(*k)).unwrap_or(0)
}

/// The damage of a cell: all of its area when the commit differs from the
/// one asked for, which happens for a new image and for a switch of the
/// step.
fn cell_damage(
    asked: Option<CommitCounter>,
    commit: CommitCounter,
    area: Rectangle<i32, Physical>,
) -> DamageSet<i32, Physical> {
    if asked == Some(commit) {
        return DamageSet::default();
    }
    DamageSet::from_slice(&[Rectangle::from_size(area.size)])
}

/// An image cell with the id of its node and cell, in place of the id of
/// the shared buffer.
pub struct CellElement<R: Renderer> {
    id: Id,
    inner: MemoryRenderBufferRenderElement<R>,
    area: Rectangle<i32, Physical>,
    commit: CommitCounter,
}

impl<R: Renderer> Element for CellElement<R> {
    fn id(&self) -> &Id {
        &self.id
    }

    fn current_commit(&self) -> CommitCounter {
        self.commit
    }

    fn location(&self, _scale: Scale<f64>) -> Point<i32, Physical> {
        self.area.loc
    }

    fn src(&self) -> Rectangle<f64, BufferCoords> {
        self.inner.src()
    }

    fn transform(&self) -> Transform {
        self.inner.transform()
    }

    fn geometry(&self, _scale: Scale<f64>) -> Rectangle<i32, Physical> {
        self.area
    }

    fn damage_since(
        &self,
        _scale: Scale<f64>,
        commit: Option<CommitCounter>,
    ) -> DamageSet<i32, Physical> {
        cell_damage(commit, self.commit, self.area)
    }

    fn opaque_regions(&self, _scale: Scale<f64>) -> OpaqueRegions<i32, Physical> {
        OpaqueRegions::default()
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

/// Blocks SIGUSR1 for the calling thread, so that the emergency reset
/// reaches the signal source of the loop and not this thread.
#[allow(unsafe_code)] // a pthread call, nothing else
fn block_usr1() {
    let rc = unsafe {
        let mut set: libc::sigset_t = std::mem::zeroed();
        libc::sigemptyset(std::ptr::addr_of_mut!(set));
        libc::sigaddset(std::ptr::addr_of_mut!(set), libc::SIGUSR1);
        libc::pthread_sigmask(
            libc::SIG_BLOCK,
            std::ptr::addr_of!(set),
            std::ptr::null_mut(),
        )
    };
    assert_eq!(rc, 0, "the thread blocks SIGUSR1");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_regular_file_opens() {
        let dir = crate::testing::scratch_dir("chora-open");
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

    /// A file over the decode budget is a limit, so that the transaction
    /// answers code 6 and not code 1.
    #[test]
    fn pixels_over_the_budget_are_a_limit() {
        let dir = crate::testing::scratch_dir("chora-budget");
        let path = dir.join("img.png");
        let mut png = Vec::new();
        {
            let mut enc = png::Encoder::new(std::io::Cursor::new(&mut png), 4, 4);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header()
                .unwrap()
                .write_image_data(&[0u8; 64])
                .unwrap();
        }
        std::fs::write(&path, png).unwrap();
        let err = decode(File::open(&path).unwrap(), 3).unwrap_err();
        assert!(err.limit, "{err}");
        assert!(decode(File::open(&path).unwrap(), MAX_DECODED).is_ok());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_image_without_a_slice_is_one_cell() {
        let cells = cells([10, 20], [30.0, 40.0], None);
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].1, [0.0, 0.0, 30.0, 40.0]);
    }

    #[test]
    fn a_slice_keeps_the_corners_and_stretches_the_rest() {
        let cells = cells([12, 12], [100.0, 50.0], Some([4.0, 4.0, 4.0, 4.0]));
        assert_eq!(cells.len(), 9);
        let (src, dst) = cells[4];
        assert_eq!((src.loc, src.size), ((4.0, 4.0).into(), (4.0, 4.0).into()));
        assert_eq!(dst, [4.0, 4.0, 96.0, 46.0]);
        assert_eq!(cells[8].1[..2], [96.0, 46.0]);
    }

    #[test]
    fn insets_that_do_not_fit_shrink() {
        let cells = cells([12, 12], [6.0, 50.0], Some([4.0, 0.0, 4.0, 0.0]));
        assert!(cells.iter().all(|(_, d)| d[2] <= 6.0));
    }

    #[test]
    fn halvings_average_blocks_and_end_at_the_step_limit() {
        let pixels = [
            0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255,
        ];
        let steps = halvings(pixels.to_vec(), [2, 2]);
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[1], (vec![128, 128, 128, 255], [1, 1]));
        assert_eq!(
            halvings(vec![0; 64 * 4 * 64], [64, 64]).len(),
            MAX_STEPS + 1
        );
    }

    #[test]
    fn a_thin_image_costs_the_sum_of_its_steps() {
        let sizes = step_sizes([1, 1000]);
        assert_eq!(sizes, [[1, 1000], [1, 500], [1, 250], [1, 125], [1, 63]]);
        assert_eq!(cost([1, 1000]), (1000 + 500 + 250 + 125 + 63) * 4);
        assert_eq!(cost([1, 1]), 4);
    }

    fn area(w: i32, h: i32) -> Size<i32, Physical> {
        (w, h).into()
    }

    fn whole(file: [f64; 2]) -> Rectangle<f64, Logical> {
        Rectangle::from_size((file[0], file[1]).into())
    }

    #[test]
    fn the_step_follows_the_ratio_down_to_one_sixteenth() {
        let src = whole([256.0, 256.0]);
        let steps = MAX_STEPS + 1;
        let at = |side: i32| step_index(&src, [256, 256], area(side, side), steps);
        assert_eq!(at(300), 0, "above 1");
        assert_eq!(at(256), 0, "1");
        assert_eq!(at(129), 0, "just above 1/2");
        assert_eq!(at(128), 1, "1/2");
        assert_eq!(at(16), 4, "1/16");
        assert_eq!(at(4), 4, "below 1/16");
        assert_eq!(step_index(&src, [256, 256], area(4, 4), 2), 1, "few steps");
    }

    #[test]
    fn an_odd_image_steps_down_to_one_pixel() {
        let src = whole([5.0, 3.0]);
        assert_eq!(step_sizes([5, 3]), [[5, 3], [3, 2], [2, 1], [1, 1]]);
        assert_eq!(step_index(&src, [5, 3], area(1, 1), 4), 2);
    }

    #[test]
    fn a_cell_takes_the_step_of_its_most_reduced_axis() {
        let src = whole([100.0, 100.0]);
        assert_eq!(step_index(&src, [100, 100], area(50, 10), 5), 3);
        assert_eq!(step_index(&src, [100, 100], area(10, 50), 5), 3);
    }

    #[test]
    fn a_step_never_crosses_the_edge_of_a_cell() {
        let center = Rectangle::new((6.0, 6.0).into(), (244.0, 244.0).into());
        assert_eq!(step_index(&center, [256, 256], area(16, 16), 5), 1);
        let aligned = Rectangle::new((8.0, 8.0).into(), (240.0, 240.0).into());
        assert_eq!(step_index(&aligned, [256, 256], area(16, 16), 5), 3);
        let fractional = Rectangle::new((4.5, 0.0).into(), (251.5, 256.0).into());
        assert_eq!(step_index(&fractional, [256, 256], area(16, 16), 5), 0);
    }

    #[test]
    fn the_edges_of_a_cell_round_in_physical_pixels() {
        let at = Placement {
            origin: (0.4, 0.0).into(),
            scale: (1.0, 1.0),
            output_scale: 1.0,
            alpha: 1.0,
        };
        let area = cell_area(&[0.0, 0.0, 10.4, 5.0], &at).unwrap();
        assert_eq!((area.loc.x, area.size.w), (0, 11));
        let next = cell_area(&[10.4, 0.0, 20.0, 5.0], &at).unwrap();
        assert_eq!(next.loc.x, area.loc.x + area.size.w, "no gap");
    }

    #[test]
    fn a_cell_has_damage_only_when_its_commit_changed() {
        let area = Rectangle::from_size(area(10, 6));
        let same = CommitCounter::from(2);
        assert!(cell_damage(Some(same), same, area).is_empty());
        let moved = cell_damage(Some(CommitCounter::from(1)), same, area);
        assert_eq!(moved.to_vec(), [Rectangle::from_size((10, 6).into())]);
        assert!(!cell_damage(None, same, area).is_empty());
    }

    #[test]
    fn decoded_pixels_are_premultiplied_rgba() {
        assert_eq!(
            rgba(png::ColorType::Rgba, &[255, 128, 0, 128]),
            [128, 64, 0, 128]
        );
        assert_eq!(rgba(png::ColorType::Grayscale, &[7]), [7, 7, 7, 255]);
    }

    /// The worker of `Images` blocks the emergency signal on its own,
    /// because it starts before the loop owns SIGUSR1.
    #[test]
    fn the_worker_thread_blocks_the_emergency_signal() {
        let (done, _watch) = channel::channel();
        let mut images = Images::default();
        images.start_worker(done);
        let blocked = || {
            std::fs::read_dir("/proc/self/task")
                .unwrap()
                .flatten()
                .any(|task| {
                    let dir = task.path();
                    let comm = std::fs::read_to_string(dir.join("comm")).unwrap_or_default();
                    if !comm.starts_with("chora-images") {
                        return false;
                    }
                    let status = std::fs::read_to_string(dir.join("status")).unwrap();
                    let line = status.lines().find(|l| l.starts_with("SigBlk")).unwrap();
                    let mask =
                        u64::from_str_radix(line.split(':').nth(1).unwrap().trim(), 16).unwrap();
                    mask & (1 << 9) != 0
                })
        };
        let end = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !blocked() {
            assert!(std::time::Instant::now() < end, "the worker blocks SIGUSR1");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}
