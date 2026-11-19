//! The XDG base directories. Empty and relative values do not count, as
//! the specification says, so that no path resolves against the current
//! directory.

use std::{ffi::OsString, path::PathBuf};

/// The folder of imported XP assets, laid out like a data directory
/// (`icons/WindowsXP`, `fonts/WindowsXP`, `wallpapers/WindowsXP`, and
/// `start-mark.png`): `$CHORA_XP_ASSETS`, when it is an absolute path.
#[must_use]
pub fn xp_assets() -> Option<PathBuf> {
    assets_of(std::env::var_os("CHORA_XP_ASSETS"))
}

/// The XDG data directories: the XP assets folder first, when set, then
/// the data home, then `$XDG_DATA_DIRS`.
#[must_use]
pub fn data_dirs() -> Vec<PathBuf> {
    xp_assets()
        .into_iter()
        .chain(data_home())
        .chain(dirs_of(std::env::var_os("XDG_DATA_DIRS")))
        .collect()
}

fn assets_of(value: Option<OsString>) -> Option<PathBuf> {
    value.map(PathBuf::from).filter(|p| p.is_absolute())
}

/// `$XDG_DATA_HOME`, else `$HOME/.local/share`; `None` without an
/// absolute one.
#[must_use]
pub fn data_home() -> Option<PathBuf> {
    chora_scene::paths::data_home()
}

/// `$XDG_CONFIG_HOME`, else `$HOME/.config`; `None` without an absolute
/// one.
#[must_use]
pub fn config_home() -> Option<PathBuf> {
    chora_scene::paths::config_home()
}

/// The first of the XDG data directories that has a file, for example
/// `icons/hicolor/index.theme`.
#[must_use]
pub fn data_file(file: &str) -> Option<PathBuf> {
    data_dirs()
        .into_iter()
        .map(|d| d.join(file))
        .find(|p| p.is_file())
}

fn dirs_of(value: Option<OsString>) -> Vec<PathBuf> {
    let dirs: Vec<PathBuf> = value
        .map(|v| {
            std::env::split_paths(&v)
                .filter(|p| p.is_absolute())
                .collect()
        })
        .unwrap_or_default();
    if dirs.is_empty() {
        return vec!["/usr/local/share".into(), "/usr/share".into()];
    }
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_relative_directories_do_not_count() {
        let some = |s: &str| Some(OsString::from(s));
        assert_eq!(
            dirs_of(some("/a::b:/c")),
            [PathBuf::from("/a"), "/c".into()]
        );
        assert_eq!(dirs_of(some("")), dirs_of(None));
        assert_eq!(dirs_of(None)[1], PathBuf::from("/usr/share"));
    }

    #[test]
    fn the_assets_folder_is_absolute_or_ignored() {
        let some = |s: &str| Some(OsString::from(s));
        assert_eq!(assets_of(some("/a")), Some("/a".into()));
        assert_eq!(assets_of(some("a")), None);
        assert_eq!(assets_of(some("")), None);
        assert_eq!(assets_of(None), None);
    }
}
