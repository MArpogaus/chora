//! The log files of chora: `$XDG_STATE_HOME/chora/logs/NAME.log`, else
//! `$HOME/.local/state/chora/logs/NAME.log`. A file grows to
//! [`MAX_BYTES`]; then it moves to `NAME.log.1`, the older ones move one
//! number up, and [`KEEP`] old files stay. A start moves the file of the
//! last run to `NAME.log.1`, so a crash stays next to the new run. Only
//! the process that holds the lock `NAME.lock` writes and moves the
//! files; a second process gets no file. The folder has mode 0700 and
//! the files 0600, also when they were there; a file that is a symlink
//! is not opened.

use std::{
    fs,
    io::{self, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};

/// The size at which a log file moves to `NAME.log.1`.
pub const MAX_BYTES: u64 = 4 << 20;

/// The number of old files of a log: `NAME.log.1` to `NAME.log.KEEP`.
pub const KEEP: usize = 3;

/// The folder of the log files; `None` without an absolute
/// `$XDG_STATE_HOME` or `$HOME`.
#[must_use]
pub fn dir() -> Option<PathBuf> {
    super::paths::state_home().map(|state| state.join("chora/logs"))
}

/// A log file that moves to its old files at [`MAX_BYTES`].
pub struct File {
    path: PathBuf,
    handle: fs::File,
    _lock: fs::File,
    size: u64,
    max: u64,
}

impl File {
    /// Opens `NAME.log` in [`dir`] for a new run.
    ///
    /// # Errors
    ///
    /// Without a folder, when the folder or the file cannot be made, or
    /// when another process holds `NAME.lock`.
    pub fn open(name: &str) -> io::Result<Self> {
        let dir = dir().ok_or_else(|| io::Error::other("no $XDG_STATE_HOME and no $HOME"))?;
        Self::open_in(&dir, name, MAX_BYTES)
    }

    fn open_in(dir: &Path, name: &str, max: u64) -> io::Result<Self> {
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(dir)?;
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
        let lock = create(&dir.join(format!("{name}.lock")))?;
        match lock.try_lock() {
            Ok(()) => {}
            Err(fs::TryLockError::WouldBlock) => {
                return Err(io::Error::other(format!(
                    "another process writes {name}.log"
                )));
            }
            Err(fs::TryLockError::Error(err)) => return Err(err),
        }
        let path = dir.join(format!("{name}.log"));
        if fs::metadata(&path).is_ok_and(|m| m.len() > 0) {
            shift(&path)?;
        }
        Ok(Self {
            handle: create(&path)?,
            _lock: lock,
            path,
            size: 0,
            max,
        })
    }

    fn rotate(&mut self) -> io::Result<()> {
        shift(&self.path)?;
        self.handle = create(&self.path)?;
        self.size = 0;
        Ok(())
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.size > 0 && self.size + buf.len() as u64 > self.max {
            self.rotate()?;
        }
        let written = self.handle.write(buf)?;
        self.size += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.handle.flush()
    }
}

fn numbered(path: &Path, n: usize) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(format!(".{n}"));
    PathBuf::from(name)
}

/// Moves `NAME.log` to `NAME.log.1` and each old file one number up; the
/// oldest goes.
fn shift(path: &Path) -> io::Result<()> {
    for n in (1..KEEP).rev() {
        let from = numbered(path, n);
        if from.exists() {
            fs::rename(&from, numbered(path, n + 1))?;
        }
    }
    fs::rename(path, numbered(path, 1))
}

/// Opens `path` for appending, never through a symlink, with mode 0600
/// also when the file was there.
fn create(path: &Path) -> io::Result<fs::File> {
    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    fn temp(name: &str) -> crate::testing::TempDir {
        crate::testing::TempDir::unmade(&format!("chora-logs-{name}"))
    }

    fn read(path: PathBuf) -> String {
        fs::read_to_string(path).unwrap_or_default()
    }

    #[test]
    fn a_full_file_moves_up_and_only_keep_old_files_stay() {
        let dir = temp("rotate");
        let mut file = File::open_in(&dir, "chora", 10).unwrap();
        for line in [
            "one 56789\n",
            "two 56789\n",
            "three 789\n",
            "four 6789\n",
            "five 6789\n",
        ] {
            file.write_all(line.as_bytes()).unwrap();
        }
        let log = dir.join("chora.log");
        assert_eq!(read(log.clone()), "five 6789\n");
        assert_eq!(read(numbered(&log, 1)), "four 6789\n");
        assert_eq!(read(numbered(&log, 3)), "two 56789\n");
        assert!(!numbered(&log, 4).exists(), "only {KEEP} old files");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_start_keeps_the_last_run_as_the_first_old_file() {
        let dir = temp("start");
        File::open_in(&dir, "voice", MAX_BYTES)
            .unwrap()
            .write_all(b"the last run\n")
            .unwrap();
        let mut file = File::open_in(&dir, "voice", MAX_BYTES).unwrap();
        file.write_all(b"this run\n").unwrap();
        let log = dir.join("voice.log");
        assert_eq!(read(log.clone()), "this run\n");
        assert_eq!(read(numbered(&log, 1)), "the last run\n");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_second_writer_leaves_the_file_of_the_first() {
        let dir = temp("two");
        let mut first = File::open_in(&dir, "chora", MAX_BYTES).unwrap();
        first.write_all(b"the first run\n").unwrap();
        assert!(
            File::open_in(&dir, "chora", MAX_BYTES).is_err(),
            "the second writer gets no file"
        );
        first.write_all(b"still the first run\n").unwrap();
        let log = dir.join("chora.log");
        assert_eq!(read(log.clone()), "the first run\nstill the first run\n");
        assert!(!numbered(&log, 1).exists(), "nothing moved");
        drop(first);
        File::open_in(&dir, "chora", MAX_BYTES).unwrap();
        assert_eq!(
            read(numbered(&log, 1)),
            "the first run\nstill the first run\n",
            "the next start after the first moves its file"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_symlink_is_not_followed_and_old_modes_are_fixed() {
        let dir = temp("nofollow");
        fs::create_dir_all(&dir).unwrap();
        let mode = |p: &Path| fs::metadata(p).unwrap().permissions().mode() & 0o777;
        let set = |p: &Path, m: u32| fs::set_permissions(p, fs::Permissions::from_mode(m)).unwrap();
        let elsewhere = dir.join("elsewhere.txt");
        std::os::unix::fs::symlink(&elsewhere, dir.join("chora.log")).unwrap();
        assert!(File::open_in(&dir, "chora", MAX_BYTES).is_err());
        assert!(!elsewhere.exists(), "the log does not follow a symlink");
        fs::write(dir.join("voice.log"), "").unwrap();
        set(&dir.join("voice.log"), 0o644);
        set(&dir, 0o755);
        File::open_in(&dir, "voice", MAX_BYTES).unwrap();
        assert_eq!(mode(&dir), 0o700);
        assert_eq!(mode(&dir.join("voice.log")), 0o600);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_symlinked_folder_gets_no_file_and_no_chmod() {
        let dir = temp("symlinked-dir");
        let target = dir.join("target");
        fs::create_dir_all(&target).unwrap();
        fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).unwrap();
        let logs = dir.join("logs");
        std::os::unix::fs::symlink(&target, &logs).unwrap();
        assert!(File::open_in(&logs, "chora", MAX_BYTES).is_err());
        let mode = fs::metadata(&target).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o755, "the target of the symlink keeps its mode");
        assert_eq!(fs::read_dir(&target).unwrap().count(), 0, "nothing in it");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn only_the_person_reads_the_logs() {
        let dir = temp("mode");
        File::open_in(&dir, "chora", MAX_BYTES).unwrap();
        let mode = |p: &Path| fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&dir), 0o700);
        assert_eq!(mode(&dir.join("chora.log")), 0o600);
        let _ = fs::remove_dir_all(&dir);
    }
}
