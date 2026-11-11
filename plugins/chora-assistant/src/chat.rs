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

use chora_kit::{
    ask::{Keys, Question, show_key},
    chat::{Chat as Transcript, Line, Measure, sanitize},
    hud::{Anchor, Chat as Look},
    words::{self, Language, Word, say},
};
use serde_json::{Value, json};

use crate::{
    assistant::origin,
    check::host_chat,
    field::{FieldEvent, say_params},
};

mod host;
mod view;

pub use host::{Announced, ShowParams, exported, token_of};
pub use view::card_origin;

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
}

impl Frame {
    /// The rectangle of the view: the box of the slot inside the work
    /// area, else the place of the `chat:` values, `margin` away from the
    /// edges of the work area; beside what the view must not cover.
    #[must_use]
    pub fn rect(&self) -> [i32; 4] {
        if let Some(slot) = self.slot.and_then(|slot| self.clipped(slot)) {
            return self.aside(slot);
        }
        let (anchor, width) = self.look.place();
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

/// The gap between the view and what it stands beside.
const ASIDE: i32 = 8;

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

fn rgba([r, g, b]: [u8; 3]) -> [u8; 4] {
    [r, g, b, 255]
}

/// The `place` line of the field for a frame, with the token if there is
/// one: at the bottom of the view, as wide as the view.
#[must_use]
pub fn field_place(frame: &Frame, token: Option<&str>) -> Value {
    let [x, y, w, h] = frame.rect();
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
            "hint": rgba(colours.text),
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
        self.status.state == "question"
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
            self.status = Status::default();
            self.head = 0;
            self.epoch = None;
            self.link = None;
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
        self.mode = Mode::List(picks);
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
        if let Some(link) = self.link.as_mut() {
            link.question(waiting);
            link.place(&field_place(frame, token));
        }
        Ok(())
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
        if let Some(link) = self.link.as_mut() {
            link.hide();
        }
    }

    fn close_view(&mut self, out: &mut dyn Outside) {
        self.close_chat(out);
        self.mode = Mode::Closed;
        self.keyboard = false;
    }

    /// The same chat in the look of another folder or on another output:
    /// the field moves, it does not restart.
    pub fn restyle(&mut self, frame: &Frame) {
        if !self.is_open() {
            return;
        }
        if let Some(link) = self.link.as_mut() {
            link.place(&field_place(frame, None));
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
                FieldEvent::Keyboard(on) => self.keyboard = on,
                FieldEvent::Closed => {
                    self.close_view(out);
                    changed = true;
                }
            }
        }
        changed
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
                self.pull(out)
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
                    link.say_failed(text);
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
        let place = if self.is_open() {
            Where::Chat
        } else {
            Where::Figure
        };
        *self.card.insert(place)
    }

    /// The question ended.
    pub fn question_ended(&mut self) {
        self.card = None;
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

    /// The hint above the field: while a question waits, that typing does
    /// not answer, with its keys.
    #[must_use]
    pub fn hint(&self, language: Language) -> Option<String> {
        let waits = match self.mode {
            Mode::Chat(_) => self.waiting(),
            Mode::List(_) => self.card.is_some(),
            Mode::Closed => false,
        };
        waits.then(|| {
            let (yes, no) = (show_key(&self.keys.yes), show_key(&self.keys.no));
            say(language, Word::TypedYes, &[("yes", &yes), ("no", &no)])
        })
    }

    /// The transcript as lines.
    pub fn lines(&mut self, language: Language, measure: &Measure) -> Vec<Line> {
        let label = self.label();
        self.transcript.lines(language, &label, measure)
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

    #[derive(Default)]
    struct Log {
        calls: Vec<(String, Value)>,
        started: u32,
        places: Vec<Value>,
        questions: Vec<bool>,
        failed: Vec<String>,
        events: Vec<FieldEvent>,
        tokens: u32,
        read: Vec<Value>,
        down: bool,
        open_error: Option<String>,
        dead: bool,
        hides: u32,
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
    }

    fn frame(look: Look) -> Frame {
        Frame {
            output: "HEADLESS-1".into(),
            size: [1280, 720],
            area: [0, 0, 1280, 720],
            look,
            slot: None,
            clear: Vec::new(),
        }
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
            [608, 400, 600, 280],
            "beside what it would cover"
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
        let place = field_place(&frame, None);
        assert_eq!(place["width"], 1200);
        assert_eq!(place["margin"], json!([0, 0, 40, 40]));
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
        let place = field_place(&frame, None);
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
    fn a_text_that_matches_no_entry_is_given_back() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session.clicked_row(&mut out, &frame).unwrap();
        log.borrow_mut().events.push(FieldEvent::Say("zzz".into()));
        session.field_events(&mut out, &frame);
        assert_eq!(log.borrow().failed, ["zzz"]);
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
        assert!(session.hint(Language::En).is_some());
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
