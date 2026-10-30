//! The configuration: the settings of the plugin (D62), a YAML file that
//! lists every option. The file `tiling.yaml` next to the crate is the
//! default and the reference.

use std::collections::BTreeMap;

use chora_kit::settings::Settings;
use chora_scene::{Binding, ShaderEffect};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::layout::Layout;

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
}

/// How a new window looks (#99).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Style {
    /// From 0 to 1.
    pub opacity: f64,
    /// Shaders on the window, as in `scene.apply`.
    pub effects: Vec<ShaderEffect>,
}

/// The look of a window without a style.
impl Default for Style {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            effects: Vec::new(),
        }
    }
}

/// The configuration of the plugin.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// What `MOD` means in the bindings.
    pub modifier: String,
    /// The gap around each window, in logical pixels.
    pub gap: u16,
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
    /// Binding with `MOD`, and its action.
    pub bindings: BTreeMap<String, Action>,
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
        if self.columns == 0 {
            return Err("columns: at least 1".into());
        }
        if !(0.0..=1.0).contains(&self.style.opacity) {
            return Err("style.opacity: from 0 to 1".into());
        }
        let mut keys = BTreeMap::new();
        for (binding, action) in &self.bindings {
            // Two spellings of one key would lose one grab at runtime.
            if let Some(other) = keys.insert(self.binding(binding)?.to_string(), binding) {
                return Err(format!("{binding}: the same key as {other}"));
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
    use chora_kit::settings::parse;

    use super::*;

    #[test]
    fn a_key_twice_and_a_step_without_a_number_fail() {
        let twice = format!("{DEFAULT}\n  Super+j: close\n");
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
    fn the_default_is_valid() {
        let config = parse::<Config>(DEFAULT).unwrap();
        assert_eq!(config.tags, 9);
        assert_eq!(config.layouts[0], Layout::Tile);
        assert_eq!(config.bindings["MOD+Return"], Action::Run("foot".into()));
        assert_eq!(
            config.binding("MOD+Shift+1").unwrap().to_string(),
            "Shift+Super+1"
        );
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
        let modifier = DEFAULT.replace("modifier: Super", "modifier: Shift");
        assert!(parse::<Config>(&modifier).is_err());
    }
}
