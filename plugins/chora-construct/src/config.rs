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
    /// The fine dots on the ground.
    pub dust: [f64; 3],
    /// The window list in the colours of mc (docs/mc-style.md): the
    /// ground, the text and the muted notes.
    pub paper: [f64; 3],
    pub text: [f64; 3],
    pub muted: [f64; 3],
}

impl Theme {
    #[must_use]
    pub fn palette(self) -> Palette {
        match self {
            Self::Light => Palette {
                sky: [0.915, 0.93, 0.95],
                horizon: [1.0, 1.0, 1.0],
                ground: [0.865, 0.85, 0.825],
                dust: [0.62, 0.6, 0.57],
                paper: [0.97, 0.96, 0.93],
                text: [0.16, 0.15, 0.14],
                muted: [0.50, 0.48, 0.45],
            },
            Self::Dark => Palette {
                sky: [0.0, 0.0, 0.0],
                horizon: [0.15, 0.15, 0.16],
                ground: [0.06, 0.06, 0.065],
                dust: [0.24, 0.24, 0.26],
                paper: [0.11, 0.11, 0.11],
                text: [0.85, 0.83, 0.79],
                muted: [0.55, 0.54, 0.51],
            },
        }
    }
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
    /// Keysym name to action.
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
        if !self.start.is_empty() {
            crate::file::check_name(&self.start).map_err(|e| format!("start: {e}"))?;
        }
        self.fly_out
            .parse::<Binding>()
            .map(drop)
            .map_err(|e| format!("fly_out: {}: {e}", self.fly_out))
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
        assert_eq!(config.keys["space"], Action::Launcher);
        assert_eq!(config.keys["Tab"], Action::Windows);
    }
}
