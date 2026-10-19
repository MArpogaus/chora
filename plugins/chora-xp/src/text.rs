//! Text through usvg and fontdb: each line of text becomes a small SVG
//! document with a `<text>` element, which resvg renders with the bundled
//! font or a system font. XP's own fonts in `fonts/WindowsXP` of the XDG
//! data directories come first, when they are installed.

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
}

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
    pub fn shadow(mut self, color: [u8; 4]) -> Self {
        self.shadow = Some(color);
        self
    }

    /// The same text, centered.
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
    specs: [Spec; 6],
}

impl Fonts {
    /// Loads the system fonts and the fonts of the theme directory.
    pub fn load(dir: &Path) -> Self {
        let mut db = Database::new();
        // Of equal faces, a query takes the first one, so XP's own Tahoma
        // goes before the Tahoma of newer Windows and the Wine Tahoma of
        // the theme.
        for data in data_dirs() {
            db.load_fonts_dir(data.join("fonts/WindowsXP"));
        }
        db.load_system_fonts();
        for file in ["fonts/tahoma.ttf", "fonts/LibreFranklin-Bold.ttf"] {
            if let Err(err) = db.load_font_file(dir.join(file)) {
                eprintln!("chora-xp: font {file}: {err}");
            }
        }
        Self::with(db)
    }

    /// The looks of the fonts from a font database.
    fn with(mut db: Database) -> Self {
        db.set_sans_serif_family("Tahoma");
        let title = first_family(&db, &["Trebuchet MS", "Tahoma"]);
        let start = first_family(&db, &["Franklin Gothic Medium", "Libre Franklin", "Tahoma"]);
        // XP draws "start" in Franklin Gothic Medium Italic, made bold.
        let spec = |family: &str, size, bold, italic| Spec::new(&db, family, size, bold, italic);
        let specs = [
            spec("Tahoma", 11, false, false),
            spec("Tahoma", 11, true, false),
            spec(&title, 13, true, false),
            spec(&start, 17, true, true),
            spec("Tahoma", 16, true, false),
            spec("Tahoma", 19, false, false),
        ];
        let options = usvg::Options {
            fontdb: Arc::new(db),
            font_family: "Tahoma".into(),
            ..usvg::Options::default()
        };
        Self { options, specs }
    }

    /// The options for parsing SVG files, with the font database.
    pub fn options(&self) -> &usvg::Options<'static> {
        &self.options
    }

    fn spec(&self, font: Font) -> &Spec {
        &self.specs[font as usize]
    }

    /// Draws a line of text onto a pixmap, in the coordinates that `base`
    /// scales to pixels.
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
        // The longest prefix that fits with the ellipsis, by bisection.
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

    /// The `<text>` element of a line, at `x` and moved down by `dy`.
    fn element(&self, line: &str, text: &Text, [x, dy]: [f64; 2], color: [u8; 4]) -> String {
        let spec = self.spec(text.font);
        let [_, y, _, h] = text.rect;
        let size = f64::from(spec.size);
        // The baseline that centers the capitals in the rectangle.
        let baseline = f64::from(y) + f64::from(h) / 2.0 + size * 0.36 + dy;
        let fill = format!("rgb({},{},{})", color[0], color[1], color[2]);
        let alpha = f64::from(color[3]) / 255.0;
        let mut out = String::new();
        if spec.slant == Slant::Sheared {
            // skewX(-8) shears around the origin; the shift keeps the
            // baseline in place.
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
        // XP draws window titles without smoothing, with stems of two
        // pixels: no anti-aliasing, and a thin stroke for the weight. A
        // family without a bold face takes a thicker stroke; one stroke
        // in all, as SVG allows one attribute of a name.
        let title = text.font == Font::Title;
        if title {
            out += r#" text-rendering="optimizeSpeed""#;
        }
        let stroke = match (spec.faux_bold, title) {
            (true, _) => Some(0.6),
            (false, true) => Some(0.3),
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

/// The first family of a list that the database has, or the last one.
fn first_family(db: &Database, families: &[&str]) -> String {
    families
        .iter()
        .find(|f| {
            db.query(&Query {
                families: &[Family::Name(f)],
                ..Query::default()
            })
            .is_some()
        })
        .or(families.last())
        .map_or_else(String::new, |f| (*f).to_string())
}

/// Whether a family has a bold face.
fn has_bold(db: &Database, family: &str) -> bool {
    db.query(&Query {
        families: &[Family::Name(family)],
        weight: Weight::BOLD,
        ..Query::default()
    })
    .and_then(|id| db.face(id))
    .is_some_and(|face| face.weight.0 >= 600)
}

/// Whether a family has an italic face.
fn has_italic(db: &Database, family: &str) -> bool {
    db.query(&Query {
        families: &[Family::Name(family)],
        style: Style::Italic,
        ..Query::default()
    })
    .and_then(|id| db.face(id))
    .is_some_and(|face| face.style != Style::Normal)
}

/// Escapes text for XML.
fn escape(text: &str) -> String {
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
        // The Wine Tahoma has no bold face; Libre Franklin has no italic.
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
        // Only the Wine Tahoma of the theme, as without Trebuchet MS.
        let mut db = Database::new();
        db.load_fonts_dir(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../flavors/xp/theme/fonts"
        ));
        let fonts = Fonts::with(db);
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
    fn markup_in_titles_stays_text() {
        assert_eq!(
            escape(r#"<a href="x">&'"#),
            "&lt;a href=&quot;x&quot;&gt;&amp;&apos;"
        );
    }
}
