//! The connection of the field to chora, the main socket of the session:
//! an unconfined connection, as every plugin has it, so that the field may
//! call `voice agent`. It needs no grant. A confined caller cannot reach
//! the field: its methods are privileged.

use std::{
    fs::File,
    io::Write,
    os::fd::OwnedFd,
    sync::mpsc::{Receiver, Sender, TryRecvError},
    time::Duration,
};

use chora_kit::{
    ask::{Keys, Question},
    runtime::{self, Plugin, Runtime},
};
use chora_scene::{
    Event,
    ipc::{ClientError, RpcError},
};
use serde_json::{Value, json};

use crate::window::Place;

/// The client name of the field.
pub const NAME: &str = "chat-field";

/// The client name of the voice agent.
const VOICE: &str = "voice";

/// How long the thread waits for an event before it looks at its requests.
const POLL: Duration = Duration::from_millis(40);

/// What the connection tells the window.
#[derive(Debug)]
pub enum Msg {
    /// Show the field here.
    Place(Box<Place>),
    /// A question of the helper starts or ends.
    Question(bool),
    /// The keys that answer the question that waits.
    Keys(Keys),
    /// A text did not reach the helper.
    SayFailed(String),
    /// End the program.
    Quit,
}

/// What the window asks of the connection.
#[derive(Debug)]
pub enum Request {
    /// Send a text to the helper.
    Say(String),
    /// Emit an event of the field.
    Emit(&'static str, Value),
}

/// Tells the window loop something and wakes it.
pub struct Wake {
    msgs: Sender<Msg>,
    pipe: File,
}

impl Wake {
    /// A wake that sends over `msgs` and writes to `pipe`.
    #[must_use]
    pub fn new(msgs: Sender<Msg>, pipe: OwnedFd) -> Self {
        Self {
            msgs,
            pipe: File::from(pipe),
        }
    }

    fn tell(&self, msg: Msg) {
        if self.msgs.send(msg).is_ok() {
            let _ = (&self.pipe).write_all(&[1]);
        }
    }
}

/// The plugin of the connection.
pub struct Ipc {
    rt: Runtime<()>,
    helper: String,
    wake: Wake,
}

impl Ipc {
    /// Connects as the field, subscribes to the voice events and exports
    /// `place` and `quit`.
    ///
    /// # Errors
    ///
    /// The socket is missing, or a field runs already.
    pub fn connect(helper: &str, wake: Wake) -> Result<Self, ClientError> {
        let methods = vec![
            runtime::privileged(runtime::method(
                "place",
                "Shows the field at a place and keeps its draft: anchor (top, bottom, left, \
                 right, or a corner such as bottom_right), margin [top, right, bottom, left], \
                 width, output, look (colors) and token, the activation token of the act that \
                 opened the chat. Without a token the field takes no keyboard.",
                json!({"anchor": {"type": "string"}, "margin": {"type": "array"},
                    "width": {"type": "integer"}, "output": {"type": "string"},
                    "token": {"type": "string"}, "look": {"type": "object"}}),
                &["anchor"],
            )),
            runtime::privileged(runtime::method(
                "quit",
                "Ends the field and drops its draft.",
                json!({}),
                &[],
            )),
        ];
        let rt = Runtime::start(NAME, Vec::new(), &["voice.*"], methods)?;
        Ok(Self {
            rt,
            helper: helper.to_lowercase(),
            wake,
        })
    }

    /// Serves events and requests until the window ends or the socket fails.
    pub fn serve(mut self, requests: &Receiver<Request>) {
        loop {
            match self.requests(requests) {
                Ok(true) => {}
                Ok(false) => return,
                Err(error) => {
                    eprintln!("chora-chat: {error}");
                    self.wake.tell(Msg::Quit);
                    return;
                }
            }
        }
    }

    fn requests(&mut self, requests: &Receiver<Request>) -> Result<bool, ClientError> {
        loop {
            match requests.try_recv() {
                Ok(request) => self.request(request)?,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return Ok(false),
            }
        }
        if let Some(event) = self.rt.conn.next_event_timeout(POLL)? {
            runtime::dispatch(self, event)?;
        }
        Ok(true)
    }

    fn request(&mut self, request: Request) -> Result<(), ClientError> {
        match request {
            Request::Say(text) => self.say(text),
            Request::Emit(event, data) => {
                let emit = json!({"event": event, "data": data});
                self.rt.conn.call::<_, Value>("events.emit", emit).map(drop)
            }
        }
    }

    fn say(&mut self, text: String) -> Result<(), ClientError> {
        let params = json!({"tool": "agent_say", "name": self.helper, "text": text});
        let call = json!({"client": VOICE, "method": "agent", "params": params});
        let answer = self.rt.conn.call::<_, Value>("methods.call", call);
        let failed = match &answer {
            Ok(Value::String(reply)) => reply.starts_with("error") || reply.contains("has ended"),
            Ok(_) => false,
            Err(ClientError::Rpc(_)) => true,
            Err(_) => return answer.map(drop),
        };
        if failed {
            self.wake.tell(Msg::SayFailed(text));
        } else {
            let sent = json!({"event": "chat-field.sent", "data": {"name": self.helper}});
            self.rt.conn.call::<_, Value>("events.emit", sent)?;
        }
        Ok(())
    }

    /// The voice events that tell whether a question of the helper waits
    /// and which keys answer it.
    fn voice_event(&self, event: &str, data: &Value) {
        let ours = (data["name"].as_str()).is_some_and(|n| n.eq_ignore_ascii_case(&self.helper));
        match event {
            "voice.state" if ours => self.wake.tell(Msg::Question(data["state"] == "question")),
            "voice.confirm" => {
                if let Some(question) = Question::from_event(data, None) {
                    self.wake.tell(Msg::Keys(question.keys));
                }
            }
            _ => {}
        }
    }
}

impl Plugin for Ipc {
    type Action = ();

    fn runtime(&mut self) -> &mut Runtime<()> {
        &mut self.rt
    }

    fn event(&mut self, event: Event) -> Result<(), ClientError> {
        if let Event::Emitted {
            event,
            client,
            data,
        } = event
            && client == VOICE
        {
            self.voice_event(&event, &data);
        }
        Ok(())
    }

    fn method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        match method {
            "place" => {
                let place: Place = runtime::params(params)?;
                self.wake.tell(Msg::Place(Box::new(place)));
            }
            "quit" => self.wake.tell(Msg::Quit),
            _ => return Err(RpcError::new(RpcError::METHOD_NOT_FOUND, method)),
        }
        Ok(json!({}))
    }
}
