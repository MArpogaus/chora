//! Icons of the installed freedesktop icon themes, by name, for the start
//! menu, the title bars and the task buttons.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

/// The themes to search, in order. XP's own icons come from `WinXPSVG`,
/// when it is installed.
const THEMES: [&str; 6] = [
    "WinXPSVG", "Adwaita", "breeze", "Papirus", "Humanity", "hicolor",
];

/// Finds icons and remembers the answers.
#[derive(Default)]
pub struct Icons {
    found: HashMap<String, Option<PathBuf>>,
}

impl Icons {
    /// The file of an icon: an SVG file if there is one, else the largest
    /// PNG file up to 64 pixels.
    pub fn find(&mut self, name: &str) -> Option<PathBuf> {
        if let Some(found) = self.found.get(name) {
            return found.clone();
        }
        let found = search_all(name);
        self.found.insert(name.into(), found.clone());
        found
    }
}

/// The directories with icon themes.
fn bases() -> Vec<PathBuf> {
    let home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| Path::new(&h).join(".local/share")));
    let dirs = std::env::var("XDG_DATA_DIRS")
        .ok()
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".into());
    home.into_iter()
        .chain(dirs.split(':').map(PathBuf::from))
        .map(|d| d.join("icons"))
        .collect()
}

fn search_all(name: &str) -> Option<PathBuf> {
    if name.is_empty() || name.contains('/') {
        return None;
    }
    let bases = bases();
    THEMES
        .iter()
        .flat_map(|theme| bases.iter().map(move |base| base.join(theme)))
        .find_map(|dir| search(&dir, name))
        .or_else(|| {
            let pixmap = Path::new("/usr/share/pixmaps").join(format!("{name}.png"));
            pixmap.is_file().then_some(pixmap)
        })
}

/// Searches the two directory levels of a theme, for example
/// `48x48/apps` or `apps/48`.
fn search(theme: &Path, name: &str) -> Option<PathBuf> {
    let mut best: Option<(u32, PathBuf)> = None;
    for outer in subdirs(theme) {
        for inner in subdirs(&outer) {
            let Some(candidate) = candidate(&inner, name) else {
                continue;
            };
            if best.as_ref().is_none_or(|(rank, _)| candidate.0 > *rank) {
                best = Some(candidate);
            }
        }
    }
    best.map(|(_, path)| path)
}

/// An icon in a directory and its rank: SVG files first, then larger PNG
/// files up to 64 pixels. Symbolic icons are monochrome and do not count.
fn candidate(dir: &Path, name: &str) -> Option<(u32, PathBuf)> {
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
    (png.is_file() && size <= 64).then_some((size, png))
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
