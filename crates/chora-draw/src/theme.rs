//! The asset folder of a block: its SVG files, rasterized with resvg into
//! PNG files for image nodes. The block names the sheets that it needs.
//!
//! A [`Picture`] describes an image as layers: elements of the SVG files
//! by id, frames by prefix, icons, fills and text. Each picture renders
//! once per scale; the name of its file is the hash of the description
//! and the scale. The size of a picture is in logical pixels (see "The XP
//! flavor" in docs/configuration.md).

use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
    rc::Rc,
};

use resvg::{
    tiny_skia::{
        self, FilterQuality, IntSize, Paint, Pattern, Pixmap, PixmapMut, PixmapPaint, SpreadMode,
        Transform,
    },
    usvg,
};
use zune_jpeg::{
    JpegDecoder,
    zune_core::{colorspace::ColorSpace, options::DecoderOptions},
};

use chora_scene::Command;

use crate::text::{Font, Fonts, Text};

/// A rectangle: x, y, width and height in logical pixels.
pub type Rect = [i32; 4];

/// An SVG file of the asset folder, by its path in the folder. The
/// elements of the file have ids that pictures name.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Sheet(pub &'static str);

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
    /// An element of a sheet where it stands in the sheet: the whole
    /// sheet scaled evenly into a rectangle, in its middle, with only this
    /// element drawn.
    InPlace {
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
    /// A line of text.
    Text(Text),
    /// SVG elements in the coordinates of the picture, such as the
    /// rectangles and gradients of a control.
    Svg(String),
}

/// An image as a stack of layers, bottom first.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Picture {
    /// Width and height in logical pixels.
    pub size: [i32; 2],
    /// The layers.
    pub layers: Vec<Layer>,
}

impl Picture {
    /// An empty picture.
    #[must_use]
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            size: [width.max(1), height.max(1)],
            layers: Vec::new(),
        }
    }

    /// Adds an element of a sheet.
    #[must_use]
    pub fn element(mut self, sheet: Sheet, id: &str, rect: Rect) -> Self {
        self.layers.push(Layer::Element {
            sheet,
            id: id.into(),
            rect,
        });
        self
    }

    /// Adds an element of a sheet where it stands in the sheet, the sheet
    /// scaled evenly into a rectangle: a figure of a sheet with one
    /// element per state keeps its aspect and its strokes.
    #[must_use]
    pub fn in_place(mut self, sheet: Sheet, id: &str, rect: Rect) -> Self {
        self.layers.push(Layer::InPlace {
            sheet,
            id: id.into(),
            rect,
        });
        self
    }

    /// Adds a frame of a sheet.
    #[must_use]
    pub fn frame(mut self, sheet: Sheet, prefix: &str, rect: Rect) -> Self {
        self.layers.push(Layer::Frame {
            sheet,
            prefix: prefix.into(),
            rect,
        });
        self
    }

    /// Adds an SVG or PNG file.
    #[must_use]
    pub fn file(mut self, path: &Path, rect: Rect) -> Self {
        self.layers.push(Layer::File {
            path: path.into(),
            rect,
            fit: Fit::Inside,
        });
        self
    }

    /// Adds a file that fills a rectangle in a way of [`Fit`].
    #[must_use]
    pub fn fit(mut self, path: &Path, rect: Rect, fit: Fit) -> Self {
        self.layers.push(Layer::File {
            path: path.into(),
            rect,
            fit,
        });
        self
    }

    /// Adds a line of text.
    #[must_use]
    pub fn text(mut self, text: Text) -> Self {
        self.layers.push(Layer::Text(text));
        self
    }

    /// Adds the layers of another picture, moved by an offset.
    #[must_use]
    pub fn layers_of(mut self, other: Self, [dx, dy]: [i32; 2]) -> Self {
        let shift = |[x, y, w, h]: Rect| [x + dx, y + dy, w, h];
        for layer in other.layers {
            self.layers.push(match layer {
                Layer::Element { sheet, id, rect } => Layer::Element {
                    sheet,
                    id,
                    rect: shift(rect),
                },
                Layer::InPlace { sheet, id, rect } => Layer::InPlace {
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
    #[must_use]
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

/// The asset folder and the cache of rendered files.
pub struct Theme {
    dir: PathBuf,
    /// Whether the folder is a HUD folder: untrusted, so that each file
    /// is read through [`chora_kit::hud::file`] and a picture is at most
    /// [`crate::assets::MAX_SIDE`] on a side.
    confined: bool,
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
    /// The parsed SVG files by path, so that a file parses once.
    trees: RefCell<HashMap<PathBuf, Rc<usvg::Tree>>>,
}

/// How many rendered files the cache keeps.
const CACHE_SIZE: usize = 512;
/// How many pictures of rendered files the theme remembers.
const PICTURES: usize = 8192;
/// The largest scale of the pictures.
const MAX_SCALE: f64 = 4.0;

impl Theme {
    /// The theme with XP's Large Fonts, see [`Fonts::large_chrome`]: for
    /// the chrome of XP, before anything is drawn.
    #[must_use]
    pub fn large_chrome(mut self) -> Self {
        self.fonts.large_chrome();
        self
    }

    /// Loads the `sheets` and the fonts of an asset folder; `family` is
    /// the base font family. Rendered files go to `cache`, which this
    /// call empties first.
    ///
    /// # Errors
    ///
    /// A sheet is missing or broken, or the cache cannot be written; the
    /// message names the file.
    pub fn load(dir: &Path, cache: &Path, names: &[Sheet], family: &str) -> Result<Self, String> {
        Self::open(dir, cache, names, (Fonts::load(dir, family), false))
    }

    /// Loads a HUD folder, which is untrusted. It may change how things
    /// look and nothing else: a file is read only inside the folder, within
    /// its size limit, an SVG over 2048 px on a side is refused, an `href`
    /// of an image is a `data:` URI or a path inside the folder, and the
    /// fonts of the folder go to a font database that [`Fonts::host`]
    /// does not share.
    ///
    /// # Errors
    ///
    /// A sheet is missing, broken or too large, or the cache cannot be
    /// written; the message names the file.
    pub fn load_hud(
        dir: &Path,
        cache: &Path,
        names: &[Sheet],
        family: &str,
    ) -> Result<Self, String> {
        Self::open(dir, cache, names, (Fonts::folder(dir, family), true))
    }

    /// A theme with the fonts of the host only and no folder, for what the
    /// host draws itself, such as the question card. It reads no file.
    ///
    /// # Errors
    ///
    /// The cache cannot be written.
    pub fn host(cache: &Path, family: &str) -> Result<Self, String> {
        Self::open(cache, cache, &[], (Fonts::host(family), true))
    }

    /// Whether the folder is a HUD folder, read as untrusted.
    #[must_use]
    pub fn confined(&self) -> bool {
        self.confined
    }

    /// The fonts of the theme.
    #[must_use]
    pub fn fonts(&self) -> &Fonts {
        &self.fonts
    }

    fn open(
        dir: &Path,
        cache: &Path,
        names: &[Sheet],
        (fonts, confined): (Fonts, bool),
    ) -> Result<Self, String> {
        let mut sheets = HashMap::new();
        for sheet in names {
            let path = dir.join(sheet.0);
            let tree = read_tree((dir, confined), fonts.options(), &path)?;
            sheets.insert(*sheet, tree);
        }
        clear(cache).map_err(|e| format!("{}: {e}", cache.display()))?;
        let mut theme = Self {
            dir: dir.into(),
            confined,
            cache: cache.into(),
            fonts,
            sheets,
            rendered: HashMap::new(),
            clock: 0,
            blank: String::new(),
            widths: RefCell::default(),
            scale: 1.0,
            pictures: HashMap::new(),
            trees: RefCell::default(),
        };
        let blank = Pixmap::new(1, 1).ok_or("no pixmap of one pixel")?;
        theme.blank = theme
            .write(0, &blank)
            .map_err(|e| format!("{}: {e}", cache.display()))?;
        Ok(theme)
    }

    fn tree(&self, path: &Path) -> Result<Rc<usvg::Tree>, String> {
        if let Some(tree) = self.trees.borrow().get(path) {
            return Ok(Rc::clone(tree));
        }
        let tree = Rc::new(read_tree(self.folder(), self.fonts.options(), path)?);
        self.trees
            .borrow_mut()
            .insert(path.to_path_buf(), Rc::clone(&tree));
        Ok(tree)
    }

    fn folder(&self) -> (&Path, bool) {
        (&self.dir, self.confined)
    }

    /// A file of the asset folder.
    pub fn path(&self, file: &str) -> PathBuf {
        self.dir.join(file)
    }

    /// Writes a file into the cache, for example a shader for the core,
    /// and returns its path.
    ///
    /// # Errors
    ///
    /// The file cannot be written.
    pub fn cache_file(&self, name: &str, data: &str) -> std::io::Result<PathBuf> {
        let path = self.cache.join(name);
        chora_kit::write_file(&path, data.as_bytes())?;
        Ok(path)
    }

    /// The width of a line of text in logical pixels.
    pub fn text_width(&self, line: &str, font: Font) -> f64 {
        let key = (line.to_owned(), font);
        if let Some(width) = self.widths.borrow().get(&key) {
            return *width;
        }
        let width = self.fonts.width(line, font).unwrap_or(0.0);
        let mut widths = self.widths.borrow_mut();
        if widths.len() >= 4096 {
            widths.clear();
        }
        widths.insert(key, width);
        width
    }

    /// The right edge of the ink of SVG text elements drawn from x 0, in
    /// logical pixels, kept for the next ask.
    pub fn svg_width(&self, body: &str) -> f64 {
        let key = (body.to_owned(), Font::Body);
        if let Some(width) = self.widths.borrow().get(&key) {
            return *width;
        }
        let width = self.fonts.svg_width(body).unwrap_or(0.0);
        let mut widths = self.widths.borrow_mut();
        if widths.len() >= 4096 {
            widths.clear();
        }
        widths.insert(key, width);
        width
    }

    /// The size of an SVG file of the folder, as its `width` and `height`
    /// or its `viewBox` say, rounded to pixels.
    pub fn sheet_size(&self, sheet: Sheet) -> Option<[i32; 2]> {
        let size = self.sheets.get(&sheet)?.size();
        #[allow(clippy::cast_possible_truncation)]
        let size = [size.width().round() as i32, size.height().round() as i32];
        (size[0] > 0 && size[1] > 0).then_some(size)
    }

    /// The size of an element, rounded to pixels.
    pub fn size(&self, sheet: Sheet, id: &str) -> Option<[i32; 2]> {
        let node = self.sheets.get(&sheet)?.node_by_id(id)?;
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
            if self.pictures.len() >= PICTURES {
                self.pictures.clear();
            }
            self.pictures.insert(path.clone(), picture.clone());
        }
        path
    }

    /// The PNG file of a picture at scale 1, for an image node with a
    /// non-zero slice.
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
                eprintln!("chora-draw: rendering failed: {err}");
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
        let data = pixmap.encode_png().map_err(|e| e.to_string())?;
        chora_kit::write_file(&path, &data).map_err(|e| e.to_string())?;
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
            if let Err(err) = self.draw_layer(layer, (base, &mut pixmap.as_mut())) {
                eprintln!("chora-draw: {err}");
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
            Layer::InPlace { sheet, id, rect } => {
                self.draw_in_place(*sheet, id, *rect, (base, pixmap))
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
        let missing = || format!("{} has no element {id:?}", sheet.0);
        let node = self.sheets.get(&sheet).and_then(|t| t.node_by_id(id));
        let node = node.ok_or_else(missing)?;
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

    /// Draws an element where it stands in its sheet, the sheet scaled
    /// evenly into the middle of a rectangle.
    fn draw_in_place(
        &self,
        sheet: Sheet,
        id: &str,
        rect: Rect,
        (base, pixmap): (Transform, &mut PixmapMut<'_>),
    ) -> Result<(), String> {
        let missing = || format!("{} has no element {id:?}", sheet.0);
        let tree = self.sheets.get(&sheet).ok_or_else(missing)?;
        let node = tree.node_by_id(id).ok_or_else(missing)?;
        let Some(layer) = node.abs_layer_bounding_box() else {
            return Ok(());
        };
        let size = tree.size();
        #[allow(
            clippy::cast_precision_loss,
            reason = "a picture is far smaller than 2^23 pixels"
        )]
        let [x, y, w, h] = rect.map(|v| v as f32);
        let scale = (w / size.width()).min(h / size.height());
        let left = x + (w - size.width() * scale) / 2.0;
        let top = y + (h - size.height() * scale) / 2.0;
        let transform = base
            .pre_translate(left, top)
            .pre_scale(scale, scale)
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
                let _ = self.draw_element(sheet, &part(prefix, name), cell, (base, pixmap));
            }
        }
    }

    /// Draws an SVG, PNG or JPEG file scaled to a rectangle.
    fn draw_file(
        &self,
        path: &Path,
        rect: Rect,
        fit: Fit,
        (base, pixmap): (Transform, &mut PixmapMut<'_>),
    ) -> Result<(), String> {
        let error = |e: &dyn std::fmt::Display| format!("{}: {e}", path.display());
        let image = if is_svg(path) {
            let tree = self.tree(path)?;
            let size = tree.size();
            if !matches!(fit, Fit::Tile(_)) {
                resvg::render(
                    &tree,
                    base.pre_concat(place(rect, [size.width(), size.height()], fit)),
                    pixmap,
                );
                return Ok(());
            }
            let [w, h] = [size.width(), size.height()].map(f32::ceil);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let mut tile = Pixmap::new(w as u32, h as u32).ok_or_else(|| error(&"empty"))?;
            resvg::render(&tree, Transform::identity(), &mut tile.as_mut());
            tile
        } else {
            load_raster(self.folder(), path).map_err(|e| error(&e))?
        };
        #[allow(clippy::cast_precision_loss)]
        let size = [image.width() as f32, image.height() as f32];
        let transform = place(rect, size, fit);
        if let Fit::Tile(_) = fit {
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
            let tree = self.tree(path).ok()?;
            [tree.size().width(), tree.size().height()].map(|n| n.round() as i32)
        } else if self.confined {
            let data = read_bytes(self.folder(), path).ok()?;
            let [w, h] = crate::assets::raster_size(&data)?;
            [i32::try_from(w).ok()?, i32::try_from(h).ok()?]
        } else {
            header_size(path)?
        };
        (size[0] > 0 && size[1] > 0).then_some(size)
    }
}

/// The width and height of a PNG or JPEG file of the trusted asset folder,
/// from its header.
fn header_size(path: &Path) -> Option<[i32; 2]> {
    if is_jpeg(path) {
        let file = std::io::BufReader::new(chora_kit::open_file(path).ok()?);
        let mut decoder = JpegDecoder::new(file);
        decoder.decode_headers().ok()?;
        let (w, h) = decoder.dimensions()?;
        return Some([i32::try_from(w).ok()?, i32::try_from(h).ok()?]);
    }
    let mut head = [0; 24];
    std::io::Read::read_exact(&mut chora_kit::open_file(path).ok()?, &mut head).ok()?;
    let word = |at: usize| u32::from_be_bytes([head[at], head[at + 1], head[at + 2], head[at + 3]]);
    Some([i32::try_from(word(16)).ok()?, i32::try_from(word(20)).ok()?])
}

/// Whether a file is an SVG file, by its extension.
fn is_svg(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("svg"))
}

/// The bytes of a file. In a HUD folder (`confined`) the file is a path
/// inside the folder, read through [`chora_kit::hud::file`].
fn read_bytes((dir, confined): (&Path, bool), path: &Path) -> std::io::Result<Vec<u8>> {
    if !confined {
        return chora_kit::read_file(path, chora_kit::MAX_PICTURE);
    }
    let outside = || {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "not a path inside the folder",
        )
    };
    let name = path.strip_prefix(dir).map_err(|_| outside())?;
    chora_kit::hud::file(dir, name.to_str().ok_or_else(outside)?)
}

/// An SVG file as a tree; in a HUD folder it is also at most
/// [`crate::assets::MAX_SIDE`] on a side.
fn read_tree(
    folder: (&Path, bool),
    options: &usvg::Options<'_>,
    path: &Path,
) -> Result<usvg::Tree, String> {
    let error = |e: &dyn std::fmt::Display| format!("{}: {e}", path.display());
    let data = read_bytes(folder, path).map_err(|e| error(&e))?;
    let tree = usvg::Tree::from_data(&data, options).map_err(|e| error(&e))?;
    if folder.1 {
        crate::assets::check_tree(&tree).map_err(|e| error(&e))?;
    }
    Ok(tree)
}

/// A PNG or JPEG file as a pixmap.
fn load_raster(folder: (&Path, bool), path: &Path) -> Result<Pixmap, String> {
    let data = read_bytes(folder, path).map_err(|e| e.to_string())?;
    let limit = if folder.1 {
        crate::assets::MAX_SIDE
    } else {
        8192
    };
    if folder.1 {
        let [w, h] = crate::assets::raster_size(&data).ok_or("not a PNG or JPEG file")?;
        if w > limit || h > limit {
            return Err(format!("picture of {w} x {h} px, at most {limit}"));
        }
    }
    if !is_jpeg(path) {
        return Pixmap::decode_png(&data).map_err(|e| e.to_string());
    }
    let options = DecoderOptions::default()
        .jpeg_set_out_colorspace(ColorSpace::RGBA)
        .set_max_width(limit as usize)
        .set_max_height(limit as usize);
    let mut decoder = JpegDecoder::new_with_options(std::io::Cursor::new(data), options);
    let pixels = decoder.decode().map_err(|e| e.to_string())?;
    let info = decoder.info().ok_or("no JPEG header")?;
    IntSize::from_wh(info.width.into(), info.height.into())
        .and_then(|size| Pixmap::from_vec(pixels, size))
        .ok_or_else(|| "a broken JPEG file".into())
}

/// Whether a file is a JPEG file, by its extension.
#[must_use]
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

/// The directory of the rendered files of a block: one per chora
/// instance, `$XDG_RUNTIME_DIR/NAME/INSTANCE`, which the core can read.
///
/// # Errors
///
/// `$XDG_RUNTIME_DIR` is not an absolute path.
pub fn cache_dir(name: &str) -> Result<PathBuf, String> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .ok_or("no absolute $XDG_RUNTIME_DIR for the cache")?;
    let instance = std::env::var_os(chora_scene::ipc::SOCKET_ENV)
        .map(PathBuf::from)
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "default".into());
    Ok(runtime.join(name).join(instance))
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

    const SHEET: Sheet = Sheet("sheet.svg");

    /// A theme with the sheet of the test data, and a cache of its own.
    fn theme(test: &str) -> (Theme, PathBuf) {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata");
        let cache = std::env::temp_dir().join(format!("chora-draw-{test}-{}", std::process::id()));
        let theme = Theme::load(&dir, &cache, &[SHEET], "sans-serif");
        (theme.expect("the theme loads"), cache)
    }

    #[test]
    #[allow(clippy::float_cmp, reason = "insets are whole pixels")]
    fn frames_have_the_sizes_of_their_parts() {
        let (mut theme, cache) = theme("frames");
        let frame = theme.frame(SHEET, "f");
        assert_eq!(frame.slice, [3.0, 4.0, 3.0, 5.0]);
        let image = Pixmap::load_png(&frame.image).expect("a PNG file");
        assert_eq!([image.width(), image.height()], [7, 10]);
        let _ = fs::remove_dir_all(cache);
    }

    #[test]
    fn jpeg_files_decode() {
        let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/red.jpg");
        let image = load_raster((Path::new(""), false), &file).expect("a JPEG file");
        assert_eq!([image.width(), image.height()], [8, 8]);
        let pixel = image.pixel(4, 4).expect("inside");
        assert!(pixel.red() > 150 && pixel.green() < 80 && pixel.alpha() == 255);
    }

    #[test]
    fn an_element_in_place_keeps_its_place_and_its_aspect() {
        let (mut theme, cache) = theme("in-place");
        let picture = Picture::new(80, 120).in_place(SHEET, "box", [0, 0, 80, 120]);
        let image = Pixmap::load_png(theme.render(&picture)).expect("a PNG file");
        let alpha = |x, y| image.pixel(x, y).expect("inside").alpha();
        assert_eq!(
            alpha(20, 30),
            255,
            "the box, twice its size, the sheet in the middle"
        );
        assert_eq!(alpha(20, 10), 0, "above the sheet");
        assert_eq!(alpha(60, 30), 0, "right of the box: not stretched");
        assert_eq!(alpha(20, 50), 0, "below the box: not stretched");
        let _ = fs::remove_dir_all(cache);
    }

    /// Every state of the paperclip of xp, at 1x and 2x: drawn whole, with
    /// an empty edge all round, so that nothing is cut off.
    #[test]
    fn the_paperclip_stays_whole_at_both_scales() {
        const CLIP: Sheet = Sheet("paperclip.svg");
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flavors/xp/assets/assistant");
        let cache = std::env::temp_dir().join(format!("chora-draw-clip-{}", std::process::id()));
        let mut theme = Theme::load(&dir, &cache, &[CLIP], "sans-serif").expect("the sheet");
        let [w, h] = theme.sheet_size(CLIP).expect("a size");
        for scale in [1.0, 2.0] {
            theme.set_scale(scale);
            for state in [
                "idle",
                "listening",
                "thinking",
                "working",
                "speaking",
                "question",
            ] {
                let picture = Picture::new(w, h).in_place(CLIP, state, [0, 0, w, h]);
                let image = Pixmap::load_png(theme.render(&picture)).expect("a PNG file");
                let (iw, ih) = (image.width(), image.height());
                let edge = (0..iw)
                    .flat_map(|x| [(x, 0), (x, ih - 1)])
                    .chain((0..ih).flat_map(|y| [(0, y), (iw - 1, y)]));
                let cut = edge
                    .filter(|&(x, y)| image.pixel(x, y).is_some_and(|p| p.alpha() > 0))
                    .count();
                assert_eq!(cut, 0, "{state} at {scale}x touches its edge");
            }
        }
        let _ = fs::remove_dir_all(cache);
    }

    #[test]
    fn an_element_is_green() {
        let (mut theme, cache) = theme("green");
        let picture = Picture::new(100, 30).element(SHEET, "box", [0, 0, 100, 30]);
        let image = Pixmap::load_png(theme.render(&picture)).expect("a PNG file");
        let pixel = image.pixel(50, 15).expect("inside");
        assert!(pixel.green() > pixel.red() && pixel.green() > pixel.blue());
        let _ = fs::remove_dir_all(cache);
    }

    #[test]
    fn pictures_render_at_the_scale_with_their_logical_size() {
        let (mut theme, cache) = theme("scale");
        let picture = Picture::new(100, 30).element(SHEET, "box", [0, 0, 100, 30]);
        let flat = theme.render(&picture);
        assert!(theme.set_scale(1.5));
        assert!(!theme.set_scale(1.5));
        let (sharp, size) = theme.rerender(&flat).expect("a picture");
        let image = Pixmap::load_png(&sharp).expect("a PNG file");
        assert_eq!([image.width(), image.height()], [150, 45]);
        assert_eq!(size, [100.0, 30.0]);
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
        let mut tile = Pixmap::new(8, 4).expect("pixels");
        let mut paint = Paint::default();
        for (x, [r, b]) in [(0.0, [255, 0]), (4.0, [0, 255])] {
            paint.set_color_rgba8(r, 0, b, 255);
            let half = tiny_skia::Rect::from_xywh(x, 0.0, 4.0, 4.0).expect("a rect");
            tile.fill_rect(half, &paint, Transform::identity(), None);
        }
        let file = cache.join("tile.png");
        tile.save_png(&file).expect("a PNG file");
        let picture = Picture::new(16, 4).fit(&file, [0, 0, 16, 4], Fit::Tile([8, 4]));
        let image = Pixmap::load_png(theme.render_at(&picture, 2.0, false)).expect("a PNG file");
        let pixel = image.pixel(12, 4).expect("inside");
        assert!(pixel.blue() > 200 && pixel.red() < 50, "{pixel:?}");
        let _ = fs::remove_dir_all(cache);
    }

    struct Folder {
        root: PathBuf,
        dir: PathBuf,
        outside: PathBuf,
    }

    impl Folder {
        fn new(test: &str) -> Self {
            let root =
                std::env::temp_dir().join(format!("chora-hud-{test}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&root);
            let dir = root.join("folder");
            let outside = root.join("outside");
            fs::create_dir_all(&dir).expect("a folder");
            fs::create_dir_all(&outside).expect("a folder");
            let red = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20"><rect width="20" height="20" fill="red"/></svg>"#;
            fs::write(outside.join("secret.svg"), red).expect("a file");
            fs::write(dir.join("ok.svg"), red).expect("a file");
            Self { root, dir, outside }
        }

        fn put(&self, name: &str, data: &str) {
            fs::write(self.dir.join(name), data).expect("a file");
        }

        fn theme(&self, names: &[Sheet]) -> Result<Theme, String> {
            Theme::load_hud(&self.dir, &self.root.join("cache"), names, "sans-serif")
        }

        fn svg_with(&self, name: &str, href: &str) {
            self.put(
                name,
                &format!(
                    r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="20" height="20"><image width="20" height="20" xlink:href="{href}"/></svg>"#
                ),
            );
        }
    }

    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn inked(theme: &Theme, path: &Path) -> bool {
        let picture = Picture::new(20, 20).file(path, [0, 0, 20, 20]);
        let pixmap = theme.draw(&picture, 1.0).expect("a pixmap");
        pixmap.pixels().iter().any(|p| p.alpha() > 0)
    }

    #[test]
    fn an_image_href_outside_the_folder_draws_nothing() {
        let folder = Folder::new("href");
        let secret = folder.outside.join("secret.svg");
        std::os::unix::fs::symlink(&secret, folder.dir.join("link.svg")).expect("a link");
        std::os::unix::fs::symlink("/dev/zero", folder.dir.join("zero.svg")).expect("a link");
        let theme = folder.theme(&[]).expect("the folder loads");
        let mut red = Pixmap::new(20, 20).expect("pixels");
        red.fill(tiny_skia::Color::from_rgba8(255, 0, 0, 255));
        fs::write(folder.dir.join("ok.png"), red.encode_png().expect("a png")).expect("a file");
        let resolver = crate::assets::href_resolver(&folder.dir);
        let options = usvg::Options::default();
        assert!((resolver.resolve_string)("ok.png", &options).is_some());
        assert!((resolver.resolve_string)("ok.svg", &options).is_none());
        let hrefs = [
            secret.display().to_string(),
            format!("file://{}", secret.display()),
            "../outside/secret.svg".into(),
            "http://example.invalid/secret.svg".into(),
            "/etc/passwd".into(),
            "/dev/zero".into(),
            "link.svg".into(),
            "zero.svg".into(),
        ];
        for (n, href) in hrefs.iter().enumerate() {
            let name = format!("evil{n}.svg");
            folder.svg_with(&name, href);
            assert!(!inked(&theme, &theme.path(&name)), "{href} was drawn");
        }
        assert!(!inked(&theme, &secret), "a file outside was drawn");
        assert!(!inked(&theme, Path::new("/etc/passwd")));
        let uri = "data:image/svg+xml;utf8,&lt;svg xmlns='http://www.w3.org/2000/svg' width='20' height='20'>&lt;rect width='20' height='20' fill='red'/&gt;&lt;/svg&gt;";
        folder.svg_with("data.svg", uri);
        assert!(!inked(&theme, &theme.path("data.svg")), "no SVG in a URI");
    }

    #[test]
    fn a_huge_svg_is_refused() {
        let folder = Folder::new("huge");
        let svg = |side: u32| {
            format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{side}" height="10"/>"#)
        };
        folder.put("huge.svg", &svg(2049));
        folder.put("edge.svg", &svg(2048));
        let theme = folder.theme(&[]).expect("the folder loads");
        assert_eq!(theme.image_size(&theme.path("huge.svg")), None);
        assert_eq!(theme.image_size(&theme.path("edge.svg")), Some([2048, 10]));
        let picture = Picture::new(20, 20).file(&theme.path("huge.svg"), [0, 0, 20, 20]);
        let (w, h) = (20, 20);
        let mut pixmap = Pixmap::new(w, h).expect("pixels");
        let err = theme
            .draw_layer(
                &picture.layers[0],
                (Transform::identity(), &mut pixmap.as_mut()),
            )
            .expect_err("refused");
        assert!(err.contains("huge.svg"), "{err}");
        let err = folder.theme(&[Sheet("huge.svg")]).err().expect("refused");
        assert!(err.contains("huge.svg"), "{err}");
        let uri = "data:image/svg+xml;utf8,&lt;svg xmlns='http://www.w3.org/2000/svg' width='9000' height='10'>&lt;rect width='9000' height='10' fill='red'/&gt;&lt;/svg&gt;";
        folder.svg_with("nested.svg", uri);
        assert!(!inked(&theme, &theme.path("nested.svg")));
    }

    fn svg_of(body: &str) -> String {
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="20" height="20">{body}</svg>"#
        )
    }

    #[test]
    fn a_picture_with_too_many_nodes_is_refused() {
        let folder = Folder::new("nodes");
        let mut defs = vec![r#"<defs><g id="g0"><rect width="1" height="1"/></g>"#.to_owned()];
        for level in 1..5 {
            let uses = format!(r##"<use xlink:href="#g{}"/>"##, level - 1).repeat(10);
            defs.push(format!(r#"<g id="g{level}">{uses}</g>"#));
        }
        defs.push(r##"</defs><use xlink:href="#g4"/>"##.to_owned());
        folder.put("fan.svg", &svg_of(&defs.concat()));
        folder.put("few.svg", &svg_of(r#"<rect width="1" height="1"/>"#));
        let theme = folder.theme(&[]).expect("the folder loads");
        assert!(theme.image_size(&theme.path("few.svg")).is_some());
        assert_eq!(theme.image_size(&theme.path("fan.svg")), None);
        let err = folder.theme(&[Sheet("fan.svg")]).err().expect("refused");
        assert!(err.contains("fan.svg"), "{err}");
    }

    #[test]
    fn a_picture_with_too_many_images_or_filters_is_refused() {
        let folder = Folder::new("costly");
        let image = r#"<image width="1" height="1" href="data:image/png;base64,iVBORw0KGgo="/>"#;
        folder.put("images.svg", &svg_of(&image.repeat(17)));
        let filter = r#"<filter id="f"><feOffset dx="1"/></filter>"#;
        let user = r#"<rect width="1" height="1" filter="url(#f)"/>"#;
        folder.put(
            "filters.svg",
            &svg_of(&format!("{filter}{}", user.repeat(17))),
        );
        folder.put("some.svg", &svg_of(&format!("{filter}{}", user.repeat(16))));
        let theme = folder.theme(&[]).expect("the folder loads");
        assert!(theme.image_size(&theme.path("some.svg")).is_some());
        assert_eq!(theme.image_size(&theme.path("filters.svg")), None);
        let uri = "data:image/svg+xml;utf8,&lt;svg xmlns='http://www.w3.org/2000/svg' width='9' height='9'/&gt;";
        folder.svg_with("nested.svg", uri);
        assert!(!inked(&theme, &theme.path("nested.svg")));
    }

    #[test]
    fn a_huge_pattern_tile_is_refused() {
        let folder = Folder::new("pattern");
        let pattern = |side: u32| {
            svg_of(&format!(
                r#"<defs><pattern id="p" patternUnits="userSpaceOnUse" width="{side}" height="{side}"><rect width="5" height="5"/></pattern></defs><rect width="20" height="20" fill="url(#p)"/>"#
            ))
        };
        folder.put("big.svg", &pattern(30000));
        folder.put("small.svg", &pattern(30));
        let theme = folder.theme(&[]).expect("the folder loads");
        assert!(theme.image_size(&theme.path("small.svg")).is_some());
        assert_eq!(theme.image_size(&theme.path("big.svg")), None);
    }

    #[test]
    fn a_parsed_svg_is_cached() {
        let folder = Folder::new("cache");
        let theme = folder.theme(&[]).expect("the folder loads");
        let path = theme.path("ok.svg");
        assert!(theme.image_size(&path).is_some());
        fs::remove_file(&path).expect("removed");
        assert!(theme.image_size(&path).is_some(), "the tree is kept");
    }
}
