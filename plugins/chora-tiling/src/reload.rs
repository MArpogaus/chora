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
    match text.char_indices().nth(MOST) {
        Some((end, _)) => format!("{}…", &text[..end]),
        None => text,
    }
}

impl Tiling {
    /// Tells the person that a reload failed and why; the notice of an
    /// earlier failure goes first. Without the server the person gets
    /// none, and the line on stderr stays.
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
        if let Some(earlier) = self.reload_notice.take() {
            let _ = notice::dismiss(&mut self.rt.conn, earlier);
        }
        self.reload_notice = notice::show_or_log(&mut self.rt.conn, None, &notice);
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
