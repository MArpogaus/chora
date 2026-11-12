//! The chat session of the host: it opens a chat with a helper
//! (`chat.open`), pulls its transcript (`agent.read`), keeps the field
//! (`chora-chat --input`) and the view in step, and closes the chat
//! (`chat.close`). The model is `chora_kit::chat`; the session owns no
//! socket and no process. It reaches voice and the field through
//! [`Outside`], so that a test plays both.
//!
//! The chat is no window: a switch of look is a restyle, the field stays
//! the same process and so keeps its draft, and the context holds while
//! the field has the keyboard. The view is drawn from the `chat:` values
//! of the folder, with the words of `chora_kit::words`; agent text is a
//! message under the name of its helper and never a card.

use std::collections::BTreeMap;

use chora_draw::bubble::Style;
use chora_kit::{
    ask::{Keys, Question, show_key},
    chat::{Chat as Transcript, Line, Measure, Role, sanitize, wrap},
    hud::{Anchor, Chat as Look, ChatStyle},
    words::{self, Language, Word, say},
};
use serde_json::{Value, json};

use crate::{
    assistant::origin,
    check::host_chat,
    config::TAIL,
    field::{FieldEvent, say_params},
};

mod face;
mod glass;
mod host;
mod view;

pub use face::Face;
pub use host::{Announced, ShowParams, exported, token_of};
pub use view::{card_origin, card_width, frame, keep_mark, mark_id, marked, show_mark};

const PULLS: usize = 8;
const MAX_STATES: usize = 64;
const BLOCKED: &str = "blocked";

fn is_error(answer: &str) -> bool {
    answer.starts_with("error")
}

/// Where the chat shows: the output, its size and work area.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    /// The name of the output.
    pub output: String,
    /// Its size in logical pixels.
    pub size: [i32; 2],
    /// Its work area: x, y, width, height.
    pub area: [i32; 4],
    /// The `chat:` values of the folder that draws.
    pub look: Look,
    /// The box that the client of `chat.slot` announced on this output.
    pub slot: Option<[i32; 4]>,
    /// What the view never covers: the figure with its balloon and the
    /// "Chat ..." row, each as x, y, width, height.
    pub clear: Vec<[i32; 4]>,
    /// The sheet of the figure, whose element of a state a glass view
    /// shows as its ring.
    pub ring: Option<chora_draw::theme::Sheet>,
    /// The look is in the dark theme.
    pub dark: bool,
    /// The scale of the output, for the field.
    pub scale: u32,
    /// The question card on the output, x, y, width and height: the
    /// transcript of a glass view flows under it.
    pub card: Option<[i32; 4]>,
    /// The head of the figure, where the tail of a `bubble` chat points.
    pub tip: Option<[i32; 2]>,
    /// The height of the field at the bottom of the view.
    pub field: i32,
}

impl Frame {
    /// The rectangle of the view: the box of the slot inside the work
    /// area exactly as announced, else the place of the `chat:` values,
    /// `margin` away from the edges of the work area; there beside what
    /// the view must not cover. The client of a slot owns the box, so the
    /// view never steps aside in it.
    #[must_use]
    pub fn rect(&self) -> [i32; 4] {
        if let Some(slot) = self.slot.and_then(|slot| self.clipped(slot)) {
            return slot;
        }
        if let Some(tip) = self.balloon_tip() {
            return self.balloon(tip);
        }
        let (anchor, mut width) = self.look.place();
        if self.look.style == chora_kit::hud::ChatStyle::Glass {
            width = glass_width(width, self.area[2]);
        }
        let margin = i32::try_from(self.look.margin).unwrap_or(0);
        let [x, y, w, h] = self.area;
        let inside = [
            x + margin,
            y + margin,
            (w - 2 * margin).max(1),
            (h - 2 * margin).max(1),
        ];
        let view = view_rect(anchor, width, inside);
        if matches!(anchor, Anchor::Left | Anchor::Right) {
            return view;
        }
        self.aside(view)
    }

    /// The head of the figure when the chat is its balloon: a `bubble`
    /// chat of a folder with a figure, and no slot.
    fn balloon_tip(&self) -> Option<[i32; 2]> {
        self.tip
            .filter(|_| self.look.style == ChatStyle::Bubble && self.slot.is_none())
    }

    /// The balloon of the figure: above its head, as wide as the chat, at
    /// most 60 % of the work area high, its tail ending at the head, inside
    /// the work area.
    fn balloon(&self, [tip_x, tip_y]: [i32; 2]) -> [i32; 4] {
        let [ax, ay, aw, ah] = self.area;
        let (_, width) = self.look.place();
        let w = i32::try_from(width).unwrap_or(i32::MAX).min(aw);
        let inset = Style::default().tail_inset;
        let right = (tip_x + inset).min(ax + aw);
        let left = (right - w).max(ax);
        let bottom = (tip_y - TAIL).max(ay + 1);
        let top = (bottom - ah * 6 / 10).max(ay);
        [left, top, w, bottom - top]
    }

    /// Where the tail of a balloon chat leaves its bottom edge, from its
    /// left edge; none for a chat that is no balloon.
    #[must_use]
    pub fn tail(&self) -> Option<i32> {
        let [tip_x, _] = self.balloon_tip()?;
        let [x, _, w, _] = self.rect();
        let style = Style::default();
        let lowest = BALLOON_RADIUS + style.tail_base;
        Some((tip_x - x).clamp(lowest, (w - BALLOON_RADIUS).max(lowest)))
    }

    /// The part of a slot box inside the work area, no wider than a chat
    /// may be; none for a box that lies outside the area.
    fn clipped(&self, [x, y, w, h]: [i32; 4]) -> Option<[i32; 4]> {
        let [ax, ay, aw, ah] = self.area;
        let widest = i32::try_from(chora_kit::hud::MAX_CHAT_WIDTH).unwrap_or(i32::MAX);
        let (left, top) = (x.max(ax), y.max(ay));
        let right = x.saturating_add(w).min(ax + aw);
        let bottom = y.saturating_add(h).min(ay + ah);
        (right > left && bottom > top)
            .then(|| [left, top, (right - left).min(widest), bottom - top])
    }

    /// The view moved sideways, beside what it would cover, where it fits
    /// in the work area; the shorter way wins. A panel at the left or the
    /// right edge keeps its edge.
    fn aside(&self, [x, y, w, h]: [i32; 4]) -> [i32; 4] {
        let covered =
            |[cx, cy, cw, ch]: &&[i32; 4]| *cx < x + w && x < cx + cw && *cy < y + h && y < cy + ch;
        let spans = self.clear.iter().filter(covered);
        let Some((from, to)) = spans
            .map(|[cx, _, cw, _]| (*cx, cx + cw))
            .reduce(|a, b| (a.0.min(b.0), a.1.max(b.1)))
        else {
            return [x, y, w, h];
        };
        let [ax, _, aw, _] = self.area;
        let fits = |left: &i32| *left >= ax && left + w <= ax + aw;
        let moved = [from - ASIDE - w, to + ASIDE]
            .into_iter()
            .filter(fits)
            .min_by_key(|left| (left - x).abs());
        [moved.unwrap_or(x), y, w, h]
    }
}

/// The width of a glass chat: 28 hundredths of the work area, at least
/// 400 and at most the width of the folder (construct HUD spec, 5).
fn glass_width(most: u32, area: i32) -> u32 {
    let share = u32::try_from(area.max(0) * 28 / 100).unwrap_or(0);
    share.clamp(GLASS_LEAST.min(most), most)
}

/// The narrowest glass chat.
const GLASS_LEAST: u32 = 400;

/// The gap between the view and what it stands beside.
const ASIDE: i32 = 8;
/// The height of a field of one line, before it tells its own.
pub const ONE_LINE_FIELD: i32 = 36;
/// The corner radius of a `bubble` chat.
pub const BALLOON_RADIUS: i32 = 10;

/// A box that a client announced for its slot: the client, the output and
/// the box.
pub type Held = (String, String, [i32; 4]);

/// The box that holds the view on an output: the one that the client of
/// `chat.slot` of the folder on the screen announced there. A box of
/// another client, kept from another folder, holds nothing.
#[must_use]
pub fn held_box(held: Option<&Held>, slot: Option<&str>, output: &str) -> Option<[i32; 4]> {
    let (client, on, rect) = held?;
    (Some(client.as_str()) == slot && on == output).then_some(*rect)
}

/// The box of a slot on an output; none when its client took it away.
pub type SlotBox = (String, Option<[i32; 4]>);

/// The output and the box of the event `CLIENT.chat.slot {output, rect}`,
/// if the client is the one that the folder names in `chat.slot`: no other
/// client fills the slot. A box has four sides that are numbers and a
/// size; the host clips it to the work area when it draws.
#[must_use]
pub fn slot_box(
    slot: Option<&str>,
    (event, client): (&str, &str),
    data: &Value,
) -> Option<SlotBox> {
    let named = slot.filter(|slot| *slot == client)?;
    if event.strip_prefix(named) != Some(".chat.slot") {
        return None;
    }
    let output = data.get("output")?.as_str()?.to_owned();
    let rect = &data["rect"];
    if rect.is_null() {
        return Some((output, None));
    }
    let sides = rect.as_array().filter(|sides| sides.len() == 4)?;
    let sides: Vec<i32> = sides
        .iter()
        .map(|side| side.as_f64().filter(|n| n.is_finite() && n.abs() < 1e6))
        .collect::<Option<Vec<f64>>>()?
        .into_iter()
        .map(pixels)
        .collect();
    let rect: [i32; 4] = sides.try_into().ok()?;
    (rect[2] > 0 && rect[3] > 0).then_some((output, Some(rect)))
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "a side of a box is below a million pixels"
)]
fn pixels(side: f64) -> i32 {
    side.round() as i32
}

/// What the session needs from the rest of the host.
pub trait Outside {
    /// Calls a method of voice.
    ///
    /// # Errors
    ///
    /// Voice does not answer or refuses.
    fn voice(&mut self, method: &str, params: Value) -> Result<Value, String>;
    /// The activation token of the act that is happening now, if the core
    /// gives one.
    fn token(&mut self) -> Option<String>;
    /// Starts the field.
    ///
    /// # Errors
    ///
    /// The program does not start.
    fn start_field(&mut self) -> Result<Box<dyn Link>, String>;
}

/// The field, as the session talks to it.
pub trait Link {
    /// Shows the field at a place.
    fn place(&mut self, place: &Value);
    /// A question of the helper waits or ended.
    fn question(&mut self, waiting: bool);
    /// A text did not reach the helper.
    fn say_failed(&mut self, text: &str);
    /// A line was typed in the list before a helper was picked.
    fn pick_first(&mut self, text: &str);
    /// The chat goes to another helper or to the list: the draft goes.
    fn clear(&mut self);
    /// A voice event, for the field if it concerns the helper.
    fn voice_event(&mut self, event: &str, data: &Value, helper: &str);
    /// What the field told since the last call.
    fn events(&mut self) -> Vec<FieldEvent>;
    /// Whether the program still runs.
    fn alive(&mut self) -> bool;
    /// Takes the surface away and keeps the draft.
    fn hide(&mut self);
}

/// An entry of the list that opens without a name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pick {
    /// A helper that runs.
    Agent(String),
    /// A project without a helper: a pick starts one.
    Project(String),
}

impl Pick {
    fn name(&self) -> &str {
        match self {
            Self::Agent(name) | Self::Project(name) => name,
        }
    }

    /// The words of the entry: a helper by its name, a project as the
    /// helper that a pick starts for it.
    fn label(&self, language: Language) -> String {
        match self {
            Self::Agent(name) => name.clone(),
            Self::Project(project) => say(language, Word::NewHelper, &[("project", project)]),
        }
    }
}

/// What the view shows.
#[derive(Debug, Default, PartialEq, Eq)]
pub enum Mode {
    /// Nothing.
    #[default]
    Closed,
    /// The numbered list of helpers and projects.
    List(Vec<Pick>),
    /// The chat with a helper.
    Chat(String),
}

/// Where a question appeared, and so where its card stays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Where {
    /// Inside the chat view.
    Chat,
    /// At the figure.
    Figure,
}

/// What `voice.state` last said of the helper.
#[derive(Debug, Default)]
struct Status {
    name: String,
    state: String,
    task: String,
    steps: Option<(u64, u64)>,
}

/// The chat of the host.
#[derive(Default)]
pub struct Session {
    mode: Mode,
    transcript: Transcript,
    link: Option<Box<dyn Link>>,
    keyboard: bool,
    keys: Keys,
    card: Option<Where>,
    status: Status,
    head: u64,
    offline: bool,
    drawn: Option<String>,
    dirty: bool,
    states: BTreeMap<String, Value>,
    epoch: Option<u64>,
    last: Option<String>,
    /// What the agent said while the view is the open balloon of the
    /// figure.
    spoken: Option<String>,
    /// The height of the card of the question in the view, whose room the
    /// transcript leaves free.
    card_height: Option<i32>,
    /// The height of the field, as it last told it.
    field_height: Option<i32>,
    /// The top of the card of the question in the view, on the output.
    card_top: Option<i32>,
    /// The question that the state still names was ended: the card went,
    /// by an answer that voice has not told yet.
    answered: Answered,
    /// What the person sent that the transcript does not hold yet, with the
    /// `seq` it was sent after: it shows at once and goes when voice has it.
    sent: Vec<(String, u64)>,
}

/// Whether the question that the state of the helper still names was
/// answered here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Answered {
    #[default]
    No,
    Yes,
}

/// The rectangle of the view in the work area: x, y, width, height. The
/// view takes the height of the work area at the left and right edge, and
/// six tenths of it elsewhere.
#[must_use]
pub fn view_rect(anchor: Anchor, width: u32, area: [i32; 4]) -> [i32; 4] {
    let w = i32::try_from(width).unwrap_or(i32::MAX).min(area[2]);
    let h = match anchor {
        Anchor::Left | Anchor::Right => area[3],
        _ => area[3] * 6 / 10,
    };
    let [x, y] = origin(anchor, area, [w, h], [0, 0]);
    [x, y, w, h]
}

/// The colours of the view: the `chat:` values over those of the host.
#[derive(Clone, Copy)]
struct Colours {
    fill: [u8; 3],
    text: [u8; 3],
    name: [u8; 3],
    border: [u8; 3],
}

fn colours(look: &Look) -> Colours {
    let host = host_chat();
    let pick =
        |own: Option<[u8; 3]>, hosts: Option<[u8; 3]>, last: [u8; 3]| own.or(hosts).unwrap_or(last);
    let fill = pick(look.fill, host.fill, [250, 250, 250]);
    let border = pick(look.border, host.border, [60, 60, 60]);
    Colours {
        fill,
        text: pick(look.text, host.text, [32, 32, 32]),
        name: pick(look.name, host.name, [0, 80, 160]),
        border: if look.style == chora_kit::hud::ChatStyle::Glass {
            hairline(border, fill)
        } else {
            border
        },
    }
}

/// The opacity of the edge of glass: a hairline, not a line.
const GLASS_EDGE: f64 = 0.24;

/// The hairline of a glass chat: its border at the opacity of an edge,
/// over the fill.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a mix of two colour values is from 0 to 255"
)]
fn hairline(border: [u8; 3], fill: [u8; 3]) -> [u8; 3] {
    std::array::from_fn(|i| {
        let mixed = f64::from(border[i]) * GLASS_EDGE + f64::from(fill[i]) * (1.0 - GLASS_EDGE);
        mixed.round() as u8
    })
}

/// The fill, the text colour and the muted colour of the page of a look,
/// and its reading face: what a card takes that stands in the page.
#[must_use]
pub fn page_ink(look: &Look) -> ([u8; 3], [u8; 3], [u8; 3], Option<Face>) {
    let colours = colours(look);
    (colours.fill, colours.text, colours.name, Face::of(look))
}

fn rgba([r, g, b]: [u8; 3]) -> [u8; 4] {
    [r, g, b, 255]
}

/// The room left of the field in the margin column for the caret of an
/// empty field: the surface is this much wider than the column.
const FIELD_INSET: i32 = 6;

/// The `place` line of the field for a frame, with the token if there is
/// one: at the bottom of the view, as wide as the view; in the margin
/// column with the name of the helper that it writes to.
#[must_use]
pub fn field_place_for(frame: &Frame, token: Option<&str>, to: Option<&str>) -> Value {
    if frame.look.style == chora_kit::hud::ChatStyle::Glass {
        return glass_field_place(frame, token);
    }
    let [x, y, w, h] = frame.rect();
    let inset = if frame.tail().is_some() { view::PAD } else { 0 };
    let [x, w, h] = [x + inset, w - 2 * inset, h - inset];
    let colours = colours(&frame.look);
    let mut place = json!({
        "anchor": "bottom_left",
        "margin": [0, 0, frame.size[1] - (y + h), x],
        "width": w,
        "output": frame.output,
        "look": {
            "background": rgba(colours.fill),
            "text": rgba(colours.text),
            "border": rgba(colours.border),
            "hint": rgba(colours.name),
        },
    });
    if frame.slot.is_some() {
        let margin = place["margin"].as_array().and_then(|m| m[3].as_i64());
        place["margin"][3] = json!(margin.unwrap_or(0) - i64::from(FIELD_INSET));
        place["width"] = json!(w + FIELD_INSET);
        place["scale"] = json!(frame.scale);
        place["look"]["column"] = json!({"inset": FIELD_INSET, "to": to});
    }
    if let Some(token) = token {
        place["token"] = token.into();
    }
    place
}

/// The `place` of the field of a glass chat: the well of 48 above the key
/// line, inset 8 in the view, with room of 6 around it for the focus ring,
/// in the colours of the glass at the scale of the output.
#[allow(
    clippy::many_single_char_names,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a rect and a colour as bytes, clamped from 0 to 255"
)]
fn glass_field_place(frame: &Frame, token: Option<&str>) -> Value {
    const RING_ROOM: i32 = 6;
    let [x, y, w, h] = frame.rect();
    let tokens = chora_kit::glass::Material::of(frame.dark);
    let byte = |c: [f64; 4]| c.map(|v| (v * 255.0).round().clamp(0.0, 255.0) as u8);
    let halo = {
        let [r, g, b, _] = tokens.accent;
        [r, g, b, chora_kit::glass::HALO_ALPHA]
    };
    let mut place = json!({
        "anchor": "bottom_left",
        "margin": [0, 0, frame.size[1] - (y + h) + 40 + 8 - RING_ROOM, x + 8 - RING_ROOM],
        "width": w - 16 + 2 * RING_ROOM,
        "output": frame.output,
        "scale": frame.scale,
        "look": {
            "background": byte(tokens.well),
            "text": byte(tokens.text),
            "border": [0, 0, 0, 0],
            "hint": byte(tokens.muted),
            "glass": {"accent": byte(tokens.accent), "halo": byte(halo)},
        },
    });
    if let Some(token) = token {
        place["token"] = token.into();
    }
    place
}

/// The entries of the text that `agent_list` answers:
/// `projects: a, b; agents: x (a, idle)`. A project that has a helper is
/// not listed again.
#[must_use]
pub fn parse_list(text: &str) -> Vec<Pick> {
    let part = |key: &str| {
        text.split(';')
            .find_map(|part| part.trim().strip_prefix(key))
            .map(str::trim)
            .filter(|rest| *rest != "none")
            .unwrap_or_default()
    };
    let agents: Vec<(String, String)> = part("agents:")
        .split("),")
        .filter_map(|entry| {
            let (name, rest) = entry.split_once('(')?;
            let project = rest.split(',').next()?.trim_end_matches(')').trim();
            Some((sanitize(name.trim()), project.to_owned()))
        })
        .collect();
    let mut picks: Vec<Pick> = agents
        .iter()
        .map(|(name, _)| Pick::Agent(name.clone()))
        .collect();
    for project in part("projects:").split(',').map(str::trim) {
        let started = agents.iter().any(|(_, p)| p == project);
        if !project.is_empty() && !started {
            picks.push(Pick::Project(sanitize(project)));
        }
    }
    picks
}

impl Session {
    /// A session with nothing open.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// What the view shows.
    #[cfg(test)]
    #[must_use]
    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    /// Whether a view shows.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.mode != Mode::Closed
    }

    /// The helper of the chat, if one is open.
    #[must_use]
    pub fn helper(&self) -> Option<&str> {
        match &self.mode {
            Mode::Chat(name) => Some(name),
            _ => None,
        }
    }

    fn waiting(&self) -> bool {
        self.status.state == "question" && self.answered == Answered::No
    }

    /// Whether the helper of the chat runs a prompt that Stop can end: it
    /// works, thinks, speaks or asks.
    #[must_use]
    pub fn running(&self) -> bool {
        ["working", "thinking", "speaking", "question"].contains(&self.status.state.as_str())
    }

    /// Whether the view is to be drawn again, and the mark taken away.
    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    /// Whether voice does not answer.
    #[must_use]
    pub fn offline(&self) -> bool {
        self.offline
    }

    /// Whether the field has the keyboard: the context holds.
    #[must_use]
    pub fn keyboard(&self) -> bool {
        self.keyboard
    }

    /// Shows the chat with `name`, or the list without one. `token` is the
    /// activation token of the act, if the caller has one. A voice that
    /// does not answer shows the view with the line that says so.
    ///
    /// # Errors
    ///
    /// The field does not start.
    pub fn show(
        &mut self,
        out: &mut dyn Outside,
        name: Option<&str>,
        token: Option<&str>,
        frame: &Frame,
    ) -> Result<(), String> {
        let name = name.map(sanitize).filter(|name| !name.is_empty());
        match name {
            Some(name) => {
                let other = self
                    .helper()
                    .is_some_and(|old| !old.eq_ignore_ascii_case(&name));
                if other && token.is_none() {
                    return Ok(());
                }
                self.open_chat(out, &name);
            }
            None => self.open_list(out),
        }
        self.place_field(out, frame, token)
    }

    fn open_chat(&mut self, out: &mut dyn Outside, name: &str) {
        let same = self
            .last
            .as_deref()
            .is_some_and(|old| old.eq_ignore_ascii_case(name));
        if !same {
            self.close_chat(out);
            self.transcript = Transcript::default();
            self.sent.clear();
            self.status = Status::default();
            self.head = 0;
            self.epoch = None;
            if let Some(link) = self.link.as_mut() {
                link.clear();
            }
        }
        self.mode = Mode::Chat(name.to_owned());
        self.last = Some(name.to_owned());
        self.announce_open(out, name);
    }

    fn announce_open(&mut self, out: &mut dyn Outside, name: &str) {
        match out.voice("chat.open", json!({"name": name})) {
            Ok(_) => self.offline = false,
            Err(error) => self.refused_or_offline(&error),
        }
        if let Some(data) = self.states.get(&name.to_lowercase()).cloned() {
            self.helper_state(out, &data);
        }
        if !self.offline && self.status.state != BLOCKED {
            self.pull(out);
        }
    }

    fn refused_or_offline(&mut self, error: &str) {
        self.offline = !error.contains("no agent");
        if !self.offline {
            BLOCKED.clone_into(&mut self.status.state);
        }
    }

    fn open_list(&mut self, out: &mut dyn Outside) {
        self.close_chat(out);
        let listed = out.voice("agent", json!({"tool": "agent_list"}));
        self.offline = listed.is_err();
        let picks = listed
            .ok()
            .and_then(|text| text.as_str().map(parse_list))
            .unwrap_or_default();
        if let Some(last) = self.only_helper(&picks) {
            self.open_chat(out, &last);
            return;
        }
        self.mode = Mode::List(picks);
        if let Some(link) = self.link.as_mut() {
            link.clear();
        }
    }

    /// The helper of the last chat when it is the only one that runs: a
    /// person who closed its chat reopens that chat, not a list of one.
    fn only_helper(&self, picks: &[Pick]) -> Option<String> {
        let mut agents = picks.iter().filter_map(|pick| match pick {
            Pick::Agent(name) => Some(name),
            Pick::Project(_) => None,
        });
        let (only, none_more) = (agents.next()?, agents.next().is_none());
        let last = self.last.as_deref()?;
        (none_more && only.eq_ignore_ascii_case(last)).then(|| only.clone())
    }

    fn close_chat(&mut self, out: &mut dyn Outside) {
        if let Mode::Chat(name) = &self.mode {
            let _ = out.voice("chat.close", json!({"name": name}));
        }
    }

    fn place_field(
        &mut self,
        out: &mut dyn Outside,
        frame: &Frame,
        token: Option<&str>,
    ) -> Result<(), String> {
        if self.link.is_none() {
            self.link = Some(out.start_field()?);
        }
        let waiting = self.waiting();
        let to = self.helper().map(str::to_owned);
        if let Some(link) = self.link.as_mut() {
            link.question(waiting);
            link.place(&field_place_for(frame, token, to.as_deref()));
        }
        Ok(())
    }

    /// A press on a node of the host takes the keyboard from the field. The
    /// press is an act of the person, so the field gets it back with the
    /// token of that act; without a token it gets nothing.
    pub fn refocus(&mut self, out: &mut dyn Outside, frame: &Frame, token: Option<&str>) {
        if self.is_open() {
            let _ = self.place_field(out, frame, token);
        }
    }

    /// A click on the "Chat ..." row: the list, with the token of the
    /// click.
    ///
    /// # Errors
    ///
    /// The field does not start.
    pub fn clicked_row(&mut self, out: &mut dyn Outside, frame: &Frame) -> Result<(), String> {
        let token = out.token();
        self.show(out, None, token.as_deref(), frame)
    }

    /// A click on entry `index` (from 1) of the list; a project starts
    /// its helper first.
    ///
    /// # Errors
    ///
    /// The field does not start.
    pub fn clicked_pick(
        &mut self,
        out: &mut dyn Outside,
        index: usize,
        frame: &Frame,
    ) -> Result<(), String> {
        let token = out.token();
        self.pick(out, index, token.as_deref(), frame)
    }

    fn pick(
        &mut self,
        out: &mut dyn Outside,
        index: usize,
        token: Option<&str>,
        frame: &Frame,
    ) -> Result<(), String> {
        let Mode::List(picks) = &self.mode else {
            return Ok(());
        };
        let Some(pick) = index.checked_sub(1).and_then(|n| picks.get(n)).cloned() else {
            return Ok(());
        };
        if let Pick::Project(project) = &pick {
            let start = json!({"tool": "agent_start", "project": project, "name": project});
            let _ = out.voice("agent", start);
        }
        self.show(out, Some(pick.name()), token, frame)
    }

    /// Closes the view and the field.
    pub fn hide(&mut self, out: &mut dyn Outside) {
        self.close_chat(out);
        self.mode = Mode::Closed;
        self.keyboard = false;
        self.spoken = None;
        if let Some(link) = self.link.as_mut() {
            link.hide();
        }
    }

    fn close_view(&mut self, out: &mut dyn Outside) {
        self.close_chat(out);
        self.mode = Mode::Closed;
        self.keyboard = false;
        self.spoken = None;
    }

    /// The same chat in the look of another folder or on another output:
    /// the field moves, it does not restart.
    pub fn restyle(&mut self, frame: &Frame) {
        if !self.is_open() {
            return;
        }
        let to = self.helper().map(str::to_owned);
        if let Some(link) = self.link.as_mut() {
            link.place(&field_place_for(frame, None, to.as_deref()));
        }
    }

    /// Takes what the field told. True when the view changed.
    pub fn field_events(&mut self, out: &mut dyn Outside, frame: &Frame) -> bool {
        if self.link.as_mut().is_some_and(|link| !link.alive()) {
            self.link = None;
            self.keyboard = false;
            if self.is_open() {
                let _ = self.place_field(out, frame, None);
            }
            return true;
        }
        let events = self.link.as_mut().map(|link| link.events());
        let mut changed = false;
        for event in events.into_iter().flatten() {
            match event {
                FieldEvent::Say(text) => changed |= self.said(out, &text, frame),
                FieldEvent::Shown => {}
                FieldEvent::Height(height) => {
                    changed |= self.field_height != Some(height);
                    self.field_height = Some(height);
                }
                FieldEvent::Keyboard(on) => self.keyboard = on,
                FieldEvent::Closed => {
                    self.close_view(out);
                    changed = true;
                }
                FieldEvent::Stop => changed |= self.cancel(out),
            }
        }
        changed
    }

    /// Stops the running prompt of the helper: voice is asked, and the
    /// header stops saying that it works until voice says otherwise. True
    /// when the view changed.
    pub fn cancel(&mut self, out: &mut dyn Outside) -> bool {
        let Mode::Chat(name) = &self.mode else {
            return false;
        };
        let _ = out.voice("agent.cancel", json!({"name": name}));
        "idle".clone_into(&mut self.status.state);
        self.status.task.clear();
        self.status.steps = None;
        true
    }

    fn said(&mut self, out: &mut dyn Outside, text: &str, frame: &Frame) -> bool {
        match &self.mode {
            Mode::Chat(name) => {
                let sent = out.voice("agent", say_params(name, text));
                let refused = sent
                    .as_ref()
                    .map_or(true, |answer| answer.as_str().is_some_and(is_error));
                if refused && let Some(link) = self.link.as_mut() {
                    link.say_failed(text);
                }
                if !refused {
                    self.sent.push((sanitize(text), self.transcript.after()));
                }
                self.pull(out);
                true
            }
            Mode::List(picks) => {
                let text = text.trim();
                let index = text.parse::<usize>().ok().or_else(|| {
                    picks
                        .iter()
                        .position(|pick| pick.name().eq_ignore_ascii_case(text))
                        .map(|n| n + 1)
                });
                if let Some(index) = index {
                    return self.pick(out, index, None, frame).is_ok();
                }
                if let Some(link) = self.link.as_mut() {
                    link.pick_first(text);
                }
                false
            }
            Mode::Closed => false,
        }
    }

    /// A voice event; the view is marked for drawing if it changed.
    pub fn voice_event(&mut self, out: &mut dyn Outside, event: &str, data: &Value) {
        if event == "voice.remove"
            && let Some(name) = data["name"].as_str()
        {
            self.states.remove(&name.to_lowercase());
            if self
                .helper()
                .is_some_and(|open| open.eq_ignore_ascii_case(name))
            {
                self.status.state = "done".into();
                self.status.task.clear();
                self.status.steps = None;
                self.dirty = true;
            }
            return;
        }
        if event == "voice.state"
            && let Some(name) = data["name"].as_str()
            && (self.states.len() < MAX_STATES || self.states.contains_key(&name.to_lowercase()))
        {
            self.states.insert(name.to_lowercase(), data.clone());
        }
        let Some(helper) = self.helper().map(str::to_owned) else {
            return;
        };
        if let Some(link) = self.link.as_mut() {
            link.voice_event(event, data, &helper);
        }
        let changed = match event {
            "voice.state" if names(data, &helper) => self.helper_state(out, data),
            "voice.confirm" => {
                if let Some(question) = Question::from_event(data, None) {
                    self.keys = question.keys;
                }
                self.card.is_some()
            }
            _ => false,
        };
        self.dirty |= changed;
    }

    fn helper_state(&mut self, out: &mut dyn Outside, data: &Value) -> bool {
        let text = |key: &str| data[key].as_str().unwrap_or_default();
        let steps = data["step"].as_u64().zip(data["steps"].as_u64());
        let status = Status {
            name: sanitize(text("name")),
            state: text("state").to_owned(),
            task: sanitize(text("task")),
            steps,
        };
        let mut changed = status.state != self.status.state
            || status.task != self.status.task
            || status.steps != self.status.steps;
        self.status = status;
        let epoch = data["epoch"].as_u64();
        if epoch.is_some() && self.epoch.is_some() && epoch != self.epoch {
            self.transcript = Transcript::default();
            changed = true;
        }
        self.epoch = epoch.or(self.epoch);
        self.head = data["seq"].as_u64().unwrap_or(self.head);
        if self.head > self.transcript.after() {
            changed |= self.pull(out);
        }
        changed
    }

    /// A question appeared: where its card goes, fixed until it ends.
    pub fn question_asked(&mut self) -> Where {
        self.answered = Answered::No;
        if let Some(link) = self.link.as_mut() {
            link.question(true);
        }
        let place = if self.is_open() {
            Where::Chat
        } else {
            Where::Figure
        };
        *self.card.insert(place)
    }

    /// The question ended.
    pub fn question_ended(&mut self) {
        self.answered = Answered::Yes;
        if let Some(link) = self.link.as_mut() {
            link.question(false);
        }
        self.card = None;
        self.card_height = None;
        self.card_top = None;
        self.dirty = true;
    }

    /// Where the card of the question that waits stays.
    #[cfg(test)]
    #[must_use]
    pub fn card_place(&self) -> Option<Where> {
        self.card
    }

    fn label(&self) -> String {
        match &self.mode {
            Mode::Chat(name) if self.status.name.is_empty() => name.clone(),
            _ => self.status.name.clone(),
        }
    }

    /// The header line of the view.
    #[must_use]
    pub fn header(&self, language: Language) -> String {
        match &self.mode {
            Mode::Closed => String::new(),
            _ if self.card == Some(Where::Figure) => say(language, Word::Waiting, &[]),
            _ if self.offline => say(language, Word::Offline, &[]),
            Mode::List(picks) if picks.is_empty() => say(language, Word::NoHelper, &[]),
            Mode::List(_) => say(language, Word::ChatLauncher, &[]),
            Mode::Chat(_) => {
                let row = words::row(
                    language,
                    &self.label(),
                    &self.status.state,
                    &self.status.task,
                );
                match self.status.steps {
                    Some((step, steps)) => format!("{row} ({step}/{steps})"),
                    None => row,
                }
            }
        }
    }

    /// The numbered entries of the list, empty outside of it.
    #[must_use]
    pub fn entries(&self, language: Language) -> Vec<String> {
        match &self.mode {
            Mode::List(picks) => (1..)
                .zip(picks)
                .map(|(n, pick)| format!("{n}  {}", self.entry(pick, language)))
                .collect(),
            _ => Vec::new(),
        }
    }

    /// The words of an entry of the list: a helper by its name as voice
    /// writes it, with its state word when voice named one.
    fn entry(&self, pick: &Pick, language: Language) -> String {
        let Pick::Agent(name) = pick else {
            return pick.label(language);
        };
        let Some(data) = self.states.get(&name.to_lowercase()) else {
            return name.clone();
        };
        let written = data["name"].as_str().map_or_else(|| name.clone(), sanitize);
        match data["state"].as_str().filter(|state| !state.is_empty()) {
            Some(state) => words::row(language, &written, state, ""),
            None => written,
        }
    }

    /// The hint above the field: while a question waits, that typing does
    /// not answer, with its keys unless the card in the view names them.
    #[must_use]
    pub fn hint(&self, language: Language) -> Option<String> {
        let waits = match self.mode {
            Mode::Chat(_) => self.waiting(),
            Mode::List(_) => self.card.is_some(),
            Mode::Closed => false,
        };
        waits.then(|| {
            if self.card == Some(Where::Chat) {
                return say(language, Word::TypingNoAnswer, &[]);
            }
            let (yes, no) = (show_key(&self.keys.yes), show_key(&self.keys.no));
            say(language, Word::TypedYes, &[("yes", &yes), ("no", &no)])
        })
    }

    /// The transcript as lines, none outside of a chat with a helper, and
    /// then what the agent said into the open balloon.
    pub fn lines(&mut self, language: Language, measure: &Measure) -> Vec<Line> {
        let mut lines = if matches!(self.mode, Mode::Chat(_)) {
            let label = self.label();
            self.transcript.lines(language, &label, measure)
        } else {
            Vec::new()
        };
        if matches!(self.mode, Mode::Chat(_)) {
            lines.extend(self.echoes(language, measure));
        }
        let spoken = self
            .spoken
            .as_deref()
            .map(|text| wrap(text, measure.max, measure.width));
        lines.extend(spoken.into_iter().flatten().map(|text| Line {
            role: Role::Notice,
            label: None,
            text,
        }));
        lines
    }

    /// The lines of what the person sent that the transcript does not hold
    /// yet: as the transcript will show it, the label over the first line.
    fn echoes(&self, language: Language, measure: &Measure) -> Vec<Line> {
        let label = say(language, Word::You, &[]);
        let mut lines = Vec::new();
        for (text, _) in &self.sent {
            for (n, part) in wrap(text, measure.max, measure.width)
                .into_iter()
                .enumerate()
            {
                lines.push(Line {
                    role: Role::User,
                    label: (n == 0).then(|| label.clone()),
                    text: part,
                });
            }
        }
        lines
    }

    /// Drops what the person sent once the transcript holds it.
    fn settle_sent(&mut self) {
        let entries = self.transcript.entries();
        self.sent.retain(|(text, after)| {
            !entries.iter().any(|entry| {
                entry.role == Role::User
                    && entry.seq.is_some_and(|seq| seq > *after)
                    && entry.text == *text
            })
        });
    }

    /// The height of the field: as it told it, else one line.
    #[must_use]
    pub fn field_height(&self) -> i32 {
        self.field_height.unwrap_or(ONE_LINE_FIELD)
    }

    /// The card of the question stands in the view, this high, its top at
    /// `top` on the output.
    pub fn card_shown(&mut self, top: i32, height: i32) {
        self.card_height = Some(height);
        self.card_top = Some(top);
        self.dirty = true;
    }

    /// The top of the card in the view, on the output.
    #[must_use]
    pub fn card_top(&self) -> Option<i32> {
        self.card_top.filter(|_| self.card == Some(Where::Chat))
    }

    /// The room that the card takes in the view, with its gap.
    #[must_use]
    pub fn card_room(&self) -> i32 {
        self.card_height
            .filter(|_| self.card == Some(Where::Chat))
            .map_or(0, |height| height + view::GAP)
    }

    /// A sentence of the agent, which shows in the view while it is the
    /// open balloon of the figure, until the view closes.
    pub fn agent_said(&mut self, text: &str) {
        self.spoken = Some(sanitize(text));
        self.dirty = true;
    }

    /// Asks voice again after it did not answer: the chat opens again and
    /// the transcript is pulled. True when the view changed.
    pub fn retry(&mut self, out: &mut dyn Outside) -> bool {
        if !self.offline {
            return false;
        }
        match &self.mode {
            Mode::Chat(name) => {
                let name = name.clone();
                self.announce_open(out, &name);
            }
            Mode::List(_) => self.open_list(out),
            Mode::Closed => return false,
        }
        !self.offline
    }

    /// Pulls the transcript from voice, until it is up to date. True when
    /// it grew or when voice stopped answering.
    pub fn pull(&mut self, out: &mut dyn Outside) -> bool {
        let Some(name) = self.helper().map(str::to_owned) else {
            return false;
        };
        let before = (self.transcript.after(), self.offline);
        for _ in 0..PULLS {
            let after = self.transcript.after();
            let params = json!({"name": name, "after": after});
            let answer = match out.voice("agent.read", params) {
                Ok(answer) => answer,
                Err(error) => {
                    self.refused_or_offline(&error);
                    break;
                }
            };
            self.offline = false;
            self.epoch = answer["epoch"].as_u64().or(self.epoch);
            self.transcript.apply(&answer);
            self.settle_sent();
            if self.transcript.after() == after || self.transcript.after() >= self.head {
                break;
            }
        }
        before != (self.transcript.after(), self.offline)
    }
}

fn names(data: &Value, helper: &str) -> bool {
    data["name"]
        .as_str()
        .is_some_and(|name| name.eq_ignore_ascii_case(helper))
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc, time::Instant};

    use chora_kit::chat::Role;
    use serde_json::json;

    use super::*;
    use crate::context::Switcher;

    #[test]
    fn the_last_helper_reopens_when_it_is_the_only_one() {
        let mut session = Session::new();
        let one = [Pick::Agent("Atlas".into()), Pick::Project("notes".into())];
        assert_eq!(session.only_helper(&one), None, "no chat was open yet");
        session.last = Some("atlas".into());
        assert_eq!(session.only_helper(&one).as_deref(), Some("Atlas"));
        let two = [Pick::Agent("Atlas".into()), Pick::Agent("Vega".into())];
        assert_eq!(session.only_helper(&two), None, "two helpers: a list");
        session.last = Some("Vega".into());
        assert_eq!(session.only_helper(&one), None, "the last one is gone");
    }

    #[derive(Default)]
    struct Log {
        calls: Vec<(String, Value)>,
        started: u32,
        places: Vec<Value>,
        questions: Vec<bool>,
        failed: Vec<String>,
        unpicked: Vec<String>,
        events: Vec<FieldEvent>,
        tokens: u32,
        read: Vec<Value>,
        down: bool,
        open_error: Option<String>,
        dead: bool,
        hides: u32,
        clears: u32,
    }

    struct Fake(Rc<RefCell<Log>>);
    struct FakeLink(Rc<RefCell<Log>>);

    impl Outside for Fake {
        fn voice(&mut self, method: &str, params: Value) -> Result<Value, String> {
            let mut log = self.0.borrow_mut();
            log.calls.push((method.to_owned(), params.clone()));
            if log.down {
                return Err("voice does not answer".into());
            }
            if method == "chat.open"
                && let Some(error) = log.open_error.clone()
            {
                return Err(error);
            }
            Ok(match method {
                "agent" if params["tool"] == "agent_list" => {
                    json!("projects: demo, notes; agents: atlas (demo, idle)")
                }
                "agent.read" if !log.read.is_empty() => log.read.remove(0),
                "agent.read" => json!({"entries": [], "seq": 0, "epoch": 1}),
                _ => json!({}),
            })
        }

        fn token(&mut self) -> Option<String> {
            let mut log = self.0.borrow_mut();
            log.tokens += 1;
            Some(format!("act-{}", log.tokens))
        }

        fn start_field(&mut self) -> Result<Box<dyn Link>, String> {
            self.0.borrow_mut().started += 1;
            Ok(Box::new(FakeLink(self.0.clone())))
        }
    }

    impl Link for FakeLink {
        fn place(&mut self, place: &Value) {
            self.0.borrow_mut().places.push(place.clone());
        }

        fn question(&mut self, waiting: bool) {
            self.0.borrow_mut().questions.push(waiting);
        }

        fn say_failed(&mut self, text: &str) {
            self.0.borrow_mut().failed.push(text.to_owned());
        }

        fn pick_first(&mut self, text: &str) {
            self.0.borrow_mut().unpicked.push(text.to_owned());
        }

        fn voice_event(&mut self, _: &str, _: &Value, _: &str) {}

        fn events(&mut self) -> Vec<FieldEvent> {
            std::mem::take(&mut self.0.borrow_mut().events)
        }

        fn alive(&mut self) -> bool {
            !self.0.borrow().dead
        }

        fn hide(&mut self) {
            self.0.borrow_mut().hides += 1;
        }

        fn clear(&mut self) {
            self.0.borrow_mut().clears += 1;
        }
    }

    fn frame(look: Look) -> Frame {
        Frame {
            output: "HEADLESS-1".into(),
            size: [1280, 720],
            area: [0, 0, 1280, 720],
            look,
            slot: None,
            clear: Vec::new(),
            ring: None,
            dark: false,
            scale: 1,
            card: None,
            tip: None,
            field: ONE_LINE_FIELD,
        }
    }

    #[test]
    fn a_glass_chat_is_28_hundredths_of_the_output_wide_from_400_to_its_width() {
        let look = Look {
            style: chora_kit::hud::ChatStyle::Glass,
            anchor: Some(Anchor::Right),
            width: Some(480),
            margin: 32,
            ..Look::default()
        };
        let mut glass = frame(look);
        assert_eq!(glass.rect(), [1280 - 32 - 400, 32, 400, 656]);
        glass.area = [0, 0, 1920, 1080];
        assert_eq!(glass.rect()[2], 480, "28 hundredths of 1920 is more");
        glass.area = [0, 0, 1600, 900];
        assert_eq!(glass.rect()[2], 448);
    }

    #[test]
    fn a_bubble_chat_is_the_balloon_of_the_figure_with_its_tail_at_it() {
        let look = Look {
            style: chora_kit::hud::ChatStyle::Bubble,
            anchor: Some(Anchor::BottomRight),
            width: Some(340),
            ..Look::default()
        };
        let mut balloon = frame(look);
        balloon.area = [0, 0, 1280, 690];
        balloon.tip = Some([1144, 542]);
        balloon.clear = vec![[1104, 542, 80, 116]];
        let [x, y, w, h] = balloon.rect();
        assert_eq!(w, 340);
        assert_eq!(y + h + TAIL, 542, "the tail ends at the head of the figure");
        assert!(x >= 0 && x + w <= 1280 && y >= 0, "inside the work area");
        let tail = balloon.tail().expect("a tail");
        assert_eq!(x + tail, 1144, "the tip of the tail over the head");
        let low = Frame {
            tip: Some([1144, 100]),
            ..balloon.clone()
        };
        assert!(
            low.rect()[1] >= 0 && low.rect()[3] > 0,
            "a figure high up still fits"
        );
        assert_eq!(frame(Look::default()).tail(), None, "a panel has no tail");
    }

    #[test]
    fn the_view_stands_beside_the_figure_and_its_balloon() {
        let look = Look {
            anchor: Some(Anchor::BottomRight),
            width: Some(340),
            ..Look::default()
        };
        let mut frame = frame(look);
        frame.area = [0, 0, 1280, 690];
        assert_eq!(frame.rect(), [940, 276, 340, 414]);
        frame.clear = vec![[1024, 430, 240, 404], [1106, 642, 70, 32]];
        assert_eq!(frame.rect(), [1024 - ASIDE - 340, 276, 340, 414]);
        frame.clear = vec![[1024, 0, 240, 100]];
        assert_eq!(frame.rect()[0], 940, "nothing in the way");
        let side = Look {
            anchor: Some(Anchor::Right),
            ..Look::default()
        };
        let mut panel = self::frame(side);
        panel.clear = vec![[1240, 680, 24, 24]];
        assert_eq!(
            panel.rect(),
            [880, 0, 400, 720],
            "a side panel keeps its edge"
        );
    }

    #[test]
    fn a_press_in_the_view_gives_the_field_the_keyboard_back_with_its_token() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session.refocus(&mut out, &frame, Some("act-0"));
        assert!(log.borrow().places.is_empty(), "a closed view has no field");
        session
            .show(&mut out, Some("atlas"), Some("act-1"), &frame)
            .unwrap();
        session.refocus(&mut out, &frame, Some("act-2"));
        assert_eq!(log.borrow().places.last().unwrap()["token"], "act-2");
        session.refocus(&mut out, &frame, None);
        assert!(log.borrow().places.last().unwrap().get("token").is_none());
    }

    #[test]
    fn a_slot_box_stays_in_the_work_area_and_in_the_limits_of_the_chat() {
        let mut frame = frame(Look::default());
        frame.area = [0, 24, 1280, 696];
        frame.slot = Some([-500, -500, 100_000, 100_000]);
        assert_eq!(frame.rect(), [0, 24, 1280, 696], "clipped to the work area");
        frame.area = [0, 0, 3840, 2160];
        frame.slot = Some([0, 0, 3840, 400]);
        let width = i32::try_from(chora_kit::hud::MAX_CHAT_WIDTH).unwrap();
        assert_eq!(frame.rect(), [0, 0, width, 400], "no wider than a chat");
        frame.area = [0, 0, 1280, 720];
        frame.slot = Some([5000, 5000, 300, 300]);
        assert_eq!(
            frame.rect(),
            [880, 0, 400, 720],
            "a box off the area is no box"
        );
        frame.slot = Some([40, 400, 600, 280]);
        frame.clear = vec![[500, 500, 100, 100]];
        assert_eq!(
            frame.rect(),
            [40, 400, 600, 280],
            "the box of a slot is the view, whatever stands in it"
        );
    }

    #[test]
    fn a_box_holds_only_the_view_of_its_client_and_output() {
        let held = ("mc".to_owned(), "DP-1".to_owned(), [1, 2, 3, 4]);
        assert_eq!(
            held_box(Some(&held), Some("mc"), "DP-1"),
            Some([1, 2, 3, 4])
        );
        assert_eq!(held_box(Some(&held), Some("xp"), "DP-1"), None);
        assert_eq!(held_box(Some(&held), None, "DP-1"), None);
        assert_eq!(held_box(Some(&held), Some("mc"), "DP-2"), None);
        assert_eq!(held_box(None, Some("mc"), "DP-1"), None);
    }

    #[test]
    fn a_box_with_a_side_that_is_no_number_is_refused() {
        let event = ("mc.chat.slot", "mc");
        for bad in [
            json!([0, 0, "wide", 10]),
            json!([0, 0, 10, 10, "x"]),
            json!([0, 0, 10, 10, 5]),
            json!([0, null, 10, 10]),
            json!([0, 0, 1e300, 10]),
            json!({"x": 0}),
        ] {
            let data = json!({"output": "HEADLESS-1", "rect": bad});
            assert_eq!(slot_box(Some("mc"), event, &data), None, "{bad}");
        }
    }

    #[test]
    fn the_view_fills_the_box_of_its_slot() {
        let look = Look {
            slot: Some("mc".into()),
            ..Look::default()
        };
        let mut frame = frame(look);
        assert_eq!(
            frame.rect(),
            [880, 0, 400, 720],
            "no box: the default place"
        );
        frame.slot = Some([40, 400, 1200, 280]);
        assert_eq!(frame.rect(), [40, 400, 1200, 280]);
        let place = field_place_for(&frame, None, Some("Atlas"));
        assert_eq!(place["width"], 1206, "the inset of the caret on the left");
        assert_eq!(place["margin"], json!([0, 0, 40, 34]));
        assert_eq!(
            place["scale"], frame.scale,
            "sharp at the scale of the output"
        );
        assert_eq!(place["look"]["column"], json!({"inset": 6, "to": "Atlas"}));
    }

    #[test]
    fn the_field_of_a_plain_chat_has_a_muted_hint_and_no_scale() {
        let mut frame = frame(Look {
            text: Some([41, 38, 36]),
            name: Some([110, 105, 99]),
            ..Look::default()
        });
        frame.scale = 2;
        let place = field_place_for(&frame, None, None);
        assert!(place.get("scale").is_none(), "the plain field paints at 1");
        assert_eq!(place["look"]["hint"], json!([110, 105, 99, 255]));
        assert_eq!(place["look"]["text"], json!([41, 38, 36, 255]));
    }

    #[test]
    fn the_margin_keeps_the_view_from_the_edges() {
        let look = Look {
            anchor: Some(Anchor::Right),
            width: Some(480),
            margin: 32,
            ..Look::default()
        };
        let mut frame = frame(look);
        frame.area = [0, 24, 1280, 696];
        assert_eq!(frame.rect(), [768, 56, 480, 632]);
    }

    #[test]
    fn only_the_named_client_fills_the_slot() {
        let data = json!({"output": "HEADLESS-1", "rect": [40.0, 400.4, 1200.0, 280.0]});
        let event = ("mc.chat.slot", "mc");
        let here = "HEADLESS-1".to_owned();
        assert_eq!(
            slot_box(Some("mc"), event, &data),
            Some((here.clone(), Some([40, 400, 1200, 280])))
        );
        let gone = json!({"output": "HEADLESS-1", "rect": null});
        assert_eq!(slot_box(Some("mc"), event, &gone), Some((here, None)));
        assert_eq!(slot_box(None, event, &data), None);
        assert_eq!(slot_box(Some("xp"), event, &data), None);
        let forged = ("mc.chat.slot", "mc.evil");
        assert_eq!(slot_box(Some("mc"), forged, &data), None);
        let other = ("mc.chat.other", "mc");
        assert_eq!(slot_box(Some("mc"), other, &data), None);
        let flat = json!({"output": "HEADLESS-1", "rect": [0, 0, -5, 10]});
        assert_eq!(slot_box(Some("mc"), event, &flat), None);
        let short = json!({"output": "HEADLESS-1", "rect": [0, 0, 5]});
        assert_eq!(slot_box(Some("mc"), event, &short), None);
    }

    fn setup() -> (Session, Fake, Rc<RefCell<Log>>) {
        let log = Rc::new(RefCell::new(Log::default()));
        (Session::new(), Fake(log.clone()), log)
    }

    fn calls(log: &Rc<RefCell<Log>>, method: &str) -> Vec<Value> {
        let log = log.borrow();
        log.calls
            .iter()
            .filter(|(m, _)| m == method)
            .map(|(_, p)| p.clone())
            .collect()
    }

    fn measure() -> Measure<'static> {
        Measure {
            max: 40.0,
            font: 0,
            width: &|_| 1.0,
        }
    }

    #[test]
    fn a_click_on_the_chat_row_opens_the_chat_with_a_token() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session.clicked_row(&mut out, &frame).expect("the list");
        assert_eq!(
            session.mode(),
            &Mode::List(vec![
                Pick::Agent("atlas".into()),
                Pick::Project("notes".into())
            ])
        );
        assert_eq!(
            log.borrow().started,
            1,
            "the field takes the keys of the list"
        );
        session.clicked_pick(&mut out, 1, &frame).expect("the chat");
        assert_eq!(session.mode(), &Mode::Chat("atlas".into()));
        assert_eq!(calls(&log, "chat.open"), [json!({"name": "atlas"})]);
        let log = log.borrow();
        let place = log.places.last().expect("a place");
        assert_eq!(
            place["token"], "act-2",
            "the token of the click on the entry"
        );
        assert_eq!(place["width"], 400);
        assert_eq!(place["output"], "HEADLESS-1");
        assert_eq!(place["anchor"], "bottom_left");
        assert_eq!(place["margin"], json!([0, 0, 0, 880]));
    }

    #[test]
    fn the_chat_follows_a_switch_of_look_and_keeps_the_draft() {
        let (mut session, mut out, log) = setup();
        let neutral = frame(Look::default());
        session
            .show(&mut out, Some("atlas"), Some("act-1"), &neutral)
            .expect("open");
        let answer = json!({"entries": [{"seq": 1, "role": "agent", "text": "Hello"}],
            "seq": 1, "epoch": 1});
        log.borrow_mut().read.push(answer);
        assert!(session.pull(&mut out));
        let before = session.lines(Language::En, &measure());
        let glass = frame(Look {
            width: Some(480),
            anchor: Some(Anchor::Right),
            ..Look::default()
        });
        session.restyle(&glass);
        let log = log.borrow();
        assert_eq!(log.started, 1, "the same field process keeps the draft");
        assert_eq!(log.places.len(), 2);
        assert_eq!(log.places[1]["width"], 480);
        assert!(log.calls.iter().all(|(m, _)| m != "chat.close"));
        assert_eq!(session.mode(), &Mode::Chat("atlas".into()));
        drop(log);
        assert_eq!(session.lines(Language::En, &measure()), before);
    }

    #[test]
    fn the_context_stays_while_the_field_has_the_keyboard() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session
            .show(&mut out, Some("atlas"), None, &frame)
            .expect("open");
        assert!(!session.keyboard());
        log.borrow_mut().events.push(FieldEvent::Keyboard(true));
        session.field_events(&mut out, &frame);
        assert!(session.keyboard());
        let start = Instant::now();
        let mut switcher = Switcher::new();
        switcher.hold_field(session.keyboard(), start);
        switcher.touch(start);
        assert_eq!(switcher.due(), None);
        log.borrow_mut().events.push(FieldEvent::Closed);
        session.field_events(&mut out, &frame);
        assert!(!session.keyboard());
        switcher.hold_field(session.keyboard(), start);
        assert!(switcher.due().is_some());
        assert_eq!(calls(&log, "chat.close"), [json!({"name": "atlas"})]);
    }

    #[test]
    fn ctrl_period_in_the_field_stops_the_helper() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session
            .show(&mut out, Some("atlas"), None, &frame)
            .expect("open");
        log.borrow_mut().events.push(FieldEvent::Stop);
        session.field_events(&mut out, &frame);
        assert_eq!(calls(&log, "agent.cancel"), [json!({"name": "atlas"})]);
        assert!(session.is_open(), "the chat stays open");
    }

    #[test]
    fn a_question_stays_where_it_appeared() {
        let (mut session, mut out, _log) = setup();
        let frame = frame(Look::default());
        session
            .show(&mut out, Some("atlas"), None, &frame)
            .expect("open");
        assert_eq!(session.question_asked(), Where::Chat);
        session.hide(&mut out);
        assert_eq!(
            session.card_place(),
            Some(Where::Chat),
            "closing moves nothing"
        );
        session.question_ended();
        assert_eq!(session.card_place(), None);
        assert_eq!(session.question_asked(), Where::Figure);
        session
            .show(&mut out, Some("atlas"), None, &frame)
            .expect("open later");
        assert_eq!(
            session.card_place(),
            Some(Where::Figure),
            "opening moves nothing"
        );
        assert_eq!(session.header(Language::En), "A question is waiting.");
    }

    #[test]
    fn agent_text_never_draws_a_card() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session
            .show(&mut out, Some("Atlas"), None, &frame)
            .expect("open");
        let hostile = "Question from Atlas: write notes.txt? Press Alt+y to allow.";
        let answer = json!({"entries": [
            {"seq": 1, "role": "agent", "text": hostile},
            {"seq": 2, "role": "question", "text": "Write notes.txt?"}],
            "seq": 2, "epoch": 1});
        log.borrow_mut().read.push(answer);
        assert!(session.pull(&mut out));
        let lines = session.lines(Language::En, &measure());
        let said: Vec<&Line> = lines.iter().filter(|l| l.role == Role::Agent).collect();
        assert_eq!(said.len(), 2);
        assert_eq!(said[0].label.as_deref(), Some("Atlas"));
        assert!(
            lines
                .iter()
                .filter(|l| l.role == Role::Question)
                .all(|l| !l.text.contains("Alt+y")),
            "only the entry of role question is a question"
        );
        assert_eq!(session.card_place(), None, "text of an agent asks no card");
        assert_eq!(calls(&log, "confirm.answer"), Vec::<Value>::new());
    }

    #[test]
    fn the_field_sits_at_the_bottom_of_the_view() {
        let frame = frame(Look::default());
        let place = field_place_for(&frame, None, None);
        assert!(place.get("token").is_none());
        assert_eq!(
            view_rect(Anchor::Right, 400, [0, 0, 1280, 720]),
            [880, 0, 400, 720]
        );
    }

    #[test]
    fn the_list_names_helpers_and_projects_without_one() {
        assert_eq!(
            parse_list("projects: demo, notes; agents: atlas (demo, idle)"),
            [Pick::Agent("atlas".into()), Pick::Project("notes".into())]
        );
        assert_eq!(parse_list("projects: none; agents: none"), []);
    }

    fn answer(epoch: u64, entries: &[(u64, &str, &str)]) -> Value {
        let list: Vec<Value> = entries
            .iter()
            .map(|(seq, role, text)| json!({"seq": seq, "role": role, "text": text}))
            .collect();
        json!({"entries": list, "seq": entries.last().map_or(0, |e| e.0), "epoch": epoch})
    }

    #[test]
    fn a_new_epoch_pulls_again_from_zero() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        log.borrow_mut()
            .read
            .push(answer(1, &[(1, "agent", "Hello")]));
        session.show(&mut out, Some("atlas"), None, &frame).unwrap();
        log.borrow_mut()
            .read
            .push(answer(2, &[(1, "agent", "Again")]));
        let state = json!({"name": "atlas", "state": "working", "seq": 1, "epoch": 2});
        session.voice_event(&mut out, "voice.state", &state);
        let reads = calls(&log, "agent.read");
        assert_eq!(reads.last().unwrap()["after"], 0);
        let texts: Vec<String> = session
            .lines(Language::En, &measure())
            .into_iter()
            .map(|l| l.text)
            .collect();
        assert_eq!(texts, ["Again"]);
    }

    #[test]
    fn a_sent_message_shows_at_once_and_the_transcript_takes_its_place() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session.show(&mut out, Some("atlas"), None, &frame).unwrap();
        log.borrow_mut()
            .events
            .push(FieldEvent::Say("Tidy up".into()));
        assert!(session.field_events(&mut out, &frame), "the view changes");
        let lines = session.lines(Language::En, &measure());
        let last = lines.last().expect("a line for the message");
        assert_eq!((last.role, last.text.as_str()), (Role::User, "Tidy up"));
        assert_eq!(last.label.as_deref(), Some("You"));
        log.borrow_mut()
            .read
            .push(answer(1, &[(1, "user", "Tidy up"), (2, "agent", "On it.")]));
        let state = json!({"name": "atlas", "state": "working", "seq": 2, "epoch": 1});
        session.voice_event(&mut out, "voice.state", &state);
        let texts: Vec<String> = session
            .lines(Language::En, &measure())
            .into_iter()
            .map(|l| l.text)
            .collect();
        assert_eq!(texts, ["Tidy up", "On it."], "said once, not twice");
    }

    #[test]
    fn a_refused_message_is_no_line_of_the_chat() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session.show(&mut out, Some("atlas"), None, &frame).unwrap();
        log.borrow_mut().down = true;
        log.borrow_mut().events.push(FieldEvent::Say("Lost".into()));
        session.field_events(&mut out, &frame);
        assert_eq!(session.lines(Language::En, &measure()).len(), 0);
    }

    #[test]
    fn stop_ends_the_working_of_the_header_until_voice_says_otherwise() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session.show(&mut out, Some("atlas"), None, &frame).unwrap();
        let state = json!({"name": "atlas", "state": "working", "task": "Tidy", "seq": 0});
        session.voice_event(&mut out, "voice.state", &state);
        assert!(session.header(Language::En).contains("working"));
        log.borrow_mut().events.push(FieldEvent::Stop);
        assert!(session.field_events(&mut out, &frame));
        assert!(
            !session.header(Language::En).contains("working"),
            "{}",
            session.header(Language::En)
        );
        assert_eq!(calls(&log, "agent.cancel").len(), 1);
        let again = json!({"name": "atlas", "state": "working", "task": "Tidy", "seq": 0});
        session.voice_event(&mut out, "voice.state", &again);
        assert!(session.header(Language::En).contains("working"));
    }

    #[test]
    fn a_text_that_matches_no_entry_asks_to_pick_a_helper_first() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session.clicked_row(&mut out, &frame).unwrap();
        log.borrow_mut().events.push(FieldEvent::Say("zzz".into()));
        session.field_events(&mut out, &frame);
        assert_eq!(log.borrow().unpicked, ["zzz"]);
        assert!(log.borrow().failed.is_empty(), "voice is not offline");
        assert!(!session.offline());
    }

    #[test]
    fn a_pick_by_keyboard_keeps_the_field_and_clears_its_draft() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session.clicked_row(&mut out, &frame).unwrap();
        log.borrow_mut().events.push(FieldEvent::Say("1".into()));
        session.field_events(&mut out, &frame);
        assert_eq!(session.helper(), Some("atlas"));
        assert_eq!(log.borrow().started, 1, "the field and its keyboard stay");
        assert_eq!(log.borrow().clears, 1);
        session
            .show(&mut out, Some("notes"), Some("act"), &frame)
            .unwrap();
        assert_eq!(log.borrow().started, 1);
        assert_eq!(log.borrow().clears, 2, "another helper gets no old draft");
    }

    #[test]
    fn the_list_opens_with_an_empty_field_so_a_number_picks() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session
            .show(&mut out, Some("atlas"), Some("act"), &frame)
            .unwrap();
        session.hide(&mut out);
        session.last = None;
        let before = log.borrow().clears;
        session.clicked_row(&mut out, &frame).unwrap();
        assert_eq!(
            log.borrow().clears,
            before + 1,
            "the draft stays out of the list"
        );
    }

    #[test]
    fn a_sentence_of_the_agent_shows_in_the_open_balloon() {
        let (mut session, mut out, _) = setup();
        let frame = frame(Look::default());
        session.clicked_row(&mut out, &frame).unwrap();
        session.agent_said("Atlas is writing your notes.");
        let lines = session.lines(Language::En, &measure());
        assert_eq!(
            lines.last().map(|l| l.text.as_str()),
            Some("Atlas is writing your notes.")
        );
        assert_eq!(lines.last().map(|l| l.role), Some(Role::Notice));
        session
            .show(&mut out, Some("atlas"), Some("act"), &frame)
            .unwrap();
        let lines = session.lines(Language::En, &measure());
        assert_eq!(
            lines.last().map(|l| l.text.as_str()),
            Some("Atlas is writing your notes.")
        );
        session.hide(&mut out);
        session.clicked_row(&mut out, &frame).unwrap();
        assert_eq!(
            session.lines(Language::En, &measure()),
            Vec::new(),
            "a new open starts empty"
        );
    }

    #[test]
    fn the_list_shows_no_line_of_the_last_chat() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session
            .show(&mut out, Some("atlas"), Some("act"), &frame)
            .unwrap();
        let answer = json!({"entries": [{"seq": 1, "role": "user", "text": "Hello"}],
            "seq": 1, "epoch": 1});
        log.borrow_mut().read.push(answer);
        assert!(session.pull(&mut out));
        session.hide(&mut out);
        session.last = None;
        session.clicked_row(&mut out, &frame).unwrap();
        assert_eq!(session.lines(Language::En, &measure()), Vec::new());
    }

    #[test]
    fn the_list_names_helpers_by_name_and_offers_a_new_one_for_a_project() {
        let (mut session, mut out, _) = setup();
        let frame = frame(Look::default());
        let state = json!({"name": "Atlas", "state": "working", "task": "Write file"});
        session.voice_event(&mut out, "voice.state", &state);
        session.clicked_row(&mut out, &frame).unwrap();
        assert_eq!(
            session.entries(Language::En),
            ["1  Atlas · working", "2  New helper for notes"],
            "the name as voice writes it, with its state"
        );
        assert_eq!(
            session.entries(Language::De)[1],
            "2  Neuer Helfer für notes"
        );
    }

    #[test]
    fn hide_keeps_the_field_and_its_draft() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session
            .show(&mut out, Some("atlas"), Some("t"), &frame)
            .unwrap();
        session.hide(&mut out);
        assert_eq!(log.borrow().hides, 1);
        session
            .show(&mut out, Some("atlas"), Some("t"), &frame)
            .unwrap();
        assert_eq!(log.borrow().started, 1);
    }

    #[test]
    fn a_bare_show_of_another_helper_leaves_the_chat_alone() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session
            .show(&mut out, Some("atlas"), Some("t"), &frame)
            .unwrap();
        session.show(&mut out, Some("coder"), None, &frame).unwrap();
        assert_eq!(session.mode(), &Mode::Chat("atlas".into()));
        assert_eq!(calls(&log, "chat.close"), Vec::<Value>::new());
        session
            .show(&mut out, Some("coder"), Some("t2"), &frame)
            .unwrap();
        assert_eq!(session.mode(), &Mode::Chat("coder".into()));
    }

    #[test]
    fn a_helper_that_is_unknown_gets_its_own_line() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        log.borrow_mut().open_error = Some("chat.open: no agent atlas".into());
        session.show(&mut out, Some("atlas"), None, &frame).unwrap();
        assert_eq!(session.header(Language::En), "atlas · blocked");
    }

    #[test]
    fn an_offline_chat_retries_and_opens_again() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        log.borrow_mut().down = true;
        session.show(&mut out, Some("atlas"), None, &frame).unwrap();
        assert_eq!(
            session.header(Language::En),
            "Cannot reach voice control. Trying again every 5 s."
        );
        log.borrow_mut().down = false;
        assert!(session.retry(&mut out));
        assert_eq!(calls(&log, "chat.open").len(), 2);
        assert_eq!(session.header(Language::En), "atlas · ");
    }

    #[test]
    fn a_field_that_crashed_is_started_again() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session.show(&mut out, Some("atlas"), None, &frame).unwrap();
        log.borrow_mut().dead = true;
        session.field_events(&mut out, &frame);
        assert_eq!(log.borrow().started, 2);
    }

    #[test]
    fn the_hint_names_only_the_question_of_its_helper() {
        let (mut session, mut out, _log) = setup();
        let frame = frame(Look::default());
        session.show(&mut out, Some("atlas"), None, &frame).unwrap();
        session.question_asked();
        assert_eq!(session.hint(Language::En), None);
        let state = json!({"name": "atlas", "state": "question"});
        session.voice_event(&mut out, "voice.state", &state);
        assert_eq!(
            session.hint(Language::En).as_deref(),
            Some("Typing does not answer."),
            "the card in the view names the keys"
        );
        session.question_ended();
        session.voice_event(&mut out, "voice.state", &state);
        assert_eq!(
            session.hint(Language::En),
            None,
            "a question that was answered leaves no hint before voice says so"
        );
        session.hide(&mut out);
        session.question_asked();
        session.show(&mut out, Some("atlas"), None, &frame).unwrap();
        assert!(
            session
                .hint(Language::En)
                .is_some_and(|h| h.contains("Alt+Y")),
            "a card elsewhere: the hint names the keys"
        );
    }

    #[test]
    fn a_second_question_in_an_open_chat_tells_the_field_to_wait() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session.show(&mut out, Some("atlas"), None, &frame).unwrap();
        session.question_asked();
        session.question_ended();
        assert_eq!(log.borrow().questions.last(), Some(&false));
        assert_eq!(session.question_asked(), Where::Chat);
        assert_eq!(log.borrow().questions.last(), Some(&true));
    }

    #[test]
    fn an_answered_question_leaves_the_field_without_its_note() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session.show(&mut out, Some("atlas"), None, &frame).unwrap();
        session.question_ended();
        assert_eq!(log.borrow().questions.last(), Some(&false));
    }

    #[test]
    fn the_transcript_leaves_room_for_the_card_in_the_view() {
        let (mut session, mut out, _log) = setup();
        let frame = frame(Look::default());
        session.show(&mut out, Some("atlas"), None, &frame).unwrap();
        assert_eq!(session.question_asked(), Where::Chat);
        session.card_shown(300, 150);
        assert_eq!(session.card_room(), 150 + view::GAP);
        session.question_ended();
        assert_eq!(session.card_room(), 0);
    }

    #[test]
    fn a_helper_that_ends_leaves_no_state() {
        let (mut session, mut out, _log) = setup();
        let frame = frame(Look::default());
        session.show(&mut out, Some("atlas"), None, &frame).unwrap();
        let state = json!({"name": "atlas", "state": "working"});
        session.voice_event(&mut out, "voice.state", &state);
        session.voice_event(&mut out, "voice.remove", &json!({"name": "atlas"}));
        assert!(session.states.is_empty());
        assert_eq!(session.header(Language::En), "atlas · done");
    }
}
