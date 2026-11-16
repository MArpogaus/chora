//! Icons of the installed freedesktop icon themes, by name, for menus,
//! title bars and task buttons.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

/// Finds icons and remembers the answers.
pub struct Icons {
    themes: Vec<String>,
    found: HashMap<String, Option<PathBuf>>,
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
    /// only as a small PNG file; anything else falls back to the theme
    /// icon of the file name without its extension.
    pub fn for_sender(&mut self, name: &str) -> Option<PathBuf> {
        let path = Path::new(name);
        if !path.is_absolute() {
            return self.find(name);
        }
        if is_small_png(path) {
            return Some(path.into());
        }
        let stem = path.file_stem()?.to_str()?;
        self.find(stem)
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

fn is_small_png(path: &Path) -> bool {
    use std::io::Read;
    let Ok(file) = fs::File::open(path) else {
        return false;
    };
    let Ok(meta) = file.metadata() else {
        return false;
    };
    if !meta.is_file() || meta.len() > MAX_SENDER_BYTES as u64 {
        return false;
    }
    let mut head = [0u8; 24];
    if file.take(24).read_exact(&mut head).is_err() || head[..8] != *b"\x89PNG\r\n\x1a\n" {
        return false;
    }
    let side = |at: usize| u32::from_be_bytes([head[at], head[at + 1], head[at + 2], head[at + 3]]);
    (1..=MAX_SENDER_PIXELS).contains(&side(16)) && (1..=MAX_SENDER_PIXELS).contains(&side(20))
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

    fn file(name: &str, bytes: &[u8]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("chora-icons-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn a_sender_path_to_an_svg_is_refused() {
        let svg = file("sender.svg", b"<svg xmlns='http://www.w3.org/2000/svg'/>");
        assert_eq!(Icons::new(["none"]).for_sender(svg.to_str().unwrap()), None);
    }

    #[test]
    fn a_sender_path_to_an_svg_named_png_is_refused() {
        let svg = file(
            "sender-lie.png",
            b"<svg xmlns='http://www.w3.org/2000/svg'/>",
        );
        assert_eq!(Icons::new(["none"]).for_sender(svg.to_str().unwrap()), None);
    }

    #[test]
    fn an_oversized_sender_png_is_refused() {
        let wide = file("wide.png", &png(MAX_SENDER_PIXELS + 1, 8, 0));
        let heavy = file("heavy.png", &png(8, 8, MAX_SENDER_BYTES));
        let mut icons = Icons::new(["none"]);
        assert_eq!(icons.for_sender(wide.to_str().unwrap()), None);
        assert_eq!(icons.for_sender(heavy.to_str().unwrap()), None);
    }

    #[test]
    fn a_small_sender_png_still_shows() {
        let small = file("small.png", &png(48, 48, 100));
        assert_eq!(
            Icons::new(["none"]).for_sender(small.to_str().unwrap()),
            Some(small)
        );
    }
}
