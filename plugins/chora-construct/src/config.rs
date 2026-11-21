//! The settings of the plugin (D62). The file `construct.yaml` next to
//! the crate is the default and the reference.

use chora_kit::actions::System;
use chora_kit::settings::{Common, Settings};
use chora_kit::volume::Change;
use chora_scene::Binding;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The default settings.
pub const DEFAULT: &str = include_str!("../construct.yaml");

/// What a key in the space does and no other look has: walking, looking
/// and the windows list. The rest, the actions that every look has, is
/// [`System`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Own {
    Forward,
    Back,
    Left,
    Right,
    TurnLeft,
    TurnRight,
    LookUp,
    LookDown,
    Home,
    FlyIn,
    /// Opens or closes the window list (F29).
    Windows,
    /// Opens the cube of the setting `cube_manager`.
    Cube,
}

impl Own {
    /// What the action does, in words.
    #[must_use]
    pub const fn words(self) -> &'static str {
        match self {
            Self::Forward => "Walk forward",
            Self::Back => "Walk back",
            Self::Left => "Step left",
            Self::Right => "Step right",
            Self::TurnLeft => "Turn left",
            Self::TurnRight => "Turn right",
            Self::LookUp => "Look up",
            Self::LookDown => "Look down",
            Self::Home => "Back to the start",
            Self::FlyIn => "Fly into the pane ahead",
            Self::Windows => "Show the windows",
            Self::Cube => "Open the cube",
        }
    }
}

/// The action of a key: one of every look, or one of the construct.
pub type Action = chora_kit::actions::Action<Own>;

impl From<Own> for Action {
    fn from(own: Own) -> Self {
        Self::Look(own)
    }
}

/// Whether the construct does the system action. A turn of the cube and a
/// step through the desks have no meaning here, a drag is a binding of the
/// flavor, and a run or a give is no feature of the construct (yet).
#[must_use]
pub const fn acts_in_construct(system: &System) -> bool {
    !matches!(
        system,
        System::Desk(_) | System::Cube(_) | System::Drag | System::Run(_) | System::Give(_)
    )
}

/// Light or dark.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Light,
    Dark,
}

/// The colours of a theme, RGB.
pub struct Palette {
    /// The sky overhead.
    pub sky: [f64; 3],
    /// The band at the horizon, into which far panes fade.
    pub horizon: [f64; 3],
    pub ground: [f64; 3],
    /// The light that glints in the glossy floor.
    pub sheen: [f64; 3],
    /// The marble of the floor: the strength of its veins, its micro
    /// bumps and its scratches, from 0 to 1, and the size of the pattern
    /// of the veins in logical pixels (fog.frag).
    pub marble: [f64; 4],
}

impl Theme {
    /// The strength of the mirror images in the glossy floor.
    #[must_use]
    pub fn gloss(self) -> f64 {
        match self {
            Self::Light => 0.4,
            Self::Dark => 0.55,
        }
    }

    #[must_use]
    pub fn palette(self) -> Palette {
        match self {
            Self::Light => Palette {
                sky: [0.915, 0.93, 0.95],
                horizon: [1.0, 1.0, 1.0],
                ground: [0.93, 0.93, 0.925],
                sheen: [1.0, 1.0, 1.0],
                marble: [0.8, 1.0, 1.0, 900.0],
            },
            Self::Dark => Palette {
                sky: [0.0, 0.0, 0.0],
                horizon: [0.15, 0.15, 0.16],
                ground: [0.012, 0.012, 0.014],
                sheen: [0.35, 0.35, 0.37],
                marble: [0.6, 1.0, 1.0, 900.0],
            },
        }
    }
}

/// The four colour roles of a theme and its glow (docs/construct-style.md,
/// rule 3).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Colors {
    /// The fill of a panel, RGBA.
    pub glass: [f64; 4],
    /// The hairline of a panel, RGBA.
    pub edge: [f64; 4],
    /// Content and the selected row, RGB.
    pub text: [f64; 3],
    /// Labels, notes and the rows that are not selected, RGB.
    pub muted: [f64; 3],
    /// The alpha of the accent in the glow around a panel.
    pub glow: f64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Themes {
    pub light: Colors,
    pub dark: Colors,
}

/// The elements over the space: the notices, the window list, the pane
/// labels and the help (docs/construct-style.md).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Style {
    /// The spacing unit in logical pixels.
    pub unit: f64,
    /// The corner radius of a panel.
    pub radius: f64,
    /// How far the glow reaches out of a panel.
    pub glow: f64,
    /// The family of labels and text.
    pub font: String,
    /// The family of data.
    pub mono: String,
    /// The one accent of both themes, RGB.
    pub accent: [f64; 3],
    pub themes: Themes,
}

impl Style {
    #[must_use]
    pub fn colors(&self, theme: Theme) -> &Colors {
        match theme {
            Theme::Light => &self.themes.light,
            Theme::Dark => &self.themes.dark,
        }
    }

    /// Rule 5 of the guide: on the glass over a black and over a white
    /// pane, `text` keeps a contrast of 7, `muted` of 4.5 and `accent` of
    /// 3 (WCAG).
    fn contrast(&self) -> Result<(), String> {
        let themes = [("light", &self.themes.light), ("dark", &self.themes.dark)];
        let cases = themes.into_iter().flat_map(|(name, c)| {
            [("black", 0.0), ("white", 1.0)].map(|(pane, under)| {
                let g = c.glass;
                let bg: [f64; 3] = std::array::from_fn(|i| g[i] * g[3] + under * (1.0 - g[3]));
                (name, c, pane, bg)
            })
        });
        for (name, c, pane, bg) in cases {
            let roles = [
                ("text", c.text, 7.0),
                ("muted", c.muted, 4.5),
                ("accent", self.accent, 3.0),
            ];
            if let Some((role, ratio, least)) = (roles.into_iter())
                .map(|(role, color, least)| (role, contrast(color, bg), least))
                .find(|(_, ratio, least)| ratio < least)
            {
                return Err(format!(
                    "style: {role} of {name} has a contrast of {ratio:.1} on the glass \
                     over a {pane} pane, below {least}"
                ));
            }
        }
        Ok(())
    }
}

/// The relative luminance of an sRGB colour (WCAG 2).
fn luminance(c: [f64; 3]) -> f64 {
    let lin = |v: f64| {
        if v <= 0.040_45 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.0722f64.mul_add(lin(c[2]), 0.2126f64.mul_add(lin(c[0]), 0.7152 * lin(c[1])))
}

/// The contrast ratio of two colours, from 1 to 21.
#[must_use]
pub fn contrast(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The construct of the flavor's start; empty for an empty space.
    pub start: String,
    pub theme: Theme,
    /// The leash radius R in logical pixels.
    pub radius: f64,
    /// The distance of a new pane in logical pixels.
    pub pane_distance: f64,
    /// Keysym name to action in `bindings` (the old name `keys` is read
    /// for one release); a key that starts or closes something names its
    /// modifier, as in `Super+t`. The test key too.
    #[serde(flatten)]
    pub common: Common<Own>,
    /// The shell command of the action `terminal`.
    pub terminal: String,
    /// The shell command of the action `launcher`.
    pub launcher: String,
    /// The grabbed binding that flies out of a pane.
    pub fly_out: String,
    /// The modifier of a drag that moves a pane and of the wheel that
    /// resizes it.
    pub modifier: String,
    /// Who draws the voice HUD (F28) and the questions: only the HUD
    /// host, with the folder `hud`. Kept to read old files.
    pub voice_hud: VoiceHud,
    /// The effects of the panes of own windows: `wobbly` bends a pane in
    /// 3D while the modifier drags it.
    #[serde(default)]
    pub effects: Vec<chora_kit::effects::Kind>,
    /// The command of the window manager of the cube; `cube.open` adds
    /// `--output NAME` for each face. It runs as the client of its
    /// program without the prefix `chora-`, such as `xp`.
    #[serde(default = "cube_manager")]
    pub cube_manager: String,
    /// The window managers of `pane.manager`: the client name of each,
    /// and its command, which gets `--output NAME`.
    #[serde(default = "managers")]
    pub managers: std::collections::BTreeMap<String, String>,
    pub style: Style,
}

/// Who draws the voice HUD and the questions of the voice agent: the HUD
/// host, the only presenter. The values `own`, `true`
/// and `false` of earlier versions read as `host` for one release (D93);
/// `own` and `true` warn that the own HUD is gone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VoiceHud {
    /// The HUD host draws them; the construct claims
    /// nothing.
    #[default]
    Host,
}

impl<'de> Deserialize<'de> for VoiceHud {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Written {
            Before(bool),
            Name(String),
        }
        let own = match Written::deserialize(deserializer)? {
            Written::Before(own) => own,
            Written::Name(name) if name == "own" => true,
            Written::Name(name) if name == "host" => false,
            Written::Name(name) => {
                return Err(serde::de::Error::custom(format!(
                    "voice_hud: {name:?} is not host"
                )));
            }
        };
        if own {
            eprintln!(
                "chora-construct: voice_hud: the own voice HUD is gone; \
                 the HUD host draws it (read as host)"
            );
        }
        Ok(Self::Host)
    }
}

fn cube_manager() -> String {
    "chora-xp".into()
}

fn managers() -> std::collections::BTreeMap<String, String> {
    [
        ("xp", "chora-xp"),
        ("tiling", "chora-tiling"),
        ("mc", "chora-mc --manage-windows"),
    ]
    .map(|(name, command)| (name.to_owned(), command.to_owned()))
    .into()
}

impl Settings for Config {
    fn renamed() -> &'static [(&'static str, &'static str)] {
        &[("keys", "bindings")]
    }

    fn refuse(&mut self) -> Vec<String> {
        let mut warnings = Vec::new();
        self.common.bindings.retain(|key, action| {
            let unfit = matches!(action, Action::System(s) if !acts_in_construct(s));
            if unfit {
                warnings.push(format!(
                    "chora-construct: the binding {key} is dropped: the construct has no action for {action:?}"
                ));
            }
            !unfit
        });
        warnings
    }

    fn check(&self) -> Result<(), String> {
        if !(self.radius.is_finite() && self.radius >= 100.0) {
            return Err(format!("radius: {} is below 100", self.radius));
        }
        if !(self.pane_distance.is_finite() && self.pane_distance > 0.0) {
            return Err(format!(
                "pane_distance: {} is not above 0",
                self.pane_distance
            ));
        }
        if !["Ctrl", "Alt", "Shift", "Super"].contains(&self.modifier.as_str()) {
            return Err(format!(
                "modifier: {} is not Ctrl, Alt, Shift or Super",
                self.modifier
            ));
        }
        if self.effects.contains(&chora_kit::effects::Kind::Lamp) {
            return Err("effects: the construct has no task button, so only wobbly".into());
        }
        self.check_style()?;
        self.check_managers()?;
        if !self.start.is_empty() {
            crate::file::check_name(&self.start).map_err(|e| format!("start: {e}"))?;
        }
        self.fly_out
            .parse::<Binding>()
            .map(drop)
            .map_err(|e| format!("fly_out: {}: {e}", self.fly_out))
    }
}

impl Config {
    /// The action of a key that the space got: the key with its Ctrl,
    /// Alt and Super in that order, as in `Super+t`. With Shift held, the
    /// name `Super+Shift+e` is tried first, then the key as the keymap
    /// shifts it, as in `Super+E`.
    #[must_use]
    pub fn action(&self, key: &str, modifiers: &[String]) -> Option<&Action> {
        let held = |name: &str| modifiers.iter().any(|m| m == name);
        let mut name: Vec<&str> = ["Ctrl", "Alt", "Super"]
            .into_iter()
            .filter(|m| held(m))
            .collect();
        let plain = [name.as_slice(), &[key]].concat().join("+");
        if held("Shift") {
            name.push("Shift");
            let shifted = if key.chars().count() == 1 {
                key.to_lowercase()
            } else {
                key.to_owned()
            };
            let with_shift = [name.as_slice(), &[shifted.as_str()]].concat().join("+");
            if let Some(action) = self.common.bindings.get(&with_shift) {
                return Some(action);
            }
        }
        self.common.bindings.get(&plain)
    }

    /// The warnings of a configuration that works but surprises: a key
    /// that starts or closes something without a modifier, which typed
    /// text presses.
    #[must_use]
    pub fn warnings(&self) -> Vec<String> {
        let opens = [System::Terminal, System::Launcher, System::Close];
        (self.common.bindings.iter())
            .filter(|(key, action)| {
                matches!(action, Action::System(s) if opens.contains(s)) && !key.contains('+')
            })
            .map(|(key, _)| {
                format!(
                    "the key {key} starts or closes something without a modifier: \
                     text typed into the space presses it; name Super, as in Super+{key}"
                )
            })
            .collect()
    }

    /// The groups of keys of the help: the keys of the construct that work
    /// now from its settings, then the keys that every look shares. The
    /// rows are those of `kit::keysheet` for the bindings; the construct
    /// names its own actions, groups them and adds the keys that no
    /// binding holds.
    #[must_use]
    #[allow(clippy::too_many_lines, reason = "one row per key")]
    pub fn help_groups(&self, cube: bool) -> Vec<crate::command::KeyGroup> {
        use crate::command::{KeyGroup, KeyRow, chord};
        use chora_kit::keysheet;
        let settings = self;
        let mut held: Vec<(String, &Action)> = (settings.common.bindings.iter())
            .map(|(key, action)| (key.clone(), action))
            .collect();
        held.sort_by_key(|(key, _)| (key.chars().count() != 1, key.clone()));
        let gives = ["tiling", "xp", "mc", "construct"]
            .map(|to| System::Give(crate::names::manager_name(to).clone()));
        let give_actions = gives.clone().map(Action::System);
        let give_keys: Vec<(String, &Action)> = (give_actions.iter().enumerate())
            .map(|(i, action)| (format!("Super+Alt+{}", i + 1), action))
            .collect();
        let all: Vec<(String, &Action)> = held.iter().cloned().chain(give_keys).collect();
        let sheet = keysheet::rows(
            all.iter().cloned(),
            chora_kit::words::Language::En,
            |_, action| match action {
                Action::Look(own) => Some(own.words().to_owned()),
                Action::System(_) => None,
            },
        );
        let bound = |action: Action| -> Vec<Vec<String>> {
            let Some(row) = sheet.iter().find(|r| r.action == action) else {
                return Vec::new();
            };
            (row.chords.iter())
                .filter_map(|c| all.iter().find(|(b, _)| keysheet::chord(b) == *c))
                .map(|(b, _)| chord(b))
                .collect()
        };
        let row = |what: &str, keys: Vec<String>, alt: Vec<String>| KeyRow {
            what: what.into(),
            keys,
            alt,
            gesture: None,
        };
        let say = |system: &System| {
            keysheet::words(system, chora_kit::words::Language::En).unwrap_or_default()
        };
        let one = |action: Action| bound(action).into_iter().next().unwrap_or_default();
        let letters = |actions: &[Action]| -> Vec<String> {
            actions.iter().flat_map(|a| one(a.clone())).collect()
        };
        let second = |action: Action| bound(action).into_iter().nth(1).unwrap_or_default();
        let keys_of_help = bound(Action::System(System::Keys));
        let shared_help = (keys_of_help
            .iter()
            .find(|k| k.first().is_some_and(|m| m == "Super")))
        .or_else(|| keys_of_help.first())
        .cloned()
        .unwrap_or_default();
        let walk = KeyGroup {
            head: "Walk".into(),
            shared: false,
            rows: vec![
                row(
                    "Walk forward, back, sideways",
                    letters(&[
                        Action::Look(Own::Forward),
                        Action::Look(Own::Left),
                        Action::Look(Own::Back),
                        Action::Look(Own::Right),
                    ]),
                    Vec::new(),
                ),
                row(
                    "Turn left or right",
                    letters(&[Action::Look(Own::TurnLeft), Action::Look(Own::TurnRight)]),
                    [
                        second(Action::Look(Own::TurnLeft)),
                        second(Action::Look(Own::TurnRight)),
                    ]
                    .concat(),
                ),
                row(
                    "Look up or down",
                    letters(&[Action::Look(Own::LookUp), Action::Look(Own::LookDown)]),
                    Vec::new(),
                ),
                row(Own::Home.words(), one(Action::Look(Own::Home)), Vec::new()),
                row(
                    Own::FlyIn.words(),
                    one(Action::Look(Own::FlyIn)),
                    Vec::new(),
                ),
                row(
                    Own::Windows.words(),
                    one(Action::Look(Own::Windows)),
                    Vec::new(),
                ),
                row(
                    Own::Cube.words(),
                    if cube {
                        Vec::new()
                    } else {
                        one(Action::Look(Own::Cube))
                    },
                    Vec::new(),
                ),
            ],
        };
        let system = |action: System| row(&say(&action), one(Action::System(action)), Vec::new());
        let panes = KeyGroup {
            head: "Panes".into(),
            shared: false,
            rows: vec![
                system(System::Terminal),
                system(System::Close),
                KeyRow {
                    what: "Move or resize a pane".into(),
                    keys: chord(&settings.modifier),
                    alt: Vec::new(),
                    gesture: Some("drag".into()),
                },
                row(
                    "Step back out of a pane",
                    chord(&settings.fly_out),
                    Vec::new(),
                ),
            ],
        };
        let mut groups = vec![walk, panes];
        if cube {
            groups.push(KeyGroup {
                head: "Cube".into(),
                shared: false,
                rows: vec![
                    row(
                        "Turn the cube",
                        chord("Ctrl+Alt+Left"),
                        chord("Ctrl+Alt+Right"),
                    ),
                    row("Step into the face", chord("Return"), Vec::new()),
                    row("Close the cube, stood back", chord("Escape"), Vec::new()),
                    row("Show these keys, stood back", chord("F1"), Vec::new()),
                ],
            });
        }
        groups.extend([
            KeyGroup {
                head: "Helper and chat".into(),
                shared: true,
                rows: vec![
                    system(System::Chat),
                    row("Answer a question", chord("Alt+y"), chord("Alt+n")),
                    row("Stop the helper", chord("Ctrl+period"), Vec::new()),
                ],
            },
            KeyGroup {
                head: "Give a window".into(),
                shared: true,
                rows: gives.into_iter().map(system).collect(),
            },
            KeyGroup {
                head: "Everywhere".into(),
                shared: true,
                rows: vec![
                    row("Back out one step", chord("Escape"), Vec::new()),
                    system(System::Launcher),
                    row(&say(&System::Keys), shared_help, Vec::new()),
                    system(System::Lock),
                    system(System::Power),
                    system(System::Screenshot),
                    system(System::Volume(Change::Up)),
                    system(System::Volume(Change::Down)),
                    system(System::Volume(Change::Mute)),
                ]
                .into_iter()
                .chain(sheet.iter().filter_map(|r| match r.action {
                    Action::System(System::Volume(Change::Set(_))) => {
                        Some(row(&r.words, chord(&r.binding), Vec::new()))
                    }
                    _ => None,
                }))
                .collect(),
            },
        ]);
        for group in &mut groups {
            group.rows.retain(|r| !r.keys.is_empty());
        }
        groups
    }

    /// Each window manager has a command.
    fn check_managers(&self) -> Result<(), String> {
        if self.cube_manager.trim().is_empty() {
            return Err("cube_manager: the command is empty".into());
        }
        match self.managers.iter().find(|(_, c)| c.trim().is_empty()) {
            Some((name, _)) => Err(format!("managers.{name}: the command is empty")),
            None => Ok(()),
        }
    }

    fn check_style(&self) -> Result<(), String> {
        let style = &self.style;
        let colors = [&style.themes.light, &style.themes.dark];
        let unit = |v: &f64| (0.0..=1.0).contains(v);
        let components = (colors.iter())
            .flat_map(|c| c.glass.iter().chain(&c.edge).chain(&c.text).chain(&c.muted))
            .chain(colors.iter().map(|c| &c.glow))
            .chain(&style.accent);
        if !components.into_iter().all(unit) {
            return Err("style: a colour component is outside 0 to 1".into());
        }
        if !(4.0..=16.0).contains(&style.unit) {
            return Err(format!("style.unit: {} is not from 4 to 16", style.unit));
        }
        for (name, v) in [("radius", style.radius), ("glow", style.glow)] {
            if !(0.0..=32.0).contains(&v) {
                return Err(format!("style.{name}: {v} is not from 0 to 32"));
            }
        }
        style.contrast()
    }
}

/// Logs the [`Config::warnings`] of a configuration, one line each.
pub fn log_warnings(config: &Config) {
    for warning in config.warnings() {
        eprintln!("chora-construct: {warning}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An action as the head 29e2cd77 spelled it.
    fn old_spelling(action: &Action) -> String {
        match action {
            Action::System(System::Keys) => "help".to_owned(),
            Action::System(System::Volume(Change::Up)) => "volume_up".to_owned(),
            Action::System(System::Volume(Change::Down)) => "volume_down".to_owned(),
            Action::System(System::Volume(Change::Mute)) => "mute".to_owned(),
            other => serde_json::to_string(other)
                .unwrap()
                .trim_matches('"')
                .to_owned(),
        }
    }

    /// The shipped config of head 29e2cd77 loads to the same table of keys
    /// as before kit took the shared actions: the table
    /// `tests/configs/29e2cd77/construct.table` was recorded on that
    /// head, one key and its action as it spelled it then.
    #[test]
    fn the_config_of_29e2cd77_loads_to_the_same_table() {
        use std::fmt::Write;
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/configs/29e2cd77");
        let file =
            chora_kit::settings::File::<Config>::load(dir.join("construct.yaml"), DEFAULT).unwrap();
        assert_eq!(file.broken(), None, "the old file loads");
        let config = file.value;
        let mut table = String::new();
        for (key, action) in &config.common.bindings {
            writeln!(table, "{key}\t\"{}\"", old_spelling(action)).unwrap();
        }
        let golden = dir.join("construct.table");
        if std::env::var_os("CHORA_BLESS").is_some() {
            std::fs::write(&golden, &table).unwrap();
        }
        assert_eq!(table, std::fs::read_to_string(&golden).unwrap());
    }

    /// One YAML binding gives one `System` action in every look: the file
    /// `tests/configs/system-bindings.yaml` loads to its `.table` here and
    /// in mc.
    #[test]
    fn one_binding_gives_the_same_system_action_in_every_look() {
        use std::fmt::Write;
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/configs");
        let text = std::fs::read_to_string(dir.join("system-bindings.yaml")).unwrap();
        let common: Common<Own> = serde_saphyr::from_str(&text).unwrap();
        let mut table = String::new();
        for (key, action) in &common.bindings {
            writeln!(table, "{key}\t{}", serde_json::to_string(action).unwrap()).unwrap();
        }
        let golden = std::fs::read_to_string(dir.join("system-bindings.table")).unwrap();
        assert_eq!(table, golden);
    }

    /// A binding that the construct has no action for is dropped with one
    /// warning that names the key, and the rest of the file loads: a step
    /// through the desks, a turn of the cube, a drag, a run and a give.
    #[test]
    fn a_system_action_without_meaning_in_the_construct_is_dropped_with_a_warning() {
        for unfit in [
            "{desk: next}",
            "{cube: left}",
            "drag",
            "{run: foot}",
            "{give: xp}",
        ] {
            let text = DEFAULT.replace("  Super+c: cube\n", &format!("  Super+c: {unfit}\n"));
            let mut config: Config = serde_saphyr::from_str(&text).unwrap();
            let warnings = config.refuse();
            assert_eq!(warnings.len(), 1, "{unfit}: {warnings:?}");
            assert!(warnings[0].contains("Super+c"), "{unfit}: {warnings:?}");
            assert_eq!(config.common.bindings.get("Super+c"), None, "{unfit}");
            assert!(
                config.common.bindings.contains_key("Super+x"),
                "the rest stays"
            );
            config.check().unwrap();
        }
    }

    /// The old name `keys` and the old spellings load, and a save writes
    /// the new ones.
    #[test]
    fn a_config_with_the_old_names_loads_and_saves_the_new_ones() {
        use chora_kit::settings::{File, Store};
        let dir = chora_kit::testing::TempDir::new("chora-construct-common");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("construct.yaml");
        let old = DEFAULT
            .replace("\nbindings:\n", "\nkeys:\n")
            .replace("Super+F1: keys", "Super+F1: help")
            .replace("XF86AudioMute: {volume: mute}", "XF86AudioMute: mute");
        std::fs::write(&path, old).unwrap();
        let mut file = File::<Config>::load(path.clone(), DEFAULT).unwrap();
        assert_eq!(
            file.value.common.bindings["XF86AudioMute"],
            Action::System(System::Volume(Change::Mute))
        );
        file.set(serde_json::json!({"bindings": {"Super+r": "lock"}}))
            .unwrap();
        file.save().unwrap();
        let saved = std::fs::read_to_string(&path).unwrap();
        assert!(
            saved.contains("bindings:") && !saved.contains("help"),
            "{saved}"
        );
        let again = File::<Config>::load(path, DEFAULT).unwrap();
        assert_eq!(
            again.value.common.bindings["Super+r"],
            Action::System(System::Lock)
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn the_default_file_is_valid() {
        let config: Config = serde_saphyr::from_str(DEFAULT).unwrap();
        config.check().unwrap();
        assert_eq!(config.common.bindings["Return"], Action::Look(Own::FlyIn));
        assert_eq!(
            config.common.bindings["Super+space"],
            Action::System(System::Launcher)
        );
        assert_eq!(config.common.bindings["Tab"], Action::Look(Own::Windows));
        assert_eq!(config.voice_hud, VoiceHud::Host, "one voice indicator");
        assert_eq!(config.style.font, "Inter");
    }

    #[test]
    fn the_terminal_of_the_default_file_pads_its_text_12_by_12() {
        let config: Config = serde_saphyr::from_str(DEFAULT).unwrap();
        let words: Vec<&str> = config.terminal.split_whitespace().collect();
        assert_eq!(words.first(), Some(&"foot"));
        assert!(words.contains(&"pad=12x12"), "{:?}", config.terminal);
    }

    /// A key that starts or closes something takes the modifier, so
    /// that text typed into the space starts nothing; the walk keys
    /// take none, and Shift is part of the key.
    #[test]
    fn keys_that_start_or_close_need_super() {
        let config: Config = serde_saphyr::from_str(DEFAULT).unwrap();
        let super_ = ["Super".to_owned()];
        for key in ["t", "space", "x"] {
            assert_eq!(config.action(key, &[]), None, "{key} alone");
        }
        assert_eq!(
            config.action("t", &super_),
            Some(&Action::System(System::Terminal))
        );
        assert_eq!(
            config.action("space", &super_),
            Some(&Action::System(System::Launcher))
        );
        assert_eq!(
            config.action("x", &super_),
            Some(&Action::System(System::Close))
        );
        assert_eq!(config.action("w", &[]), Some(&Action::Look(Own::Forward)));
        assert_eq!(config.action("w", &super_), None);
        let shift = ["Shift".to_owned()];
        assert_eq!(
            config.action("Return", &shift),
            Some(&Action::Look(Own::FlyIn))
        );
    }

    /// A start or close key without a modifier still works, but the load
    /// warns: typed text would start or close something.
    #[test]
    fn a_start_or_close_key_without_a_modifier_warns() {
        let config: Config = serde_saphyr::from_str(DEFAULT).unwrap();
        assert_eq!(config.warnings(), Vec::<String>::new());
        let plain = DEFAULT.replace("Super+t: terminal", "t: terminal");
        let config: Config = serde_saphyr::from_str(&plain).unwrap();
        assert_eq!(
            config.warnings(),
            ["the key t starts or closes something without a modifier: \
              text typed into the space presses it; name Super, as in Super+t"]
        );
    }

    /// The defaults reach the ratios of rule 5, and a muted that is too
    /// pale, or a colour out of range, fails.
    #[test]
    fn the_style_keeps_its_contrast() {
        let config: Config = serde_saphyr::from_str(DEFAULT).unwrap();
        config.check().unwrap();
        let light = &config.style.themes.light;
        let over_white = |c| {
            let g = light.glass;
            contrast(c, std::array::from_fn(|i| g[i] * g[3] + (1.0 - g[3])))
        };
        assert!(over_white(light.text) > 12.0, "{}", over_white(light.text));
        // Over a black pane the light glass stays a clean white, not grey.
        let over_black: [f64; 3] = std::array::from_fn(|i| light.glass[i] * light.glass[3]);
        assert!(over_black.iter().all(|c| *c > 0.9), "{over_black:?}");
        assert!(contrast(light.text, over_black) > 15.0);
        let mut pale = config.clone();
        pale.style.themes.light.muted = [0.6, 0.6, 0.6];
        assert!(pale.check().unwrap_err().contains("muted of light"));
        let mut dim = config.clone();
        dim.style.themes.dark.text = [0.5, 0.5, 0.5];
        assert!(dim.check().unwrap_err().contains("text of dark"));
        let mut out = config;
        out.style.accent = [1.2, 0.5, 0.5];
        assert!(out.check().is_err());
    }

    /// `own`, `true` and `false`, the values of earlier versions, load
    /// as `host` for one release (D93); another value fails.
    #[test]
    fn old_values_of_voice_hud_load_as_host() {
        let with = |value: &str| {
            let text = DEFAULT.replace("voice_hud: host", &format!("voice_hud: {value}"));
            serde_saphyr::from_str::<Config>(&text)
                .ok()
                .map(|config| config.voice_hud)
        };
        for value in ["host", "own", "true", "false"] {
            assert_eq!(with(value), Some(VoiceHud::Host), "{value}");
        }
        assert_eq!(with("maybe"), None);
        let written = serde_json::to_value(VoiceHud::Host).unwrap();
        assert_eq!(written, serde_json::json!("host"));
    }

    /// Rule 5 holds for the block `chat:` of the folder `hud`: `text` 7:1
    /// and the names 4.5:1 on the fill; the chat is the glass panel at
    /// the right, 480 wide, in the colours of the light theme.
    #[test]
    fn the_chat_of_the_folder_keeps_the_contrast_of_rule_5() {
        use chora_kit::hud::{self, Anchor, ChatStyle};
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("hud");
        let folder = hud::load(&dir).unwrap_or_else(|errors| panic!("{errors:?}"));
        let chat = folder.chat;
        assert_eq!(chat.style, ChatStyle::Glass);
        assert_eq!(chat.place(), (Anchor::Right, 480));
        let rgb = |c: Option<[u8; 3]>| c.expect("a colour").map(|v| f64::from(v) / 255.0);
        let fill: [f64; 3] = rgb(chat.fill).map(|side| side * hud::GLASS_ALPHA);
        assert!(contrast(rgb(chat.text), fill) >= 7.0);
        assert!(contrast(rgb(chat.name), fill) >= 4.5);
        let light = chora_kit::glass::Material::of(false);
        let near = |a: [f64; 3], b: [f64; 4]| (0..3).all(|i| (a[i] - b[i]).abs() < 1.0 / 255.0);
        assert!(near(rgb(chat.text), light.text));
        assert!(near(rgb(chat.name), light.muted));
    }

    /// The glass of the chat of the folder `hud` is the glass of the
    /// light theme: its fill and opacity, its edge and the accent as the
    /// glow (rule 6).
    #[test]
    fn the_chat_of_the_folder_is_the_glass_of_the_light_theme() {
        use chora_kit::{glass::Material, hud};
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("hud");
        let chat = hud::load(&dir).unwrap().chat;
        let config: Config = serde_saphyr::from_str(DEFAULT).unwrap();
        let light = Material::of(false);
        let rgb = |c: Option<[u8; 3]>| c.expect("a colour").map(|v| f64::from(v) / 255.0);
        let near = |a: [f64; 3], b: &[f64]| (0..3).all(|i| (a[i] - b[i]).abs() < 1.0 / 255.0);
        assert!(near(rgb(chat.fill), &light.tint));
        assert!(near(rgb(chat.text), &light.text));
        assert!(near(rgb(chat.name), &light.muted));
        assert!(near(rgb(chat.glow), &config.style.accent));
    }

    /// A window manager needs a command: an empty one fails the check.
    #[test]
    fn a_manager_without_a_command_fails() {
        let config: Config = serde_saphyr::from_str(DEFAULT).unwrap();
        let mut empty = config.clone();
        empty.managers.insert("tiling".into(), "  ".into());
        assert!(empty.check().unwrap_err().contains("managers.tiling"));
        let mut cube = config;
        cube.cube_manager = String::new();
        assert!(cube.check().unwrap_err().contains("cube_manager"));
    }

    /// Super+C opens the cube: a key, not a button.
    #[test]
    fn super_c_opens_the_cube() {
        let config: Config = serde_saphyr::from_str(DEFAULT).unwrap();
        assert_eq!(
            config.common.bindings.get("Super+c"),
            Some(&Action::Look(Own::Cube))
        );
        let opens = (config.help_groups(false).into_iter())
            .flat_map(|g| g.rows)
            .find(|r| r.what == "Open the cube")
            .map(|r| r.keys);
        assert_eq!(opens.unwrap(), ["Super", "C"]);
    }

    /// The help lists the keys of the cube only while a cube is open, and
    /// the bindings of the settings, also changed ones.
    #[test]
    fn the_help_follows_the_settings_and_the_cube() {
        let mut config: Config = serde_saphyr::from_str(DEFAULT).unwrap();
        let heads = |cube| -> Vec<String> {
            config
                .help_groups(cube)
                .into_iter()
                .map(|g| g.head)
                .collect()
        };
        assert!(!heads(false).contains(&"Cube".to_owned()));
        assert!(heads(true).contains(&"Cube".to_owned()));
        let keys = |config: &Config, what: &str| {
            (config.help_groups(false).into_iter())
                .flat_map(|g| g.rows)
                .find(|r| r.what == what)
                .map(|r| r.keys)
        };
        assert_eq!(
            keys(&config, "Walk forward, back, sideways").unwrap(),
            ["W", "A", "S", "D"]
        );
        assert_eq!(keys(&config, "All keys").unwrap(), ["Super", "F1"]);
        assert_eq!(keys(&config, "Move or resize a pane").unwrap(), ["Super"]);
        config.modifier = "Alt".into();
        config.common.bindings.remove("h");
        assert_eq!(keys(&config, "Move or resize a pane").unwrap(), ["Alt"]);
        assert_eq!(keys(&config, "Back to the start"), None);
    }

    /// `F1` and `Super+F1` show the help, the row stands in the group
    /// Everywhere under `Super+F1`, `Super+/` is the silent alias, and no
    /// row stands twice.
    #[test]
    fn f1_and_super_f1_show_the_help() {
        let config: Config = serde_saphyr::from_str(DEFAULT).unwrap();
        assert_eq!(
            config.common.bindings.get("Super+F1"),
            Some(&Action::System(System::Keys))
        );
        assert_eq!(
            config.common.bindings.get("Super+slash"),
            Some(&Action::System(System::Keys))
        );
        assert_eq!(
            config.common.bindings.get("F1"),
            Some(&Action::System(System::Keys))
        );
        let groups = config.help_groups(false);
        let keys_of = |head: &str| -> Vec<Vec<String>> {
            (groups.iter().filter(|g| g.head == head))
                .flat_map(|g| g.rows.iter().map(|r| r.keys.clone()))
                .collect()
        };
        assert!(keys_of("Everywhere").contains(&vec!["Super".to_owned(), "F1".to_owned()]));
        assert!(!keys_of("Walk").iter().any(|k| k.contains(&"F1".to_owned())));
        let mut whats: Vec<&str> = (groups.iter())
            .flat_map(|g| g.rows.iter().map(|r| r.what.as_str()))
            .collect();
        let rows = whats.len();
        whats.sort_unstable();
        whats.dedup();
        assert_eq!(whats.len(), rows, "a row stands twice");
    }

    /// The keys of every look are bound, Shift names itself where the key
    /// has no shifted form, and every one is in a shared group of the help.
    #[test]
    fn the_keys_of_every_look_are_bound_and_named_in_the_help() {
        let config: Config = serde_saphyr::from_str(DEFAULT).unwrap();
        let held =
            |names: &[&str]| -> Vec<String> { names.iter().map(|n| (*n).to_owned()).collect() };
        let super_shift = held(&["Shift", "Super"]);
        assert_eq!(
            config.action("Escape", &super_shift),
            Some(&Action::System(System::Lock))
        );
        assert_eq!(
            config.action("E", &super_shift),
            Some(&Action::System(System::Power))
        );
        assert_eq!(config.action("Escape", &held(&["Super"])), None);
        assert_eq!(
            config.action("a", &held(&["Super"])),
            Some(&Action::System(System::Chat))
        );
        assert_eq!(config.action("a", &[]), Some(&Action::Look(Own::Left)));
        assert_eq!(
            config.action("Print", &held(&["Super"])),
            Some(&Action::System(System::Screenshot))
        );
        assert_eq!(
            config.action("XF86AudioMute", &[]),
            Some(&Action::System(System::Volume(Change::Mute)))
        );
        let groups = config.help_groups(false);
        let shared: Vec<String> = (groups.iter().filter(|g| g.shared))
            .flat_map(|g| g.rows.iter().map(|r| r.keys.join("+")))
            .collect();
        for chord in [
            "Super+F1",
            "Super+Space",
            "Super+A",
            "Super+Shift+Esc",
            "Super+Shift+E",
            "Super+Print",
            "Vol+",
            "Vol-",
            "Mute",
        ] {
            assert!(shared.contains(&chord.to_owned()), "{chord} in {shared:?}");
        }
    }

    /// KIT3: the rows of the help are the rows of `kit::keysheet` for the
    /// bindings: any action of the system that the construct acts on has
    /// a row with the words of kit, also one the shipped file does not
    /// bind.
    #[test]
    fn every_action_of_the_system_has_a_row_with_the_words_of_the_keysheet() {
        let mut config: Config = serde_saphyr::from_str(DEFAULT).unwrap();
        config.common.bindings.insert(
            "Super+v".into(),
            Action::System(System::Volume(Change::Set(30))),
        );
        let rows: Vec<_> = (config.help_groups(false).into_iter())
            .flat_map(|g| g.rows)
            .collect();
        for (binding, action) in &config.common.bindings {
            let Action::System(system) = action else {
                continue;
            };
            let words =
                chora_kit::keysheet::words(system, chora_kit::words::Language::En).expect("words");
            let keys: Vec<Vec<String>> = (config.common.bindings.iter())
                .filter(|(_, a)| *a == action)
                .map(|(b, _)| chora_kit::keysheet::caps(b))
                .collect();
            assert!(
                rows.iter()
                    .any(|r| r.what == words && keys.contains(&r.keys)),
                "{binding}: no row says {words:?} with one of its keys"
            );
        }
    }

    /// Polish 14: the help names the window managers as people read them.
    #[test]
    fn the_help_gives_a_window_to_managers_by_their_display_names() {
        let config: Config = serde_saphyr::from_str(DEFAULT).unwrap();
        let gives: Vec<String> = (config.help_groups(false).into_iter())
            .filter(|g| g.head == "Give a window")
            .flat_map(|g| g.rows.into_iter().map(|r| r.what))
            .collect();
        assert_eq!(
            gives,
            [
                "Give window to Tiling",
                "Give window to XP",
                "Give window to Media center",
                "Give window to Construct"
            ]
        );
    }
}
