//! XP's Confirm File Delete: the question before Empty the Recycle Bin
//! deletes for good, or before Delete sends a file or a folder to the
//! Recycle Bin; and Confirm File Replace before a drop replaces a file.
//! An icon, the question, Yes and No.

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
    /// What Yes does.
    pub then: Then,
}

/// What Yes does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Then {
    /// Empties the Recycle Bin.
    Empty,
    /// Sends files and folders to the Recycle Bin.
    Trash(Vec<PathBuf>),
    /// Drops the first file into a folder in place of the file of its
    /// name; the question for the next file follows.
    Replace(Vec<PathBuf>, PathBuf),
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

    /// The question before Delete or a drop on the Recycle Bin sends
    /// files and folders there, as XP asks it: one by its name, more by
    /// their number.
    pub fn trash(theme: &Theme, paths: Vec<PathBuf>, icon: Option<PathBuf>, area: Rect) -> Self {
        let name = paths
            .first()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let (title, question) = if paths.len() > 1 {
            (
                "Confirm Multiple File Delete",
                format!(
                    "Are you sure you want to send these {} items to the Recycle Bin?",
                    paths.len()
                ),
            )
        } else if paths.first().is_some_and(|p| p.is_dir()) {
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
        dialog.then = Then::Trash(paths);
        dialog
    }

    /// The question before a drop replaces a file of the same name in a
    /// folder, as XP asks it, for the first of the files.
    pub fn replace(
        theme: &Theme,
        paths: Vec<PathBuf>,
        dir: PathBuf,
        icon: Option<PathBuf>,
        area: Rect,
    ) -> Self {
        let name = paths
            .first()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let question = format!(
            "This folder already contains a file named '{name}'. Would you like to replace it?"
        );
        let mut dialog = Self::ask(theme, "Confirm File Replace", question, icon, area);
        dialog.then = Then::Replace(paths, dir);
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
            then: Then::Empty,
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
            ..Page::default()
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
        let file = ConfirmDialog::trash(&theme, vec!["/d/notes.txt".into()], None, area);
        assert_eq!(
            file.question,
            "Are you sure you want to send 'notes.txt' to the Recycle Bin?"
        );
        assert_eq!(file.then, Then::Trash(vec!["/d/notes.txt".into()]));
        let both = ConfirmDialog::trash(&theme, vec!["/a".into(), "/b".into()], None, area);
        assert_eq!(
            both.question,
            "Are you sure you want to send these 2 items to the Recycle Bin?"
        );
        let replace =
            ConfirmDialog::replace(&theme, vec!["/d/a.txt".into()], "/e".into(), None, area);
        assert!(replace.question.contains("a file named 'a.txt'"));
        let _ = std::fs::remove_dir_all(cache);
    }
}
