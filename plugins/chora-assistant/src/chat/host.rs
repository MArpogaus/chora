//! The host side of the chat: the methods `chat.show` and `chat.hide`,
//! the connection to voice and the core that the session reaches through
//! [`Outside`], the field program, and the steps that keep view, field and
//! context together. The "Chat ..." row is drawn here too.

use std::time::{Duration, Instant};

use chora_draw::{
    bubble::Style,
    text::{Font, Text},
    theme::Picture,
};
use chora_kit::{conn::Conn, hud::Chat as Look, runtime, words::Word};
use chora_scene::{
    Command, InputMode, NodeKind, PropsUpdate,
    ipc::{ClientError, RpcError},
};
use serde_json::{Value, json};

use super::{
    Frame, Link, Outside, colours, rgba,
    view::{CLOSE, GAP, LINE, PAD, ROW, STOP, create, image, pick_of, refuses_injected, style_of},
};
use crate::{
    Agent,
    assistant::{self, origin},
    config::Values,
    field::Field,
};

/// How long the chat goes without a pull before the host reads again: voice
/// ends a chat that nobody reads for ten minutes.
const KEEPALIVE: Duration = Duration::from_secs(120);

/// The time of the next pull that keeps the chat alive.
fn due(last: Instant) -> Instant {
    last + KEEPALIVE
}

/// The group of the "Chat ..." row under `screen:OUTPUT`.
const ROW_GROUP: &str = "hud-rows";
/// How often the host looks at the field while a chat is open.
const FIELD_POLL: Duration = Duration::from_millis(100);
/// How often the host asks voice again while it does not answer.
const RETRY: Duration = Duration::from_secs(5);

/// The parameters of `chat.show`.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShowParams {
    pub name: Option<String>,
    pub token: Option<String>,
}

/// The methods that the host exports for the chat. Neither is privileged:
/// the voice agent may call them, so "show the chat with Atlas" works by
/// voice.
#[must_use]
pub fn exported() -> Vec<chora_scene::ipc::ExportMethod> {
    vec![
        runtime::method(
            "chat.show",
            "Shows the chat with the helper name, or the list of helpers without a name. \
             token is the activation token of the act of a person.",
            json!({"name": {"type": "string"}, "token": {"type": "string"}}),
            &[],
        ),
        runtime::method(
            "chat.hide",
            "Closes the chat and its field.",
            json!({}),
            &[],
        ),
    ]
}

/// Refuses a caller on a confined connection: the host acts in its own
/// unconfined name, so only an unconfined caller may ask it to.
///
/// # Errors
///
/// The caller is confined.
pub fn guard(method: &str, confined: bool) -> Result<(), RpcError> {
    if confined {
        return Err(RpcError::new(
            RpcError::REJECTED,
            format!("{method}: not for a confined connection"),
        ));
    }
    Ok(())
}

/// What `assistant.chat` said last: open, with whom, and where.
pub type Announced = (bool, Option<String>, String);

/// The data of `assistant.chat` when it differs from the last one.
pub fn announcement(last: &mut Option<Announced>, now: Announced) -> Option<Value> {
    if last.as_ref() == Some(&now) {
        return None;
    }
    let data = json!({"open": now.0, "name": now.1, "output": now.2});
    *last = Some(now);
    Some(data)
}

/// The connection of the host as the session sees voice and the core.
struct Bridge<'a> {
    conn: &'a mut Conn,
    client: String,
}

impl Outside for Bridge<'_> {
    fn voice(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let call = json!({"client": self.client, "method": method, "params": params});
        self.conn
            .call("methods.call", call)
            .map_err(|err| err.to_string())
    }

    fn token(&mut self) -> Option<String> {
        let answer: Value = self.conn.call("activation.token", Value::Null).ok()?;
        answer["token"].as_str().map(str::to_owned)
    }

    fn start_field(&mut self) -> Result<Box<dyn Link>, String> {
        let exe = std::env::current_exe().ok();
        let program = exe
            .as_deref()
            .and_then(std::path::Path::parent)
            .map(|dir| dir.join("chora-chat"))
            .filter(|path| path.exists())
            .unwrap_or_else(|| "chora-chat".into());
        let mut command = std::process::Command::new(program);
        command.args(["--input", "chat"]);
        let field = Field::start(command).map_err(|err| format!("chora-chat: {err}"))?;
        Ok(Box::new(field))
    }
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "the width of a label is a few hundred pixels"
)]
fn text_pixels(width: f64) -> i32 {
    width.ceil() as i32
}

/// The client name of the voice agent.
fn voice_client(voice: Option<&(String, &'static str)>) -> String {
    voice.map_or_else(|| "voice".to_owned(), |(client, _)| client.clone())
}

/// The `chat:` values of the folder on the screen.
fn chat_look(values: &Values) -> Look {
    values
        .hud
        .as_ref()
        .map_or_else(Look::default, |hud| hud.chat.clone())
}

impl Agent {
    /// Where the chat shows now: the output of the focus.
    pub fn chat_frame(&mut self) -> Result<Option<Frame>, ClientError> {
        let Some((output, scale)) = self.output()? else {
            return Ok(None);
        };
        self.theme.set_scale(scale);
        let frame = Frame {
            output: output.name.clone(),
            size: output.size,
            area: assistant::work_area(&output),
            look: chat_look(&self.values),
        };
        self.chat_seen = Some(frame.clone());
        Ok(Some(frame))
    }

    /// Draws the view again, and tells the context and the drawer of mc.
    pub fn chat_draw(&mut self) -> Result<(), ClientError> {
        let Some(frame) = self.chat_frame()? else {
            return Ok(());
        };
        let top = self.top.is_some();
        let commands = self
            .chat
            .commands(&mut self.theme, &frame, (self.language, top));
        self.apply(commands)?;
        self.switcher.set_chat(self.chat.is_open(), Instant::now());
        self.chat_announce(&frame)
    }

    fn chat_announce(&mut self, frame: &Frame) -> Result<(), ClientError> {
        let now = (
            self.chat.is_open(),
            self.chat.helper().map(str::to_owned),
            frame.output.clone(),
        );
        let Some(data) = announcement(&mut self.announced, now) else {
            return Ok(());
        };
        let emit = json!({"event": "assistant.chat", "data": data});
        self.rt.conn.call::<_, Value>("events.emit", emit).map(drop)
    }

    /// Shows the chat with `name`, or the list.
    ///
    /// # Errors
    ///
    /// The field does not start or the socket fails.
    pub fn chat_show(&mut self, name: Option<&str>, token: Option<&str>) -> Result<(), String> {
        let frame = self
            .chat_frame()
            .map_err(|err| err.to_string())?
            .ok_or("no output shows the chat")?;
        let mut bridge = Bridge {
            conn: &mut self.rt.conn,
            client: voice_client(self.voice.as_ref()),
        };
        self.chat.show(&mut bridge, name, token, &frame)?;
        self.chat_pulled = Instant::now();
        self.chat_draw().map_err(|err| err.to_string())
    }

    /// Closes the chat.
    ///
    /// # Errors
    ///
    /// The socket fails.
    pub fn chat_hide(&mut self) -> Result<(), ClientError> {
        let mut bridge = Bridge {
            conn: &mut self.rt.conn,
            client: voice_client(self.voice.as_ref()),
        };
        self.chat.hide(&mut bridge);
        self.chat_draw()
    }

    /// A press on a node of the chat: the row, an entry of the list, Stop
    /// or Close. True when the node was one.
    pub fn chat_pressed(&mut self, node: &str, injected: bool) -> Result<bool, ClientError> {
        if injected && refuses_injected(node) {
            return Ok(true);
        }
        if node == ROW {
            self.chat_row()?;
            return Ok(true);
        }
        if let Some(index) = pick_of(node) {
            self.chat_pick(index)?;
            return Ok(true);
        }
        match node {
            STOP => {
                if let Some(name) = self.chat.helper().map(str::to_owned) {
                    let _ = self.voice_call("agent.cancel", &json!({"name": name}));
                }
                Ok(true)
            }
            CLOSE => {
                self.chat_hide()?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn chat_row(&mut self) -> Result<(), ClientError> {
        let Some(frame) = self.chat_frame()? else {
            return Ok(());
        };
        let mut bridge = Bridge {
            conn: &mut self.rt.conn,
            client: voice_client(self.voice.as_ref()),
        };
        if let Err(err) = self.chat.clicked_row(&mut bridge, &frame) {
            eprintln!("chora-assistant: chat: {err}");
        }
        self.chat_draw()
    }

    fn chat_pick(&mut self, index: usize) -> Result<(), ClientError> {
        let Some(frame) = self.chat_frame()? else {
            return Ok(());
        };
        let mut bridge = Bridge {
            conn: &mut self.rt.conn,
            client: voice_client(self.voice.as_ref()),
        };
        if let Err(err) = self.chat.clicked_pick(&mut bridge, index, &frame) {
            eprintln!("chora-assistant: chat: {err}");
        }
        self.chat_draw()
    }

    /// A voice event for the chat; the view is drawn at the next step.
    pub fn chat_voice(&mut self, event: &str, data: &Value) {
        let mut bridge = Bridge {
            conn: &mut self.rt.conn,
            client: voice_client(self.voice.as_ref()),
        };
        self.chat.voice_event(&mut bridge, event, data);
    }

    /// The look of another folder or output: the field moves, the view is
    /// drawn again.
    pub fn chat_restyle(&mut self) -> Result<(), ClientError> {
        if !self.chat.is_open() {
            return Ok(());
        }
        if let Some(frame) = self.chat_frame()? {
            self.chat.restyle(&frame);
        }
        self.chat_draw()
    }

    /// When the host looks at the chat next.
    pub fn chat_due(&self, now: Instant) -> Option<Instant> {
        let retry = self.chat.offline().then(|| self.chat_retried + RETRY);
        self.chat.is_open().then(|| {
            [now + FIELD_POLL, due(self.chat_pulled)]
                .into_iter()
                .chain(retry)
                .min()
                .unwrap_or(now)
        })
    }

    /// Takes what the field told and keeps the chat alive.
    pub fn chat_step(&mut self, now: Instant) -> Result<(), ClientError> {
        if !self.chat.is_open() && !self.chat.keyboard() {
            return Ok(());
        }
        let Some(frame) = self.chat_seen.clone() else {
            return Ok(());
        };
        let mut bridge = Bridge {
            conn: &mut self.rt.conn,
            client: voice_client(self.voice.as_ref()),
        };
        let mut changed = self.chat.take_dirty();
        changed |= self.chat.field_events(&mut bridge, &frame);
        if due(self.chat_pulled) <= now {
            self.chat_pulled = now;
            changed |= self.chat.pull(&mut bridge);
        }
        if self.chat.offline() && self.chat_retried + RETRY <= now {
            self.chat_retried = now;
            changed |= self.chat.retry(&mut bridge);
        }
        if changed {
            self.chat_draw()?;
        }
        Ok(())
    }

    /// The "Chat ..." row, drawn again on the output of the focus.
    pub fn sync_row(&mut self) -> Result<Vec<Command>, ClientError> {
        let Some((output, scale)) = self.output()? else {
            return Ok(Vec::new());
        };
        self.theme.set_scale(scale);
        let mut commands = Vec::new();
        if self.row.take().is_some() {
            commands.push(Command::Remove {
                id: ROW_GROUP.into(),
                to: None,
            });
        }
        let colours = colours(&chat_look(&self.values));
        let label = Word::ChatRow.text(self.language);
        let width = text_pixels(self.theme.text_width(label, Font::Body)) + 2 * PAD;
        let size = [width, LINE + GAP * 2];
        let style = Style {
            radius: 8,
            ..style_of(&Look::default(), colours)
        };
        let picture = Picture::new(size[0], size[1])
            .svg(style.shape(size, 0))
            .text(Text::new(
                label,
                Font::Body,
                rgba(colours.text),
                [PAD, GAP, size[0] - 2 * PAD, LINE],
            ));
        let (anchor, margin) = self.values.rows;
        let at = origin(anchor, assistant::work_area(&output), size, margin);
        let path = self.theme.render(&picture);
        commands.push(Command::Create {
            id: ROW_GROUP.into(),
            kind: NodeKind::Group,
            parent: format!("screen:{}", output.name),
            props: PropsUpdate::default(),
        });
        commands.push(create(
            ROW,
            ROW_GROUP,
            image(path, at, label, InputMode::Accept),
        ));
        if self.top.is_some() {
            commands.push(Command::Move {
                id: crate::card::TOP.into(),
                parent: None,
                index: None,
            });
        }
        self.row = Some(output.name);
        Ok(commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_confined_caller_cannot_show_or_hide_the_chat() {
        assert!(guard("chat.show", true).is_err());
        assert!(guard("chat.hide", true).is_err());
        assert!(guard("chat.show", false).is_ok());
    }

    #[test]
    fn mc_hears_of_a_chat_that_moves_to_another_output() {
        let mut last = None;
        let first = (true, Some("atlas".to_owned()), "DP-1".to_owned());
        assert!(announcement(&mut last, first.clone()).is_some());
        assert!(announcement(&mut last, first).is_none());
        let moved = (true, Some("atlas".to_owned()), "DP-2".to_owned());
        let said = announcement(&mut last, moved).unwrap();
        assert_eq!(said["output"], "DP-2");
    }
}
