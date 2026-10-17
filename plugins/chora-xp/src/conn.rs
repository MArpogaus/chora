//! The connection to the core: the blocking client of `chora-scene`. Its
//! timeout for events drives the clock and the frames of the effects.

use std::time::Duration;

use chora_scene::{
    Command, Event, Node,
    ipc::{Client, ClientError, Hello, Query, RpcError},
};
use serde_json::Value;

/// A connection with the name of the plugin.
pub struct Conn {
    client: Client,
}

impl Conn {
    /// Connects to the socket in `CHORA_SOCKET` and sends `hello`.
    pub fn connect(name: &str) -> Result<Self, ClientError> {
        let hello = Hello {
            name: name.into(),
            keep_nodes: false,
            resume: true,
        };
        Ok(Self {
            client: Client::from_env(&hello)?,
        })
    }

    /// Sends a request and waits for its response.
    pub fn call<P: serde::Serialize, R: serde::de::DeserializeOwned>(
        &mut self,
        method: &str,
        params: P,
    ) -> Result<R, ClientError> {
        self.client.call(method, params)
    }

    /// The next event, or `None` after `timeout` without one.
    /// A signal that interrupts the wait is no event: the read of a
    /// socket with a timeout does not restart.
    pub fn next_event(&mut self, timeout: Duration) -> Result<Option<Event>, ClientError> {
        match self.client.next_event_timeout(timeout) {
            Err(ClientError::Io(e)) if e.kind() == std::io::ErrorKind::Interrupted => Ok(None),
            other => other,
        }
    }

    /// Applies a transaction and returns the error of the core, if any.
    /// Commands on windows that closed in the meantime drop out.
    pub fn try_apply(&mut self, commands: Vec<Command>) -> Result<Option<RpcError>, ClientError> {
        self.client.apply(commands)
    }

    /// Applies a transaction. A rejected transaction only logs: a surface
    /// can disappear between an event and the answer to it.
    pub fn apply(&mut self, commands: Vec<Command>) -> Result<(), ClientError> {
        if let Some(err) = self.try_apply(commands)?
            && !err.is_gone_surface()
        {
            eprintln!("chora-xp: {err}");
        }
        Ok(())
    }

    /// The nodes that match a selector.
    pub fn query(&mut self, selector: &str) -> Result<Vec<Node>, ClientError> {
        let nodes: Vec<Value> = self.client.call(
            "scene.query",
            Query {
                selector: selector.into(),
            },
        )?;
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
