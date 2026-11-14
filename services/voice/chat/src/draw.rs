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

use chora_draw::text::{Font, Fonts, SANS_SERIF, Text, escape};
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
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
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
    /// The radius of the corners of the field; 0 is a square field.
    pub radius: u32,
    /// The glass of the construct HUD: the field is a well of 48 with
    /// Inter 16 and the focus ring of the accent while it has the keyboard.
    pub glass: Option<Glass>,
    /// The field in the margin column of a client: no box, a hairline over
    /// it, the face of the column on lines of its grid.
    pub column: Option<Column>,
}

/// The field in the margin column of a client.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Column {
    /// The helper that the field writes to, for its empty line.
    #[serde(default)]
    pub to: Option<String>,
    /// The room left of the text for the caret of an empty field, in
    /// logical pixels: the surface is this much wider than the column.
    #[serde(default)]
    pub inset: u32,
}

/// The line of the grid of the column and the face of its text.
const COLUMN_LINE: u32 = 28;
const COLUMN_FACE: &str = "Inter";
/// The key that sends, at the right end of the first line of an empty field.
const SEND_KEY: &str = "Enter";

/// The colours of the focus ring of a glass field.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Glass {
    /// The ring of 2 and the caret.
    pub accent: [u8; 4],
    /// The halo of 4 around the ring.
    pub halo: [u8; 4],
}

/// The room around the well of a glass field for its focus ring.
pub const GLASS_INSET: u32 = 6;
/// The padding of the text in the well, the line, the size of the text and
/// the radius of the well.
const GLASS_PAD_X: u32 = 16;
const GLASS_PAD_Y: u32 = 12;
const GLASS_LINE: u32 = 24;
const GLASS_SIZE: f32 = 16.0;
const GLASS_RADIUS: f32 = 16.0;

impl Default for Look {
    fn default() -> Self {
        Self {
            background: [30, 32, 38, 255],
            text: [236, 238, 242, 255],
            border: [96, 104, 120, 255],
            hint: [160, 168, 182, 255],
            radius: 0,
            glass: None,
            column: None,
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
    /// The layout of a glass field.
    glass: bool,
    /// The layout of a field in the margin column: its inset.
    column: Option<u32>,
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
        if self.glass {
            return 2 * GLASS_INSET + 2 * GLASS_PAD_Y + rows.max(1) * GLASS_LINE;
        }
        if self.column.is_some() {
            return rows.max(1) * COLUMN_LINE;
        }
        2 * PAD + rows.max(1) * LINE
    }
}

/// Draws the field.
pub struct Painter {
    fonts: Fonts,
    column: Fonts,
    widths: RefCell<HashMap<(char, Face), f32>>,
}

/// Which face measures and draws the text of a field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Face {
    Plain,
    Glass,
    Column,
}

impl Face {
    fn of(look: &Look) -> Self {
        if look.glass.is_some() {
            Self::Glass
        } else if look.column.is_some() {
            Self::Column
        } else {
            Self::Plain
        }
    }
}

impl Painter {
    /// A painter with the fonts of the host.
    #[must_use]
    pub fn new() -> Self {
        Self {
            fonts: Fonts::host(SANS_SERIF),
            column: Fonts::host(COLUMN_FACE),
            widths: RefCell::new(HashMap::new()),
        }
    }

    /// The advance of a character.
    fn advance(&self, c: char, face: Face) -> f32 {
        if let Some(&width) = self.widths.borrow().get(&(c, face)) {
            return width;
        }
        let measure = |line: &str| match face {
            Face::Glass => self
                .fonts
                .svg_width(&glass_text(line, [0.0, 0.0], [0, 0, 0, 255])),
            Face::Column => self.column.width(line, Font::Body),
            Face::Plain => self.fonts.width(line, Font::Body),
        };
        let width = match (measure(&format!("|{c}|")), measure("||")) {
            (Some(with), Some(without)) => (with - without) as f32,
            _ => FALLBACK_WIDTH,
        };
        self.widths.borrow_mut().insert((c, face), width);
        width
    }

    fn width_of(&self, text: &str, face: Face) -> f32 {
        text.chars().map(|c| self.advance(c, face)).sum()
    }

    /// The layout of a draft in a field of `width` pixels.
    #[must_use]
    pub fn layout(&self, session: &Session, width: u32, look: &Look) -> Layout {
        let face = Face::of(look);
        let glass = face == Face::Glass;
        let column = look.column.as_ref();
        let pad = match face {
            Face::Glass => GLASS_INSET + GLASS_PAD_X,
            Face::Column => column.map_or(0, |c| c.inset),
            Face::Plain => PAD,
        };
        let inner = width.saturating_sub(2 * pad) as f32;
        let advance = |c| self.advance(c, face);
        let before = session.before_cursor();
        let whole = format!("{before}{}", session.after_cursor());
        let lines = wrap(&whole, inner, advance);
        let caret_lines = wrap(&before, inner, advance);
        let caret_line = caret_lines.len().saturating_sub(1).min(lines.len() - 1);
        let first = (caret_line + 1).saturating_sub(MAX_LINES);
        let caret_x = self.caret_x(&before, caret_lines.last(), face);
        let status = session
            .status()
            .map(|s| wrap(&s, inner, advance))
            .unwrap_or_default();
        let status = status.into_iter().take(MAX_STATUS).collect();
        let mut shown: Vec<String> = lines.into_iter().skip(first).take(MAX_LINES).collect();
        let to = column.and_then(|c| c.to.as_deref());
        let placeholder = to
            .filter(|_| session.placeholder().is_some())
            .map(|to| session.write_to(to))
            .or_else(|| session.placeholder());
        if let Some(text) = &placeholder {
            shown = vec![text.clone()];
        }
        Layout {
            lines: shown,
            placeholder: placeholder.is_some(),
            caret: Some((caret_line - first, caret_x)),
            status,
            glass,
            column: column.map(|c| c.inset),
        }
    }

    fn caret_x(&self, before: &str, last: Option<&String>, face: Face) -> f32 {
        let line = last.map_or("", String::as_str);
        let trailing = before.chars().rev().take_while(|c| *c == ' ').count();
        let trimmed = line.trim_end_matches(' ');
        let spaces = if line.len() == trimmed.len() {
            trailing
        } else {
            0
        };
        self.width_of(line, face) + self.advance(' ', face) * spaces as f32
    }

    /// The pixels of the field in `width` by the height of the layout, as
    /// `wl_shm` ARGB8888 holds them on a little-endian machine. The caret
    /// shows while the field has the keyboard.
    #[must_use]
    pub fn paint(
        &self,
        layout: &Layout,
        look: &Look,
        (width, scale): (u32, u32),
        focused: bool,
    ) -> Vec<u8> {
        if let Some(glass) = &look.glass {
            return self.paint_glass(layout, (look, glass), (width, scale), focused);
        }
        if layout.column.is_some() {
            return self.paint_column(layout, look, (width, scale), focused);
        }
        let height = layout.height();
        let Some(mut pixmap) = Pixmap::new(width.max(1), height.max(1)) else {
            return Vec::new();
        };
        if look.radius > 0 {
            self.rounded_box(&mut pixmap, (width, height), look);
        } else {
            pixmap.fill(color(look.border));
            fill(
                &mut pixmap,
                [1, 1, width.saturating_sub(2), height.saturating_sub(2)],
                look.background,
            );
        }
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

    /// The pixels of a field in the margin column at `scale`: no box, a
    /// hairline on the top, the text of the column on its lines of 28, and
    /// the key that sends at the right end of the first line while the
    /// field is empty.
    fn paint_column(
        &self,
        layout: &Layout,
        look: &Look,
        (width, scale): (u32, u32),
        focused: bool,
    ) -> Vec<u8> {
        let (height, scale) = (layout.height(), scale.max(1));
        let inset = layout.column.unwrap_or(0);
        let Some(mut pixmap) = Pixmap::new(width.max(1) * scale, height.max(1) * scale) else {
            return Vec::new();
        };
        pixmap.fill(color(look.background));
        let zoom = scale as f32;
        let at = Transform::from_scale(zoom, zoom);
        let rule = [inset, 0, width.saturating_sub(inset), 1].map(|v| v as f32 * zoom);
        fill_f(&mut pixmap, rule, look.border);
        let text_color = if layout.placeholder {
            look.hint
        } else {
            look.text
        };
        let rows = layout.lines.iter().map(|l| (l, text_color));
        let status = layout.status.iter().map(|l| (l, look.hint));
        for (row, (line, ink)) in rows.chain(status).enumerate() {
            let top = u32::try_from(row).unwrap_or(0) * COLUMN_LINE;
            let rect = [inset, top, width.saturating_sub(inset), COLUMN_LINE]
                .map(|v| i32::try_from(v).unwrap_or(0));
            self.column_line(&mut pixmap, line, (rect, at), ink);
        }
        let room = width.saturating_sub(inset) as f32;
        let first = layout
            .lines
            .first()
            .map_or(0.0, |l| self.width_of(l, Face::Column));
        let key = self.width_of(SEND_KEY, Face::Column);
        if layout.placeholder || first + key + 14.0 < room {
            let left = (width as f32 - key).max(0.0) as i32;
            let rect = [left, 0, key.ceil() as i32 + 1, 28];
            self.column_line(&mut pixmap, SEND_KEY, (rect, at), look.hint);
        }
        if let (true, Some((row, x))) = (focused, layout.caret) {
            let top = u32::try_from(row).unwrap_or(0) * COLUMN_LINE + 6;
            let left = if layout.placeholder {
                inset.saturating_sub(2 * CARET) as f32
            } else {
                inset as f32 + x.round()
            };
            let caret = [left, top as f32, CARET as f32, 17.0].map(|v| v * zoom);
            fill_f(&mut pixmap, caret, look.text);
        }
        bgra(pixmap.data())
    }

    fn column_line(
        &self,
        pixmap: &mut Pixmap,
        line: &str,
        (rect, at): ([i32; 4], Transform),
        color: [u8; 4],
    ) {
        let text = Text::new(line, Font::Body, color, rect);
        if let Err(error) = self.column.draw(&text, at, &mut pixmap.as_mut()) {
            eprintln!("chora-chat: {error}");
        }
    }

    /// The pixels of a glass field at `scale`: the well, the text in
    /// Inter 16, the caret and, with the keyboard, the focus ring.
    fn paint_glass(
        &self,
        layout: &Layout,
        (look, glass): (&Look, &Glass),
        (width, scale): (u32, u32),
        focused: bool,
    ) -> Vec<u8> {
        let height = layout.height();
        let scale = scale.max(1);
        let Some(mut pixmap) = Pixmap::new(width.max(1) * scale, height.max(1) * scale) else {
            return Vec::new();
        };
        let inset = GLASS_INSET as f32;
        let (w, h) = (width as f32 - 2.0 * inset, height as f32 - 2.0 * inset);
        let mut svg = rounded(
            [inset, inset, w, h],
            GLASS_RADIUS,
            &paint("fill", look.background),
        );
        if focused {
            svg += &focus_ring([inset, inset, w, h], *glass);
        }
        let left = (GLASS_INSET + GLASS_PAD_X) as f32;
        let row_top = |row: usize| inset + GLASS_PAD_Y as f32 + row as f32 * GLASS_LINE as f32;
        let text_color = if layout.placeholder {
            look.hint
        } else {
            look.text
        };
        let indent = if layout.placeholder {
            2.0 * CARET as f32
        } else {
            0.0
        };
        for (row, line) in layout.lines.iter().enumerate() {
            svg += &glass_text(line, [left + indent, row_top(row)], text_color);
        }
        for (row, line) in layout.status.iter().enumerate() {
            svg += &glass_text(line, [left, row_top(layout.lines.len() + row)], look.hint);
        }
        if let (true, Some((row, x))) = (focused, layout.caret) {
            svg += &rounded(
                [
                    left + x.round() + 1.0,
                    row_top(row) + 2.0,
                    CARET as f32,
                    20.0,
                ],
                1.0,
                &paint("fill", glass.accent),
            );
        }
        let document = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}">{svg}</svg>"#
        );
        match resvg::usvg::Tree::from_str(&document, self.fonts.options()) {
            Ok(tree) => resvg::render(
                &tree,
                Transform::from_scale(scale as f32, scale as f32),
                &mut pixmap.as_mut(),
            ),
            Err(error) => eprintln!("chora-chat: {error}"),
        }
        bgra(pixmap.data())
    }

    /// The box of a field with rounded corners: the ground and a hairline
    /// inside the edge, with the corners left clear.
    fn rounded_box(&self, pixmap: &mut Pixmap, (width, height): (u32, u32), look: &Look) {
        let svg = rounded(
            [0.5, 0.5, width as f32 - 1.0, height as f32 - 1.0],
            look.radius as f32,
            &format!(
                "{} {} stroke-width=\"1\"",
                paint("fill", look.background),
                paint("stroke", look.border)
            ),
        );
        let document = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}">{svg}</svg>"#
        );
        match resvg::usvg::Tree::from_str(&document, self.fonts.options()) {
            Ok(tree) => resvg::render(&tree, Transform::identity(), &mut pixmap.as_mut()),
            Err(error) => eprintln!("chora-chat: {error}"),
        }
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

/// The focus ring around the well at `[x, y, w, h]`: 2 in the accent and
/// a halo of 4.
#[allow(clippy::many_single_char_names, reason = "the names of SVG")]
fn focus_ring([x, y, w, h]: [f32; 4], glass: Glass) -> String {
    let halo = rounded(
        [x - 4.0, y - 4.0, w + 8.0, h + 8.0],
        GLASS_RADIUS + 4.0,
        &format!(
            r#"fill="none" stroke-width="4" {}"#,
            paint("stroke", glass.halo)
        ),
    );
    let ring = rounded(
        [x - 1.0, y - 1.0, w + 2.0, h + 2.0],
        GLASS_RADIUS + 1.0,
        &format!(
            r#"fill="none" stroke-width="2" {}"#,
            paint("stroke", glass.accent)
        ),
    );
    halo + &ring
}

/// An SVG paint of an RGBA colour.
fn paint(attr: &str, [r, g, b, a]: [u8; 4]) -> String {
    format!(
        r#"{attr}="rgb({r},{g},{b})" {attr}-opacity="{:.3}""#,
        f32::from(a) / 255.0
    )
}

/// A rounded rect at `[x, y, w, h]` with the attributes of its paint.
#[allow(clippy::many_single_char_names, reason = "the names of SVG")]
fn rounded([x, y, w, h]: [f32; 4], r: f32, attrs: &str) -> String {
    format!(r#"<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" {attrs}/>"#)
}

/// A line of a glass field in Inter 16 with its line box of 24 at
/// `[x, top]`.
fn glass_text(line: &str, [x, top]: [f32; 2], color: [u8; 4]) -> String {
    let baseline =
        top + (GLASS_LINE as f32 - 1.210_937_5 * GLASS_SIZE) / 2.0 + 0.968_75 * GLASS_SIZE;
    format!(
        r#"<text x="{x}" y="{baseline:.2}" font-family="{}" font-size="{GLASS_SIZE}" {} xml:space="preserve">{}</text>"#,
        chora_kit::glass::SANS,
        paint("fill", color),
        escape(line)
    )
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

fn fill_f(pixmap: &mut Pixmap, [x, y, w, h]: [f32; 4], rgba: [u8; 4]) {
    let Some(rect) = Rect::from_xywh(x, y, w, h) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color(color(rgba));
    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
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
        let one = painter.layout(&session, 300, &Look::default()).height();
        session.key(Key::Text("word ".repeat(80)));
        let layout = painter.layout(&session, 300, &Look::default());
        assert_eq!(layout.lines.len(), MAX_LINES);
        assert!(layout.height() > one);
        assert!(layout.caret.is_some_and(|(row, _)| row == MAX_LINES - 1));
    }

    #[test]
    fn a_glass_field_is_a_well_of_48_with_room_for_its_ring_at_its_scale() {
        let painter = Painter::new();
        let look = Look {
            glass: Some(Glass {
                accent: [38, 128, 180, 255],
                halo: [38, 128, 180, 56],
            }),
            ..Look::default()
        };
        let session = Session::new(Language::En);
        let layout = painter.layout(&session, 396, &look);
        assert_eq!(layout.height(), 48 + 2 * GLASS_INSET);
        let pixels = painter.paint(&layout, &look, (396, 2), true);
        assert_eq!(pixels.len(), 396 * 2 * 60 * 2 * 4);
        let alpha = |x: usize, y: usize| pixels[(y * 396 * 2 + x) * 4 + 3];
        assert_eq!(alpha(0, 0), 0, "the corner outside the halo is clear");
        assert!(
            alpha(2 * 6, 2 * 30) > 200,
            "the ring on the left edge of the well"
        );
    }

    #[test]
    fn the_caret_stands_before_the_placeholder() {
        let painter = Painter::new();
        let session = Session::new(Language::En);
        let layout = painter.layout(&session, 300, &Look::default());
        let caret_right = PAD + CARET;
        assert!(
            layout.text_left() > caret_right,
            "the hint starts after the caret"
        );
        let mut typed = Session::new(Language::En);
        typed.key(Key::Text("hi".into()));
        assert_eq!(
            painter.layout(&typed, 300, &Look::default()).text_left(),
            PAD,
            "a draft starts at the pad"
        );
    }

    #[test]
    fn the_empty_field_shows_the_hint_in_a_visible_size() {
        let painter = Painter::new();
        let session = Session::new(Language::En);
        let layout = painter.layout(&session, 300, &Look::default());
        assert!(layout.placeholder);
        assert!(painter.fonts.size(Font::Body) >= 14);
        let pixels = painter.paint(&layout, &Look::default(), (300, 1), true);
        assert_eq!(pixels.len(), 300 * layout.height() as usize * 4);
    }

    fn column_look() -> Look {
        Look {
            background: [255, 255, 255, 255],
            text: [0, 0, 0, 255],
            border: [200, 0, 0, 255],
            hint: [0, 0, 200, 255],
            radius: 0,
            glass: None,
            column: Some(Column {
                to: Some("Atlas".into()),
                inset: 6,
            }),
        }
    }

    /// The columns and rows of ink of a layer: the pixels that are not the ground.
    fn ink(pixels: &[u8], width: usize, rows: std::ops::Range<usize>) -> Vec<(usize, usize)> {
        let mut found = Vec::new();
        for y in rows {
            for x in 0..width {
                let p = &pixels[(y * width + x) * 4..][..4];
                if p[..3] != [255, 255, 255] && p[3] != 0 {
                    found.push((x, y));
                }
            }
        }
        found
    }

    #[test]
    fn a_field_in_the_column_is_lines_of_the_grid_without_a_box_and_sharp_at_two() {
        let painter = Painter::new();
        let look = column_look();
        let layout = painter.layout(&Session::new(Language::En), 336, &look);
        assert_eq!(layout.height(), 28, "one line of the grid of 28");
        for scale in [1usize, 2] {
            let pixels = painter.paint(&layout, &look, (336, scale as u32), false);
            assert_eq!(
                pixels.len(),
                336 * 28 * scale * scale * 4,
                "a buffer at the scale"
            );
            no_box_and_a_hairline(&pixels, scale);
            text_in_place(&pixels, scale);
        }
    }

    fn no_box_and_a_hairline(pixels: &[u8], scale: usize) {
        let (width, height) = (336 * scale, 28 * scale);
        let at = |x: usize, y: usize| pixels[(y * width + x) * 4..][..3].to_vec();
        assert_eq!(at(0, height / 2), [255; 3], "no box on the left");
        assert_eq!(at(width - 1, height / 2), [255; 3], "no box on the right");
        assert_ne!(at(width / 2, 0), [255; 3], "a hairline on the top");
        assert_eq!(at(width / 2, scale), [255; 3], "and nothing under it");
    }

    fn text_in_place(pixels: &[u8], scale: usize) {
        let (width, height) = (336 * scale, 28 * scale);
        let ink = ink(pixels, width, scale..height);
        let left = ink.iter().map(|p| p.0).min().unwrap();
        assert!(
            (6 * scale..=8 * scale).contains(&left),
            "the text starts at the inset 6, not at {left} of {scale}"
        );
        let right = ink.iter().map(|p| p.0).max().unwrap();
        assert!(
            right + 2 * scale >= width,
            "Enter ends at the edge, at {right} of {width}"
        );
        let baseline = ink.iter().map(|p| p.1).max().unwrap() + 1;
        assert!(
            baseline.abs_diff(19 * scale) <= scale,
            "the baseline is {baseline}"
        );
    }

    #[test]
    fn the_lines_of_a_field_in_the_column_start_at_one_inset() {
        let painter = Painter::new();
        let look = column_look();
        let mut session = Session::new(Language::En);
        session.pick_first("word");
        let layout = painter.layout(&session, 336, &look);
        assert!(layout.height() >= 56 && layout.height().is_multiple_of(28));
        let pixels = painter.paint(&layout, &look, (336, 1), false);
        let rows =
            |r: std::ops::Range<usize>| ink(&pixels, 336, r).iter().map(|p| p.0).min().unwrap();
        let (first, second) = (rows(2..28), rows(30..56));
        assert!(first.abs_diff(second) <= 1, "{first} and {second}");
    }

    #[test]
    fn a_field_with_a_radius_leaves_its_corners_clear() {
        let painter = Painter::new();
        let session = Session::new(Language::En);
        let square = Look::default();
        let round = Look {
            radius: 8,
            ..Look::default()
        };
        let alpha = |look: &Look| {
            let layout = painter.layout(&session, 300, look);
            painter.paint(&layout, look, (300, 1), false)[3]
        };
        assert_eq!(alpha(&square), 255, "a square field fills its corner");
        assert_eq!(alpha(&round), 0, "a round field leaves it clear");
    }
}
