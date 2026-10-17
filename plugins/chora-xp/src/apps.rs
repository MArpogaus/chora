//! The programs of All Programs: the applications of the XDG desktop
//! files, in program groups by their main category, as on XP.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use chora_kit::xdg;

/// An entry of All Programs or of one of its groups.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    /// The name.
    pub title: String,
    /// The icon name, or the path of an icon file.
    pub icon: String,
    /// A group or a program.
    pub kind: Kind,
}

/// What an entry is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A program group with its entries.
    Group(Vec<Node>),
    /// A program and its shell command.
    Run(String),
    /// A setting of the Control Panel, with a check mark if it is the
    /// current one.
    Pick(crate::control::Setting, bool),
}

/// The program groups: the freedesktop main categories and their names
/// on the menu. Audio and Video come without `AudioVideo` in some files.
const GROUPS: [(&str, &str); 13] = [
    ("Utility", "Accessories"),
    ("Development", "Development"),
    ("Education", "Education"),
    ("Game", "Games"),
    ("Graphics", "Graphics"),
    ("Network", "Internet"),
    ("AudioVideo", "Multimedia"),
    ("Audio", "Multimedia"),
    ("Video", "Multimedia"),
    ("Office", "Office"),
    ("Science", "Science"),
    ("Settings", "Settings"),
    ("System", "System"),
];

/// The icon of all program groups, as on XP.
const GROUP_ICON: &str = "applications-other";

/// The fields of a desktop file that the menu uses.
#[derive(Debug, Default, PartialEq, Eq)]
struct Entry {
    name: String,
    exec: String,
    icon: String,
    categories: Vec<String>,
    terminal: bool,
    /// Hidden, not shown, or not an application.
    hidden: bool,
}

/// The applications of the desktop files in the XDG data directories, in
/// groups. `terminal` runs the programs that need one.
pub fn load(terminal: &str) -> Vec<Node> {
    let desktops = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    let desktops: Vec<&str> = desktops.split(':').filter(|d| !d.is_empty()).collect();
    // A desktop file ID in an earlier directory hides those in later ones.
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
    group(entries.into_values().flatten(), terminal)
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

/// The command line without the field codes: the menu opens no files.
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

/// The groups of the applications by their first main category, sorted
/// by name, and then the applications without one.
fn group(entries: impl Iterator<Item = Entry>, terminal: &str) -> Vec<Node> {
    let mut groups: Vec<(&str, Vec<Node>)> = Vec::new();
    let mut loose = Vec::new();
    for entry in entries.filter(|e| !e.hidden && !e.exec.is_empty()) {
        let name = entry
            .categories
            .iter()
            .find_map(|c| GROUPS.iter().find(|(main, _)| main == c))
            .map(|(_, name)| *name);
        let command = command(entry.exec, entry.terminal, terminal);
        let node = Node {
            title: entry.name,
            icon: entry.icon,
            kind: Kind::Run(command),
        };
        match (name, groups.iter_mut().find(|(g, _)| Some(*g) == name)) {
            (None, _) => loose.push(node),
            (Some(_), Some((_, nodes))) => nodes.push(node),
            (Some(name), None) => groups.push((name, vec![node])),
        }
    }
    let mut out: Vec<Node> = groups
        .into_iter()
        .map(|(name, mut nodes)| {
            sort(&mut nodes);
            Node {
                title: name.into(),
                icon: GROUP_ICON.into(),
                kind: Kind::Group(nodes),
            }
        })
        .collect();
    sort(&mut out);
    sort(&mut loose);
    out.extend(loose);
    out
}

/// The command of a program, in the terminal if it needs one.
fn command(exec: String, in_terminal: bool, terminal: &str) -> String {
    if in_terminal {
        format!("{terminal} -e {exec}")
    } else {
        exec
    }
}

/// The name, icon and command of a desktop file, for an icon on the
/// desktop, if it is an application to show.
pub fn launcher(path: &Path, terminal: &str) -> Option<(String, String, String)> {
    let entry = parse(&fs::read_to_string(path).ok()?, &[])?;
    (!entry.hidden && !entry.exec.is_empty()).then(|| {
        let command = command(entry.exec, entry.terminal, terminal);
        (entry.name, entry.icon, command)
    })
}

fn sort(nodes: &mut [Node]) {
    nodes.sort_by_cached_key(|n| n.title.to_lowercase());
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

    #[test]
    fn programs_group_like_xp() {
        let entry = |name: &str, categories: &[&str], terminal| Entry {
            name: name.into(),
            exec: name.to_lowercase(),
            categories: categories.iter().map(|c| (*c).to_string()).collect(),
            terminal,
            ..Entry::default()
        };
        let nodes = group(
            [
                entry("Zed", &["Development"], false),
                entry("htop", &["System", "Monitor"], true),
                entry("Calc", &["GTK", "Utility"], false),
                entry("Abc", &["Development", "IDE"], false),
                entry("Loose", &[], false),
                entry("Player", &["Audio", "Player"], false),
            ]
            .into_iter(),
            "foot",
        );
        let titles: Vec<&str> = nodes.iter().map(|n| n.title.as_str()).collect();
        assert_eq!(
            titles,
            [
                "Accessories",
                "Development",
                "Multimedia",
                "System",
                "Loose"
            ]
        );
        let Kind::Group(dev) = &nodes[1].kind else {
            panic!("a group")
        };
        assert_eq!(dev[0].title, "Abc");
        assert_eq!(dev[1].title, "Zed");
        let Kind::Group(system) = &nodes[3].kind else {
            panic!("a group")
        };
        assert_eq!(system[0].kind, Kind::Run("foot -e htop".into()));
    }
}
