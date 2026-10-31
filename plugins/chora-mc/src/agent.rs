//! The agents that report their state with `agent.set`, for the status
//! line of the regions and the agents scene, and the notifications of
//! `notify.push`.

use chora_scene::ipc::RpcError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    Mc,
    config::{Color, Style},
    invalid,
    region::{self, Content, Item, MAX_LINES, Region},
    scene::{Panel, check_name},
};

/// The scene that shows the agents, one column each.
const SCENE: &str = "agents";

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
                    id: format!("{id}-{i}"),
                    slot,
                    props: ty.props(&text),
                    first: i == 0,
                }
            })
            .collect()
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
        // Newest on top, an empty line between two.
        self.set_text(NOTIFY, None, |old| {
            let mut lines = vec![line];
            if !old.is_empty() {
                lines.push(String::new());
            }
            lines.extend(old);
            lines.truncate(MAX_LINES);
            while lines.last().is_some_and(String::is_empty) {
                lines.pop();
            }
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
        } = confirm;
        let rejected =
            |e: chora_scene::ipc::ClientError| RpcError::new(RpcError::REJECTED, e.to_string());
        if done {
            if self.confirming.as_deref() != Some(&id) {
                return Ok(());
            }
            self.confirming = None;
            self.set_text(CONFIRM, None, |_| Vec::new())?;
            if self.solo.as_ref().is_some_and(|s| s.content == CONFIRM) {
                self.solo(CONFIRM, Region::Drawer, Some(false), false)
                    .map_err(rejected)?;
            }
            return Ok(());
        }
        let lines = detail.lines().map(Into::into).collect();
        self.set_text(CONFIRM, Some(prompt), |_| lines)?;
        self.confirming = Some(id);
        // ponytail: it takes the place of a solo of the user, which does
        // not come back after the answer.
        self.solo(CONFIRM, Region::Drawer, Some(true), false)
            .map_err(rejected)?;
        Ok(())
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
    fn one_agent_takes_the_whole_width_two_half_each() {
        let cell = [0, 0, 1, 2];
        assert_eq!(super::agent_cell(cell, 1, 1, 4), [0, 0, 4, 2]);
        assert_eq!(super::agent_cell(cell, 3, 1, 4), [0, 0, 4, 2]);
        assert_eq!(super::agent_cell(cell, 2, 2, 4), [2, 0, 2, 2]);
        assert_eq!(super::agent_cell(cell, 3, 3, 4), [2, 0, 1, 2]);
        assert_eq!(super::agent_cell(cell, 4, 4, 4), [3, 0, 1, 2]);
    }

    /// voice.confirm shows its prompt and its whole detail alone in the
    /// drawer; the done event of its id removes it, of another id not.
    #[test]
    fn a_confirmation_shows_until_its_answer() {
        use serde_json::json;
        let (mut mc, _) = crate::testing::mc("confirm", [1920, 1080]);
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

    /// The agent sent the id as a number: the question showed on no
    /// screen (review 1.1).
    #[test]
    fn a_confirmation_with_a_numeric_id_shows() {
        use serde_json::json;
        let (mut mc, _) = crate::testing::mc("confirm-number", [1920, 1080]);
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
