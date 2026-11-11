//! Control Panel as a menu, as XP's start menu could show it: Display
//! opens Display Properties, Desktop lists the wallpapers, and Effects
//! the effects of the plugin. The current choice of each has a check
//! mark. Effects also turns XP's sounds on and off. Sound mutes and sets
//! the volume, and Keyboard Shortcuts lists the keys that every window
//! manager shares.

use std::path::{Path, PathBuf};

use chora_kit::volume::{Change, Volume};

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
    /// Changes the volume of the default sound output.
    Volume(Change),
    /// Gives the focused window to the window manager of this name (D55).
    Give(&'static str),
    /// Tells something and changes nothing.
    Note,
}

/// The volumes of Sound, in percent.
const VOLUMES: [u32; 4] = [25, 50, 75, 100];

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

/// Which effects are on.
#[allow(clippy::struct_excessive_bools)] // independent switches
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
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

impl Toggles {
    fn get(self, effect: Effect) -> bool {
        match effect {
            Effect::Wobbly => self.wobbly,
            Effect::Lamp => self.lamp,
            Effect::Cube => self.cube,
            Effect::Sounds => self.sounds,
        }
    }

    /// The effects of the shared driver that are on.
    pub fn kinds(self) -> Vec<chora_kit::effects::Kind> {
        use chora_kit::effects::Kind::{Lamp, Wobbly};
        [(self.wobbly, Wobbly), (self.lamp, Lamp)]
            .into_iter()
            .filter_map(|(on, kind)| on.then_some(kind))
            .collect()
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
pub fn menu(
    wallpaper: Option<&Path>,
    walls: &[PathBuf],
    toggles: Toggles,
    volume: Option<Volume>,
) -> Vec<Node> {
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
            // XP draws a calendar; the WindowsXP theme has no
            // preferences-system-time.
            icon: "x-office-calendar".into(),
            kind: Kind::Pick(Setting::DateTime, false),
        },
        group("Desktop", "preferences-desktop-wallpaper", walls),
        Node {
            title: "Display".into(),
            icon: "preferences-desktop-display".into(),
            kind: Kind::Pick(Setting::Display, false),
        },
        group("Effects", "preferences-desktop-effects", effects),
        group(
            "Keyboard Shortcuts",
            "preferences-desktop-keyboard",
            shortcuts(),
        ),
        group("Sound", "audio-volume-medium", sound(volume)),
        Node {
            title: "Taskbar and Start Menu".into(),
            icon: "preferences-system-windows".into(),
            kind: Kind::Pick(Setting::Taskbar, false),
        },
    ]
}

/// The rows of Sound: Mute and the volumes, the current one checked
/// when it is one of them;
/// without a sound output one row that says so.
fn sound(volume: Option<Volume>) -> Vec<Node> {
    let pick = |title: String, setting, checked| Node {
        title,
        icon: String::new(),
        kind: Kind::Pick(setting, checked),
    };
    let Some(volume) = volume else {
        return vec![pick(chora_kit::volume::label(None), Setting::Note, false)];
    };
    let current = (!volume.muted).then_some(volume.percent);
    let mut rows = vec![pick(
        "Mute".into(),
        Setting::Volume(Change::Mute),
        volume.muted,
    )];
    rows.extend(VOLUMES.into_iter().map(|v| {
        pick(
            format!("{v}%"),
            Setting::Volume(Change::Set(v)),
            Some(v) == current,
        )
    }));
    rows
}

/// The rows of Keyboard Shortcuts: the keys that give the focused window
/// to another window manager, and the key of the chat, which also opens
/// it from here.
fn shortcuts() -> Vec<Node> {
    let mut rows: Vec<Node> = (chora_kit::give::KEYS.into_iter())
        .map(|(key, to)| Node {
            title: format!("{key}: window to {to}"),
            icon: String::new(),
            kind: Kind::Pick(Setting::Give(to), false),
        })
        .collect();
    rows.push(Node {
        title: format!("{CHAT_KEY}: chat"),
        icon: String::new(),
        kind: Kind::Run(CHAT.into()),
    });
    rows
}

/// The key that opens the chat, on every window manager.
pub const CHAT_KEY: &str = "Super+a";
/// The command that opens the chat.
pub const CHAT: &str = "chora-chat";

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
    use chora_kit::{
        effects::Kind as Shared,
        volume::{Change, Volume},
    };

    use super::*;

    #[test]
    fn xp_keeps_its_wobble_and_lamp() {
        let mut toggles = crate::config::defaults().effects;
        assert_eq!(toggles.kinds(), [Shared::Wobbly, Shared::Lamp]);
        toggles.flip(Effect::Wobbly);
        assert_eq!(toggles.kinds(), [Shared::Lamp]);
        toggles.flip(Effect::Lamp);
        assert_eq!(toggles.kinds(), Vec::<Shared>::new());
    }

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
        let mut toggles = crate::config::defaults().effects;
        toggles.flip(Effect::Lamp);
        let menu = menu(Some(Path::new("/w/b.svg")), &walls, toggles, None);
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

    fn group<'a>(menu: &'a [Node], title: &str) -> &'a Node {
        menu.iter().find(|n| n.title == title).expect(title)
    }

    #[test]
    fn sound_shows_the_volume_and_mutes() {
        let toggles = crate::config::defaults().effects;
        let at = |percent, muted| Some(Volume { percent, muted });
        let loud = menu(None, &[], toggles, at(50, false));
        assert_eq!(
            picks(group(&loud, "Sound")),
            [
                ("Mute", false),
                ("25%", false),
                ("50%", true),
                ("75%", false),
                ("100%", false)
            ]
        );
        let Kind::Group(rows) = &group(&loud, "Sound").kind else {
            panic!("a group");
        };
        assert_eq!(
            rows[0].kind,
            Kind::Pick(Setting::Volume(Change::Mute), false)
        );
        assert_eq!(
            rows[4].kind,
            Kind::Pick(Setting::Volume(Change::Set(100)), false)
        );
        let muted = menu(None, &[], toggles, at(50, true));
        assert_eq!(
            picks(group(&muted, "Sound"))[..3],
            [("Mute", true), ("25%", false), ("50%", false)]
        );
        let silent = menu(None, &[], toggles, None);
        assert_eq!(picks(group(&silent, "Sound")), [("No sound", false)]);
    }

    #[test]
    fn the_keys_of_every_window_manager_are_on_the_menu() {
        let menu = menu(None, &[], crate::config::defaults().effects, None);
        let Kind::Group(rows) = &group(&menu, "Keyboard Shortcuts").kind else {
            panic!("a group");
        };
        let titles: Vec<&str> = rows.iter().map(|r| r.title.as_str()).collect();
        assert_eq!(
            titles,
            [
                "Super+Alt+1: window to tiling",
                "Super+Alt+2: window to xp",
                "Super+Alt+3: window to mc",
                "Super+Alt+4: window to construct",
                "Super+a: chat",
            ]
        );
        assert_eq!(rows[0].kind, Kind::Pick(Setting::Give("tiling"), false));
        assert_eq!(rows[4].kind, Kind::Run("chora-chat".into()));
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
