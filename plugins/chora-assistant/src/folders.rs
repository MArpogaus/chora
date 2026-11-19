//! Where HUD folders live: the folders of the user in
//! `$XDG_DATA_HOME/chora/huds`, the shipped ones in `chora/huds` of each
//! directory of `$XDG_DATA_DIRS` and, for a binary run from
//! `target/<profile>/`, those of the source tree: the neutral folder of the
//! host and the folder of each look in the directory of that look. A folder
//! of the user wins over a shipped one of the same name.

use std::path::{Path, PathBuf};

use chora_kit::xdg;

/// The folders of HUDs below a data directory.
const BELOW: &str = "chora/huds";

/// The data directories of the system when `$XDG_DATA_DIRS` names none.
const DATA_DIRS: &str = "/usr/local/share:/usr/share";

/// The places of HUD folders in the source tree, below its root: a path
/// that holds a folder for each name, and paths with `{}` where the name of
/// the look stands, each of which is the folder of that look.
const SOURCE: &[&str] = &[
    "plugins/chora-assistant/huds",
    "plugins/chora-{}/hud",
    "flavors/{}/assets/assistant",
];

/// The mark in a root where the name of the folder stands.
const NAME_MARK: &str = "{}";

/// The most characters of the name of a folder.
const MAX_NAME: usize = 64;

/// Whether `name` is a name that a setting may give to a folder: it never
/// leaves the folder of the HUDs.
#[must_use]
pub fn good_name(name: &str) -> bool {
    (1..=MAX_NAME).contains(&name.len())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// The directories that hold HUD folders, the first one wins: the user's,
/// those of `$XDG_DATA_DIRS`, the source tree of the executable.
#[must_use]
pub fn roots(exe: Option<&Path>) -> Vec<PathBuf> {
    let dirs = std::env::var("XDG_DATA_DIRS").unwrap_or_default();
    roots_in(exe, xdg::data_home().as_deref(), &dirs)
}

/// [`roots`] for a folder of the user and the list `$XDG_DATA_DIRS`.
#[must_use]
pub fn roots_in(exe: Option<&Path>, data_home: Option<&Path>, data_dirs: &str) -> Vec<PathBuf> {
    let user = data_home.map(|home| home.join(BELOW));
    let listed = if data_dirs.is_empty() {
        DATA_DIRS
    } else {
        data_dirs
    };
    let shipped = listed
        .split(':')
        .filter(|dir| dir.starts_with('/'))
        .map(|dir| Path::new(dir).join(BELOW));
    let tree = exe
        .and_then(Path::parent)
        .and_then(Path::parent)
        .filter(|target| target.file_name().is_some_and(|name| name == "target"))
        .and_then(Path::parent)
        .map(Path::to_path_buf);
    let source = tree.into_iter().flat_map(|root| {
        SOURCE
            .iter()
            .map(move |below| root.join(below))
            .collect::<Vec<_>>()
    });
    user.into_iter().chain(shipped).chain(source).collect()
}

/// Where the folder `name` would be below a root: in the root, or at the
/// mark of a root that has one.
fn place(root: &Path, name: &str) -> PathBuf {
    let text = root.to_string_lossy();
    if text.contains(NAME_MARK) {
        PathBuf::from(text.replace(NAME_MARK, name))
    } else {
        root.join(name)
    }
}

/// The names that a root may hold: its entries, or for a root with the
/// mark those entries of the directory of the mark that fit its text.
fn names_in(root: &Path) -> Vec<String> {
    let text = root.to_string_lossy();
    let Some((before, _)) = text.split_once(NAME_MARK) else {
        return entries(root);
    };
    let (parent, stem) = before.rsplit_once('/').unwrap_or((".", before));
    entries(Path::new(parent))
        .into_iter()
        .filter_map(|entry| entry.strip_prefix(stem).map(str::to_owned))
        .collect()
}

fn entries(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect()
}

/// The folder `name`: the first of the directories that has it.
#[must_use]
pub fn find(roots: &[PathBuf], name: &str) -> Option<PathBuf> {
    if !good_name(name) {
        return None;
    }
    roots
        .iter()
        .map(|root| place(root, name))
        .find(|dir| dir.join("hud.yaml").is_file())
}

/// The folders by name, sorted; each name once, with the folder that wins.
#[must_use]
pub fn list(roots: &[PathBuf]) -> Vec<(String, PathBuf)> {
    let mut names: Vec<String> = roots
        .iter()
        .flat_map(|root| names_in(root))
        .filter(|name| good_name(name))
        .collect();
    names.sort();
    names.dedup();
    names
        .into_iter()
        .filter_map(|name| find(roots, &name).map(|dir| (name, dir)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(test: &str, names: &[(&str, &str)]) -> chora_kit::testing::TempDir {
        let root = chora_kit::testing::TempDir::unmade(&format!("chora-assistant-{test}"));
        for (dir, name) in names {
            std::fs::create_dir_all(root.join(dir).join(name)).unwrap();
            std::fs::write(root.join(dir).join(name).join("hud.yaml"), "hud: 1\n").unwrap();
        }
        root
    }

    #[test]
    fn names_never_leave_the_folder_of_huds() {
        for bad in ["", ".", "..", "a/b", "../x", "a b", "a.b", "é"] {
            assert!(!good_name(bad), "{bad:?}");
        }
        for good in ["neutral", "xp", "my_hud-2", "A"] {
            assert!(good_name(good), "{good:?}");
        }
        assert!(!good_name(&"a".repeat(65)));
    }

    #[test]
    fn a_folder_of_the_user_wins_over_a_shipped_one() {
        let root = tree(
            "wins",
            &[
                ("user", "neutral"),
                ("user", "mine"),
                ("shipped", "neutral"),
                ("shipped", "xp"),
            ],
        );
        let roots = vec![root.join("user"), root.join("shipped")];
        assert_eq!(find(&roots, "neutral"), Some(root.join("user/neutral")));
        assert_eq!(find(&roots, "xp"), Some(root.join("shipped/xp")));
        assert_eq!(find(&roots, "nothing"), None);
        assert_eq!(find(&roots, "../user/mine"), None);
        let names: Vec<String> = list(&roots).into_iter().map(|(name, _)| name).collect();
        assert_eq!(names, ["mine", "neutral", "xp"]);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn the_source_tree_counts_only_for_a_binary_in_a_target_folder() {
        let tree = |exe: &str| roots(Some(Path::new(exe))).len();
        let without = tree("/nonexistent/bin/chora-assistant");
        assert_eq!(
            tree("/nonexistent/x/target/debug/chora-assistant"),
            without + SOURCE.len()
        );
        assert_eq!(tree("/nonexistent/x/other/debug/chora-assistant"), without);
    }

    #[test]
    fn the_data_dirs_of_the_system_hold_shipped_folders() {
        let found = roots_in(
            None,
            Some(Path::new("/home/u/.local/share")),
            "/usr/local/share:/usr/share",
        );
        assert_eq!(
            found,
            [
                PathBuf::from("/home/u/.local/share/chora/huds"),
                PathBuf::from("/usr/local/share/chora/huds"),
                PathBuf::from("/usr/share/chora/huds"),
            ]
        );
        let none = roots_in(None, None, "");
        assert_eq!(
            none,
            [
                PathBuf::from("/usr/local/share/chora/huds"),
                PathBuf::from("/usr/share/chora/huds"),
            ]
        );
        let relative = roots_in(None, None, "share:/opt/share");
        assert_eq!(relative, [PathBuf::from("/opt/share/chora/huds")]);
    }

    #[test]
    fn the_assistant_ships_only_the_neutral_folder() {
        let own = Path::new(env!("CARGO_MANIFEST_DIR")).join("huds");
        let mut names: Vec<String> = std::fs::read_dir(own)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        assert_eq!(names, ["neutral"], "a look ships the folder of its costume");
    }

    #[test]
    fn the_source_tree_has_the_folder_of_each_look() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let exe = repo.join("target/debug/chora-assistant");
        let found = roots(Some(&exe));
        let names: Vec<String> = ["construct", "mc", "neutral", "tiling", "xp"]
            .into_iter()
            .filter(|name| find(&found, name).is_some())
            .map(str::to_owned)
            .collect();
        assert_eq!(names, ["construct", "mc", "neutral", "tiling", "xp"]);
        let tiling = find(&found, "tiling").unwrap().canonicalize().unwrap();
        assert!(
            tiling.ends_with("hud") && !tiling.starts_with(repo.join("plugins/chora-assistant")),
            "{}",
            tiling.display()
        );
    }

    #[test]
    fn a_fifth_look_gets_a_costume_from_its_own_directory() {
        let source = tree("fifth", &[]);
        let own = source.join("plugins/chora-fifth/hud");
        std::fs::create_dir_all(&own).unwrap();
        std::fs::write(own.join("hud.yaml"), "hud: 1\n").unwrap();
        let exe = source.join("target/debug/chora-assistant");
        let found = roots(Some(&exe));
        assert_eq!(
            find(&found, "fifth").and_then(|dir| dir.canonicalize().ok()),
            own.canonicalize().ok()
        );
        let _ = std::fs::remove_dir_all(source);
    }

    #[test]
    fn a_directory_without_hud_yaml_is_no_folder() {
        let root = tree("empty", &[]);
        std::fs::create_dir_all(root.join("user/bare")).unwrap();
        let roots = vec![root.join("user")];
        assert_eq!(find(&roots, "bare"), None);
        assert_eq!(list(&roots).len(), 0);
        let _ = std::fs::remove_dir_all(root);
    }
}
