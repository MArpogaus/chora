//! The connection to the core: the blocking client of `chora-scene`, with
//! a timeout for events, so that the clock ticks while no event comes.

use std::{io::ErrorKind, os::unix::net::UnixStream, time::Duration};

use chora_scene::{
    Command, Event, Node,
    ipc::{Apply, Client, ClientError, Hello, Query, RpcError, SOCKET_ENV},
};
use serde_json::Value;

/// A connection with the name of the plugin.
pub struct Conn {
    client: Client,
    /// A second handle of the socket, to set the read timeout.
    stream: UnixStream,
}

impl Conn {
    /// Connects to the socket in `CHORA_SOCKET` and sends `hello`.
    pub fn connect(name: &str) -> Result<Self, ClientError> {
        let path = std::env::var_os(SOCKET_ENV).ok_or(ClientError::NoSocket)?;
        let stream = UnixStream::connect(path)?;
        let handle = stream.try_clone()?;
        let hello = Hello {
            name: name.into(),
            keep_nodes: false,
        };
        Ok(Self {
            client: Client::from_stream(stream, &hello)?,
            stream: handle,
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
    pub fn next_event(&mut self, timeout: Duration) -> Result<Option<Event>, ClientError> {
        self.stream
            .set_read_timeout(Some(timeout.max(Duration::from_millis(1))))?;
        let event = self.client.next_event();
        self.stream.set_read_timeout(None)?;
        match event {
            Ok(event) => Ok(Some(event)),
            Err(ClientError::Io(err))
                if matches!(err.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) =>
            {
                Ok(None)
            }
            // A line that a timeout cut off; the next line is whole again.
            Err(ClientError::Json(err)) => {
                eprintln!("chora-xp: skipped a message: {err}");
                Ok(None)
            }
            Err(err) => Err(err),
        }
    }

    /// Applies a transaction and returns the error of the core, if any.
    pub fn try_apply(&mut self, commands: Vec<Command>) -> Result<Option<RpcError>, ClientError> {
        if commands.is_empty() {
            return Ok(None);
        }
        match self
            .client
            .call::<_, Value>("scene.apply", Apply { commands })
        {
            Ok(_) => Ok(None),
            Err(ClientError::Rpc(err)) => Ok(Some(err)),
            Err(err) => Err(err),
        }
    }

    /// Applies a transaction. A rejected transaction only logs: a surface
    /// can disappear between an event and the answer to it.
    pub fn apply(&mut self, commands: Vec<Command>) -> Result<(), ClientError> {
        if let Some(err) = self.try_apply(commands)? {
            eprintln!("chora-xp: {err}");
        }
        Ok(())
    }

    /// The nodes that match a selector.
    pub fn query(&mut self, selector: &str) -> Result<Vec<Node>, ClientError> {
        self.client.call(
            "scene.query",
            Query {
                selector: selector.into(),
            },
        )
    }
}
