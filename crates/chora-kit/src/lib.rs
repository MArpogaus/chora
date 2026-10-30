//! Building blocks for chora plugins that are not part of the API of the
//! core: the IPC and event connection to the scene, effects that a window
//! manager computes itself, such as wobbly windows and the magic lamp,
//! the freedesktop files that a shell reads, such as icon themes and
//! desktop entries, the start of apps, and the runtime of a plugin with
//! its settings.
pub mod conn;
pub mod desktop_entry;
pub mod icons;
pub mod lamp;
pub mod runtime;
pub mod settings;
pub mod wobbly;
pub mod xdg;

/// Starts a shell command as an app with `choractl run` (D42). The app
/// gets an activation token, and a unit of its own when a user manager
/// runs. `&` lets the shell exit at once, so no zombie stays.
pub fn run(command: &str) {
    let started = std::process::Command::new("sh")
        .args(["-c", r#"choractl run sh -c "$0" >/dev/null &"#, command])
        .status();
    if let Err(err) = started {
        eprintln!("cannot run {command:?}: {err}");
    }
}

/// The most bytes of a picture file that a plugin reads.
pub const MAX_PICTURE: u64 = 64 << 20;
/// The most bytes of a text file, such as settings, that a plugin reads.
pub const MAX_TEXT: u64 = 1 << 20;

/// Reads a file that a user named, such as a wallpaper or a settings
/// file, if it is a regular file of at most `max` bytes. A device such as
/// `/dev/zero` never ends, and a FIFO blocks: the open does not wait for a
/// writer, and the check runs on the open file, so a swap of the path in
/// between changes nothing. A file of `/proc` claims 0 bytes; the read
/// stops after `max` bytes all the same.
///
/// # Errors
///
/// The file cannot be read, is no regular file, or has more than `max`
/// bytes.
pub fn read_file(path: &std::path::Path, max: u64) -> std::io::Result<Vec<u8>> {
    use std::io::Read as _;
    let mut data = Vec::new();
    open_file(path)?
        .take(max.saturating_add(1))
        .read_to_end(&mut data)?;
    if data.len() as u64 > max {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("larger than {max} bytes"),
        ));
    }
    Ok(data)
}

/// Opens a file that a user named if it is a regular file, for a reader
/// that stops by itself, such as one of a header; see [`read_file`].
///
/// # Errors
///
/// The file cannot be opened, or it is no regular file.
pub fn open_file(path: &std::path::Path) -> std::io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt as _;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)?;
    if !file.metadata()?.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "not a regular file",
        ));
    }
    Ok(file)
}

/// [`read_file`] as UTF-8 text.
///
/// # Errors
///
/// As [`read_file`], or the file is no UTF-8.
pub fn read_text(path: &std::path::Path, max: u64) -> std::io::Result<String> {
    String::from_utf8(read_file(path, max)?)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Checks a file that a setting names, before a plugin or the core reads
/// it: a regular file outside `/proc`, `/sys` and `/dev`, whose files are
/// no pictures or shaders.
///
/// # Errors
///
/// The file is missing or of another kind; the message names it.
pub fn check_file(path: &std::path::Path) -> Result<(), String> {
    let real = std::fs::canonicalize(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let special = ["/proc", "/sys", "/dev"]
        .iter()
        .any(|d| real.starts_with(d));
    if special || !real.is_file() {
        return Err(format!("{}: not a regular file", path.display()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn devices_fifos_and_large_files_are_refused() {
        let err = super::read_file(Path::new("/dev/zero"), 1 << 20).unwrap_err();
        assert!(err.to_string().contains("not a regular file"), "{err}");
        assert!(super::check_file(Path::new("/dev/zero")).is_err());
        assert!(super::check_file(Path::new("/proc/self/mem")).is_err());
        // A file of /proc claims 0 bytes; the read stops at the cap.
        assert!(super::read_file(Path::new("/proc/self/maps"), 10).is_err());
    }

    #[test]
    fn fifos_are_refused_and_files_are_capped() {
        let dir = std::env::temp_dir().join(format!("chora-kit-read-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let fifo = dir.join("fifo");
        let made = std::process::Command::new("mkfifo").arg(&fifo).status();
        assert!(made.unwrap().success());
        assert!(
            super::read_file(&fifo, 10).is_err(),
            "a FIFO must not block"
        );
        assert!(super::check_file(&fifo).is_err());
        let file = dir.join("ten");
        std::fs::write(&file, b"0123456789").unwrap();
        assert_eq!(super::read_file(&file, 10).unwrap().len(), 10);
        assert!(super::read_file(&file, 9).is_err());
        assert!(super::check_file(&file).is_ok());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
