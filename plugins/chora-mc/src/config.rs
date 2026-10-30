//! The settings of the plugin (D62). The file `mc.yaml` next to the crate
//! is the default and the reference.

use std::collections::BTreeMap;

use chora_kit::settings::Settings;
use chora_scene::Binding;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The default settings.
pub const DEFAULT: &str = include_str!("../mc.yaml");

/// What a binding does.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    NextScene,
    PreviousScene,
    /// Shows a scene by its name.
    Scene(String),
    /// Runs a shell command as an app; its window gets a scene of its own.
    Run(String),
}

/// Light or dark.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Light,
    Dark,
}

/// The colors of a theme, RGBA from 0 to 1.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Colors {
    pub background: [f64; 4],
    pub text: [f64; 4],
    pub accent: [f64; 4],
}

/// The colors of both themes.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Themes {
    pub light: Colors,
    pub dark: Colors,
}

/// The font of all text.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Font {
    pub family: String,
    /// The size of the lines in logical pixels.
    pub size: f64,
}

/// How new text writes itself.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Write {
    pub enabled: bool,
    /// The time of the whole write.
    pub duration_ms: f64,
}

/// The settings of the plugin.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub theme: Theme,
    pub themes: Themes,
    pub font: Font,
    pub write: Write,
    /// The scene at the start.
    pub default_scene: String,
    /// A binding and its action.
    pub bindings: BTreeMap<String, Action>,
}

impl Config {
    /// The colors of the current theme.
    pub fn colors(&self) -> &Colors {
        match self.theme {
            Theme::Light => &self.themes.light,
            Theme::Dark => &self.themes.dark,
        }
    }
}

impl Settings for Config {
    fn check(&self) -> Result<(), String> {
        for (name, colors) in [("light", &self.themes.light), ("dark", &self.themes.dark)] {
            let all = [colors.background, colors.text, colors.accent];
            if all.iter().flatten().any(|c| !(0.0..=1.0).contains(c)) {
                return Err(format!("themes.{name}: each color from 0 to 1"));
            }
        }
        // The title is 1.5 times as large, and the core takes at most 1024.
        if !(4.0..=256.0).contains(&self.font.size) {
            return Err("font.size: from 4 to 256".into());
        }
        if !(0.0..=10_000.0).contains(&self.write.duration_ms) {
            return Err("write.duration_ms: from 0 to 10000".into());
        }
        let mut keys = BTreeMap::new();
        for binding in self.bindings.keys() {
            let parsed: Binding = binding.parse().map_err(|e| format!("{binding}: {e}"))?;
            // Two spellings of one key would lose one grab at runtime.
            if let Some(other) = keys.insert(parsed.to_string(), binding) {
                return Err(format!("{binding}: the same key as {other}"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chora_kit::settings::parse;

    use super::*;

    #[test]
    fn the_default_is_valid_and_mistakes_fail() {
        let config = parse::<Config>(DEFAULT).unwrap();
        assert_eq!(config.theme, Theme::Dark);
        assert_eq!(config.bindings["Super+a"], Action::Scene("agenda".into()));
        assert_eq!(config.font.family, "Inter");
        assert!(config.write.enabled);
        let slow = DEFAULT.replace("duration_ms: 900", "duration_ms: 99999");
        assert!(
            parse::<Config>(&slow)
                .unwrap_err()
                .contains("write.duration_ms")
        );
        let color = DEFAULT.replace("[0.07, 0.07, 0.07, 1.0]", "[2, 0, 0, 1]");
        assert!(parse::<Config>(&color).unwrap_err().contains("themes.dark"));
        let twice = format!("{DEFAULT}  super+a: next_scene\n");
        assert!(parse::<Config>(&twice).is_err());
    }
}
