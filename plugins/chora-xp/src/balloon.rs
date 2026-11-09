//! The balloon of the speech of the assistant: a pale yellow rounded
//! rectangle with a black border and a tail towards the head of the
//! paperclip. It goes with the assistant into chora-assistant.

use std::fmt::Write as _;

use chora_draw::text::wrap;

use crate::{
    text::{Font, Text},
    theme::{Picture, Theme},
};

/// The widest line of the assistant's speech.
const MAX_SPEECH: f64 = 220.0;
const PAD: i32 = 10;
const LINE: i32 = 13;
const RADIUS: i32 = 7;
/// The height of the tail and the width of its base.
pub const TAIL: i32 = 18;
/// The inner margin of a balloon.
pub const BALLOON_PAD: i32 = PAD;
const TAIL_BASE: i32 = 16;
const MIN_WIDTH: i32 = 120;
const BLACK: [u8; 4] = [0, 0, 0, 255];

/// The top-left corner of a balloon of a size, and the x of its tail tip
/// in it, for a tip at a point of a screen of a width.
pub fn place([w, h]: [i32; 2], [tip_x, tip_y]: [i32; 2], screen: i32) -> ([i32; 2], i32) {
    let right = (tip_x + 40).min(screen - 2);
    let left = (right - w).max(0);
    let tail = (tip_x - left).clamp(RADIUS + TAIL_BASE, w - RADIUS);
    ([left, tip_y - TAIL - h], tail)
}

/// The rounded rectangle with the tail as one outline, so that the black
/// border runs around both.
fn shape([w, h]: [i32; 2], tail: i32) -> String {
    let (x1, y1) = (f64::from(w) - 0.5, f64::from(h) - 0.5);
    let r = f64::from(RADIUS);
    let t = f64::from(tail) - 0.5;
    let mut path = String::new();
    let _ = write!(
        path,
        "M{a},0.5 H{b} A{r},{r} 0 0 1 {x1},{a} V{c} A{r},{r} 0 0 1 {b},{y1} \
         H{t} V{tip} L{base},{y1} H{a} A{r},{r} 0 0 1 0.5,{c} V{a} A{r},{r} 0 0 1 {a},0.5 Z",
        a = 0.5 + r,
        b = x1 - r,
        c = y1 - r,
        tip = y1 + f64::from(TAIL),
        base = t - f64::from(TAIL_BASE),
    );
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{}"><path d="{path}" fill="#FFFFE1" stroke="#000" stroke-width="1"/></svg>"##,
        h + TAIL
    )
}

/// A balloon with text only, as the assistant speaks: its picture and its
/// top-left corner, for a tail tip at a point of a screen of a width.
/// `room` is a width that the inside keeps at least and a height under
/// the text, for the buttons of a question.
pub fn speech(
    theme: &mut Theme,
    text: &str,
    tip: [i32; 2],
    screen: i32,
    [room_w, room_h]: [i32; 2],
) -> (Picture, [i32; 2]) {
    let width = |s: &str| theme.text_width(s, Font::Ui);
    let lines = wrap(text, MAX_SPEECH, width);
    let text_width = lines.iter().map(|l| width(l)).fold(0.0, f64::max);
    #[allow(clippy::cast_possible_truncation)]
    let w = (PAD + (text_width.ceil() as i32).max(room_w) + PAD).max(MIN_WIDTH);
    let h = PAD + i32::try_from(lines.len()).unwrap_or(0) * LINE + room_h + PAD;
    let (origin, tail) = place([w, h], tip, screen);
    let file = theme
        .cache_file(&format!("balloon-{w}x{h}-{tail}.svg"), &shape([w, h], tail))
        .unwrap_or_default();
    let mut picture = Picture::new(w, h + TAIL).file(&file, [0, 0, w, h + TAIL]);
    for (line, y) in lines.iter().zip((PAD..).step_by(LINE as usize)) {
        picture = picture.text(Text::new(
            line,
            Font::Ui,
            BLACK,
            [PAD, y, w - 2 * PAD, LINE],
        ));
    }
    (picture, origin)
}
