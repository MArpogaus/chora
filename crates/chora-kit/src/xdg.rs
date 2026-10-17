//! The XDG base directories.

use std::path::{Path, PathBuf};

/// The XDG data directories: `$XDG_DATA_HOME` first, then
/// `$XDG_DATA_DIRS`.
#[must_use]
pub fn data_dirs() -> Vec<PathBuf> {
    let home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| Path::new(&h).join(".local/share")));
    let dirs = std::env::var("XDG_DATA_DIRS")
        .ok()
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".into());
    home.into_iter()
        .chain(dirs.split(':').map(PathBuf::from))
        .collect()
}

/// The first of the XDG data directories that has a file, for example
/// `wallpapers/WindowsXP/bliss.jpg`.
#[must_use]
pub fn data_file(file: &str) -> Option<PathBuf> {
    data_dirs()
        .into_iter()
        .map(|d| d.join(file))
        .find(|p| p.is_file())
}
