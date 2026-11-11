//! The look of the field: the layout of the draft in whole lines and its
//! pixels, drawn with `chora-draw`. The text is never smaller than
//! [`chora_draw::text::BODY_SIZE`].

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    reason = "the sizes of the pixels of a text field are small"
)]

use std::{cell::RefCell, collections::HashMap};

use chora_draw::text::{Font, Fonts, SANS_SERIF, Text};
use chora_kit::chat::wrap;
use resvg::tiny_skia::{Color, Paint, Pixmap, Rect, Transform};
use serde::Deserialize;

use crate::session::Session;

/// The space around the text.
const PAD: u32 = 8;
/// The height of a line.
const LINE: u32 = 20;
/// The most lines of the draft that show.
const MAX_LINES: usize = 4;
/// The most lines of the status under the draft.
const MAX_STATUS: usize = 3;
/// The width of a character that the font does not measure.
const FALLBACK_WIDTH: f32 = 8.0;
/// The width of the caret.
const CARET: u32 = 2;

/// The colors of the field, RGBA. A look gives values and nothing else.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Look {
    /// The ground.
    pub background: [u8; 4],
    /// The text and the caret.
    pub text: [u8; 4],
    /// The line around the field.
    pub border: [u8; 4],
    /// The hint, the status and the empty field.
    pub hint: [u8; 4],
}

impl Default for Look {
    fn default() -> Self {
        Self {
            background: [30, 32, 38, 255],
            text: [236, 238, 242, 255],
            border: [96, 104, 120, 255],
            hint: [160, 168, 182, 255],
        }
    }
}

/// The draft in lines, and where the caret sits.
pub struct Layout {
    /// The lines that show.
    lines: Vec<String>,
    /// Whether the lines are the placeholder of an empty field.
    placeholder: bool,
    /// The line of the caret among the lines that show, and its x.
    caret: Option<(usize, f32)>,
    /// The lines of the status.
    status: Vec<String>,
}

impl Layout {
    /// The left edge of the lines of the draft: after the caret for the
    /// placeholder, which the caret stands before, else at the pad.
    #[must_use]
    pub fn text_left(&self) -> u32 {
        if self.placeholder {
            PAD + 2 * CARET
        } else {
            PAD
        }
    }

    /// The height of the field in pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        let rows = u32::try_from(self.lines.len() + self.status.len()).unwrap_or(1);
        2 * PAD + rows.max(1) * LINE
    }
}

/// Draws the field.
pub struct Painter {
    fonts: Fonts,
    widths: RefCell<HashMap<char, f32>>,
}

impl Painter {
    /// A painter with the fonts of the host.
    #[must_use]
    pub fn new() -> Self {
        Self {
            fonts: Fonts::host(SANS_SERIF),
            widths: RefCell::new(HashMap::new()),
        }
    }

    /// The advance of a character.
    fn advance(&self, c: char) -> f32 {
        if let Some(&width) = self.widths.borrow().get(&c) {
            return width;
        }
        let measure = |line: &str| self.fonts.width(line, Font::Body);
        let width = match (measure(&format!("|{c}|")), measure("||")) {
            (Some(with), Some(without)) => (with - without) as f32,
            _ => FALLBACK_WIDTH,
        };
        self.widths.borrow_mut().insert(c, width);
        width
    }

    fn width_of(&self, text: &str) -> f32 {
        text.chars().map(|c| self.advance(c)).sum()
    }

    /// The layout of a draft in a field of `width` pixels.
    #[must_use]
    pub fn layout(&self, session: &Session, width: u32) -> Layout {
        let inner = width.saturating_sub(2 * PAD) as f32;
        let before = session.before_cursor();
        let whole = format!("{before}{}", session.after_cursor());
        let lines = wrap(&whole, inner, |c| self.advance(c));
        let caret_lines = wrap(&before, inner, |c| self.advance(c));
        let caret_line = caret_lines.len().saturating_sub(1).min(lines.len() - 1);
        let first = (caret_line + 1).saturating_sub(MAX_LINES);
        let caret_x = self.caret_x(&before, caret_lines.last());
        let status = session
            .status()
            .map(|s| wrap(&s, inner, |c| self.advance(c)))
            .unwrap_or_default();
        let status = status.into_iter().take(MAX_STATUS).collect();
        let mut shown: Vec<String> = lines.into_iter().skip(first).take(MAX_LINES).collect();
        let placeholder = session.placeholder();
        if let Some(text) = &placeholder {
            shown = vec![text.clone()];
        }
        Layout {
            lines: shown,
            placeholder: placeholder.is_some(),
            caret: Some((caret_line - first, caret_x)),
            status,
        }
    }

    fn caret_x(&self, before: &str, last: Option<&String>) -> f32 {
        let line = last.map_or("", String::as_str);
        let trailing = before.chars().rev().take_while(|c| *c == ' ').count();
        let trimmed = line.trim_end_matches(' ');
        let spaces = if line.len() == trimmed.len() {
            trailing
        } else {
            0
        };
        self.width_of(line) + self.advance(' ') * spaces as f32
    }

    /// The pixels of the field in `width` by the height of the layout, as
    /// `wl_shm` ARGB8888 holds them on a little-endian machine. The caret
    /// shows while the field has the keyboard.
    #[must_use]
    pub fn paint(&self, layout: &Layout, look: &Look, width: u32, focused: bool) -> Vec<u8> {
        let height = layout.height();
        let Some(mut pixmap) = Pixmap::new(width.max(1), height.max(1)) else {
            return Vec::new();
        };
        pixmap.fill(color(look.border));
        fill(
            &mut pixmap,
            [1, 1, width.saturating_sub(2), height.saturating_sub(2)],
            look.background,
        );
        let text_color = if layout.placeholder {
            look.hint
        } else {
            look.text
        };
        for (row, line) in layout.lines.iter().enumerate() {
            let rect = row_rect(row, width, layout.text_left());
            self.line(&mut pixmap, line, rect, text_color);
        }
        let status_row = layout.lines.len();
        for (row, line) in layout.status.iter().enumerate() {
            self.line(
                &mut pixmap,
                line,
                row_rect(status_row + row, width, PAD),
                look.hint,
            );
        }
        if let (true, Some((row, x))) = (focused, layout.caret) {
            let top = PAD + u32::try_from(row).unwrap_or(0) * LINE + 2;
            let left = PAD + x.round() as u32;
            fill(&mut pixmap, [left, top, CARET, LINE - 4], look.text);
        }
        bgra(pixmap.data())
    }

    fn line(&self, pixmap: &mut Pixmap, line: &str, rect: [i32; 4], color: [u8; 4]) {
        let text = Text::new(line, Font::Body, color, rect);
        let drawn = self
            .fonts
            .draw(&text, Transform::identity(), &mut pixmap.as_mut());
        if let Err(error) = drawn {
            eprintln!("chora-chat: {error}");
        }
    }
}

/// The rectangle of a row of text in a field of `width`, from `left`.
fn row_rect(row: usize, width: u32, left: u32) -> [i32; 4] {
    let top = PAD + u32::try_from(row).unwrap_or(0) * LINE;
    [left, top, width.saturating_sub(left + PAD), LINE].map(|v| i32::try_from(v).unwrap_or(0))
}

impl Default for Painter {
    fn default() -> Self {
        Self::new()
    }
}

fn color([r, g, b, a]: [u8; 4]) -> Color {
    Color::from_rgba8(r, g, b, a)
}

fn fill(pixmap: &mut Pixmap, [x, y, w, h]: [u32; 4], rgba: [u8; 4]) {
    let Some(rect) = Rect::from_xywh(x as f32, y as f32, w as f32, h as f32) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color(color(rgba));
    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
}

/// Premultiplied RGBA as premultiplied ARGB8888 bytes.
fn bgra(rgba: &[u8]) -> Vec<u8> {
    (rgba.as_chunks::<4>().0.iter())
        .flat_map(|p| [p[2], p[1], p[0], p[3]])
        .collect()
}

#[cfg(test)]
mod tests {
    use chora_kit::words::Language;

    use super::*;
    use crate::session::Key;

    #[test]
    fn a_long_draft_wraps_and_the_field_grows_to_four_lines() {
        let painter = Painter::new();
        let mut session = Session::new(Language::En);
        let one = painter.layout(&session, 300).height();
        session.key(Key::Text("word ".repeat(80)));
        let layout = painter.layout(&session, 300);
        assert_eq!(layout.lines.len(), MAX_LINES);
        assert!(layout.height() > one);
        assert!(layout.caret.is_some_and(|(row, _)| row == MAX_LINES - 1));
    }

    #[test]
    fn the_caret_stands_before_the_placeholder() {
        let painter = Painter::new();
        let session = Session::new(Language::En);
        let layout = painter.layout(&session, 300);
        let caret_right = PAD + CARET;
        assert!(
            layout.text_left() > caret_right,
            "the hint starts after the caret"
        );
        let mut typed = Session::new(Language::En);
        typed.key(Key::Text("hi".into()));
        assert_eq!(
            painter.layout(&typed, 300).text_left(),
            PAD,
            "a draft starts at the pad"
        );
    }

    #[test]
    fn the_empty_field_shows_the_hint_in_a_visible_size() {
        let painter = Painter::new();
        let session = Session::new(Language::En);
        let layout = painter.layout(&session, 300);
        assert!(layout.placeholder);
        assert!(painter.fonts.size(Font::Body) >= 14);
        let pixels = painter.paint(&layout, &Look::default(), 300, true);
        assert_eq!(pixels.len(), 300 * layout.height() as usize * 4);
    }
}
