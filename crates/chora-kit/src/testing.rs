//! A fake core for the tests of a plugin: the scene model of the core
//! behind a socket, without a renderer and without layout.
#![allow(
    clippy::unwrap_used,
    clippy::missing_panics_doc,
    reason = "a fake core for tests: a failure is a failed test"
)]

use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use chora_scene::{Command, ipc::RpcError};
use serde_json::{Value, json};

use crate::conn::Conn;

mod script;

pub use script::{Script, Subject, created, created_by, detach, output, owner, phases, surface};

/// The scene model of the fake core, shared with the test.
pub type Model = Arc<Mutex<Core>>;

/// What the fake core keeps of a connection.
#[derive(Default)]
pub struct Core {
    /// The scene that the plugin built.
    pub scene: chora_scene::Scene,
    /// The focus commands that the plugin sent.
    pub focuses: Vec<Option<String>>,
    /// The methods that the plugin called through `methods.call`.
    pub calls: Vec<String>,
    /// How often the plugin asked `scene.query`.
    pub queries: usize,
    /// The parameters of each `methods.call`: client, method and params.
    pub called: Vec<Value>,
    /// The events that the plugin emitted through `events.emit`.
    pub emits: Vec<Value>,
    /// The methods whose next call fails, in the order that they were planted.
    pub refusals: Vec<String>,
    /// The bindings that the plugin let go with `input.ungrab`.
    pub ungrabs: Vec<Value>,
    /// Whether `scene.query` answers every selector; else only `id:`.
    pub selects: bool,
    /// Whether `activation.token` mints a token; without, it answers null.
    pub mints_tokens: bool,
}

/// A fake core on a socket of its own, with one model for the
/// connections to it.
pub struct FakeCore {
    path: PathBuf,
    model: Model,
}

impl FakeCore {
    /// Starts a fake core that answers on a new socket.
    ///
    /// # Panics
    ///
    /// The socket cannot be bound.
    #[must_use]
    pub fn start() -> Self {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "chora-fake-core-{}-{}.sock",
            std::process::id(),
            COUNT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path).unwrap();
        let model = Model::default();
        let shared = model.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let model = shared.clone();
                std::thread::spawn(move || serve(&stream, &model));
            }
        });
        Self { path, model }
    }

    /// Makes `scene.query` answer every selector (`kind:`, `owner:`, ...)
    /// from the model, as the core does; by default it answers `id:` only.
    #[must_use]
    pub fn selecting(self) -> Self {
        self.model.lock().unwrap().selects = true;
        self
    }

    /// Plants a refusal: the next call of `method` fails with
    /// `REJECTED`, as when the client of a method is gone. The call after
    /// it passes again.
    pub fn refuse_next(&self, method: &str) {
        self.model.lock().unwrap().refusals.push(method.to_owned());
    }

    /// Makes `activation.token` mint a new token on each call, as the
    /// core does for a plugin that holds an act.
    pub fn mint_tokens(&self) {
        self.model.lock().unwrap().mints_tokens = true;
    }

    /// The scene model, shared with the fake core.
    #[must_use]
    pub fn model(&self) -> Model {
        self.model.clone()
    }

    /// Connects a plugin named `name`.
    ///
    /// # Panics
    ///
    /// The connection fails.
    #[must_use]
    #[allow(
        clippy::disallowed_methods,
        reason = "the test connects to its fake core"
    )]
    pub fn connect(&self, name: &str) -> Conn {
        Conn::connect_at(name, &self.path).unwrap()
    }
}

impl Drop for FakeCore {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Answers the requests of one connection.
fn serve(stream: &UnixStream, model: &Model) {
    let mut writer = stream.try_clone().unwrap();
    let mut owner = String::new();
    for line in BufReader::new(stream).lines().map_while(Result::ok) {
        let request: Value = serde_json::from_str(&line).unwrap();
        if request["method"] == "hello" {
            request["params"]["name"]
                .as_str()
                .unwrap_or_default()
                .clone_into(&mut owner);
        }
        let answer = match answer(model, &owner, &request) {
            Ok(result) => json!({"jsonrpc": "2.0", "id": request["id"], "result": result}),
            Err((code, message)) => json!({"jsonrpc": "2.0", "id": request["id"],
                "error": {"code": code, "message": message}}),
        };
        writeln!(writer, "{answer}").unwrap();
    }
}

fn answer(model: &Model, owner: &str, request: &Value) -> Result<Value, (i64, String)> {
    let params = &request["params"];
    let method = request["method"].as_str().unwrap_or_default();
    {
        let mut core = model.lock().unwrap();
        if let Some(at) = core.refusals.iter().position(|m| m == method) {
            core.refusals.remove(at);
            return Err((RpcError::REJECTED, format!("{method} is refused")));
        }
    }
    match method {
        "hello" => Ok(json!({"version": chora_scene::ipc::VERSION})),
        "scene.apply" => {
            let commands: Vec<Command> =
                serde_json::from_value(params["commands"].clone()).unwrap();
            let mut core = model.lock().unwrap();
            core.focuses.extend(commands.iter().filter_map(|c| match c {
                Command::Focus { surface } => Some(surface.clone()),
                _ => None,
            }));
            match core.scene.apply(owner, &commands) {
                Ok(_) => Ok(Value::Null),
                Err(e) => Err((
                    RpcError::REJECTED,
                    format!("command {}: {}", e.index, e.error),
                )),
            }
        }
        "activation.token" if model.lock().unwrap().mints_tokens => Ok(activation_token()),
        "input.grab" => Ok(params["binding"].clone()),
        "input.ungrab" => {
            let mut core = model.lock().unwrap();
            core.ungrabs.push(params["binding"].clone());
            Ok(Value::Null)
        }
        "methods.call" => {
            let method = params["method"].as_str().unwrap_or_default().to_owned();
            let mut core = model.lock().unwrap();
            core.calls.push(method);
            core.called.push(params.clone());
            Ok(Value::Null)
        }
        "output.list" => Ok(json!([])),
        "events.emit" => {
            model.lock().unwrap().emits.push(params.clone());
            Ok(Value::Null)
        }
        "scene.query" => Ok(query(
            model,
            params["selector"].as_str().unwrap_or_default(),
        )),
        _ => Ok(Value::Null),
    }
}

/// The nodes that a selector finds: every selector when the core selects,
/// else only `id:`.
fn query(model: &Model, selector: &str) -> Value {
    let mut core = model.lock().unwrap();
    core.queries += 1;
    let found: Vec<Value> = if core.selects {
        let selector = selector.parse::<chora_scene::Selector>().unwrap();
        (core.scene.query(&selector).into_iter())
            .filter_map(|n| serde_json::to_value(n).ok())
            .collect()
    } else {
        let id = selector.strip_prefix("id:");
        (id.and_then(|id| core.scene.node(id)))
            .and_then(|n| serde_json::to_value(n).ok())
            .into_iter()
            .collect()
    };
    Value::Array(found)
}

/// A new activation token, as the core mints one for a plugin.
fn activation_token() -> Value {
    static TOKENS: AtomicUsize = AtomicUsize::new(0);
    let n = TOKENS.fetch_add(1, Ordering::Relaxed);
    json!({"token": format!("fake-token-{}-{n}", std::process::id())})
}

/// Whether the model has a node.
pub fn has(model: &Model, id: &str) -> bool {
    model.lock().unwrap().scene.node(id).is_some()
}

/// The text of a text node of the model.
pub fn text(model: &Model, id: &str) -> Option<String> {
    model.lock().unwrap().scene.node(id)?.props.text.clone()
}
