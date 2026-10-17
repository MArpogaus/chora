//! What XP's Explorer shows, without the drawing: the places it goes to,
//! the files of a folder in XP's order, their kinds by extension, sizes
//! in XP's words, and the history of Back, Forward and Up.

use std::{
    fs,
    os::unix::ffi::OsStringExt as _,
    os::unix::fs::PermissionsExt as _,
    path::{Path, PathBuf},
    time::SystemTime,
};

/// What an Explorer window shows.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Place {
    /// A folder of the file system.
    Folder(PathBuf),
    /// My Computer: the user's documents and the drives.
    Computer,
    /// The Recycle Bin.
    Trash,
}

impl Place {
    /// The title of the window and the text of the address bar: the name
    /// of a folder, or its path for `/`.
    pub fn title(&self) -> String {
        match self {
            Self::Folder(path) => path.file_name().map_or_else(
                || path.to_string_lossy().into_owned(),
                |n| n.to_string_lossy().into_owned(),
            ),
            Self::Computer => "My Computer".into(),
            Self::Trash => "Recycle Bin".into(),
        }
    }

    /// The text of the address bar: the path of a folder.
    pub fn address(&self) -> String {
        match self {
            Self::Folder(path) => path.to_string_lossy().into_owned(),
            _ => self.title(),
        }
    }

    /// The place that Up goes to: the parent of a folder, My Computer for
    /// `/`, and the Desktop folder for My Computer and the Recycle Bin,
    /// which sit on the desktop, as on XP.
    pub fn up(&self) -> Self {
        match self {
            Self::Folder(path) => path
                .parent()
                .map_or(Self::Computer, |p| Self::Folder(p.into())),
            Self::Computer | Self::Trash => Self::Folder(crate::explorer::desktop()),
        }
    }
}

/// The places of Back and Forward around the current one.
#[derive(Clone, Debug)]
pub struct History {
    back: Vec<Place>,
    forward: Vec<Place>,
    /// The place that shows.
    pub current: Place,
}

impl History {
    /// A history that starts at a place.
    pub fn new(place: Place) -> Self {
        Self {
            back: Vec::new(),
            forward: Vec::new(),
            current: place,
        }
    }

    /// Goes to a place: the current one goes back, and Forward forgets.
    pub fn go(&mut self, place: Place) {
        if place == self.current {
            return;
        }
        self.back.push(std::mem::replace(&mut self.current, place));
        self.forward.clear();
    }

    /// Whether Back has a place.
    pub fn can_back(&self) -> bool {
        !self.back.is_empty()
    }

    /// Whether Forward has a place.
    pub fn can_forward(&self) -> bool {
        !self.forward.is_empty()
    }

    /// Goes back; false when there is no place.
    pub fn back(&mut self) -> bool {
        let Some(place) = self.back.pop() else {
            return false;
        };
        self.forward
            .push(std::mem::replace(&mut self.current, place));
        true
    }

    /// Goes forward; false when there is no place.
    pub fn forward(&mut self) -> bool {
        let Some(place) = self.forward.pop() else {
            return false;
        };
        self.back.push(std::mem::replace(&mut self.current, place));
        true
    }

    /// Goes up, as a new place; false at the top.
    pub fn up(&mut self) -> bool {
        self.go(self.current.up());
        true
    }
}

/// The kind of a file, by its extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Folder,
    Text,
    Image,
    Audio,
    Video,
    Archive,
    Executable,
    Generic,
}

impl Kind {
    /// The kind of a file of a name; `executable` for a file with the
    /// mode bits to run it.
    pub fn of(name: &str, dir: bool, executable: bool) -> Self {
        if dir {
            return Self::Folder;
        }
        let ext = extension(name);
        match ext.as_str() {
            "txt" | "md" | "log" | "rs" | "py" | "c" | "h" | "cpp" | "toml" | "yaml" | "yml"
            | "json" | "xml" | "html" | "css" | "js" | "ini" | "conf" | "cfg" | "csv" | "org"
            | "tex" => Self::Text,
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" | "tif" | "tiff" | "ico" => {
                Self::Image
            }
            "mp3" | "wav" | "ogg" | "oga" | "flac" | "opus" | "m4a" | "wma" | "mid" => Self::Audio,
            "mp4" | "mkv" | "avi" | "webm" | "mov" | "mpg" | "mpeg" | "wmv" | "ogv" => Self::Video,
            "zip" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "zst" | "7z" | "rar" | "rpm" | "deb" => {
                Self::Archive
            }
            "exe" | "sh" | "appimage" | "run" | "bin" => Self::Executable,
            _ if executable => Self::Executable,
            _ => Self::Generic,
        }
    }

    /// The icon names to look for, the most fitting first.
    pub fn icons(self) -> &'static [&'static str] {
        match self {
            Self::Folder => &["folder"],
            Self::Text => &["text-x-generic"],
            Self::Image => &["image-x-generic", "text-x-generic"],
            Self::Audio => &["audio-x-generic", "text-x-generic"],
            Self::Video => &["video-x-generic", "text-x-generic"],
            Self::Archive => &[
                "package-x-generic",
                "application-x-archive",
                "text-x-generic",
            ],
            Self::Executable => &[
                "application-x-executable",
                "application-x-generic",
                "text-x-generic",
            ],
            Self::Generic => &["application-x-generic", "unknown", "text-x-generic"],
        }
    }
}

/// The lowercase extension of a file name, without the dot; empty for
/// none and for a name that starts with its only dot.
fn extension(name: &str) -> String {
    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => ext.to_lowercase(),
        _ => String::new(),
    }
}

/// XP's name of the type of a file, as the Tiles view shows it below the
/// name.
pub fn type_name(name: &str, kind: Kind) -> String {
    let ext = extension(name);
    let upper = ext.to_uppercase();
    match (kind, ext.as_str()) {
        (Kind::Folder, _) => "File Folder".into(),
        (Kind::Text, "txt") => "Text Document".into(),
        (Kind::Image, "jpg" | "jpeg") => "JPEG Image".into(),
        (Kind::Image, "bmp") => "Bitmap Image".into(),
        (Kind::Image, _) => format!("{upper} Image"),
        (Kind::Audio, "wav") => "Wave Sound".into(),
        (Kind::Audio, _) => format!("{upper} Format Sound"),
        (Kind::Video, _) => "Video Clip".into(),
        (Kind::Archive, "zip") => "Compressed (zipped) Folder".into(),
        (Kind::Executable, _) => "Application".into(),
        (_, "") => "File".into(),
        _ => format!("{upper} File"),
    }
}

/// A size as XP's Explorer writes it: bytes up to 1023, then three
/// digits of KB, MB, GB or TB, cut, not rounded: `12.3 KB`.
pub fn size_text(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["KB", "MB", "GB", "TB"];
    if bytes < 1024 {
        return format!("{bytes} bytes");
    }
    #[allow(clippy::cast_precision_loss)]
    let mut value = bytes as f64 / 1024.0;
    let mut unit = 0;
    while value >= 1000.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    let decimals: usize = if value < 10.0 {
        2
    } else {
        usize::from(value < 100.0)
    };
    let scale = [1.0, 10.0, 100.0][decimals];
    let cut = (value * scale).floor() / scale;
    format!("{cut:.decimals$} {}", UNITS[unit])
}

/// A file or folder of a folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The file name.
    pub name: String,
    /// The whole path.
    pub path: PathBuf,
    /// What it is.
    pub kind: Kind,
    /// The size in bytes; 0 for a folder.
    pub size: u64,
    /// When it changed last.
    pub modified: Option<SystemTime>,
}

/// The files and folders of a folder, as XP lists them: folders first,
/// then by name without case; hidden files, whose name starts with a dot,
/// are left out.
pub fn list(dir: &Path) -> std::io::Result<Vec<Entry>> {
    let mut entries: Vec<Entry> = fs::read_dir(dir)?
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                return None;
            }
            let path = e.path();
            // Links count as what they point at.
            let meta = fs::metadata(&path).or_else(|_| e.metadata()).ok()?;
            let executable = meta.is_file() && meta.permissions().mode() & 0o111 != 0;
            Some(Entry {
                kind: Kind::of(&name, meta.is_dir(), executable),
                size: if meta.is_dir() { 0 } else { meta.len() },
                modified: meta.modified().ok(),
                name,
                path,
            })
        })
        .collect();
    sort(&mut entries);
    Ok(entries)
}

/// XP's order: folders first, then by name without case, with the
/// numbers in names by their value: `file2` before `file10`.
pub fn sort(entries: &mut [Entry]) {
    entries.sort_by_cached_key(|e| (e.kind != Kind::Folder, natural(&e.name)));
}

/// A part of a name for XP's order: a number before any text.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Chunk {
    Number(u128),
    Text(String),
}

/// The key of a name in XP's order: its runs of digits and of other
/// characters, without case.
fn natural(name: &str) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let mut rest = name;
    while let Some(first) = rest.chars().next() {
        let digit = first.is_ascii_digit();
        let end = rest
            .find(|c: char| c.is_ascii_digit() != digit)
            .unwrap_or(rest.len());
        let (run, tail) = rest.split_at(end);
        chunks.push(match run.parse() {
            Ok(n) if digit => Chunk::Number(n),
            _ => Chunk::Text(run.to_lowercase()),
        });
        rest = tail;
    }
    chunks
}

/// A mounted file system that My Computer shows as a drive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drive {
    /// The device, such as `/dev/nvme0n1p2`.
    pub device: String,
    /// Where it is mounted.
    pub dir: PathBuf,
    /// The file system, such as `ext4`.
    pub fs: String,
    /// A removable medium: below `/run/media` or `/media`, or a disc.
    pub removable: bool,
}

impl Drive {
    /// XP's name of the drive: `Local Disk (/)` for a disk, the name of
    /// the medium for a removable one.
    pub fn title(&self) -> String {
        match self.dir.file_name() {
            Some(name) if self.removable => name.to_string_lossy().into_owned(),
            _ => format!("Local Disk ({})", self.dir.display()),
        }
    }
}

/// The file systems of disks and media; the others, such as `proc`,
/// `tmpfs` or `overlay`, are no drives.
const DISK_FILE_SYSTEMS: [&str; 17] = [
    "btrfs", "ext2", "ext3", "ext4", "xfs", "f2fs", "jfs", "reiserfs", "zfs", "bcachefs", "vfat",
    "exfat", "ntfs", "ntfs3", "fuseblk", "hfsplus", "iso9660",
];

/// Directories whose mounts are no drives.
const SYSTEM_DIRS: [&str; 6] = ["/dev", "/sys", "/proc", "/etc", "/var/lib", "/var/log"];

/// The drives of `/proc/self/mounts`: the mounts of disk file systems
/// and of discs (`udf` too), each device once, at its shortest path, as
/// a file system with subvolumes mounts its device several times.
pub fn drives(mounts: &str) -> Vec<Drive> {
    let mut drives: Vec<Drive> = Vec::new();
    for line in mounts.lines() {
        let mut fields = line.split_whitespace();
        let (Some(device), Some(dir), Some(fs)) = (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        if !DISK_FILE_SYSTEMS.contains(&fs) && fs != "udf" {
            continue;
        }
        let dir = PathBuf::from(unescape(dir));
        // A container binds files and system directories of a disk
        // there, such as /etc/hostname.
        if SYSTEM_DIRS.iter().any(|d| dir.starts_with(d)) {
            continue;
        }
        let disc = matches!(fs, "iso9660" | "udf");
        let removable = disc || dir.starts_with("/run/media") || dir.starts_with("/media");
        let drive = Drive {
            device: unescape(device),
            dir,
            fs: fs.into(),
            removable,
        };
        match drives.iter_mut().find(|d| d.device == drive.device) {
            Some(known) if drive.dir.as_os_str().len() < known.dir.as_os_str().len() => {
                *known = drive;
            }
            Some(_) => {}
            None => drives.push(drive),
        }
    }
    drives.sort_by(|a, b| a.dir.cmp(&b.dir));
    drives
}

/// A field of `/proc/self/mounts`, whose spaces and tabs come as octal
/// escapes such as `\040`.
fn unescape(field: &str) -> String {
    let mut out = String::new();
    let mut rest = field;
    while let Some(i) = rest.find('\\') {
        out += &rest[..i];
        let code = rest
            .get(i + 1..i + 4)
            .and_then(|o| u8::from_str_radix(o, 8).ok());
        if let Some(byte) = code {
            out.push(char::from(byte));
            rest = &rest[i + 4..];
        } else {
            out.push('\\');
            rest = &rest[i + 1..];
        }
    }
    out + rest
}

/// The total and the free bytes of the file system of a directory, from
/// `df`: its second line holds them.
pub fn space(dir: &Path) -> Option<(u64, u64)> {
    let output = std::process::Command::new("df")
        .args(["-B1", "--output=size,avail"])
        .arg(dir)
        .output()
        .ok()?;
    parse_df(&String::from_utf8_lossy(&output.stdout))
}

/// The size and the free bytes of the output of `df --output=size,avail`.
fn parse_df(text: &str) -> Option<(u64, u64)> {
    let mut numbers = text.lines().nth(1)?.split_whitespace();
    Some((numbers.next()?.parse().ok()?, numbers.next()?.parse().ok()?))
}

/// A file or folder in the trash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trashed {
    /// The file as the trash holds it.
    pub entry: Entry,
    /// Where it was, from its `.trashinfo` file.
    pub original: Option<PathBuf>,
    /// When it went to the trash, as the `.trashinfo` file writes it:
    /// `2026-10-15T09:12:30`.
    pub deleted: Option<String>,
}

/// The trash of the freedesktop.org specification: `Trash` of the
/// first XDG data directory, `$XDG_DATA_HOME`.
pub fn trash_root() -> PathBuf {
    crate::icons::data_dirs()
        .into_iter()
        .next()
        .unwrap_or_default()
        .join("Trash")
}

/// The files of a trash, in XP's order, with where they were and when
/// they went.
pub fn trash(root: &Path) -> Vec<Trashed> {
    list(&root.join("files"))
        .unwrap_or_default()
        .into_iter()
        .map(|entry| {
            let info = root.join("info").join(format!("{}.trashinfo", entry.name));
            let (original, deleted) = fs::read_to_string(info)
                .map(|text| trash_info(&text))
                .unwrap_or_default();
            Trashed {
                entry,
                original,
                deleted,
            }
        })
        .collect()
}

/// The path and the date of a `.trashinfo` file. The path is
/// percent-encoded, as in a URL.
fn trash_info(text: &str) -> (Option<PathBuf>, Option<String>) {
    let mut in_section = false;
    let (mut path, mut date) = (None, None);
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            in_section = line == "[Trash Info]";
        } else if !in_section {
        } else if let Some(value) = line.strip_prefix("Path=") {
            let bytes = percent_decode(value);
            path = Some(PathBuf::from(std::ffi::OsString::from_vec(bytes)));
        } else if let Some(value) = line.strip_prefix("DeletionDate=") {
            date = Some(value.to_string());
        }
    }
    (path, date)
}

/// The bytes of a percent-encoded text: `%20` is a space.
fn percent_decode(text: &str) -> Vec<u8> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes
            .get(i + 1..i + 3)
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match (bytes[i], hex) {
            (b'%', Some(byte)) => {
                out.push(byte);
                i += 3;
            }
            (byte, _) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    out
}

/// Moves a file or folder into a trash, as the freedesktop.org
/// specification says: first the `.trashinfo` file in `info/` with the
/// percent-encoded path and the date, made only if no such file exists,
/// so that its name is reserved; then the file itself by a rename into
/// `files/`. A name that the trash holds already gets a number: `a.2.txt`.
/// The date is the local time as `2026-10-17T08:55:31`. Returns the name
/// in the trash. A file on another file system than the trash fails, and
/// stays where it is.
pub fn trash_file(root: &Path, path: &Path, date: &str) -> std::io::Result<String> {
    use std::io::Write as _;
    let (files, info) = (root.join("files"), root.join("info"));
    fs::create_dir_all(&files)?;
    fs::create_dir_all(&info)?;
    let path = std::path::absolute(path)?;
    let name = path
        .file_name()
        .ok_or_else(|| std::io::Error::other("no file name"))?
        .to_string_lossy()
        .into_owned();
    for n in 1.. {
        let candidate = numbered(&name, n);
        let info_file = info.join(format!("{candidate}.trashinfo"));
        if files.join(&candidate).symlink_metadata().is_ok() {
            continue;
        }
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&info_file)
        {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err),
        };
        file.write_all(trash_info_text(&path, date).as_bytes())?;
        if let Err(err) = fs::rename(&path, files.join(&candidate)) {
            let _ = fs::remove_file(&info_file);
            return Err(err);
        }
        return Ok(candidate);
    }
    unreachable!("the names never run out")
}

/// What a drop into a folder did with a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dropped {
    /// Moved, on the same file system.
    Moved,
    /// Copied, to another file system, as XP copies to another drive.
    Copied,
    /// Left as it is: it is in the folder already.
    Stayed,
}

/// Drops a file or a folder into a folder, as XP does: a move on the same
/// file system, a copy to another one. A name that the folder has already
/// stays as it is, and the drop fails.
pub fn drop_into(path: &Path, dir: &Path) -> std::io::Result<Dropped> {
    let name = path
        .file_name()
        .ok_or_else(|| std::io::Error::other("no file name"))?;
    if path.parent() == Some(dir) {
        return Ok(Dropped::Stayed);
    }
    let target = dir.join(name);
    if target.symlink_metadata().is_ok() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("{} exists", target.display()),
        ));
    }
    if dir.starts_with(path) {
        return Err(std::io::Error::other("a folder into itself"));
    }
    match fs::rename(path, &target) {
        Ok(()) => Ok(Dropped::Moved),
        Err(err) if err.kind() == std::io::ErrorKind::CrossesDevices => {
            copy_all(path, &target)?;
            Ok(Dropped::Copied)
        }
        Err(err) => Err(err),
    }
}

/// Copies a file, or a folder with all it holds.
fn copy_all(from: &Path, to: &Path) -> std::io::Result<()> {
    if !from.is_dir() {
        return fs::copy(from, to).map(|_| ());
    }
    fs::create_dir(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        copy_all(&entry.path(), &to.join(entry.file_name()))?;
    }
    Ok(())
}

/// The `n`th name for a file in the trash: the name itself, then with a
/// number before the extension.
fn numbered(name: &str, n: u32) -> String {
    if n == 1 {
        return name.into();
    }
    match name.rsplit_once('.').filter(|(stem, _)| !stem.is_empty()) {
        Some((stem, ext)) => format!("{stem}.{n}.{ext}"),
        None => format!("{name}.{n}"),
    }
}

/// The text of a `.trashinfo` file.
fn trash_info_text(path: &Path, date: &str) -> String {
    use std::os::unix::ffi::OsStrExt as _;
    format!(
        "[Trash Info]\nPath={}\nDeletionDate={date}\n",
        percent_encode(path.as_os_str().as_bytes())
    )
}

/// Percent-encodes the bytes of a path as a URL does, keeping the
/// slashes and the characters that need no encoding.
fn percent_encode(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len());
    for &b in bytes {
        if b.is_ascii_alphanumeric() || b"/-_.~".contains(&b) {
            out.push(char::from(b));
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}

/// Empties a trash: deletes its files and folders for good, and their
/// `.trashinfo` files. A link goes, not what it points at.
pub fn empty_trash(root: &Path) -> std::io::Result<()> {
    for dir in ["files", "info"] {
        let Ok(entries) = fs::read_dir(root.join(dir)) else {
            continue;
        };
        for entry in entries {
            let path = entry?.path();
            if fs::symlink_metadata(&path)?.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_read_as_in_xp() {
        assert_eq!(size_text(0), "0 bytes");
        assert_eq!(size_text(1023), "1023 bytes");
        assert_eq!(size_text(1024), "1.00 KB");
        assert_eq!(size_text(12_650), "12.3 KB");
        assert_eq!(size_text(126_000), "123 KB");
        assert_eq!(size_text(1_040_000), "0.99 MB");
        assert_eq!(size_text(5 * 1024 * 1024 * 1024), "5.00 GB");
    }

    #[test]
    fn kinds_come_from_the_extension() {
        assert_eq!(Kind::of("notes.TXT", false, false), Kind::Text);
        assert_eq!(Kind::of("photo.jpeg", false, false), Kind::Image);
        assert_eq!(Kind::of("song.mp3", false, false), Kind::Audio);
        assert_eq!(Kind::of("film.mkv", false, false), Kind::Video);
        assert_eq!(Kind::of("a.tar.gz", false, false), Kind::Archive);
        assert_eq!(Kind::of("run", false, true), Kind::Executable);
        assert_eq!(Kind::of("data.bin2", false, false), Kind::Generic);
        assert_eq!(Kind::of(".bashrc", false, false), Kind::Generic);
        assert_eq!(Kind::of("x.txt", true, false), Kind::Folder);
    }

    #[test]
    fn a_file_goes_to_the_trash_with_its_info() {
        let dir = std::env::temp_dir().join(format!("chora-xp-trash1-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let (desk, root) = (dir.join("Desktop"), dir.join("data/Trash"));
        fs::create_dir_all(desk.join("a folder")).unwrap();
        fs::write(desk.join("notes 100%.txt"), "x").unwrap();
        let date = "2026-10-17T08:55:31";
        let name = trash_file(&root, &desk.join("notes 100%.txt"), date).unwrap();
        assert_eq!(name, "notes 100%.txt");
        assert!(!desk.join("notes 100%.txt").exists());
        assert!(root.join("files/notes 100%.txt").is_file());
        let info = fs::read_to_string(root.join("info/notes 100%.txt.trashinfo")).unwrap();
        let path = format!("{}/notes%20100%25.txt", desk.display());
        assert_eq!(
            info,
            format!("[Trash Info]\nPath={path}\nDeletionDate={date}\n")
        );
        // It reads back as the Recycle Bin shows it.
        assert_eq!(
            trash_info(&info),
            (Some(desk.join("notes 100%.txt")), Some(date.into()))
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn names_in_the_trash_get_numbers() {
        let dir = std::env::temp_dir().join(format!("chora-xp-trash2-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let (desk, root) = (dir.join("Desktop"), dir.join("data/Trash"));
        fs::create_dir_all(desk.join("a folder")).unwrap();
        let date = "2026-10-17T08:55:31";
        let trash = |name: &str| trash_file(&root, &desk.join(name), date).ok();
        for (content, expected) in [
            ("x", "notes.txt"),
            ("y", "notes.2.txt"),
            ("z", "notes.3.txt"),
        ] {
            fs::write(desk.join("notes.txt"), content).unwrap();
            assert_eq!(trash("notes.txt").as_deref(), Some(expected));
        }
        fs::write(desk.join("b"), "x").unwrap();
        assert_eq!(trash("b").as_deref(), Some("b"));
        fs::write(desk.join("b"), "y").unwrap();
        assert_eq!(trash("b").as_deref(), Some("b.2"));
        assert_eq!(trash("a folder").as_deref(), Some("a folder"));
        assert!(root.join("files/a folder").is_dir());
        // A missing file fails and leaves no info behind.
        assert_eq!(trash("gone"), None);
        assert!(!root.join("info/gone.trashinfo").exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn trash_names_and_paths_are_encoded() {
        assert_eq!(numbered(".bashrc", 3), ".bashrc.3");
        assert_eq!(numbered("a.tar.gz", 2), "a.tar.2.gz");
        assert_eq!(percent_encode("/ä b".as_bytes()), "/%C3%A4%20b");
    }

    #[test]
    fn types_have_xp_names() {
        assert_eq!(type_name("a.txt", Kind::Text), "Text Document");
        assert_eq!(type_name("a.rs", Kind::Text), "RS File");
        assert_eq!(type_name("b.png", Kind::Image), "PNG Image");
        assert_eq!(type_name("Makefile", Kind::Generic), "File");
        assert_eq!(type_name("src", Kind::Folder), "File Folder");
    }

    #[test]
    fn a_folder_lists_folders_first_by_name_without_hidden_files() {
        let dir = std::env::temp_dir().join(format!("chora-xp-list-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for d in ["b-dir", "A-dir", ".hidden-dir"] {
            fs::create_dir_all(dir.join(d)).unwrap();
        }
        for f in ["c.txt", "a.png", ".hidden", "B.md"] {
            fs::write(dir.join(f), "12345").unwrap();
        }
        let names: Vec<String> = list(&dir).unwrap().into_iter().map(|e| e.name).collect();
        assert_eq!(names, ["A-dir", "b-dir", "a.png", "B.md", "c.txt"]);
        assert_eq!(list(&dir).unwrap()[2].size, 5);
        let mut entries: Vec<Entry> = ["file10", "File2", "file1", "a"]
            .map(|name| Entry {
                name: name.into(),
                path: name.into(),
                kind: Kind::Text,
                size: 0,
                modified: None,
            })
            .into();
        sort(&mut entries);
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["a", "file1", "File2", "file10"]);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn back_forward_and_up_walk_the_history() {
        let folder = |p: &str| Place::Folder(p.into());
        let mut h = History::new(folder("/home/u"));
        assert!(!h.can_back() && !h.back());
        h.go(folder("/home/u/Music"));
        assert!(h.up());
        assert_eq!(h.current, folder("/home/u"));
        assert!(h.back());
        assert_eq!(h.current, folder("/home/u/Music"));
        assert!(h.back() && h.can_forward());
    }

    #[test]
    fn forward_and_up_walk_the_history() {
        let folder = |p: &str| Place::Folder(p.into());
        let mut h = History::new(folder("/home/u"));
        h.go(folder("/home/u/Music"));
        h.back();
        assert_eq!(h.current, folder("/home/u"));
        assert!(h.forward());
        assert_eq!(h.current, folder("/home/u/Music"));
        // A new place forgets Forward.
        h.back();
        h.go(Place::Computer);
        assert!(!h.can_forward());
        assert!(h.up());
        assert_eq!(h.current, Place::Folder(crate::explorer::desktop()));
        assert_eq!(folder("/").up(), Place::Computer);
        assert_eq!(folder("/").title(), "/");
        assert_eq!(folder("/home/u").title(), "u");
    }

    #[test]
    fn drives_are_the_disks_and_media_of_the_mounts() {
        let mounts = "\
/dev/nvme0n1p3 / btrfs rw,subvol=/root 0 0
proc /proc proc rw 0 0
tmpfs /tmp tmpfs rw 0 0
/dev/nvme0n1p3 /home btrfs rw,subvol=/home 0 0
/dev/sdb1 /etc/hostname btrfs ro 0 0
/dev/nvme0n1p1 /boot/efi vfat rw 0 0
/dev/sda1 /run/media/u/MY\\040STICK exfat rw 0 0
/dev/sr0 /mnt/cd iso9660 ro 0 0
overlay /var/lib/x overlay rw 0 0
";
        let drives = drives(mounts);
        let titles: Vec<String> = drives.iter().map(Drive::title).collect();
        assert_eq!(
            titles,
            ["Local Disk (/)", "Local Disk (/boot/efi)", "cd", "MY STICK"]
        );
        let removable: Vec<bool> = drives.iter().map(|d| d.removable).collect();
        assert_eq!(removable, [false, false, true, true]);
        assert_eq!(drives[0].device, "/dev/nvme0n1p3");
    }

    #[test]
    fn df_gives_the_size_and_the_free_space() {
        let out = "    1B-blocks      Avail\n 511101108224 98765432100\n";
        assert_eq!(parse_df(out), Some((511_101_108_224, 98_765_432_100)));
        assert_eq!(parse_df("header only\n"), None);
    }

    #[test]
    fn a_trashinfo_file_tells_where_and_when() {
        let text =
            "[Trash Info]\nPath=/home/u/My%20Notes%2Btodo.txt\nDeletionDate=2026-10-15T09:12:30\n";
        assert_eq!(
            trash_info(text),
            (
                Some("/home/u/My Notes+todo.txt".into()),
                Some("2026-10-15T09:12:30".into())
            )
        );
        // Another section does not count, and a broken escape stays.
        let text = "[Other]\nPath=/x\n[Trash Info]\nPath=/a%2\n";
        assert_eq!(trash_info(text), (Some("/a%2".into()), None));
    }

    #[test]
    fn emptying_the_trash_deletes_files_folders_and_their_info() {
        let root = std::env::temp_dir().join(format!("chora-xp-trash-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("files/old-dir/sub")).unwrap();
        fs::create_dir_all(root.join("info")).unwrap();
        fs::write(root.join("files/a.txt"), "a").unwrap();
        fs::write(
            root.join("info/a.txt.trashinfo"),
            "[Trash Info]\nPath=/x/a.txt\n",
        )
        .unwrap();
        // A link to a folder outside the trash: the folder stays.
        let outside = root.join("outside");
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("keep"), "k").unwrap();
        std::os::unix::fs::symlink(&outside, root.join("files/link")).unwrap();
        let items = trash(&root);
        assert_eq!(items.len(), 3);
        // The link to a folder counts as a folder.
        let names: Vec<&str> = items.iter().map(|t| t.entry.name.as_str()).collect();
        assert_eq!(names, ["link", "old-dir", "a.txt"]);
        assert_eq!(items[2].original, Some("/x/a.txt".into()));
        empty_trash(&root).unwrap();
        assert_eq!(trash(&root).len(), 0);
        assert_eq!(fs::read_dir(root.join("info")).unwrap().count(), 0);
        assert!(outside.join("keep").is_file());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_drop_moves_into_the_folder_and_keeps_what_is_there() {
        let dir = std::env::temp_dir().join(format!("chora-xp-drop-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("into")).expect("a folder");
        fs::write(dir.join("a.txt"), "a").expect("a file");
        assert_eq!(
            drop_into(&dir.join("a.txt"), &dir.join("into")).ok(),
            Some(Dropped::Moved)
        );
        assert!(dir.join("into/a.txt").is_file() && !dir.join("a.txt").exists());
        // The name is taken: the file stays where it is.
        fs::write(dir.join("a.txt"), "b").expect("a file");
        assert!(drop_into(&dir.join("a.txt"), &dir.join("into")).is_err());
        assert!(dir.join("a.txt").is_file());
        // A file into its own folder stays.
        assert_eq!(
            drop_into(&dir.join("a.txt"), &dir).ok(),
            Some(Dropped::Stayed)
        );
        // Not a folder into itself.
        assert!(drop_into(&dir.join("into"), &dir.join("into")).is_err());
        // The copy of another file system takes the folder whole.
        copy_all(&dir.join("into"), &dir.join("copy")).expect("a copy");
        assert!(dir.join("copy/a.txt").is_file());
        let _ = fs::remove_dir_all(dir);
    }
}
