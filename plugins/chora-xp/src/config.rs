//! The settings of the plugin (D62), a YAML file that lists every option.
//! The file `xp.yaml` next to the crate is the default and the reference.

use std::path::PathBuf;

use chora_kit::actions::{Step, System, Turn};
use chora_kit::settings::{Common, Settings};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{control::Toggles, taskbar, theme::Theme, wallpaper};

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
    /// The file of the picture, with XP's Bliss when it is installed, else
    /// Bløff of the theme, as the default.
    pub fn file(&self, theme: &Theme) -> Option<PathBuf> {
        match &self.picture {
            Picture::Default => Some(
                chora_kit::xdg::data_file("wallpapers/WindowsXP/bliss.jpg")
                    .unwrap_or_else(|| theme.path("wallpaper/bloff.png")),
            ),
            Picture::None => None,
            Picture::File(file) => Some(file.clone()),
        }
    }

    /// The wallpaper with the file of the default, as Display Properties
    /// lists it.
    pub fn resolved(&self, theme: &Theme) -> Self {
        Self {
            picture: self.file(theme).map_or(Picture::None, Picture::File),
            ..self.clone()
        }
    }

    /// A picture filling the screen, on XP's blue.
    #[cfg(test)]
    pub fn new(file: PathBuf) -> Self {
        Self {
            picture: Picture::File(file),
            position: wallpaper::Position::default(),
            color: wallpaper::DESKTOP_BLUE,
        }
    }
}

/// The shell commands of the buttons of Turn Off and Log Off.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Session {
    /// Without it, Turn Off runs the shared power script.
    pub turn_off: Option<String>,
    /// Without it, Restart runs the shared power script.
    pub restart: Option<String>,
    /// Without it, Stand By is disabled.
    pub stand_by: Option<String>,
    /// Without it, Log Off runs the shared power script.
    pub log_off: Option<String>,
}

/// What a binding of xp does besides the actions of every look.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Look {
    /// Opens the file manager.
    Files,
    /// Shows the desktop, or brings the windows back.
    Desktop,
    /// A key pressed and released alone opens or closes the Start menu
    /// (the Super key).
    Tap,
    /// Opens or closes the Start menu.
    Start,
    /// XP's Alt+Tab: opens the switcher or moves its choice.
    Switch(Step),
    /// Moves the focused window along with a turn of the cube.
    Carry(Turn),
    /// With a pointer button: the pointer turns the cube.
    Cube,
}

/// What a binding does: an action of every look, or one of xp.
pub type KeyAction = chora_kit::actions::Action<Look>;

impl From<Look> for KeyAction {
    fn from(look: Look) -> Self {
        Self::Look(look)
    }
}

/// Whether xp does something for a system action. A step through the desks
/// is no key of xp (the desktops are the faces of its cube), the terminal
/// is a row of the Start menu, a drag belongs to the flavors that claim
/// windows, and a run or a give is no feature of xp (yet): a binding to
/// one is dropped with a warning.
#[must_use]
pub const fn acts_in_xp(system: &System) -> bool {
    !matches!(
        system,
        System::Desk(_) | System::Terminal | System::Drag | System::Run(_) | System::Give(_)
    )
}

/// Whether the key of a binding is a pointer button, such as `BTN_LEFT`.
fn is_button(binding: &str) -> bool {
    binding
        .parse::<chora_scene::Binding>()
        .is_ok_and(|b| b.key.to_ascii_uppercase().starts_with("BTN_"))
}

/// The settings of the plugin.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The shell command of the terminal, for the Terminal of the start
    /// menu and the desktop files that need one.
    pub terminal: String,
    /// The bindings and the switch of the tests, as in every look. With
    /// `test_accept_injected_keys` the keys of `input.inject` count as the
    /// user's; off, the bindings ignore injected keys.
    #[serde(flatten)]
    pub common: Common<Look>,
    /// The shell command of Run and of Super+Space, and of All Programs
    /// when no desktop file lists a program.
    pub launcher: String,
    /// The shell command of the file manager, for Super+e, the places of
    /// the start menu and the folders of the desktop; the folder follows
    /// it.
    pub file_manager: String,
    /// The shell command of the plain text editor, Notepad of the start
    /// menu. Without it, or when its program is not installed, the first
    /// installed editor with a window serves; a terminal editor serves
    /// only when it is set here.
    #[serde(default)]
    pub editor: Option<String>,
    /// The programs at the top left of the start menu, below the
    /// terminal.
    pub programs: Vec<Program>,
    /// Each taskbar lists only the windows of its own output. Off, every
    /// taskbar of a screen lists all windows. The taskbar of a virtual
    /// output always lists its own windows only.
    #[serde(default)]
    pub tasks_per_output: bool,
    /// The format of the clock of the taskbar, as in strftime.
    pub clock_format: String,
    pub wallpaper: Wallpaper,
    pub taskbar: taskbar::Settings,
    pub effects: Toggles,
    pub session: Session,
}

impl Settings for Config {
    fn refuse(&mut self) -> Vec<String> {
        let mut warnings = Vec::new();
        self.common.bindings.retain(|key, action| {
            let unfit = matches!(action, KeyAction::System(s) if !acts_in_xp(s));
            let odd = is_button(key) != matches!(action, KeyAction::Look(Look::Cube));
            if unfit {
                warnings.push(format!(
                    "chora-xp: the binding {key} is dropped: xp has no action for {action:?}"
                ));
            } else if odd {
                warnings.push(format!(
                    "chora-xp: the binding {key} is dropped: only a pointer button turns \
                     the cube, and a pointer button does nothing else ({action:?})"
                ));
            }
            !unfit && !odd
        });
        warnings
    }

    /// Two spellings of one key would lose one grab at runtime, so they
    /// fail the settings, as a binding that is no key does.
    fn check(&self) -> Result<(), String> {
        let mut keys = std::collections::BTreeMap::new();
        for binding in self.common.bindings.keys() {
            let parsed: chora_scene::Binding = binding
                .parse()
                .map_err(|e| format!("bindings: {binding}: {e}"))?;
            if let Some(other) = keys.insert(parsed.to_string(), binding) {
                return Err(format!("bindings: {binding}: the same key as {other}"));
            }
        }
        let session = &self.session;
        let commands = [
            ("terminal", Some(&self.terminal)),
            ("launcher", Some(&self.launcher)),
            ("file_manager", Some(&self.file_manager)),
            ("editor", self.editor.as_ref()),
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
        // The clock fits the tray of the taskbar; a long format such as
        // 600 times %A made a text of kilobytes each minute (W10).
        if self.clock_format.chars().count() > 64 {
            return Err("clock_format: at most 64 characters".into());
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
        // A missing file may come back, for example with a disk; a device
        // or a FIFO never is a picture.
        if let Picture::File(file) = &self.wallpaper.picture
            && file.exists()
        {
            chora_kit::check_file(file).map_err(|e| format!("wallpaper.picture: {e}"))?;
        }
        Ok(())
    }
}

/// The way out of an old settings file, for the log: the key `assistant`
/// moved into `chora-assistant.yaml` as `enabled`.
pub fn migration_hint(error: &str) -> Option<&'static str> {
    error.contains("unknown field `assistant`").then_some(
        "assistant.enabled moved to chora-assistant.yaml as enabled; remove the key \
         assistant (Migration in docs/configuration.md)",
    )
}

/// The default settings, for the tests.
#[cfg(test)]
pub fn defaults() -> Config {
    chora_kit::settings::parse(DEFAULT).unwrap()
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
        let session = config.session;
        assert_eq!(session.turn_off, None);
        assert_eq!(session.restart, None);
        assert_eq!(session.stand_by.as_deref(), Some("systemctl suspend"));
        assert_eq!(session.log_off, None);
    }

    #[test]
    fn the_old_assistant_key_is_named_with_the_way_out() {
        let dir = std::env::temp_dir().join(format!("chora-xp-old-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("chora-xp.yaml");
        std::fs::write(&path, "terminal: kitty\nassistant:\n  enabled: false\n").unwrap();
        let file = File::<Config>::load(path, DEFAULT).unwrap();
        let err = file.broken().expect("the old key breaks the file");
        assert!(err.contains("assistant"), "{err}");
        let hint = migration_hint(err).expect("a hint");
        assert!(hint.contains("chora-assistant.yaml"), "{hint}");
        assert_eq!(migration_hint("terminal: an empty command"), None);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_wrong_value_fails() {
        let config: Config = parse(DEFAULT).unwrap();
        let wrong = [
            ("terminal", json!("")),
            ("clock_format", json!("%Q")),
            ("clock_format", json!("%A ".repeat(22))),
            (
                "wallpaper",
                json!({"picture": {"file": "bliss.jpg"}, "position": "fill", "color": [0, 78, 152]}),
            ),
            (
                "wallpaper",
                json!({"picture": {"file": "/dev/zero"}, "position": "fill", "color": [0, 78, 152]}),
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
        file.save().unwrap();
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
        assert_eq!(session.stand_by.as_deref(), Some("systemctl suspend"));
        assert_eq!(session.log_off, None);
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// Every key that xp handles on 1d1aa5fd (its hard-coded `KEYS`), with
    /// the action that the default `bindings` give it (F2). A key of a
    /// `System` action has the same spelling as in every other look.
    const HANDLED: [(&str, &str); 23] = [
        ("Super+Super_L", r#""tap""#),
        ("Super+Super_R", r#""tap""#),
        ("Ctrl+Escape", r#""start""#),
        ("Super+d", r#""desktop""#),
        ("Super+space", r#""launcher""#),
        ("Super+F1", r#""keys""#),
        ("Super+slash", r#""keys""#),
        ("Super+Shift+Escape", r#""lock""#),
        ("Super+Shift+e", r#""power""#),
        ("Super+Print", r#""screenshot""#),
        ("Super+BTN_LEFT", r#""cube""#),
        ("Alt+F4", r#""close""#),
        ("Alt+Tab", r#"{"switch":"next"}"#),
        ("Alt+Shift+Tab", r#"{"switch":"previous"}"#),
        ("Super+e", r#""files""#),
        ("Super+a", r#""chat""#),
        ("XF86AudioRaiseVolume", r#"{"volume":"up"}"#),
        ("XF86AudioLowerVolume", r#"{"volume":"down"}"#),
        ("XF86AudioMute", r#"{"volume":"mute"}"#),
        ("Ctrl+Alt+Left", r#"{"cube":"left"}"#),
        ("Ctrl+Alt+Right", r#"{"cube":"right"}"#),
        ("Ctrl+Alt+Shift+Left", r#"{"carry":"left"}"#),
        ("Ctrl+Alt+Shift+Right", r#"{"carry":"right"}"#),
    ];

    /// The default file binds exactly the keys that xp handled before it
    /// had `bindings`, each to the action of its hard-coded key.
    #[test]
    fn the_default_binds_every_key_that_xp_handled_on_1d1aa5fd() {
        let file: serde_json::Value = serde_saphyr::from_str(DEFAULT).unwrap();
        let table: std::collections::BTreeMap<String, String> = file
            .get("bindings")
            .and_then(|b| b.as_object())
            .map(|b| b.iter().map(|(k, v)| (k.clone(), v.to_string())).collect())
            .unwrap_or_default();
        let want: std::collections::BTreeMap<String, String> = HANDLED
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        assert_eq!(table, want);
    }

    /// One YAML binding gives one `System` action in every look: the file
    /// `tests/configs/system-bindings.yaml` loads to its `.table` here, as
    /// in mc and construct.
    #[test]
    fn one_binding_gives_the_same_system_action_in_every_look() {
        use std::fmt::Write;
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/configs");
        let text = std::fs::read_to_string(dir.join("system-bindings.yaml")).unwrap();
        let common: Common<Look> = serde_saphyr::from_str(&text).unwrap();
        let mut table = String::new();
        for (key, action) in &common.bindings {
            writeln!(table, "{key}\t{}", serde_json::to_string(action).unwrap()).unwrap();
        }
        let golden = std::fs::read_to_string(dir.join("system-bindings.table")).unwrap();
        assert_eq!(table, golden);
    }

    /// The default loads, and xp has an action for each of its bindings.
    #[test]
    fn the_default_bindings_all_act_in_xp() {
        let mut config: Config = parse(DEFAULT).unwrap();
        assert_eq!(config.refuse(), Vec::<String>::new());
        assert_eq!(config.common.bindings.len(), HANDLED.len());
        for (key, want) in HANDLED {
            let action = &config.common.bindings[key];
            assert_eq!(serde_json::to_string(action).unwrap(), want, "{key}");
        }
    }

    /// A binding that xp has no action for is dropped with one warning that
    /// names the key; the rest of the file loads.
    #[test]
    fn a_system_action_without_meaning_in_xp_is_dropped_with_a_warning() {
        for unfit in [
            "{desk: next}",
            "terminal",
            "drag",
            "{run: foot}",
            "{give: mc}",
        ] {
            let mut config: Config = parse(DEFAULT).unwrap();
            let action: KeyAction =
                serde_json::from_value(serde_saphyr::from_str::<serde_json::Value>(unfit).unwrap())
                    .unwrap();
            config.common.bindings.insert("Super+h".into(), action);
            let warnings = config.refuse();
            assert_eq!(warnings.len(), 1, "{unfit}: {warnings:?}");
            assert!(warnings[0].contains("Super+h"), "{unfit}: {warnings:?}");
            assert_eq!(config.common.bindings.get("Super+h"), None, "{unfit}");
            assert!(config.common.bindings.contains_key("Super+F1"), "the rest");
        }
    }

    /// A pointer button runs `cube` or no other action here, and `cube`
    /// needs a pointer button (rw-1): the others are dropped with one
    /// warning that names the key.
    #[test]
    fn a_button_and_the_cube_go_together_or_are_dropped() {
        for (key, action) in [
            ("Super+BTN_RIGHT", r#""close""#),
            ("Super+BTN_MIDDLE", r#"{"switch":"next"}"#),
            ("Super+x", r#""cube""#),
        ] {
            let mut config: Config = parse(DEFAULT).unwrap();
            let action: KeyAction = serde_json::from_str(action).unwrap();
            config.common.bindings.insert(key.into(), action);
            let warnings = config.refuse();
            assert_eq!(warnings.len(), 1, "{key}: {warnings:?}");
            assert!(warnings[0].contains(key), "{key}: {warnings:?}");
            assert_eq!(config.common.bindings.get(key), None, "{key}");
            assert!(config.common.bindings.contains_key("Super+BTN_LEFT"));
        }
        let mut config: Config = parse(DEFAULT).unwrap();
        config
            .common
            .bindings
            .insert("Super+BTN_RIGHT".into(), KeyAction::Look(Look::Cube));
        assert_eq!(config.refuse(), Vec::<String>::new(), "cube on a button");
    }

    /// A change of the bindings moves a key: it is written as a map of the
    /// file, and reads back as the same bindings.
    #[test]
    fn set_moves_a_key_and_the_next_start_reads_it() {
        let dir = std::env::temp_dir().join(format!("chora-xp-bindings-{}", std::process::id()));
        let path = dir.join("chora-xp.yaml");
        let mut file = File::<Config>::load(path.clone(), DEFAULT).unwrap();
        file.set(
            json!({"bindings": {"Super+h": "keys", "Alt+F4": null, "Super+g": {"carry": "left"}}}),
        )
        .unwrap();
        file.save().unwrap();
        let again = File::<Config>::load(path, DEFAULT).unwrap();
        let bindings = &again.value.common.bindings;
        assert_eq!(bindings["Super+h"], KeyAction::System(System::Keys));
        assert_eq!(
            bindings["Super+g"],
            KeyAction::Look(Look::Carry(Turn::Left))
        );
        assert!(!bindings.contains_key("Alt+F4"));
        assert_eq!(bindings["Super+F1"], KeyAction::System(System::Keys));
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// A binding that is no key, or two spellings of one key, fail the
    /// settings instead of the grab.
    #[test]
    fn a_binding_that_is_no_key_fails() {
        let config: Config = parse(DEFAULT).unwrap();
        for (key, other) in [("Hyper+x", "close"), ("super+h", "close")] {
            let mut changed = config.clone();
            changed.common.bindings.insert(
                key.into(),
                serde_json::from_str(&format!("{other:?}")).unwrap(),
            );
            if key == "super+h" {
                changed
                    .common
                    .bindings
                    .insert("Super+h".into(), KeyAction::System(System::Keys));
            }
            assert!(changed.check().is_err(), "{key} passed");
        }
    }
}
