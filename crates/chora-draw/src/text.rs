//! Text through usvg and fontdb: each line of text becomes a small SVG
//! document with a `<text>` element, which resvg renders with a font of
//! the asset folder or a system font. The base family is a value of the
//! asset folder; `sans-serif` names the sans-serif family of the system.
//! XP's own fonts in `fonts/WindowsXP` of the XDG data directories come
//! first, when they are installed.

use std::{fmt::Write as _, path::Path, sync::Arc};

use chora_kit::xdg::data_dirs;
use resvg::{
    tiny_skia::{PixmapMut, Transform},
    usvg::{
        self,
        fontdb::{Database, Family, Query, Style, Weight},
    },
};

use crate::theme::Rect;

/// The family name that stands for the sans-serif family of the system.
pub const SANS_SERIF: &str = "sans-serif";

/// How many font files of a HUD folder load.
pub const MAX_FOLDER_FONTS: usize = 16;

/// How many entries of the `fonts` directory of a HUD folder are listed.
const MAX_LISTED: usize = 256;

/// The sorted names of the first [`MAX_LISTED`] entries of the `fonts`
/// directory of a HUD folder. A `fonts` that is a link lists nothing.
fn font_names(dir: &Path) -> Vec<String> {
    let fonts = dir.join("fonts");
    if !std::fs::symlink_metadata(&fonts).is_ok_and(|meta| meta.is_dir()) {
        return Vec::new();
    }
    let entries = std::fs::read_dir(fonts).into_iter().flatten();
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .take(MAX_LISTED)
        .collect();
    names.sort();
    names
}

/// The system fonts and XP's fonts.
fn host_database() -> Database {
    let mut db = Database::new();
    for data in data_dirs() {
        db.load_fonts_dir(data.join("fonts/WindowsXP"));
    }
    db.load_system_fonts();
    db
}

/// The fonts of the theme.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Font {
    /// Menus, task buttons and the clock.
    Ui,
    /// Bold menu entries.
    UiBold,
    /// Window titles.
    Title,
    /// The label of the start button.
    Start,
    /// The user name in the header of the start menu.
    Name,
    /// The title of the Log Off and Turn Off dialogs.
    Dialog,
    /// Text that a person must read: [`BODY_SIZE`] pixels.
    Body,
    /// Bold text that a person must read: [`BODY_SIZE`] pixels.
    BodyBold,
}

/// The size of [`Font::Body`] and [`Font::BodyBold`] in pixels; the
/// smallest size of text that a person must read.
pub const BODY_SIZE: u32 = 14;

/// The size of the label of the start button in pixels.
pub const START_SIZE: u32 = 21;

/// Where a line of text sits in its rectangle.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Align {
    /// At the left edge.
    Left,
    /// In the middle.
    Center,
}

/// A line of text in a rectangle, vertically centered. Text that is too
/// wide ends with an ellipsis.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Text {
    /// The characters.
    pub line: String,
    /// The font.
    pub font: Font,
    /// RGBA.
    pub color: [u8; 4],
    /// The color of a shadow one pixel to the bottom right.
    pub shadow: Option<[u8; 4]>,
    /// The rectangle.
    pub rect: Rect,
    /// The horizontal alignment.
    pub align: Align,
}

impl Text {
    /// Left-aligned text without a shadow.
    #[must_use]
    pub fn new(text: &str, font: Font, color: [u8; 4], rect: Rect) -> Self {
        Self {
            line: text.into(),
            font,
            color,
            shadow: None,
            rect,
            align: Align::Left,
        }
    }

    /// The same text with a shadow.
    #[must_use]
    pub fn shadow(mut self, color: [u8; 4]) -> Self {
        self.shadow = Some(color);
        self
    }

    /// The same text, centered.
    #[must_use]
    pub fn centered(mut self) -> Self {
        self.align = Align::Center;
        self
    }
}

/// The look of one font.
struct Spec {
    family: String,
    size: u32,
    bold: bool,
    /// The family has no bold face, so a thin stroke thickens the glyphs.
    faux_bold: bool,
    slant: Slant,
}

/// How a font slants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Slant {
    Upright,
    /// The italic face of the family.
    Italic,
    /// The family has no italic face, so a shear slants the glyphs.
    Sheared,
}

impl Spec {
    fn new(db: &Database, family: &str, size: u32, bold: bool, italic: bool) -> Self {
        let slant = match (italic, has_italic(db, family)) {
            (false, _) => Slant::Upright,
            (true, true) => Slant::Italic,
            (true, false) => Slant::Sheared,
        };
        Self {
            family: family.into(),
            size,
            bold,
            faux_bold: bold && !has_bold(db, family),
            slant,
        }
    }
}

/// The font database and the look of each font.
pub struct Fonts {
    options: usvg::Options<'static>,
    specs: [Spec; 8],
    host_only: bool,
}

impl Fonts {
    /// Loads the system fonts and the font files in `fonts/` of the asset
    /// directory. `family` is the base family of the text. The asset
    /// directory is trusted: use [`Fonts::folder`] for a HUD folder.
    #[must_use]
    pub fn load(dir: &Path, family: &str) -> Self {
        let mut db = host_database();
        let files = std::fs::read_dir(dir.join("fonts")).into_iter().flatten();
        for file in files.flatten().map(|entry| entry.path()) {
            if let Err(err) = db.load_font_file(&file) {
                eprintln!("chora-draw: font {}: {err}", file.display());
            }
        }
        let mut fonts = Self::with(db, family);
        fonts.host_only = false;
        fonts
    }

    /// The fonts of the host only: the system fonts and XP's fonts. The
    /// question card is drawn with these and never with a font of a HUD
    /// folder, which [`Fonts::folder`] keeps in a database of its own.
    #[must_use]
    pub fn host(family: &str) -> Self {
        Self::with(host_database(), family)
    }

    /// The fonts for the text of a HUD folder: the host fonts and the
    /// font files in `fonts/` of the folder, read through
    /// [`chora_kit::hud::file`], so that no link leaves the folder and
    /// each file stays within its size limit. At most [`MAX_FOLDER_FONTS`]
    /// of the first [`MAX_LISTED`] entries load; a `fonts` link loads none. The database is separate from [`Fonts::host`]. Pictures
    /// that the text document links resolve only inside the folder.
    #[must_use]
    pub fn folder(dir: &Path, family: &str) -> Self {
        let mut db = host_database();
        for name in font_names(dir).iter().take(MAX_FOLDER_FONTS) {
            match chora_kit::hud::file(dir, &format!("fonts/{name}")) {
                Ok(data) => db.load_font_data(data),
                Err(err) => eprintln!("chora-draw: font {name}: {err}"),
            }
        }
        let mut fonts = Self::with(db, family);
        fonts.host_only = false;
        fonts.options.resources_dir = Some(dir.to_path_buf());
        fonts.options.image_href_resolver = crate::assets::href_resolver(dir);
        fonts
    }

    /// The looks of the fonts from a font database.
    fn with(mut db: Database, base: &str) -> Self {
        if base != SANS_SERIF {
            db.set_sans_serif_family(base);
        }
        let title = first_family(&db, &["Trebuchet MS", base]);
        let start = first_family(&db, &["Franklin Gothic Medium", "Libre Franklin", base]);
        let spec = |family: &str, size, bold, italic| Spec::new(&db, family, size, bold, italic);
        let specs = [
            spec(base, 11, false, false),
            spec(base, 11, true, false),
            spec(&title, 13, true, false),
            spec(&start, 17, true, true),
            spec(base, 16, true, false),
            spec(base, 19, false, false),
            spec(base, BODY_SIZE, false, false),
            spec(base, BODY_SIZE, true, false),
        ];
        let options = usvg::Options {
            fontdb: Arc::new(db),
            font_family: base.into(),
            ..usvg::Options::default()
        };
        Self {
            options,
            specs,
            host_only: true,
        }
    }

    /// Sets the chrome fonts to XP's Large Fonts: the menu and button
    /// fonts and the title at [`BODY_SIZE`] pixels, the label of the start
    /// button at [`START_SIZE`].
    pub fn large_chrome(&mut self) {
        for font in [Font::Ui, Font::UiBold, Font::Title] {
            self.specs[font as usize].size = BODY_SIZE;
        }
        self.specs[Font::Start as usize].size = START_SIZE;
    }

    /// Whether the fonts are those of the host only, with no font file of
    /// an asset or HUD folder.
    #[must_use]
    pub fn is_host(&self) -> bool {
        self.host_only
    }

    /// The size of a font in pixels.
    #[must_use]
    pub fn size(&self, font: Font) -> u32 {
        self.spec(font).size
    }

    /// The options for parsing SVG files, with the font database.
    #[must_use]
    pub fn options(&self) -> &usvg::Options<'static> {
        &self.options
    }

    fn spec(&self, font: Font) -> &Spec {
        &self.specs[font as usize]
    }

    /// Draws a line of text onto a pixmap, in the coordinates that `base`
    /// scales to pixels.
    ///
    /// # Errors
    ///
    /// The text does not parse as SVG.
    pub fn draw(
        &self,
        text: &Text,
        base: Transform,
        pixmap: &mut PixmapMut<'_>,
    ) -> Result<(), String> {
        let [x, _, w, _] = text.rect;
        let line = self.fit(&text.line, text.font, w);
        let x = match text.align {
            Align::Left => f64::from(x),
            Align::Center => {
                let width = self.width(&line, text.font).unwrap_or(0.0);
                f64::from(x) + (f64::from(w) - width).max(0.0) / 2.0
            }
        };
        let mut body = String::new();
        if let Some(shadow) = text.shadow {
            body += &self.element(&line, text, [x + 1.0, 1.0], shadow);
        }
        body += &self.element(&line, text, [x, 0.0], text.color);
        let tree = self.parse(&body, pixmap.width(), pixmap.height())?;
        resvg::render(&tree, base, pixmap);
        Ok(())
    }

    /// The text, shortened with an ellipsis to fit a width.
    fn fit(&self, text: &str, font: Font, width: i32) -> String {
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        let fits = |s: &str| self.width(s, font).is_none_or(|w| w <= f64::from(width));
        if fits(&clean) {
            return clean;
        }
        let chars: Vec<char> = clean.chars().collect();
        let (mut low, mut high) = (0, chars.len());
        while low < high {
            let mid = (low + high).div_ceil(2);
            let candidate: String = chars[..mid].iter().chain(['…'].iter()).collect();
            if fits(&candidate) {
                low = mid;
            } else {
                high = mid - 1;
            }
        }
        let prefix: String = chars[..low].iter().collect();
        format!("{}…", prefix.trim_end())
    }

    /// The width of a line from its start to the right edge of its ink.
    #[must_use]
    pub fn width(&self, line: &str, font: Font) -> Option<f64> {
        let probe = Text::new(line, font, [0, 0, 0, 255], [0, 0, 4096, 64]);
        let tree = self
            .parse(
                &self.element(line, &probe, [0.0, 0.0], probe.color),
                4096,
                64,
            )
            .ok()?;
        let bbox = tree.root().abs_bounding_box();
        (bbox.width() > 0.0).then(|| f64::from(bbox.right()))
    }

    /// The advance of the SVG elements `body` from x 0: the right edge of
    /// their ink, with the fonts of the theme.
    #[must_use]
    pub fn svg_width(&self, body: &str) -> Option<f64> {
        let tree = self.parse(body, 4096, 256).ok()?;
        let bbox = tree.root().abs_bounding_box();
        (bbox.width() > 0.0).then(|| f64::from(bbox.right()))
    }

    /// The `<text>` element of a line, at `x` and moved down by `dy`. A
    /// window title draws without smoothing and with a thin stroke for its
    /// weight, as XP does, but none on the real Trebuchet MS Bold; a family
    /// without a bold face takes a thicker stroke. A family without an italic face is sheared.
    fn element(&self, line: &str, text: &Text, [x, dy]: [f64; 2], color: [u8; 4]) -> String {
        let spec = self.spec(text.font);
        let [_, y, _, h] = text.rect;
        let size = f64::from(spec.size);
        let baseline = f64::from(y) + f64::from(h) / 2.0 + size * 0.36 + dy;
        let fill = format!("rgb({},{},{})", color[0], color[1], color[2]);
        let alpha = f64::from(color[3]) / 255.0;
        let mut out = String::new();
        if spec.slant == Slant::Sheared {
            let _ = write!(
                out,
                r#"<g transform="translate({:.2} 0) skewX(-8)">"#,
                baseline * 0.1405
            );
        }
        let _ = write!(
            out,
            r#"<text x="{x:.2}" y="{baseline:.2}" font-family="{}" font-size="{size}" font-weight="{}" font-style="{}" fill="{fill}" fill-opacity="{alpha:.3}" xml:space="preserve""#,
            escape(&spec.family),
            if spec.bold { "bold" } else { "normal" },
            if spec.slant == Slant::Italic {
                "italic"
            } else {
                "normal"
            },
        );
        let title = text.font == Font::Title;
        if title {
            out += r#" text-rendering="optimizeSpeed""#;
        }
        let stroke = match (spec.faux_bold, title) {
            (true, _) => Some(0.6),
            (false, true) => (spec.family != "Trebuchet MS").then_some(0.3),
            (false, false) => None,
        };
        if let Some(width) = stroke {
            let _ = write!(
                out,
                r#" stroke="{fill}" stroke-opacity="{alpha:.3}" stroke-width="{width}" stroke-linejoin="round""#
            );
        }
        let _ = write!(out, ">{}</text>", escape(line));
        if spec.slant == Slant::Sheared {
            out += "</g>";
        }
        out
    }

    fn parse(&self, body: &str, width: u32, height: u32) -> Result<usvg::Tree, String> {
        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}">{body}</svg>"#
        );
        usvg::Tree::from_str(&svg, &self.options).map_err(|e| format!("text: {e}"))
    }
}

/// Breaks text into lines no wider than `max`, at spaces where it can, and
/// at its line breaks. A word wider than a line breaks between its
/// characters.
pub fn wrap(text: &str, max: f64, width: impl Fn(&str) -> f64) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.lines() {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if line.is_empty() {
                word.to_string()
            } else {
                format!("{line} {word}")
            };
            if width(&candidate) <= max {
                line = candidate;
                continue;
            }
            if !line.is_empty() {
                lines.push(std::mem::take(&mut line));
            }
            for c in word.chars() {
                line.push(c);
                if width(&line) > max && line.chars().count() > 1 {
                    line.pop();
                    lines.push(std::mem::replace(&mut line, c.to_string()));
                }
            }
        }
        lines.push(line);
    }
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    lines
}

/// The family of a name; `sans-serif` is the generic family.
fn family(name: &str) -> Family<'_> {
    if name == SANS_SERIF {
        Family::SansSerif
    } else {
        Family::Name(name)
    }
}

/// The first family of a list that the database has, or the last one.
fn first_family(db: &Database, families: &[&str]) -> String {
    families
        .iter()
        .find(|f| {
            db.query(&Query {
                families: &[family(f)],
                ..Query::default()
            })
            .is_some()
        })
        .or(families.last())
        .map_or_else(String::new, |f| (*f).to_string())
}

/// Whether a family has a bold face.
fn has_bold(db: &Database, name: &str) -> bool {
    db.query(&Query {
        families: &[family(name)],
        weight: Weight::BOLD,
        ..Query::default()
    })
    .and_then(|id| db.face(id))
    .is_some_and(|face| face.weight.0 >= 600)
}

/// Whether a family has an italic face.
fn has_italic(db: &Database, name: &str) -> bool {
    db.query(&Query {
        families: &[family(name)],
        style: Style::Italic,
        ..Query::default()
    })
    .and_then(|id| db.face(id))
    .is_some_and(|face| face.style != Style::Normal)
}

/// Escapes text for XML.
/// The text with the characters that XML reserves escaped.
#[must_use]
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_faces_are_faked() {
        let mut db = Database::new();
        db.load_fonts_dir(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../flavors/xp/theme/fonts"
        ));
        let bold = Spec::new(&db, "Tahoma", 11, true, false);
        assert!(bold.faux_bold && bold.slant == Slant::Upright);
        let start = Spec::new(&db, "Libre Franklin", 17, true, true);
        assert!(!start.faux_bold && start.slant == Slant::Sheared);
        assert_eq!(
            first_family(&db, &["Franklin Gothic Medium", "Libre Franklin"]),
            "Libre Franklin"
        );
    }

    #[test]
    fn a_title_renders_without_a_bold_face() {
        let mut db = Database::new();
        db.load_fonts_dir(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../flavors/xp/theme/fonts"
        ));
        let fonts = Fonts::with(db, "Tahoma");
        assert!(fonts.spec(Font::Title).faux_bold);
        let mut pixmap = resvg::tiny_skia::Pixmap::new(120, 20).expect("a pixmap");
        let title = Text::new(
            "My Computer",
            Font::Title,
            [255, 255, 255, 255],
            [0, 0, 120, 20],
        );
        fonts
            .draw(&title, Transform::identity(), &mut pixmap.as_mut())
            .expect("the title parses");
        assert!(pixmap.pixels().iter().any(|p| p.alpha() > 0));
    }

    #[test]
    fn the_title_is_the_real_trebuchet_bold_when_the_import_has_it() {
        let Some(assets) = std::env::var_os("CHORA_XP_ASSETS") else {
            return;
        };
        let mut db = Database::new();
        db.load_fonts_dir(std::path::Path::new(&assets).join("fonts/WindowsXP"));
        let fonts = Fonts::with(db, "Tahoma");
        let title = fonts.spec(Font::Title);
        assert_eq!(title.family, "Trebuchet MS");
        assert!(!title.faux_bold, "the bold face must load, not be faked");
        let line = Text::new("foot", Font::Title, [255, 255, 255, 255], [0, 0, 80, 20]);
        let svg = fonts.element("foot", &line, [0.0, 0.0], line.color);
        assert!(
            !svg.contains("stroke"),
            "a real bold face takes no stroke: {svg}"
        );
        let width = fonts.width("foot", Font::Title).expect("a width");
        assert!(
            (23.0..=25.5).contains(&width),
            "foot is {width} px wide, trebucbd.ttf at 13 px makes 24.7"
        );
    }

    /// Six pixels per character.
    #[allow(clippy::cast_precision_loss)]
    fn mono(s: &str) -> f64 {
        s.chars().count() as f64 * 6.0
    }

    #[test]
    fn text_wraps_at_spaces_and_breaks() {
        assert_eq!(wrap("aaa bbb ccc", 42.0, mono), ["aaa bbb", "ccc"]);
        assert_eq!(wrap("one\n\ntwo", 300.0, mono), ["one", "", "two"]);
        assert_eq!(wrap("abcdefghij", 24.0, mono), ["abcd", "efgh", "ij"]);
        assert_eq!(wrap("", 300.0, mono).len(), 0);
    }

    #[test]
    fn markup_in_titles_stays_text() {
        assert_eq!(
            escape(r#"<a href="x">&'"#),
            "&lt;a href=&quot;x&quot;&gt;&amp;&apos;"
        );
    }

    fn font_folder(test: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("chora-fonts-{test}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("fonts")).expect("a folder");
        dir
    }

    fn families(db: &Database) -> Vec<String> {
        db.faces()
            .flat_map(|face| face.families.iter().map(|(name, _)| name.clone()))
            .collect()
    }

    fn theme_fonts() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flavors/xp/theme/fonts")
    }

    fn renamed_font(file: &str, from: &str, to: &str, target: &Path) {
        assert_eq!(from.len(), to.len());
        let utf16 =
            |text: &str| -> Vec<u8> { text.encode_utf16().flat_map(u16::to_be_bytes).collect() };
        let mut data = std::fs::read(theme_fonts().join(file)).unwrap();
        for (old, new) in [
            (from.as_bytes().to_vec(), to.as_bytes().to_vec()),
            (utf16(from), utf16(to)),
        ] {
            let mut at = 0;
            while at + old.len() <= data.len() {
                if data[at..at + old.len()] == old[..] {
                    data[at..at + old.len()].copy_from_slice(&new);
                    at += old.len();
                } else {
                    at += 1;
                }
            }
        }
        std::fs::write(target, data).unwrap();
    }

    fn has_family(db: &Database, family: &str) -> bool {
        families(db).iter().any(|name| name == family)
    }

    #[test]
    fn fonts_load_only_from_the_folder() {
        let dir = font_folder("only");
        let outside = font_folder("outside");
        renamed_font("tahoma.ttf", "Tahoma", "Qxzvkw", &dir.join("fonts/a.ttf"));
        renamed_font(
            "LibreFranklin-Bold.ttf",
            "Franklin",
            "Zqwvxkjh",
            &outside.join("fonts/f.ttf"),
        );
        std::os::unix::fs::symlink(outside.join("fonts/f.ttf"), dir.join("fonts/link.ttf"))
            .unwrap();
        std::fs::write(dir.join("fonts/notes.txt"), "not a font").unwrap();
        let fonts = Fonts::folder(&dir, SANS_SERIF);
        let db = &fonts.options().fontdb;
        let face = db
            .faces()
            .find(|face| face.families.iter().any(|(name, _)| name == "Qxzvkw"))
            .expect("the font of the folder loads");
        assert!(matches!(face.source, usvg::fontdb::Source::Binary(_)));
        assert!(!has_family(db, "Zqwvxkjh"), "{:?}", families(db));
        let _ = std::fs::remove_dir_all(dir);
        let _ = std::fs::remove_dir_all(outside);
    }

    #[test]
    fn large_chrome_makes_all_chrome_text_14_px_but_the_start_label_and_the_names() {
        let mut host = Fonts::host(SANS_SERIF);
        assert_eq!(host.size(Font::Ui), 11);
        host.large_chrome();
        for font in [
            Font::Ui,
            Font::UiBold,
            Font::Title,
            Font::Body,
            Font::BodyBold,
        ] {
            assert_eq!(host.size(font), 14, "{font:?}");
        }
        assert_eq!(host.size(Font::Start), 21);
        assert!(host.size(Font::Name) >= 14 && host.size(Font::Dialog) >= 14);
    }

    #[test]
    fn body_text_is_at_least_14_px_and_only_the_host_fonts_are_host() {
        let host = Fonts::host(SANS_SERIF);
        assert!(host.is_host());
        assert!(host.size(Font::Body) >= 14 && host.size(Font::BodyBold) >= 14);
        let dir = font_folder("kind");
        assert!(!Fonts::folder(&dir, SANS_SERIF).is_host());
        assert!(!Fonts::load(&dir, SANS_SERIF).is_host());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_folder_font_is_not_in_the_host_fontdb() {
        let dir = font_folder("host");
        renamed_font("tahoma.ttf", "Tahoma", "Qxzvkw", &dir.join("fonts/a.ttf"));
        let folder = Fonts::folder(&dir, SANS_SERIF);
        let host = Fonts::host(SANS_SERIF);
        assert!(has_family(&folder.options().fontdb, "Qxzvkw"));
        assert!(!has_family(&host.options().fontdb, "Qxzvkw"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_fonts_directory_that_is_a_link_is_refused() {
        let dir = font_folder("linked");
        let outside = font_folder("linked-outside");
        renamed_font(
            "tahoma.ttf",
            "Tahoma",
            "Qxzvkw",
            &outside.join("fonts/a.ttf"),
        );
        std::fs::remove_dir_all(dir.join("fonts")).unwrap();
        std::os::unix::fs::symlink(outside.join("fonts"), dir.join("fonts")).unwrap();
        let fonts = Fonts::folder(&dir, SANS_SERIF);
        assert!(!has_family(&fonts.options().fontdb, "Qxzvkw"));
        let _ = std::fs::remove_dir_all(dir);
        let _ = std::fs::remove_dir_all(outside);
    }

    #[test]
    fn at_most_256_entries_of_the_fonts_directory_are_listed() {
        let dir = font_folder("many");
        for n in 0..300 {
            std::fs::write(dir.join(format!("fonts/{n:03}.txt")), "").unwrap();
        }
        assert_eq!(font_names(&dir).len(), MAX_LISTED);
        let _ = std::fs::remove_dir_all(dir);
    }
}
