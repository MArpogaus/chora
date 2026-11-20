//! Icons of the installed freedesktop icon themes, by name, for menus,
//! title bars and task buttons.

use std::{
    collections::{HashMap, VecDeque},
    fs,
    io::{Read, Write},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};

/// Finds icons and remembers the answers.
pub struct Icons {
    themes: Vec<String>,
    found: HashMap<String, Option<PathBuf>>,
    /// Private copies of sender icons, oldest first.
    copies: VecDeque<PathBuf>,
    /// The private folder in the temp directory, made at the first copy
    /// without a runtime directory; its name has a random part.
    temp_dir: Option<PathBuf>,
}

/// Numbers the copies of all `Icons` of a process.
static SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// A random number as hex, for a name that no other user can guess. The
/// keys of `RandomState` come from the system.
fn random_suffix() -> String {
    use std::hash::{BuildHasher, Hasher};
    format!(
        "{:016x}",
        std::collections::hash_map::RandomState::new()
            .build_hasher()
            .finish()
    )
}

/// Copies of sender icons kept at once.
const KEPT_COPIES: usize = 16;

impl Drop for Icons {
    fn drop(&mut self) {
        for copy in self.copies.drain(..) {
            let _ = fs::remove_file(copy);
        }
        if let Some(dir) = self.temp_dir.take() {
            let _ = fs::remove_dir(dir);
        }
    }
}

impl Icons {
    /// Searches the themes in order, by the start of their directory
    /// name, such as `breeze` for `breeze` and `breeze-dark`, then
    /// `/usr/share/pixmaps`.
    #[must_use]
    pub fn new(themes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            themes: themes.into_iter().map(Into::into).collect(),
            found: HashMap::new(),
            copies: VecDeque::new(),
            temp_dir: None,
        }
    }

    /// The file of an icon: an SVG file if there is one, else the largest
    /// PNG file up to 64 pixels. An absolute path is its own file.
    pub fn find(&mut self, name: &str) -> Option<PathBuf> {
        if let Some(found) = self.found.get(name) {
            return found.clone();
        }
        let found = search_all(&self.themes, name, None);
        self.found.insert(name.into(), found.clone());
        found
    }

    /// The file of the icon a notification sender names. A path is used
    /// only as a small PNG file, read once and handed on as a private
    /// copy that the sender cannot write; anything else falls back to the
    /// theme icon of the file name without its extension.
    pub fn for_sender(&mut self, name: &str) -> Option<PathBuf> {
        let path = Path::new(name);
        if !path.is_absolute() {
            return self.find(name);
        }
        if let Some(copy) = read_small_png(path).and_then(|bytes| self.private_copy(&bytes)) {
            return Some(copy);
        }
        let stem = path.file_stem()?.to_str()?;
        self.find(stem)
    }

    /// Writes bytes to a new file of mode 0600 in a private folder.
    fn private_copy(&mut self, bytes: &[u8]) -> Option<PathBuf> {
        self.private_copy_in(chora_scene::paths::runtime_home(), bytes)
    }

    /// [`Self::private_copy`] below a given runtime directory.
    fn private_copy_in(&mut self, runtime: Option<PathBuf>, bytes: &[u8]) -> Option<PathBuf> {
        let name = format!("chora-icons-{}", std::process::id());
        let dir = if let Some(runtime) = runtime.filter(|d| d.is_dir()) {
            chora_scene::paths::runtime_subdir_in(Some(runtime), &name).ok()?
        } else {
            let dir = self
                .temp_dir
                .get_or_insert_with(|| {
                    std::env::temp_dir().join(format!("{name}-{}", random_suffix()))
                })
                .clone();
            chora_scene::paths::private_dir(&dir).ok()?;
            dir
        };
        let serial = SERIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = dir.join(format!("{serial}.png"));
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .ok()?;
        if file.write_all(bytes).is_err() {
            let _ = fs::remove_file(&path);
            return None;
        }
        self.copies.push_back(path.clone());
        while self.copies.len() > KEPT_COPIES {
            if let Some(old) = self.copies.pop_front() {
                let _ = fs::remove_file(old);
            }
        }
        Some(path)
    }

    /// The file of the icon of a program, else the generic program
    /// icon `application-x-executable`.
    pub fn program(&mut self, name: &str) -> Option<PathBuf> {
        self.find(name)
            .or_else(|| self.find("application-x-executable"))
    }

    /// The icon of a program as [`Self::program`], for a size in pixels.
    pub fn program_sized(&mut self, name: &str, size: u32) -> Option<PathBuf> {
        self.find_sized(name, size)
            .or_else(|| self.find_sized("application-x-executable", size))
    }

    /// The file of an icon for a size in pixels: a PNG file of that size
    /// first, as themes draw their small icons by hand, else as [`find`].
    ///
    /// [`find`]: Self::find
    pub fn find_sized(&mut self, name: &str, size: u32) -> Option<PathBuf> {
        let key = format!("{name}@{size}");
        if let Some(found) = self.found.get(&key) {
            return found.clone();
        }
        let found = search_all(&self.themes, name, Some(size));
        self.found.insert(key, found.clone());
        found
    }

    /// The first of several icons that a theme has.
    pub fn first_sized(&mut self, names: &[&str], size: u32) -> Option<PathBuf> {
        names.iter().find_map(|n| self.find_sized(n, size))
    }
}

const MAX_SENDER_BYTES: usize = 512 * 1024;
const MAX_SENDER_PIXELS: u32 = 512;

/// The bytes of a PNG file of a sender, if it is a regular file of at most
/// 512 KiB and a PNG of 1 to 512 pixels on a side. The open does not
/// block on a FIFO, and the type is checked on the handle.
fn read_small_png(path: &Path) -> Option<Vec<u8>> {
    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)
        .ok()?;
    let meta = file.metadata().ok()?;
    if !meta.is_file() || meta.len() > MAX_SENDER_BYTES as u64 {
        return None;
    }
    let mut bytes = Vec::new();
    file.take(MAX_SENDER_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > MAX_SENDER_BYTES || bytes.len() < 24 || bytes[..8] != *b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    let side =
        |at: usize| u32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    ((1..=MAX_SENDER_PIXELS).contains(&side(16)) && (1..=MAX_SENDER_PIXELS).contains(&side(20)))
        .then_some(bytes)
}

fn search_all(order: &[String], name: &str, size: Option<u32>) -> Option<PathBuf> {
    // Desktop files may name an icon by its path.
    let path = Path::new(name);
    if path.is_absolute() {
        return path.is_file().then(|| path.into());
    }
    if name.is_empty() || name.contains('/') {
        return None;
    }
    let bases: Vec<PathBuf> = crate::xdg::data_dirs()
        .iter()
        .map(|d| d.join("icons"))
        .collect();
    order
        .iter()
        .flat_map(|theme| bases.iter().flat_map(move |base| themes(base, theme)))
        .find_map(|dir| search(&dir, name, size))
        .or_else(|| {
            let pixmap = Path::new("/usr/share/pixmaps").join(format!("{name}.png"));
            pixmap.is_file().then_some(pixmap)
        })
}

/// The theme directories of a base whose name starts with a prefix.
fn themes(base: &Path, prefix: &str) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = subdirs(base)
        .into_iter()
        .filter(|d| {
            d.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with(prefix))
        })
        .collect();
    dirs.sort();
    dirs
}

/// Searches the two directory levels of a theme, for example
/// `48x48/apps` or `apps/48`.
fn search(theme: &Path, name: &str, size: Option<u32>) -> Option<PathBuf> {
    let mut best: Option<(u32, PathBuf)> = None;
    for outer in subdirs(theme) {
        for inner in subdirs(&outer) {
            let Some(candidate) = candidate(&inner, name, size) else {
                continue;
            };
            if best.as_ref().is_none_or(|(rank, _)| candidate.0 > *rank) {
                best = Some(candidate);
            }
        }
    }
    best.map(|(_, path)| path)
}

/// An icon in a directory and its rank: a PNG file of the wanted size,
/// then SVG files, then larger PNG files up to 64 pixels. Symbolic icons
/// are monochrome and do not count.
fn candidate(dir: &Path, name: &str, wanted: Option<u32>) -> Option<(u32, PathBuf)> {
    let text = dir.to_string_lossy();
    if text.contains("symbolic") {
        return None;
    }
    let svg = dir.join(format!("{name}.svg"));
    if svg.is_file() {
        return Some((1000, svg));
    }
    let png = dir.join(format!("{name}.png"));
    let size = text
        .rsplit('/')
        .take(2)
        .find_map(|part| part.split('x').next()?.parse::<u32>().ok())
        .unwrap_or(16);
    let rank = if wanted == Some(size) { 2000 } else { size };
    (png.is_file() && size <= 64).then_some((rank, png))
}

fn subdirs(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(width: u32, height: u32, padding: usize) -> Vec<u8> {
        let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        bytes.extend(width.to_be_bytes());
        bytes.extend(height.to_be_bytes());
        bytes.extend([8, 6, 0, 0, 0]);
        bytes.resize(bytes.len() + padding, 0);
        bytes
    }

    fn file(name: &str, bytes: &[u8]) -> (crate::testing::TempDir, PathBuf) {
        let dir = crate::testing::TempDir::new("chora-icons-src");
        let path = dir.join(name);
        fs::write(&path, bytes).unwrap();
        (dir, path)
    }

    fn runtime_with(tag: &str, mode: u32) -> PathBuf {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = crate::testing::TempDir::new(&format!("chora-icons-rt-{tag}")).keep();
        fs::set_permissions(&dir, fs::Permissions::from_mode(mode)).unwrap();
        dir
    }

    #[test]
    fn no_copy_is_made_in_a_runtime_dir_that_others_write() {
        let runtime = runtime_with("open", 0o777);
        let mut icons = Icons::new(["none"]);
        assert_eq!(icons.private_copy_in(Some(runtime.clone()), b"png"), None);
        assert_eq!(
            fs::read_dir(&runtime).unwrap().count(),
            0,
            "nothing is made"
        );
        fs::remove_dir_all(runtime).unwrap();
    }

    #[test]
    fn a_copy_is_private_below_a_runtime_dir_of_the_user() {
        use std::os::unix::fs::PermissionsExt as _;
        let runtime = runtime_with("ok", 0o700);
        let mut icons = Icons::new(["none"]);
        let copy = icons
            .private_copy_in(Some(runtime.clone()), b"png")
            .unwrap();
        assert!(copy.starts_with(&runtime), "{}", copy.display());
        let dir = fs::metadata(copy.parent().unwrap())
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(dir & 0o7777, 0o700);
        let file = fs::metadata(&copy).unwrap().permissions().mode();
        assert_eq!(file & 0o7777, 0o600);
        fs::remove_dir_all(runtime).unwrap();
    }

    #[test]
    fn a_sender_path_to_an_svg_is_refused() {
        let (_svg_dir, svg) = file("sender.svg", b"<svg xmlns='http://www.w3.org/2000/svg'/>");
        assert_eq!(Icons::new(["none"]).for_sender(svg.to_str().unwrap()), None);
    }

    #[test]
    fn a_sender_path_to_an_svg_named_png_is_refused() {
        let (_svg_dir, svg) = file(
            "sender-lie.png",
            b"<svg xmlns='http://www.w3.org/2000/svg'/>",
        );
        assert_eq!(Icons::new(["none"]).for_sender(svg.to_str().unwrap()), None);
    }

    #[test]
    fn an_oversized_sender_png_is_refused() {
        let (_wide_dir, wide) = file("wide.png", &png(MAX_SENDER_PIXELS + 1, 8, 0));
        let (_heavy_dir, heavy) = file("heavy.png", &png(8, 8, MAX_SENDER_BYTES));
        let mut icons = Icons::new(["none"]);
        assert_eq!(icons.for_sender(wide.to_str().unwrap()), None);
        assert_eq!(icons.for_sender(heavy.to_str().unwrap()), None);
    }

    #[test]
    fn a_small_sender_png_still_shows() {
        let (_small_dir, small) = file("small.png", &png(48, 48, 100));
        let mut icons = Icons::new(["none"]);
        let shown = icons.for_sender(small.to_str().unwrap()).unwrap();
        assert_eq!(fs::read(shown).unwrap(), png(48, 48, 100));
    }

    #[test]
    fn a_fifo_as_a_sender_path_falls_back_at_once() {
        let dir = crate::testing::TempDir::new("chora-icons-fifo");
        fs::create_dir_all(&dir).unwrap();
        let fifo = dir.join("pipe.png");
        let _ = fs::remove_file(&fifo);
        let made = std::process::Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap();
        assert!(made.success());
        let (tx, rx) = std::sync::mpsc::channel();
        let name = fifo.to_str().unwrap().to_string();
        std::thread::spawn(move || {
            let _ = tx.send(Icons::new(["none"]).for_sender(&name));
        });
        let got = rx.recv_timeout(std::time::Duration::from_secs(1));
        assert_eq!(got, Ok(None));
    }

    #[test]
    fn a_sender_file_swapped_after_the_check_is_not_read() {
        let (_path_dir, path) = file("swap.png", &png(32, 32, 0));
        let mut icons = Icons::new(["none"]);
        let shown = icons.for_sender(path.to_str().unwrap()).unwrap();
        fs::write(&path, b"<svg xmlns='http://www.w3.org/2000/svg'/>").unwrap();
        assert_eq!(fs::read(&shown).unwrap(), png(32, 32, 0));
        let mode =
            std::os::unix::fs::PermissionsExt::mode(&fs::metadata(&shown).unwrap().permissions());
        assert_eq!(mode & 0o077, 0, "the copy is private");
    }

    #[test]
    fn the_limits_hold_at_their_edges() {
        let mut icons = Icons::new(["none"]);
        let at = |icons: &mut Icons, name: &str, bytes: &[u8]| {
            let (_path_dir, path) = file(name, bytes);
            icons.for_sender(path.to_str().unwrap()).is_some()
        };
        assert!(at(&mut icons, "e512.png", &png(512, 512, 0)));
        assert!(!at(&mut icons, "e513w.png", &png(513, 512, 0)));
        assert!(!at(&mut icons, "e513h.png", &png(512, 513, 0)));
        let full = MAX_SENDER_BYTES - png(8, 8, 0).len();
        assert!(at(&mut icons, "k512.png", &png(8, 8, full)));
        assert!(!at(&mut icons, "k512p1.png", &png(8, 8, full + 1)));
        assert!(!at(&mut icons, "empty.png", b""));
    }

    #[test]
    fn the_temp_fallback_dir_has_a_random_name_and_goes_with_the_icons() {
        let predictable = format!("chora-icons-{}", std::process::id());
        let mut first = Icons::new(["none"]);
        let mut second = Icons::new(["none"]);
        let one = first.private_copy_in(None, b"png").expect("a copy");
        let two = second.private_copy_in(None, b"png").expect("a copy");
        let (one_dir, two_dir) = (one.parent().unwrap(), two.parent().unwrap());
        assert_ne!(
            one_dir.file_name().unwrap(),
            predictable.as_str(),
            "a name another user can guess"
        );
        assert_ne!(one_dir, two_dir, "one dir for each of the icons");
        let name = one_dir.file_name().unwrap().to_string_lossy();
        assert!(name.starts_with("chora-icons-"), "{name}");
        drop(first);
        drop(second);
        assert!(!one_dir.exists() && !two_dir.exists(), "the dirs go too");
    }

    #[test]
    fn the_copies_are_removed_with_the_icons() {
        let (_path_dir, path) = file("gone.png", &png(8, 8, 0));
        let mut icons = Icons::new(["none"]);
        let copy = icons.for_sender(path.to_str().unwrap()).unwrap();
        assert!(copy.is_file());
        drop(icons);
        assert!(!copy.exists());
    }
}
