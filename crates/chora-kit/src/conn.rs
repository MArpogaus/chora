//! The connection skeleton that every plugin repeats: `hello` with the
//! name of the plugin, calls, node queries, and a transaction that
//! tolerates a surface that closed between the event and the answer.

use std::time::Duration;

use chora_scene::{
    Command, Event, Node,
    ipc::{Client, ClientError, Hello, Query, RpcError},
};
use serde_json::Value;

/// A connection with the name of the plugin, for the log lines.
pub struct Conn {
    client: Client,
    name: String,
    /// Whether the plugin handles a method call of a confined caller now.
    pub(crate) serving_confined: bool,
}

impl Conn {
    /// Connects to the socket in `CHORA_SOCKET` and sends `hello` with
    /// `resume`: a restart of the plugin takes its nodes again, and a
    /// second instance must not take the nodes of a running one.
    ///
    /// # Errors
    ///
    /// The socket is missing or fails, or the core refuses the `hello`.
    pub fn connect(name: &str) -> Result<Self, ClientError> {
        let path = std::env::var_os(chora_scene::ipc::SOCKET_ENV).ok_or(ClientError::NoSocket)?;
        Self::connect_at(name, path.as_ref())
    }

    /// Connects as [`Self::connect`] does, to the socket at `path`; for
    /// the tests of a plugin against a fake core.
    ///
    /// # Errors
    ///
    /// The socket fails, or the core refuses the `hello`.
    pub fn connect_at(name: &str, path: &std::path::Path) -> Result<Self, ClientError> {
        let hello = Hello {
            name: name.into(),
            resume: true,
            ..Hello::default()
        };
        Ok(Self {
            client: Client::connect(path, &hello)?,
            name: name.into(),
            serving_confined: false,
        })
    }

    /// The client name of the plugin.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// An activation token of the plugin for [`crate::run`] and
    /// [`crate::launch`]. It carries the act of the person when the act
    /// went to this plugin: a key or button it grabbed, a key to its node
    /// or a press on its node. `None` when the core refuses one, and while
    /// the plugin handles a method call of a confined caller, so that such
    /// a caller never turns an earlier act of the person into a token; the
    /// core refuses the stamp then too.
    pub fn activation_token(&mut self) -> Option<String> {
        if self.serving_confined {
            return None;
        }
        let answer: Value = self.call("activation.token", Value::Null).ok()?;
        answer["token"].as_str().map(str::to_owned)
    }

    /// Sends a request and waits for its response.
    ///
    /// # Errors
    ///
    /// The socket fails, or the core answers with an error.
    pub fn call<P: serde::Serialize, R: serde::de::DeserializeOwned>(
        &mut self,
        method: &str,
        params: P,
    ) -> Result<R, ClientError> {
        self.client.call(method, params)
    }

    /// Calls a method of the core as [`Self::call`] does, and gives up
    /// after `timeout` without an answer.
    ///
    /// # Errors
    ///
    /// The socket fails, the core answers with an error, or the time ends.
    pub fn call_within<P: serde::Serialize, R: serde::de::DeserializeOwned>(
        &mut self,
        method: &str,
        params: P,
        timeout: Duration,
    ) -> Result<R, ClientError> {
        self.client.call_within(method, params, timeout)
    }

    /// Answers a call of an exported method.
    ///
    /// # Errors
    ///
    /// The socket fails.
    pub fn answer(
        &mut self,
        call: u64,
        result: Result<Value, RpcError>,
    ) -> Result<(), ClientError> {
        self.client.answer(call, result)
    }

    /// The next event, from the socket.
    ///
    /// # Errors
    ///
    /// The socket fails or closes.
    pub fn next_event(&mut self) -> Result<Event, ClientError> {
        self.client.next_event()
    }

    /// The next event, or `None` after `timeout` without one. The timeout
    /// drives the clock and the frames of the effects.
    ///
    /// # Errors
    ///
    /// The socket fails or closes.
    pub fn next_event_timeout(&mut self, timeout: Duration) -> Result<Option<Event>, ClientError> {
        self.client.next_event_timeout(timeout)
    }

    /// Applies a transaction and returns the error of the core, if any.
    ///
    /// # Errors
    ///
    /// The socket fails.
    pub fn try_apply(&mut self, commands: Vec<Command>) -> Result<Option<RpcError>, ClientError> {
        self.client.apply(commands)
    }

    /// Applies a transaction. A rejected transaction only logs: a surface
    /// can disappear between an event and the answer to it.
    ///
    /// # Errors
    ///
    /// The socket fails.
    pub fn apply(&mut self, commands: Vec<Command>) -> Result<(), ClientError> {
        if let Some(err) = self.try_apply(commands)?
            && !err.is_gone_surface()
        {
            eprintln!("{}: {err}", self.name);
        }
        Ok(())
    }

    /// The nodes that match a selector.
    ///
    /// # Errors
    ///
    /// The socket fails, or a node cannot be read.
    pub fn query(&mut self, selector: &str) -> Result<Vec<Node>, ClientError> {
        let nodes: Vec<Value> = self.call("scene.query", Query::new(selector))?;
        nodes.into_iter().map(node).collect()
    }
}

/// A node of a query. The core leaves out `effects` when a node has none,
/// but `Node` needs the field, so it comes back here as an empty list.
fn node(mut value: Value) -> Result<Node, ClientError> {
    if let Some(object) = value.as_object_mut() {
        object
            .entry("effects")
            .or_insert_with(|| Value::Array(Vec::new()));
    }
    Ok(serde_json::from_value(value)?)
}

/// A core that answers from a script, for the tests of the kit.
#[cfg(test)]
pub(crate) mod fake_core {
    use std::{
        io::{BufRead, BufReader, Write},
        os::unix::net::UnixListener,
        path::PathBuf,
        sync::atomic::{AtomicU32, Ordering},
        thread::JoinHandle,
    };

    use serde_json::{Value, json};

    /// What the core answers to a request: a result, an error message, or
    /// nothing at all.
    pub(crate) enum Answer {
        Result(Value),
        Error(&'static str),
        Silence,
    }

    static COUNT: AtomicU32 = AtomicU32::new(0);

    /// Serves one connection with `script`, which gets the method and the
    /// parameters of each request, until the client leaves. The thread
    /// returns the methods that came.
    pub(crate) fn serve(
        script: impl Fn(&str, &Value) -> Answer + Send + 'static,
    ) -> (PathBuf, JoinHandle<Vec<String>>) {
        let path = std::env::temp_dir().join(format!(
            "chora-kit-fake-{}-{}.sock",
            std::process::id(),
            COUNT.fetch_add(1, Ordering::Relaxed)
        ));
        let listener = UnixListener::bind(&path).expect("bind the fake core");
        let thread = std::thread::spawn(move || {
            let (stream, _) = listener.accept().expect("a client");
            let mut writer = stream.try_clone().expect("clone");
            let mut methods = Vec::new();
            for line in BufReader::new(stream).lines().map_while(Result::ok) {
                let request: Value = serde_json::from_str(&line).expect("json");
                let method = request["method"].as_str().unwrap_or_default().to_owned();
                let id = request["id"].clone();
                let reply = match script(&method, &request["params"]) {
                    Answer::Result(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
                    Answer::Error(message) => {
                        json!({"jsonrpc": "2.0", "id": id, "error": {"code": 1, "message": message}})
                    }
                    Answer::Silence => {
                        methods.push(method);
                        continue;
                    }
                };
                methods.push(method);
                writeln!(writer, "{reply}").expect("reply");
            }
            methods
        });
        (path, thread)
    }
}
