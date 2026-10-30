//! The XDG base directories. Empty and relative values do not count, as
//! the specification says, so that no path resolves against the current
//! directory.

use std::{ffi::OsString, path::PathBuf};

/// The XDG data directories: the data home first, then `$XDG_DATA_DIRS`.
#[must_use]
pub fn data_dirs() -> Vec<PathBuf> {
    data_home()
        .into_iter()
        .chain(dirs_of(std::env::var_os("XDG_DATA_DIRS")))
        .collect()
}

/// `$XDG_DATA_HOME`, else `$HOME/.local/share`; `None` without an
/// absolute one.
#[must_use]
pub fn data_home() -> Option<PathBuf> {
    home_of(
        std::env::var_os("XDG_DATA_HOME"),
        std::env::var_os("HOME"),
        ".local/share",
    )
}

/// `$XDG_CONFIG_HOME`, else `$HOME/.config`; `None` without an absolute
/// one.
#[must_use]
pub fn config_home() -> Option<PathBuf> {
    home_of(
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
        ".config",
    )
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

fn home_of(xdg: Option<OsString>, home: Option<OsString>, below: &str) -> Option<PathBuf> {
    xdg.map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| {
            home.map(PathBuf::from)
                .filter(|p| p.is_absolute())
                .map(|h| h.join(below))
        })
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
    fn empty_and_relative_values_do_not_count() {
        let some = |s: &str| Some(OsString::from(s));
        let share = ".local/share";
        assert_eq!(home_of(some("/d"), some("/h"), share), Some("/d".into()));
        assert_eq!(
            home_of(some(""), some("/h"), share),
            Some("/h/.local/share".into())
        );
        assert_eq!(
            home_of(some("d"), some("/h"), share),
            Some("/h/.local/share".into())
        );
        assert_eq!(home_of(some(""), some(""), share), None);
        assert_eq!(home_of(None, None, share), None);
        assert_eq!(
            dirs_of(some("/a::b:/c")),
            [PathBuf::from("/a"), "/c".into()]
        );
        assert_eq!(dirs_of(some("")), dirs_of(None));
        assert_eq!(dirs_of(None)[1], PathBuf::from("/usr/share"));
    }
}
