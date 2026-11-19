//! The directories of chora: the runtime, config, state and data homes of
//! the user and the one check of a private directory. Empty and relative
//! values do not count, as the XDG specification says, so that no path
//! resolves against the current directory.

use std::{
    ffi::OsString,
    fs, io,
    os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
};

/// The variable of the runtime directory, for a child process or a test
/// that sets it.
pub const RUNTIME_ENV: &str = "XDG_RUNTIME_DIR";

/// The mode of a new private directory.
const PRIVATE_MODE: u32 = 0o700;

/// The group and other bits, which a private directory has none of.
const OTHERS: u32 = 0o077;

/// The group and other write bits, which the runtime directory has none of.
const WRITABLE: u32 = 0o022;

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

/// Creates a directory with mode 0700, with its missing parents, makes
/// an old one of the user that group and other cannot write (0755 of the
/// last release) 0700 and checks it with [`check_private`]. A directory
/// that group or other can write stays as it is and is refused.
///
/// # Errors
///
/// The directory cannot be made, or it is not private.
pub fn private_dir(dir: &Path) -> io::Result<()> {
    fs::DirBuilder::new()
        .recursive(true)
        .mode(PRIVATE_MODE)
        .create(dir)?;
    tighten(dir)?;
    check_private(dir)
}

/// Sets mode 0700 on a directory of the current user that is no symlink
/// and that group and other can read but not write. The mode is set on the
/// directory that was checked, not on what the path names a moment later.
fn tighten(dir: &Path) -> io::Result<()> {
    let meta = fs::symlink_metadata(dir)?;
    let mode = meta.mode() & 0o7777;
    let old = meta.is_dir()
        && meta.uid() == rustix::process::geteuid().as_raw()
        && mode & OTHERS != 0
        && mode & WRITABLE == 0;
    if !old {
        return Ok(());
    }
    let opened = fs::File::open(dir)?;
    let same = opened.metadata()?;
    if same.dev() != meta.dev() || same.ino() != meta.ino() {
        return Err(io::Error::other(format!(
            "{} was replaced while it was checked",
            dir.display()
        )));
    }
    opened.set_permissions(fs::Permissions::from_mode(PRIVATE_MODE))
}

/// The runtime directory `chora/` or another name below it, made private
/// and checked; the runtime directory itself is checked first.
///
/// # Errors
///
/// There is no absolute runtime directory, or one of the checks fails.
pub fn runtime_subdir(name: &str) -> io::Result<PathBuf> {
    runtime_subdir_in(runtime_home(), name)
}

/// [`runtime_subdir`] below a given runtime directory. The directory is
/// resolved once, to its canonical path; the check and the path that is
/// returned are of that one, so that a link cannot be retargeted between
/// the check and the use.
///
/// # Errors
///
/// As [`runtime_subdir`].
pub fn runtime_subdir_in(runtime: Option<PathBuf>, name: &str) -> io::Result<PathBuf> {
    let runtime = runtime
        .filter(|p| p.is_absolute())
        .ok_or_else(|| io::Error::other("no absolute runtime directory in the environment"))?;
    let runtime = fs::canonicalize(&runtime)
        .map_err(|error| io::Error::new(error.kind(), format!("{}: {error}", runtime.display())))?;
    check_runtime_home(&runtime)?;
    let dir = runtime.join(name);
    private_dir(&dir)?;
    Ok(dir)
}

/// Checks the runtime directory itself: a directory of the current user
/// that no group or other can write to, so that none can plant or swap
/// a socket or file below it. A refusal is logged with the path and the
/// mode.
///
/// # Errors
///
/// The directory is missing or one of the checks does not hold.
pub fn check_runtime_home(dir: &Path) -> io::Result<()> {
    let meta = fs::metadata(dir)
        .map_err(|error| io::Error::new(error.kind(), format!("{}: {error}", dir.display())))?;
    let uid = rustix::process::geteuid().as_raw();
    check_runtime_meta(dir, &meta, uid).inspect_err(|error| tracing::warn!("{error}"))
}

fn check_runtime_meta(dir: &Path, meta: &fs::Metadata, uid: u32) -> io::Result<()> {
    let refuse = |reason: &str| {
        Err(io::Error::other(format!(
            "{} is no safe runtime directory: {reason}",
            dir.display()
        )))
    };
    if !meta.is_dir() {
        return refuse("it is not a directory");
    }
    if meta.uid() != uid {
        return refuse(&format!("its owner is user {}, not user {uid}", meta.uid()));
    }
    let mode = meta.mode() & 0o7777;
    if mode & WRITABLE != 0 {
        return refuse(&format!("its mode is {mode:04o}, group or other can write"));
    }
    Ok(())
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
        for mode in [0o700, 0o2700, 0o1700] {
            let dir = base.join(format!("m{mode:o}"));
            fs::create_dir(&dir).expect("the dir");
            fs::set_permissions(&dir, fs::Permissions::from_mode(mode)).expect("the mode");
            private_dir(&dir).unwrap_or_else(|e| panic!("mode {mode:04o} is private: {e}"));
        }
        fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn a_dir_that_the_owner_cannot_write_is_refused_with_path_and_mode() {
        let base = scratch("owner-bits");
        for mode in [0o500, 0o400, 0o100, 0o600, 0o550] {
            let dir = dir_with(&base, &format!("m{mode:o}"), mode);
            let error = private_dir(&dir).expect_err("refused");
            let text = error.to_string();
            assert!(text.contains(&dir.display().to_string()), "{text}");
            assert!(text.contains(&format!("{mode:04o}")), "{text}");
            let kept = fs::metadata(&dir).expect("the dir").permissions().mode();
            assert_eq!(kept & 0o7777, mode, "a refused dir keeps its mode");
        }
        fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn a_runtime_dir_that_the_owner_cannot_write_is_refused_with_path_and_mode() {
        let base = scratch("runtime-owner-bits");
        for mode in [0o500, 0o400, 0o600] {
            let dir = dir_with(&base, &format!("m{mode:o}"), mode);
            let error = check_runtime_home(&dir).expect_err("refused");
            let text = error.to_string();
            assert!(text.contains(&dir.display().to_string()), "{text}");
            assert!(text.contains(&format!("{mode:04o}")), "{text}");
            let error = runtime_subdir_in(Some(dir.clone()), "chora").expect_err("refused");
            assert!(
                error.to_string().contains(&format!("{mode:04o}")),
                "{error}"
            );
        }
        fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn a_dir_that_group_or_other_can_write_is_refused() {
        let base = scratch("bits");
        for mode in [0o770, 0o707, 0o775, 0o757, 0o777] {
            let dir = base.join(format!("m{mode:o}"));
            fs::create_dir(&dir).expect("the dir");
            fs::set_permissions(&dir, fs::Permissions::from_mode(mode)).expect("the mode");
            let error = private_dir(&dir).expect_err("refused");
            assert!(error.to_string().contains("mode"), "{error}");
            let kept = fs::metadata(&dir).expect("the dir").permissions().mode();
            assert_eq!(kept & 0o7777, mode, "a refused dir keeps its mode");
        }
        fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn an_old_dir_without_write_bits_of_others_is_tightened_to_0700() {
        let base = scratch("tighten");
        for mode in [0o755, 0o750, 0o705, 0o751, 0o754, 0o2755] {
            let dir = base.join(format!("m{mode:o}"));
            fs::create_dir(&dir).expect("the dir");
            fs::set_permissions(&dir, fs::Permissions::from_mode(mode)).expect("the mode");
            private_dir(&dir).unwrap_or_else(|e| panic!("mode {mode:04o} is tightened: {e}"));
            let now = fs::metadata(&dir).expect("the dir").permissions().mode();
            assert_eq!(now & 0o7777, 0o700, "from {mode:04o}");
        }
        fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn a_symlink_is_not_tightened() {
        let base = scratch("tighten-link");
        let real = dir_with(&base, "real", 0o755);
        let link = base.join("link");
        symlink(&real, &link).expect("the link");
        assert!(private_dir(&link).is_err());
        let kept = fs::metadata(&real).expect("the dir").permissions().mode();
        assert_eq!(kept & 0o7777, 0o755, "the target of a link keeps its mode");
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

    /// A new directory with a mode.
    fn dir_with(base: &Path, name: &str, mode: u32) -> PathBuf {
        let dir = base.join(name);
        fs::create_dir(&dir).expect("the dir");
        fs::set_permissions(&dir, fs::Permissions::from_mode(mode)).expect("the mode");
        dir
    }

    #[test]
    fn a_runtime_dir_that_others_can_write_is_refused() {
        let base = scratch("runtime-write");
        for mode in [0o777, 0o770, 0o757, 0o722, 0o1777, 0o720] {
            let dir = dir_with(&base, &format!("m{mode:o}"), mode);
            let error = check_runtime_home(&dir).expect_err("refused");
            let text = error.to_string();
            assert!(text.contains(&dir.display().to_string()), "{text}");
            assert!(text.contains(&format!("{mode:04o}")), "{text}");
        }
        fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn a_runtime_dir_of_the_user_is_accepted() {
        let base = scratch("runtime-ok");
        for mode in [0o700, 0o755, 0o750] {
            let dir = dir_with(&base, &format!("m{mode:o}"), mode);
            check_runtime_home(&dir).unwrap_or_else(|e| panic!("mode {mode:04o}: {e}"));
        }
        assert!(check_runtime_home(&base.join("missing")).is_err());
        fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn a_runtime_dir_of_another_owner_is_refused() {
        let base = scratch("runtime-owner");
        let meta = fs::metadata(&base).expect("the metadata");
        let other = meta.uid().wrapping_add(1);
        let error = check_runtime_meta(&base, &meta, other).expect_err("refused");
        let text = error.to_string();
        assert!(text.contains("owner"), "{text}");
        assert!(text.contains(&base.display().to_string()), "{text}");
        fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn a_subdir_is_private_in_a_checked_runtime_dir() {
        let base = scratch("sub-ok");
        let runtime = dir_with(&base, "run", 0o700);
        let dir = runtime_subdir_in(Some(runtime.clone()), "chora").expect("the subdir");
        assert_eq!(dir, runtime.join("chora"));
        let mode = fs::metadata(&dir).expect("the dir").permissions().mode();
        assert_eq!(mode & 0o7777, 0o700);
        assert!(runtime_subdir_in(None, "chora").is_err());
        assert!(runtime_subdir_in(Some("run".into()), "chora").is_err());
        fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn a_symlinked_runtime_dir_is_resolved_once_and_used_below_its_target() {
        let base = scratch("sub-link");
        let real = dir_with(&base, "real", 0o700);
        let link = base.join("link");
        symlink(&real, &link).expect("the link");
        let dir = runtime_subdir_in(Some(link.clone()), "chora").expect("the subdir");
        let target = fs::canonicalize(&real).expect("the target");
        assert_eq!(dir, target.join("chora"), "the path runs below the target");
        assert!(!dir.starts_with(&link), "no part of the path is the link");
        fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn a_link_to_a_runtime_dir_that_others_write_is_refused_by_its_target() {
        let base = scratch("sub-link-open");
        let open = dir_with(&base, "open", 0o777);
        let link = base.join("link");
        symlink(&open, &link).expect("the link");
        let error = runtime_subdir_in(Some(link), "chora").expect_err("refused");
        let target = fs::canonicalize(&open).expect("the target");
        let text = error.to_string();
        assert!(text.contains(&target.display().to_string()), "{text}");
        assert!(text.contains("0777"), "{text}");
        assert!(!open.join("chora").exists(), "nothing is made");
        fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn no_subdir_is_made_in_a_runtime_dir_that_others_write() {
        let base = scratch("sub-open");
        let runtime = dir_with(&base, "run", 0o777);
        let error = runtime_subdir_in(Some(runtime.clone()), "chora").expect_err("refused");
        assert!(error.to_string().contains("0777"), "{error}");
        assert!(!runtime.join("chora").exists(), "nothing is made");
        fs::remove_dir_all(&base).expect("cleanup");
    }

    /// A writer that keeps what a test logs.
    #[derive(Clone, Default)]
    struct Log(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

    impl io::Write for Log {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.lock().expect("the log").extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn the_refusal_of_a_runtime_dir_is_logged_with_path_and_mode() {
        let base = scratch("runtime-log");
        let dir = dir_with(&base, "run", 0o777);
        let log = Log::default();
        let writer = log.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(move || writer.clone())
            .with_ansi(false)
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            assert!(runtime_subdir_in(Some(dir.clone()), "chora").is_err());
        });
        let text = String::from_utf8(log.0.lock().expect("the log").clone()).expect("text");
        assert!(text.contains("WARN"), "{text}");
        assert!(text.contains(&dir.display().to_string()), "{text}");
        assert!(text.contains("0777"), "{text}");
        fs::remove_dir_all(&base).expect("cleanup");
    }

    /// Runs the test `name` again in a child process that has only this
    /// environment, since a test sets no variable of its own process.
    fn in_child(name: &str, env: &[(&str, &Path)]) {
        let output = std::process::Command::new(std::env::current_exe().expect("this test"))
            .args(["--exact", &format!("paths::tests::{name}"), "--nocapture"])
            .env_clear()
            .envs(env.iter().copied())
            .env(CHILD, "1")
            .output()
            .expect("the child");
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    const CHILD: &str = "CHORA_PATHS_TEST_CHILD";

    #[test]
    fn child_reads_the_homes() {
        if std::env::var_os(CHILD).is_none() {
            return;
        }
        assert_eq!(runtime_home(), Some("/run/user/7".into()));
        assert_eq!(config_home(), Some("/cfg".into()));
        assert_eq!(state_home(), Some("/home/a/.local/state".into()));
        assert_eq!(data_home(), Some("/home/a/.local/share".into()));
        assert_eq!(home(), Some("/home/a".into()));
    }

    #[test]
    fn the_homes_read_the_environment() {
        in_child(
            "child_reads_the_homes",
            &[
                (RUNTIME_ENV, Path::new("/run/user/7")),
                ("XDG_CONFIG_HOME", Path::new("/cfg")),
                ("XDG_STATE_HOME", Path::new("")),
                ("HOME", Path::new("/home/a")),
            ],
        );
    }

    #[test]
    fn child_refuses_the_runtime_dir() {
        if std::env::var_os(CHILD).is_none() {
            return;
        }
        let error = runtime_subdir("chora").expect_err("refused");
        assert!(error.to_string().contains("0777"), "{error}");
        let runtime = runtime_home().expect("the runtime dir");
        assert!(!runtime.join("chora").exists());
    }

    #[test]
    fn the_wrapper_of_the_runtime_dir_refuses_a_world_writable_one() {
        let base = scratch("wrapper-open");
        let runtime = dir_with(&base, "run", 0o777);
        in_child("child_refuses_the_runtime_dir", &[(RUNTIME_ENV, &runtime)]);
        fs::remove_dir_all(&base).expect("cleanup");
    }
}
