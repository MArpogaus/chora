//! The notice of a reload that failed (ti-53): MOD+Ctrl+r with a broken
//! file says why, in the notification server, and the old settings stay.

use chora_kit::{
    notice::{self, Notice, Urgency},
    words::Language,
};

use crate::{NAME, Tiling};

/// How long the notice shows, in milliseconds.
const SHOWN_MS: i32 = 12_000;
/// The most characters of the reason in the notice.
const MOST: usize = 160;
/// The most characters of the reason on the chip.
const MOST_ON_CHIP: usize = 70;
/// How long the chip shows the reason, where no notice can.
const CHIP_MS: u64 = 5_000;

/// The text cut to `most` characters, with a `…` where it is cut.
fn cut(text: &str, most: usize) -> String {
    match text.char_indices().nth(most) {
        Some((end, _)) => format!("{}…", &text[..end]),
        None => text.to_owned(),
    }
}

/// The reason of a failed reload for a notice: the file by its name, then
/// the line and the cause as the parser has them, cut short.
fn reason(message: &str) -> String {
    let (file, cause) = message.split_once(": ").unwrap_or(("", message));
    let name = file.rsplit('/').next().unwrap_or(file);
    let cause = cause
        .split(", expected ")
        .next()
        .unwrap_or(cause)
        .lines()
        .next()
        .unwrap_or_default();
    let text = if name.is_empty() {
        cause.to_owned()
    } else {
        format!("{name}: {cause}")
    };
    cut(&text, MOST)
}

impl Tiling {
    /// Takes the card of a failed reload down after a good one: the
    /// settings that did apply must not show next to "not applied".
    pub(crate) fn dismiss_reload_notice(&mut self) {
        if let Some(earlier) = self.reload_notice.take() {
            let _ = notice::dismiss(&mut self.rt.conn, earlier);
        }
    }

    /// Tells the person that a reload failed and why; the notice of an
    /// earlier failure goes first. Without the server the reason shows on
    /// the chip of tiling, and the line on stderr stays.
    pub(crate) fn tell_reload_failed(&mut self, message: &str) {
        let german = Language::of_session() == Language::De;
        let (title, rest) = if german {
            ("Einstellungen nicht übernommen", "Die alten gelten weiter.")
        } else {
            ("Settings not applied", "The old settings stay.")
        };
        let body = format!("{}\n{rest}", reason(message));
        let notice = Notice {
            app: NAME,
            icon: "",
            summary: title,
            body: &body,
            expire_ms: SHOWN_MS,
            urgency: Urgency::Normal,
        };
        self.dismiss_reload_notice();
        self.reload_notice = notice::show_or_log(&mut self.rt.conn, None, &notice);
        if self.reload_notice.is_none() {
            let word = format!("{title}: {}", cut(&reason(message), MOST_ON_CHIP));
            let time = std::time::Duration::from_millis(CHIP_MS);
            let _ = self.show_words(&word, time);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::reason;

    #[test]
    fn the_reason_names_the_file_and_the_line() {
        let message =
            "/home/a/.config/chora/chora-tiling.yaml: simple key expected ':' at line 24, column 3";
        assert_eq!(
            reason(message),
            "chora-tiling.yaml: simple key expected ':' at line 24, column 3"
        );
    }

    #[test]
    fn a_list_of_variants_is_cut_and_a_long_text_ends_in_a_dot_dot_dot() {
        let message =
            "/x/chora-tiling.yaml: unknown variant `frobnicate`, expected one of `close`, `run`";
        assert_eq!(
            reason(message),
            "chora-tiling.yaml: unknown variant `frobnicate`"
        );
        let long = format!("/x/a.yaml: {}", "y".repeat(400));
        assert!(reason(&long).chars().count() <= 161, "{}", reason(&long));
    }
}
