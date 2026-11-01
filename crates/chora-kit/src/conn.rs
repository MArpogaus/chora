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
        })
    }

    /// The client name of the plugin.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
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
