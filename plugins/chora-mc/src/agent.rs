//! The agents that report their state with `agent.set`, for the status
//! line of the regions and the agents scene, and the notifications of
//! `notify.push`.

use chora_scene::{
    Command, InputMode, NodeKind, PropsUpdate, Transform,
    ipc::{ClientError, RpcError},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    Mc,
    config::{Color, Style},
    invalid,
    region::{self, Content, Item, Put, Region},
    scene::{Panel, check_name},
};

/// The scene that shows the agents, one column each.
const SCENE: &str = "agents";

/// The most helpers that a side shows, one row each.
const MAX_ROWS: usize = 4;
/// The lines of the notifications: three, with a blank line between them.
const NOTIFY_LINES: usize = 3;
/// The last words of an agent that its column shows, at most.
const SAID_CHARS: usize = 600;

/// An agent as `agent.set` reports it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Agent {
    pub name: String,
    pub state: String,
    pub task: String,
    pub said: String,
    pub voice: String,
    /// The steps of its plan, each with a sign of its state.
    pub plan: Vec<String>,
    /// The question that waits for the user.
    pub question: String,
    /// The last lines of its transcript.
    pub log: Vec<String>,
    /// The time of the first report, as HH:MM.
    pub since: String,
}

/// The fields of `agent.set` that a state of the voice agent may carry; the
/// others, such as `seq` and `epoch` for the host, are not for mc.
pub const STATE_FIELDS: [&str; 8] = [
    "name", "state", "task", "said", "voice", "plan", "question", "log",
];

/// Parameters of `agent.set`; a missing field keeps its value.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetAgent {
    name: String,
    state: Option<String>,
    task: Option<String>,
    said: Option<String>,
    voice: Option<String>,
    plan: Option<Vec<String>>,
    question: Option<String>,
    log: Option<Vec<String>>,
}

/// Parameters of `agent.focus`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Focus {
    name: String,
}

/// Parameters of `agent.remove`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoveAgent {
    name: String,
}

/// Parameters of `notify.push`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Push {
    title: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    urgency: Urgency,
}

/// The event `voice.confirm`: the voice agent asks for a spoken
/// confirmation, or with `done` the question ended.
#[derive(Deserialize)]
pub struct Confirm {
    /// A string or a number: the agent sent numbers before.
    #[serde(deserialize_with = "id_text")]
    id: String,
    #[serde(default)]
    prompt: String,
    /// The whole command or value, uncut.
    #[serde(default)]
    detail: String,
    #[serde(default)]
    done: bool,
    /// A question about a security key: only `keys` or a button answer.
    #[serde(default)]
    physical: bool,
    #[serde(default)]
    keys: Option<Keys>,
    /// mc may show buttons that answer by `confirm.answer`.
    #[serde(default)]
    buttons: bool,
}

/// The keys of a physical question, such as `Alt+y`.
#[derive(Clone, Debug, Deserialize)]
pub struct Keys {
    yes: String,
    no: String,
}

/// The confirmation that shows: its id, whether it has buttons, and the
/// keys of a physical one.
#[derive(Clone, Debug)]
pub struct Confirming {
    id: String,
    buttons: bool,
    keys: Option<Keys>,
}

/// The group of the answers of a confirmation, over the regions.
const ANSWERS: &str = "mc-answers";
/// The click areas of the answers.
pub const ANSWER_YES: &str = "mc-answer-yes";
pub const ANSWER_NO: &str = "mc-answer-no";

impl Confirming {
    /// The parameters of `voice.confirm.answer` for a press on `node`:
    /// an answer of a confirmation with buttons. Only a pointer press on
    /// the node calls it; no method of mc does.
    pub fn answer(&self, node: &str, injected: bool) -> Option<Value> {
        if injected {
            return None;
        }
        let yes = match node {
            ANSWER_YES => true,
            ANSWER_NO => false,
            _ => return None,
        };
        self.buttons.then(|| json!({"id": self.id, "yes": yes}))
    }

    /// The answers as margin notes, right to left from the right edge:
    /// per answer its key in the style `margin` and its word in `body`,
    /// the word with a hairline under it while it takes a click.
    fn parts(&self) -> Vec<(String, &'static str, String)> {
        let key = |k: &str| match k.rsplit_once('+') {
            Some((m, c)) if c.chars().count() == 1 => format!("{m}+{}", c.to_uppercase()),
            _ => k.to_owned(),
        };
        [(true, "Ja"), (false, "Nein")]
            .into_iter()
            .map(|(yes, word)| {
                let id = if yes { ANSWER_YES } else { ANSWER_NO };
                let k = (self.keys.as_ref())
                    .map(|k| key(if yes { &k.yes } else { &k.no }))
                    .unwrap_or_default();
                (id.to_owned(), word, k)
            })
            .collect()
    }
}

/// An id as text, from a string or a number.
fn id_text<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Id {
        Text(String),
        Number(serde_json::Number),
    }
    Ok(match Id::deserialize(d)? {
        Id::Text(text) => text,
        Id::Number(n) => n.to_string(),
    })
}

/// As in the freedesktop notifications: critical breaks through in red.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Urgency {
    Low,
    #[default]
    Normal,
    Critical,
}

/// The content that stacks the notifications, newest on top.
const NOTIFY: &str = "notify";
/// The content of the last critical notification.
const URGENT: &str = "urgent";
/// The content of a spoken confirmation: its prompt as the title, its
/// detail in mono.
const CONFIRM: &str = "confirm";

/// Whom a `chat.show` was for, for its error line: never an empty name.
fn chat_who(name: Option<&str>) -> String {
    name.map_or_else(
        || "for the helper that the host chooses".to_owned(),
        |name| format!("with {name}"),
    )
}

impl Mc {
    /// Adds an agent or changes it.
    pub fn set_agent(&mut self, set: SetAgent) -> Result<(), RpcError> {
        check_name(&set.name).map_err(|e| invalid(format!("name: {e}")))?;
        if let Some(state) = &set.state
            && !self.settings.value.states.contains_key(state)
        {
            let known: Vec<&String> = self.settings.value.states.keys().collect();
            return Err(invalid(format!("no state {state}; one of {known:?}")));
        }
        let at = self.agents.iter().position(|a| a.name == set.name);
        let at = at.unwrap_or_else(|| {
            self.agents.push(Agent {
                name: set.name.clone(),
                state: "idle".into(),
                task: String::new(),
                said: String::new(),
                voice: String::new(),
                plan: Vec::new(),
                question: String::new(),
                log: Vec::new(),
                since: region::date("%H:%M"),
            });
            self.agents.len() - 1
        });
        let agent = &mut self.agents[at];
        let fields = [
            (&mut agent.state, set.state),
            (&mut agent.task, set.task),
            (&mut agent.said, set.said),
            (&mut agent.voice, set.voice),
            (&mut agent.question, set.question),
        ];
        for (field, new) in fields {
            if let Some(new) = new {
                *field = new;
            }
        }
        for (field, new) in [(&mut agent.plan, set.plan), (&mut agent.log, set.log)] {
            if let Some(new) = new {
                *field = new;
            }
        }
        self.show_agents()
    }

    pub fn remove_agent(&mut self, remove: &RemoveAgent) -> Result<(), RpcError> {
        let before = self.agents.len();
        self.agents.retain(|a| a.name != remove.name);
        if self.agents.len() == before {
            return Err(invalid(format!("no agent {}", remove.name)));
        }
        if self.agent_focus.as_ref() == Some(&remove.name) {
            self.agent_focus = None;
        }
        self.show_agents()
    }

    /// What `agent.list` answers.
    pub fn list_agents(&self) -> Value {
        serde_json::json!({"agents": self.agents})
    }

    /// The status line and the agents scene anew.
    fn show_agents(&mut self) -> Result<(), RpcError> {
        self.fill_agents()?;
        self.render().map_err(crate::rejected)
    }

    /// The sign and the name of each agent, as the status line has them.
    pub fn agent_line(&self) -> Vec<String> {
        let states = &self.settings.value.states;
        (self.agents.iter())
            .map(|a| {
                let sign = states.get(&a.state).map_or("", |s| s.sign.as_str());
                format!("{sign} {}", a.name)
            })
            .collect()
    }

    /// One item per agent, in the color of its state.
    pub fn agent_items(
        &self,
        region: Region,
        slot: i32,
        name: &str,
        content: &Content,
    ) -> Vec<Item> {
        if !region.is_line() {
            return self.agent_rows(region, slot, name, content);
        }
        let config = &self.settings.value;
        let id = format!("{}-{name}", self.region_id(region));
        let mut ty = self.content_type(content, false);
        let base = ty.color;
        (self.agents.iter().zip(self.agent_line()).enumerate())
            .map(|(i, (agent, text))| {
                let color = config.states.get(&agent.state).map(|s| &s.color);
                ty.color = color
                    .and_then(|c| config.colors().get(c))
                    .copied()
                    .unwrap_or(base);
                Item {
                    hit: Some(format!("agent:{}", agent.name)),
                    ..Item::text(format!("{id}-{i}"), slot, ty.props(&text), i == 0)
                }
            })
            .collect()
    }

    /// The rows of the helpers in a side: the sign in the color of the
    /// state, the name in `text`, and the state and the task in `muted`
    /// after it; at most four rows, then how many more there are.
    fn agent_rows(&self, region: Region, slot: i32, name: &str, content: &Content) -> Vec<Item> {
        let config = &self.settings.value;
        let id = format!("{}-{name}", self.region_id(region));
        let colors = config.colors();
        let muted = self.content_type(content, false);
        let color = |name: &str| colors.get(name).copied().unwrap_or(muted.color);
        let (mut sign, mut text) = (muted.clone(), muted.clone());
        text.color = color("text");
        let mut items = Vec::new();
        for (i, agent) in self.agents.iter().take(MAX_ROWS).enumerate() {
            let state = config.states.get(&agent.state);
            sign.color = state.map_or(muted.color, |s| color(&s.color));
            let mark = state.map_or("", |s| s.sign.as_str());
            let words = state
                .and_then(|s| s.label.as_deref())
                .unwrap_or(&agent.state);
            let detail = if agent.task.is_empty() {
                words.to_owned()
            } else {
                format!("{words} · {}", agent.task)
            };
            let row = |part: &str, put, x, props| Item {
                x,
                put,
                ..Item::text(format!("{id}-{i}-{part}"), slot, props, false)
            };
            items.push(Item {
                hit: Some(format!("agent:{}", agent.name)),
                ..row("sign", Put::Stack, 0.0, sign.props(mark))
            });
            items.push(row(
                "name",
                Put::Row,
                chora_kit::hud::LABEL_COLUMN,
                text.props(&agent.name),
            ));
            items.push(row(
                "state",
                Put::After,
                chora_kit::hud::LABEL_COLUMN + 60.0,
                muted.props(&detail),
            ));
        }
        let more = self.agents.len().saturating_sub(MAX_ROWS);
        if more > 0 {
            let props = muted.props(&format!("+{more} more"));
            items.push(Item {
                put: Put::Stack,
                x: chora_kit::hud::LABEL_COLUMN,
                ..Item::text(format!("{id}-more"), slot, props, false)
            });
        }
        if !items.is_empty() && !content.title.is_empty() {
            let head = self.content_type(content, true).props(&content.title);
            items.insert(0, Item::text(format!("{id}-title"), slot, head, true));
        } else if let Some(first) = items.first_mut() {
            first.first = true;
        }
        items
    }

    /// A click on the column of a helper in the agents scene asks the HUD
    /// host to show the chat with that helper. Answers whether the click
    /// was on such a column.
    pub fn click_agent(&mut self, node: &str, at: [f64; 2]) -> bool {
        if self.current.as_deref() != Some(SCENE) || node != crate::wheel_id(SCENE) {
            return false;
        }
        let Some(output) = &self.output else {
            return false;
        };
        let scroll = self.scroll.get(SCENE).copied().unwrap_or_default();
        let [x, y] = [
            at[0] - f64::from(output.position[0]),
            at[1] - f64::from(output.position[1]) + scroll,
        ];
        let shown: Vec<String> = match &self.agent_focus {
            Some(name) => vec![name.clone()],
            None => self.agents.iter().map(|a| a.name.clone()).collect(),
        };
        let Some(scene) = self.scenes.get(SCENE) else {
            return false;
        };
        let hit = scene.panels.iter().find_map(|panel| {
            let n: usize = panel.id.strip_prefix("agent-")?.parse().ok()?;
            let [px, py, pw, ph] = self.place(SCENE, panel)?;
            let inside = (px..px + pw).contains(&x) && (py..py + ph).contains(&y);
            inside
                .then(|| shown.get(n.checked_sub(1)?).cloned())
                .flatten()
        });
        let Some(name) = hit else {
            return false;
        };
        self.chat_with(Some(&name));
        true
    }

    /// Asks the HUD host to show the chat, with a helper or with the one
    /// that it chooses.
    pub fn chat_with(&mut self, name: Option<&str>) {
        let mut params = json!({});
        if let Some(name) = name {
            params["name"] = json!(name);
        }
        if let Some(token) = self.rt.conn.activation_token() {
            params["token"] = json!(token);
        }
        let call = json!({"client": "assistant", "method": "chat.show", "params": params});
        if let Err(err) = self.rt.conn.call::<_, Value>("methods.call", call) {
            eprintln!("chora-mc: chat.show {}: {err}", chat_who(name));
        }
    }

    /// Asks the HUD host to close the chat.
    pub fn chat_close(&mut self) {
        let call = json!({"client": "assistant", "method": "chat.hide", "params": {}});
        if let Err(err) = self.rt.conn.call::<_, Value>("methods.call", call) {
            eprintln!("chora-mc: chat.hide: {err}");
        }
    }

    /// Shows one agent alone in the agents scene, with its transcript in
    /// the other columns; an empty name shows all agents again.
    pub fn focus_agent(&mut self, focus: &Focus) -> Result<(), RpcError> {
        if focus.name.is_empty() {
            self.agent_focus = None;
        } else if self.agents.iter().any(|a| a.name == focus.name) {
            self.agent_focus = Some(focus.name.clone());
        } else {
            return Err(invalid(format!("no agent {}", focus.name)));
        }
        self.fill_agents()?;
        if self.scenes.contains_key(SCENE) {
            self.show(SCENE, false)?;
        }
        self.render().map_err(crate::rejected)
    }

    /// Fills the agents scene: the panel `agent-N` with the N-th agent,
    /// its state, its question, its task, its plan and its last words,
    /// and the panel `band-N` with its voice and its start. Panels
    /// without an agent are empty. With a focus, `agent-1` has that agent
    /// and the other `agent-N` its transcript, newest last.
    fn fill_agents(&mut self) -> Result<(), RpcError> {
        let running = self.agents.iter().filter(|a| a.state != "asleep").count();
        let focused = (self.agent_focus.as_ref())
            .and_then(|name| self.agents.iter().find(|a| a.name == *name))
            .cloned();
        let shown = focused
            .clone()
            .map_or_else(|| self.agents.clone(), |a| vec![a]);
        let Some(scene) = self.scenes.get_mut(SCENE) else {
            return Ok(());
        };
        scene.chapter.running_head = Some(match &focused {
            Some(agent) => format!("Agents  ·  {}", crate::title_of(&agent.name)),
            None => format!("Agents  ·  {running} running"),
        });
        let slots = (scene.panels.iter())
            .filter(|p| p.id.starts_with("agent-"))
            .count();
        let columns = slots.saturating_sub(1);
        let grid = scene.grid()[0];
        let count = (focused.as_ref()).map_or(shown.len(), |_| slots);
        let transcript = focused
            .as_ref()
            .map(|a| transcript(&a.log, columns))
            .unwrap_or_default();
        let mut panels = Vec::new();
        for panel in &scene.panels.clone() {
            let (kind, n) = panel.id.rsplit_once('-').unwrap_or_default();
            let Ok(n) = n.parse::<usize>() else {
                continue;
            };
            let agent = n.checked_sub(1).and_then(|i| shown.get(i));
            let panel = &Panel {
                cell: agent_cell(panel.cell, n, count.min(slots), grid),
                ..panel.clone()
            };
            let new = match (kind, agent) {
                ("agent", None) if focused.is_some() => Panel {
                    title: if n == 2 {
                        "Transcript".into()
                    } else {
                        String::new()
                    },
                    lines: transcript.get(n - 2).cloned().unwrap_or_default(),
                    title_style: None,
                    style: None,
                    ..panel.clone()
                },
                ("agent", agent) => self.agent_panel(panel, agent),
                ("band", agent) => Panel {
                    lines: agent.map(band).unwrap_or_default(),
                    ..panel.clone()
                },
                _ => continue,
            };
            panels.push(new);
        }
        for panel in panels {
            self.replace_panel(SCENE, &panel, false)?;
        }
        Ok(())
    }

    /// The column of one agent: the title in the color of its state, and
    /// the body muted while it rests.
    fn agent_panel(&self, panel: &Panel, agent: Option<&Agent>) -> Panel {
        let Some(agent) = agent else {
            return Panel {
                title: String::new(),
                lines: Vec::new(),
                ..panel.clone()
            };
        };
        let state = self.settings.value.states.get(&agent.state);
        let color = state.map_or("text", |s| s.color.as_str());
        // A faint title would vanish; it is muted instead.
        let title_color = if color == "faint" { "muted" } else { color };
        let style = |base: &str, color: &str| Style {
            base: Some(base.into()),
            color: Some(Color::Name(color.into())),
            ..Style::default()
        };
        let resting = color == "faint";
        let label = state.and_then(|s| s.label.clone());
        let mut lines = vec![format!(
            "{}  {}",
            state.map_or("", |s| s.sign.as_str()),
            label.unwrap_or_else(|| agent.state.clone())
        )];
        if !agent.question.is_empty() {
            lines.extend([String::new(), format!("? {}  — ja / nein", agent.question)]);
        }
        if !agent.task.is_empty() {
            lines.extend([String::new(), agent.task.clone()]);
        }
        if !agent.plan.is_empty() {
            lines.push(String::new());
            lines.extend(agent.plan.iter().cloned());
        }
        if !agent.said.is_empty() {
            // ponytail: a fixed cut keeps the panel under its 4 KiB.
            let said: String = agent.said.chars().take(SAID_CHARS).collect();
            lines.extend([String::new(), format!("“{said}”")]);
        }
        Panel {
            title: crate::title_of(&agent.name),
            lines,
            title_style: Some(style("heading", title_color)),
            style: resting.then(|| style("body", "muted")),
            ..panel.clone()
        }
    }

    /// Adds a notification on top of the stack; a critical one also
    /// shows in red until the next one or `notify.clear`.
    pub fn push(&mut self, push: Push) -> Result<(), RpcError> {
        let Push {
            title,
            body,
            urgency,
        } = push;
        let text = if body.is_empty() {
            title
        } else {
            format!("{title}: {body}")
        };
        let line = format!("{}  {}", region::date("%H:%M"), text.replace('\n', " "));
        if urgency == Urgency::Critical {
            self.set_text(URGENT, None, |_| vec![format!("✕  {text}")])?;
        }
        self.set_text(NOTIFY, None, |old| {
            let mut lines = vec![line];
            lines.extend(old);
            lines.truncate(NOTIFY_LINES);
            lines
        })
    }

    /// Shows a spoken confirmation alone in the drawer over any scene,
    /// with its whole detail, until the event `done` of its id.
    pub fn confirm(&mut self, confirm: Confirm) -> Result<(), RpcError> {
        let Confirm {
            id,
            prompt,
            detail,
            done,
            physical,
            keys,
            buttons,
        } = confirm;
        let rejected =
            |e: chora_scene::ipc::ClientError| RpcError::new(RpcError::REJECTED, e.to_string());
        if done {
            if self.confirming.as_ref().is_none_or(|c| c.id != id) {
                return Ok(());
            }
            self.confirming = None;
            self.apply(vec![Command::Remove {
                id: ANSWERS.into(),
                to: None,
            }])
            .map_err(rejected)?;
            self.set_text(CONFIRM, None, |_| Vec::new())?;
            if self.solo.as_ref().is_some_and(|s| s.content == CONFIRM) {
                self.solo(CONFIRM, Region::Drawer, Some(false), false)
                    .map_err(rejected)?;
            }
            return Ok(());
        }
        let lines = detail.lines().map(Into::into).collect();
        self.set_text(CONFIRM, Some(prompt), |_| lines)?;
        let had = self.confirming.replace(Confirming {
            id,
            buttons,
            keys: keys.filter(|_| physical),
        });
        // ponytail: it takes the place of a solo of the user, which does
        // not come back after the answer.
        self.solo(CONFIRM, Region::Drawer, Some(true), false)
            .map_err(rejected)?;
        self.show_answers(had.is_some()).map_err(rejected)
    }

    /// The answers of the confirmation at the top right of the drawer, on
    /// the line of its title, flush right as margin notes: Ja and Nein
    /// while the agent takes clicks, each with the key of a physical
    /// question before it.
    #[allow(clippy::too_many_lines, reason = "the nodes, then their places")]
    fn show_answers(&mut self, had: bool) -> Result<(), ClientError> {
        let mut commands = Vec::new();
        if had {
            commands.push(Command::Remove {
                id: ANSWERS.into(),
                to: None,
            });
        }
        let Some(c) = self
            .confirming
            .clone()
            .filter(|c| c.buttons || c.keys.is_some())
        else {
            return self.apply(commands);
        };
        let Some([x, top, w, _]) = self.drawer_box() else {
            return self.apply(commands);
        };
        let config = &self.settings.value;
        let b = config.scale.baseline;
        let (Ok(word), Ok(data)) = (config.resolve("body", None), config.resolve("margin", None))
        else {
            return self.apply(commands);
        };
        let faint = config.colors().get("faint").copied().unwrap_or([0.5; 4]);
        let muted = config.colors().get("muted").copied().unwrap_or([0.5; 4]);
        let hidden = PropsUpdate {
            opacity: Some(0.0),
            ..PropsUpdate::default()
        };
        commands.push(crate::create(
            ANSWERS,
            NodeKind::Group,
            crate::region::REGIONS,
            hidden,
        ));
        let parts = c.parts();
        for (id, w, k) in &parts {
            let mut props = word.props(w);
            if c.buttons {
                props.input = Some(InputMode::Pass);
            } else {
                props.color = Some(muted);
            }
            commands.push(crate::create(
                &format!("{id}-word"),
                NodeKind::Text,
                ANSWERS,
                props,
            ));
            if !k.is_empty() {
                let props = PropsUpdate {
                    color: Some(muted),
                    ..data.props(k)
                };
                commands.push(crate::create(
                    &format!("{id}-key"),
                    NodeKind::Text,
                    ANSWERS,
                    props,
                ));
            }
        }
        self.apply(commands)?;
        // Right to left, by the widths that the core measured.
        let mut right = x + w;
        let mut moves = Vec::new();
        let place = |id: &str, x: f64, y: f64| Command::Update {
            id: id.into(),
            props: PropsUpdate {
                transform: Some(Transform::translate(x.round(), y.round(), 0.0)),
                ..PropsUpdate::default()
            },
        };
        for (id, _, k) in parts.iter().rev() {
            let width = |s: &mut Self, node: &str| -> Result<f64, ClientError> {
                Ok(s.rect(node)?.map_or(0.0, |r| r[2]))
            };
            let ww = width(self, &format!("{id}-word"))?;
            right -= ww;
            moves.push(place(&format!("{id}-word"), right, top));
            if c.buttons {
                // A click area over the word, and a hairline under it.
                let area = PropsUpdate {
                    transform: Some(Transform::translate(right, top, 0.0)),
                    size: Some([ww, word.line]),
                    color: Some([0.0; 4]),
                    input: Some(InputMode::Accept),
                    ..PropsUpdate::default()
                };
                moves.push(crate::create(id, NodeKind::Rect, ANSWERS, area));
                let rule = PropsUpdate {
                    transform: Some(Transform::translate(right, top + word.line, 0.0)),
                    size: Some([ww, 1.0]),
                    color: Some(faint),
                    ..PropsUpdate::default()
                };
                moves.push(crate::create(
                    &format!("{id}-rule"),
                    NodeKind::Rect,
                    ANSWERS,
                    rule,
                ));
            }
            if !k.is_empty() {
                let kw = width(self, &format!("{id}-key"))?;
                right -= kw + b;
                moves.push(place(
                    &format!("{id}-key"),
                    right,
                    top + word.line - data.line,
                ));
            }
            right -= 2.0 * b;
        }
        moves.push(Command::Update {
            id: ANSWERS.into(),
            props: PropsUpdate {
                opacity: Some(1.0),
                ..PropsUpdate::default()
            },
        });
        self.apply(moves)
    }

    /// Answers the confirmation by the click on `node`, with
    /// `voice.confirm.answer`; a refusal, as for an old id, only logs.
    /// True when `node` was an answer.
    pub fn press_answer(&mut self, node: &str, injected: bool) -> bool {
        let Some(params) = (self.confirming.as_ref()).and_then(|c| c.answer(node, injected)) else {
            return false;
        };
        let call = json!({"client": crate::VOICE, "method": "confirm.answer", "params": params});
        if let Err(err) = self.rt.conn.call::<_, Value>("methods.call", call) {
            eprintln!("chora-mc: confirm.answer: {err}");
        }
        true
    }

    /// Empties the notifications.
    pub fn clear_notifications(&mut self) -> Result<(), RpcError> {
        for content in [NOTIFY, URGENT] {
            if self.texts.contains_key(content) {
                self.set_text(content, None, |_| Vec::new())?;
            }
        }
        Ok(())
    }
}

/// The cell of the `n`-th column of the agents, for `count` agents on a
/// grid of `grid` columns: one agent takes the whole width, two take half
/// each, more take a column each. A column without an agent lies under
/// the last one with an agent.
fn agent_cell([_, y, _, h]: [u32; 4], n: usize, count: usize, grid: u32) -> [u32; 4] {
    let (n, count) = (n.min(count).max(1), count.max(1));
    let span = u32::try_from(count).map_or(1, |c| (grid / c).max(1));
    let at = u32::try_from(n - 1).unwrap_or_default() * span;
    [at.min(grid.saturating_sub(span)), y, span, h]
}

/// The transcript in `columns` columns of about the same length, the
/// newest line last in the last column; an empty line between two.
fn transcript(log: &[String], columns: usize) -> Vec<Vec<String>> {
    let per = log.len().div_ceil(columns.max(1)).max(1);
    log.chunks(per)
        .map(|chunk| {
            let mut lines = Vec::new();
            for line in chunk {
                if !lines.is_empty() {
                    lines.push(String::new());
                }
                lines.push(line.clone());
            }
            lines
        })
        .collect()
}

/// The band under the column of an agent: its voice and its start.
fn band(agent: &Agent) -> Vec<String> {
    let mut lines = Vec::new();
    if !agent.voice.is_empty() {
        lines.push(format!("voice  {}", agent.voice));
    }
    lines.push(format!("since  {}", agent.since));
    lines
}

#[cfg(test)]
mod tests {
    #[test]
    fn an_error_of_chat_show_without_a_name_names_the_reason() {
        assert_eq!(
            super::chat_who(None),
            "for the helper that the host chooses"
        );
        assert_eq!(super::chat_who(Some("Atlas")), "with Atlas");
    }

    #[test]
    fn one_agent_takes_the_whole_width_two_half_each() {
        let cell = [0, 0, 1, 2];
        assert_eq!(super::agent_cell(cell, 1, 1, 4), [0, 0, 4, 2]);
        assert_eq!(super::agent_cell(cell, 3, 1, 4), [0, 0, 4, 2]);
        assert_eq!(super::agent_cell(cell, 2, 2, 4), [2, 0, 2, 2]);
        assert_eq!(super::agent_cell(cell, 3, 3, 4), [2, 0, 1, 2]);
        assert_eq!(super::agent_cell(cell, 4, 4, 4), [3, 0, 1, 2]);
    }

    /// A click on the column of a helper in the agents scene asks the
    /// host to show the chat with that helper.
    #[test]
    fn a_click_on_an_agent_column_shows_its_chat() {
        use serde_json::json;
        let (mut mc, model) = crate::testing::mc("column-chat", [1920, 1080]);
        for name in ["Atlas", "Bert"] {
            mc.follow(
                "voice.state",
                "voice",
                json!({"name": name, "state": "working"}),
            )
            .unwrap();
        }
        mc.show(super::SCENE, false).unwrap();
        let panel = mc.scenes[super::SCENE]
            .panels
            .iter()
            .find(|p| p.id == "agent-2")
            .cloned()
            .unwrap();
        let [x, y, w, h] = mc.place(super::SCENE, &panel).expect("the column of Bert");
        let at = [x + w / 2.0, y + h / 2.0];
        let press = chora_scene::Event::PointerPressed {
            node: crate::wheel_id(super::SCENE),
            button: 272,
            position: at,
            local: at,
            modifiers: Vec::new(),
            injected: false,
        };
        chora_kit::runtime::Plugin::event(&mut mc, press).unwrap();
        let called = model.lock().unwrap().called.clone();
        assert_eq!(
            called.last(),
            Some(
                &json!({"client": "assistant", "method": "chat.show", "params": {"name": "Bert"}})
            )
        );
    }

    /// A click on the row of a helper in the margin asks the host to show
    /// the chat with that helper, with no key.
    #[test]
    fn a_click_on_the_row_of_a_helper_shows_its_chat() {
        use serde_json::json;
        let (mut mc, model) = crate::testing::mc("row-chat", [1920, 1080]);
        mc.settings.value.voice_hud = crate::config::VoiceHud::Own;
        let state = json!({"name": "Atlas", "state": "working"});
        mc.follow("voice.state", "voice", state).unwrap();
        mc.show("agenda", false).unwrap();
        let name = mc.current.clone().unwrap();
        let id = (mc.items(&name).into_values())
            .flat_map(|(items, _)| items)
            .find(|i| i.hit.as_deref() == Some("agent:Atlas"))
            .map(|i| i.id)
            .expect("an item for Atlas that takes a click");
        assert!(mc.click_hit(&format!("{id}-hit")));
        let called = model.lock().unwrap().called.clone();
        assert_eq!(
            called.last(),
            Some(
                &json!({"client": "assistant", "method": "chat.show", "params": {"name": "Atlas"}})
            )
        );
    }

    /// voice.confirm shows its prompt and its whole detail alone in the
    /// drawer; the done event of its id removes it, of another id not.
    #[test]
    fn a_confirmation_shows_until_its_answer() {
        use serde_json::json;
        let (mut mc, _) = crate::testing::mc("confirm", [1920, 1080]);
        mc.settings.value.voice_hud = crate::config::VoiceHud::Own;
        let detail = format!("rm -r build\n{}", "x".repeat(400));
        let ask = json!({"id": "c1", "prompt": "Run it?", "detail": detail});
        mc.follow("voice.confirm", "voice", ask).unwrap();
        let (title, lines) = mc.texts["confirm"].clone();
        assert_eq!(title, "Run it?");
        assert_eq!(lines.join("\n"), detail, "never cut");
        assert!(mc.solo.as_ref().is_some_and(|s| s.content == "confirm"));
        let done = |id| json!({"id": id, "done": true});
        mc.follow("voice.confirm", "voice", done("c0")).unwrap();
        assert!(mc.solo.is_some(), "another id");
        mc.follow("voice.confirm", "voice", done("c1")).unwrap();
        assert!(mc.texts["confirm"].1.is_empty() && mc.solo.is_none());
    }

    /// A click on Ja or Nein maps to `confirm.answer` with the id as
    /// text, only while the confirmation has buttons.
    #[test]
    fn a_click_on_an_answer_maps_to_confirm_answer() {
        use serde_json::json;
        let with = super::Confirming {
            id: "4".into(),
            buttons: true,
            keys: None,
        };
        assert_eq!(
            with.answer(super::ANSWER_YES, false),
            Some(json!({"id": "4", "yes": true}))
        );
        assert_eq!(
            with.answer(super::ANSWER_NO, false),
            Some(json!({"id": "4", "yes": false}))
        );
        assert_eq!(with.answer("mc-answer-yes-word", false), None);
        let without = super::Confirming {
            buttons: false,
            ..with
        };
        assert_eq!(without.answer(super::ANSWER_YES, false), None);
        assert_eq!(
            super::Confirming {
                buttons: true,
                ..without
            }
            .answer(super::ANSWER_YES, true),
            None,
            "an injected press answers nothing"
        );
        let (mut mc, _) = crate::testing::mc("answers", [1920, 1080]);
        mc.settings.value.voice_hud = crate::config::VoiceHud::Own;
        let ask = json!({"id": "5", "prompt": "Set allow?", "detail": "allow: [sh]",
            "physical": true, "keys": {"yes": "Alt+y", "no": "Alt+n"}, "buttons": true});
        mc.follow("voice.confirm", "voice", ask).unwrap();
        let parts = mc.confirming.as_ref().unwrap().parts();
        assert_eq!(parts[0].2, "Alt+Y");
        assert_eq!(parts[1].1, "Nein");
    }

    /// The agent sent the id as a number: the question showed on no
    /// screen (review 1.1).
    #[test]
    fn a_confirmation_with_a_numeric_id_shows() {
        use serde_json::json;
        let (mut mc, _) = crate::testing::mc("confirm-number", [1920, 1080]);
        mc.settings.value.voice_hud = crate::config::VoiceHud::Own;
        let ask = json!({"id": 7, "prompt": "Start foot?", "detail": "foot"});
        mc.follow("voice.confirm", "voice", ask).unwrap();
        assert_eq!(mc.texts["confirm"].0, "Start foot?");
        assert!(mc.solo.as_ref().is_some_and(|s| s.content == "confirm"));
        mc.follow("voice.confirm", "voice", json!({"id": 7, "done": true}))
            .unwrap();
        assert!(mc.solo.is_none(), "the done of the number closes it");
    }

    #[test]
    fn the_transcript_flows_into_columns_newest_last() {
        let log: Vec<String> = ["a", "b", "c", "d", "e"].map(Into::into).to_vec();
        let columns = super::transcript(&log, 3);
        assert_eq!(columns, [vec!["a", "", "b"], vec!["c", "", "d"], vec!["e"]]);
        assert_eq!(super::transcript(&log[..1], 0), [vec!["a"]]);
        assert_eq!(super::transcript(&[], 3), Vec::<Vec<String>>::new());
    }
}
