//! A counted temporary folder for one test. A copy of
//! `chora_kit::testing::TempDir` for a crate that cannot depend on the
//! kit; keep the two the same (tools/copies.toml, "temp folder by pid").
#![allow(
    dead_code,
    clippy::expect_used,
    reason = "a test helper: a failure is a failed test"
)]

use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

/// A new, empty folder `name-<pid>-<n>` in the temporary directory; the
/// folder and what is in it go when the value goes.
pub struct TempDir(PathBuf);

impl TempDir {
    /// Makes the folder.
    pub fn new(name: &str) -> Self {
        let dir = Self::unmade(name);
        std::fs::create_dir_all(&dir.0).expect("a temporary folder");
        dir
    }

    /// Takes the name of the folder as `new` does, without making it.
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

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl std::ops::Deref for TempDir {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for TempDir {
    fn as_ref(&self) -> &Path {
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
