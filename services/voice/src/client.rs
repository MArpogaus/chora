//! The chora client in a thread of its own: the methods of `voice`, the
//! settings (D62), the push-to-talk grab and the events of the HUD (D65),
//! on the `chora-kit` runtime.

use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use chora_kit::{
    runtime::{self, Plugin, Runtime},
    settings::{self, Store},
};
use chora_scene::{
    Event,
    ipc::{ClientError, Emit, ExportMethod, PressState, RpcError},
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    config::Config,
    core::{Call, Input},
};

/// How long a method waits for the core.
const ANSWER: Duration = Duration::from_secs(1);
/// How often the thread looks for events to emit.
// ponytail: one thread cannot block on the socket and the channel at once,
// so it polls; a wake-up fd if the idle wake-ups ever matter.
const EMIT_POLL: Duration = Duration::from_millis(50);
/// The wait before a state that the core refused goes again: the core
/// counts its rate per second.
const RESEND: Duration = Duration::from_secs(1);

struct Client {
    rt: Runtime<()>,
    settings: settings::File<Config>,
    tx: Sender<Input>,
    claims: Claims,
}

/// The clients that show the state of the agent themselves and claimed
/// the indicator: while one is connected, the dot hides. The claims of a
/// session end with it; the plugins claim again when `voice` connects.
#[derive(Default)]
struct Claims(std::collections::BTreeSet<String>);

impl Claims {
    /// A client claims the indicator; answers whether the dot hides.
    fn claim(&mut self, client: &str) -> bool {
        self.0.insert(client.to_owned());
        true
    }

    /// A client is gone; answers whether the dot still hides.
    fn gone(&mut self, client: &str) -> bool {
        self.0.remove(client);
        !self.0.is_empty()
    }
}

/// The parameters of `confirm`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
enum Answer {
    Yes,
    No,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfirmParams {
    answer: Answer,
}

/// The parameters of `talk`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Talk {
    down: bool,
}

/// The parameters of `mute`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Mute {
    mic: Option<bool>,
    speaker: Option<bool>,
}

/// Starts the thread. It sends the inputs of chora to the core, and
/// `ChoraGone` when the runtime gives up. It emits the events of `emits`
/// as `NAME.EVENT`.
pub fn start(
    name: String,
    settings: settings::File<Config>,
    tx: Sender<Input>,
    emits: Receiver<crate::mc::Emit>,
) {
    thread::spawn(move || {
        let path = settings.file().to_owned();
        let mut settings = Some(settings);
        runtime::run(&name, || {
            let settings = match settings.take() {
                Some(s) => s,
                None => load(path.clone())?,
            };
            session(&name, settings, &tx, &emits)
        });
        let _ = tx.send(Input::ChoraGone);
    });
}

/// Reads the settings file again; a broken file is an error, never the
/// defaults.
fn load(path: PathBuf) -> Result<settings::File<Config>, String> {
    settings::File::load(path, "{}").and_then(crate::unbroken)
}

fn session(
    name: &str,
    settings: settings::File<Config>,
    tx: &Sender<Input>,
    emits: &Receiver<crate::mc::Emit>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut methods = exported_methods();
    methods.extend(settings::File::<Config>::methods());
    let rt = Runtime::start(name, Vec::new(), &["client.disconnected"], methods)?;
    let mut client = Client {
        rt,
        settings,
        tx: tx.clone(),
        claims: Claims::default(),
    };
    // The claims of the session before are gone.
    let _ = tx.send(Input::Claimed(false));
    // A new session reads the file again, which an edit may have changed.
    client.apply().map_err(|e| e.message)?;
    crate::log::info(&format!("connected to chora as {name}"));
    crate::health::ok("chora");
    let result = serve(&mut client, name, emits);
    if let Err(e) = &result {
        crate::health::down("chora", &e.to_string());
    }
    // A key that nobody follows is up.
    let _ = tx.send(Input::PttUp);
    Ok(result?)
}

/// Handles the events of chora and emits the events of the HUD until the
/// connection fails.
fn serve(
    client: &mut Client,
    name: &str,
    emits: &Receiver<crate::mc::Emit>,
) -> Result<(), ClientError> {
    let mut held = Held::default();
    loop {
        match client.rt.conn.next_event_timeout(EMIT_POLL)? {
            // The runtime drops the caller, which a claim needs.
            Some(Event::MethodCalled {
                call,
                caller,
                method,
                ..
            }) if method == "indicator.claim" => {
                let _ = client.tx.send(Input::Claimed(client.claims.claim(&caller)));
                client.rt.conn.answer(call, Ok(json!({})))?;
            }
            Some(event) => runtime::dispatch(client, event)?,
            None => {}
        }
        held.emit(emits.try_iter(), Instant::now(), |event, data| {
            let emit = Emit {
                event: format!("{name}.{event}"),
                data: data.clone(),
            };
            client
                .rt
                .conn
                .call::<_, Value>("events.emit", emit)
                .map(drop)
        })?;
    }
}

/// The states that the core refused over its rate limit (D65): the last
/// `state` or `remove` of each name waits for `RESEND` and goes again,
/// and so do the states that come meanwhile. The last `confirm` waits
/// the same way. A refused `say` is gone, as its sound plays now; another
/// refusal only logs.
#[derive(Default)]
struct Held {
    events: BTreeMap<String, crate::mc::Emit>,
    until: Option<Instant>,
}

impl Held {
    fn emit(
        &mut self,
        new: impl Iterator<Item = crate::mc::Emit>,
        now: Instant,
        mut send: impl FnMut(&str, &Value) -> Result<(), ClientError>,
    ) -> Result<(), ClientError> {
        let mut out: Vec<crate::mc::Emit> = Vec::new();
        if self.until.is_some_and(|t| now >= t) {
            self.until = None;
            out.extend(std::mem::take(&mut self.events).into_values());
        }
        out.extend(new);
        for (event, data) in out {
            // The sub-agent of the state, or none for the agent; a NUL
            // is in no name.
            let key = match event {
                "state" | "remove" => Some(data["name"].as_str().unwrap_or_default().to_owned()),
                "confirm" => Some("\0confirm".to_owned()),
                _ => None,
            };
            if let (Some(key), Some(_)) = (&key, self.until) {
                self.events.insert(key.clone(), (event, data));
                continue;
            }
            match send(event, &data) {
                Err(ClientError::Rpc(e)) if e.code == RpcError::LIMIT_REACHED => {
                    if let Some(key) = key {
                        self.events.insert(key, (event, data));
                        self.until = Some(now + RESEND);
                    }
                }
                Err(ClientError::Rpc(e)) => crate::log::debug(&format!("events.emit: {e}")),
                other => other?,
            }
        }
        Ok(())
    }
}

#[allow(clippy::too_many_lines)] // one entry per method
fn exported_methods() -> Vec<ExportMethod> {
    let none = json!({});
    vec![
        runtime::method(
            "state",
            "Answers the state of the voice agent: state, since_ms, seq, awake, muted, turn, \
             last_transcript and last_reply.",
            none.clone(),
            &[],
        ),
        runtime::method(
            "cancel",
            "Stops the current turn; a coding task goes on.",
            none.clone(),
            &[],
        ),
        runtime::method(
            "sleep",
            "Lets the agent sleep: it hears only its wake words and the push-to-talk key.",
            none.clone(),
            &[],
        ),
        runtime::method("wake", "Wakes the agent up.", none.clone(), &[]),
        runtime::method(
            "mute",
            "Mutes or unmutes the mic and the speaker, each on its own; a muted mic still \
             records with the push-to-talk key.",
            json!({"mic": {"type": "boolean"}, "speaker": {"type": "boolean"}}),
            &[],
        ),
        // Privileged: a yes runs programs and allows what sub-agents ask.
        runtime::privileged(runtime::method(
            "confirm",
            "Answers the question of the agent before a tool call: yes runs the call, no \
             declines it.",
            json!({"answer": {"type": "string", "enum": ["yes", "no"]}}),
            &["answer"],
        )),
        runtime::method(
            "talk",
            "The push-to-talk key of another client: down stops a reply and records, up \
             ends the utterance.",
            json!({"down": {"type": "boolean"}}),
            &["down"],
        ),
        runtime::method(
            "health",
            "Answers the health of the servers asr, tts, llm, mcp and chora: status ok, down \
             or unknown, the last error and its Unix time error_at.",
            none.clone(),
            &[],
        ),
        // Privileged (D62): it starts programs.
        runtime::privileged(runtime::method(
            "agent",
            "Runs a tool of the ACP agents without the LLM: tool is agent_list, agent_start \
             (project, name, voice), agent_say (name, text), agent_stop (name) or \
             agent_status (name); answers its text.",
            json!({"tool": {"type": "string"}, "project": {"type": "string"},
                "name": {"type": "string"}, "voice": {"type": "string"},
                "text": {"type": "string"}}),
            &["tool"],
        )),
        runtime::method(
            "indicator.claim",
            "Hides the state dot of the agent while the calling client is connected: the \
             caller shows the state itself. Call it again after the voice client connects.",
            none.clone(),
            &[],
        ),
        runtime::method(
            "restart",
            "Reads the settings file again, for example after an edit by hand.",
            none,
            &[],
        ),
    ]
}

impl Client {
    /// Grabs the push-to-talk key; a key that another client holds only
    /// logs.
    fn grab(&mut self) -> Result<(), ClientError> {
        let key = &self.settings.value.ptt_key;
        let binding = key.parse().expect("the settings check the key");
        if !self.rt.grab(binding, ())? {
            crate::log::warn(&format!("no push-to-talk key {key}"));
        }
        Ok(())
    }

    /// Sends a call to the core and waits for the state record.
    fn ask(&self, call: Call) -> Result<Value, RpcError> {
        let (tx, rx) = mpsc::channel();
        let gone = || RpcError::new(RpcError::REJECTED, "the core does not answer");
        self.tx.send(Input::Call(call, tx)).map_err(|_| gone())?;
        rx.recv_timeout(ANSWER).map_err(|_| gone())
    }

    /// Hands the settings to the core and grabs the key again.
    fn apply(&mut self) -> Result<(), RpcError> {
        let rejected = |e: ClientError| RpcError::new(RpcError::REJECTED, e.to_string());
        self.rt.ungrab_all().map_err(rejected)?;
        self.grab().map_err(rejected)?;
        let config = Box::new(self.settings.value.clone());
        self.tx
            .send(Input::Settings(config))
            .map_err(|_| RpcError::new(RpcError::REJECTED, "the core does not answer"))
    }
}

impl Plugin for Client {
    type Action = ();

    fn runtime(&mut self) -> &mut Runtime<()> {
        &mut self.rt
    }

    fn event(&mut self, event: Event) -> Result<(), ClientError> {
        if let Event::ClientDisconnected { name } = &event
            && self.claims.0.contains(name)
        {
            let _ = self.tx.send(Input::Claimed(self.claims.gone(name)));
        }
        if let Event::InputKey { binding, state } = event
            && self.rt.action(&binding).is_some()
        {
            let input = match state {
                PressState::Pressed => Input::PttDown,
                PressState::Released => Input::PttUp,
            };
            // The core is gone only when the process ends.
            let _ = self.tx.send(input);
        }
        Ok(())
    }

    fn method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        let call = match method {
            "health" => return Ok(crate::health::record()),
            "agent" => {
                let tool = params["tool"].as_str().unwrap_or_default().to_owned();
                if !crate::agents::TOOLS.contains(&tool.as_str()) {
                    let tools = crate::agents::TOOLS.join(", ");
                    return Err(RpcError::new(
                        RpcError::INVALID_PARAMS,
                        format!("tool: one of {tools}"),
                    ));
                }
                let (tx, rx) = mpsc::channel();
                let gone = || RpcError::new(RpcError::REJECTED, "the agent does not answer");
                (self.tx.send(Input::AgentCall(tool, params, tx))).map_err(|_| gone())?;
                return rx.recv_timeout(ANSWER).map_err(|_| gone());
            }
            "state" => Call::State,
            "cancel" => Call::Cancel,
            "sleep" => Call::Sleep,
            "wake" => Call::Wake,
            "confirm" => {
                let ConfirmParams { answer } = runtime::params(params)?;
                Call::Confirm(matches!(answer, Answer::Yes))
            }
            "talk" => {
                let Talk { down } = runtime::params(params)?;
                Call::Talk(down)
            }
            "mute" => {
                let Mute { mic, speaker } = runtime::params(params)?;
                Call::Mute { mic, speaker }
            }
            "restart" => {
                let path = self.settings.file().to_owned();
                self.settings =
                    load(path).map_err(|e| RpcError::new(RpcError::INVALID_PARAMS, e))?;
                self.apply()?;
                return Ok(self.settings.get());
            }
            _ => return Err(RpcError::new(RpcError::METHOD_NOT_FOUND, method)),
        };
        self.ask(call)
    }

    fn settings(&mut self) -> Option<&mut dyn Store> {
        Some(&mut self.settings)
    }

    fn settings_changed(&mut self) -> Result<(), RpcError> {
        self.apply()
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use chora_scene::ipc::{ClientError, ExportMethod, RpcError};
    use serde_json::{Value, json};

    use super::{Claims, Held, RESEND};

    /// A claim hides the dot, the claimer's disconnect shows it again,
    /// and a second claimer keeps it hidden.
    #[test]
    fn claims_hide_the_dot_while_a_claimer_is_connected() {
        let mut claims = Claims::default();
        assert!(claims.claim("mc"));
        assert!(!claims.gone("mc"), "the dot shows again");
        assert!(claims.claim("mc"));
        assert!(claims.claim("xp"));
        assert!(claims.gone("mc"), "xp still claims");
        assert!(claims.gone("other"), "not a claimer");
        assert!(!claims.gone("xp"));
    }

    /// Emits `new` at `now`; the core takes them or refuses them over its
    /// rate limit. Answers what it took.
    fn emit(
        held: &mut Held,
        new: Vec<(&'static str, Value)>,
        now: Instant,
        refuse: bool,
    ) -> Vec<(String, Value)> {
        let mut sent: Vec<(String, Value)> = Vec::new();
        held.emit(new.into_iter(), now, |event, data| {
            if refuse {
                let e = RpcError::new(RpcError::LIMIT_REACHED, "too many");
                return Err(ClientError::Rpc(e));
            }
            sent.push((event.to_owned(), data.clone()));
            Ok(())
        })
        .unwrap();
        sent
    }

    /// A burst over the rate limit: the last state of each name goes
    /// again after the wait, a removal too; a refused sentence is gone.
    #[test]
    fn refused_states_go_again_after_the_wait() {
        let mut held = Held::default();
        let none: Vec<(String, Value)> = Vec::new();
        let t0 = Instant::now();
        let state = |name: &str, s: &str| ("state", json!({"name": name, "state": s}));
        let burst = vec![
            state("a", "working"),
            ("say", json!({"text": "Hallo"})),
            state("b", "idle"),
            ("remove", json!({"name": "b"})),
            ("state", json!({"state": "listening"})),
        ];
        assert_eq!(emit(&mut held, burst, t0, true), none);
        // Within the wait a new state waits behind the others.
        assert_eq!(emit(&mut held, vec![state("a", "done")], t0, false), none);
        let before = t0 + RESEND.saturating_sub(Duration::from_millis(1));
        assert_eq!(emit(&mut held, vec![], before, false), none);
        let mut sent = emit(&mut held, vec![], t0 + RESEND, false);
        sent.sort_by_key(|(_, d)| d.to_string());
        let want = [
            ("remove", json!({"name": "b"})),
            ("state", json!({"name": "a", "state": "done"})),
            ("state", json!({"state": "listening"})),
        ];
        let mut want: Vec<(String, Value)> = want.map(|(e, d)| (e.to_owned(), d)).into();
        want.sort_by_key(|(_, d)| d.to_string());
        assert_eq!(sent, want);
        assert_eq!(emit(&mut held, vec![], t0 + RESEND * 2, false), none);
    }

    /// A yes and the agent tools start programs: a confined client needs
    /// `methods.privileged` for them.
    #[test]
    fn confirm_and_agent_are_privileged() {
        let privileged: Vec<String> = (super::exported_methods().into_iter())
            .filter_map(|m| match m {
                ExportMethod::Described(d) if d.privileged => Some(d.name),
                _ => None,
            })
            .collect();
        assert_eq!(privileged, ["confirm", "agent"]);
    }
}
