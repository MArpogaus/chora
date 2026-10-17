//! Desktop entries: the applications of the XDG desktop files, as a
//! menu or a launcher shows them.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::xdg;

/// The fields of a desktop file that a menu uses.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Entry {
    /// The name.
    pub name: String,
    /// The command line without field codes.
    pub exec: String,
    /// The icon name, or the path of an icon file.
    pub icon: String,
    /// The categories, main and additional.
    pub categories: Vec<String>,
    /// The program runs in a terminal.
    pub terminal: bool,
    /// Hidden, not shown, or not an application.
    pub hidden: bool,
}

/// The entries of the desktop files in the XDG data directories, hidden
/// ones too. A desktop file ID in an earlier directory hides those in
/// later ones.
#[must_use]
pub fn applications() -> Vec<Entry> {
    let desktops = current_desktops();
    let desktops: Vec<&str> = desktops.iter().map(String::as_str).collect();
    let mut entries = HashMap::new();
    for dir in xdg::data_dirs() {
        let dir = dir.join("applications");
        for (id, path) in desktop_files(&dir, &dir) {
            entries.entry(id).or_insert_with(|| {
                fs::read_to_string(path)
                    .ok()
                    .and_then(|text| parse(&text, &desktops))
            });
        }
    }
    entries.into_values().flatten().collect()
}

/// The names of the current desktop, from `$XDG_CURRENT_DESKTOP`, for
/// `OnlyShowIn` and `NotShowIn`.
fn current_desktops() -> Vec<String> {
    std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .split(':')
        .filter(|d| !d.is_empty())
        .map(String::from)
        .collect()
}

/// The desktop files below a directory and their IDs: the path from
/// `base` with `-` for `/`.
fn desktop_files(dir: &Path, base: &Path) -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    for path in fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
    {
        if path.is_dir() {
            out.extend(desktop_files(&path, base));
        } else if path.extension().is_some_and(|e| e == "desktop") {
            let id = path.strip_prefix(base).unwrap_or(&path);
            out.push((id.to_string_lossy().replace('/', "-"), path));
        }
    }
    out
}

/// The `[Desktop Entry]` group of a desktop file, if it has one with a
/// name. `desktops` are the names of the current desktop for `OnlyShowIn`
/// and `NotShowIn`.
fn parse(text: &str, desktops: &[&str]) -> Option<Entry> {
    let mut entry = Entry::default();
    let mut found = false;
    let mut group = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            group = line == "[Desktop Entry]";
            found |= group;
            continue;
        }
        let Some((key, value)) = line.split_once('=').filter(|_| group) else {
            continue;
        };
        let value = unescape(value.trim());
        let listed = value.split(';').any(|d| desktops.contains(&d));
        match key.trim() {
            "Name" => entry.name = value,
            "Exec" => entry.exec = strip_field_codes(&value),
            "Icon" => entry.icon = value,
            "Categories" => entry.categories = list(&value),
            "Terminal" => entry.terminal = value == "true",
            "Type" => entry.hidden |= value != "Application",
            "NoDisplay" | "Hidden" => entry.hidden |= value == "true",
            "OnlyShowIn" => entry.hidden |= !listed,
            "NotShowIn" => entry.hidden |= listed,
            _ => {}
        }
    }
    (found && !entry.name.is_empty()).then_some(entry)
}

/// The escapes of a string value: `\s`, `\n`, `\t`, `\r` and `\\`.
fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('s') => out.push(' '),
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

fn list(value: &str) -> Vec<String> {
    value
        .split(';')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// The command line without the field codes: a menu opens no files.
/// `%%` is a percent sign.
fn strip_field_codes(exec: &str) -> String {
    let mut out = String::with_capacity(exec.len());
    let mut chars = exec.chars();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
        } else if chars.next() == Some('%') {
            out.push('%');
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The command of a program, in the terminal if it needs one.
#[must_use]
pub fn command(exec: String, in_terminal: bool, terminal: &str) -> String {
    if in_terminal {
        format!("{terminal} -e {exec}")
    } else {
        exec
    }
}

/// A desktop file as a launcher shows it.
#[derive(Debug, PartialEq, Eq)]
pub struct Launcher {
    /// The name.
    pub name: String,
    /// The icon name, or the path of an icon file.
    pub icon: String,
    /// The command line, in the terminal if the program needs one.
    pub command: String,
}

/// The launcher of a desktop file, as on the desktop, if it is an
/// application to show.
#[must_use]
pub fn launcher(path: &Path, terminal: &str) -> Option<Launcher> {
    let desktops = current_desktops();
    let desktops: Vec<&str> = desktops.iter().map(String::as_str).collect();
    let entry = parse(&fs::read_to_string(path).ok()?, &desktops)?;
    (!entry.hidden && !entry.exec.is_empty()).then(|| {
        let command = command(entry.exec, entry.terminal, terminal);
        Launcher {
            name: entry.name,
            icon: entry.icon,
            command,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = "\
[Desktop Entry]
Type=Application
Name=Text\\sEditor
Name[de]=Texteditor
Exec=gedit --new-window %U
Icon=accessories-text-editor
Categories=GNOME;GTK;Utility;TextEditor;

[Desktop Action new-window]
Name=New Window
Exec=gedit --other
";

    #[test]
    fn a_desktop_file_gives_its_main_group() {
        let entry = parse(FILE, &[]).expect("an entry");
        assert_eq!(entry.name, "Text Editor");
        assert_eq!(entry.exec, "gedit --new-window");
        assert_eq!(entry.icon, "accessories-text-editor");
        assert_eq!(entry.categories, ["GNOME", "GTK", "Utility", "TextEditor"]);
        assert!(!entry.hidden && !entry.terminal);
        assert_eq!(parse("Name=x\n", &[]), None);
    }

    #[test]
    fn hidden_and_foreign_entries_hide() {
        let hidden = |extra: &str, desktops: &[&str]| {
            parse(&format!("[Desktop Entry]\nName=a\n{extra}\n"), desktops)
                .is_some_and(|e| e.hidden)
        };
        assert!(hidden("NoDisplay=true", &[]));
        assert!(hidden("Hidden=true", &[]));
        assert!(hidden("Type=Link", &[]));
        assert!(hidden("OnlyShowIn=KDE;", &["GNOME"]));
        assert!(!hidden("OnlyShowIn=KDE;", &["KDE"]));
        assert!(hidden("NotShowIn=KDE;", &["KDE"]));
        assert!(!hidden("Terminal=true", &[]));
    }

    #[test]
    fn field_codes_go() {
        assert_eq!(strip_field_codes("app %f %F %u %U %i %c %k"), "app");
        assert_eq!(strip_field_codes("app --x=%u 100%%"), "app --x= 100%");
        assert_eq!(strip_field_codes("sh -c \"a %F\""), "sh -c \"a \"");
    }
}
