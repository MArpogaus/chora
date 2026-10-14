//! The configuration: a YAML file that writes out every option. The file
//! `tiling.yaml` next to the crate is the default and the reference.

use std::{collections::BTreeMap, path::Path};

use chora_scene::Binding;
use serde::{Deserialize, Serialize};

use crate::layout::Layout;

/// The default configuration.
pub const DEFAULT: &str = include_str!("../tiling.yaml");

/// The most tags per output: one bit each in a `u16`.
pub const MAX_TAGS: usize = 16;

/// Where a new window goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NewWindow {
    /// First in the list: the new master.
    Master,
    /// Last in the list.
    Stack,
}

/// How the focus follows the pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Focus {
    /// A click on a window focuses it.
    Click,
    /// The window under the pointer has the focus, except in the
    /// magnifier layout.
    FollowMouse,
}

/// What a binding does.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    /// Runs a shell command.
    Spawn(String),
    FocusNext,
    FocusPrevious,
    /// Focuses the window that had the focus before.
    FocusBack,
    /// Shows and focuses a window that asked for attention.
    FocusUrgent,
    SwapNext,
    SwapPrevious,
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
    /// Changes the master share of the current tag.
    MasterWidth(f64),
    /// Changes the number of master windows of the current tag.
    Masters(i32),
    /// Changes the number of columns of the current tag.
    Columns(i32),
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

/// The configuration of the plugin.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// What `MOD` means in the bindings.
    pub modifier: String,
    /// The gap around each window, in logical pixels.
    pub gap: u16,
    /// The tag names of each output.
    pub tags: Vec<String>,
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
    /// Binding with `MOD`, and its action.
    pub bindings: BTreeMap<String, Action>,
}

impl Config {
    /// Parses and checks a configuration.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut options = serde_saphyr::Options::default();
        options.strict_booleans = true;
        options.reject_unsupported_tags = true;
        let config: Self =
            serde_saphyr::from_str_with_options(text, options).map_err(|e| e.to_string())?;
        config.check()?;
        Ok(config)
    }

    /// Reads a configuration file.
    pub fn read(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::parse(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    fn check(&self) -> Result<(), String> {
        if self.tags.is_empty() || self.tags.len() > MAX_TAGS {
            return Err(format!("tags: 1 to {MAX_TAGS} names"));
        }
        if self.layouts.is_empty() {
            return Err("layouts: at least one".into());
        }
        if !(0.05..=0.95).contains(&self.master_width) {
            return Err("master_width: from 0.05 to 0.95".into());
        }
        if self.columns == 0 {
            return Err("columns: at least 1".into());
        }
        for (binding, action) in &self.bindings {
            self.binding(binding)?;
            if let Action::ViewTag(n)
            | Action::ToggleTagView(n)
            | Action::MoveToTag(n)
            | Action::ToggleWindowTag(n) = action
                && !(1..=self.tags.len()).contains(n)
            {
                return Err(format!("{binding}: no tag {n}"));
            }
        }
        Ok(())
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
    use super::*;

    #[test]
    fn the_default_is_valid() {
        let config = Config::parse(DEFAULT).unwrap();
        assert_eq!(config.tags.len(), 9);
        assert_eq!(config.layouts[0], Layout::Tile);
        assert_eq!(config.bindings["MOD+Return"], Action::Spawn("foot".into()));
        assert_eq!(
            config.binding("MOD+Shift+1").unwrap().to_string(),
            "Shift+Super+1"
        );
    }

    #[test]
    fn mistakes_fail() {
        let wrong = DEFAULT.replace("master_width: 0.5", "master_width: 2");
        assert!(Config::parse(&wrong).is_err());
        let unknown = DEFAULT.replace("gap: 4", "gap: 4\nborder: 2");
        assert!(Config::parse(&unknown).is_err());
        let tag = DEFAULT.replace("{view_tag: 9}", "{view_tag: 10}");
        assert!(Config::parse(&tag).unwrap_err().contains("no tag 10"));
        let key = DEFAULT.replace("MOD+j:", "MOD+Hyper+j:");
        assert!(Config::parse(&key).is_err());
    }
}
