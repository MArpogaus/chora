//! The configuration: the settings of the plugin (D62), a YAML file that
//! lists every option. The file `tiling.yaml` next to the crate is the
//! default and the reference.

use std::{collections::BTreeMap, path::Path};

use chora_kit::settings::Settings;
use chora_scene::{Binding, ShaderEffect};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    layout::Layout,
    theme::{self, Colors, Scheme, Theme},
};

/// The default configuration.
pub const DEFAULT: &str = include_str!("../tiling.yaml");

/// The most tags per output: one bit each in a `u16`.
pub const MAX_TAGS: usize = 16;

/// The range of the master share.
pub const SHARE: std::ops::RangeInclusive<f64> = 0.05..=0.95;

/// The modifiers that `MOD` can be. Ctrl and Shift are parts of other
/// bindings.
const MODIFIERS: [&str; 2] = ["Super", "Alt"];

/// Where a new window goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NewWindow {
    /// First in the list: the new master.
    Master,
    /// Last in the list.
    Stack,
}

/// How the focus follows the pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Focus {
    /// A click on a window focuses it.
    Click,
    /// The window under the pointer has the focus, except in the
    /// magnifier layout.
    FollowMouse,
}

/// What a binding does.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    /// Runs a shell command as an app.
    Run(String),
    FocusNext,
    FocusPrevious,
    /// Focuses the window that had the focus before.
    FocusBack,
    /// Shows and focuses a window that asked for attention.
    FocusUrgent,
    SwapNext,
    SwapPrevious,
    /// Swaps the focused window with the master.
    MoveToMaster,
    Close,
    ToggleFloating,
    ToggleFullscreen,
    ToggleMaximized,
    ToggleMaximizedHorizontal,
    ToggleMaximizedVertical,
    /// Keeps a window above the others that do not have this state.
    ToggleOnTop,
    Minimize,
    /// Shows the last minimized window of the current tags.
    RestoreMinimized,
    /// Adds to the master share of the current tag.
    IncMasterWidth(f64),
    /// Adds to the number of master windows of the current tag.
    IncMasters(isize),
    /// Adds to the number of columns of the current tag.
    IncColumns(isize),
    NextLayout,
    PreviousLayout,
    FocusNextOutput,
    FocusPreviousOutput,
    MoveToNextOutput,
    /// Shows only this tag, by its number.
    ViewTag(usize),
    /// Shows or hides this tag next to the others.
    ToggleTagView(usize),
    /// Puts the focused window on this tag only.
    MoveToTag(usize),
    /// Adds or removes this tag of the focused window.
    ToggleWindowTag(usize),
    ViewNext,
    ViewPrevious,
    /// Shows the tags that were shown before.
    ViewBack,
    /// Reads the configuration file again.
    Reload,
    /// Gives the focused window to the window manager of this name (D55),
    /// whoever owns it.
    Give(String),
    /// On a pointer button: drags the window under the pointer. Dropped
    /// in the claim of another window manager, it goes there (D55).
    Drag,
    /// Changes the volume of the default sound output and shows it for
    /// a moment: `up`, `down`, `mute` or `{set: PERCENT}`.
    Volume(chora_kit::volume::Change),
    /// Shows the list of all keys, or hides it.
    ShowKeys,
    /// Does nothing and keeps the key from the program: the action of
    /// every `MOD` chord that no binding uses. It is not a setting.
    #[serde(skip)]
    Swallow,
}

/// The keys that `MOD` chords swallow when no binding uses them: letters,
/// digits, punctuation, the function keys and the keys of navigation.
const SWALLOWED: &[&str] = &[
    "a",
    "b",
    "c",
    "d",
    "e",
    "f",
    "g",
    "h",
    "i",
    "j",
    "k",
    "l",
    "m",
    "n",
    "o",
    "p",
    "q",
    "r",
    "s",
    "t",
    "u",
    "v",
    "w",
    "x",
    "y",
    "z",
    "0",
    "1",
    "2",
    "3",
    "4",
    "5",
    "6",
    "7",
    "8",
    "9",
    "F1",
    "F2",
    "F3",
    "F4",
    "F5",
    "F6",
    "F7",
    "F8",
    "F9",
    "F10",
    "F11",
    "F12",
    "Return",
    "Tab",
    "space",
    "Escape",
    "BackSpace",
    "Delete",
    "Insert",
    "Home",
    "End",
    "Prior",
    "Next",
    "Left",
    "Right",
    "Up",
    "Down",
    "minus",
    "equal",
    "bracketleft",
    "bracketright",
    "semicolon",
    "apostrophe",
    "grave",
    "backslash",
    "comma",
    "period",
    "slash",
    "Print",
];

/// The extra modifiers of the swallowed chords: none, Shift and Ctrl.
const SWALLOW_WITH: [&str; 3] = ["", "Shift+", "Ctrl+"];

/// How a new window looks (#99).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Style {
    /// From 0 to 1.
    pub opacity: f64,
    /// Shaders on the window, as in `scene.apply`.
    pub effects: Vec<ShaderEffect>,
}

/// The look of the windows: the values of the design, in logical pixels.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields, default)]
pub struct Look {
    /// The gap between the tiles and the edge of the work area. `null`
    /// takes `gap`.
    pub gap_outer: Option<u16>,
    /// The gap between two tiles. `null` takes twice `gap`.
    pub gap_inner: Option<u16>,
    /// The radius of the corners of a window; 0 for square ones.
    pub radius: u16,
    /// The width of the border around a window, outside it; 0 for none.
    pub border: u16,
    /// Which palette the look takes: `system` follows the colour scheme
    /// of the system and takes Ink when it says nothing; `ink` and `paper`
    /// are fixed.
    pub theme: Theme,
    /// Colours that replace those of the theme.
    pub palette: Palette,
}

/// The colours of the look that a ricer changes, each `#rrggbb`; a colour
/// that is `null` or missing comes from the theme.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields, default)]
pub struct Palette {
    /// The wallpaper colour and the lock screen.
    pub base: Option<String>,
    /// The bar islands and the overlays.
    pub surface: Option<String>,
    /// Hover, keycaps and wells.
    pub surface_2: Option<String>,
    /// The border of a window that has no focus, and hairlines.
    pub line: Option<String>,
    /// Text.
    pub text: Option<String>,
    /// Secondary text.
    pub muted: Option<String>,
    /// The focus: the border at the top left of its gradient, the current
    /// tag and the match in the launcher.
    pub accent: Option<String>,
    /// Text on the accent.
    pub on_accent: Option<String>,
    /// The bottom right end of the gradient of the focus border; the same
    /// as `accent` for a solid border.
    pub accent_2: Option<String>,
    /// The border of a window that asked for attention, and urgent tags.
    pub urgent: Option<String>,
}

impl Palette {
    fn entries(&self) -> [(&'static str, &Option<String>); 10] {
        [
            ("base", &self.base),
            ("surface", &self.surface),
            ("surface_2", &self.surface_2),
            ("line", &self.line),
            ("text", &self.text),
            ("muted", &self.muted),
            ("accent", &self.accent),
            ("on_accent", &self.on_accent),
            ("accent_2", &self.accent_2),
            ("urgent", &self.urgent),
        ]
    }

    /// The colours of a scheme with the colours of the file over them.
    #[must_use]
    pub fn over(&self, scheme: Scheme) -> Colors {
        let mut colors = Colors::of(scheme);
        for (name, value) in self.entries() {
            if let Some(value) = value {
                colors.set(name, value);
            }
        }
        colors
    }

    fn check(&self) -> Result<(), String> {
        for (name, value) in self.entries() {
            if let Some(value) = value
                && theme::rgb(value).is_none()
            {
                return Err(format!("look.palette.{name}: #rrggbb"));
            }
        }
        Ok(())
    }
}

impl Look {
    /// Whether the values are within their limits.
    fn check(&self) -> Result<(), String> {
        for (name, gap) in [
            ("look.gap_outer", self.gap_outer),
            ("look.gap_inner", self.gap_inner),
        ] {
            if gap.is_some_and(|g| g > MAX_GAP) {
                return Err(format!("{name}: at most {MAX_GAP}"));
            }
        }
        self.palette.check()?;
        if self.border > MAX_BORDER {
            return Err(format!("look.border: at most {MAX_BORDER}"));
        }
        if self.radius > MAX_RADIUS {
            return Err(format!("look.radius: at most {MAX_RADIUS}"));
        }
        Ok(())
    }
}

impl Default for Look {
    fn default() -> Self {
        Self {
            gap_outer: Some(12),
            gap_inner: Some(10),
            radius: 10,
            border: 2,
            theme: Theme::System,
            palette: Palette::default(),
        }
    }
}

/// The most that a gap of the look can be.
pub const MAX_GAP: u16 = 128;

/// The most that the border of the look can be.
pub const MAX_BORDER: u16 = 16;

/// The most that the radius of the look can be.
pub const MAX_RADIUS: u16 = 64;

/// The configuration of the plugin.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// What `MOD` means in the bindings.
    pub modifier: String,
    /// For tests only: take the keys of `input.inject` as keys of the
    /// user. Off, the bindings ignore injected keys.
    #[serde(default)]
    pub test_accept_injected_keys: bool,
    /// The gap of the old look, in logical pixels around each window: the
    /// edge gets it, two tiles get twice. It applies where `look` has
    /// `null` for a gap, and `set_layout` sets it.
    pub gap: u16,
    /// The look of the windows.
    #[serde(default)]
    pub look: Look,
    /// The number of tags of each output.
    pub tags: usize,
    /// The layouts in the order of `next_layout`.
    pub layouts: Vec<Layout>,
    /// The share of the master area of a new tag.
    pub master_width: f64,
    /// The number of master windows of a new tag.
    pub masters: usize,
    /// The number of stack columns of a new tag.
    pub columns: usize,
    pub new_window: NewWindow,
    pub focus: Focus,
    /// How a new window looks.
    pub style: Style,
    /// The window effects that tiling drives: `wobbly` and `lamp`.
    #[serde(default)]
    pub effects: Vec<chora_kit::effects::Kind>,
    /// Binding with `MOD`, and its action.
    pub bindings: BTreeMap<String, Action>,
    /// The words of a binding in the key list, by the binding as the file
    /// spells it; a binding without one gets the words of its action, and
    /// `run` its command.
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
}

impl Settings for Config {
    fn check(&self) -> Result<(), String> {
        if !(1..=MAX_TAGS).contains(&self.tags) {
            return Err(format!("tags: 1 to {MAX_TAGS}"));
        }
        if !MODIFIERS.contains(&self.modifier.as_str()) {
            return Err(format!("modifier: one of {MODIFIERS:?}"));
        }
        if self.layouts.is_empty() {
            return Err("layouts: at least one".into());
        }
        if !SHARE.contains(&self.master_width) {
            return Err("master_width: from 0.05 to 0.95".into());
        }
        self.look.check()?;
        if self.columns == 0 {
            return Err("columns: at least 1".into());
        }
        if !(0.0..=1.0).contains(&self.style.opacity) {
            return Err("style.opacity: from 0 to 1".into());
        }
        if self.effects.contains(&chora_kit::effects::Kind::Lamp) {
            return Err("effects: tiling has no task button, so only wobbly".into());
        }
        // Checked here, so that settings.set refuses it, and not at every
        // new window.
        for effect in &self.style.effects {
            chora_kit::check_file(Path::new(&effect.shader))
                .map_err(|e| format!("style.effects: no shader file {e}"))?;
        }
        let mut keys = BTreeMap::new();
        for (binding, action) in &self.bindings {
            // Two spellings of one key would lose one grab at runtime.
            if let Some(other) = keys.insert(self.binding(binding)?.to_string(), binding) {
                return Err(format!("{binding}: the same key as {other}"));
            }
            if let Action::Run(command) = action
                && command.trim().is_empty()
            {
                return Err(format!("{binding}: run takes a command"));
            }
            if let Action::Give(to) = action
                && to.trim().is_empty()
            {
                return Err(format!("{binding}: give takes a client name"));
            }
            if let Action::IncMasterWidth(step) = action
                && !step.is_finite()
            {
                return Err(format!("{binding}: inc_master_width takes a number"));
            }
            if let Action::ViewTag(n)
            | Action::ToggleTagView(n)
            | Action::MoveToTag(n)
            | Action::ToggleWindowTag(n) = action
                && !(1..=self.tags).contains(n)
            {
                return Err(format!("{binding}: no tag {n}"));
            }
        }
        Ok(())
    }
}

impl Config {
    /// The gaps in use: the one to the edge and the one between tiles.
    #[must_use]
    pub fn gaps(&self) -> (i32, i32) {
        let gap = i32::from(self.gap);
        (
            self.look.gap_outer.map_or(gap, i32::from),
            self.look.gap_inner.map_or(2 * gap, i32::from),
        )
    }

    /// The `MOD` chords that a program must not get: every key of
    /// [`SWALLOWED`] with `MOD`, `MOD+Shift` and `MOD+Ctrl`.
    pub fn swallowed(&self) -> impl Iterator<Item = Binding> + '_ {
        SWALLOW_WITH.iter().flat_map(move |with| {
            SWALLOWED.iter().filter_map(move |key| {
                let mut parts = vec!["MOD"];
                parts.extend(with.strip_suffix('+'));
                parts.push(key);
                self.binding(&parts.join("+")).ok()
            })
        })
    }

    /// A binding of the file, with `MOD` replaced by the modifier.
    pub fn binding(&self, text: &str) -> Result<Binding, String> {
        let replaced: Vec<&str> = text
            .split('+')
            .map(|part| if part == "MOD" { &*self.modifier } else { part })
            .collect();
        replaced
            .join("+")
            .parse()
            .map_err(|e| format!("{text}: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use chora_kit::{settings::parse, volume::Change};

    use super::*;

    #[test]
    fn a_key_twice_and_a_step_without_a_number_fail() {
        let twice = DEFAULT.replace(
            "  MOD+Ctrl+r: reload\n",
            "  MOD+Ctrl+r: reload\n  Super+j: close\n",
        );
        assert!(
            parse::<Config>(&twice)
                .unwrap_err()
                .contains("the same key as MOD+j")
        );
        let nan = DEFAULT.replace(
            "MOD+l: {inc_master_width: 0.05}",
            "MOD+l: {inc_master_width: .nan}",
        );
        assert_ne!(nan, DEFAULT);
        assert!(
            parse::<Config>(&nan)
                .unwrap_err()
                .contains("takes a number")
        );
    }

    #[test]
    fn every_mod_chord_is_swallowed_unless_bound() {
        let config = parse::<Config>(DEFAULT).unwrap();
        let chords: Vec<String> = config.swallowed().map(|b| b.to_string()).collect();
        for chord in ["Super+q", "Super+F5", "Shift+Super+x", "Ctrl+Super+z"] {
            assert!(chords.contains(&chord.to_owned()), "{chord}");
        }
        assert!(!chords.iter().any(|c| c.starts_with("Alt+")));
    }

    #[test]
    fn the_look_has_the_gaps_and_the_radius_of_the_design() {
        let config = parse::<Config>(DEFAULT).unwrap();
        assert_eq!(config.gaps(), (12, 10));
        assert_eq!(config.look.radius, 10);
        let old = DEFAULT.replace("gap_outer: 12", "gap_outer: null");
        let old = old.replace("gap_inner: 10", "gap_inner: null");
        assert_eq!(parse::<Config>(&old).unwrap().gaps(), (4, 8));
        let wide = DEFAULT.replace("gap_inner: 10", "gap_inner: 900");
        assert!(parse::<Config>(&wide).unwrap_err().contains("gap_inner"));
        let round = DEFAULT.replace("radius: 10", "radius: 900");
        assert!(parse::<Config>(&round).unwrap_err().contains("radius"));
    }

    #[test]
    fn a_binding_gives_the_focused_window() {
        let give = DEFAULT.replace(
            "MOD+Ctrl+r: reload",
            "MOD+g: {give: xp}\n  MOD+Ctrl+r: reload",
        );
        let config = parse::<Config>(&give).unwrap();
        assert_eq!(config.bindings["MOD+g"], Action::Give("xp".into()));
        let empty = give.replace("{give: xp}", "{give: ' '}");
        assert!(
            parse::<Config>(&empty)
                .unwrap_err()
                .contains("give takes a client name")
        );
    }

    #[test]
    fn the_default_is_valid() {
        let config = parse::<Config>(DEFAULT).unwrap();
        assert_eq!(config.tags, 9);
        assert_eq!(config.layouts[0], Layout::Tile);
        assert_eq!(
            config.bindings["MOD+Return"],
            Action::Run("chora-tiling --launch foot".into())
        );
        assert_eq!(
            config.binding("MOD+Shift+1").unwrap().to_string(),
            "Shift+Super+1"
        );
    }

    #[test]
    fn the_media_keys_change_the_volume() {
        let config = parse::<Config>(DEFAULT).unwrap();
        for (key, change) in [
            ("XF86AudioRaiseVolume", Change::Up),
            ("XF86AudioLowerVolume", Change::Down),
            ("XF86AudioMute", Change::Mute),
        ] {
            assert_eq!(config.bindings[key], Action::Volume(change));
            assert_eq!(config.binding(key).unwrap().to_string(), key);
        }
    }

    /// W9: settings.set took {"run": ""}.
    #[test]
    fn a_run_without_a_command_fails() {
        let run = DEFAULT.replace(
            "MOD+Return: {run: chora-tiling --launch foot}",
            "MOD+Return: {run: ' '}",
        );
        assert_ne!(run, DEFAULT);
        let err = parse::<Config>(&run).unwrap_err();
        assert!(err.contains("run takes a command"), "{err}");
    }

    #[test]
    fn mistakes_fail() {
        let wrong = DEFAULT.replace("master_width: 0.5", "master_width: 2");
        assert!(parse::<Config>(&wrong).is_err());
        let unknown = DEFAULT.replace("gap: 4", "gap: 4\nborder: 2");
        assert!(parse::<Config>(&unknown).is_err());
        let tag = DEFAULT.replace("{view_tag: 9}", "{view_tag: 10}");
        assert!(parse::<Config>(&tag).unwrap_err().contains("no tag 10"));
        let key = DEFAULT.replace("MOD+j:", "MOD+Hyper+j:");
        assert!(parse::<Config>(&key).is_err());
        let opacity = DEFAULT.replace("opacity: 1.0", "opacity: 1.5");
        let err = parse::<Config>(&opacity).unwrap_err();
        assert!(err.contains("style.opacity"), "{err}");
        let missing = DEFAULT.replace(
            "  effects: []",
            "  effects: [{shader: /nonexistent/chora-test.frag}]",
        );
        let err = parse::<Config>(&missing).unwrap_err();
        assert!(err.contains("/nonexistent/chora-test.frag"), "{err}");
        let proc = DEFAULT.replace("  effects: []", "  effects: [{shader: /proc/self/mem}]");
        let err = parse::<Config>(&proc).unwrap_err();
        assert!(err.contains("not a regular file"), "{err}");
        let lamp = DEFAULT.replace("\neffects: []\n", "\neffects: [lamp]\n");
        let err = parse::<Config>(&lamp).unwrap_err();
        assert!(err.contains("effects"), "{err}");
        let modifier = DEFAULT.replace("modifier: Super", "modifier: Shift");
        assert!(parse::<Config>(&modifier).is_err());
    }
}
