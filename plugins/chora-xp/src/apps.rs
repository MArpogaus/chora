//! The programs of All Programs: the applications of the XDG desktop
//! files, in program groups by their main category, as on XP.

use chora_kit::desktop_entry::{self, Entry, command};

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

/// The title of the plain text editor, as on XP.
pub const NOTEPAD: &str = "Notepad";

/// The group of the plain text editor, as on XP.
const ACCESSORIES: &str = "Accessories";

/// The plain text editors with a window of their own, the first that is
/// installed serves.
const EDITORS: [&str; 11] = [
    "gnome-text-editor",
    "gedit",
    "mousepad",
    "kwrite",
    "kate",
    "xed",
    "pluma",
    "featherpad",
    "leafpad",
    "l3afpad",
    "geany",
];

/// The editors of the terminal that serve when `VISUAL` and `EDITOR`
/// name none.
const TERMINAL_EDITORS: [&str; 3] = ["nano", "vim", "vi"];

/// The applications of the desktop files in the XDG data directories
/// whose programs are installed, in groups, and Notepad, the plain text
/// editor, among the accessories. `terminal` runs the programs that need
/// one, `editor` is the setting of that name.
pub fn load(terminal: &str, editor: Option<&str>) -> Vec<Node> {
    let on_path = desktop_entry::on_path;
    let installed = desktop_entry::applications()
        .into_iter()
        .filter(|e| e.installed(on_path));
    let mut nodes = group(installed, terminal);
    let env = |name: &str| std::env::var(name).ok();
    if let Some(command) = self::editor(editor, terminal, env, on_path) {
        add_notepad(&mut nodes, command);
    }
    nodes
}

/// The shell command of the plain text editor: the setting `editor` when
/// its program is installed, else the first installed editor with a
/// window, else the editor of `VISUAL` or `EDITOR`, nano, vim or vi in the
/// terminal. None when none of them is installed.
pub fn editor(
    configured: Option<&str>,
    terminal: &str,
    env: impl Fn(&str) -> Option<String>,
    on_path: impl Fn(&str) -> bool,
) -> Option<String> {
    let installed = |command: &str| desktop_entry::installed(command, &on_path);
    if let Some(command) = configured.filter(|c| installed(c)) {
        return Some(command.to_owned());
    }
    if let Some(program) = EDITORS.iter().find(|p| on_path(p)) {
        return Some((*program).to_owned());
    }
    let named = ["VISUAL", "EDITOR"].into_iter().filter_map(env);
    named
        .chain(TERMINAL_EDITORS.map(String::from))
        .find(|command| installed(command))
        .map(|command| {
            let program = crate::launch::program_name(&command);
            let windowed = EDITORS.contains(&program.as_str());
            desktop_entry::command(command, !windowed, terminal)
        })
}

/// Puts Notepad with its command among the accessories, in a new group
/// when no program is one, unless a program has that title already.
fn add_notepad(nodes: &mut Vec<Node>, command: String) {
    let notepad = Node {
        title: NOTEPAD.into(),
        icon: "accessories-text-editor".into(),
        kind: Kind::Run(command),
    };
    let group = nodes.iter_mut().find_map(|n| match &mut n.kind {
        Kind::Group(inner) if n.title == ACCESSORIES => Some(inner),
        _ => None,
    });
    match group {
        Some(inner) if inner.iter().any(|n| n.title == NOTEPAD) => {}
        Some(inner) => {
            inner.push(notepad);
            sort(inner);
        }
        None => {
            let groups = nodes
                .iter()
                .take_while(|n| matches!(n.kind, Kind::Group(_)) && n.title.as_str() < ACCESSORIES)
                .count();
            nodes.insert(
                groups,
                Node {
                    title: ACCESSORIES.into(),
                    icon: GROUP_ICON.into(),
                    kind: Kind::Group(vec![notepad]),
                },
            );
        }
    }
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

fn sort(nodes: &mut [Node]) {
    nodes.sort_by_cached_key(|n| n.title.to_lowercase());
}

#[cfg(test)]
mod tests {
    use chora_kit::desktop_entry::Entry;

    use super::*;

    #[test]
    fn notepad_is_the_configured_editor_or_the_first_that_is_installed() {
        let none = |_: &str| None;
        let on = |list: &'static [&'static str]| move |p: &str| list.contains(&p);
        assert_eq!(
            editor(Some("pad --plain"), "foot", none, on(&["pad", "gedit"])),
            Some("pad --plain".into())
        );
        assert_eq!(
            editor(
                Some("pad --plain"),
                "foot",
                none,
                on(&["kate", "gedit", "nano"])
            ),
            Some("gedit".into())
        );
        assert_eq!(
            editor(None, "foot", none, on(&["nano", "vi"])),
            Some("foot -e nano".into())
        );
        let env = |name: &str| (name == "EDITOR").then(|| "hx -w".to_owned());
        assert_eq!(
            editor(None, "foot", env, on(&["hx", "nano"])),
            Some("foot -e hx -w".into())
        );
        let env = |name: &str| (name == "VISUAL").then(|| "kate".to_owned());
        assert_eq!(
            editor(None, "foot", env, on(&["nano"])),
            Some("foot -e nano".into())
        );
        assert_eq!(editor(None, "foot", none, on(&[])), None);
        assert_eq!(editor(Some("pad"), "foot", none, on(&[])), None);
    }

    #[test]
    fn notepad_stands_among_the_accessories() {
        let run = |title: &str| Node {
            title: title.into(),
            icon: String::new(),
            kind: Kind::Run(title.to_lowercase()),
        };
        let group = |title: &str, inner: Vec<Node>| Node {
            title: title.into(),
            icon: GROUP_ICON.into(),
            kind: Kind::Group(inner),
        };
        let titles = |nodes: &[Node]| nodes.iter().map(|n| n.title.clone()).collect::<Vec<_>>();
        let mut nodes = vec![
            group("Accessories", vec![run("Calc"), run("Zip")]),
            run("Loose"),
        ];
        add_notepad(&mut nodes, "gedit".into());
        let Kind::Group(inner) = &nodes[0].kind else {
            panic!("a group")
        };
        assert_eq!(titles(inner), ["Calc", "Notepad", "Zip"]);
        assert_eq!(inner[1].kind, Kind::Run("gedit".into()));
        add_notepad(&mut nodes, "kate".into());
        let Kind::Group(inner) = &nodes[0].kind else {
            panic!("a group")
        };
        assert_eq!(inner.len(), 3);
        let mut nodes = vec![group("Games", vec![run("Mines")]), run("Loose")];
        add_notepad(&mut nodes, "gedit".into());
        assert_eq!(titles(&nodes), ["Accessories", "Games", "Loose"]);
        let mut nodes = Vec::new();
        add_notepad(&mut nodes, "gedit".into());
        assert_eq!(titles(&nodes), ["Accessories"]);
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
