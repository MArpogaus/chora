//! The settings of the plugin (D62), a YAML file that lists every option.
//! The file `assistant.yaml` next to the crate is the default and the
//! reference. The look comes from a HUD folder, named by `hud`, or from
//! the asset folder of a flavor (`--assets`).

use chora_draw::{
    bubble::{Corner, Style},
    theme::Sheet,
};
use chora_kit::{
    hud::{Anchor, Context, Hud, Source},
    settings::Settings,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::folders;

/// The default settings.
pub const DEFAULT: &str = include_str!("../assistant.yaml");

/// The name of the neutral folder, which `hud` names by default.
pub const NEUTRAL: &str = "neutral";

/// The smallest and the largest side of the figure of a voice panel.
const MIN_FIGURE: i32 = 12;
const MAX_FIGURE: i32 = 256;

/// The settings of the plugin.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The assistant shows while the voice agent is connected.
    pub enabled: bool,
    /// The corner of the first output where the figure shows; without it,
    /// the place that the HUD folder names.
    pub position: Option<Corner>,
    /// The name of the HUD folder that draws the assistant.
    pub hud: String,
}

impl Settings for Config {
    fn check(&self) -> Result<(), String> {
        if folders::good_name(&self.hud) {
            Ok(())
        } else {
            Err(format!(
                "hud: {:?} is not a folder name: 1 to 64 of a-z, A-Z, 0-9, _ and -",
                self.hud
            ))
        }
    }
}

/// What the figure shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Figure {
    /// The paperclip of the XP flavor, drawn in code until the paperclip
    /// is a set of SVG files.
    Paperclip,
    /// An element of an SVG file of the folder: the id is the template
    /// expanded with the state of the voice agent; a missing id shows
    /// `idle`.
    Sheet {
        /// The SVG file.
        sheet: Sheet,
        /// The template of the id of the element.
        template: String,
        /// The size of the SVG file; an element fills it.
        size: [i32; 2],
    },
    /// The dot of the host, for a folder without a usable voice panel.
    Dot,
}

/// What the assistant looks like, from a HUD folder or an asset folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Values {
    /// The base font family of the text.
    pub font: String,
    /// The figure.
    pub figure: Figure,
    /// Where the figure stands in the output.
    pub anchor: Anchor,
    /// The distance of the figure from the edges.
    pub margin: [i32; 2],
    /// A click on the figure mutes the mic.
    pub mutes: bool,
    /// The look of the bubble of a sentence.
    pub speech: Style,
    /// The template and the length of the text of a sentence.
    pub line: Line,
}

/// The panel of the sentence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    /// The template of the text: `{text}`.
    pub text: String,
    /// The most characters, if the folder sets it.
    pub max_chars: Option<usize>,
}

impl Default for Line {
    fn default() -> Self {
        Self {
            text: "{text}".into(),
            max_chars: None,
        }
    }
}

/// The values of the neutral dot: the corner and the margin of the
/// neutral folder.
const NEUTRAL_ANCHOR: Anchor = Anchor::BottomRight;
const NEUTRAL_MARGIN: [i32; 2] = [16, 16];

impl Values {
    /// The values of a HUD folder. `size` gives the size of a sheet. A folder
    /// without a voice panel that shows in the
    /// neutral context, with a picture and an element of at least 12 by 12
    /// pixels, gets the dot of the host.
    #[must_use]
    pub fn from_hud(hud: &Hud, size: impl Fn(Sheet) -> Option<[i32; 2]>) -> Self {
        let context = Context::default();
        let voice = hud.panels.iter().find(|panel| {
            panel.source == Source::Voice && panel.picture.is_some() && panel.when.matches(&context)
        });
        let sentence = hud
            .panels
            .iter()
            .find(|panel| panel.source == Source::Say && panel.when.matches(&context));
        let figure = voice.and_then(|panel| {
            let (file, template) = panel.picture.as_deref()?.split_once('#')?;
            let sheet = Sheet(leak(file));
            let size = size(sheet).filter(|[w, h]| {
                (MIN_FIGURE..=MAX_FIGURE).contains(w) && (MIN_FIGURE..=MAX_FIGURE).contains(h)
            })?;
            Some(Figure::Sheet {
                sheet,
                template: template.to_owned(),
                size,
            })
        });
        let (anchor, margin) = match (&figure, voice) {
            (Some(_), Some(panel)) => (
                panel.anchor.unwrap_or(NEUTRAL_ANCHOR),
                panel.margin.unwrap_or(NEUTRAL_MARGIN),
            ),
            _ => (NEUTRAL_ANCHOR, NEUTRAL_MARGIN),
        };
        let mutes = figure.is_none()
            || voice.is_some_and(|panel| panel.action == Some(chora_kit::hud::Action::Mute));
        Self {
            font: hud.font.clone(),
            mutes,
            figure: figure.unwrap_or(Figure::Dot),
            anchor,
            margin,
            speech: Style::default(),
            line: sentence.map_or_else(Line::default, |panel| Line {
                text: panel.text.clone().unwrap_or_else(|| "{text}".into()),
                max_chars: panel.max_chars.map(|n| n as usize),
            }),
        }
    }

    /// The dot of the host in the neutral place, where a click mutes.
    #[must_use]
    pub fn neutral(font: &str) -> Self {
        Self {
            font: font.to_owned(),
            figure: Figure::Dot,
            anchor: NEUTRAL_ANCHOR,
            margin: NEUTRAL_MARGIN,
            mutes: true,
            speech: Style::default(),
            line: Line::default(),
        }
    }

    /// The values of an asset folder of a flavor.
    #[must_use]
    pub fn from_assets(raw: Raw, size: impl Fn(Sheet) -> Option<[i32; 2]>) -> Self {
        let figure = match raw.figure {
            Kind::Paperclip => Figure::Paperclip,
            Kind::Dot => size(DOT).map_or(Figure::Dot, |size| Figure::Sheet {
                sheet: DOT,
                template: "{state}".into(),
                size,
            }),
        };
        Self {
            font: raw.font,
            figure,
            anchor: NEUTRAL_ANCHOR,
            margin: raw.margin,
            mutes: true,
            speech: raw.speech,
            line: Line::default(),
        }
    }
}

/// The sheet of the figure `dot` of an asset folder.
pub const DOT: Sheet = Sheet("dot.svg");

/// The name of a file of a folder as a name that lives as long as the
/// program; a folder names a few files.
fn leak(name: &str) -> &'static str {
    Box::leak(name.to_owned().into_boxed_str())
}

/// The figure of `values.yaml` of an asset folder.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// A dot with the elements of `dot.svg`, one per state.
    #[default]
    Dot,
    /// The paperclip of the XP flavor.
    Paperclip,
}

/// The values of `values.yaml` of an asset folder.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Raw {
    /// The base font family of the text; `sans-serif` is the system's.
    pub font: String,
    /// The figure.
    pub figure: Kind,
    /// The distance of the figure from the corner of the output.
    pub margin: [i32; 2],
    /// The look of the bubble of a sentence.
    pub speech: Style,
    /// Read and ignored: the question card is the host's.
    pub buttons: Style,
}

impl Default for Raw {
    fn default() -> Self {
        Self {
            font: chora_draw::text::SANS_SERIF.into(),
            figure: Kind::default(),
            margin: NEUTRAL_MARGIN,
            speech: Style::default(),
            buttons: Style::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chora_kit::{hud, settings::parse};

    use super::*;

    #[test]
    fn the_default_shows_the_assistant_where_the_folder_says() {
        let config: Config = parse(DEFAULT).unwrap();
        assert!(config.enabled);
        assert_eq!(config.position, None);
        assert_eq!(config.hud, NEUTRAL);
    }

    #[test]
    fn a_wrong_value_fails() {
        assert!(parse::<Config>("enabled: yes please\nposition: null\nhud: neutral").is_err());
        assert!(parse::<Config>("enabled: true\nposition: middle\nhud: neutral").is_err());
        assert!(parse::<Config>("enabled: true\nposition: null\nhud: ../x").is_err());
        assert!(parse::<Config>("enabled: true\nposition: top_left\nhud: mine").is_ok());
    }

    fn values(text: &str) -> Values {
        Values::from_hud(&hud::parse(text).unwrap(), |_| Some([24, 24]))
    }

    #[test]
    fn a_folder_without_a_voice_panel_gets_the_dot() {
        let only_a_line =
            "hud: 1\npanels:\n- {id: line, source: say, anchor: top, text: '{text}'}\n";
        let none = "hud: 1\n";
        let hidden = "hud: 1\npanels:\n- {id: f, source: voice, when: {wm: [xp]}, \
            anchor: top, picture: 'a.svg#{state}'}\n";
        let no_picture = "hud: 1\npanels:\n- {id: f, source: voice, anchor: top, text: x}\n";
        for text in [only_a_line, none, hidden, no_picture] {
            let values = values(text);
            assert_eq!(values.figure, Figure::Dot, "{text}");
            assert_eq!(values.anchor, Anchor::BottomRight, "{text}");
        }
        let tiny = Values::from_hud(
            &hud::parse(
                "hud: 1\npanels:\n- {id: f, source: voice, anchor: top, picture: 'a.svg#x'}\n",
            )
            .unwrap(),
            |_| Some([8, 40]),
        );
        assert_eq!(tiny.figure, Figure::Dot, "under 12 by 12 pixels");
        let huge = Values::from_hud(
            &hud::parse(
                "hud: 1\npanels:\n- {id: f, source: voice, anchor: top, picture: 'a.svg#x'}\n",
            )
            .unwrap(),
            |_| Some([257, 40]),
        );
        assert_eq!(huge.figure, Figure::Dot, "over 256 pixels");
        let missing = Values::from_hud(
            &hud::parse(
                "hud: 1\npanels:\n- {id: f, source: voice, anchor: top, picture: 'a.svg#x'}\n",
            )
            .unwrap(),
            |_| None,
        );
        assert_eq!(missing.figure, Figure::Dot, "no element idle");
    }

    #[test]
    fn a_voice_panel_sets_the_place_and_the_mute_click() {
        let text = "hud: 1\npanels:\n- {id: f, source: voice, anchor: top_left, \
            margin: [3, 4], picture: 'a.svg#{state}', action: mute}\n";
        let values = values(text);
        assert!(matches!(values.figure, Figure::Sheet { .. }));
        assert_eq!((values.anchor, values.margin), (Anchor::TopLeft, [3, 4]));
        assert!(values.mutes);
        let quiet = text.replace(", action: mute", "");
        assert!(!self::values(&quiet).mutes);
    }
}
