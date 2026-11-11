//! The settings of the plugin (D62). The file `construct.yaml` next to
//! the crate is the default and the reference.

use std::collections::BTreeMap;

use chora_kit::settings::Settings;
use chora_scene::Binding;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The default settings.
pub const DEFAULT: &str = include_str!("../construct.yaml");

/// What a key in the space does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Action {
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
    /// Starts the setting `terminal`.
    Terminal,
    /// Starts the setting `launcher`.
    Launcher,
    /// Asks the window of the pane in the middle of the view to close.
    Close,
    /// Opens or closes the window list (F29).
    Windows,
    /// Shows or hides the help panel with the bindings.
    Help,
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
            },
            Self::Dark => Palette {
                sky: [0.0, 0.0, 0.0],
                horizon: [0.15, 0.15, 0.16],
                ground: [0.012, 0.012, 0.014],
                sheen: [0.35, 0.35, 0.37],
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

/// The elements over the space: the voice HUD, the window list, the pane
/// labels and the questions (docs/construct-style.md).
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
    /// For tests only: take the keys of `input.inject` as keys of the
    /// user. Off, the bindings ignore injected keys.
    #[serde(default)]
    pub test_accept_injected_keys: bool,
    pub theme: Theme,
    /// The leash radius R in logical pixels.
    pub radius: f64,
    /// The distance of a new pane in logical pixels.
    pub pane_distance: f64,
    /// Keysym name to action; a key that starts or closes something
    /// names its modifier, as in `Super+t`.
    pub keys: BTreeMap<String, Action>,
    /// The shell command of the action `terminal`.
    pub terminal: String,
    /// The shell command of the action `launcher`.
    pub launcher: String,
    /// The grabbed binding that flies out of a pane.
    pub fly_out: String,
    /// The modifier of a drag that moves a pane and of the wheel that
    /// resizes it.
    pub modifier: String,
    /// Who draws the voice HUD (F28) and the questions: the construct,
    /// or the HUD host with the folder `hud`.
    pub voice_hud: VoiceHud,
    /// The effects of the panes of own windows: `wobbly` bends a pane in
    /// 3D while the modifier drags it.
    #[serde(default)]
    pub effects: Vec<chora_kit::effects::Kind>,
    /// The command of the window manager of the cube; `cube.open` adds
    /// `--output NAME` for each face. It runs as the client `xp`.
    #[serde(default = "cube_manager")]
    pub cube_manager: String,
    /// The window managers of `pane.manager`: the client name of each,
    /// and its command, which gets `--output NAME`.
    #[serde(default = "managers")]
    pub managers: std::collections::BTreeMap<String, String>,
    pub style: Style,
}

/// Who draws the voice HUD and the questions of the voice agent. The
/// values `true` and `false` of earlier versions read as `own` and
/// `host` for one release.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VoiceHud {
    /// The construct draws them and claims the indicator of the agent.
    Own,
    /// The HUD host `chora-assistant` draws them; the construct claims
    /// nothing.
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
        match Written::deserialize(deserializer)? {
            Written::Before(true) => Ok(Self::Own),
            Written::Before(false) => Ok(Self::Host),
            Written::Name(name) if name == "own" => Ok(Self::Own),
            Written::Name(name) if name == "host" => Ok(Self::Host),
            Written::Name(name) => Err(serde::de::Error::custom(format!(
                "voice_hud: {name:?} is not own or host"
            ))),
        }
    }
}

impl Config {
    /// Whether the construct draws the voice HUD and the questions and
    /// claims the indicator of the voice agent.
    #[must_use]
    pub fn draws_voice(&self) -> bool {
        self.voice_hud == VoiceHud::Own
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
    /// Alt and Super in that order, as in `Super+t`. Shift is part of
    /// the key, as in `T`.
    #[must_use]
    pub fn action(&self, key: &str, modifiers: &[String]) -> Option<Action> {
        let mut name: Vec<&str> = ["Ctrl", "Alt", "Super"]
            .into_iter()
            .filter(|m| modifiers.iter().any(|held| held == m))
            .collect();
        name.push(key);
        self.keys.get(&name.join("+")).copied()
    }

    /// The warnings of a configuration that works but surprises: a key
    /// that starts or closes something without a modifier, which typed
    /// text presses.
    #[must_use]
    pub fn warnings(&self) -> Vec<String> {
        let opens = [Action::Terminal, Action::Launcher, Action::Close];
        (self.keys.iter())
            .filter(|(key, action)| opens.contains(action) && !key.contains('+'))
            .map(|(key, _)| {
                format!(
                    "the key {key} starts or closes something without a modifier: \
                     text typed into the space presses it; name Super, as in Super+{key}"
                )
            })
            .collect()
    }

    /// The groups of keys of the help: the keys of the construct that work
    /// now from its settings, then the keys that every look shares.
    #[must_use]
    #[allow(clippy::too_many_lines, reason = "one row per key")]
    pub fn help_groups(&self, cube: bool) -> Vec<crate::command::KeyGroup> {
        use crate::command::{KeyGroup, KeyRow, chord};
        let settings = self;
        let bound = |action: Action| -> Vec<Vec<String>> {
            let mut keys: Vec<&String> = (settings.keys.iter())
                .filter(|(_, a)| **a == action)
                .map(|(k, _)| k)
                .collect();
            keys.sort_by_key(|k| (k.chars().count() != 1, (*k).clone()));
            keys.into_iter().map(|k| chord(k)).collect()
        };
        let row = |what: &str, keys: Vec<String>, alt: Vec<String>| KeyRow {
            what: what.into(),
            keys,
            alt,
            gesture: None,
        };
        let one = |action: Action| bound(action).into_iter().next().unwrap_or_default();
        let letters =
            |actions: &[Action]| -> Vec<String> { actions.iter().flat_map(|a| one(*a)).collect() };
        let second = |action: Action| bound(action).into_iter().nth(1).unwrap_or_default();
        let walk = KeyGroup {
            head: "Walk".into(),
            shared: false,
            rows: vec![
                row(
                    "Walk forward, back, sideways",
                    letters(&[Action::Forward, Action::Left, Action::Back, Action::Right]),
                    Vec::new(),
                ),
                row(
                    "Turn left or right",
                    letters(&[Action::TurnLeft, Action::TurnRight]),
                    [second(Action::TurnLeft), second(Action::TurnRight)].concat(),
                ),
                row(
                    "Look up or down",
                    letters(&[Action::LookUp, Action::LookDown]),
                    Vec::new(),
                ),
                row("Back to the start", one(Action::Home), Vec::new()),
                row("Fly into the pane ahead", one(Action::FlyIn), Vec::new()),
                row("Show the windows", one(Action::Windows), Vec::new()),
            ],
        };
        let panes = KeyGroup {
            head: "Panes".into(),
            shared: false,
            rows: vec![
                row("Open a terminal", one(Action::Terminal), Vec::new()),
                row("Start a program", one(Action::Launcher), Vec::new()),
                row("Close the pane ahead", one(Action::Close), Vec::new()),
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
        for group in &mut groups {
            group.rows.retain(|r| !r.keys.is_empty());
        }
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
                ],
            });
        }
        groups.extend([
            KeyGroup {
                head: "Helper and chat".into(),
                shared: true,
                rows: vec![
                    row("Answer yes to a question", chord("Alt+y"), Vec::new()),
                    row("Answer no to a question", chord("Alt+n"), Vec::new()),
                    row("Stop the helper", chord("Ctrl+period"), Vec::new()),
                ],
            },
            KeyGroup {
                head: "Give a window".into(),
                shared: true,
                rows: [
                    ("tiling", "1"),
                    ("xp", "2"),
                    ("mc", "3"),
                    ("the construct", "4"),
                ]
                .into_iter()
                .map(|(to, n)| {
                    row(
                        &format!("Give it to {to}"),
                        chord(&format!("Super+Alt+{n}")),
                        Vec::new(),
                    )
                })
                .collect(),
            },
            KeyGroup {
                head: "Everywhere".into(),
                shared: true,
                rows: vec![
                    row("Back out one step", chord("Escape"), Vec::new()),
                    row("Show these keys", one(Action::Help), Vec::new()),
                ],
            },
        ]);
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

    #[test]
    fn the_default_file_is_valid() {
        let config: Config = serde_saphyr::from_str(DEFAULT).unwrap();
        config.check().unwrap();
        assert_eq!(config.keys["Return"], Action::FlyIn);
        assert_eq!(config.keys["Super+space"], Action::Launcher);
        assert_eq!(config.keys["Tab"], Action::Windows);
        assert_eq!(config.voice_hud, VoiceHud::Host, "one voice indicator");
        assert_eq!(config.style.font, "Inter");
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
        assert_eq!(config.action("t", &super_), Some(Action::Terminal));
        assert_eq!(config.action("space", &super_), Some(Action::Launcher));
        assert_eq!(config.action("x", &super_), Some(Action::Close));
        assert_eq!(config.action("w", &[]), Some(Action::Forward));
        assert_eq!(config.action("w", &super_), None);
        let shift = ["Shift".to_owned()];
        assert_eq!(config.action("Return", &shift), Some(Action::FlyIn));
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

    /// With `voice_hud: host` the host draws the voice HUD and the
    /// question: the construct draws neither.
    #[test]
    fn voice_hud_host_draws_no_voice_status() {
        let mut config: Config = serde_saphyr::from_str(DEFAULT).unwrap();
        assert_eq!(config.voice_hud, VoiceHud::Host);
        assert!(!config.draws_voice());
        config.voice_hud = VoiceHud::Own;
        assert!(config.draws_voice());
        let text = DEFAULT.replace("voice_hud: host", "voice_hud: own");
        let own: Config = serde_saphyr::from_str(&text).unwrap();
        assert_eq!(own.voice_hud, VoiceHud::Own);
    }

    /// `true` and `false`, the values of the setting before `own` and
    /// `host`, stay aliases for one release.
    #[test]
    fn true_and_false_are_aliases_of_own_and_host() {
        let with = |value: &str| {
            let text = DEFAULT.replace("voice_hud: host", &format!("voice_hud: {value}"));
            serde_saphyr::from_str::<Config>(&text)
                .ok()
                .map(|config| config.voice_hud)
        };
        assert_eq!(with("true"), Some(VoiceHud::Own));
        assert_eq!(with("false"), Some(VoiceHud::Host));
        assert_eq!(with("own"), Some(VoiceHud::Own));
        assert_eq!(with("maybe"), None);
        let written = serde_json::to_value(VoiceHud::Own).unwrap();
        assert_eq!(written, serde_json::json!("own"));
    }

    /// With `voice_hud: host` the construct never claims the indicator of
    /// voice, also not when the claim is due.
    #[test]
    fn voice_hud_host_claims_no_indicator() {
        let mut config: Config = serde_saphyr::from_str(DEFAULT).unwrap();
        config.voice_hud = VoiceHud::Host;
        assert!(!config.draws_voice());
        config.voice_hud = VoiceHud::Own;
        assert!(config.draws_voice());
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
        assert_eq!(keys(&config, "Show these keys").unwrap(), ["F1"]);
        assert_eq!(keys(&config, "Move or resize a pane").unwrap(), ["Super"]);
        config.modifier = "Alt".into();
        config.keys.remove("h");
        assert_eq!(keys(&config, "Move or resize a pane").unwrap(), ["Alt"]);
        assert_eq!(keys(&config, "Back to the start"), None);
    }
}
