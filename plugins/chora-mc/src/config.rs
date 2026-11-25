//! The settings of the plugin (D62). The file `mc.yaml` next to the crate
//! is the default and the reference.

use std::collections::BTreeMap;

use chora_kit::actions::System;
use chora_kit::settings::{Common, Settings};
use chora_scene::{Binding, FontStyle, PropsUpdate};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::region::{self, Content, Preset, Region, Regions, Rule};

/// The default settings.
pub const DEFAULT: &str = include_str!("../mc.yaml");

/// The styles that the plugin uses itself.
const NEEDED: [&str; 8] = [
    "chapter_number",
    "chapter",
    "running_head",
    "heading",
    "body",
    "margin",
    "hud",
    "label",
];
/// The most styles that one style can build on, in a row.
const MAX_DEPTH: usize = 8;

/// What a binding of mc does besides the actions of every look.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Look {
    NextScene,
    PreviousScene,
    /// One step back as Escape on a page; a window that holds the keyboard
    /// leaves it to its page first.
    Back,
    /// Shows a scene by its name.
    Scene(String),
    /// Shows or hides a region; a docked app takes the focus first.
    Toggle(Region),
    /// Shows a content alone in a region over any scene, or hides it.
    Solo {
        content: String,
        region: Region,
    },
    /// Does nothing and keeps the key from the window: the action of
    /// every Super chord that no binding uses. It is not a setting.
    #[serde(skip)]
    Swallow,
}

/// What a binding does: an action of every look, or one of mc.
pub type Action = chora_kit::actions::Action<Look>;

impl From<Look> for Action {
    fn from(look: Look) -> Self {
        Self::Look(look)
    }
}

/// Whether mc does something for a system action: the others have no
/// meaning in a look of pages, and a binding to one is refused.
fn acts_in_mc(system: &System) -> bool {
    !matches!(
        system,
        System::Terminal | System::Launcher | System::Close | System::Desk(_) | System::Cube(_)
    )
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
#[allow(clippy::struct_field_names, reason = "`style` is the name of the file")]
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
    /// The family of this type, as written in the style.
    pub fn family(&self) -> Option<&str> {
        self.style.family.as_deref()
    }

    pub fn weight(&self) -> Option<u16> {
        self.style.weight
    }

    /// The capitals of this type, from the top of its text box.
    pub fn ink(&self) -> crate::fonts::Ink {
        let style = &self.style;
        let family = style.family.as_deref();
        crate::fonts::ink(family, style.weight, self.size, self.line)
    }

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
            family: style.family.as_deref().map(crate::fonts::pick),
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

/// Who draws the voice HUD in mc: the HUD host with the
/// folder of mc, which draws the state, the sentence and the question of
/// the voice agent; mc shows only the helpers and claims nothing. The
/// setting chose before; `own`, the HUD of mc itself, is gone (KIT7) and
/// reads as `host`, with a warning, for one release.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VoiceHud {
    #[default]
    Host,
}

impl<'de> Deserialize<'de> for VoiceHud {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        match name.as_str() {
            "host" => Ok(Self::Host),
            "own" => {
                tracing::warn!(
                    "voice_hud: own is gone; the HUD host draws the voice \
                     HUD of mc (read as host)"
                );
                Ok(Self::Host)
            }
            _ => Err(serde::de::Error::unknown_variant(&name, &["host", "own"])),
        }
    }
}

/// The settings of the plugin.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub theme: Theme,
    /// Who draws the voice HUD: only `host`; read for one release, never
    /// written.
    #[serde(default, skip_serializing)]
    #[allow(dead_code, reason = "an old file loads with it; nothing chooses by it")]
    pub voice_hud: VoiceHud,
    /// The bindings and the switch of the tests, as in every look. With
    /// `test_accept_injected_keys` the hits and the agent columns take
    /// injected presses too.
    #[serde(flatten)]
    pub common: Common<Look>,
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
        for binding in self.common.bindings.keys() {
            let parsed: Binding = binding.parse().map_err(|e| format!("{binding}: {e}"))?;
            // Two spellings of one key would lose one grab at runtime.
            if let Some(other) = keys.insert(parsed.to_string(), binding) {
                return Err(format!("{binding}: the same key as {other}"));
            }
            if let Action::System(unfit) = &self.common.bindings[binding]
                && !acts_in_mc(unfit)
            {
                return Err(format!("{binding}: mc has no action for {unfit:?}"));
            }
            if let Action::Look(Look::Solo { content, region }) = &self.common.bindings[binding] {
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

    /// The relative luminance of a color, as WCAG 2 has it.
    fn luminance([r, g, b, _]: [f64; 4]) -> f64 {
        let lin = |c: f64| {
            if c <= 0.039_28 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)
    }

    fn contrast(a: [f64; 4], b: [f64; 4]) -> f64 {
        let (x, y) = (luminance(a), luminance(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    /// The colors that carry text read on the paper at 7:1 in both themes.
    #[test]
    fn the_text_colors_have_contrast_on_the_page() {
        let config = parse::<Config>(DEFAULT).unwrap();
        for (theme, colors) in [
            ("light", &config.themes.light),
            ("dark", &config.themes.dark),
        ] {
            let page = colors["background"];
            let names = ["text", "muted", "accent", "green", "amber", "blue", "red"];
            for (name, least) in names.map(|c| (c, 7.0)) {
                let ratio = contrast(colors[name], page);
                assert!(
                    ratio >= least,
                    "{theme} {name}: {ratio:.2} is under {least}"
                );
            }
        }
    }

    /// Nothing that a person reads is under 14 pixels: the review of the
    /// look found the running head and the clock at 11.6.
    #[test]
    fn no_style_of_the_default_is_under_14_pixels() {
        let config = parse::<Config>(DEFAULT).unwrap();
        for name in config.styles.keys() {
            let size = config.resolve(name, None).unwrap().size;
            assert!(size >= 14.0, "{name} is {size} px");
        }
    }

    /// The plugin reads the style of the margin labels, so a file without
    /// it is refused with a message instead of panicking the plugin.
    #[test]
    fn a_styles_map_without_the_label_style_is_refused() {
        let mut config = parse::<Config>(DEFAULT).unwrap();
        config.styles.remove("label");
        assert_eq!(config.check(), Err("styles: no label".into()));
    }

    /// The default drags a window of mc with Super and the left button.
    #[test]
    fn the_default_binds_the_drag_to_a_pointer_button() {
        let config = parse::<Config>(DEFAULT).unwrap();
        assert_eq!(
            config.common.bindings["Super+BTN_LEFT"],
            Action::System(System::Drag)
        );
    }

    /// The host draws the voice HUD: one voice indicator. The old values
    /// `host` and `own` still load, as `host`, for one release; any other
    /// value fails, and a save writes no `voice_hud`.
    #[test]
    fn the_voice_hud_is_the_host_and_old_values_load() {
        let config = parse::<Config>(DEFAULT).unwrap();
        assert_eq!(config.voice_hud, VoiceHud::Host);
        let with = |value: &str| {
            parse::<Config>(&format!("{DEFAULT}voice_hud: {value}\n")).map(|c| c.voice_hud)
        };
        assert_eq!(with("host"), Ok(VoiceHud::Host));
        assert_eq!(with("own"), Ok(VoiceHud::Host));
        for wrong in ["maybe", "true", "false"] {
            assert!(with(wrong).is_err(), "{wrong}");
        }
        let written = serde_json::to_value(&config).unwrap();
        assert!(written.get("voice_hud").is_none());
    }

    /// lc-2: a warn of mc reaches stderr through the kit subscriber with
    /// its level and its target (the crate of mc). The test starts itself
    /// as the child, which starts the kit log and loads `voice_hud: own`.
    #[test]
    fn a_warn_of_mc_reaches_stderr_with_level_and_target() {
        if std::env::var_os("CHORA_MC_LOG_CHILD").is_some() {
            chora_kit::log::init();
            let _ = parse::<Config>(&format!("{DEFAULT}voice_hud: own\n"));
            return;
        }
        let out = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "--nocapture",
                "config::tests::a_warn_of_mc_reaches_stderr_with_level_and_target",
            ])
            .env("CHORA_MC_LOG_CHILD", "1")
            .env("RUST_LOG", "info")
            .output()
            .unwrap();
        let err = String::from_utf8_lossy(&out.stderr);
        let line = err
            .lines()
            .find(|l| l.contains("voice_hud: own is gone"))
            .unwrap_or_else(|| panic!("no mc line on stderr: {err}"));
        assert!(line.contains("WARN"), "{line}");
        assert!(line.contains("chora_mc::config"), "{line}");
    }

    /// A settings file of mc with `voice_hud: own` loads, and a save
    /// drops the setting.
    #[test]
    fn a_file_with_voice_hud_own_loads_and_a_save_drops_it() {
        use chora_kit::settings::{File, Store};
        let dir = chora_kit::testing::TempDir::new("chora-mc-own");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("mc.yaml");
        std::fs::write(&path, "theme: light\nvoice_hud: own\n").unwrap();
        let mut file = File::<Config>::load(path.clone(), DEFAULT).unwrap();
        assert!(file.broken().is_none());
        assert_eq!(file.value.voice_hud, VoiceHud::Host);
        assert_eq!(file.value.theme, Theme::Light);
        file.set(serde_json::json!({"theme": "dark"})).unwrap();
        file.save().unwrap();
        let saved = std::fs::read_to_string(&path).unwrap();
        assert!(!saved.contains("voice_hud"), "{saved}");
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// Super+a is the chat as in every look; the terminal has its own key.
    #[test]
    fn the_default_keys_of_the_chat_and_the_terminal() {
        let config = parse::<Config>(DEFAULT).unwrap();
        assert_eq!(
            config.common.bindings["Super+a"],
            Action::System(System::Chat)
        );
        assert_eq!(
            config.common.bindings["Super+Return"],
            Action::Look(Look::Scene("terminal".into()))
        );
    }

    /// Every look has the keys of the key list, the lock, the power menu,
    /// the screenshot and the volume.
    #[test]
    fn the_default_has_the_shared_keys_of_every_look() {
        use chora_kit::volume::Change;
        let config = parse::<Config>(DEFAULT).unwrap();
        let keys = &config.common.bindings;
        assert_eq!(keys["Super+slash"], Action::System(System::Keys));
        assert_eq!(keys["Super+Shift+Escape"], Action::System(System::Lock));
        assert_eq!(keys["Super+Shift+e"], Action::System(System::Power));
        assert_eq!(keys["Super+Print"], Action::System(System::Screenshot));
        assert_eq!(
            keys["XF86AudioRaiseVolume"],
            Action::System(System::Volume(Change::Up))
        );
        assert_eq!(
            keys["XF86AudioLowerVolume"],
            Action::System(System::Volume(Change::Down))
        );
        assert_eq!(
            keys["XF86AudioMute"],
            Action::System(System::Volume(Change::Mute))
        );
        assert!(!keys.contains_key("F1"));
    }

    /// The configs of head 29e2cd77 (the shipped default and the bindings
    /// that the docs list) load to the same table of bindings as before
    /// kit took the shared actions (KIT2): `tests/configs/29e2cd77`.
    #[test]
    fn the_configs_of_29e2cd77_load_to_the_same_table() {
        use std::fmt::Write;
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/configs/29e2cd77");
        for name in ["mc", "mc-docs-bindings"] {
            let text = std::fs::read_to_string(dir.join(format!("{name}.yaml"))).unwrap();
            let config = parse::<Config>(&text).unwrap();
            let mut table = String::new();
            for (key, action) in &config.common.bindings {
                writeln!(table, "{key}\t{}", serde_json::to_string(action).unwrap()).unwrap();
            }
            let golden = dir.join(format!("{name}.table"));
            if std::env::var_os("CHORA_BLESS").is_some() {
                std::fs::write(&golden, &table).unwrap();
            }
            assert_eq!(table, std::fs::read_to_string(&golden).unwrap(), "{name}");
        }
    }

    /// One YAML binding gives one `System` action in every look: the file
    /// `tests/configs/system-bindings.yaml` loads to its `.table` here and
    /// in construct .
    #[test]
    fn one_binding_gives_the_same_system_action_in_every_look() {
        use std::fmt::Write;
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/configs");
        let text = std::fs::read_to_string(dir.join("system-bindings.yaml")).unwrap();
        let common: Common<Look> = serde_saphyr::from_str(&text).unwrap();
        let mut table = String::new();
        for (key, action) in &common.bindings {
            writeln!(table, "{key}\t{}", serde_json::to_string(action).unwrap()).unwrap();
        }
        let golden = std::fs::read_to_string(dir.join("system-bindings.table")).unwrap();
        assert_eq!(table, golden);
    }

    /// A `settings.set` of a binding merges into the common settings that
    /// kit holds, is written as a map of the file, and reads back as the
    /// same action.
    #[test]
    fn a_binding_set_through_the_settings_is_saved_and_read_back() {
        use chora_kit::settings::{File, Store};
        let dir = chora_kit::testing::TempDir::new("chora-mc-common");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("mc.yaml");
        let mut file = File::<Config>::load(path.clone(), DEFAULT).unwrap();
        let change =
            serde_json::json!({"bindings": {"Super+x": {"run": "foot"}, "Super+Up": null}});
        file.set(change).unwrap();
        file.save().unwrap();
        let again = File::<Config>::load(path, DEFAULT).unwrap();
        let bindings = &again.value.common.bindings;
        assert_eq!(
            bindings["Super+x"],
            Action::System(System::Run("foot".into()))
        );
        assert!(!bindings.contains_key("Super+Up"));
        assert_eq!(bindings["Super+a"], Action::System(System::Chat));
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// A key that mc has no action for is refused with the binding named.
    #[test]
    fn a_system_action_without_meaning_in_mc_is_refused() {
        let text = DEFAULT.replace("Super+a: chat", "Super+a: terminal");
        let err = parse::<Config>(&text).unwrap_err();
        assert!(err.contains("Super+a") && err.contains("Terminal"), "{err}");
    }

    #[test]
    fn the_default_is_valid_and_mistakes_fail() {
        let config = parse::<Config>(DEFAULT).unwrap();
        assert_eq!(config.theme, Theme::Dark);
        assert_eq!(
            config.common.bindings["Super+Home"],
            Action::Look(Look::Scene("agenda".into()))
        );
        let give = DEFAULT.replace("Super+n:", "Super+g: {give: xp}\n  Super+n:");
        let given = parse::<Config>(&give).unwrap();
        assert_eq!(
            given.common.bindings["Super+g"],
            Action::System(System::Give("xp".into()))
        );
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
            assert!(command.contains("colors.regular5="), "{command}");
            assert!(!command.contains("colors-dark"), "{command}");
            assert!(command.contains("pixelsize=15"), "{command}");
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
        assert!((head(&config) - 14.25).abs() < 0.01, "{}", head(&config));
        config.scale.base = 27.0;
        assert!((head(&config) - 20.25).abs() < 0.01, "{}", head(&config));
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
