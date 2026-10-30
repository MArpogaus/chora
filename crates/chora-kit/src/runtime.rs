//! The runtime that every plugin repeats: it connects, subscribes to the
//! events, exports the methods, grabs bindings, answers method calls and
//! `settings.get`, `settings.set` and `settings.reload`, keeps to the
//! outputs of `--output`, and comes back after the core closed the
//! connection.

use std::{
    process::ExitCode,
    thread::sleep,
    time::{Duration, Instant},
};

use chora_scene::{
    Binding, Event, OutputInfo,
    ipc::{ClientError, DescribedMethod, Export, ExportMethod, Grab, RpcError, Subscribe},
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::{conn::Conn, settings::Store};

/// A plugin on the runtime: it handles the events and the calls of its
/// exported methods.
pub trait Plugin {
    /// What a grabbed binding does.
    type Action;

    /// The runtime of the plugin.
    fn runtime(&mut self) -> &mut Runtime<Self::Action>;

    /// Handles an event other than a method call. Events of outputs that
    /// `--output` leaves to others do not come here.
    ///
    /// # Errors
    ///
    /// The socket fails.
    fn event(&mut self, event: Event) -> Result<(), ClientError>;

    /// Runs an exported method; the runtime sends the answer.
    ///
    /// # Errors
    ///
    /// The method fails; the caller gets the error.
    fn method(&mut self, method: &str, params: Value) -> Result<Value, RpcError>;

    /// The settings behind `settings.get` and `settings.set`; none by
    /// default. The plugin exports the methods of its settings file.
    fn settings(&mut self) -> Option<&mut dyn Store> {
        None
    }

    /// Applies the settings after a `settings.set`.
    ///
    /// # Errors
    ///
    /// The change cannot be applied; the caller gets the error.
    fn settings_changed(&mut self) -> Result<(), RpcError> {
        Ok(())
    }
}

/// The connection of a plugin, with its outputs and grabs.
pub struct Runtime<A> {
    /// The connection to the core.
    pub conn: Conn,
    name: String,
    /// The outputs that `--output` leaves to the plugin; all when empty.
    outputs: Vec<String>,
    /// The grabbed bindings, as the core names them.
    grabs: Vec<(String, A)>,
}

impl<A> Runtime<A> {
    /// Connects as `name`, subscribes to `events` and exports `methods`.
    /// `outputs` are the names of `--output`.
    ///
    /// # Errors
    ///
    /// The socket fails, or the core refuses the `hello`, or another
    /// instance exports the methods.
    pub fn start(
        name: &str,
        outputs: Vec<String>,
        events: &[&str],
        methods: Vec<ExportMethod>,
    ) -> Result<Self, ClientError> {
        let mut rt = Self::connect(name, outputs)?;
        rt.announce(events, methods)?;
        Ok(rt)
    }

    /// Connects as `name` without events and methods, for a plugin that
    /// builds its scene before it calls [`Self::announce`].
    ///
    /// # Errors
    ///
    /// The socket fails, or the core refuses the `hello`.
    pub fn connect(name: &str, outputs: Vec<String>) -> Result<Self, ClientError> {
        Ok(Self {
            conn: Conn::connect(name)?,
            name: name.into(),
            outputs,
            grabs: Vec::new(),
        })
    }

    /// Subscribes to `events` and exports `methods`. The events of the
    /// changes before, such as the detach of the surfaces of a removed
    /// group, do not come.
    ///
    /// # Errors
    ///
    /// The socket fails, or another instance exports the methods.
    pub fn announce(
        &mut self,
        events: &[&str],
        methods: Vec<ExportMethod>,
    ) -> Result<(), ClientError> {
        self.conn.call::<_, Value>(
            "events.subscribe",
            Subscribe {
                events: events.iter().map(|&e| e.into()).collect(),
            },
        )?;
        self.conn
            .call::<_, Value>("methods.export", Export { names: methods })
            .map(drop)
    }

    /// Whether `--output` leaves an output to the plugin.
    #[must_use]
    pub fn manages(&self, output: &str) -> bool {
        self.outputs.is_empty() || self.outputs.iter().any(|o| o == output)
    }

    /// Whether `--output` names outputs.
    #[must_use]
    pub fn filters(&self) -> bool {
        !self.outputs.is_empty()
    }

    /// The outputs of the plugin, as the core has them.
    ///
    /// # Errors
    ///
    /// The socket fails.
    pub fn outputs(&mut self) -> Result<Vec<OutputInfo>, ClientError> {
        let mut outputs: Vec<OutputInfo> = self.conn.call("output.list", Value::Null)?;
        outputs.retain(|o| self.manages(&o.name));
        Ok(outputs)
    }

    /// Grabs a binding for an action. False, with a log line, when another
    /// client holds it.
    ///
    /// # Errors
    ///
    /// The socket fails.
    pub fn grab(&mut self, binding: Binding, action: A) -> Result<bool, ClientError> {
        let text = binding.to_string();
        match self.conn.call::<_, String>("input.grab", Grab { binding }) {
            Ok(normal) => {
                self.grabs.push((normal, action));
                Ok(true)
            }
            Err(ClientError::Rpc(err)) => {
                eprintln!("chora-{}: {text}: {err}", self.name);
                Ok(false)
            }
            Err(err) => Err(err),
        }
    }

    /// Lets all grabbed bindings go.
    ///
    /// # Errors
    ///
    /// The socket fails, or the core refuses an ungrab.
    ///
    /// # Panics
    ///
    /// The core named a grabbed binding that does not parse.
    pub fn ungrab_all(&mut self) -> Result<(), ClientError> {
        for (normal, _) in std::mem::take(&mut self.grabs) {
            let binding = normal.parse().expect("the core names valid bindings");
            self.conn
                .call::<_, Value>("input.ungrab", Grab { binding })?;
        }
        Ok(())
    }

    /// The action of a grabbed binding, as the core names it.
    #[must_use]
    pub fn action(&self, binding: &str) -> Option<&A> {
        self.grabs
            .iter()
            .find(|(b, _)| b == binding)
            .map(|(_, a)| a)
    }
}

/// Handles events until the connection fails.
///
/// # Errors
///
/// The socket fails or closes.
pub fn serve<P: Plugin>(plugin: &mut P) -> Result<(), ClientError> {
    loop {
        let event = plugin.runtime().conn.next_event()?;
        dispatch(plugin, event)?;
    }
}

/// Answers a method call, drops the events of outputs of others and hands
/// the rest to the plugin. The call may have ended before the answer, for
/// example when its caller left; that is no reason to exit. A plugin with
/// a loop of its own, for example for animations, calls it per event.
///
/// # Errors
///
/// The socket fails.
pub fn dispatch<P: Plugin>(plugin: &mut P, event: Event) -> Result<(), ClientError> {
    let rt = plugin.runtime();
    match event {
        Event::MethodCalled {
            call,
            method,
            params,
            ..
        } => {
            let result = match method.as_str() {
                "settings.get" => {
                    (plugin.settings().map(|s| s.get())).ok_or_else(|| no_settings("settings.get"))
                }
                "settings.set" => set_settings(plugin, params),
                "settings.reload" => reload_settings(plugin),
                _ => plugin.method(&method, params),
            };
            let rt = plugin.runtime();
            match rt.conn.answer(call, result.map_err(capped)) {
                Err(ClientError::Rpc(err)) => {
                    eprintln!("chora-{}: {err}", rt.name);
                    Ok(())
                }
                other => other,
            }
        }
        Event::OutputChanged { output } if !rt.manages(&output.name) => Ok(()),
        Event::OutputRemoved { name } if !rt.manages(&name) => Ok(()),
        event => plugin.event(event),
    }
}

/// `settings.set`: checks the change, has the plugin apply it, then saves
/// it. A change that the plugin cannot apply is taken back and not saved.
/// A change that cannot be saved stays applied, and the caller gets the
/// error.
///
/// # Errors
///
/// The plugin has no settings, a change is wrong, the plugin cannot apply
/// it, or the file cannot be written.
fn set_settings<P: Plugin>(plugin: &mut P, changes: Value) -> Result<Value, RpcError> {
    let no_settings = || no_settings("settings.set");
    let answer = plugin.settings().ok_or_else(no_settings)?.set(changes)?;
    apply(plugin)?;
    plugin
        .settings()
        .ok_or_else(no_settings)?
        .save()
        .map_err(|e| RpcError::new(e.code, format!("applied, not saved: {}", e.message)))?;
    Ok(answer)
}

/// The most bytes of the message of an error that a method answers.
const MAX_MESSAGE: usize = 512;

/// An error with its message cut to [`MAX_MESSAGE`] bytes. A parser may
/// quote a whole broken file, and an answer past the line limit of the
/// core costs the plugin its connection.
fn capped(mut err: RpcError) -> RpcError {
    if err.message.len() > MAX_MESSAGE {
        let mut end = MAX_MESSAGE;
        while !err.message.is_char_boundary(end) {
            end -= 1;
        }
        err.message.truncate(end);
        err.message.push('…');
    }
    err
}

/// The error of a settings method of a plugin without settings.
fn no_settings(method: &str) -> RpcError {
    RpcError::new(RpcError::METHOD_NOT_FOUND, method)
}

/// `settings.reload`: reads the settings file again, after an edit by
/// hand, and has the plugin apply it. A wrong file, or one that the plugin
/// cannot apply, changes nothing. Public for a binding of the plugin
/// that reloads, which has no method call to answer.
///
/// # Errors
///
/// The plugin has no settings, the file is wrong, or the plugin cannot
/// apply it.
pub fn reload_settings<P: Plugin>(plugin: &mut P) -> Result<Value, RpcError> {
    let answer = (plugin.settings())
        .ok_or_else(|| no_settings("settings.reload"))?
        .reload()?;
    apply(plugin)?;
    Ok(answer)
}

/// Has the plugin apply its settings; takes them back when it fails.
fn apply<P: Plugin>(plugin: &mut P) -> Result<(), RpcError> {
    plugin.settings_changed().inspect_err(|_| {
        if let Some(settings) = plugin.settings() {
            settings.undo();
        }
    })
}

/// An exported method with its description and the schema of its
/// parameters, as `methods.list` answers it.
#[must_use]
pub fn method(name: &str, description: &str, properties: Value, required: &[&str]) -> ExportMethod {
    let mut schema = json!({"type": "object", "required": required});
    schema["properties"] = properties;
    ExportMethod::Described(DescribedMethod {
        name: name.into(),
        description: Some(description.into()),
        input_schema: Some(schema),
        privileged: false,
    })
}

/// A [`method`] that a caller on a client socket may call only with the
/// capability `methods.privileged`: one that runs a command.
#[must_use]
pub fn privileged(method: ExportMethod) -> ExportMethod {
    match method {
        ExportMethod::Described(d) => ExportMethod::Described(DescribedMethod {
            privileged: true,
            ..d
        }),
        ExportMethod::Plain(name) => ExportMethod::Described(DescribedMethod {
            name,
            description: None,
            input_schema: None,
            privileged: true,
        }),
    }
}

/// Parses the parameters of an exported method.
///
/// # Errors
///
/// The parameters do not fit `T`.
pub fn params<T: DeserializeOwned>(params: Value) -> Result<T, RpcError> {
    serde_json::from_value(params)
        .map_err(|e| RpcError::new(RpcError::INVALID_PARAMS, e.to_string()))
}

/// Runs sessions of the plugin `name` until one ends without an error.
/// The core may close the connection and stay: the emergency reset
/// revokes grants and disconnects, and the scene survives it. Come back
/// in that case; give up when only failures follow within the episode.
pub fn run(
    name: &str,
    mut session: impl FnMut() -> Result<(), Box<dyn std::error::Error>>,
) -> ExitCode {
    let mut failing = Duration::ZERO;
    loop {
        let start = Instant::now();
        match session() {
            Ok(()) => return ExitCode::SUCCESS,
            Err(err) => {
                failing = next_failing(start.elapsed(), failing);
                if failing >= COME_BACK {
                    eprintln!("chora-{name}: {err}");
                    return ExitCode::FAILURE;
                }
                eprintln!("chora-{name}: {err}; I come back");
                sleep(TRY_AGAIN);
            }
        }
    }
}

/// How long a served session must last to start a new episode of errors.
const SERVED: Duration = Duration::from_millis(500);
/// How long one episode of failures may last before the plugin gives up.
const COME_BACK: Duration = Duration::from_secs(30);
/// The wait between two attempts.
const TRY_AGAIN: Duration = Duration::from_millis(500);

/// The failures of the current episode after a session of `served`.
fn next_failing(served: Duration, failing: Duration) -> Duration {
    if served >= SERVED {
        TRY_AGAIN
    } else {
        failing + served + TRY_AGAIN
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{
        File,
        tests::{DEFAULT, Demo, dir},
    };

    /// A plugin whose apply of settings fails on demand.
    struct Fake {
        settings: File<Demo>,
        fail: bool,
        applied: u16,
    }

    impl Plugin for Fake {
        type Action = ();

        fn runtime(&mut self) -> &mut Runtime<()> {
            unreachable!("set_settings needs no connection")
        }

        fn event(&mut self, _: Event) -> Result<(), ClientError> {
            Ok(())
        }

        fn method(&mut self, method: &str, _: Value) -> Result<Value, RpcError> {
            Err(RpcError::new(RpcError::METHOD_NOT_FOUND, method))
        }

        fn settings(&mut self) -> Option<&mut dyn Store> {
            Some(&mut self.settings)
        }

        fn settings_changed(&mut self) -> Result<(), RpcError> {
            if self.fail {
                return Err(RpcError::new(RpcError::REJECTED, "cannot apply"));
            }
            self.applied = self.settings.value.gap;
            Ok(())
        }
    }

    /// Check, apply, save: a change that fails to apply leaves the file
    /// as it was; a change that fails to save stays applied.
    #[test]
    fn set_settings_applies_before_it_saves() {
        let dir = dir("runtime-set");
        let path = dir.join("demo.yaml");
        let settings = File::<Demo>::load(path.clone(), DEFAULT).unwrap();
        let mut plugin = Fake {
            settings,
            fail: true,
            applied: 0,
        };
        assert!(set_settings(&mut plugin, json!({"gap": 8})).is_err());
        assert!(!path.exists(), "a change that fails to apply is not saved");
        assert_eq!(plugin.settings.value.gap, 4, "and taken back");
        plugin.fail = false;
        set_settings(&mut plugin, json!({"gap": 9})).unwrap();
        assert_eq!(plugin.applied, 9);
        assert!(std::fs::read_to_string(&path).unwrap().contains("gap: 9"));

        // A directory where the file should be: the save fails.
        let blocked = dir.join("blocked.yaml");
        plugin.settings = File::<Demo>::load(blocked.clone(), DEFAULT).unwrap();
        std::fs::create_dir_all(&blocked).unwrap();
        let err = set_settings(&mut plugin, json!({"gap": 10})).unwrap_err();
        assert_eq!(err.code, RpcError::REJECTED);
        assert_eq!(plugin.applied, 10, "the change stays applied");
        assert_eq!(plugin.settings.value.gap, 10);
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// An edit by hand applies with a reload; a wrong edit changes nothing.
    #[test]
    fn reload_settings_reads_the_file_and_applies_it() {
        let dir = dir("runtime-reload");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("demo.yaml");
        let settings = File::<Demo>::load(path.clone(), DEFAULT).unwrap();
        let mut plugin = Fake {
            settings,
            fail: false,
            applied: 0,
        };
        std::fs::write(&path, "gap: 6\n").unwrap();
        assert_eq!(reload_settings(&mut plugin).unwrap()["gap"], 6);
        assert_eq!(plugin.applied, 6);
        std::fs::write(&path, "gap: wide\n").unwrap();
        let err = reload_settings(&mut plugin).unwrap_err();
        assert_eq!(err.code, RpcError::INVALID_PARAMS);
        assert_eq!(plugin.settings.value.gap, 6);
        std::fs::write(&path, "gap: 7\n").unwrap();
        plugin.fail = true;
        assert!(reload_settings(&mut plugin).is_err());
        assert_eq!(plugin.settings.value.gap, 6, "a failed apply is taken back");
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// A broken file of 1 MiB made an error of 1 MiB, past the line limit
    /// of the core (W9).
    #[test]
    fn a_long_error_is_cut() {
        let long = RpcError::new(RpcError::INVALID_PARAMS, "ä".repeat(1 << 19));
        let message = capped(long).message;
        assert!(
            message.len() <= MAX_MESSAGE + '…'.len_utf8(),
            "{}",
            message.len()
        );
        assert!(message.ends_with('…'));
        assert_eq!(capped(RpcError::new(1, "short")).message, "short");
    }

    #[test]
    fn a_served_session_starts_a_new_episode_of_failures() {
        assert_eq!(next_failing(SERVED * 2, COME_BACK / 2), TRY_AGAIN);
    }

    #[test]
    fn an_episode_of_pure_failures_gives_up_after_come_back() {
        let mut failing = Duration::ZERO;
        for _ in 0..1000 {
            failing = next_failing(Duration::ZERO, failing);
            if failing >= COME_BACK {
                return;
            }
        }
        panic!("the plugin never gave up");
    }
}
