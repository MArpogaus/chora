//! Regions around the page, after the Emacs package auto-side-windows: a
//! header and a footer line, a left and a right side, and a drawer at the
//! bottom. Each holds contents in slots: the context of the scene, the
//! clock, the state and the words of the agent, lines of text such as
//! notes, or a docked app. Rules decide which content goes to which region
//! and slot, by content, by kind and by scene; the first rule that matches
//! wins.

use std::collections::BTreeMap;

use chora_scene::{Command, Easing, NodeKind, PropsUpdate, ToplevelState, ipc::ClientError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    Mc,
    config::{Style, Type},
    create, move_to,
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
    fn is_line(self) -> bool {
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

/// The milliseconds to the next full minute, and a little more.
pub fn to_next_minute() -> f64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    #[allow(clippy::cast_precision_loss)] // below 60000
    let ms = (60_000 - now % 60_000) as f64;
    ms + 50.0
}

/// One text node of a region.
pub struct Item {
    pub id: String,
    pub slot: i32,
    /// The props, without the place.
    pub props: PropsUpdate,
    /// Where a stack in a side puts space before it: a new content.
    pub first: bool,
}

/// The items and the docked app of each region.
type Plan = BTreeMap<Region, (Vec<Item>, Option<String>)>;

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

    /// Whether a scene is one of an app: no head, no panels.
    pub fn is_app(&self, name: &str) -> bool {
        self.scenes.get(name).is_none_or(|s| !Self::has_chapter(s))
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
    /// then shows as before. Answers whether it shows.
    pub fn solo(
        &mut self,
        content: &str,
        region: Region,
        shown: Option<bool>,
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
            if self
                .focused
                .as_ref()
                .is_some_and(|f| self.docks.values().any(|d| d == f))
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
            _ if title => "running_head",
            Kind::Head | Kind::Clock | Kind::State => "running_head",
            Kind::Hud | Kind::Agents => "hud",
            Kind::Text | Kind::App => "margin",
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
            Kind::App => (String::new(), String::new()),
        }
    }

    /// The running head of a scene: the one of its chapter, or its number
    /// and title; the name of an app.
    fn head(&self, name: &str) -> String {
        let Some(scene) = self.scenes.get(name) else {
            return String::new();
        };
        if !Self::has_chapter(scene) {
            return title_of(name);
        }
        let chapter = &scene.chapter;
        chapter.running_head.clone().unwrap_or_else(|| {
            let title = chapter.title.clone().unwrap_or_else(|| title_of(name));
            format!("{}  {title}", self.number(name, scene))
        })
    }

    /// Whether a region shows in a scene: as the toggle says, else as the
    /// scene says, else as the settings say; over an app the sides show
    /// only by a toggle, so that the app keeps its width.
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
                shown && !(side && self.is_app(name))
            })
    }

    /// The items of the regions of a scene, and the docked app of each.
    fn items(&self, name: &str) -> Plan {
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
        if let Some(scene) = self.scenes.get(name) {
            let [left, right] = self.side_widths(name, scene);
            out.retain(|region, _| match region {
                Region::Left => left > 0.0,
                Region::Right => right > 0.0,
                _ => true,
            });
        }
        out
    }

    /// The text items of a content in a region: one in a line, else its
    /// title and its lines.
    fn content_items(&self, region: Region, slot: i32, name: &str, scene: &str) -> Vec<Item> {
        let content = &self.settings.value.contents[name];
        if content.kind == Kind::App {
            return Vec::new();
        }
        if content.kind == Kind::Agents {
            return self.agent_items(region, slot, name, content);
        }
        let (title, text) = self.content_text(name, content, scene);
        let id = format!("{}-{name}", self.region_id(region));
        let item = |id: String, is_title: bool, text: &str, first: bool| Item {
            id,
            slot,
            props: self.content_type(content, is_title).props(text),
            first,
        };
        if region.is_line() {
            let text = text.replace('\n', "  ·  ");
            return (!text.is_empty())
                .then(|| item(id, false, &text, true))
                .into_iter()
                .collect();
        }
        // A text with a title shows it in a side that the user opened, so
        // that the toggle shows where notes go.
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
    /// less than three quarters of its measure.
    pub fn side_widths(&self, name: &str, scene: &Scene) -> [f64; 2] {
        let Some(output) = &self.output else {
            return [0.0, 0.0];
        };
        let config = &self.settings.value;
        let (em, b) = (config.scale.base, config.scale.baseline);
        let page = self.page(scene);
        let gap = page.gap * em;
        let cols = f64::from(self.grid(scene)[0]);
        let want = cols * page.measure * em + (cols - 1.0) * (page.margin * em + 2.0 * gap);
        let room = f64::from(output.size[0]) - 4.0 * b - 2.0 * (page.margin * em + gap);
        let placed = self.placed(name);
        let sides = [Region::Left, Region::Right].map(|region| {
            if placed.contains_key(&region) {
                config.regions.get(region).size * em
            } else {
                0.0
            }
        });
        let need: f64 = sides.iter().filter(|w| **w > 0.0).map(|w| w + gap).sum();
        if room - need < 0.75 * want {
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
        let app = self.is_app(name);
        // The text block of the scene, or of a page of one column over an
        // app, so that the lines stay where they are between scenes.
        let geo = if app {
            self.geometry_of(name, &Scene::default())?
        } else {
            self.geometry(name)?
        };
        let [x, top, width, _] = geo.text;
        let (note, gap) = (geo.note, geo.gap);
        let mut regions = BTreeMap::new();
        // Over the text of a page, else over the whole app.
        let (lx, lw) = if app {
            (2.0 * b, w - 4.0 * b)
        } else {
            (x, width)
        };
        regions.insert(Region::Header, [lx, 0.0, lw, head]);
        regions.insert(Region::Footer, [lx, h - foot, lw, foot]);
        let frame = [x - note - gap, width + 2.0 * (note + gap)];
        let drawer_top = h - foot - drawer;
        regions.insert(Region::Drawer, [frame[0], drawer_top, frame[1], drawer]);
        let [lw, rw] = self.side_widths(name, self.scenes.get(name)?);
        let side_top = if app { head + b } else { top };
        let side_h = (drawer_top - b - side_top).max(b);
        let mut window = [0.0, head, w, (drawer_top - head).max(b)];
        if app {
            // Each side has a box, also one that fades out.
            regions.insert(Region::Left, [b, side_top, lw, side_h]);
            regions.insert(Region::Right, [w - b - rw, side_top, rw, side_h]);
            if on(Region::Left) {
                window[0] = lw + 2.0 * b;
            }
            let right = if on(Region::Right) { rw + 2.0 * b } else { 0.0 };
            window[2] = (w - window[0] - right).max(b);
        } else {
            let left = x - note - 2.0 * gap - lw;
            regions.insert(Region::Left, [left, side_top, lw, side_h]);
            regions.insert(Region::Right, [x + width + gap, side_top, rw, side_h]);
        }
        Some(Boxes { regions, window })
    }

    /// The regions that show in the current scene.
    fn visible(&self, items: &Plan) -> Vec<Region> {
        let name = self.current.clone().unwrap_or_default();
        (items.keys().copied())
            .filter(|r| self.shown(*r, &name))
            .collect()
    }

    /// Draws the regions of the current scene anew.
    pub fn render(&mut self) -> Result<(), ClientError> {
        let commands = self.regions()?;
        self.apply(commands)
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
        if self.rendered.as_ref() == Some(&key) {
            self.sigs = sigs;
            return self.update(&items, &boxes);
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
        // The window of an app scene leaves the regions free.
        if let Some(surface) = self.windows.get(&name) {
            commands.extend(place(surface, boxes.window, self.output_size()));
        }
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
        let mut updates = vec![Command::Remove {
            id: PROBE.into(),
            to: None,
        }];
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
        if moves.is_err() {
            self.apply(updates.drain(..1).collect())?;
        }
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
        for item in items {
            let props = PropsUpdate {
                max_width: (!region.is_line()).then_some(width),
                ..item.props.clone()
            };
            commands.push(create(&item.id, NodeKind::Text, &id, props));
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
        let baseline = self.settings.value.scale.baseline;
        let mut commands = Vec::new();
        for (region, (items, _)) in items {
            let [_, _, width, height] = boxes.regions[region];
            if region.is_line() {
                commands.extend(self.line(items, [width, height], probed)?);
                continue;
            }
            let mut y = 0.0;
            for item in items {
                if item.first && y > 0.0 {
                    y += baseline;
                }
                commands.push(move_to(item.id.clone(), 0.0, y));
                let [_, _, _, item_height] = self.measure(&item.id, probed)?;
                y += item_height.max(baseline);
            }
        }
        Ok(commands)
    }

    /// Places the items of a line: the end keeps its place, the start ends
    /// before it, and the middle sits between them; what does not fit is
    /// cut.
    fn line(
        &mut self,
        items: &[Item],
        [width, height]: [f64; 2],
        probed: bool,
    ) -> Result<Vec<Command>, ClientError> {
        let config = &self.settings.value;
        let (baseline, gap) = (config.scale.baseline, config.page.gap * config.scale.base);
        let mut sizes = Vec::new();
        for item in items {
            let [_, _, w, h] = self.measure(&item.id, probed)?;
            sizes.push((item, w, h));
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
        for (group, x0, room) in [
            (start, 0.0, high),
            (middle, mid_x, high - mid_x),
            (end, end_x, width - end_x),
        ] {
            let mut x = x0;
            for (item, w, h) in group {
                let left = x0 + room - x;
                if w > left + 0.5 {
                    commands.push(shorten(item, left / w));
                }
                let y = ((height - h.max(baseline)) / 2.0).round();
                commands.push(move_to(item.id.clone(), x, y));
                x += w.min(left) + gap;
            }
        }
        Ok(commands)
    }

    /// Shows or hides a region, by key or by method. Without `shown`, a
    /// region whose docked app has no focus takes it first; a second
    /// press hides it. Answers whether it shows.
    pub fn toggle(&mut self, region: Region, shown: Option<bool>) -> Result<bool, ClientError> {
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
        if shown.is_none()
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
                Some(surface) => self.focus(Some(surface.clone()))?,
                None => self.start_dock(app),
            }
        }
        if !new && surface.is_some() && self.focused == surface {
            let window = self.windows.get(&name).cloned();
            self.focus(window)?;
        }
        self.render()?;
        Ok(new)
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
        if let Some((token, _)) = crate::launch(&command) {
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
fn place(surface: &str, [x, y, w, h]: [f64; 4], output: [i32; 2]) -> [Command; 2] {
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
        assert_eq!(
            names(&page, Region::Header),
            ["head", "agents", "state", "clock"]
        );
        assert_eq!(names(&page, Region::Footer), ["hud", "urgent", "next"]);
        assert_eq!(names(&page, Region::Right), ["notes", "notify"]);
        assert_eq!(names(&page, Region::Drawer), ["shell"]);
        // Over an app, the words of the agent go to the header.
        assert_eq!(
            names(&app, Region::Header),
            ["head", "hud", "agents", "state", "clock"]
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
            ["next", "notes", "notify", "outline", "overlay", "urgent"]
        );
        assert!(!moved.contains_key(&Region::Right));
        let other = layout(&rules, &config.contents, "mail", false);
        assert_eq!(names(&other, Region::Right), ["notes", "notify"]);
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

    #[test]
    fn colors_go_into_the_command() {
        let colors = [("background".to_owned(), [1.0, 0.5, 0.0, 1.0])].into();
        assert_eq!(
            command("foot -o bg={background} {x}", &colors),
            "foot -o bg=ff8000 {x}"
        );
    }
}
