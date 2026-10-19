//! Control Panel as a menu, as XP's start menu could show it: Display
//! opens Display Properties, Desktop lists the wallpapers, and Effects
//! the effects of the plugin. The current choice of each has a check
//! mark. Effects also turns XP's sounds on and off.

use std::path::{Path, PathBuf};

use crate::apps::{Kind, Node};

/// The scales of the outputs that Display Properties offers, in percent.
pub const SCALES: [u32; 4] = [100, 125, 150, 200];
/// The most wallpapers of a directory on the menu.
const MAX_WALLPAPERS: usize = 20;

/// What an entry of the Control Panel changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Setting {
    /// The scale of the output of the menu, in percent.
    Scale(u32),
    /// The wallpaper of every output.
    Wallpaper(PathBuf),
    /// Turns an effect on or off.
    Effect(Effect),
    /// Opens Taskbar and Start Menu Properties.
    Taskbar,
    /// Opens Date and Time Properties.
    DateTime,
    /// Opens Display Properties.
    Display,
}

/// An effect that the menu turns on and off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Wobbly windows.
    Wobbly,
    /// The magic lamp.
    Lamp,
    /// The desktop cube.
    Cube,
    /// XP's sounds.
    Sounds,
}

/// Which effects are on, for the session.
#[allow(clippy::struct_excessive_bools)] // independent switches
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Toggles {
    /// Wobbly windows.
    pub wobbly: bool,
    /// The magic lamp.
    pub lamp: bool,
    /// The desktop cube.
    pub cube: bool,
    /// XP's sounds.
    pub sounds: bool,
}

impl Default for Toggles {
    fn default() -> Self {
        Self {
            wobbly: true,
            lamp: true,
            cube: true,
            sounds: true,
        }
    }
}

impl Toggles {
    fn get(self, effect: Effect) -> bool {
        match effect {
            Effect::Wobbly => self.wobbly,
            Effect::Lamp => self.lamp,
            Effect::Cube => self.cube,
            Effect::Sounds => self.sounds,
        }
    }

    /// Turns an effect on or off.
    pub fn flip(&mut self, effect: Effect) {
        let on = match effect {
            Effect::Wobbly => &mut self.wobbly,
            Effect::Lamp => &mut self.lamp,
            Effect::Cube => &mut self.cube,
            Effect::Sounds => &mut self.sounds,
        };
        *on = !*on;
    }
}

/// The PNG, JPEG and SVG files of directories, without their subdirectories:
/// each directory sorted by name, at most 20 of each.
pub fn wallpapers(dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for dir in dirs {
        let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.is_file()
                    && (crate::theme::is_jpeg(p)
                        || p.extension().is_some_and(|e| {
                            e.eq_ignore_ascii_case("png") || e.eq_ignore_ascii_case("svg")
                        }))
            })
            .collect();
        files.sort();
        files.truncate(MAX_WALLPAPERS);
        out.extend(files);
    }
    out
}

/// The cascade of the Control Panel for the wallpaper and the effects.
pub fn menu(wallpaper: Option<&Path>, walls: &[PathBuf], toggles: Toggles) -> Vec<Node> {
    let pick = |title: String, setting, checked| Node {
        title,
        icon: String::new(),
        kind: Kind::Pick(setting, checked),
    };
    let walls = walls
        .iter()
        .map(|w| {
            let stem = w.file_stem().unwrap_or_default().to_string_lossy();
            let title = name_of(&stem);
            pick(
                title,
                Setting::Wallpaper(w.clone()),
                Some(w.as_path()) == wallpaper,
            )
        })
        .collect();
    let effects = [
        ("Wobbly Windows", Effect::Wobbly),
        ("Magic Lamp", Effect::Lamp),
        ("Desktop Cube", Effect::Cube),
        ("Sounds", Effect::Sounds),
    ]
    .into_iter()
    .map(|(title, e)| pick(title.into(), Setting::Effect(e), toggles.get(e)))
    .collect();
    let group = |title: &str, icon: &str, children| Node {
        title: title.into(),
        icon: icon.into(),
        kind: Kind::Group(children),
    };
    vec![
        Node {
            title: "Date and Time".into(),
            icon: "preferences-system-time".into(),
            kind: Kind::Pick(Setting::DateTime, false),
        },
        group("Desktop", "preferences-desktop-wallpaper", walls),
        Node {
            title: "Display".into(),
            icon: "preferences-desktop-display".into(),
            kind: Kind::Pick(Setting::Display, false),
        },
        group("Effects", "preferences-desktop-effects", effects),
        Node {
            title: "Taskbar and Start Menu".into(),
            icon: "preferences-system-windows".into(),
            kind: Kind::Pick(Setting::Taskbar, false),
        },
    ]
}

/// The name of a wallpaper file as XP shows it: "blue-hills" is "Blue
/// hills", and XP's short file names have their long names.
pub fn name_of(stem: &str) -> String {
    let long = match stem {
        "moon" => "Moon flower",
        "purplef" => "Purple flower",
        "redmoon" => "Red moon desert",
        "stoneh" => "Stonehenge",
        "vortec" => "Vortec space",
        _ => "",
    };
    if !long.is_empty() {
        return long.into();
    }
    let words = stem.replace(['-', '_'], " ");
    let mut chars = words.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn picks(node: &Node) -> Vec<(&str, bool)> {
        let Kind::Group(children) = &node.kind else {
            panic!("a group");
        };
        children
            .iter()
            .map(|c| match &c.kind {
                Kind::Pick(_, checked) => (c.title.as_str(), *checked),
                _ => panic!("a pick"),
            })
            .collect()
    }

    #[test]
    fn the_current_choices_have_check_marks() {
        let walls = [PathBuf::from("/w/a.png"), PathBuf::from("/w/b.svg")];
        let mut toggles = Toggles::default();
        toggles.flip(Effect::Lamp);
        let menu = menu(Some(Path::new("/w/b.svg")), &walls, toggles);
        assert_eq!(picks(&menu[1]), [("A", false), ("B", true)]);
        assert_eq!(name_of("blue_hills-2"), "Blue hills 2");
        assert_eq!(name_of("purplef"), "Purple flower");
        assert_eq!(menu[2].kind, Kind::Pick(Setting::Display, false));
        assert_eq!(
            picks(&menu[3]),
            [
                ("Wobbly Windows", true),
                ("Magic Lamp", false),
                ("Desktop Cube", true),
                ("Sounds", true)
            ]
        );
    }

    #[test]
    fn wallpapers_are_the_pictures_of_the_first_level() {
        let dir = std::env::temp_dir().join(format!("chora-xp-walls-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("sub")).expect("a directory");
        for name in ["b.PNG", "a.svg", "c.jpg", "d.txt", "sub/e.png"] {
            std::fs::write(dir.join(name), "").expect("a file");
        }
        for i in 0..25 {
            std::fs::write(dir.join(format!("z{i:02}.png")), "").expect("a file");
        }
        let found = wallpapers(&[dir.clone(), dir.join("missing")]);
        assert_eq!(found.len(), MAX_WALLPAPERS);
        assert_eq!(
            found[..3],
            [dir.join("a.svg"), dir.join("b.PNG"), dir.join("c.jpg")]
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
