//! The settings of the plugin (D62). The file `mc.yaml` next to the crate
//! is the default and the reference.

use std::collections::BTreeMap;

use chora_kit::settings::Settings;
use chora_scene::{Binding, FontStyle, PropsUpdate};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::region::{self, Content, Preset, Region, Regions, Rule};

/// The default settings.
pub const DEFAULT: &str = include_str!("../mc.yaml");

/// The styles that the plugin uses itself.
const NEEDED: [&str; 7] = [
    "chapter_number",
    "chapter",
    "running_head",
    "heading",
    "body",
    "margin",
    "hud",
];
/// The most styles that one style can build on, in a row.
const MAX_DEPTH: usize = 8;

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
    /// Shows or hides a region; a docked app takes the focus first.
    Toggle(Region),
    /// Shows a content alone in a region over any scene, or hides it.
    Solo {
        content: String,
        region: Region,
    },
}

/// How an agent of `agent.set` shows in a state.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub sign: String,
    /// A color of the theme by name.
    pub color: String,
    /// The words for the state in the agents scene; the name without it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Light or dark.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Light,
    Dark,
}

/// The colors of both themes, RGBA from 0 to 1, by name. Each theme has
/// a `background`. A color that a theme of a file lacks comes from the
/// theme of the same name in `mc.yaml`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields, from = "Colors")]
pub struct Themes {
    pub light: BTreeMap<String, [f64; 4]>,
    pub dark: BTreeMap<String, [f64; 4]>,
}

/// The colors of both themes as a file has them.
#[derive(Clone, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Colors {
    light: BTreeMap<String, [f64; 4]>,
    dark: BTreeMap<String, [f64; 4]>,
}

impl From<Colors> for Themes {
    fn from(colors: Colors) -> Self {
        #[derive(Deserialize)]
        struct File {
            themes: Colors,
        }
        static DEFAULT_COLORS: std::sync::OnceLock<Colors> = std::sync::OnceLock::new();
        let default = DEFAULT_COLORS.get_or_init(|| {
            (serde_saphyr::from_str::<File>(DEFAULT))
                .map_or_else(|_| Colors::default(), |f| f.themes)
        });
        let fill = |mut own: BTreeMap<String, [f64; 4]>, all: &BTreeMap<String, [f64; 4]>| {
            for (name, rgba) in all {
                own.entry(name.clone()).or_insert(*rgba);
            }
            own
        };
        Self {
            light: fill(colors.light, &default.light),
            dark: fill(colors.dark, &default.dark),
        }
    }
}

/// A color of a theme by its name, or RGBA from 0 to 1.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum Color {
    Name(String),
    Rgba([f64; 4]),
}

/// How a text looks. A style builds on its `base` and changes the fields
/// that it sets; a scene can name a style and change single fields.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[allow(clippy::struct_field_names)] // `style` is the name of the file
pub struct Style {
    /// The style that this one changes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    /// From 1 to 1000: 300 light, 400 normal, 700 bold.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<FontStyle>,
    /// The font size in logical pixels; `em` or `step` without it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<f64>,
    /// The font size in em of `scale.base`: it grows with the base.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub em: Option<f64>,
    /// The font size as a step of the modular scale: `scale.base` times
    /// `scale.ratio` to the power of the step.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    /// The line height as a multiple of the size; the plugin rounds the
    /// line up to whole lines of the baseline grid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height: Option<f64>,
    /// Extra space after each letter, in em.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<f64>,
    /// OpenType features, for example `smcp: 1` for small caps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<BTreeMap<String, u32>>,
    /// Shows the text in capitals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uppercase: Option<bool>,
}

impl Style {
    /// This style over `under`: the fields that this one sets win.
    fn over(&self, under: Self) -> Self {
        let sized = self.size.is_some() || self.em.is_some() || self.step.is_some();
        Self {
            base: None,
            family: self.family.clone().or(under.family),
            weight: self.weight.or(under.weight),
            style: self.style.or(under.style),
            // A size, an em and a step replace each other.
            size: if sized { self.size } else { under.size },
            em: if sized { self.em } else { under.em },
            step: if sized { self.step } else { under.step },
            color: self.color.clone().or(under.color),
            line_height: self.line_height.or(under.line_height),
            letter_spacing: self.letter_spacing.or(under.letter_spacing),
            features: self.features.clone().or(under.features),
            uppercase: self.uppercase.or(under.uppercase),
        }
    }
}

/// A style with all its values, ready for a text node.
#[derive(Clone, Debug, PartialEq)]
pub struct Type {
    pub size: f64,
    /// The height of a line in logical pixels, whole lines of the grid.
    pub line: f64,
    pub color: [f64; 4],
    style: Style,
}

impl Type {
    /// The props of a text node with this type.
    pub fn props(&self, text: &str) -> PropsUpdate {
        let style = &self.style;
        let text = if style.uppercase == Some(true) {
            text.to_uppercase()
        } else {
            text.into()
        };
        PropsUpdate {
            // A text node needs a text.
            text: Some(if text.is_empty() { " ".into() } else { text }),
            family: style.family.clone(),
            font_size: Some(self.size),
            line_height: Some(self.line / self.size),
            color: Some(self.color),
            weight: style.weight,
            style: style.style,
            letter_spacing: style.letter_spacing,
            features: style.features.clone(),
            ..PropsUpdate::default()
        }
    }
}

/// The modular scale of the font sizes and the baseline grid.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Scale {
    /// The size of step 0 in logical pixels.
    pub base: f64,
    /// The factor from one step to the next.
    pub ratio: f64,
    /// The distance of the lines of the grid in logical pixels.
    pub baseline: f64,
}

/// How the head of a chapter looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    /// The number, a bar and the title on one line at the left edge of
    /// the text column.
    Number,
    /// No number and no title: the panels or the window start where the
    /// head would stand.
    None,
}

/// The head of each scene.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Chapter {
    pub layout: Layout,
}

/// The text block of a page.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Page {
    /// The width of a column of text, in em of `scale.base`.
    pub measure: f64,
    /// The width of the margin column left of the text, in em.
    pub margin: f64,
    /// The space between the margin and the text, and between columns,
    /// in em.
    pub gap: f64,
}

impl Page {
    pub fn check(&self) -> Result<(), String> {
        let Self {
            measure,
            margin,
            gap,
        } = *self;
        if !(10.0..=200.0).contains(&measure)
            || !(0.0..=50.0).contains(&margin)
            || !(0.0..=10.0).contains(&gap)
        {
            return Err("page: a measure from 10 to 200, a margin to 50, a gap to 10".into());
        }
        Ok(())
    }
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
    pub scale: Scale,
    /// The named styles of text.
    pub styles: BTreeMap<String, Style>,
    pub chapter: Chapter,
    pub page: Page,
    /// The least width and height of a panel, in em of `scale.base`. A
    /// narrow output shows fewer columns, a short one scrolls.
    pub min_panel: [f64; 2],
    pub write: Write,
    /// The scene at the start.
    pub default_scene: String,
    /// The order of the scenes, and so their numbers; the other scenes
    /// follow in the order they appeared.
    pub order: Vec<String>,
    /// The states of the agents of `agent.set`, by name.
    pub states: BTreeMap<String, State>,
    /// A binding and its action.
    pub bindings: BTreeMap<String, Action>,
    /// The regions around the page.
    pub regions: Regions,
    /// What the regions can show, by name.
    pub contents: BTreeMap<String, Content>,
    /// Which content goes to which region; the first rule that matches.
    pub rules: Vec<Rule>,
    /// The arrangement of the regions, a name of `presets`.
    pub preset: String,
    /// Arrangements of the regions by name.
    pub presets: BTreeMap<String, Preset>,
}

impl Config {
    /// The colors of the current theme.
    pub fn colors(&self) -> &BTreeMap<String, [f64; 4]> {
        match self.theme {
            Theme::Light => &self.themes.light,
            Theme::Dark => &self.themes.dark,
        }
    }

    /// The background of the current theme.
    pub fn background(&self) -> [f64; 4] {
        self.colors()["background"]
    }

    /// The type of a style of the sheet, changed by `local`, whose base
    /// is `name` when it names none.
    pub fn resolve(&self, name: &str, local: Option<&Style>) -> Result<Type, String> {
        let mut style = local.cloned().unwrap_or_default();
        let mut next = Some(style.base.clone().unwrap_or_else(|| name.into()));
        let mut depth = 0;
        while let Some(base) = next.take() {
            depth += 1;
            if depth > MAX_DEPTH {
                return Err(format!("styles.{name}: more than {MAX_DEPTH} bases"));
            }
            let under = (self.styles.get(&base)).ok_or_else(|| format!("no style {base}"))?;
            next.clone_from(&under.base);
            style = style.over(under.clone());
        }
        self.finish(name, style)
    }

    /// The size, the line and the color of a style.
    fn finish(&self, name: &str, style: Style) -> Result<Type, String> {
        let Scale {
            base,
            ratio,
            baseline,
        } = self.scale;
        let size = (style.size)
            .or(style.em.map(|em| base * em))
            .unwrap_or_else(|| base * ratio.powi(style.step.unwrap_or(0)));
        if !(4.0..=512.0).contains(&size) {
            return Err(format!("styles.{name}: a size from 4 to 512"));
        }
        let height = style.line_height.unwrap_or(1.2);
        if !(0.5..=4.0).contains(&height) {
            return Err(format!("styles.{name}: a line_height from 0.5 to 4"));
        }
        // Whole lines of the grid; a hair less than a line stays one.
        let line = baseline * (size * height / baseline - 0.05).ceil().max(1.0);
        let color = match style.color.clone().unwrap_or(Color::Name("text".into())) {
            Color::Rgba(rgba) => rgba,
            Color::Name(color) => *(self.colors().get(&color))
                .ok_or_else(|| format!("styles.{name}: no color {color}"))?,
        };
        if style.weight.is_some_and(|w| !(1..=1000).contains(&w))
            || style
                .letter_spacing
                .is_some_and(|l| !(-1.0..=1.0).contains(&l))
            || color.iter().any(|c| !(0.0..=1.0).contains(c))
        {
            return Err(format!(
                "styles.{name}: a weight from 1 to 1000, a letter_spacing from -1 to 1, \
                 colors from 0 to 1"
            ));
        }
        Ok(Type {
            size,
            line,
            color,
            style,
        })
    }
}

impl Config {
    /// The scale, the page and the least size of a panel.
    fn check_layout(&self) -> Result<(), String> {
        let Scale {
            base,
            ratio,
            baseline,
        } = self.scale;
        if !(4.0..=128.0).contains(&base)
            || !(1.0..=3.0).contains(&ratio)
            || !(4.0..=256.0).contains(&baseline)
        {
            return Err(
                "scale: a base from 4 to 128, a ratio from 1 to 3, a baseline from 4 to 256".into(),
            );
        }
        self.page.check()?;
        if self.min_panel.iter().any(|m| !(1.0..=200.0).contains(m)) {
            return Err("min_panel: a width and a height from 1 to 200".into());
        }
        Ok(())
    }
}

impl Settings for Config {
    fn check(&self) -> Result<(), String> {
        for (name, colors) in [("light", &self.themes.light), ("dark", &self.themes.dark)] {
            if colors.values().flatten().any(|c| !(0.0..=1.0).contains(c)) {
                return Err(format!("themes.{name}: each color from 0 to 1"));
            }
            if !colors.contains_key("background") {
                return Err(format!("themes.{name}: a background"));
            }
        }
        self.check_layout()?;
        for name in NEEDED {
            if !self.styles.contains_key(name) {
                return Err(format!("styles: no {name}"));
            }
        }
        // Both themes, so that a switch of the theme cannot fail.
        for theme in [Theme::Light, Theme::Dark] {
            let config = Self {
                theme,
                ..self.clone()
            };
            for name in self.styles.keys() {
                config.resolve(name, None)?;
            }
        }
        if !(0.0..=10_000.0).contains(&self.write.duration_ms) {
            return Err("write.duration_ms: from 0 to 10000".into());
        }
        for (name, state) in &self.states {
            let colors = [&self.themes.light, &self.themes.dark];
            if colors.iter().any(|c| !c.contains_key(&state.color)) {
                return Err(format!("states.{name}: no color {}", state.color));
            }
        }
        for (i, name) in self.order.iter().enumerate() {
            crate::scene::check_name(name).map_err(|e| format!("order: {e}"))?;
            if self.order[..i].contains(name) {
                return Err(format!("order: {name} twice"));
            }
        }
        let presets = (self.preset.as_str(), &self.presets);
        region::check(&self.regions, &self.contents, &self.rules, presets)?;
        let mut keys = BTreeMap::new();
        for binding in self.bindings.keys() {
            let parsed: Binding = binding.parse().map_err(|e| format!("{binding}: {e}"))?;
            // Two spellings of one key would lose one grab at runtime.
            if let Some(other) = keys.insert(parsed.to_string(), binding) {
                return Err(format!("{binding}: the same key as {other}"));
            }
            if let Action::Solo { content, region } = &self.bindings[binding] {
                let found = (self.contents.get(content))
                    .ok_or_else(|| format!("{binding}: no content {content}"))?;
                region::put_check(found, *region).map_err(|e| format!("{binding}: {e}"))?;
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
        assert_eq!(config.chapter.layout, Layout::Number);
        assert!(config.write.enabled);
        let slow = DEFAULT.replace("duration_ms: 900", "duration_ms: 99999");
        assert!(
            parse::<Config>(&slow)
                .unwrap_err()
                .contains("write.duration_ms")
        );
        let color = DEFAULT.replace("[0.11, 0.11, 0.11, 1.0]", "[2, 0, 0, 1]");
        assert!(parse::<Config>(&color).unwrap_err().contains("themes.dark"));
        let twice = format!("{DEFAULT}  super+a: next_scene\n");
        assert!(parse::<Config>(&twice).is_err());
        let gone = DEFAULT.replace("color: accent", "color: purple");
        assert!(
            parse::<Config>(&gone)
                .unwrap_err()
                .contains("no color purple")
        );
    }

    #[test]
    fn both_themes_fill_every_color_of_the_terminal() {
        // A file whose themes lack magenta, and cyan in the dark one.
        let text = DEFAULT.replace("    magenta: [0.52, 0.25, 0.50, 1.0]\n", "");
        let text = text.replace("    cyan: [0.52, 0.76, 0.76, 1.0]\n", "");
        assert!(!text.contains("magenta: [0.52") && !text.contains("cyan: [0.52"));
        let config = parse::<Config>(&text).unwrap();
        let shell = region::command(&config.contents["shell"].command, &config.themes.light);
        assert!(!shell.contains('{'), "{shell}");
        assert_eq!(config.themes.dark["cyan"], [0.52, 0.76, 0.76, 1.0]);

        let config = parse::<Config>(DEFAULT).unwrap();
        for colors in [&config.themes.light, &config.themes.dark] {
            let command = region::command(&config.contents["shell"].command, colors);
            assert!(!command.contains('{'), "{command}");
            assert!(command.contains("colors-dark.regular5="), "{command}");
        }
    }

    fn sheet() -> Config {
        let mut config = parse::<Config>(DEFAULT).unwrap();
        config.scale = Scale {
            base: 20.0,
            ratio: 1.5,
            baseline: 30.0,
        };
        config
    }

    #[test]
    fn a_style_builds_on_its_base_and_snaps_to_the_grid() {
        let mut config = sheet();
        let local = Style {
            base: Some("margin".into()),
            step: Some(2),
            color: Some(Color::Rgba([1.0, 0.0, 0.0, 1.0])),
            ..Style::default()
        };
        let margin = config.resolve("margin", None).unwrap();
        let ty = config.resolve("body", Some(&local)).unwrap();
        assert_eq!(ty.size, 45.0, "20 times 1.5 squared");
        assert_eq!(ty.color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(ty.style.family, margin.style.family, "from the base");
        assert_eq!(ty.line % 30.0, 0.0, "whole lines of the grid");
        assert!(ty.line >= 45.0);
        // A local size replaces the step of the base.
        config.styles.get_mut("margin").unwrap().step = Some(3);
        let sized = Style {
            size: Some(12.0),
            ..Style::default()
        };
        assert_eq!(config.resolve("margin", Some(&sized)).unwrap().size, 12.0);
        // An em follows the base: the running head of the default.
        let mut config = parse::<Config>(DEFAULT).unwrap();
        let head = |c: &Config| c.resolve("running_head", None).unwrap().size;
        assert!((head(&config) - 11.0).abs() < 0.01, "{}", head(&config));
        config.scale.base = 27.0;
        assert!((head(&config) - 16.5).abs() < 0.01, "{}", head(&config));
        let caps = Style {
            uppercase: Some(true),
            ..Style::default()
        };
        let props = config.resolve("body", Some(&caps)).unwrap().props("Agenda");
        assert_eq!(props.text.as_deref(), Some("AGENDA"));
    }

    #[test]
    fn a_loop_of_bases_fails() {
        let mut config = sheet();
        config.styles.get_mut("body").unwrap().base = Some("heading".into());
        config.styles.get_mut("heading").unwrap().base = Some("body".into());
        assert!(config.resolve("body", None).unwrap_err().contains("bases"));
        assert!(config.check().is_err());
    }
}
