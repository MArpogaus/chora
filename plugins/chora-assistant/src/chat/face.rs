//! The face of the reading text of a plain chat: the family and the size
//! that the `chat:` block of a folder names, drawn as SVG text with the
//! fonts of the host and a serif behind it, since the host does not read
//! fontconfig and a family such as TeX Gyre Pagella may be missing.

use chora_draw::{text::Font, theme::Theme};
use chora_kit::hud::{Chat as Look, ChatForm};

/// What stands behind the family of the folder, so that a missing family
/// still shows in a serif.
const BEHIND: &str = "DejaVu Serif, Liberation Serif, serif";

/// What stands behind the family of an XP chat, which is a sans.
const BEHIND_SANS: &str = "DejaVu Sans, Liberation Sans, sans-serif";

/// The distance of two lines of the text, in tenths of its size.
const LEADING: u32 = 15;

/// A family and a size.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Face {
    family: String,
    size: u32,
    xp: bool,
}

impl Face {
    /// The face of a look that names a font; none for a look that does not,
    /// and for a name with a character that a family does not have.
    #[must_use]
    pub fn of(look: &Look) -> Option<Self> {
        let family = look.font.as_deref()?.trim();
        let plain = |c: char| c.is_alphanumeric() || " _-".contains(c);
        (!family.is_empty() && family.chars().all(plain)).then(|| Self {
            family: family.to_owned(),
            size: look.size,
            xp: look.form == ChatForm::Xp,
        })
    }

    /// A face of a serif of the host at a size.
    #[must_use]
    pub fn serif(size: u32) -> Self {
        Self {
            family: "serif".to_owned(),
            size,
            xp: false,
        }
    }

    /// The height of a line of the text.
    #[must_use]
    pub fn line(&self) -> i32 {
        i32::try_from((self.size * LEADING).div_ceil(10)).unwrap_or(i32::MAX)
    }

    /// Whether the face is that of the XP form of the balloon.
    #[must_use]
    pub fn is_xp(&self) -> bool {
        self.xp
    }

    /// The SVG element of a line of text whose row starts at `y`, `x` from
    /// the left, in a colour.
    #[must_use]
    pub fn element(&self, text: &str, at: [i32; 2], rgb: [u8; 3]) -> String {
        self.weighted(text, at, rgb, false)
    }

    /// The same line in bold.
    #[must_use]
    pub fn bold_element(&self, text: &str, at: [i32; 2], rgb: [u8; 3]) -> String {
        self.weighted(text, at, rgb, true)
    }

    fn weighted(
        &self,
        text: &str,
        [x, y]: [i32; 2],
        [red, green, blue]: [u8; 3],
        bold: bool,
    ) -> String {
        let baseline = f64::from(y) + f64::from(self.line()) / 2.0 + f64::from(self.size) * 0.34;
        let behind = if self.xp { BEHIND_SANS } else { BEHIND };
        let weight = if bold { r#" font-weight="bold""# } else { "" };
        format!(
            r#"<text x="{x}" y="{baseline:.2}" font-family="{}, {behind}"{weight} font-size="{}" fill="rgb({red},{green},{blue})" xml:space="preserve">{}</text>"#,
            self.family,
            self.size,
            escape(text)
        )
    }

    /// The advance of a character: the width of a string with it between
    /// two letters, less that of the letters alone, which also gives a
    /// space its width.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, reason = "a width of a few pixels")]
    pub fn advance(&self, theme: &Theme, c: char) -> f32 {
        let width = |text: &str| theme.svg_width(&self.element(text, [0, 0], [0, 0, 0]));
        (width(&format!("x{c}x")) - width("xx")) as f32
    }
}

/// The advance of a character in the sans of the host at the size of
/// reading text.
#[must_use]
#[allow(clippy::cast_possible_truncation, reason = "a width of a few pixels")]
pub fn sans_advance(theme: &Theme, c: char) -> f32 {
    let width = |text: &str| theme.text_width(text, Font::Body);
    (width(&format!("x{c}x")) - width("xx")) as f32
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn look(font: Option<&str>, size: u32) -> Look {
        Look {
            font: font.map(str::to_owned),
            size,
            ..Look::default()
        }
    }

    #[test]
    fn a_look_without_a_font_has_no_face_and_a_odd_name_is_refused() {
        assert_eq!(Face::of(&look(None, 18)), None);
        assert_eq!(Face::of(&look(Some("a\"b"), 18)), None);
        assert_eq!(Face::of(&look(Some("x, <b>"), 18)), None);
        let face = Face::of(&look(Some("TeX Gyre Pagella"), 18)).unwrap();
        assert_eq!(face.line(), 27);
    }

    #[test]
    fn the_line_is_text_that_a_person_can_not_turn_into_markup() {
        let face = Face::of(&look(Some("TeX Gyre Pagella"), 18)).unwrap();
        let element = face.element("a <b> & \"c\"", [14, 40], [1, 2, 3]);
        assert!(
            element.contains("a &lt;b&gt; &amp; &quot;c&quot;"),
            "{element}"
        );
        assert!(element.contains("font-size=\"18\""), "{element}");
        assert!(
            element.contains("TeX Gyre Pagella, DejaVu Serif"),
            "{element}"
        );
    }

    #[test]
    fn a_space_has_a_width() {
        let (theme, _, cache) = crate::tests::loaded("face-space");
        let face = Face::of(&look(Some("DejaVu Serif"), 18)).unwrap();
        assert!(
            face.advance(&theme, ' ') > 3.0,
            "{}",
            face.advance(&theme, ' ')
        );
        assert!(sans_advance(&theme, ' ') > 3.0);
        assert!(face.advance(&theme, 'm') > face.advance(&theme, 'i'));
        let _ = std::fs::remove_dir_all(cache);
    }
}
