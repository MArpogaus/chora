//! The settings of the plugin (D62), a YAML file that lists every option.
//! The file `assistant.yaml` next to the crate is the default and the
//! reference. The values of the asset folder are the choices of the
//! flavor author.

use chora_draw::bubble::{Corner, Style};
use chora_kit::settings::Settings;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The default settings.
pub const DEFAULT: &str = include_str!("../assistant.yaml");

/// The settings of the plugin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The assistant shows while the voice agent is connected.
    pub enabled: bool,
    /// The corner of the first output where it shows.
    pub position: Corner,
}

impl Settings for Config {
    fn check(&self) -> Result<(), String> {
        Ok(())
    }
}

/// What the assistant looks like.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Figure {
    /// A dot with the elements of `dot.svg`, one per pose.
    #[default]
    Dot,
    /// The paperclip of the XP flavor, drawn in code until the paperclip
    /// is a set of SVG files.
    Paperclip,
}

/// The values of `values.yaml` of the asset folder.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Values {
    /// The base font family of the text; `sans-serif` is the system's.
    pub font: String,
    /// The figure.
    pub figure: Figure,
    /// The distance of the figure from the corner of the output.
    pub margin: [i32; 2],
    /// The look of the bubble of a sentence.
    pub speech: Style,
    /// The look of the buttons of a question; their tail is ignored.
    pub buttons: Style,
}

impl Default for Values {
    fn default() -> Self {
        Self {
            font: chora_draw::text::SANS_SERIF.into(),
            figure: Figure::default(),
            margin: [16, 16],
            speech: Style::default(),
            buttons: Style {
                fill: [235, 235, 235],
                radius: 4,
                ..Style::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use chora_kit::settings::parse;

    use super::*;

    #[test]
    fn the_default_shows_the_assistant_at_the_bottom_right() {
        let config: Config = parse(DEFAULT).unwrap();
        assert!(config.enabled);
        assert_eq!(config.position, Corner::BottomRight);
    }

    #[test]
    fn a_wrong_value_fails() {
        assert!(parse::<Config>("enabled: yes please\nposition: bottom_right").is_err());
        assert!(parse::<Config>("enabled: true\nposition: middle").is_err());
    }
}
