//! The XP theme: the SVG files of the theme directory, rasterized with
//! resvg into PNG files for image nodes.
//!
//! A [`Picture`] describes an image as layers: elements of the SVG files
//! by id, frames by prefix, icons, fills and text. Each picture renders
//! once per scale; the name of its file is the hash of the description
//! and the scale. Its size is in logical pixels, and its file has the
//! pixels of the largest scale of the outputs, so that text and icons
//! stay sharp on a scaled output.

use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
};

use resvg::{
    tiny_skia::{
        self, FilterQuality, IntSize, Paint, Pattern, Pixmap, PixmapMut, PixmapPaint,
        PremultipliedColorU8, SpreadMode, Transform,
    },
    usvg,
};
use zune_jpeg::{
    JpegDecoder,
    zune_core::{colorspace::ColorSpace, options::DecoderOptions},
};

use chora_scene::Command;

use crate::text::{Font, Fonts, Text};

/// A rectangle: x, y, width and height in pixels.
pub type Rect = [i32; 4];

/// The SVG files of the theme.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Sheet {
    /// The window frame of Exposé.
    Decoration,
    /// The close button.
    Close,
    /// The maximize button.
    Maximize,
    /// The minimize button.
    Minimize,
    /// The restore button of a maximized window.
    Restore,
    /// The panel of Exposé, the background of the taskbar.
    Panel,
    /// The XP artwork: start button, start menu, notification area.
    Winxp,
    /// The task buttons.
    Tasks,
}

impl Sheet {
    const ALL: [Self; 8] = [
        Self::Decoration,
        Self::Close,
        Self::Maximize,
        Self::Minimize,
        Self::Restore,
        Self::Panel,
        Self::Winxp,
        Self::Tasks,
    ];

    /// The file in the theme directory.
    fn file(self) -> &'static str {
        match self {
            Self::Decoration => "expose/decoration.svg",
            Self::Close => "expose/close.svg",
            Self::Maximize => "expose/maximize.svg",
            Self::Minimize => "expose/minimize.svg",
            Self::Restore => "expose/restore.svg",
            Self::Panel => "expose/panel-background.svg",
            Self::Winxp => "winxp/winxp.svg",
            Self::Tasks => "winxp/tasks.svg",
        }
    }
}

/// One layer of a picture.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Layer {
    /// An element of a sheet, by id, scaled to a rectangle.
    Element {
        /// The sheet.
        sheet: Sheet,
        /// The id of the element.
        id: String,
        /// The rectangle.
        rect: Rect,
    },
    /// A frame of a sheet, by the prefix of its nine elements, stretched
    /// over a rectangle: the corners keep their size.
    Frame {
        /// The sheet.
        sheet: Sheet,
        /// For example `normal` for `normal-topleft` and the others.
        prefix: String,
        /// The rectangle.
        rect: Rect,
    },
    /// An SVG, PNG or JPEG file, scaled to a rectangle.
    File {
        /// The file.
        path: PathBuf,
        /// The rectangle.
        rect: Rect,
        /// How the file fills the rectangle.
        fit: Fit,
    },
    /// A file as [`Layer::File`], in light gray, as XP shows the icon of
    /// a disabled button.
    Gray {
        /// The file.
        path: PathBuf,
        /// The rectangle.
        rect: Rect,
    },
    /// A line of text.
    Text(Text),
    /// SVG elements in the coordinates of the picture, such as the
    /// rectangles and gradients of a control.
    Svg(String),
}

/// An image as a stack of layers, bottom first.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Picture {
    /// Width and height in pixels.
    pub size: [i32; 2],
    /// The layers.
    pub layers: Vec<Layer>,
}

impl Picture {
    /// An empty picture.
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            size: [width.max(1), height.max(1)],
            layers: Vec::new(),
        }
    }

    /// Adds an element of a sheet.
    pub fn element(mut self, sheet: Sheet, id: &str, rect: Rect) -> Self {
        self.layers.push(Layer::Element {
            sheet,
            id: id.into(),
            rect,
        });
        self
    }

    /// Adds a frame of a sheet.
    pub fn frame(mut self, sheet: Sheet, prefix: &str, rect: Rect) -> Self {
        self.layers.push(Layer::Frame {
            sheet,
            prefix: prefix.into(),
            rect,
        });
        self
    }

    /// Adds an SVG or PNG file.
    pub fn file(mut self, path: &Path, rect: Rect) -> Self {
        self.layers.push(Layer::File {
            path: path.into(),
            rect,
            fit: Fit::Inside,
        });
        self
    }

    /// Adds a file that fills a rectangle in a way of [`Fit`].
    pub fn fit(mut self, path: &Path, rect: Rect, fit: Fit) -> Self {
        self.layers.push(Layer::File {
            path: path.into(),
            rect,
            fit,
        });
        self
    }

    /// Adds an SVG or PNG file in light gray.
    pub fn gray(mut self, path: &Path, rect: Rect) -> Self {
        self.layers.push(Layer::Gray {
            path: path.into(),
            rect,
        });
        self
    }

    /// Adds a line of text.
    pub fn text(mut self, text: Text) -> Self {
        self.layers.push(Layer::Text(text));
        self
    }

    /// Adds the layers of another picture, moved by an offset.
    pub fn layers_of(mut self, other: Self, [dx, dy]: [i32; 2]) -> Self {
        let shift = |[x, y, w, h]: Rect| [x + dx, y + dy, w, h];
        for layer in other.layers {
            self.layers.push(match layer {
                Layer::Element { sheet, id, rect } => Layer::Element {
                    sheet,
                    id,
                    rect: shift(rect),
                },
                Layer::Frame {
                    sheet,
                    prefix,
                    rect,
                } => Layer::Frame {
                    sheet,
                    prefix,
                    rect: shift(rect),
                },
                Layer::File { path, rect, fit } => Layer::File {
                    path,
                    rect: shift(rect),
                    fit,
                },
                Layer::Gray { path, rect } => Layer::Gray {
                    path,
                    rect: shift(rect),
                },
                Layer::Text(mut text) => {
                    text.rect = shift(text.rect);
                    Layer::Text(text)
                }
                Layer::Svg(body) => {
                    Layer::Svg(format!(r#"<g transform="translate({dx} {dy})">{body}</g>"#))
                }
            });
        }
        self
    }

    /// Adds SVG elements, drawn in the coordinates of the picture.
    pub fn svg(mut self, body: impl Into<String>) -> Self {
        self.layers.push(Layer::Svg(body.into()));
        self
    }
}

/// A nine-slice image for an image node.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    /// The PNG file.
    pub image: String,
    /// The insets left, top, right and bottom.
    pub slice: [f64; 4],
}

/// A rendered file and the time of its last use.
struct Rendered {
    path: String,
    used: u64,
}

/// The theme and the cache of rendered files.
pub struct Theme {
    dir: PathBuf,
    cache: PathBuf,
    fonts: Fonts,
    sheets: HashMap<Sheet, usvg::Tree>,
    rendered: HashMap<u64, Rendered>,
    clock: u64,
    /// A transparent pixel, for pictures that fail to render.
    blank: String,
    /// The widths of lines of text, as their layout is slow.
    widths: RefCell<HashMap<(String, Font), f64>>,
    /// Pixels per logical pixel of the rendered files.
    scale: f64,
    /// The pictures of the rendered files at the scale, by path, to give
    /// image nodes their logical size and to render them again at
    /// another scale.
    pictures: HashMap<String, Picture>,
}

/// How many rendered files the cache keeps.
const CACHE_SIZE: usize = 512;
/// How many pictures of rendered files the theme remembers.
const PICTURES: usize = 8192;
/// The largest scale of the pictures.
const MAX_SCALE: f64 = 4.0;

impl Theme {
    /// Loads the theme from a directory. Rendered files go to `cache`,
    /// which this call empties first.
    pub fn load(dir: &Path, cache: &Path) -> Result<Self, String> {
        let mut sheets = HashMap::new();
        let fonts = Fonts::load(dir);
        for sheet in Sheet::ALL {
            let path = dir.join(sheet.file());
            let data = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let tree = usvg::Tree::from_data(&data, fonts.options())
                .map_err(|e| format!("{}: {e}", path.display()))?;
            sheets.insert(sheet, tree);
        }
        clear(cache).map_err(|e| format!("{}: {e}", cache.display()))?;
        let mut theme = Self {
            dir: dir.into(),
            cache: cache.into(),
            fonts,
            sheets,
            rendered: HashMap::new(),
            clock: 0,
            blank: String::new(),
            widths: RefCell::default(),
            scale: 1.0,
            pictures: HashMap::new(),
        };
        theme.blank = theme
            .write(0, &Pixmap::new(1, 1).expect("one pixel"))
            .map_err(|e| format!("{}: {e}", cache.display()))?;
        Ok(theme)
    }

    /// A file of the theme directory.
    pub fn path(&self, file: &str) -> PathBuf {
        self.dir.join(file)
    }

    /// Writes a file into the cache, for example a shader for the core,
    /// and returns its path.
    pub fn cache_file(&self, name: &str, data: &str) -> std::io::Result<PathBuf> {
        let path = self.cache.join(name);
        fs::write(&path, data)?;
        Ok(path)
    }

    /// The width of a line of text in pixels.
    pub fn text_width(&self, line: &str, font: Font) -> f64 {
        let key = (line.to_owned(), font);
        if let Some(width) = self.widths.borrow().get(&key) {
            return *width;
        }
        let width = self.fonts.width(line, font).unwrap_or(0.0);
        let mut widths = self.widths.borrow_mut();
        // ponytail: forget all at a size, rather than track their use.
        if widths.len() >= 4096 {
            widths.clear();
        }
        widths.insert(key, width);
        width
    }

    /// The size of an element, rounded to pixels.
    pub fn size(&self, sheet: Sheet, id: &str) -> Option<[i32; 2]> {
        let node = self.sheets[&sheet].node_by_id(id)?;
        let bbox = node.abs_bounding_box();
        #[allow(clippy::cast_possible_truncation)]
        let size = [bbox.width().round() as i32, bbox.height().round() as i32];
        (size[0] > 0 && size[1] > 0).then_some(size)
    }

    /// The nine elements of a frame at their own size, as one file with the
    /// insets for the `slice` of an image node.
    pub fn frame(&mut self, sheet: Sheet, prefix: &str) -> Frame {
        let [l, t, r, b] = self.borders(sheet, prefix);
        let center = self.size(sheet, &part(prefix, "center")).unwrap_or([1, 1]);
        let size = [l + center[0] + r, t + center[1] + b];
        let picture = Picture::new(size[0], size[1]).frame(sheet, prefix, [0, 0, size[0], size[1]]);
        Frame {
            image: self.render_flat(&picture),
            slice: [l, t, r, b].map(f64::from),
        }
    }

    /// The width of the left and right edges and the height of the top and
    /// bottom edges of a frame.
    fn borders(&self, sheet: Sheet, prefix: &str) -> [i32; 4] {
        let size = |name: &str| self.size(sheet, &part(prefix, name)).unwrap_or([0, 0]);
        [
            size("left")[0].max(size("topleft")[0]),
            size("top")[1].max(size("topleft")[1]),
            size("right")[0].max(size("bottomright")[0]),
            size("bottom")[1].max(size("bottomright")[1]),
        ]
    }

    /// The PNG file of a picture at the scale, rendered on first use. A
    /// picture that fails to render logs and gets a transparent pixel.
    pub fn render(&mut self, picture: &Picture) -> String {
        let path = self.render_at(picture, self.scale, false);
        if path != self.blank && !self.pictures.contains_key(&path) {
            // ponytail: forget all at a size, rather than track which
            // files nodes still show; those keep their scale.
            if self.pictures.len() >= PICTURES {
                self.pictures.clear();
            }
            self.pictures.insert(path.clone(), picture.clone());
        }
        path
    }

    /// The PNG file of a picture at its own size, for an image node with
    /// a slice: the core takes the insets of a slice in pixels of the
    /// file, also for the corners on the screen.
    pub fn render_flat(&mut self, picture: &Picture) -> String {
        self.render_at(picture, 1.0, true)
    }

    /// The file of a picture at a scale; a flat file never shares the
    /// path of a sharp one, so that a rescale never touches it.
    fn render_at(&mut self, picture: &Picture, scale: f64, flat: bool) -> String {
        let mut hasher = DefaultHasher::new();
        picture.hash(&mut hasher);
        scale.to_bits().hash(&mut hasher);
        flat.hash(&mut hasher);
        let key = hasher.finish() | 1;
        self.clock += 1;
        if let Some(entry) = self.rendered.get_mut(&key) {
            entry.used = self.clock;
            return entry.path.clone();
        }
        let written = self.draw(picture, scale).and_then(|p| self.write(key, &p));
        match written {
            Ok(path) => {
                self.evict();
                self.rendered.insert(
                    key,
                    Rendered {
                        path: path.clone(),
                        used: self.clock,
                    },
                );
                path
            }
            Err(err) => {
                eprintln!("chora-xp: rendering failed: {err}");
                self.blank.clone()
            }
        }
    }

    /// Takes the largest scale of the outputs, at least 1, but at most 4.
    /// Returns whether it changed.
    pub fn set_scale(&mut self, scale: f64) -> bool {
        let scale = scale.min(MAX_SCALE);
        if (scale - self.scale).abs() < 1e-9 {
            return false;
        }
        self.scale = scale;
        true
    }

    /// A rendered file at the current scale with its logical size, for a
    /// file of an earlier scale; `None` for a file of no picture.
    pub fn rerender(&mut self, path: &str) -> Option<(String, [f64; 2])> {
        let picture = self.pictures.get(path)?.clone();
        Some((self.render(&picture), picture.size.map(f64::from)))
    }

    /// Gives the image nodes of rendered pictures their logical size,
    /// unless a command sets a size or a slice itself.
    pub fn size_images(&self, commands: &mut [Command]) {
        for command in commands {
            let (Command::Create { props, .. } | Command::Update { props, .. }) = command else {
                continue;
            };
            if props.size.is_some() || props.slice.is_some() {
                continue;
            }
            if let Some(picture) = props.image.as_ref().and_then(|p| self.pictures.get(p)) {
                props.size = Some(picture.size.map(f64::from));
            }
        }
    }

    /// Deletes the older half of the files when the cache is full. A node
    /// may still show a deleted file: the core keeps the texture, and the
    /// next use of the picture renders it again.
    fn evict(&mut self) {
        if self.rendered.len() < CACHE_SIZE {
            return;
        }
        let mut uses: Vec<u64> = self.rendered.values().map(|r| r.used).collect();
        uses.sort_unstable();
        let cut = uses[uses.len() / 2];
        self.rendered.retain(|_, r| {
            let keep = r.used > cut;
            if !keep {
                let _ = fs::remove_file(&r.path);
            }
            keep
        });
    }

    fn write(&self, key: u64, pixmap: &Pixmap) -> Result<String, String> {
        let path = self.cache.join(format!("{key:016x}.png"));
        let partial = path.with_extension("part");
        let data = pixmap.encode_png().map_err(|e| e.to_string())?;
        fs::write(&partial, data).map_err(|e| e.to_string())?;
        fs::rename(&partial, &path).map_err(|e| e.to_string())?;
        path.into_os_string()
            .into_string()
            .map_err(|p| format!("not UTF-8: {}", p.display()))
    }

    fn draw(&self, picture: &Picture, scale: f64) -> Result<Pixmap, String> {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let [w, h] = picture
            .size
            .map(|n| (f64::from(n) * scale).ceil().max(1.0) as u32);
        let mut pixmap = Pixmap::new(w, h).ok_or("an empty picture")?;
        #[allow(clippy::cast_possible_truncation)]
        let base = Transform::from_scale(scale as f32, scale as f32);
        for layer in &picture.layers {
            // A missing icon or element leaves the rest of the picture.
            if let Err(err) = self.draw_layer(layer, (base, &mut pixmap.as_mut())) {
                eprintln!("chora-xp: {err}");
            }
        }
        Ok(pixmap)
    }

    /// Draws a layer in the coordinates of the picture, which `base`
    /// scales to pixels.
    fn draw_layer(
        &self,
        layer: &Layer,
        (base, pixmap): (Transform, &mut PixmapMut<'_>),
    ) -> Result<(), String> {
        match layer {
            Layer::Element { sheet, id, rect } => {
                self.draw_element(*sheet, id, *rect, (base, pixmap))
            }
            Layer::Frame {
                sheet,
                prefix,
                rect,
            } => {
                self.draw_frame(*sheet, prefix, *rect, (base, pixmap));
                Ok(())
            }
            Layer::File { path, rect, fit } => self.draw_file(path, *rect, *fit, (base, pixmap)),
            Layer::Gray { path, rect } => self.draw_gray(path, *rect, (base, pixmap)),
            Layer::Text(text) => self.fonts.draw(text, base, pixmap),
            Layer::Svg(body) => {
                let svg = format!(
                    r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">{body}</svg>"#,
                    pixmap.width(),
                    pixmap.height()
                );
                let tree = usvg::Tree::from_str(&svg, self.fonts.options())
                    .map_err(|e| format!("svg layer: {e}"))?;
                resvg::render(&tree, base, pixmap);
                Ok(())
            }
        }
    }

    /// Draws an element so that its bounding box fills the rectangle.
    fn draw_element(
        &self,
        sheet: Sheet,
        id: &str,
        rect: Rect,
        (base, pixmap): (Transform, &mut PixmapMut<'_>),
    ) -> Result<(), String> {
        let missing = || format!("{} has no element {id:?}", sheet.file());
        let node = self.sheets[&sheet].node_by_id(id).ok_or_else(missing)?;
        let bbox = node
            .abs_bounding_box()
            .to_non_zero_rect()
            .ok_or_else(missing)?;
        let Some(layer) = node.abs_layer_bounding_box() else {
            return Ok(());
        };
        let [x, y, w, h] = rect.map(f64::from);
        #[allow(clippy::cast_possible_truncation)]
        let transform = base
            .pre_translate(x as f32, y as f32)
            .pre_scale(w as f32 / bbox.width(), h as f32 / bbox.height())
            .pre_translate(-bbox.x(), -bbox.y())
            .pre_concat(parent_transform(node))
            .pre_translate(layer.x(), layer.y());
        resvg::render_node(node, transform, pixmap);
        Ok(())
    }

    /// Draws the nine parts of a frame over a rectangle; missing parts
    /// stay empty.
    fn draw_frame(
        &self,
        sheet: Sheet,
        prefix: &str,
        rect: Rect,
        (base, pixmap): (Transform, &mut PixmapMut<'_>),
    ) {
        let [left, top, right, bottom] = self.borders(sheet, prefix);
        let [x, y, width, height] = rect;
        let middle = (width - left - right).max(0);
        let center = (height - top - bottom).max(0);
        let (x1, x2) = (x + left, x + width - right);
        let (y1, y2) = (y + top, y + height - bottom);
        let cells = [
            ("topleft", [x, y, left, top]),
            ("top", [x1, y, middle, top]),
            ("topright", [x2, y, right, top]),
            ("left", [x, y1, left, center]),
            ("center", [x1, y1, middle, center]),
            ("right", [x2, y1, right, center]),
            ("bottomleft", [x, y2, left, bottom]),
            ("bottom", [x1, y2, middle, bottom]),
            ("bottomright", [x2, y2, right, bottom]),
        ];
        for (name, cell) in cells {
            if cell[2] > 0 && cell[3] > 0 {
                // A frame without some of its parts is fine.
                let _ = self.draw_element(sheet, &part(prefix, name), cell, (base, pixmap));
            }
        }
    }

    /// Draws an SVG, PNG or JPEG file scaled to a rectangle.
    /// A file in light gray: drawn alone, then each pixel turns to its
    /// luma, halfway to white.
    fn draw_gray(
        &self,
        path: &Path,
        rect: Rect,
        (base, pixmap): (Transform, &mut PixmapMut<'_>),
    ) -> Result<(), String> {
        let mut temp = Pixmap::new(pixmap.width(), pixmap.height()).ok_or("no pixels")?;
        self.draw_file(path, rect, Fit::Inside, (base, &mut temp.as_mut()))?;
        for pixel in temp.pixels_mut() {
            let [red, green, blue] = [pixel.red(), pixel.green(), pixel.blue()].map(u32::from);
            let alpha = pixel.alpha();
            // Premultiplied, white is the alpha.
            let luma = (red * 30 + green * 59 + blue * 11) / 100;
            let light = u8::try_from(u32::midpoint(luma, u32::from(alpha))).unwrap_or(u8::MAX);
            if let Some(gray) = PremultipliedColorU8::from_rgba(light, light, light, alpha) {
                *pixel = gray;
            }
        }
        let paint = PixmapPaint::default();
        pixmap.draw_pixmap(0, 0, temp.as_ref(), &paint, Transform::identity(), None);
        Ok(())
    }

    fn draw_file(
        &self,
        path: &Path,
        rect: Rect,
        fit: Fit,
        (base, pixmap): (Transform, &mut PixmapMut<'_>),
    ) -> Result<(), String> {
        let error = |e: &dyn std::fmt::Display| format!("{}: {e}", path.display());
        let image = if is_svg(path) {
            let data = fs::read(path).map_err(|e| error(&e))?;
            let tree = usvg::Tree::from_data(&data, self.fonts.options()).map_err(|e| error(&e))?;
            let size = tree.size();
            if !matches!(fit, Fit::Tile(_)) {
                resvg::render(
                    &tree,
                    base.pre_concat(place(rect, [size.width(), size.height()], fit)),
                    pixmap,
                );
                return Ok(());
            }
            // A tile of an SVG file: its pixels at its own size.
            let [w, h] = [size.width(), size.height()].map(f32::ceil);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let mut tile = Pixmap::new(w as u32, h as u32).ok_or_else(|| error(&"empty"))?;
            resvg::render(&tree, Transform::identity(), &mut tile.as_mut());
            tile
        } else {
            load_raster(path).map_err(|e| error(&e))?
        };
        #[allow(clippy::cast_precision_loss)]
        let size = [image.width() as f32, image.height() as f32];
        let transform = place(rect, size, fit);
        if let Fit::Tile(_) = fit {
            // fill_rect applies its transform to the pattern too.
            let paint = Paint {
                shader: Pattern::new(
                    image.as_ref(),
                    SpreadMode::Repeat,
                    FilterQuality::Bicubic,
                    1.0,
                    transform,
                ),
                ..Paint::default()
            };
            #[allow(clippy::cast_precision_loss)]
            let [x, y, w, h] = rect.map(|n| n as f32);
            if let Some(area) = tiny_skia::Rect::from_xywh(x, y, w, h) {
                pixmap.fill_rect(area, &paint, base, None);
            }
            return Ok(());
        }
        let paint = PixmapPaint {
            quality: FilterQuality::Bicubic,
            ..PixmapPaint::default()
        };
        pixmap.draw_pixmap(
            0,
            0,
            image.as_ref(),
            &paint,
            base.pre_concat(transform),
            None,
        );
        Ok(())
    }

    /// The width and height of an SVG, PNG or JPEG file in pixels, from
    /// its header.
    pub fn image_size(&self, path: &Path) -> Option<[i32; 2]> {
        #[allow(clippy::cast_possible_truncation)]
        let size = if is_svg(path) {
            let tree = usvg::Tree::from_data(&fs::read(path).ok()?, self.fonts.options()).ok()?;
            [tree.size().width(), tree.size().height()].map(|n| n.round() as i32)
        } else if is_jpeg(path) {
            let file = std::io::BufReader::new(fs::File::open(path).ok()?);
            let mut decoder = JpegDecoder::new(file);
            decoder.decode_headers().ok()?;
            let (w, h) = decoder.dimensions()?;
            [i32::try_from(w).ok()?, i32::try_from(h).ok()?]
        } else {
            // The IHDR chunk of a PNG file: width and height at 16 and 20.
            let mut head = [0; 24];
            std::io::Read::read_exact(&mut fs::File::open(path).ok()?, &mut head).ok()?;
            let word = |at: usize| {
                u32::from_be_bytes([head[at], head[at + 1], head[at + 2], head[at + 3]])
            };
            [i32::try_from(word(16)).ok()?, i32::try_from(word(20)).ok()?]
        };
        (size[0] > 0 && size[1] > 0).then_some(size)
    }
}

/// Whether a file is an SVG file, by its extension.
fn is_svg(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("svg"))
}

/// A PNG or JPEG file as a pixmap.
fn load_raster(path: &Path) -> Result<Pixmap, String> {
    if !is_jpeg(path) {
        return Pixmap::load_png(path).map_err(|e| e.to_string());
    }
    let file = std::io::BufReader::new(fs::File::open(path).map_err(|e| e.to_string())?);
    let options = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA);
    let mut decoder = JpegDecoder::new_with_options(file, options);
    // Opaque pixels are premultiplied already.
    let pixels = decoder.decode().map_err(|e| e.to_string())?;
    let info = decoder.info().ok_or("no JPEG header")?;
    IntSize::from_wh(info.width.into(), info.height.into())
        .and_then(|size| Pixmap::from_vec(pixels, size))
        .ok_or_else(|| "a broken JPEG file".into())
}

/// Whether a file is a JPEG file, by its extension.
pub fn is_jpeg(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("jpg") || e.eq_ignore_ascii_case("jpeg"))
}

/// How a file fills a rectangle.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Fit {
    /// Inside the rectangle and centered, keeping the aspect ratio.
    Inside,
    /// Over the whole rectangle, keeping the aspect ratio: the overflow
    /// is cut.
    Cover,
    /// Over the whole rectangle, out of its aspect ratio.
    Stretch,
    /// Tiles of a width and height from the top-left corner of the
    /// rectangle, repeated over it.
    Tile([i32; 2]),
}

/// The transform that puts an image of a size into a rectangle; for a
/// tile, the transform of the first tile.
fn place(rect: Rect, [width, height]: [f32; 2], fit: Fit) -> Transform {
    #[allow(clippy::cast_precision_loss)]
    let [x, y, room_x, room_y] = rect.map(|n| n as f32);
    let (across, down) = (room_x / width, room_y / height);
    let scale = match fit {
        Fit::Inside => across.min(down),
        Fit::Cover => across.max(down),
        Fit::Stretch => return Transform::from_translate(x, y).pre_scale(across, down),
        Fit::Tile([w, h]) => {
            #[allow(clippy::cast_precision_loss)]
            let [w, h] = [w as f32, h as f32];
            return Transform::from_translate(x, y).pre_scale(w / width, h / height);
        }
    };
    Transform::from_translate(
        x + (room_x - width * scale) / 2.0,
        y + (room_y - height * scale) / 2.0,
    )
    .pre_scale(scale, scale)
}

/// The transform of the parent of a node: `render_node` applies the
/// transform of a group itself.
fn parent_transform(node: &usvg::Node) -> Transform {
    match node {
        usvg::Node::Group(group) => group
            .transform()
            .invert()
            .map_or(group.abs_transform(), |inv| {
                group.abs_transform().pre_concat(inv)
            }),
        other => other.abs_transform(),
    }
}

/// The id of a part of a frame: `prefix-name`, or `name` without prefix.
fn part(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.into()
    } else {
        format!("{prefix}-{name}")
    }
}

/// Creates a directory, or removes the PNG files in it.
fn clear(dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "png" || e == "part") {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The theme of the source tree, with a cache of its own for a test.
    pub fn theme(test: &str) -> (Theme, PathBuf) {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flavors/xp/theme");
        let cache = std::env::temp_dir().join(format!("chora-xp-{test}-{}", std::process::id()));
        (Theme::load(&dir, &cache).expect("the theme loads"), cache)
    }

    #[test]
    #[allow(clippy::float_cmp)] // insets are whole pixels
    fn frames_have_the_sizes_of_their_parts() {
        let (mut theme, cache) = theme("frames");
        let frame = theme.frame(Sheet::Decoration, "decoration");
        assert_eq!(frame.slice, [34.0, 40.0, 34.0, 34.0]);
        let button = theme.frame(Sheet::Close, "active");
        assert_eq!(button.slice, [3.0, 3.0, 3.0, 3.0]);
        let image = Pixmap::load_png(&frame.image).expect("a PNG file");
        assert_eq!([image.width(), image.height()], [156, 162]);
        let _ = fs::remove_dir_all(cache);
    }

    #[test]
    fn jpeg_files_decode() {
        let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/red.jpg");
        let image = load_raster(&file).expect("a JPEG file");
        assert_eq!([image.width(), image.height()], [8, 8]);
        let pixel = image.pixel(4, 4).expect("inside");
        assert!(pixel.red() > 150 && pixel.green() < 80 && pixel.alpha() == 255);
    }

    #[test]
    fn the_start_button_is_green() {
        let (mut theme, cache) = theme("start");
        let picture =
            Picture::new(100, 30).element(Sheet::Winxp, "startbutton-normal", [0, 0, 100, 30]);
        let image = Pixmap::load_png(theme.render(&picture)).expect("a PNG file");
        let pixel = image.pixel(50, 15).expect("inside");
        assert!(pixel.green() > pixel.red() && pixel.green() > pixel.blue());
        let _ = fs::remove_dir_all(cache);
    }

    #[test]
    fn pictures_render_at_the_scale_with_their_logical_size() {
        let (mut theme, cache) = theme("scale");
        let picture =
            Picture::new(100, 30).element(Sheet::Winxp, "startbutton-normal", [0, 0, 100, 30]);
        let flat = theme.render(&picture);
        assert!(theme.set_scale(1.5));
        assert!(!theme.set_scale(1.5));
        let (sharp, size) = theme.rerender(&flat).expect("a picture");
        let image = Pixmap::load_png(&sharp).expect("a PNG file");
        assert_eq!([image.width(), image.height()], [150, 45]);
        assert_eq!(size, [100.0, 30.0]);
        // The node of the picture gets its logical size, one with a
        // slice keeps the pixels of its file.
        let image = |slice: Option<[f64; 4]>| Command::Update {
            id: "n".into(),
            props: chora_scene::PropsUpdate {
                image: Some(sharp.clone()),
                slice,
                ..chora_scene::PropsUpdate::default()
            },
        };
        let mut commands = [image(None), image(Some([1.0; 4]))];
        theme.size_images(&mut commands);
        let sizes = commands.map(|c| match c {
            Command::Update { props, .. } => props.size,
            _ => None,
        });
        assert_eq!(sizes, [Some([100.0, 30.0]), None]);
        let _ = fs::remove_dir_all(cache);
    }

    #[test]
    fn tiles_keep_their_size_at_a_scale() {
        let (mut theme, cache) = theme("tile");
        // A tile of 8 by 4, red on the left, blue on the right.
        let mut tile = Pixmap::new(8, 4).expect("pixels");
        tile.fill_rect(
            tiny_skia::Rect::from_xywh(0.0, 0.0, 4.0, 4.0).expect("a rect"),
            &Paint {
                shader: tiny_skia::Shader::SolidColor(tiny_skia::Color::from_rgba8(255, 0, 0, 255)),
                ..Paint::default()
            },
            Transform::identity(),
            None,
        );
        tile.fill_rect(
            tiny_skia::Rect::from_xywh(4.0, 0.0, 4.0, 4.0).expect("a rect"),
            &Paint {
                shader: tiny_skia::Shader::SolidColor(tiny_skia::Color::from_rgba8(0, 0, 255, 255)),
                ..Paint::default()
            },
            Transform::identity(),
            None,
        );
        let file = cache.join("tile.png");
        tile.save_png(&file).expect("a PNG file");
        let picture = Picture::new(16, 4).fit(&file, [0, 0, 16, 4], Fit::Tile([8, 4]));
        let image = Pixmap::load_png(theme.render_at(&picture, 2.0, false)).expect("a PNG file");
        // Logical x 6 is in the blue half of the first tile.
        let pixel = image.pixel(12, 4).expect("inside");
        assert!(pixel.blue() > 200 && pixel.red() < 50, "{pixel:?}");
        let _ = fs::remove_dir_all(cache);
    }
}
