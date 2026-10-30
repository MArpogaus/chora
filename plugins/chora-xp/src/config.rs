//! The settings of the plugin (D62), a YAML file that lists every option.
//! The file `xp.yaml` next to the crate is the default and the reference.

use std::path::PathBuf;

use chora_kit::settings::Settings;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{apps, control::Toggles, startmenu, taskbar, theme::Theme, wallpaper};

/// The default settings.
pub const DEFAULT: &str = include_str!("../xp.yaml");

/// A program of the start menu.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub title: String,
    /// A shell command.
    pub command: String,
    /// The name of the icon in the icon theme.
    pub icon: String,
}

/// The picture of the desktop.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Picture {
    /// XP's Bliss when it is installed, else Bløff of the theme.
    Default,
    /// The color only.
    None,
    /// A PNG, JPEG or SVG file, by its absolute path.
    File(PathBuf),
}

/// The background of the desktop.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Wallpaper {
    pub picture: Picture,
    pub position: wallpaper::Position,
    /// Red, green and blue around a centered picture and without one.
    pub color: [u8; 3],
}

impl Wallpaper {
    /// The style that the wallpaper nodes draw, with Bløff of the theme
    /// as the default without Bliss.
    pub fn style(&self, theme: &Theme) -> wallpaper::Style {
        let file = match &self.picture {
            Picture::Default => Some(
                chora_kit::xdg::data_file("wallpapers/WindowsXP/bliss.jpg")
                    .unwrap_or_else(|| theme.path("wallpaper/bloff.png")),
            ),
            Picture::None => None,
            Picture::File(file) => Some(file.clone()),
        };
        wallpaper::Style {
            file,
            position: self.position,
            color: self.color,
        }
    }

    /// The wallpaper of a style that a dialog chose.
    pub fn of(style: &wallpaper::Style) -> Self {
        Self {
            picture: style.file.clone().map_or(Picture::None, Picture::File),
            position: style.position,
            color: style.color,
        }
    }
}

/// The shell commands of the buttons of Turn Off and Log Off.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Session {
    /// Without it, Turn Off ends chora-xp.
    pub turn_off: Option<String>,
    /// Without it, Restart starts chora-xp again.
    pub restart: Option<String>,
    /// Without it, Stand By is disabled.
    pub stand_by: Option<String>,
    /// Without it, Log Off closes the windows and ends chora-xp. The
    /// default `choractl quit` ends chora.
    pub log_off: Option<String>,
}

/// The settings of the plugin.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The shell command of the terminal, for Super+e and the desktop
    /// files that need one.
    pub terminal: String,
    /// The shell command of Run, and of All Programs when no desktop file
    /// lists a program.
    pub launcher: String,
    /// The shell command of the file manager; the folder follows it.
    pub file_manager: String,
    /// The programs at the top left of the start menu.
    pub programs: Vec<Program>,
    /// The format of the clock of the taskbar, as in strftime.
    pub clock_format: String,
    pub wallpaper: Wallpaper,
    pub taskbar: taskbar::Settings,
    pub effects: Toggles,
    pub session: Session,
}

impl Config {
    /// The programs and places of the start menu.
    pub fn menu(&self) -> startmenu::Config {
        startmenu::Config {
            user: std::env::var("USER")
                .or_else(|_| std::env::var("LOGNAME"))
                .unwrap_or_else(|_| "User".into()),
            programs: self.programs.clone(),
            launcher: self.launcher.clone(),
            file_manager: self.file_manager.clone(),
            apps: apps::load(&self.terminal),
        }
    }
}

impl Settings for Config {
    fn check(&self) -> Result<(), String> {
        let session = &self.session;
        let commands = [
            ("terminal", Some(&self.terminal)),
            ("launcher", Some(&self.launcher)),
            ("file_manager", Some(&self.file_manager)),
            ("session.turn_off", session.turn_off.as_ref()),
            ("session.restart", session.restart.as_ref()),
            ("session.stand_by", session.stand_by.as_ref()),
            ("session.log_off", session.log_off.as_ref()),
        ];
        if let Some((key, _)) = commands
            .iter()
            .find(|(_, c)| c.is_some_and(|c| c.trim().is_empty()))
        {
            return Err(format!("{key}: an empty command"));
        }
        if self
            .programs
            .iter()
            .any(|p| p.title.is_empty() || p.command.trim().is_empty())
        {
            return Err("programs: a program without a title or a command".into());
        }
        if chrono::format::StrftimeItems::new(&self.clock_format)
            .any(|item| item == chrono::format::Item::Error)
        {
            return Err(format!(
                "clock_format: {:?} is no strftime format",
                self.clock_format
            ));
        }
        if let Picture::File(file) = &self.wallpaper.picture
            && !file.is_absolute()
        {
            return Err(format!(
                "wallpaper.picture: {} is no absolute path",
                file.display()
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chora_kit::settings::{File, Store, parse};
    use serde_json::json;

    use super::*;

    #[test]
    fn the_default_lists_the_defaults_of_xp() {
        let config: Config = parse(DEFAULT).unwrap();
        assert_eq!(config.file_manager, "thunar");
        assert_eq!(config.wallpaper.picture, Picture::Default);
        assert_eq!(config.wallpaper.color, wallpaper::DESKTOP_BLUE);
        assert_eq!(config.taskbar, taskbar::Settings::default());
        assert_eq!(config.effects, Toggles::default());
        let session = config.session;
        assert_eq!(session.turn_off.as_deref(), Some("systemctl poweroff"));
        assert_eq!(session.restart.as_deref(), Some("systemctl reboot"));
        assert_eq!(session.stand_by.as_deref(), Some("systemctl suspend"));
        assert_eq!(session.log_off.as_deref(), Some("choractl quit"));
    }

    #[test]
    fn a_wrong_value_fails() {
        let config: Config = parse(DEFAULT).unwrap();
        let wrong = [
            ("terminal", json!("")),
            ("clock_format", json!("%Q")),
            (
                "wallpaper",
                json!({"picture": {"file": "bliss.jpg"}, "position": "fill", "color": [0, 78, 152]}),
            ),
            (
                "wallpaper",
                json!({"picture": "default", "position": "diagonal", "color": [0, 78, 152]}),
            ),
            (
                "programs",
                json!([{"title": "", "command": "foot", "icon": "foot"}]),
            ),
            (
                "session",
                json!({"turn_off": "", "restart": null, "stand_by": null, "log_off": null}),
            ),
        ];
        for (key, value) in wrong {
            let mut changed = serde_json::to_value(&config).unwrap();
            changed[key] = value.clone();
            let err = serde_json::from_value::<Config>(changed)
                .map_err(|e| e.to_string())
                .and_then(|c| c.check());
            assert!(err.is_err(), "{key}: {value} passed");
        }
    }

    #[test]
    fn set_saves_a_wallpaper_that_the_next_start_reads() {
        let dir = std::env::temp_dir().join(format!("chora-xp-config-{}", std::process::id()));
        let path = dir.join("chora-xp.yaml");
        let mut file = File::<Config>::load(path.clone(), DEFAULT).unwrap();
        let red = json!({"picture": {"file": "/tmp/red.jpg"}, "position": "center", "color": [255, 0, 0]});
        file.set(json!({"wallpaper": red})).unwrap();
        let saved = File::<Config>::load(path, DEFAULT).unwrap();
        assert_eq!(
            saved.value.wallpaper.picture,
            Picture::File("/tmp/red.jpg".into())
        );
        assert_eq!(saved.value.wallpaper.position, wallpaper::Position::Center);
        assert_eq!(saved.value.terminal, "foot", "the other keys stay");
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// A file that names a few keys keeps the defaults of the others, and
    /// `log_off: null` keeps the fallback of chora-xp.
    #[test]
    fn a_file_with_a_few_keys_takes_the_defaults() {
        let dir = std::env::temp_dir().join(format!("chora-xp-partial-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("chora-xp.yaml");
        std::fs::write(&path, "terminal: kitty\nsession:\n  log_off: null\n").unwrap();
        let file = File::<Config>::load(path, DEFAULT).unwrap();
        assert_eq!(file.value.terminal, "kitty");
        assert_eq!(file.value.file_manager, "thunar");
        let session = &file.value.session;
        assert_eq!(session.turn_off.as_deref(), Some("systemctl poweroff"));
        assert_eq!(session.log_off, None);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_style_of_a_dialog_is_a_wallpaper_again() {
        let style = wallpaper::Style {
            file: None,
            position: wallpaper::Position::Tile,
            color: [1, 2, 3],
        };
        let wallpaper = Wallpaper::of(&style);
        assert_eq!(wallpaper.picture, Picture::None);
        let (theme, cache) = crate::theme::tests::theme("config");
        assert_eq!(wallpaper.style(&theme), style);
        let _ = std::fs::remove_dir_all(cache);
    }
}
