//! Regions around the page, after the Emacs package auto-side-windows: a
//! header and a footer line, a left and a right side, and a drawer at the
//! bottom. Each holds contents in slots: the context of the scene, the
//! clock, the state and the words of the agent, lines of text such as
//! notes, or a docked app. Rules decide which content goes to which region
//! and slot, by content, by kind and by scene; the first rule that matches
//! wins.

use std::collections::BTreeMap;

use chora_scene::{
    Command, Easing, NodeKind, PropsUpdate, ToplevelState, Transform, ipc::ClientError,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    Mc,
    config::{Style, Type},
    create,
    fonts::{self, Ink},
    move_to,
    scene::{MAX_TEXT, Scene, check_name},
    title_of,
};

/// The group of all regions, over the scenes. It holds the parked apps and
/// the regions of the last two renders, which cross-fade.
pub const REGIONS: &str = "mc-regions";
/// A hidden group for the docked apps whose region does not show, and for
/// a new docked app until its first size fits. Its empty animation times
/// the wait for that size.
pub const PARKED: &str = "mc-parked";
/// A hidden group of copies of the items, to measure their new texts.
const PROBE: &str = "mc-probe";

/// The id of the hidden copy of an item.
pub fn probe(id: &str) -> String {
    format!("{id}-probe")
}
/// How long a new docked app waits for a size that fits, in milliseconds.
pub const FIT_MS: f64 = 1000.0;
/// The most lines that `content.add` keeps.
pub const MAX_LINES: usize = 12;

/// A region around the page.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Region {
    Header,
    Footer,
    Left,
    Right,
    Drawer,
}

impl Region {
    pub const ALL: [Self; 5] = [
        Self::Header,
        Self::Footer,
        Self::Left,
        Self::Right,
        Self::Drawer,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Header => "header",
            Self::Footer => "footer",
            Self::Left => "left",
            Self::Right => "right",
            Self::Drawer => "drawer",
        }
    }

    /// A line across the page, its contents side by side.
    pub fn is_line(self) -> bool {
        matches!(self, Self::Header | Self::Footer)
    }
}

/// One region: whether it shows, and its size: lines of the baseline grid
/// for the header, the footer and the drawer, em of `scale.base` for the
/// sides.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Area {
    pub shown: bool,
    pub size: f64,
}

/// The five regions.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Regions {
    pub header: Area,
    pub footer: Area,
    pub left: Area,
    pub right: Area,
    pub drawer: Area,
}

impl Regions {
    pub fn get(&self, region: Region) -> Area {
        match region {
            Region::Header => self.header,
            Region::Footer => self.footer,
            Region::Left => self.left,
            Region::Right => self.right,
            Region::Drawer => self.drawer,
        }
    }
}

/// What a content shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// The context of the scene: the running head of its chapter, or its
    /// number and title; the name of an app.
    Head,
    /// The time, as `date` formats it, each minute anew.
    Clock,
    /// The state of the agent, from `hud.set` and `hud.say`.
    State,
    /// The words of the agent, from `hud.set` and `hud.say`.
    Hud,
    /// The sign and the name of each agent of `agent.set`, in the color
    /// of its state.
    Agents,
    /// Lines that `content.set` and `content.add` change, such as notes.
    Text,
    /// An app docked in a side or the drawer; it starts when its region
    /// shows the first time.
    App,
    /// The keys of the bindings, each a key and its words, pinned to the
    /// foot of a side; the key Super+F1 shows all of them.
    Keys,
    /// The chat of the HUD host in a side: a door that opens it while it
    /// is closed, and its box while it is open.
    Chat,
    /// The doors of the actions that have a key: the launcher, the lock,
    /// the power menu, the screenshot and the volume, in two lines at the
    /// foot of a side.
    Doors,
}

/// A content of the regions.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Content {
    pub kind: Kind,
    /// The title over the lines of a text in a side or the drawer.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub title: String,
    /// The lines of a text at the start.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<String>,
    /// The shell command of an app; `{NAME}` is the color NAME of the
    /// theme as RRGGBB.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub command: String,
    /// The format of a clock, as `date` takes it.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub format: String,
    /// A text shows `Nothing new.` while it has no lines, in the language
    /// of the session, under its title.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub empty: bool,
    /// A text in a side folds into its title and the count of its lines,
    /// with `Show` flush right, while the chat is open.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub folds: bool,
    /// The style of the text: `running_head` without a base for the head,
    /// the clock and the state, `hud` for the HUD, `margin` for a text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<Style>,
}

/// Where a content goes.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    /// The content by its name; without it, every content of `kind`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<Kind>,
    /// The scenes of the rule by name, `@app` for each scene of an app and
    /// `@page` for each scene of panels; all scenes without.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scenes: Vec<String>,
    pub region: Region,
    /// The place in the region, lowest first. In the header and the footer
    /// a slot below 0 goes left, 0 in the middle, above 0 right.
    #[serde(default)]
    pub slot: i32,
}

impl Rule {
    fn matches(&self, name: &str, content: &Content, scene: &str, app: bool) -> bool {
        let kind = if app { "@app" } else { "@page" };
        self.content.as_deref().is_none_or(|c| c == name)
            && self.kind.is_none_or(|k| k == content.kind)
            && (self.scenes.is_empty() || self.scenes.iter().any(|s| s == scene || s == kind))
    }
}

/// An arrangement of the regions: which show, and the rules; without
/// them, those of the settings.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Preset {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub shown: BTreeMap<Region, bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Rule>,
}

/// The contents of each region of a scene, by slot.
pub fn layout<'a>(
    rules: &[Rule],
    contents: &'a BTreeMap<String, Content>,
    scene: &str,
    app: bool,
) -> BTreeMap<Region, Vec<(i32, &'a str)>> {
    let mut out: BTreeMap<Region, Vec<(i32, &str)>> = BTreeMap::new();
    for (name, content) in contents {
        let rule = rules.iter().find(|r| r.matches(name, content, scene, app));
        // An app has no place in a line.
        if let Some(rule) = rule.filter(|r| !(r.region.is_line() && content.kind == Kind::App)) {
            out.entry(rule.region)
                .or_default()
                .push((rule.slot, name.as_str()));
        }
    }
    for slots in out.values_mut() {
        slots.sort_by_key(|(slot, _)| *slot);
    }
    out
}

/// Checks the regions, the contents and the rules of the settings.
pub fn check(
    regions: &Regions,
    contents: &BTreeMap<String, Content>,
    rules: &[Rule],
    (preset, presets): (&str, &BTreeMap<String, Preset>),
) -> Result<(), String> {
    if !presets.contains_key(preset) {
        return Err(format!("preset: no preset {preset}"));
    }
    for (name, preset) in presets {
        check_rules(contents, &preset.rules).map_err(|e| format!("presets.{name}: {e}"))?;
    }
    for region in Region::ALL {
        let size = regions.get(region).size;
        if !(1.0..=60.0).contains(&size) {
            return Err(format!("regions.{}: a size from 1 to 60", region.name()));
        }
    }
    for (name, content) in contents {
        check_name(name).map_err(|e| format!("contents.{name}: {e}"))?;
        let empty = match content.kind {
            Kind::App => content.command.trim().is_empty(),
            Kind::Clock => content.format.trim().is_empty(),
            _ => false,
        };
        if empty {
            return Err(format!(
                "contents.{name}: an app needs a command, a clock a format"
            ));
        }
        if content.title.len() + content.lines.join("\n").len() > MAX_TEXT {
            return Err(format!("contents.{name}: more than 4 KiB of text"));
        }
    }
    check_rules(contents, rules)
}

/// Checks rules against the contents.
fn check_rules(contents: &BTreeMap<String, Content>, rules: &[Rule]) -> Result<(), String> {
    for rule in rules {
        if rule.content.is_none() && rule.kind.is_none() {
            return Err("rules: each rule names a content or a kind".into());
        }
        if let Some(name) = &rule.content {
            let content = contents
                .get(name)
                .ok_or_else(|| format!("rules: no content {name}"))?;
            put_check(content, rule.region).map_err(|e| format!("rules: {name}: {e}"))?;
        }
    }
    Ok(())
}

/// Whether a content can go to a region.
pub fn put_check(content: &Content, region: Region) -> Result<(), String> {
    if content.kind == Kind::App && region.is_line() {
        return Err(format!("an app cannot go to the {}", region.name()));
    }
    Ok(())
}

/// A color as RRGGBB.
fn hex(rgba: [f64; 4]) -> String {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // 0 to 255
    let [r, g, b] = [0, 1, 2].map(|i| (rgba[i].clamp(0.0, 1.0) * 255.0).round() as u8);
    format!("{r:02x}{g:02x}{b:02x}")
}

/// The command of an app with the colors of the theme in it.
pub fn command(command: &str, colors: &BTreeMap<String, [f64; 4]>) -> String {
    colors.iter().fold(command.into(), |text, (name, rgba)| {
        text.replace(&format!("{{{name}}}"), &hex(*rgba))
    })
}

/// The time now as `date` formats it; empty when `date` fails.
pub fn date(format: &str) -> String {
    std::process::Command::new("date")
        .arg(format!("+{format}"))
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .unwrap_or_default()
}

/// The milliseconds to the next full minute, and a little more, never
/// above the longest duration of the core.
pub fn to_next_minute() -> f64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    next_minute_after(now)
}

/// `to_next_minute` at `now` milliseconds since the epoch.
fn next_minute_after(now: u128) -> f64 {
    #[allow(clippy::cast_precision_loss)] // below 60000
    let ms = (60_000 - now % 60_000) as f64;
    (ms + 50.0).min(chora_scene::MAX_DURATION)
}

/// How many doors stand in a line.
const DOORS_PER_LINE: usize = 3;
/// The space from one door of a line to the next, in pixels.
const DOOR_STEP: f64 = 100.0;

/// The content that names the next entry of the day in the footer.
const NEXT: &str = "next";

/// The content that names the previous page in the footer.
const PREV: &str = "prev";
const URGENT: &str = "urgent";

/// One text node of a region.
pub struct Item {
    pub id: String,
    pub slot: i32,
    /// The props, without the place.
    pub props: PropsUpdate,
    /// Where a stack in a side puts space before it: a new content.
    pub first: bool,
    /// The place across a side, from its left edge.
    pub x: f64,
    pub put: Put,
    /// Space before the item in a stack, in pixels.
    pub before: f64,
    /// What a click on it does: a binding as `key:Super+n`, a helper as
    /// `agent:NAME`; it also shows the focus mark under the pointer.
    pub hit: Option<String>,
    /// A close mark stands before the text: two bars, drawn, not a glyph.
    pub mark: bool,
}

/// Where an item of a side stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Put {
    /// One under the other from the top.
    Stack,
    /// At the foot of the side, above its lower edge.
    Foot,
    /// At the foot, with a hairline over the width of the side above it.
    FootRule,
    /// At the foot, in the line of the item before it.
    FootRow,
    /// At the foot, the first of a line whose items take the pointer by
    /// their own width.
    FootStart,
    /// In the line of the item before it, at `x`.
    Row,
    /// In the line of the item before it, flush right.
    End,
    /// Where the box of the chat begins: it runs to the foot of the side.
    Slot,
}

impl Put {
    fn foot(self) -> bool {
        matches!(
            self,
            Self::Foot | Self::FootRule | Self::FootRow | Self::FootStart
        )
    }

    fn row(self) -> bool {
        matches!(self, Self::FootRow | Self::Row | Self::End)
    }
}

impl Item {
    /// A text item that stacks in a side.
    pub fn text(id: String, slot: i32, props: PropsUpdate, first: bool) -> Self {
        Self {
            id,
            slot,
            props,
            first,
            x: 0.0,
            put: Put::Stack,
            before: 0.0,
            hit: None,
            mark: false,
        }
    }
}

/// The items and the docked app of each region.
pub type Plan = BTreeMap<Region, (Vec<Item>, Option<String>)>;

/// The boxes of the regions and of the window of an app scene, x, y,
/// width and height on the output.
struct Boxes {
    regions: BTreeMap<Region, [f64; 4]>,
    window: [f64; 4],
}

/// The group of the regions of a render: 0 or 1, in turns.
fn set_id(set: u8) -> String {
    format!("{REGIONS}-{set}")
}

/// The box of a docked app in a region of a width and a height: under the
/// text items, if there are any.
fn dock_box(width: f64, height: f64, items: bool, baseline: f64) -> [f64; 4] {
    let below = if items { baseline * 2.0 } else { 0.0 };
    [0.0, below, width, (height - below).max(1.0)]
}

/// What a region shows, to compare two renders: its box, its items
/// without their ids, and its docked app.
fn signature((items, dock): &(Vec<Item>, Option<String>), area: [f64; 4]) -> String {
    let items: Vec<_> = (items.iter())
        .map(|i| (i.slot, i.first, &i.props))
        .collect();
    format!("{area:?} {items:?} {dock:?}")
}

impl Mc {
    /// The id of the group of a region in the current set.
    pub fn region_id(&self, region: Region) -> String {
        format!("mc-r{}-{}", self.set, region.name())
    }

    /// The rules of the session first, then those of the preset or of the
    /// settings.
    fn rules(&self) -> Vec<Rule> {
        let config = &self.settings.value;
        let preset = config.presets.get(&config.preset);
        let settings = (preset.map(|p| &p.rules))
            .filter(|r| !r.is_empty())
            .unwrap_or(&config.rules);
        self.put.iter().chain(settings).cloned().collect()
    }

    /// Whether a scene is one of an app, or has no panels.
    pub fn is_app(&self, name: &str) -> bool {
        (self.scenes.get(name)).is_none_or(|s| s.app.is_some() || s.panels.is_empty())
    }

    /// The contents of each region of a scene, by slot; a solo content
    /// is alone in its region.
    pub fn placed(&self, name: &str) -> BTreeMap<Region, Vec<(i32, String)>> {
        let contents = &self.settings.value.contents;
        let mut placed: BTreeMap<Region, Vec<(i32, String)>> =
            layout(&self.rules(), contents, name, self.is_app(name))
                .into_iter()
                .map(|(r, v)| (r, v.into_iter().map(|(s, n)| (s, n.into())).collect()))
                .collect();
        if let Some(solo) = &self.solo {
            for slots in placed.values_mut() {
                slots.retain(|(_, c)| *c != solo.content);
            }
            placed.insert(solo.region, vec![(0, solo.content.clone())]);
            placed.retain(|_, slots| !slots.is_empty());
        }
        placed
    }

    /// Shows a content alone in a region, or hides it again: the region
    /// then shows as before. Answers whether it shows. Only for the user
    /// the keyboard leaves a docked app that the solo hides.
    pub fn solo(
        &mut self,
        content: &str,
        region: Region,
        shown: Option<bool>,
        user: bool,
    ) -> Result<bool, ClientError> {
        let on = (self.solo.as_ref()).is_some_and(|s| s.content == content && s.region == region);
        let new = shown.unwrap_or(!on);
        if let Some(old) = self.solo.take() {
            match old.before {
                Some(before) => self.toggled.insert(old.region, before),
                None => self.toggled.remove(&old.region),
            };
        }
        if new {
            self.solo = Some(crate::Solo {
                content: content.into(),
                region,
                before: self.toggled.get(&region).copied(),
            });
            self.toggled.insert(region, true);
            // A docked app of the region waits hidden: the keyboard goes
            // back to the scene.
            if user && (self.focused.as_ref()).is_some_and(|f| self.docks.values().any(|d| d == f))
            {
                let name = self.current.clone().unwrap_or_default();
                let window = self.windows.get(&name).cloned();
                self.focus(window)?;
            }
        }
        self.render()?;
        Ok(new)
    }

    /// The type of a content: its own style on the base of its kind.
    pub fn content_type(&self, content: &Content, title: bool) -> Type {
        let base = match content.kind {
            _ if title => "label",
            Kind::Head | Kind::Clock => "running_head",
            Kind::Hud | Kind::Agents => "hud",
            Kind::Chat => "body",
            Kind::State | Kind::Text | Kind::App | Kind::Keys | Kind::Doors => "margin",
        };
        let style = content.style.as_ref().filter(|_| !title);
        let mut ty = self.ty(base, style);
        // The state of the agent in the accent, as long as no style says
        // otherwise.
        if content.kind == Kind::State && style.is_none_or(|s| s.color.is_none()) {
            ty.color = (self.settings.value.colors())
                .get("accent")
                .copied()
                .unwrap_or(ty.color);
        }
        ty
    }

    /// The title and the text of a content now in a scene; an empty text
    /// hides it.
    fn content_text(&self, name: &str, content: &Content, scene: &str) -> (String, String) {
        match content.kind {
            Kind::Head => (String::new(), self.head(scene)),
            Kind::Clock => (
                String::new(),
                self.clocks.get(name).cloned().unwrap_or_default(),
            ),
            Kind::State => (String::new(), self.hud.state.clone()),
            Kind::Hud => (String::new(), self.hud.text.clone()),
            Kind::Agents => (String::new(), self.agent_line().join("  ")),
            Kind::Text => self
                .texts
                .get(name)
                .map(|(title, lines)| (title.clone(), lines.join("\n")))
                .unwrap_or_default(),
            Kind::App | Kind::Keys | Kind::Chat | Kind::Doors => (String::new(), String::new()),
        }
    }

    /// The running head of a scene: the one of its chapter, or its number
    /// and title.
    fn head(&self, name: &str) -> String {
        let Some(scene) = self.scenes.get(name) else {
            return String::new();
        };
        let chapter = &scene.chapter;
        chapter.running_head.clone().unwrap_or_else(|| {
            let title = chapter.title.clone().unwrap_or_else(|| title_of(name));
            format!("{}  {title}", self.number(name))
        })
    }

    /// Whether a region shows in a scene: as the toggle says, else as the
    /// scene says, else as the settings say; over a window without a
    /// scene the sides show only by a toggle, so that the window keeps its
    /// width. The window of an app scene fills its text block, so that the
    /// column shows beside it as beside a page.
    fn shown(&self, region: Region, name: &str) -> bool {
        let scene = self.scenes.get(name);
        let side = matches!(region, Region::Left | Region::Right);
        self.toggled
            .get(&region)
            .or_else(|| scene.and_then(|s: &Scene| s.regions.get(&region)))
            .copied()
            .unwrap_or_else(|| {
                let config = &self.settings.value;
                let preset = config.presets.get(&config.preset);
                let shown = (preset.and_then(|p| p.shown.get(&region)).copied())
                    .unwrap_or(config.regions.get(region).shown);
                let bare = scene.is_none_or(|s| !Self::has_chapter(s));
                shown && !(side && bare)
            })
    }

    /// The binding of an action, as the key list names it.
    fn key_of(&self, action: &crate::config::Action) -> Option<String> {
        (self.settings.value.bindings.iter())
            .find(|(_, a)| *a == action)
            .map(|(key, _)| format!("key:{key}"))
    }

    /// The items of the regions of a scene, and the docked app of each.
    pub fn items(&self, name: &str) -> Plan {
        let contents = &self.settings.value.contents;
        let mut out = Plan::new();
        for (region, slots) in self.placed(name) {
            let dock = (slots.iter().map(|(_, c)| c))
                .find(|c| contents[*c].kind == Kind::App && self.docks.contains_key(*c))
                .cloned();
            let items: Vec<Item> = (slots.iter())
                .flat_map(|(slot, c)| self.content_items(region, *slot, c, name))
                .collect();
            if !items.is_empty() || dock.is_some() {
                out.insert(region, (items, dock));
            }
        }
        // The sides show only where they fit.
        let [left, right] = self.side_widths(name);
        out.retain(|region, _| match region {
            Region::Left => left > 0.0,
            Region::Right => right > 0.0,
            _ => true,
        });
        out
    }

    /// The text items of a content in a region: one in a line, else its
    /// title and its lines.
    fn content_items(&self, region: Region, slot: i32, name: &str, scene: &str) -> Vec<Item> {
        let content = &self.settings.value.contents[name];
        let side = !region.is_line();
        let all_keys = side && self.keys == crate::keys::List::All;
        if all_keys && (content.kind == Kind::Chat || content.folds) {
            return Vec::new();
        }
        match content.kind {
            Kind::Agents => self.agent_items(region, slot, name, content),
            Kind::Chat if side => self.chat_items(slot, name, content, region),
            Kind::Keys if side => self.key_items(slot, name, content, region),
            Kind::Doors if side => self.door_items(slot, name, content, region),
            Kind::App | Kind::Chat | Kind::Keys | Kind::Doors => Vec::new(),
            _ => self.text_items(region, slot, (name, scene)),
        }
    }

    /// The text of a content that has none of its own: a free line for the
    /// state word of the host, the line to the next page in the footer.
    fn filled(
        &self,
        region: Region,
        (name, scene): (&str, &str),
        text: &str,
    ) -> (String, Option<String>) {
        let content = &self.settings.value.contents[name];
        let host = self.settings.value.voice_hud == crate::config::VoiceHud::Host;
        if !text.is_empty() {
            return (text.to_owned(), None);
        }
        if content.kind == Kind::State && !region.is_line() && host {
            return (" ".to_owned(), None);
        }
        if name == NEXT && region.is_line() {
            let line = self.next_page_line(scene).unwrap_or_default();
            return (line, self.key_of(&crate::config::Action::NextScene));
        }
        if name == PREV && region.is_line() {
            let line = self.previous_page_line(scene).unwrap_or_default();
            return (line, self.key_of(&crate::config::Action::PreviousScene));
        }
        (String::new(), None)
    }

    fn text_items(&self, region: Region, slot: i32, (name, scene): (&str, &str)) -> Vec<Item> {
        let content = &self.settings.value.contents[name];
        let (title, text) = self.content_text(name, content, scene);
        let (mut text, hit) = self.filled(region, (name, scene), &text);
        let id = format!("{}-{name}", self.region_id(region));
        if content.folds && self.chat == crate::Chat::Open && !region.is_line() {
            return self.folded_items(slot, &id, content, (&title, &text));
        }
        let quiet = content.empty && text.is_empty() && !region.is_line();
        if quiet {
            text = crate::keys::nothing_new(chora_kit::words::Language::of_session()).into();
        }
        if content.folds && !region.is_line() && !quiet && !text.is_empty() {
            return self.note_items(slot, &id, content, (&title, &text));
        }
        let item = |id: String, is_title: bool, text: &str, first: bool| {
            let mut props = self.content_type(content, is_title).props(text);
            if quiet && !is_title {
                props.color = self.settings.value.colors().get("muted").copied();
            }
            let urgent = name == URGENT && region.is_line();
            Item {
                hit: if urgent {
                    Some("urgent:clear".into())
                } else {
                    hit.clone()
                },
                mark: urgent,
                ..Item::text(id, slot, props, first)
            }
        };
        if region.is_line() {
            let text = text.replace('\n', "  ·  ");
            return (!text.is_empty())
                .then(|| item(id, false, &text, true))
                .into_iter()
                .collect();
        }
        let opened = self.toggled.get(&region) == Some(&true);
        let mut items = Vec::new();
        if !title.is_empty() && (opened || !text.is_empty()) {
            items.push(item(format!("{id}-title"), true, &title, true));
        }
        if !text.is_empty() {
            let first = items.is_empty();
            items.push(item(id, false, &text, first));
        }
        items
    }

    /// The widths of the left and the right side of a scene, 0 for a side
    /// without contents, and both 0 when the sides would leave the text
    /// less than three quarters of the measure of a column.
    pub fn side_widths(&self, name: &str) -> [f64; 2] {
        let Some(output) = &self.output else {
            return [0.0, 0.0];
        };
        let config = &self.settings.value;
        let (em, b) = (config.scale.base, config.scale.baseline);
        let page = &config.page;
        let gap = page.gap * em;
        let room = f64::from(output.size[0]) - 4.0 * b - 2.0 * (page.margin * em + gap);
        let placed = self.placed(name);
        let output_w = f64::from(output.size[0]);
        let sides = [Region::Left, Region::Right].map(|region| {
            if !placed.contains_key(&region) {
                return 0.0;
            }
            let size = config.regions.get(region).size;
            if region == Region::Right {
                column_width(size, b, output_w)
            } else {
                size * b
            }
        });
        let need: f64 = sides.iter().filter(|w| **w > 0.0).map(|w| w + gap).sum();
        // One column of text, since the columns of the grid wrap where
        // they do not fit; all of them hid the sides below 1900 pixels.
        if room - need < 0.75 * page.measure * em {
            [0.0, 0.0]
        } else {
            sides
        }
    }

    /// The boxes of the regions that show in a scene, and of its window.
    fn boxes(&self, name: &str, visible: &[Region]) -> Option<Boxes> {
        let output = self.output.as_ref()?;
        let [w, h] = output.size.map(f64::from);
        let config = &self.settings.value;
        let b = config.scale.baseline;
        let size = |region: Region| config.regions.get(region).size;
        let on = |region: Region| visible.contains(&region);
        let line = |region: Region| if on(region) { size(region) * b } else { 0.0 };
        let (head, foot) = (line(Region::Header), line(Region::Footer));
        let drawer = if on(Region::Drawer) {
            (size(Region::Drawer) * b).min(((h - head - foot) / 2.0).max(b))
        } else {
            0.0
        };
        // A scene without a head, such as of a window that no scene
        // started: the text block of an empty page, so that the lines
        // stay where they are between scenes.
        let headless = (self.scenes.get(name)).is_none_or(|s| !Self::has_chapter(s));
        let geo = if headless {
            self.geometry_of(name, &Scene::default())?
        } else {
            self.geometry(name)?
        };
        let [x, top, width, _] = geo.text;
        let (note, gap) = (geo.note, geo.gap);
        let mut regions = BTreeMap::new();
        // Over the text of a page, else over the whole app.
        let (lx, lw) = if headless {
            (2.0 * b, w - 4.0 * b)
        } else {
            (x, width)
        };
        regions.insert(Region::Header, [lx, 0.0, lw, head]);
        regions.insert(Region::Footer, [lx, h - foot, lw, foot]);
        let frame = [x - note - gap, width + 2.0 * (note + gap)];
        let drawer_top = h - foot - drawer;
        regions.insert(Region::Drawer, [frame[0], drawer_top, frame[1], drawer]);
        let [lw, rw] = self.side_widths(name);
        let side_top = if headless { head + b } else { top };
        let side_h = (drawer_top - b - side_top).max(b);
        let column_top = b;
        let column_end = (h / b).floor() * b;
        let column = [
            w - 2.0 * b - rw,
            column_top,
            rw,
            (column_end - column_top).max(b),
        ];
        let mut window = [0.0, head, w, (drawer_top - head).max(b)];
        if headless {
            // Each side has a box, also one that fades out.
            regions.insert(Region::Left, [b, side_top, lw, side_h]);
            regions.insert(Region::Right, column);
            if on(Region::Left) {
                window[0] = lw + 2.0 * b;
            }
            let right = if on(Region::Right) { rw + 4.0 * b } else { 0.0 };
            window[2] = (w - window[0] - right).max(b);
        } else {
            // The window of an app scene fills its text block, above the
            // drawer.
            let [_, _, _, height] = geo.text;
            window = [x, top, width, height.min(drawer_top - b - top).max(b)];
            let left = x - note - 2.0 * gap - lw;
            regions.insert(Region::Left, [left, side_top, lw, side_h]);
            regions.insert(Region::Right, column);
        }
        Some(Boxes { regions, window })
    }

    /// The regions that show in the current scene.
    fn visible(&self, items: &Plan) -> Vec<Region> {
        let name = self.current.clone().unwrap_or_default();
        self.visible_in(&name, items)
    }

    /// The boxes of the regions of a scene, as they show now.
    #[cfg(test)]
    pub fn boxes_of(&self, name: &str) -> BTreeMap<Region, [f64; 4]> {
        let visible = self.visible_in(name, &self.items(name));
        self.boxes(name, &visible)
            .map(|b| b.regions)
            .unwrap_or_default()
    }

    /// The box of the chat in the margin column, while it is open and the
    /// column shows.
    pub fn chat_slot(&self) -> Option<[f64; 4]> {
        self.slot.filter(|_| self.chat == crate::Chat::Open)
    }

    /// Tells the HUD host where the chat goes, when the box or the output
    /// changed since the host was told last. A failed emit is tried again
    /// at the next layout.
    pub fn send_slot(&mut self) -> Result<(), ClientError> {
        let rect = self.chat_slot();
        let Some(output) = self.output.as_ref().map(|o| o.name.clone()) else {
            return Ok(());
        };
        let told = self.slot_sent.as_ref();
        let unchanged = match told {
            Some((sent, sent_rect)) => *sent == output && *sent_rect == rect,
            None => rect.is_none(),
        };
        if unchanged {
            return Ok(());
        }
        let emit =
            serde_json::json!({"event": "mc.chat.slot", "data": {"output": output, "rect": rect}});
        self.rt
            .conn
            .call::<_, serde_json::Value>("events.emit", emit)?;
        self.slot_sent = Some((output, rect));
        Ok(())
    }

    /// The box of the drawer of the current scene, while it shows.
    pub fn drawer_box(&self) -> Option<[f64; 4]> {
        let name = self.current.clone()?;
        let items = self.items(&name);
        let visible = self.visible(&items);
        visible.contains(&Region::Drawer).then_some(())?;
        self.boxes(&name, &visible)?
            .regions
            .get(&Region::Drawer)
            .copied()
    }

    /// The regions with `items` that show in a scene.
    fn visible_in(&self, name: &str, items: &Plan) -> Vec<Region> {
        (items.keys().copied())
            .filter(|r| self.shown(*r, name))
            .collect()
    }

    /// The box of the window of an app scene: below its head, free of
    /// the regions that show in it.
    pub fn window_box(&self, name: &str) -> Option<[f64; 4]> {
        let visible = self.visible_in(name, &self.items(name));
        Some(self.boxes(name, &visible)?.window)
    }

    /// Draws the regions of the current scene anew.
    pub fn render(&mut self) -> Result<(), ClientError> {
        let commands = self.regions()?;
        self.apply(commands)?;
        self.send_slot()
    }

    /// The regions of the current scene: what goes into the transaction
    /// that shows them, so that a scene switch can take them along. The
    /// same regions with the same contents only change their text. Else
    /// the new set of regions waits invisible until the core has laid it
    /// out, and then cross-fades with the old one: a region that changed
    /// fades in over it, one that stayed the same shows at once.
    pub fn regions(&mut self) -> Result<Vec<Command>, ClientError> {
        let Some(name) = self.current.clone() else {
            return Ok(Vec::new());
        };
        if !self.built {
            return Ok(Vec::new());
        }
        let mut items = self.items(&name);
        let visible = self.visible(&items);
        let Some(boxes) = self.boxes(&name, &visible) else {
            return Ok(Vec::new());
        };
        items.retain(|r, _| visible.contains(r));
        // The ids without the set, which alternates with each new set.
        let set = format!("mc-r{}-", self.set);
        let key: Vec<String> = std::iter::once(name.clone())
            .chain(visible.iter().map(|r| r.name().into()))
            .chain(items.values().flat_map(|(i, d)| {
                (i.iter().map(|i| i.id.replacen(&set, "", 1))).chain(d.iter().cloned())
            }))
            .collect();
        let sigs: BTreeMap<Region, String> = (items.iter())
            .map(|(r, plan)| (*r, signature(plan, boxes.regions[r])))
            .collect();
        // The window of an app scene leaves the regions free; also when
        // the regions stay, since it may have come after them.
        let window = (self.windows.get(&name)).map(|s| place(s, boxes.window, self.output_size()));
        if self.rendered.as_ref() == Some(&key) {
            self.sigs = sigs;
            let mut commands = self.update(&items, &boxes)?;
            commands.extend(window.into_iter().flatten());
            return Ok(commands);
        }
        let old = self.rendered.is_some().then(|| set_id(self.set));
        let items = self.prepare(&name, &visible, &boxes, &sigs)?;
        let (mut commands, placed) = self.switch(&items, &boxes, &sigs)?;
        // A docked app whose region hides fades out with the old set.
        let left: Vec<String> = (self.docks.values())
            .filter(|s| !placed.contains(s))
            .cloned()
            .collect();
        if let Some(old) = &old {
            commands.push(Command::Animate {
                id: old.clone(),
                props: PropsUpdate {
                    opacity: Some(0.0),
                    visible: Some(false),
                    ..PropsUpdate::default()
                },
                duration: self.fade,
                easing: Easing::EaseInOut,
            });
        }
        self.left = old.is_some().then_some(left);
        self.sigs = sigs;
        self.rendered = Some(key);
        commands.extend(window.into_iter().flatten());
        Ok(commands)
    }

    /// The same items with new texts: measured on hidden copies first, so
    /// that their texts change and they move in one transaction, without
    /// a fade.
    fn update(&mut self, items: &Plan, boxes: &Boxes) -> Result<Vec<Command>, ClientError> {
        let hidden = PropsUpdate {
            opacity: Some(0.0),
            ..PropsUpdate::default()
        };
        let mut probes = vec![create(PROBE, NodeKind::Group, REGIONS, hidden)];
        let mut updates = Vec::new();
        for (region, (region_items, _)) in items {
            let width = boxes.regions[region][2];
            for item in region_items {
                let props = PropsUpdate {
                    max_width: (!region.is_line()).then_some(width),
                    ..item.props.clone()
                };
                probes.push(create(&probe(&item.id), NodeKind::Text, PROBE, props));
                updates.push(Command::Update {
                    id: item.id.clone(),
                    props: item.props.clone(),
                });
            }
        }
        self.apply(probes)?;
        let moves = self.arrange(items, boxes, true);
        // In a transaction of its own: the caller's may be refused.
        self.apply(vec![Command::Remove {
            id: PROBE.into(),
            to: None,
        }])?;
        updates.extend(moves?);
        Ok(updates)
    }

    /// Creates the new set of the regions, invisible, so that the core
    /// lays it out; the set before the last one goes, with the docked apps
    /// that faded out in it. Answers the items of the new set.
    fn prepare(
        &mut self,
        name: &str,
        visible: &[Region],
        boxes: &Boxes,
        sigs: &BTreeMap<Region, String>,
    ) -> Result<Plan, ClientError> {
        self.set ^= 1;
        let mut items = self.items(name);
        items.retain(|r, _| visible.contains(r));
        let new = set_id(self.set);
        let mut commands = Vec::new();
        if let Some(left) = &self.left {
            let docks: Vec<&String> = self.docks.values().collect();
            let parks = (left.iter()).filter(|s| docks.contains(s));
            commands.extend(parks.map(|s| park(s, PARKED)));
            commands.push(Command::Remove {
                id: new.clone(),
                to: None,
            });
        }
        let hidden = PropsUpdate {
            opacity: Some(0.0),
            ..PropsUpdate::default()
        };
        commands.push(create(&new, NodeKind::Group, REGIONS, hidden));
        for (region, plan) in &items {
            let same = self.sigs.get(region) == sigs.get(region);
            commands.extend(self.region(*region, plan, boxes, same));
        }
        // A refused transaction changed nothing: the last set stays.
        if let Err(err) = self.apply(commands) {
            self.set ^= 1;
            return Err(err);
        }
        self.left = None;
        Ok(items)
    }

    /// Shows the new set, laid out now: the items go to their places, the
    /// regions that changed fade in and the docked apps move in. Answers
    /// the commands and the docked apps that it placed.
    fn switch(
        &mut self,
        items: &Plan,
        boxes: &Boxes,
        sigs: &BTreeMap<Region, String>,
    ) -> Result<(Vec<Command>, Vec<String>), ClientError> {
        let mut commands = self.arrange(items, boxes, false)?;
        commands.push(Command::Update {
            id: set_id(self.set),
            props: PropsUpdate {
                opacity: Some(1.0),
                ..PropsUpdate::default()
            },
        });
        let baseline = self.settings.value.scale.baseline;
        let mut placed = Vec::new();
        for (region, (region_items, dock)) in items {
            let id = self.region_id(*region);
            if self.sigs.get(region) != sigs.get(region) {
                commands.push(Command::Animate {
                    id: id.clone(),
                    props: PropsUpdate {
                        opacity: Some(1.0),
                        ..PropsUpdate::default()
                    },
                    duration: self.fade,
                    easing: Easing::EaseInOut,
                });
            }
            if let Some(surface) = dock.as_ref().and_then(|d| self.docks.get(d)) {
                let [_, _, width, height] = boxes.regions[region];
                let window = dock_box(width, height, !region_items.is_empty(), baseline);
                commands.push(park(surface, &id));
                commands.extend(place(surface, window, [0, 0]));
                placed.push(surface.clone());
            }
        }
        Ok((commands, placed))
    }

    /// The size of a docked app that starts, in its region of the current
    /// scene, which then shows.
    pub fn dock_size(&self, content: &str) -> Option<[i32; 2]> {
        let name = self.current.clone()?;
        let placed = self.placed(&name);
        let (region, _) = (placed.iter()).find(|(_, s)| s.iter().any(|(_, c)| c == content))?;
        let items = self.items(&name);
        let mut visible = self.visible(&items);
        visible.push(*region);
        let boxes = self.boxes(&name, &visible)?;
        let [_, _, width, height] = boxes.regions[region];
        let text = items.get(region).is_some_and(|(i, _)| !i.is_empty());
        let [_, _, w, h] = dock_box(width, height, text, self.settings.value.scale.baseline);
        #[allow(clippy::cast_possible_truncation)] // pixels of a screen
        Some([w.round() as i32, h.round() as i32])
    }

    fn output_size(&self) -> [i32; 2] {
        self.output.as_ref().map_or([0, 0], |o| o.size)
    }

    /// Creates a region with its items, in the new set: at once when it
    /// shows the same as in the old set, else transparent until it fades
    /// in. Its docked app comes with the switch.
    fn region(
        &self,
        region: Region,
        (items, _): &(Vec<Item>, Option<String>),
        boxes: &Boxes,
        same: bool,
    ) -> Vec<Command> {
        let [left, top, width, height] = boxes.regions[&region];
        let id = self.region_id(region);
        let mut commands = vec![create(
            &id,
            NodeKind::Group,
            &set_id(self.set),
            PropsUpdate {
                transform: Some(crate::translate(left, top)),
                opacity: Some(if same { 1.0 } else { 0.0 }),
                ..PropsUpdate::default()
            },
        )];
        let config = &self.settings.value;
        let colors = config.colors();
        if region == Region::Right {
            let baseline = config.scale.baseline;
            let faint = colors.get("faint").copied().unwrap_or(config.background());
            let caps = self.ty("margin", None).ink().height;
            let first = self.phase() - caps;
            let last = height - (baseline - self.phase());
            let edge = [-baseline, first, 1.0, (last - first).max(1.0)];
            commands.push(rect(&format!("{id}-edge"), &id, edge, faint));
        }
        if region == Region::Drawer {
            // A sheet of the page over the text under it, and a hairline.
            let baseline = config.scale.baseline;
            let screen = f64::from(self.output_size()[0]);
            commands.push(rect(
                &format!("{id}-sheet"),
                &id,
                [-left, -baseline, screen, height + baseline],
                config.background(),
            ));
            let faint = colors.get("faint").copied().unwrap_or(config.background());
            let rule = [0.0, -(baseline / 2.0).round(), width, 1.0];
            commands.push(rect(&format!("{id}-rule"), &id, rule, faint));
        }
        let faint = colors.get("faint").copied().unwrap_or(config.background());
        for item in items {
            let room = (width - item.x).max(1.0);
            let props = PropsUpdate {
                max_width: (!region.is_line()).then_some(room),
                ..item.props.clone()
            };
            commands.push(create(&item.id, NodeKind::Text, &id, props));
            commands.extend(close_nodes(item, &id, faint));
            if item.hit.is_some() {
                let ink = colors.get("text").copied().unwrap_or(faint);
                commands.extend(hit_nodes(&item.id, &id, ink));
            }
            if item.put == Put::FootRule {
                commands.push(rect(
                    &format!("{}-rule", item.id),
                    &id,
                    [0.0, 0.0, width, 1.0],
                    faint,
                ));
            }
        }
        commands
    }

    /// Moves the items of the regions to their places, as the core lays
    /// them out: side by side in a line, else one under the other.
    fn arrange(
        &mut self,
        items: &Plan,
        boxes: &Boxes,
        probed: bool,
    ) -> Result<Vec<Command>, ClientError> {
        let mut commands = Vec::new();
        self.slot = None;
        for (region, (items, _)) in items {
            let [left, top, width, height] = boxes.regions[region];
            if region.is_line() {
                commands.extend(self.line(items, [top, width, height], probed)?);
                continue;
            }
            commands.extend(self.stack(items, [left, top, width, height], probed)?);
        }
        Ok(commands)
    }

    /// Places the items of a side: one under the other from the top, the
    /// items of the foot above the lower edge, a row beside its first item.
    fn stack(
        &mut self,
        items: &[Item],
        [left, top, width, height]: [f64; 4],
        probed: bool,
    ) -> Result<Vec<Command>, ClientError> {
        let baseline = self.settings.value.scale.baseline;
        let (mut heights, mut widths, mut lefts) = (Vec::new(), Vec::new(), Vec::new());
        for item in items {
            let [_, _, w, h] = self.measure(&item.id, probed)?;
            heights.push(h.max(baseline));
            widths.push(w);
            let at = match item.put {
                Put::End => (width - w + trailing(&item.props)).max(0.0),
                _ => item.x,
            };
            lefts.push(at);
        }
        let (flow, end) = lines(items, &heights, false, baseline);
        let (foot, foot_end) = lines(items, &heights, true, baseline);
        let foot_top = (height - foot_end).max(end);
        let mut commands = Vec::new();
        if let Some(at) = items.iter().position(|i| i.put == Put::Slot)
            && let Some((_, y)) = flow.iter().find(|(i, _)| *i == at)
        {
            self.slot = Some([left, top + y, width, (height - y).max(baseline)]);
        }
        for (item, y) in flow
            .into_iter()
            .chain(foot.into_iter().map(|(i, y)| (i, y + foot_top)))
        {
            let idx = item;
            let item = &items[idx];
            let ink = props_ink(&item.props);
            let shift = self.to_grid(top + y + ink.top + ink.height);
            commands.push(move_to(item.id.clone(), lefts[idx], y + shift));
            if item.hit.is_some() {
                let own = matches!(
                    item.put,
                    Put::End | Put::Row | Put::FootRow | Put::FootStart
                );
                let [x, w] = if own {
                    [lefts[idx], widths[idx]]
                } else {
                    [0.0, width]
                };
                let at = if item.put == Put::End { x } else { item.x };
                let ink = Ink {
                    top: ink.top + shift,
                    ..ink
                };
                commands.extend(hit_place(&item.id, [x, y, w, heights[idx]], at, ink));
            }
            if item.put == Put::FootRule {
                let y = y - (baseline / 2.0).round();
                commands.push(move_to(format!("{}-rule", item.id), 0.0, y));
                commands.push(Command::Update {
                    id: format!("{}-rule", item.id),
                    props: PropsUpdate {
                        size: Some([width, 1.0]),
                        ..PropsUpdate::default()
                    },
                });
            }
        }
        Ok(commands)
    }

    /// The chat in a side: while it is closed a door, `Chat with a helper`
    /// and the key of the chat flush right, under a head; while it is open
    /// the head with `Esc  Close`, and the place of the box of the host.
    fn chat_items(&self, slot: i32, name: &str, content: &Content, region: Region) -> Vec<Item> {
        let id = format!("{}-{name}", self.region_id(region));
        let head = self.content_type(content, true).props("Chat");
        let mut items = vec![Item::text(format!("{id}-title"), slot, head, true)];
        let margin = self.ty("margin", None);
        let open = self.chat == crate::Chat::Open;
        let end = |part: &str, text: &str, hit: Option<&str>| Item {
            put: Put::End,
            hit: hit.map(str::to_owned),
            ..Item::text(format!("{id}-{part}"), slot, margin.props(text), false)
        };
        if open {
            items.push(Item {
                put: Put::End,
                ..end("close", "Esc  Close", Some("chat:close"))
            });
            let blank = Item::text(format!("{id}-slot"), slot, margin.props(" "), false);
            items.push(Item {
                put: Put::Slot,
                ..blank
            });
            return items;
        }
        let door = self
            .content_type(content, false)
            .props("Chat with a helper ...");
        items.push(Item {
            hit: Some("chat:open".into()),
            ..Item::text(format!("{id}-door"), slot, door, false)
        });
        let key = (self.settings.value.bindings.iter())
            .find(|(_, a)| **a == crate::config::Action::Chat)
            .map(|(k, _)| crate::keys::human(k));
        items.extend(key.map(|key| end("key", &key, None)));
        items
    }

    /// The notes of a list that folds: each on one line, the time in the
    /// margin style and the words in the reading face beside it, a longer
    /// note wrapping under its first word.
    fn note_items(
        &self,
        slot: i32,
        id: &str,
        content: &Content,
        (title, text): (&str, &str),
    ) -> Vec<Item> {
        let mut items = Vec::new();
        if !title.is_empty() {
            let props = self.content_type(content, true).props(title);
            items.push(Item::text(format!("{id}-title"), slot, props, true));
        }
        let baseline = self.settings.value.scale.baseline;
        let note = self.ty("margin", None);
        let body = self.content_type(content, false);
        let x = chora_kit::hud::LABEL_COLUMN;
        let lines = text.lines().filter(|line| !line.trim().is_empty());
        for (i, line) in lines.enumerate() {
            let Some((time, words)) = split_time(line) else {
                let props = body.props(line);
                items.push(Item::text(format!("{id}-{i}"), slot, props, false));
                continue;
            };
            items.push(Item {
                before: if i > 0 { baseline } else { 0.0 },
                ..Item::text(format!("{id}-{i}-time"), slot, note.props(time), false)
            });
            items.push(Item {
                put: Put::Row,
                x,
                ..Item::text(format!("{id}-{i}"), slot, body.props(words), false)
            });
        }
        items
    }

    /// A text that folds while the chat is open: its head with the count
    /// of its lines, and `Show` flush right.
    fn folded_items(
        &self,
        slot: i32,
        id: &str,
        content: &Content,
        (title, text): (&str, &str),
    ) -> Vec<Item> {
        let count = text.lines().filter(|l| !l.trim().is_empty()).count();
        let head = format!("{title}  {count}");
        let props = self.content_type(content, true).props(&head);
        let mut items = vec![Item::text(format!("{id}-title"), slot, props, true)];
        if count > 0 {
            let props = self.ty("margin", None).props("Show");
            items.push(Item {
                put: Put::End,
                hit: Some("chat:close".into()),
                ..Item::text(format!("{id}-show"), slot, props, false)
            });
        }
        items
    }

    /// The rows of the key list: the key in the text color, its words in
    /// the margin color beside it, and a head over them.
    fn key_items(&self, slot: i32, name: &str, content: &Content, region: Region) -> Vec<Item> {
        if self.chat == crate::Chat::Open {
            return Vec::new();
        }
        let config = &self.settings.value;
        let full = self.keys == crate::keys::List::All;
        let rows = if full {
            crate::keys::all(config)
        } else {
            crate::keys::short(config)
        };
        let id = format!("{}-{name}", self.region_id(region));
        let title = if full { "All keys" } else { "Keys" };
        let head = self.content_type(content, true).props(title);
        let mut items = vec![Item {
            put: Put::FootRule,
            ..Item::text(format!("{id}-title"), slot, head, true)
        }];
        let mut ty = self.content_type(content, false);
        let words_x = chora_kit::hud::LABEL_COLUMN;
        let words = ty.clone();
        ty.color = config.colors().get("text").copied().unwrap_or(ty.color);
        for (i, row) in rows.iter().enumerate() {
            let mut props = ty.props(&row.key);
            props.weight = Some(500);
            items.push(Item {
                put: Put::Foot,
                hit: row.binding.as_ref().map(|b| format!("key:{b}")),
                ..Item::text(format!("{id}-{i}-key"), slot, props, false)
            });
            items.push(Item {
                put: Put::FootRow,
                x: words_x,
                ..Item::text(
                    format!("{id}-{i}-words"),
                    slot,
                    words.props(&row.words),
                    false,
                )
            });
        }
        items
    }

    /// The doors at the foot of a side: a word for each action that has a
    /// key, three in a line, clicked like its key.
    fn door_items(&self, slot: i32, name: &str, content: &Content, region: Region) -> Vec<Item> {
        let id = format!("{}-{name}", self.region_id(region));
        let mut ty = self.content_type(content, false);
        ty.color = self
            .settings
            .value
            .colors()
            .get("text")
            .copied()
            .unwrap_or(ty.color);
        let volume = self.volume_words();
        let words = [
            ("programs", "Programs"),
            ("lock", "Lock"),
            ("power", "Power"),
            ("screenshot", "Screenshot"),
            ("volume", volume.as_str()),
        ];
        let mut items = Vec::new();
        for (i, (door, text)) in words.into_iter().enumerate() {
            let column = i % DOORS_PER_LINE;
            #[allow(clippy::cast_precision_loss, reason = "below 3")]
            let x = column as f64 * DOOR_STEP;
            let put = if column == 0 {
                Put::FootStart
            } else {
                Put::FootRow
            };
            items.push(Item {
                put,
                x,
                hit: Some(format!("door:{door}")),
                ..Item::text(format!("{id}-{door}"), slot, ty.props(text), i == 0)
            });
        }
        items
    }

    /// The words of the volume door: the level, or why there is none.
    fn volume_words(&self) -> String {
        let language = chora_kit::words::Language::of_session();
        let title = chora_kit::words::Word::Volume.text(language);
        match self.volume_reading {
            Some(reading @ chora_kit::volume::Reading::Level(volume)) if !volume.muted => {
                format!("{title} {}", reading.words(language))
            }
            Some(reading) => reading.words(language),
            None => title.to_owned(),
        }
    }

    /// What a click on a door does: the action of its key.
    fn door(&mut self, what: &str) {
        use crate::config::Action;
        let action = match what {
            "programs" => Action::Run("fuzzel".into()),
            "lock" => Action::Lock,
            "power" => Action::Power,
            "screenshot" => Action::Screenshot,
            "volume" => Action::Volume(chora_kit::volume::Change::Mute),
            _ => return,
        };
        self.perform(action);
    }

    /// Places the items of a line: the end keeps its place, the start ends
    /// before it, and the middle sits between them; what does not fit is
    /// cut.
    #[allow(
        clippy::too_many_lines,
        reason = "one pass over the three groups of a line"
    )]
    fn line(
        &mut self,
        items: &[Item],
        [top, width, height]: [f64; 3],
        probed: bool,
    ) -> Result<Vec<Command>, ClientError> {
        let config = &self.settings.value;
        let (baseline, gap) = (config.scale.baseline, config.page.gap * config.scale.base);
        let mut sizes = Vec::new();
        for item in items {
            let [_, _, w, h] = self.measure(&item.id, probed)?;
            sizes.push((item, w + lead(item), h));
        }
        let group = |pick: fn(i32) -> bool| -> Vec<(&Item, f64, f64)> {
            (sizes.iter())
                .filter(|(i, _, _)| pick(i.slot))
                .copied()
                .collect()
        };
        let span = |g: &[(&Item, f64, f64)]| -> f64 {
            #[allow(clippy::cast_precision_loss)] // a few items
            let gaps = g.len().saturating_sub(1) as f64 * gap;
            g.iter().map(|(_, w, _)| w).sum::<f64>() + gaps
        };
        let (start, middle, end) = (group(|s| s < 0), group(|s| s == 0), group(|s| s > 0));
        let end_x = (width - span(&end)).max(0.0);
        let high = if end.is_empty() { end_x } else { end_x - gap };
        let low = if start.is_empty() {
            0.0
        } else {
            span(&start).min(high) + gap
        };
        let mid_x = ((width - span(&middle)) / 2.0)
            .round()
            .min(high - span(&middle))
            .max(low);
        let mut commands = Vec::new();
        for (group, x0, room, flush) in [
            (start, 0.0, high, false),
            (middle, mid_x, high - mid_x, false),
            (end, end_x, width - end_x, true),
        ] {
            let mut next = x0;
            let last = group.len().saturating_sub(1);
            for (i, (item, w, h)) in group.into_iter().enumerate() {
                let left = x0 + room - next;
                let lead = lead(item);
                let x = if flush && i == last {
                    next + trailing(&item.props)
                } else {
                    next
                };
                if w > left + 0.5 {
                    commands.push(shorten(item, left / w));
                }
                let y = ((height - h.max(baseline)) / 2.0).round();
                let ink = props_ink(&item.props);
                let y = y + self.to_grid(top + y + ink.top + ink.height);
                commands.push(move_to(item.id.clone(), x + lead, y));
                commands.extend(close_place(item, [x, y + ink.top + ink.height / 2.0]));
                if item.hit.is_some() {
                    let at = [x, y, w.min(left), h.max(baseline)];
                    commands.extend(hit_place(&item.id, at, x, props_ink(&item.props)));
                }
                next += w.min(left) + gap;
            }
        }
        Ok(commands)
    }

    /// Shows or hides a region, by key or by method. Without `shown`, a
    /// region whose docked app has no focus takes it first; a second
    /// press hides it. Answers whether it shows. Only a toggle of the
    /// user moves the keyboard.
    pub fn toggle(
        &mut self,
        region: Region,
        shown: Option<bool>,
        user: bool,
    ) -> Result<bool, ClientError> {
        let name = self.current.clone().unwrap_or_default();
        let items = self.items(&name);
        let visible = self.visible(&items).contains(&region);
        let app = self.placed(&name).get(&region).and_then(|slots| {
            let contents = &self.settings.value.contents;
            (slots.iter())
                .find(|(_, c)| contents[c].kind == Kind::App)
                .map(|(_, c)| c.clone())
        });
        let surface = app.as_ref().and_then(|a| self.docks.get(a)).cloned();
        if user
            && shown.is_none()
            && visible
            && let Some(surface) = &surface
            && self.focused.as_ref() != Some(surface)
        {
            self.focus(Some(surface.clone()))?;
            return Ok(true);
        }
        let new = shown.unwrap_or(!visible);
        self.toggled.insert(region, new);
        if new && let Some(app) = &app {
            match &surface {
                Some(surface) if user => self.focus(Some(surface.clone()))?,
                Some(_) => {}
                None => {
                    self.start_dock(app);
                    if user {
                        self.dock_focus = Some(app.clone());
                    }
                }
            }
        }
        if user && !new && surface.is_some() && self.focused == surface {
            let window = self.windows.get(&name).cloned();
            self.focus(window)?;
        }
        self.render()?;
        Ok(new)
    }

    /// Shows the focus mark of a hit while the pointer is over it, and
    /// hides it when the pointer leaves.
    pub fn mark(&mut self, node: &str, on: bool) -> Result<(), ClientError> {
        let Some(base) = node
            .strip_suffix("-hit")
            .filter(|_| node.starts_with("mc-"))
        else {
            return Ok(());
        };
        self.apply(vec![Command::Update {
            id: mark_id(base),
            props: PropsUpdate {
                opacity: Some(if on { 1.0 } else { 0.0 }),
                ..PropsUpdate::default()
            },
        }])
    }

    /// What a click on a hit does: a key row acts as its key, a helper
    /// opens the chat with it, a row of the overview shows its scene, a
    /// row of another page is announced as `mc.row.chosen`. Answers
    /// whether the node is a hit.
    pub fn click_hit(&mut self, node: &str) -> bool {
        let Some(base) = node.strip_suffix("-hit") else {
            return false;
        };
        if base.ends_with("-chapter-head") {
            return self.show_overview();
        }
        let name = self.current.clone().unwrap_or_default();
        let token = (self.items(&name).into_values())
            .flat_map(|(items, _)| items)
            .find(|i| i.id == base)
            .and_then(|i| i.hit);
        match token.as_deref().and_then(|t| t.split_once(':')) {
            Some(("key", binding)) => self.act(binding),
            Some(("agent", agent)) => self.chat_with(Some(agent)),
            Some(("chat", "open")) => self.chat_with(None),
            Some(("chat", "close")) => self.chat_close(),
            Some(("door", what)) => self.door(what),
            Some(("urgent", "clear")) => {
                if let Err(err) = self.clear_notifications() {
                    eprintln!("chora-mc: urgent: {}", err.message);
                }
            }
            _ => return self.click_row(base),
        }
        true
    }

    /// Shows the page list, as a click on the head of a page does.
    fn show_overview(&mut self) -> bool {
        if let Err(err) = self.show(crate::manage::OVERVIEW, true) {
            eprintln!("chora-mc: {}: {}", crate::manage::OVERVIEW, err.message);
        }
        true
    }

    /// A click on a row of a page: `mc-s-SCENE-panels-PANEL-row-N`.
    fn click_row(&mut self, base: &str) -> bool {
        let Some((scene, rest)) = base
            .strip_prefix("mc-s-")
            .and_then(|b| b.split_once("-panels-"))
        else {
            return false;
        };
        let Some((panel, index)) = rest.rsplit_once("-row-") else {
            return false;
        };
        let Ok(index) = index.parse::<usize>() else {
            return false;
        };
        if scene == crate::manage::OVERVIEW {
            if let Some(target) = self.order().get(index).cloned()
                && let Err(err) = self.show(&target, true)
            {
                eprintln!("chora-mc: {target}: {}", err.message);
            }
            return true;
        }
        let emit = serde_json::json!({"event": "mc.row.chosen",
            "data": {"scene": scene, "panel": panel, "index": index}});
        if let Err(err) = self
            .rt
            .conn
            .call::<_, serde_json::Value>("events.emit", emit)
        {
            eprintln!("chora-mc: mc.row.chosen: {err}");
        }
        true
    }

    /// Starts the docked apps of the regions that show in the current
    /// scene; a build calls it, so a failing command does not repeat.
    pub fn autostart(&mut self) {
        let name = self.current.clone().unwrap_or_default();
        let contents = &self.settings.value.contents;
        let apps: Vec<String> = (self.placed(&name).into_iter())
            .filter(|(region, _)| self.shown(*region, &name))
            .flat_map(|(_, slots)| slots.into_iter().map(|(_, c)| c))
            .filter(|c| contents[c].kind == Kind::App && !self.docks.contains_key(c))
            .collect();
        for app in apps {
            self.start_dock(&app);
        }
    }

    /// Starts the app of a content, once.
    fn start_dock(&mut self, app: &str) {
        let waiting = (self.starting.values()).any(|t| *t == crate::Target::Dock(app.into()));
        if waiting {
            return;
        }
        let config = &self.settings.value;
        let command = command(&config.contents[app].command, config.colors());
        let mint = self.rt.conn.activation_token();
        if let Some((token, _)) = crate::launch(&command, mint.as_deref()) {
            self.starting.insert(token, crate::Target::Dock(app.into()));
        }
    }

    /// Gives the keyboard to a surface, or to none.
    pub fn focus(&mut self, surface: Option<String>) -> Result<(), ClientError> {
        self.focused.clone_from(&surface);
        self.apply(vec![Command::Focus { surface }])
    }

    /// The clocks anew, and a tick at the next minute.
    pub fn tick_clocks(&mut self) -> Vec<Command> {
        let contents = &self.settings.value.contents;
        self.clocks = (contents.iter())
            .filter(|(_, c)| c.kind == Kind::Clock)
            .map(|(name, c)| (name.clone(), date(&c.format)))
            .collect();
        vec![Command::Animate {
            id: crate::CLOCK.into(),
            props: PropsUpdate::default(),
            duration: to_next_minute(),
            easing: Easing::Linear,
        }]
    }

    /// What `region.list` answers.
    pub fn list(&self) -> serde_json::Value {
        let name = self.current.clone().unwrap_or_default();
        let placed = self.placed(&name);
        let items = self.items(&name);
        let visible = self.visible(&items);
        let regions: serde_json::Map<String, serde_json::Value> = Region::ALL
            .iter()
            .map(|r| {
                let contents: Vec<&String> = placed
                    .get(r)
                    .into_iter()
                    .flatten()
                    .map(|(_, c)| c)
                    .collect();
                let value = serde_json::json!({
                    "shown": self.shown(*r, &name),
                    "visible": visible.contains(r),
                    "contents": contents,
                });
                (r.name().to_owned(), value)
            })
            .collect();
        let contents: serde_json::Map<String, serde_json::Value> = (self.settings.value.contents)
            .iter()
            .map(|(n, c)| {
                let (title, text) = self.content_text(n, c, &name);
                let value = serde_json::json!({
                    "kind": c.kind,
                    "title": title,
                    "text": text,
                    "running": self.docks.contains_key(n),
                });
                (n.clone(), value)
            })
            .collect();
        serde_json::json!({"scene": name, "regions": regions, "contents": contents})
    }
}

/// The width of the margin column on an output of `output` pixels: 26 per
/// cent of it in whole lines of the grid, from `lines` lines up to a
/// third more.
pub fn column_width(lines: f64, baseline: f64, output: f64) -> f64 {
    let most = (lines * 4.0 / 3.0).round() * baseline;
    ((0.26 * output / baseline).round() * baseline).clamp(lines * baseline, most)
}

/// The id of the focus mark of a hit.
pub fn mark_id(base: &str) -> String {
    format!("{base}-mark")
}

/// The transparent rect that takes the pointer over something that a
/// click acts on, and the focus mark that shows beside it under the
/// pointer: a bar of 2 pixels in the color `ink`, as high as the capitals that it
/// marks, which `hit_place` sets.
pub fn hit_nodes(base: &str, parent: &str, ink: [f64; 4]) -> [Command; 2] {
    let mark = PropsUpdate {
        input: Some(chora_scene::InputMode::Pass),
        opacity: Some(0.0),
        ..PropsUpdate::default()
    };
    [
        rect(
            &format!("{base}-hit"),
            parent,
            [0.0, 0.0, 1.0, 1.0],
            [0.0; 4],
        ),
        Command::Create {
            id: mark_id(base),
            kind: NodeKind::Rect,
            parent: parent.into(),
            props: PropsUpdate {
                size: Some([MARK_WIDTH, MARK_WIDTH]),
                color: Some(ink),
                ..mark
            },
        },
    ]
}

/// Sizes the hit of an item to its box, and puts its mark 12 pixels left
/// of the item at `item_x`, as high as the capitals of its text, which
/// start `top` pixels below the box.
pub fn hit_place(base: &str, [x, y, w, h]: [f64; 4], item_x: f64, ink: Ink) -> [Command; 4] {
    [
        move_to(format!("{base}-hit"), x, y),
        Command::Update {
            id: format!("{base}-hit"),
            props: PropsUpdate {
                size: Some([w.max(1.0), h.max(1.0)]),
                ..PropsUpdate::default()
            },
        },
        move_to(mark_id(base), item_x - MARK_GAP, y + ink.top),
        Command::Update {
            id: mark_id(base),
            props: PropsUpdate {
                size: Some([MARK_WIDTH, ink.height.max(1.0)]),
                ..PropsUpdate::default()
            },
        },
    ]
}

/// The side of the close mark of an urgent line and its hit; the bars are
/// as thick as the hairline of the hover mark.
pub const CLOSE: f64 = 14.0;
/// The space between the close mark and the words.
const CLOSE_GAP: f64 = 8.0;

/// The room that an item keeps before its text for its close mark.
fn lead(item: &Item) -> f64 {
    if item.mark { CLOSE + CLOSE_GAP } else { 0.0 }
}

/// The id of a bar of the close mark of an item: `n` is 0 or 1.
pub fn close_id(base: &str, n: usize) -> String {
    format!("{base}-close-{n}")
}

/// The two bars of the close mark of an item, if it has one, in the color
/// of its text; `close_place` crosses them at 45 degrees.
fn close_nodes(item: &Item, parent: &str, fallback: [f64; 4]) -> Vec<Command> {
    let color = item.props.color.unwrap_or(fallback);
    (0..2)
        .filter(|_| item.mark)
        .map(|n| {
            let mut bar = rect(
                &close_id(&item.id, n),
                parent,
                [0.0, 0.0, CLOSE, 2.0],
                color,
            );
            if let Command::Create { props, .. } = &mut bar {
                props.input = Some(chora_scene::InputMode::Pass);
            }
            bar
        })
        .collect()
}

/// Puts the bars on the cross of a mark whose left edge is `x` and whose
/// centre stands at `y`.
fn close_place(item: &Item, [x, y]: [f64; 2]) -> Vec<Command> {
    let centre = [x + CLOSE / 2.0, y];
    (0..2)
        .filter(|_| item.mark)
        .map(|n| {
            let turn = if n == 0 { 45.0 } else { -45.0 };
            let at = Transform::translate(centre[0], centre[1], 0.0)
                .then(&Transform::rotate_z(turn))
                .then(&Transform::translate(-CLOSE / 2.0, -1.0, 0.0));
            Command::Update {
                id: close_id(&item.id, n),
                props: PropsUpdate {
                    transform: Some(at),
                    ..PropsUpdate::default()
                },
            }
        })
        .collect()
}

/// The time and the words of a note, `18:51  Update ready`.
fn split_time(line: &str) -> Option<(&str, &str)> {
    let (time, words) = line.split_once("  ")?;
    let digits = |part: &str| part.len() == 2 && part.bytes().all(|b| b.is_ascii_digit());
    let (hour, minute) = time.split_once(':')?;
    (digits(hour) && digits(minute)).then_some((time, words.trim_start()))
}

/// The space that a text carries after its last letter: the tracking, which
/// an item flush right gives back so that its ink meets the edge.
fn trailing(props: &PropsUpdate) -> f64 {
    props.letter_spacing.unwrap_or(0.0) * props.font_size.unwrap_or(0.0)
}

/// The capitals of the text that the props describe, from the top of its box.
fn props_ink(props: &PropsUpdate) -> Ink {
    let size = props.font_size.unwrap_or(16.0);
    let line = size * props.line_height.unwrap_or(1.2);
    fonts::ink(props.family.as_deref(), props.weight, size, line)
}

/// The size of the focus mark, and its distance from what it marks.
const MARK_WIDTH: f64 = 2.0;
const MARK_GAP: f64 = 12.0;

/// The vertical places of the items that sit in the foot, or of those
/// that do not, from 0, and where they end.
pub fn lines(
    items: &[Item],
    heights: &[f64],
    foot: bool,
    baseline: f64,
) -> (Vec<(usize, f64)>, f64) {
    let (mut y, mut line, mut out) = (0.0, 0.0, Vec::new());
    for (i, item) in items
        .iter()
        .enumerate()
        .filter(|(_, it)| it.put.foot() == foot)
    {
        if item.put.row() {
            out.push((i, line));
            y = f64::max(y, line + heights[i]);
            continue;
        }
        if item.first && y > 0.0 {
            y += baseline;
        }
        y += item.before;
        line = y;
        out.push((i, y));
        y += heights[i];
    }
    (out, y)
}

/// Cuts the text of an item to a share of its width, with an ellipsis;
/// nothing below a few letters.
// ponytail: by the share of the letters, not by their widths, and one
// frame shows the whole text first; an ellipsis of the core would fix both.
fn shorten(item: &Item, share: f64) -> Command {
    let text = item.props.text.as_deref().unwrap_or_default();
    let count = text.chars().count();
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )] // a line of a screen
    let keep = (count as f64 * share).floor().max(0.0) as usize;
    let short = if keep < 4 {
        " ".to_owned()
    } else {
        let cut: String = text.chars().take(keep - 1).collect();
        format!("{}…", cut.trim_end())
    };
    Command::Update {
        id: item.id.clone(),
        props: PropsUpdate {
            text: Some(short),
            ..PropsUpdate::default()
        },
    }
}

/// Moves a surface under a group.
fn park(surface: &str, parent: &str) -> Command {
    Command::Move {
        id: surface.into(),
        parent: Some(parent.into()),
        index: None,
    }
}

/// Puts a window at a box of its parent: full screen at the size of the
/// output, else tiled.
pub fn place(surface: &str, [x, y, w, h]: [f64; 4], output: [i32; 2]) -> [Command; 2] {
    #[allow(clippy::cast_possible_truncation)] // pixels of a screen
    let size = [w.round() as i32, h.round() as i32];
    let full = size == output && x == 0.0 && y == 0.0;
    let state = if full {
        ToplevelState::Fullscreen
    } else {
        ToplevelState::Tiled
    };
    [
        move_to(surface.into(), x, y),
        Command::Configure {
            surface: surface.into(),
            size: Some(size),
            states: vec![state],
        },
    ]
}

pub fn rect(id: &str, parent: &str, [x, y, w, h]: [f64; 4], color: [f64; 4]) -> Command {
    create(
        id,
        NodeKind::Rect,
        parent,
        PropsUpdate {
            transform: Some(crate::translate(x, y)),
            size: Some([w, h]),
            color: Some(color),
            ..PropsUpdate::default()
        },
    )
}

#[cfg(test)]
mod tests {
    use chora_kit::settings::parse;

    use super::*;
    use crate::config::{Config, DEFAULT};

    /// The clock never asks for more than the core allows, also in the
    /// first milliseconds of a minute.
    #[test]
    fn the_clock_duration_stays_in_range_at_a_minute_boundary() {
        for now in [0, 1, 49, 50, 59_999, 60_000, 120_000, 120_049] {
            let ms = next_minute_after(now);
            assert!(
                (0.0..=chora_scene::MAX_DURATION).contains(&ms),
                "{now}: {ms}"
            );
        }
        assert_eq!(next_minute_after(59_000), 1_050.0);
    }

    fn names(placed: &BTreeMap<Region, Vec<(i32, &str)>>, region: Region) -> Vec<String> {
        placed[&region]
            .iter()
            .map(|(_, n)| (*n).to_owned())
            .collect()
    }

    #[test]
    fn the_first_rule_wins_by_content_kind_and_scene() {
        let config = parse::<Config>(DEFAULT).unwrap();
        let page = layout(&config.rules, &config.contents, "agenda", false);
        let app = layout(&config.rules, &config.contents, "foot", true);
        assert_eq!(names(&page, Region::Header), ["head", "clock"]);
        assert_eq!(
            names(&page, Region::Footer),
            ["prev", "hud", "urgent", "next"]
        );
        assert_eq!(
            names(&page, Region::Right),
            [
                "state", "agents", "notes", "notify", "chat", "keys", "doors"
            ]
        );
        assert!(!page.contains_key(&Region::Drawer), "no drawer by default");
        assert_eq!(
            names(&app, Region::Header),
            ["head", "hud", "agents", "clock"]
        );
        assert_eq!(names(&app, Region::Footer), ["urgent"]);
        // A rule of the session goes first; one by kind catches the rest.
        let mut rules = vec![Rule {
            content: None,
            kind: Some(Kind::Text),
            scenes: vec!["agenda".into()],
            region: Region::Left,
            slot: 3,
        }];
        rules.extend(config.rules.clone());
        let moved = layout(&rules, &config.contents, "agenda", false);
        assert_eq!(
            names(&moved, Region::Left),
            [
                "confirm", "next", "notes", "notify", "outline", "overlay", "prev", "urgent"
            ]
        );
        assert_eq!(
            names(&moved, Region::Right),
            ["state", "agents", "chat", "keys", "doors"]
        );
        let other = layout(&rules, &config.contents, "mail", false);
        assert_eq!(
            names(&other, Region::Right),
            [
                "state", "agents", "notes", "notify", "chat", "keys", "doors"
            ]
        );
    }

    #[test]
    fn the_column_grows_with_the_output_in_whole_lines() {
        assert_eq!(column_width(12.0, 28.0, 1280.0), 336.0);
        assert_eq!(column_width(12.0, 28.0, 1920.0), 448.0);
        assert_eq!(column_width(12.0, 28.0, 1440.0), 364.0);
        assert_eq!(column_width(12.0, 28.0, 800.0), 336.0);
    }

    #[test]
    fn mistakes_fail() {
        let config = parse::<Config>(DEFAULT).unwrap();
        let mut rules = config.rules.clone();
        rules.push(Rule {
            content: Some("shell".into()),
            kind: None,
            scenes: Vec::new(),
            region: Region::Header,
            slot: 0,
        });
        assert!(
            check(
                &config.regions,
                &config.contents,
                &rules,
                (&config.preset, &config.presets)
            )
            .is_err()
        );
        let gone = DEFAULT.replace("content: notes,", "content: nothing,");
        assert!(parse::<Config>(&gone).unwrap_err().contains("no content"));
        let mut regions = config.regions.clone();
        regions.drawer.size = 0.0;
        assert!(
            check(
                &regions,
                &config.contents,
                &config.rules,
                (&config.preset, &config.presets)
            )
            .is_err()
        );
    }

    /// Live at 1156 logical pixels (scale 1.5): the right side showed
    /// as open but drew nothing, since the sides had to leave room for
    /// all four columns of the grid, which wrap there anyway.
    #[test]
    fn the_sides_show_on_a_narrow_output() {
        let (mut mc, model) = crate::testing::mc("narrow", [1156, 1115]);
        mc.show("agenda", true).unwrap();
        mc.set_text("notes", None, |_| vec!["Call Lena".into()])
            .unwrap();
        let id = format!("{}-notes", mc.region_id(Region::Right));
        assert!(crate::testing::has(&model, &id), "{id}");
        assert!(mc.side_widths("agenda")[1] > 0.0);
        // A side that does not leave a column of text still hides.
        let (mc, _) = crate::testing::mc("tiny", [700, 700]);
        assert_eq!(mc.side_widths("agenda"), [0.0, 0.0]);
    }

    /// The probe of an update whose transaction the caller drops, as
    /// the core refuses it, must not stay: every later update failed with
    /// Exists.
    #[test]
    fn the_probe_goes_also_when_the_update_is_dropped() {
        let (mut mc, model) = crate::testing::mc("probe", [1920, 1080]);
        mc.show("agenda", true).unwrap();
        let dropped = mc.regions().unwrap();
        assert!(!dropped.is_empty(), "an update of the same regions");
        assert!(!crate::testing::has(&model, PROBE));
    }

    #[test]
    fn colors_go_into_the_command() {
        let colors = [("background".to_owned(), [1.0, 0.5, 0.0, 1.0])].into();
        assert_eq!(
            command("foot -o bg={background} {x}", &colors),
            "foot -o bg=ff8000 {x}"
        );
    }
}
