//! The directories of chora: the runtime, config, state and data homes of
//! the user and the one check of a private directory. Empty and relative
//! values do not count, as the XDG specification says, so that no path
//! resolves against the current directory.

use std::{
    ffi::OsString,
    fs, io,
    os::unix::fs::{DirBuilderExt, MetadataExt},
    path::{Path, PathBuf},
};

/// The variable of the runtime directory, for a child process or a test
/// that sets it.
pub const RUNTIME_ENV: &str = "XDG_RUNTIME_DIR";

/// The mode of a new private directory.
const PRIVATE_MODE: u32 = 0o700;

/// The group and other bits, which a private directory has none of.
const OTHERS: u32 = 0o077;

/// `$XDG_RUNTIME_DIR`; `None` unless it is an absolute path.
#[must_use]
pub fn runtime_home() -> Option<PathBuf> {
    absolute(std::env::var_os(RUNTIME_ENV))
}

/// `$XDG_CONFIG_HOME`, else `$HOME/.config`; `None` without an absolute
/// one.
#[must_use]
pub fn config_home() -> Option<PathBuf> {
    home_of("XDG_CONFIG_HOME", ".config")
}

/// `$XDG_STATE_HOME`, else `$HOME/.local/state`; `None` without an
/// absolute one.
#[must_use]
pub fn state_home() -> Option<PathBuf> {
    home_of("XDG_STATE_HOME", ".local/state")
}

/// `$XDG_DATA_HOME`, else `$HOME/.local/share`; `None` without an
/// absolute one.
#[must_use]
pub fn data_home() -> Option<PathBuf> {
    home_of("XDG_DATA_HOME", ".local/share")
}

/// `$HOME`; `None` unless it is an absolute path.
#[must_use]
pub fn home() -> Option<PathBuf> {
    absolute(std::env::var_os("HOME"))
}

fn home_of(xdg: &str, below: &str) -> Option<PathBuf> {
    resolve(std::env::var_os(xdg), std::env::var_os("HOME"), below)
}

fn resolve(xdg: Option<OsString>, home: Option<OsString>, below: &str) -> Option<PathBuf> {
    absolute(xdg).or_else(|| absolute(home).map(|h| h.join(below)))
}

fn absolute(value: Option<OsString>) -> Option<PathBuf> {
    value.map(PathBuf::from).filter(|p| p.is_absolute())
}

/// Creates a directory with mode 0700, with its missing parents, and
/// checks it with [`check_private`].
///
/// # Errors
///
/// The directory cannot be made, or it is not private.
pub fn private_dir(dir: &Path) -> io::Result<()> {
    fs::DirBuilder::new()
        .recursive(true)
        .mode(PRIVATE_MODE)
        .create(dir)?;
    check_private(dir)
}

/// Checks that a directory belongs to the current user, has no group or
/// other bit (0700, but also 02700 or 0500) and is no symlink. A refusal is logged with its reason.
///
/// # Errors
///
/// The directory is missing or one of the three does not hold.
pub fn check_private(dir: &Path) -> io::Result<()> {
    let meta = fs::symlink_metadata(dir)?;
    let uid = rustix::process::geteuid().as_raw();
    check_meta(dir, &meta, uid).inspect_err(|error| tracing::warn!("{error}"))
}

fn check_meta(dir: &Path, meta: &fs::Metadata, uid: u32) -> io::Result<()> {
    let refuse = |reason: &str| {
        Err(io::Error::other(format!(
            "{} is no private directory: {reason}",
            dir.display()
        )))
    };
    if meta.file_type().is_symlink() {
        return refuse("it is a symlink");
    }
    if !meta.is_dir() {
        return refuse("it is not a directory");
    }
    if meta.uid() != uid {
        return refuse(&format!("its owner is user {}, not user {uid}", meta.uid()));
    }
    let mode = meta.mode() & 0o7777;
    if mode & OTHERS != 0 {
        return refuse(&format!(
            "its mode is {mode:04o}, it has group or other bits"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::{PermissionsExt, symlink};

    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("chora-paths-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("a scratch directory");
        dir
    }

    #[test]
    fn an_empty_or_relative_value_does_not_count() {
        assert_eq!(absolute(Some("".into())), None);
        assert_eq!(absolute(Some("run/user".into())), None);
        assert_eq!(
            absolute(Some("/run/user/1".into())),
            Some("/run/user/1".into())
        );
        assert_eq!(absolute(None), None);
    }

    #[test]
    fn the_xdg_value_wins_and_home_is_the_fallback() {
        let config = resolve(Some("/cfg".into()), Some("/home/a".into()), ".config");
        assert_eq!(config, Some("/cfg".into()));
        let below = resolve(Some("".into()), Some("/home/a".into()), ".config");
        assert_eq!(below, Some("/home/a/.config".into()));
        let relative = resolve(Some("cfg".into()), Some("home".into()), ".config");
        assert_eq!(relative, None);
    }

    #[test]
    fn a_new_private_dir_has_mode_0700() {
        let base = scratch("new");
        let dir = base.join("a/b");
        private_dir(&dir).expect("a new private dir");
        let mode = fs::metadata(&dir).expect("the dir").permissions().mode();
        assert_eq!(mode & 0o7777, 0o700);
        private_dir(&dir).expect("an old private dir stays");
        fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn a_dir_without_group_or_other_bits_is_accepted() {
        let base = scratch("strict");
        for mode in [0o700, 0o2700, 0o500, 0o1700] {
            let dir = base.join(format!("m{mode:o}"));
            fs::create_dir(&dir).expect("the dir");
            fs::set_permissions(&dir, fs::Permissions::from_mode(mode)).expect("the mode");
            private_dir(&dir).unwrap_or_else(|e| panic!("mode {mode:04o} is private: {e}"));
        }
        fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn a_dir_with_group_or_other_bits_is_refused() {
        let base = scratch("bits");
        for mode in [0o770, 0o750, 0o707, 0o755] {
            let dir = base.join(format!("m{mode:o}"));
            fs::create_dir(&dir).expect("the dir");
            fs::set_permissions(&dir, fs::Permissions::from_mode(mode)).expect("the mode");
            let error = private_dir(&dir).expect_err("refused");
            assert!(error.to_string().contains("mode"), "{error}");
        }
        fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn a_symlink_is_refused_also_to_a_private_dir() {
        let base = scratch("link");
        let real = base.join("real");
        private_dir(&real).expect("the real dir");
        let link = base.join("link");
        symlink(&real, &link).expect("the link");
        let error = private_dir(&link).expect_err("refused");
        assert!(error.to_string().contains("symlink"), "{error}");
        fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn a_dir_of_another_owner_is_refused() {
        let base = scratch("owner");
        private_dir(&base.join("d")).expect("the dir");
        let meta = fs::symlink_metadata(base.join("d")).expect("the metadata");
        let other = meta.uid().wrapping_add(1);
        let error = check_meta(&base.join("d"), &meta, other).expect_err("refused");
        assert!(error.to_string().contains("owner"), "{error}");
        fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn a_file_is_no_private_dir() {
        let base = scratch("file");
        let file = base.join("f");
        fs::write(&file, b"x").expect("the file");
        let error = check_private(&file).expect_err("refused");
        assert!(error.to_string().contains("directory"), "{error}");
        fs::remove_dir_all(&base).expect("cleanup");
    }
}
