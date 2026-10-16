//! The XP theme: the SVG files of the theme directory, rasterized with
//! resvg into PNG files for image nodes.
//!
//! A [`Picture`] describes an image as layers: elements of the SVG files
//! by id, frames by prefix, icons, fills and text. Each picture renders
//! once; the name of its file is the hash of the description.

use std::{
    collections::HashMap,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
};

use resvg::{
    tiny_skia::{FilterQuality, Pixmap, PixmapMut, PixmapPaint, Transform},
    usvg,
};

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
    /// An SVG or PNG file, scaled to a rectangle.
    File {
        /// The file.
        path: PathBuf,
        /// The rectangle.
        rect: Rect,
        /// Scale to cover the rectangle and cut the overflow, keeping the
        /// aspect ratio.
        cover: bool,
    },
    /// A line of text.
    Text(Text),
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
            cover: false,
        });
        self
    }

    /// Adds an SVG or PNG file that covers the whole picture.
    pub fn cover(mut self, path: &Path) -> Self {
        self.layers.push(Layer::File {
            path: path.into(),
            rect: [0, 0, self.size[0], self.size[1]],
            cover: true,
        });
        self
    }

    /// Adds a line of text.
    pub fn text(mut self, text: Text) -> Self {
        self.layers.push(Layer::Text(text));
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
}

/// How many rendered files the cache keeps.
const CACHE_SIZE: usize = 512;

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

    /// The width of a line of text in pixels.
    pub fn text_width(&self, line: &str, font: Font) -> f64 {
        self.fonts.width(line, font).unwrap_or(0.0)
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
            image: self.render(&picture),
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

    /// The PNG file of a picture, rendered on first use. A picture that
    /// fails to render logs and gets a transparent pixel.
    pub fn render(&mut self, picture: &Picture) -> String {
        let mut hasher = DefaultHasher::new();
        picture.hash(&mut hasher);
        let key = hasher.finish() | 1;
        self.clock += 1;
        if let Some(entry) = self.rendered.get_mut(&key) {
            entry.used = self.clock;
            return entry.path.clone();
        }
        let written = self.draw(picture).and_then(|p| self.write(key, &p));
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

    fn draw(&self, picture: &Picture) -> Result<Pixmap, String> {
        let [w, h] = picture.size.map(|n| u32::try_from(n).unwrap_or(1));
        let mut pixmap = Pixmap::new(w, h).ok_or("an empty picture")?;
        for layer in &picture.layers {
            // A missing icon or element leaves the rest of the picture.
            if let Err(err) = self.draw_layer(layer, &mut pixmap.as_mut()) {
                eprintln!("chora-xp: {err}");
            }
        }
        Ok(pixmap)
    }

    fn draw_layer(&self, layer: &Layer, pixmap: &mut PixmapMut<'_>) -> Result<(), String> {
        match layer {
            Layer::Element { sheet, id, rect } => self.draw_element(*sheet, id, *rect, pixmap),
            Layer::Frame {
                sheet,
                prefix,
                rect,
            } => {
                self.draw_frame(*sheet, prefix, *rect, pixmap);
                Ok(())
            }
            Layer::File { path, rect, cover } => self.draw_file(path, *rect, *cover, pixmap),
            Layer::Text(text) => self.fonts.draw(text, pixmap),
        }
    }

    /// Draws an element so that its bounding box fills the rectangle.
    fn draw_element(
        &self,
        sheet: Sheet,
        id: &str,
        rect: Rect,
        pixmap: &mut PixmapMut<'_>,
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
        let transform = Transform::from_translate(x as f32, y as f32)
            .pre_scale(w as f32 / bbox.width(), h as f32 / bbox.height())
            .pre_translate(-bbox.x(), -bbox.y())
            .pre_concat(parent_transform(node))
            .pre_translate(layer.x(), layer.y());
        resvg::render_node(node, transform, pixmap);
        Ok(())
    }

    /// Draws the nine parts of a frame over a rectangle; missing parts
    /// stay empty.
    fn draw_frame(&self, sheet: Sheet, prefix: &str, rect: Rect, pixmap: &mut PixmapMut<'_>) {
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
                let _ = self.draw_element(sheet, &part(prefix, name), cell, pixmap);
            }
        }
    }

    /// Draws an SVG or a PNG file scaled to a rectangle.
    fn draw_file(
        &self,
        path: &Path,
        rect: Rect,
        cover: bool,
        pixmap: &mut PixmapMut<'_>,
    ) -> Result<(), String> {
        let error = |e: &dyn std::fmt::Display| format!("{}: {e}", path.display());
        if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("svg"))
        {
            let data = fs::read(path).map_err(|e| error(&e))?;
            let tree = usvg::Tree::from_data(&data, self.fonts.options()).map_err(|e| error(&e))?;
            let size = tree.size();
            let transform = fit(rect, [size.width(), size.height()], cover);
            resvg::render(&tree, transform, pixmap);
            return Ok(());
        }
        let image = Pixmap::load_png(path).map_err(|e| error(&e))?;
        #[allow(clippy::cast_precision_loss)]
        let size = [image.width() as f32, image.height() as f32];
        let paint = PixmapPaint {
            quality: FilterQuality::Bicubic,
            ..PixmapPaint::default()
        };
        pixmap.draw_pixmap(0, 0, image.as_ref(), &paint, fit(rect, size, cover), None);
        Ok(())
    }
}

/// The transform that scales a size into a rectangle: to fit inside and
/// centered, or to cover it.
fn fit(rect: Rect, [width, height]: [f32; 2], cover: bool) -> Transform {
    #[allow(clippy::cast_precision_loss)]
    let [x, y, room_x, room_y] = rect.map(|n| n as f32);
    let (across, down) = (room_x / width, room_y / height);
    let scale = if cover {
        across.max(down)
    } else {
        across.min(down)
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
    fn the_start_button_is_green() {
        let (mut theme, cache) = theme("start");
        let picture =
            Picture::new(100, 30).element(Sheet::Winxp, "startbutton-normal", [0, 0, 100, 30]);
        let image = Pixmap::load_png(theme.render(&picture)).expect("a PNG file");
        let pixel = image.pixel(50, 15).expect("inside");
        assert!(pixel.green() > pixel.red() && pixel.green() > pixel.blue());
        let _ = fs::remove_dir_all(cache);
    }
}
