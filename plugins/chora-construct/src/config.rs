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
    /// The light that glints in the glossy floor.
    pub sheen: [f64; 3],
}

impl Theme {
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
    /// Whether the construct draws the voice HUD (F28) and hides the dot
    /// of the voice agent.
    pub voice_hud: bool,
    pub style: Style,
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
        self.check_style()?;
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
        assert!(config.voice_hud);
        assert_eq!(config.style.font, "Inter");
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
}
