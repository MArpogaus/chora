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

/// The applications of the desktop files in the XDG data directories, in
/// groups. `terminal` runs the programs that need one.
pub fn load(terminal: &str) -> Vec<Node> {
    group(desktop_entry::applications().into_iter(), terminal)
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
