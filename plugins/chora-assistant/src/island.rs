//! The voice island of a folder with `style: island`: one glass capsule
//! with the state ring, the state word and the task of the helper; while
//! the agent speaks, its sentence in place of the word and the task
//! (construct HUD spec, sections 5 and 6).

use chora_draw::{
    text::wrap,
    theme::{Picture, Theme},
};
use chora_kit::glass::{CAPSULE, Material, Type, UNIT};

use crate::glass::{Line, rect, surface};

/// The widest line of a sentence in the island, and the most lines.
pub const SENTENCE_WIDTH: f64 = 480.0;
const SENTENCE_LINES: usize = 2;
/// The ring and its inset from the left and the top of the capsule.
pub const RING: i32 = 24;
const RING_INSET: f64 = 8.0;
/// The room after the content.
const END: f64 = 16.0;
const ELLIPSIS: char = '\u{2026}';

/// What the island shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Content {
    /// The state word, as the word table has it.
    pub word: String,
    /// The task of the helper with its step, empty without one.
    pub task: String,
    /// The sentence that the agent speaks, if it speaks.
    pub sentence: Option<String>,
}

/// The lines of a sentence wrapped at [`SENTENCE_WIDTH`]: the last two,
/// the first of them after an ellipsis when earlier lines were left out.
fn sentence_lines(theme: &Theme, text: &str, m: &Material) -> Vec<String> {
    let width = |line: &str| Line::new(line, Type::Text, m.text).width(theme);
    let lines = wrap(text, SENTENCE_WIDTH, width);
    if lines.len() <= SENTENCE_LINES {
        return lines;
    }
    let room = SENTENCE_WIDTH - width(&ELLIPSIS.to_string());
    let tail = lines[lines.len() - SENTENCE_LINES..].join(" ");
    let mut kept = wrap(&tail, room, width);
    kept.drain(..kept.len().saturating_sub(SENTENCE_LINES));
    kept[0] = format!("{ELLIPSIS}{}", kept[0]);
    kept
}

/// The picture of the island, without the ring, which the caller lays
/// on it at [`ring_rect`], and its size.
#[must_use]
pub fn picture(theme: &Theme, content: &Content, dark: bool) -> Picture {
    let m = Material::of(dark);
    let mut body = String::new();
    let start = RING_INSET + f64::from(RING) + UNIT;
    let (w, h) = if let Some(sentence) = &content.sentence {
        let lines = sentence_lines(theme, sentence, &m);
        let line_h = Type::Text.face().line;
        #[allow(clippy::cast_precision_loss, reason = "two lines")]
        let rows = lines.len().max(1) as f64;
        let h = (rows * line_h + 2.0 * UNIT).max(CAPSULE);
        let top = (h - rows * line_h) / 2.0;
        let mut widest: f64 = 0.0;
        for (k, text) in lines.iter().enumerate() {
            let line = Line::new(text, Type::Text, m.text);
            widest = widest.max(line.width(theme));
            #[allow(clippy::cast_precision_loss, reason = "two lines")]
            let y = top + k as f64 * line_h;
            body += &line.at([start, y], false);
        }
        ((start + widest + END).ceil(), h)
    } else {
        let word = Line::new(&content.word, Type::Label, m.text);
        let label_h = Type::Label.face().line;
        body += &word.at([start, (CAPSULE - label_h) / 2.0], false);
        let mut end = start + word.width(theme);
        if !content.task.is_empty() {
            let sep = end + UNIT + 4.0;
            body += &rect([sep, (CAPSULE - 16.0) / 2.0, 1.0, 16.0], 0.0, m.separator);
            let task = Line::new(&content.task, Type::Small, m.muted);
            let at = sep + 1.0 + 4.0 + UNIT;
            body += &task.at([at, (CAPSULE - Type::Small.face().line) / 2.0], false);
            end = at + task.width(theme);
        }
        ((end + END).ceil(), CAPSULE)
    };
    let glass = surface([w, h], h.min(CAPSULE) / 2.0, &m, false);
    #[allow(clippy::cast_possible_truncation, reason = "a capsule fits a screen")]
    Picture::new(w as i32, h as i32).svg(glass + &body)
}

/// Where the ring stands on an island of `height`.
#[must_use]
pub fn ring_rect(height: i32) -> [i32; 4] {
    #[allow(clippy::cast_possible_truncation, reason = "a few pixels")]
    let inset = RING_INSET as i32;
    [inset, (height - RING) / 2, RING, RING]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme() -> Theme {
        let cache = std::env::temp_dir().join(format!("chora-island-{}", std::process::id()));
        Theme::host(&cache, "Inter").unwrap()
    }

    #[test]
    fn the_island_is_a_capsule_40_high_that_hugs_its_content() {
        let theme = theme();
        let idle = Content {
            word: "Listening".into(),
            ..Content::default()
        };
        let busy = Content {
            word: "Working".into(),
            task: "Tidy notes · 1 of 2".into(),
            sentence: None,
        };
        let [w, h] = picture(&theme, &idle, false).size;
        let [wide, high] = picture(&theme, &busy, false).size;
        assert_eq!((h, high), (40, 40));
        assert!((100..160).contains(&w), "LISTENING is about 120 wide: {w}");
        assert!(wide > w + 100, "the task widens the island: {wide}");
        assert_eq!(ring_rect(40), [8, 8, 24, 24]);
    }

    #[test]
    fn a_long_sentence_shows_its_last_two_lines() {
        let theme = theme();
        let long = "The next train to Basel leaves at 14:32 from platform 7. ".repeat(4);
        let spoken = Content {
            word: "Speaking".into(),
            task: String::new(),
            sentence: Some(long),
        };
        let [w, h] = picture(&theme, &spoken, true).size;
        assert_eq!(h, 64, "two lines of 24 and a unit above and below");
        assert!(w <= 8 + 24 + 8 + 480 + 16, "{w}");
        let lines = sentence_lines(&theme, &"word ".repeat(80), &Material::of(false));
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with(ELLIPSIS));
    }
}
