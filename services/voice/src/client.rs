//! The chora client in a thread of its own: the methods of `voice`, the
//! settings (D62) and the push-to-talk grab, on the `chora-kit` runtime.

use std::{
    path::PathBuf,
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};

use chora_kit::{
    runtime::{self, Plugin, Runtime},
    settings::{self, Store},
};
use chora_scene::{
    Event,
    ipc::{ClientError, ExportMethod, PressState, RpcError},
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    config::Config,
    core::{Call, Input},
};

/// How long a method waits for the core.
const ANSWER: Duration = Duration::from_secs(1);

struct Client {
    rt: Runtime<()>,
    settings: settings::File<Config>,
    tx: Sender<Input>,
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
/// `ChoraGone` when the runtime gives up.
pub fn start(name: String, settings: settings::File<Config>, tx: Sender<Input>) {
    thread::spawn(move || {
        let path = settings.file().to_owned();
        let mut settings = Some(settings);
        runtime::run(&name, || {
            let settings = match settings.take() {
                Some(s) => s,
                None => load(path.clone())?,
            };
            session(&name, settings, &tx)
        });
        let _ = tx.send(Input::ChoraGone);
    });
}

/// Reads the settings file again.
fn load(path: PathBuf) -> Result<settings::File<Config>, String> {
    settings::File::load(path, "{}")
}

fn session(
    name: &str,
    settings: settings::File<Config>,
    tx: &Sender<Input>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut methods = exported_methods();
    methods.extend(settings::File::<Config>::methods());
    let rt = Runtime::start(name, Vec::new(), &[], methods)?;
    let mut client = Client {
        rt,
        settings,
        tx: tx.clone(),
    };
    // A new session reads the file again, which an edit may have changed.
    client.apply().map_err(|e| e.message)?;
    crate::log::info(&format!("connected to chora as {name}"));
    crate::health::ok("chora");
    let result = runtime::serve(&mut client);
    if let Err(e) = &result {
        crate::health::down("chora", &e.to_string());
    }
    // A key that nobody follows is up.
    let _ = tx.send(Input::PttUp);
    Ok(result?)
}

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
        runtime::method(
            "confirm",
            "Answers the question of the agent before a tool call: yes runs the call, no \
             declines it.",
            json!({"answer": {"type": "string", "enum": ["yes", "no"]}}),
            &["answer"],
        ),
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
