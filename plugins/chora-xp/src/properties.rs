//! XP's Properties of a file, a folder, the Recycle Bin or an entry of the
//! start menu: the tab General with the icon and the name at the top and
//! lines of what it is below, in groups between etched lines. The dialog
//! only shows: chora-xp renames nothing and changes no attributes, so the
//! name is gray, OK and Cancel close it and Apply stays gray.

use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::{
    dialog::{self, Control, Dialog, Page},
    folder::{self, Kind},
    text::{Font, Text},
    theme::{Picture, Rect, Theme},
};

/// The client area, as XP's Properties of a file.
const SIZE: [i32; 2] = [355, 395];
const PAGE: Rect = [6, 27, 343, 333];
/// The column of the values.
const VALUE_X: i32 = 104;
const LINE: i32 = 22;

/// A line of the page: a label and its value, or an etched rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    /// A label such as `Location:` and its value.
    Text(String, String),
    /// The rule between two groups.
    Rule,
}

fn text(label: &str, value: impl Into<String>) -> Line {
    Line::Text(label.into(), value.into())
}

/// The open dialog.
pub struct PropertiesDialog {
    /// The window.
    pub dialog: Dialog,
    name: String,
    icon: Option<PathBuf>,
    lines: Vec<Line>,
}

impl PropertiesDialog {
    /// The Properties of a name with its icon and lines, in the middle of
    /// a work area.
    pub fn new(name: &str, icon: Option<PathBuf>, lines: Vec<Line>, area: Rect) -> Self {
        Self {
            dialog: Dialog::new("properties", &format!("{name} Properties"), SIZE, area),
            name: name.into(),
            icon,
            lines,
        }
    }

    /// The tab General and the buttons.
    pub fn page(&self, theme: &Theme) -> Page {
        let mut body = dialog::face(SIZE).svg(dialog::tab_page(PAGE));
        if let Some(icon) = &self.icon {
            body = body.file(icon, [21, 42, 32, 32]);
        }
        let field = [74, 47, 258, 21];
        body = body.svg(dialog::field(field)).text(Text::new(
            &self.name,
            Font::Ui,
            dialog::GRAY,
            [field[0] + 4, field[1], field[2] - 8, field[3]],
        ));
        let mut y = 88;
        body = rule(body, y);
        y += 10;
        for line in &self.lines {
            match line {
                Line::Rule => {
                    body = rule(body, y);
                    y += 10;
                }
                Line::Text(label, value) => {
                    body = dialog::label(body, label, [21, y, VALUE_X - 25, 13], true);
                    body =
                        dialog::label(body, value, [VALUE_X, y, PAGE[2] - VALUE_X - 6, 13], true);
                    y += LINE;
                }
            }
        }
        let mut controls = dialog::tabs(theme, [PAGE[0], PAGE[1]], &["General"], 0);
        let [w, h] = SIZE;
        controls.extend([
            Control::push("ok", "OK", [w - 3 * 73 - 2 * 8 - 7, h - 29]).default(),
            Control::push("cancel", "Cancel", [w - 2 * 73 - 8 - 7, h - 29]),
            Control::push("apply", "Apply", [w - 73 - 7, h - 29]).enabled(false),
        ]);
        Page {
            body: Some(body),
            controls,
            ..Page::default()
        }
    }
}

/// XP's etched line across the page at a height.
fn rule(picture: Picture, y: i32) -> Picture {
    picture.svg(format!(
        r##"<rect x="21" y="{y}" width="311" height="1" fill="#D0D0BF"/><rect x="21" y="{}" width="311" height="1" fill="#fff"/>"##,
        y + 1
    ))
}

/// A number with XP's thousands separators: `12,650`.
fn grouped(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// A number of items: `1 item`, `2 items`.
fn items(n: usize) -> String {
    if n == 1 {
        "1 item".into()
    } else {
        format!("{n} items")
    }
}

/// A size as XP's Properties writes it: `12.3 KB (12,650 bytes)`.
pub fn size_line(bytes: u64) -> String {
    let short = folder::size_text(bytes);
    if bytes < 1024 {
        return short;
    }
    format!("{short} ({} bytes)", grouped(bytes))
}

/// A time as XP's Properties writes it: `Saturday, October 17, 2026,
/// 8:55:31 AM`.
fn time_line(time: SystemTime) -> String {
    chrono::DateTime::<chrono::Local>::from(time)
        .format("%A, %B %-d, %Y, %-I:%M:%S %p")
        .to_string()
}

/// The lines of a file or a folder: its type, where it is, its size or
/// what it holds, and its times.
pub fn of_path(path: &Path) -> Vec<Line> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let meta = fs::metadata(path).ok();
    let dir = meta.as_ref().is_some_and(fs::Metadata::is_dir);
    let executable = meta.as_ref().is_some_and(|m| {
        use std::os::unix::fs::PermissionsExt as _;
        m.permissions().mode() & 0o111 != 0
    });
    let kind = Kind::of(&name, dir, executable);
    let label = if dir { "Type:" } else { "Type of file:" };
    let mut lines = vec![text(label, folder::type_name(&name, kind)), Line::Rule];
    let location = path
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    lines.push(text("Location:", location));
    if dir {
        // ponytail: the folder's own entries, not the whole tree as XP
        // counts it; a walk of a large home directory would block.
        let count = fs::read_dir(path).map_or(0, Iterator::count);
        lines.push(text("Contains:", items(count)));
    } else if let Some(meta) = &meta {
        lines.push(text("Size:", size_line(meta.len())));
    }
    lines.push(Line::Rule);
    if let Some(meta) = &meta {
        for (label, time) in [
            ("Created:", meta.created()),
            ("Modified:", meta.modified()),
            ("Accessed:", meta.accessed()),
        ] {
            if let Ok(time) = time {
                lines.push(text(label, time_line(time)));
            }
        }
    }
    lines
}

/// The lines of the Recycle Bin: how many items it holds and their
/// size, from the files of a trash.
pub fn of_trash(trashed: &[folder::Trashed]) -> Vec<Line> {
    let size: u64 = trashed.iter().map(|t| t.entry.size).sum();
    vec![
        text("Type:", "System Folder"),
        Line::Rule,
        text("Location:", "Desktop"),
        text("Contains:", items(trashed.len())),
        text("Size:", size_line(size)),
    ]
}

/// The lines of a shortcut of the start menu or of a desktop file: the
/// command it runs.
pub fn of_shortcut(target: &str) -> Vec<Line> {
    vec![
        text("Type of file:", "Shortcut"),
        Line::Rule,
        text("Target:", target),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialog::tests::{preview, theme};

    #[test]
    fn sizes_and_numbers_read_as_in_xp() {
        assert_eq!(grouped(12_650), "12,650");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(1_234_567), "1,234,567");
        assert_eq!(size_line(12_650), "12.3 KB (12,650 bytes)");
        assert_eq!(size_line(5), "5 bytes");
    }

    #[test]
    fn a_file_shows_its_type_place_and_size() {
        let dir = std::env::temp_dir().join(format!("chora-xp-props-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("notes.txt"), "12345").unwrap();
        let lines = of_path(&dir.join("notes.txt"));
        assert_eq!(lines[0], text("Type of file:", "Text Document"));
        assert_eq!(lines[2], text("Location:", dir.display().to_string()));
        assert_eq!(lines[3], text("Size:", "5 bytes"));
        assert!(
            lines
                .iter()
                .any(|l| matches!(l, Line::Text(label, _) if label == "Modified:"))
        );
        let folder = of_path(&dir);
        assert_eq!(folder[0], text("Type:", "File Folder"));
        assert_eq!(folder[3], text("Contains:", "2 items"));
        let (mut theme, cache) = theme("properties");
        let mut p = PropertiesDialog::new("notes.txt", None, lines, [0, 0, 800, 600]);
        let page = p.page(&theme);
        p.dialog.create(&mut theme, "root", page);
        preview(&mut theme, &p.dialog, "properties");
        fs::remove_dir_all(&dir).unwrap();
        let _ = fs::remove_dir_all(cache);
    }
}
