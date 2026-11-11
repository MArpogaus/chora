//! The settings of the plugin (D62), a YAML file that lists every option.
//! The file `assistant.yaml` next to the crate is the default and the
//! reference. The look comes from a HUD folder, named by `hud`, or from
//! the asset folder of a flavor (`--assets`).

use std::collections::BTreeSet;

use chora_draw::{
    bubble::{Corner, Style},
    theme::Sheet,
};
use chora_kit::{
    hud::{Anchor, Context, Hud, Source, When},
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
    /// The name of the HUD folder that draws the assistant when no rule of
    /// `huds` matches.
    pub hud: String,
    /// Which folder draws in which context: the first rule that matches
    /// wins.
    #[serde(default)]
    pub huds: Vec<Rule>,
}

/// A rule of `huds`: the folder that draws while `when` matches the
/// context; without `when` it always matches.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    /// The condition on the context.
    #[serde(default)]
    pub when: When,
    /// The name of the HUD folder.
    pub folder: String,
}

impl Config {
    /// The folder that draws in a context: that of the first matching rule,
    /// else `hud`.
    #[must_use]
    pub fn folder(&self, context: &Context) -> &str {
        self.huds
            .iter()
            .find(|rule| rule.when.matches(context))
            .map_or(self.hud.as_str(), |rule| rule.folder.as_str())
    }
}

impl Settings for Config {
    fn check(&self) -> Result<(), String> {
        let mut errors = Vec::new();
        if !folders::good_name(&self.hud) {
            errors.push(format!(
                "hud: {:?} is not a folder name: 1 to 64 of a-z, A-Z, 0-9, _ and -",
                self.hud
            ));
        }
        for (index, rule) in self.huds.iter().enumerate() {
            let at = format!("huds[{index}]");
            if !folders::good_name(&rule.folder) {
                errors.push(format!(
                    "{at}.folder: {:?} is not a folder name: 1 to 64 of a-z, A-Z, 0-9, _ and -",
                    rule.folder
                ));
            }
            rule.when.check(&at, &mut errors);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }
}

/// What the figure shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Figure {
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
    /// Where the "Chat ..." row stands and its distance from the edges.
    pub rows: (Anchor, [i32; 2]),
    /// The most rows of helpers, and the colours of their words.
    pub helpers: (usize, chora_kit::hud::Colours),
    /// The fill and the hairline of the rows, if the folder names them.
    pub row_box: (Option<[u8; 3]>, Option<[u8; 3]>),
    /// The fields of the context that the folder reads in a `when`, so
    /// that the look is built again when one of them changes.
    pub watches: BTreeSet<String>,
    /// The folder that the values come from, to pick them again in another
    /// context without loading it again; none for the dot and an asset
    /// folder.
    pub hud: Option<Hud>,
}

/// The panel of the sentence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    /// The template of the text: `{text}`.
    pub text: String,
    /// The most characters, if the folder sets it.
    pub max_chars: Option<usize>,
    /// The sentence has no box and stands beside the figure.
    pub plain: bool,
    /// The tail of the bubble is near its left end, and the bubble grows
    /// to the right.
    pub tail_left: bool,
}

impl Default for Line {
    fn default() -> Self {
        Self {
            text: "{text}".into(),
            max_chars: None,
            plain: false,
            tail_left: false,
        }
    }
}

/// The values of the neutral dot: the corner and the margin of the
/// neutral folder.
const NEUTRAL_ANCHOR: Anchor = Anchor::BottomRight;
const NEUTRAL_MARGIN: [i32; 2] = [16, 16];
const NEUTRAL_ROWS: (Anchor, [i32; 2]) = (Anchor::TopRight, [16, 16]);
/// The height of the tail of a bubble with a tail.
const TAIL: i32 = 18;

/// The look of the sentence of a panel: its fill, hairline and text
/// colour over those of the neutral bubble, and a tail if it names one.
/// A plain sentence takes the text colour and the fill of the chat, of
/// the folder or of the host: the fill backs the text without a hairline.
fn speech_of(panel: &chora_kit::hud::Panel, hud: &Hud) -> Style {
    let neutral = Style::default();
    let plain = panel.style == Some(chora_kit::hud::Style::Plain);
    let host = chora_kit::hud::Chat::host();
    let page = |own: Option<[u8; 3]>, hosts: Option<[u8; 3]>| own.or(hosts).filter(|_| plain);
    let (paper, ink) = (
        page(hud.chat.fill, host.fill),
        page(hud.chat.text, host.text),
    );
    Style {
        fill: panel.fill.or(paper).unwrap_or(neutral.fill),
        border: panel.border.unwrap_or(neutral.border),
        text: panel.colours.text.or(ink).unwrap_or(neutral.text),
        tail: if panel.tail.is_some() { TAIL } else { 0 },
        ..neutral
    }
}

/// The text of the sentence of a panel, and where its box stands.
fn line_of(panel: &chora_kit::hud::Panel) -> Line {
    Line {
        text: panel.text.clone().unwrap_or_else(|| "{text}".into()),
        max_chars: panel.max_chars.map(|n| n as usize),
        plain: panel.style == Some(chora_kit::hud::Style::Plain),
        tail_left: panel.tail == Some(chora_kit::hud::Tail::Left),
    }
}

/// The most rows of helpers of a folder that names no number.
pub const MAX_ROWS: usize = 4;

/// The panel of the helpers that shows in a context.
fn helpers_panel<'a>(hud: &'a Hud, context: &Context) -> Option<&'a chora_kit::hud::Panel> {
    hud.panels.iter().find(|panel| {
        panel.source == Source::Agents
            && panel.action == Some(chora_kit::hud::Action::Chat)
            && panel.when.matches(context)
    })
}

/// The rows of helpers of a folder in a context where no panel of the
/// helpers shows: none when the folder has such a panel, since its `when`
/// hides them, else those of the host.
fn hidden_rows(hud: &Hud) -> usize {
    let has_panel = hud.panels.iter().any(|panel| {
        panel.source == Source::Agents && panel.action == Some(chora_kit::hud::Action::Chat)
    });
    if has_panel { 0 } else { MAX_ROWS }
}

/// Where the "Chat ..." row stands: where the panel of the helpers puts
/// it, also while its `when` hides the helpers, else beside the figure,
/// which a click on the row never covers.
fn rows_of(
    hud: &Hud,
    context: &Context,
    (anchor, margin): (Anchor, [i32; 2]),
    figure_height: i32,
) -> (Anchor, [i32; 2]) {
    let hidden = hud.panels.iter().find(|panel| {
        panel.source == Source::Agents && panel.action == Some(chora_kit::hud::Action::Chat)
    });
    helpers_panel(hud, context).or(hidden).map_or(
        (anchor, [margin[0], margin[1] + figure_height + ROW_GAP]),
        |panel| {
            (
                panel.anchor.unwrap_or(NEUTRAL_ROWS.0),
                panel.margin.unwrap_or(NEUTRAL_ROWS.1),
            )
        },
    )
}

/// The gap between the figure of an asset folder and its Chat row.
const ROW_GAP: i32 = 8;
/// The height of the dot of the host.
const NEUTRAL_FIGURE: i32 = 24;

impl Values {
    /// The values of a HUD folder in a context. `size` gives the size of a
    /// sheet. A folder without a voice panel that shows in the context, with
    /// a picture and an element of at least 12 by 12 pixels, gets the dot
    /// of the host.
    #[must_use]
    pub fn from_hud(
        hud: &Hud,
        context: &Context,
        size: impl Fn(Sheet) -> Option<[i32; 2]>,
    ) -> Self {
        let voice = hud.panels.iter().find(|panel| {
            panel.source == Source::Voice && panel.picture.is_some() && panel.when.matches(context)
        });
        let sentence = hud
            .panels
            .iter()
            .find(|panel| panel.source == Source::Say && panel.when.matches(context));
        let picture = voice
            .and_then(|panel| panel.picture.as_deref()?.split_once('#'))
            .or_else(|| hud.figure.as_deref().map(|file| (file, "{state}")));
        let figure = picture.and_then(|(file, template)| {
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
        let height = figure
            .as_ref()
            .map_or(NEUTRAL_FIGURE, |figure| figure.size()[1]);
        let mutes = figure.is_none()
            || voice.is_none_or(|panel| panel.action == Some(chora_kit::hud::Action::Mute));
        Self {
            font: hud.font.clone(),
            mutes,
            figure: figure.unwrap_or(Figure::Dot),
            anchor,
            margin,
            speech: sentence.map_or_else(Style::default, |panel| speech_of(panel, hud)),
            line: sentence.map_or_else(Line::default, line_of),
            rows: rows_of(hud, context, (anchor, margin), height),
            helpers: helpers_panel(hud, context).map_or_else(
                || (hidden_rows(hud), chora_kit::hud::Colours::default()),
                |panel| {
                    let most = panel.max_rows.map_or(MAX_ROWS, |n| n as usize);
                    (most, panel.colours)
                },
            ),
            row_box: helpers_panel(hud, context)
                .map_or((None, None), |panel| (panel.fill, panel.border)),
            watches: hud
                .panels
                .iter()
                .flat_map(|panel| panel.when.keys())
                .map(str::to_owned)
                .collect(),
            hud: Some(hud.clone()),
        }
    }

    /// The values of the same folder in another context, none for values
    /// that do not come from a folder.
    #[must_use]
    pub fn repick(
        &self,
        context: &Context,
        size: impl Fn(Sheet) -> Option<[i32; 2]>,
    ) -> Option<Self> {
        self.hud
            .as_ref()
            .map(|hud| Self::from_hud(hud, context, size))
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
            rows: NEUTRAL_ROWS,
            helpers: (MAX_ROWS, chora_kit::hud::Colours::default()),
            row_box: (None, None),
            watches: BTreeSet::new(),
            hud: None,
        }
    }

    /// The values of an asset folder of a flavor.
    #[must_use]
    pub fn from_assets(raw: Raw, size: impl Fn(Sheet) -> Option<[i32; 2]>) -> Self {
        let figure = size(DOT).map_or(Figure::Dot, |size| Figure::Sheet {
            sheet: DOT,
            template: "{state}".into(),
            size,
        });
        let above = raw.margin[1] + figure.size()[1] + ROW_GAP;
        Self {
            font: raw.font,
            figure,
            anchor: NEUTRAL_ANCHOR,
            margin: raw.margin,
            mutes: true,
            speech: raw.speech,
            line: Line::default(),
            rows: (Anchor::BottomRight, [raw.margin[0], above]),
            helpers: (MAX_ROWS, chora_kit::hud::Colours::default()),
            row_box: (None, None),
            watches: BTreeSet::new(),
            hud: None,
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

/// The values of `values.yaml` of an asset folder.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Raw {
    /// The base font family of the text; `sans-serif` is the system's.
    pub font: String,
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

    fn with_rules(rules: &str) -> Result<Config, String> {
        parse(&format!(
            "enabled: true\nposition: null\nhud: neutral\n{rules}"
        ))
    }

    fn context(wm: &str, app: &str) -> Context {
        Context {
            wm: wm.into(),
            space: wm.into(),
            app: app.into(),
            ..Context::default()
        }
    }

    #[test]
    fn the_first_rule_that_matches_picks_the_folder() {
        let config = with_rules(
            "huds:\n  - {when: {app: foot}, folder: mine}\n  - {when: {wm: [xp, mc]}, folder: xp}\n  - {folder: last}\n",
        )
        .unwrap();
        assert_eq!(config.folder(&context("xp", "foot")), "mine");
        assert_eq!(config.folder(&context("mc", "firefox")), "xp");
        assert_eq!(config.folder(&context("tiling", "firefox")), "last");
    }

    #[test]
    fn hud_is_the_folder_when_no_rule_matches() {
        let config = with_rules("huds:\n  - {when: {wm: xp}, folder: xp}\n").unwrap();
        assert_eq!(config.folder(&context("tiling", "")), "neutral");
        assert_eq!(
            with_rules("").unwrap().folder(&context("xp", "")),
            "neutral"
        );
    }

    #[test]
    fn a_bad_rule_fails_with_its_place() {
        let bad_folder = with_rules("huds:\n  - {folder: ../x}\n").unwrap_err();
        assert!(bad_folder.contains("huds[0].folder"), "{bad_folder}");
        let bad_key = with_rules("huds:\n  - {when: {mood: calm}, folder: a}\n").unwrap_err();
        assert!(bad_key.contains("huds[0].when.mood"), "{bad_key}");
        let bad_value =
            with_rules("huds:\n  - {folder: a}\n  - {when: {chat: opened}, folder: a}\n")
                .unwrap_err();
        assert!(bad_value.contains("huds[1].when.chat"), "{bad_value}");
        assert!(
            with_rules("huds:\n  - {when: {wm: xp}}\n").is_err(),
            "a rule names a folder"
        );
    }

    #[test]
    fn the_default_has_no_rules() {
        assert_eq!(parse::<Config>(DEFAULT).unwrap().huds.len(), 0);
    }

    #[test]
    fn a_wrong_value_fails() {
        assert!(parse::<Config>("enabled: yes please\nposition: null\nhud: neutral").is_err());
        assert!(parse::<Config>("enabled: true\nposition: middle\nhud: neutral").is_err());
        assert!(parse::<Config>("enabled: true\nposition: null\nhud: ../x").is_err());
        assert!(parse::<Config>("enabled: true\nposition: top_left\nhud: mine").is_ok());
    }

    fn values(text: &str) -> Values {
        Values::from_hud(&hud::parse(text).unwrap(), &Context::default(), |_| {
            Some([24, 24])
        })
    }

    #[test]
    fn the_figure_of_the_folder_shows_without_a_voice_panel() {
        let hud = hud::parse(
            "hud: 1\nfigure: paperclip.svg\npanels:\n  - {id: line, source: say, anchor: \
             bottom_right, text: '{text}'}\n",
        )
        .unwrap();
        let values = Values::from_hud(&hud, &Context::default(), |_| Some([80, 116]));
        let Figure::Sheet {
            sheet,
            template,
            size,
        } = &values.figure
        else {
            panic!("the figure is {:?}", values.figure);
        };
        assert_eq!(
            (sheet.0, template.as_str(), *size),
            ("paperclip.svg", "{state}", [80, 116])
        );
        assert!(values.mutes, "a click on the figure mutes");
        assert_eq!(
            (values.anchor, values.margin),
            (NEUTRAL_ANCHOR, NEUTRAL_MARGIN)
        );
    }

    #[test]
    fn helper_rows_follow_the_when_of_their_panel() {
        let hud = hud::parse(
            "hud: 1\npanels:\n  - {id: tray, source: agents, when: {chat: none}, anchor: \
             bottom_right, rows: x, max_rows: 3, action: chat}\n",
        )
        .unwrap();
        let rows_in = |chat: &str| {
            let context = Context {
                chat: chat.into(),
                ..Context::default()
            };
            Values::from_hud(&hud, &context, |_| None).helpers.0
        };
        assert_eq!(rows_in("none"), 3);
        assert_eq!(rows_in("open"), 0, "the panel does not show");
        let hidden = Context {
            chat: "open".into(),
            ..Context::default()
        };
        let place = Values::from_hud(&hud, &hidden, |_| None).rows;
        assert_eq!(
            place,
            (Anchor::BottomRight, NEUTRAL_ROWS.1),
            "the Chat row keeps the place of its panel"
        );
        let plain = hud::parse("hud: 1\n").unwrap();
        let values = Values::from_hud(&plain, &Context::default(), |_| None);
        assert_eq!(
            values.helpers.0, MAX_ROWS,
            "no panel of helpers: the rows of the host"
        );
    }

    #[test]
    fn the_sentence_takes_the_fill_and_the_tail_of_its_panel() {
        let hud = hud::parse(
            "hud: 1\npanels:\n  - {id: line, source: say, anchor: bottom_right, style: bubble, \
             text: '{text}', fill: [255, 255, 225], border: [0, 0, 0], tail: right}\n",
        )
        .unwrap();
        let values = Values::from_hud(&hud, &Context::default(), |_| None);
        assert_eq!(values.speech.fill, [255, 255, 225]);
        assert_eq!(values.speech.border, [0, 0, 0]);
        assert!(values.speech.tail > 0, "a tail");
        assert!(!values.line.plain && !values.line.tail_left);
        let plain = hud::parse(
            "hud: 1\npanels:\n  - {id: line, source: say, anchor: top_right, style: plain, \
             text: '{text}'}\nchat: {fill: [28, 28, 28], text: [217, 212, 201]}\n",
        )
        .unwrap();
        let values = Values::from_hud(&plain, &Context::default(), |_| None);
        assert!(values.line.plain);
        assert_eq!(values.speech.text, [217, 212, 201], "the text of the chat");
        assert_eq!(
            values.speech.fill,
            [28, 28, 28],
            "the fill of the chat backs it"
        );
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
            &Context::default(),
            |_| Some([8, 40]),
        );
        assert_eq!(tiny.figure, Figure::Dot, "under 12 by 12 pixels");
        let huge = Values::from_hud(
            &hud::parse(
                "hud: 1\npanels:\n- {id: f, source: voice, anchor: top, picture: 'a.svg#x'}\n",
            )
            .unwrap(),
            &Context::default(),
            |_| Some([257, 40]),
        );
        assert_eq!(huge.figure, Figure::Dot, "over 256 pixels");
        let missing = Values::from_hud(
            &hud::parse(
                "hud: 1\npanels:\n- {id: f, source: voice, anchor: top, picture: 'a.svg#x'}\n",
            )
            .unwrap(),
            &Context::default(),
            |_| None,
        );
        assert_eq!(missing.figure, Figure::Dot, "no element idle");
    }

    #[test]
    fn a_panel_follows_the_context_and_the_folder_names_what_it_reads() {
        let text = "hud: 1\npanels:\n- {id: f, source: voice, when: {wm: xp}, anchor: top_left, \
            picture: 'a.svg#{state}'}\n- {id: g, source: voice, when: {app: [foot, vim]}, \
            anchor: top, text: x}\n";
        let hud = hud::parse(text).unwrap();
        let in_context = |wm: &str| {
            let context = Context {
                wm: wm.into(),
                ..Context::default()
            };
            Values::from_hud(&hud, &context, |_| Some([24, 24]))
        };
        assert!(matches!(in_context("xp").figure, Figure::Sheet { .. }));
        assert_eq!(in_context("tiling").figure, Figure::Dot);
        let watches: Vec<_> = in_context("xp").watches.into_iter().collect();
        assert_eq!(watches, ["app", "wm"]);
    }

    #[test]
    fn the_values_are_picked_again_from_the_folder_in_another_context() {
        let text = "hud: 1\npanels:\n- {id: f, source: voice, when: {state: speaking}, \
            anchor: top_left, picture: 'a.svg#{state}'}\n";
        let hud = hud::parse(text).unwrap();
        let quiet = Context {
            state: "listening".into(),
            ..Context::default()
        };
        let values = Values::from_hud(&hud, &quiet, |_| Some([24, 24]));
        assert_eq!(values.figure, Figure::Dot);
        let speaking = Context {
            state: "speaking".into(),
            ..Context::default()
        };
        let again = values.repick(&speaking, |_| Some([24, 24])).unwrap();
        assert!(matches!(again.figure, Figure::Sheet { .. }));
        assert_eq!(
            Values::neutral("sans-serif").repick(&speaking, |_| None),
            None
        );
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

    #[test]
    fn a_folder_without_a_chat_panel_puts_the_row_at_its_figure() {
        let text = "hud: 1\npanels:\n- {id: f, source: voice, anchor: bottom_left, margin: [8, 8], \
                    picture: 'dot.svg#{state}'}\n";
        let values = Values::from_hud(&hud::parse(text).unwrap(), &Context::default(), |_| {
            Some([24, 24])
        });
        assert_eq!(values.rows.0, Anchor::BottomLeft);
    }
}
