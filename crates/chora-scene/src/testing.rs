//! Helpers for the tests of this crate and of the crates above it; behind
//! the `testing` feature outside this crate's own tests.
#![allow(
    clippy::unwrap_used,
    clippy::missing_panics_doc,
    reason = "a test helper: a failure is a failed test"
)]

use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

/// A new, empty folder in the temporary directory for one test: named by
/// `name`, the process and a counter of the process, so that two folders
/// of one name never clash, in one process or between two. The folder
/// and what is in it go when the value goes, also after a failed test.
///
/// The one copy: the kit re-exports it as `chora_kit::testing::TempDir`.
pub struct TempDir(PathBuf);

impl TempDir {
    /// Makes the folder `name-<pid>-<n>` in the temporary directory.
    ///
    /// # Panics
    ///
    /// The folder cannot be made.
    #[must_use]
    pub fn new(name: &str) -> Self {
        let dir = Self::unmade(name);
        std::fs::create_dir_all(&dir.0).unwrap();
        dir
    }

    /// Takes the name of a folder as `new` does but does not make it,
    /// for a test whose code under test makes it. What is there when the
    /// value goes is removed.
    #[must_use]
    pub fn unmade(name: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("{name}-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        Self(path)
    }

    /// Gives up the guard: the folder stays after the test. For a helper
    /// that hands the folder to an object that outlives the helper.
    #[must_use]
    pub fn keep(self) -> PathBuf {
        let path = self.0.clone();
        std::mem::forget(self);
        path
    }

    /// The path of the folder.
    #[must_use]
    pub fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl std::ops::Deref for TempDir {
    type Target = std::path::Path;

    fn deref(&self) -> &std::path::Path {
        &self.0
    }
}

impl AsRef<std::path::Path> for TempDir {
    fn as_ref(&self) -> &std::path::Path {
        &self.0
    }
}

impl AsRef<std::ffi::OsStr> for TempDir {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.0.as_os_str()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::TempDir;

    /// Two folders of one name in one process never clash, and each goes
    /// when its owner goes.
    #[test]
    fn same_name_twice_gives_two_folders_removed_on_drop() {
        let first = TempDir::new("chora-temp-twice");
        let second = TempDir::new("chora-temp-twice");
        assert_ne!(first.path(), second.path());
        assert!(first.path().is_dir() && second.path().is_dir());
        let (a, b) = (first.path().to_path_buf(), second.path().to_path_buf());
        std::fs::write(a.join("file"), b"x").unwrap();
        drop(first);
        assert!(!a.exists(), "the first folder is removed on drop");
        assert!(b.is_dir(), "the second folder stays while it lives");
        drop(second);
        assert!(!b.exists(), "the second folder is removed on drop");
    }
}
