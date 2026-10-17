//! XP's Confirm File Delete: the question before Empty the Recycle Bin
//! deletes for good, or before Delete sends a file or a folder to the
//! Recycle Bin. The icon of the bin, the question, Yes and No.

use std::path::PathBuf;

use crate::{
    dialog::{self, BUTTON, Control, Dialog, Hit, Page},
    text::Font,
    theme::{Rect, Theme},
};

/// The open question.
pub struct ConfirmDialog {
    /// The window.
    pub dialog: Dialog,
    question: String,
    icon: Option<PathBuf>,
    size: [i32; 2],
    /// The file that Yes sends to the Recycle Bin; none to empty it.
    pub trash: Option<PathBuf>,
}

impl ConfirmDialog {
    /// The question for the names of the items, in the middle of a work
    /// area: one item by its name, more by their number, as XP asks.
    pub fn new(theme: &Theme, names: &[String], icon: Option<PathBuf>, area: Rect) -> Self {
        let (title, question) = match names {
            [name] => (
                "Confirm File Delete",
                format!("Are you sure you want to delete '{name}'?"),
            ),
            _ => (
                "Confirm Multiple File Delete",
                format!(
                    "Are you sure you want to delete these {} items?",
                    names.len()
                ),
            ),
        };
        Self::ask(theme, title, question, icon, area)
    }

    /// The question before Delete sends a file or a folder to the
    /// Recycle Bin, as XP asks it.
    pub fn trash(theme: &Theme, path: PathBuf, icon: Option<PathBuf>, area: Rect) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let (title, question) = if path.is_dir() {
            (
                "Confirm Folder Delete",
                format!(
                    "Are you sure you want to remove the folder '{name}' and move all its contents to the Recycle Bin?"
                ),
            )
        } else {
            (
                "Confirm File Delete",
                format!("Are you sure you want to send '{name}' to the Recycle Bin?"),
            )
        };
        let mut dialog = Self::ask(theme, title, question, icon, area);
        dialog.trash = Some(path);
        dialog
    }

    fn ask(
        theme: &Theme,
        title: &str,
        question: String,
        icon: Option<PathBuf>,
        area: Rect,
    ) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        let text = theme.text_width(&question, Font::Ui).ceil() as i32;
        let size = [(text + 75).clamp(260, 560), 98];
        Self {
            dialog: Dialog::new("confirm", title, size, area),
            question,
            icon,
            size,
            trash: None,
        }
    }

    /// The face with the icon and the question, and the buttons.
    pub fn page(&self) -> Page {
        let [w, h] = self.size;
        let mut body = dialog::face(self.size);
        if let Some(icon) = &self.icon {
            body = body.file(icon, [11, 11, 32, 32]);
        }
        body = dialog::label(body, &self.question, [55, 20, w - 65, 13], true);
        let left = (w - 2 * BUTTON[0] - 6) / 2;
        Page {
            body: Some(body),
            controls: vec![
                Control::push("yes", "Yes", [left, h - 32]).default(),
                Control::push("no", "No", [left + BUTTON[0] + 6, h - 32]),
            ],
        }
    }

    /// Whether a click says Yes.
    pub fn yes(hit: &Hit) -> bool {
        matches!(hit, Hit::Control(name, _) if name == "yes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialog::tests::{preview, theme};

    #[test]
    fn the_question_names_one_item_or_counts_them() {
        let (mut theme, cache) = theme("confirm");
        let area = [0, 0, 800, 600];
        let one = ConfirmDialog::new(&theme, &["notes.txt".into()], None, area);
        assert_eq!(one.question, "Are you sure you want to delete 'notes.txt'?");
        let mut two = ConfirmDialog::new(&theme, &["a".into(), "b".into()], None, area);
        assert_eq!(
            two.question,
            "Are you sure you want to delete these 2 items?"
        );
        let page = two.page();
        two.dialog.create(&mut theme, "root", page);
        preview(&mut theme, &two.dialog, "confirm");
        assert!(ConfirmDialog::yes(&Hit::Control("yes".into(), [1, 1])));
        assert!(!ConfirmDialog::yes(&Hit::Close));
        let file = ConfirmDialog::trash(&theme, "/d/notes.txt".into(), None, area);
        assert_eq!(
            file.question,
            "Are you sure you want to send 'notes.txt' to the Recycle Bin?"
        );
        assert_eq!(file.trash, Some("/d/notes.txt".into()));
        let _ = std::fs::remove_dir_all(cache);
    }
}
