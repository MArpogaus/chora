//! Where HUD folders live: the folders of the user in
//! `$XDG_DATA_HOME/chora/huds`, the shipped ones in `chora/huds` of each
//! directory of `$XDG_DATA_DIRS` and, for a binary run from
//! `target/<profile>/`, those of the host in the source tree. A folder of
//! the user wins over a shipped one of the same name.

use std::path::{Path, PathBuf};

use chora_kit::xdg;

/// The folders of HUDs below a data directory.
const BELOW: &str = "chora/huds";

/// The data directories of the system when `$XDG_DATA_DIRS` names none.
const DATA_DIRS: &str = "/usr/local/share:/usr/share";

/// The folders of the source tree that hold HUDs, below its root.
const SOURCE: &[&str] = &["plugins/chora-assistant/huds"];

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

/// The folder `name`: the first of the directories that has it.
#[must_use]
pub fn find(roots: &[PathBuf], name: &str) -> Option<PathBuf> {
    if !good_name(name) {
        return None;
    }
    roots
        .iter()
        .map(|root| root.join(name))
        .find(|dir| dir.join("hud.yaml").is_file())
}

/// The folders by name, sorted; each name once, with the folder that wins.
#[must_use]
pub fn list(roots: &[PathBuf]) -> Vec<(String, PathBuf)> {
    let mut names: Vec<String> = roots
        .iter()
        .flat_map(|root| std::fs::read_dir(root).into_iter().flatten().flatten())
        .filter_map(|entry| entry.file_name().into_string().ok())
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

    fn tree(test: &str, names: &[(&str, &str)]) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("chora-assistant-{test}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
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
            without + 1
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
    fn the_source_tree_has_the_folder_of_each_look() {
        let own = Path::new(env!("CARGO_MANIFEST_DIR")).join("huds");
        let names: Vec<String> = list(&[own]).into_iter().map(|(name, _)| name).collect();
        assert_eq!(names, ["construct", "mc", "neutral", "xp"]);
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
