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
}

/// Light or dark.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    /// The colour of the empty space and the colour of the fog, RGB.
    #[must_use]
    pub fn colors(self) -> ([f64; 3], [f64; 3]) {
        match self {
            Self::Light => ([1.0, 1.0, 1.0], [0.84, 0.86, 0.88]),
            Self::Dark => ([0.0, 0.0, 0.0], [0.13, 0.13, 0.14]),
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
    }
}
