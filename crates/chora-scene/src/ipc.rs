//! The JSON-RPC 2.0 protocol of the IPC socket, and a blocking client.
//!
//! Every message is one line of JSON. Events arrive as notifications with
//! the method `event`.

use std::{
    collections::VecDeque,
    io::{self, BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::Path,
};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::{Command, Event};

/// The environment variable with the path of the IPC socket.
pub const SOCKET_ENV: &str = "CHORA_SOCKET";

/// A request or a notification from the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Always `2.0`.
    pub jsonrpc: String,
    /// The request id; missing for notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    /// The method name, for example `scene.apply`.
    pub method: String,
    /// The parameters.
    #[serde(default)]
    pub params: Value,
}

/// A message from the core: a response or an event notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Always `2.0`.
    pub jsonrpc: String,
    /// The id of the request; missing for notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    /// The result of a successful request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// The error of a failed request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    /// `event` for notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// The event of a notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// A JSON-RPC error object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[error("{message} (code {code})")]
pub struct RpcError {
    /// The error code.
    pub code: i64,
    /// A readable description.
    pub message: String,
}

impl RpcError {
    /// The method does not exist.
    pub const METHOD_NOT_FOUND: i64 = -32601;
    /// The parameters are invalid.
    pub const INVALID_PARAMS: i64 = -32602;
    /// The request breaks a rule of the scene or of the core.
    pub const REJECTED: i64 = 1;

    /// An error with a code and a message.
    pub fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl Message {
    /// A successful response.
    #[must_use]
    pub fn result(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id: Some(id),
            result: Some(result),
            error: None,
            method: None,
            params: None,
        }
    }

    /// A failed response.
    #[must_use]
    pub fn error(id: Value, error: RpcError) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id: Some(id),
            result: None,
            error: Some(error),
            method: None,
            params: None,
        }
    }

    /// An event notification.
    ///
    /// # Panics
    ///
    /// Never: events always serialize.
    #[must_use]
    pub fn event(event: &Event) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id: None,
            result: None,
            error: None,
            method: Some("event".into()),
            params: Some(serde_json::to_value(event).expect("events always serialize")),
        }
    }
}

/// Parameters of `hello`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hello {
    /// The client name, the owner of the nodes of this client.
    pub name: String,
    /// Keep the nodes after the last connection of the name closes.
    #[serde(default)]
    pub keep_nodes: bool,
}

/// Parameters of `scene.query`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    /// See [`crate::Selector`]; every node if missing.
    #[serde(default)]
    pub selector: String,
}

/// Parameters of `scene.apply`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Apply {
    /// The commands of the transaction.
    pub commands: Vec<Command>,
}

/// Parameters of `events.subscribe`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Subscribe {
    /// Event names, for example `surface.created`; every event if empty.
    #[serde(default)]
    pub events: Vec<String>,
}

/// Parameters of `input.grab`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grab {
    /// The binding, for example `Super+Return`.
    pub binding: crate::Binding,
}

/// Errors of the blocking [`Client`].
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// The socket failed.
    #[error("socket: {0}")]
    Io(#[from] io::Error),
    /// A message was no valid JSON.
    #[error("invalid message: {0}")]
    Json(#[from] serde_json::Error),
    /// The core rejected the request.
    #[error(transparent)]
    Rpc(#[from] RpcError),
    /// The core closed the connection.
    #[error("the core closed the connection")]
    Closed,
    /// `CHORA_SOCKET` is not set.
    #[error("{SOCKET_ENV} is not set")]
    NoSocket,
}

/// A blocking client of the IPC socket.
pub struct Client {
    reader: BufReader<UnixStream>,
    writer: UnixStream,
    next_id: u64,
    events: VecDeque<Event>,
}

impl Client {
    /// Connects to the socket in `CHORA_SOCKET` and sends `hello`.
    ///
    /// # Errors
    ///
    /// The variable is missing, or the connection fails.
    pub fn from_env(hello: &Hello) -> Result<Self, ClientError> {
        let path = std::env::var_os(SOCKET_ENV).ok_or(ClientError::NoSocket)?;
        Self::connect(path, hello)
    }

    /// Connects to a socket and sends `hello`.
    ///
    /// # Errors
    ///
    /// The connection or the `hello` request fails.
    pub fn connect(path: impl AsRef<Path>, hello: &Hello) -> Result<Self, ClientError> {
        Self::from_stream(UnixStream::connect(path)?, hello)
    }

    /// Uses a connected stream and sends `hello`.
    ///
    /// # Errors
    ///
    /// The `hello` request fails.
    pub fn from_stream(writer: UnixStream, hello: &Hello) -> Result<Self, ClientError> {
        let reader = BufReader::new(writer.try_clone()?);
        let mut client = Self {
            reader,
            writer,
            next_id: 1,
            events: VecDeque::new(),
        };
        client.call::<_, Value>("hello", hello)?;
        Ok(client)
    }

    /// Sends a request and waits for its response. Events that arrive in
    /// between wait in a queue for [`Client::next_event`].
    ///
    /// # Errors
    ///
    /// The socket fails, or the core rejects the request.
    pub fn call<P: Serialize, R: DeserializeOwned>(
        &mut self,
        method: &str,
        params: P,
    ) -> Result<R, ClientError> {
        let id = self.next_id;
        self.next_id += 1;
        let request = Request {
            jsonrpc: "2.0".into(),
            id: Some(id.into()),
            method: method.into(),
            params: serde_json::to_value(params)?,
        };
        let mut line = serde_json::to_string(&request)?;
        line.push('\n');
        self.writer.write_all(line.as_bytes())?;
        loop {
            let message = self.read()?;
            // An error without id answers a request that the core could not
            // parse, so it belongs to this call.
            let mine = match &message.id {
                Some(v) => v.as_u64() == Some(id),
                None => message.error.is_some(),
            };
            if !mine {
                self.queue(message);
                continue;
            }
            if let Some(error) = message.error {
                return Err(error.into());
            }
            return Ok(serde_json::from_value(
                message.result.unwrap_or(Value::Null),
            )?);
        }
    }

    /// The next event, from the queue or from the socket.
    ///
    /// # Errors
    ///
    /// The socket fails or closes.
    pub fn next_event(&mut self) -> Result<Event, ClientError> {
        loop {
            if let Some(event) = self.events.pop_front() {
                return Ok(event);
            }
            let message = self.read()?;
            self.queue(message);
        }
    }

    /// Keeps an event notification. Events of a newer core that this crate
    /// does not know are skipped.
    fn queue(&mut self, message: Message) {
        if message.method.as_deref() != Some("event") {
            return;
        }
        if let Some(event) = message.params.and_then(|p| serde_json::from_value(p).ok()) {
            self.events.push_back(event);
        }
    }

    fn read(&mut self) -> Result<Message, ClientError> {
        let mut line = String::new();
        if self.reader.read_line(&mut line)? == 0 {
            return Err(ClientError::Closed);
        }
        Ok(serde_json::from_str(&line)?)
    }
}

#[cfg(test)]
mod tests {
    use std::{io::BufRead, thread};

    use super::*;

    /// A fake core that answers `hello`, then writes `lines` after the next
    /// request.
    fn fake_core(lines: &'static [&'static str]) -> UnixStream {
        let (client, server) = UnixStream::pair().unwrap();
        thread::spawn(move || {
            let mut reader = BufReader::new(server.try_clone().unwrap());
            let mut writer = server;
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            writeln!(writer, r#"{{"jsonrpc":"2.0","id":1,"result":null}}"#).unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            for l in lines {
                writeln!(writer, "{l}").unwrap();
            }
        });
        client
    }

    fn hello() -> Hello {
        Hello {
            name: "test".into(),
            keep_nodes: false,
        }
    }

    #[test]
    fn unknown_events_do_not_break_a_call() {
        let stream = fake_core(&[
            r#"{"jsonrpc":"2.0","method":"event","params":{"event":"output.added"}}"#,
            r#"{"jsonrpc":"2.0","id":2,"result":[]}"#,
        ]);
        let mut client = Client::from_stream(stream, &hello()).unwrap();
        let nodes: Vec<Value> = client.call("scene.query", Value::Null).unwrap();
        assert!(nodes.is_empty());
    }

    #[test]
    fn an_error_without_id_ends_the_call() {
        let stream = fake_core(&[
            r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"parse error"}}"#,
        ]);
        let mut client = Client::from_stream(stream, &hello()).unwrap();
        let err = client
            .call::<_, Value>("scene.query", Value::Null)
            .unwrap_err();
        assert!(
            matches!(err, ClientError::Rpc(RpcError { code: -32700, .. })),
            "{err}"
        );
    }
}
