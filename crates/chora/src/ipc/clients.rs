//! Client sockets (D33): `clients/NAME/ipc.sock` next to the main socket.
//! A connection through it carries the name NAME. It cannot say `hello`
//! with another name, and it cannot change grants or make client sockets.
//! The main socket refuses a `hello` with a name that has a client socket.

use std::{
    io,
    os::unix::{fs::DirBuilderExt, net::UnixListener},
    path::{Path, PathBuf},
};

use serde::Deserialize;
use serde_json::{Value, json};
use smithay::reexports::calloop::{Interest, Mode, PostAction, generic::Generic};

use super::{RpcError, rejected};
use crate::state::State;

/// Parameters of `clients.create`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Create {
    name: String,
}

impl State {
    /// `clients.create`: opens the client socket of a name and returns its
    /// path.
    pub(super) fn ipc_client_socket(
        &mut self,
        conn: u64,
        create: &Create,
    ) -> Result<Value, RpcError> {
        let name = &create.name;
        if self.ipc.fixed(conn).is_some() {
            return Err(rejected("a client socket cannot make client sockets"));
        }
        if !crate::methods::valid_name(name) {
            return Err(rejected(format!(
                "{name:?}: a client name for a socket has lowercase letters, digits, _, - and ., from a letter"
            )));
        }
        if self.ipc.clients.contains_key(name) {
            return Err(rejected(format!("{name:?} has a client socket already")));
        }
        let base = (self.ipc.path.as_ref())
            .and_then(|p| p.parent())
            .ok_or_else(|| rejected("the core has no socket directory"))?;
        let path = open(&base.join("clients").join(name))
            .map_err(|e| rejected(format!("{name:?}: {e}")))?;
        let listener = UnixListener::bind(&path).map_err(|e| rejected(e.to_string()))?;
        listener
            .set_nonblocking(true)
            .map_err(|e| rejected(e.to_string()))?;
        let fixed = name.clone();
        let token = self
            .loop_handle
            .insert_source(
                Generic::new(listener, Interest::READ, Mode::Level),
                move |_, listener, state| {
                    if let Some(error) = state.ipc_accept_all(listener, Some(&fixed)) {
                        tracing::warn!("client socket {fixed:?}: {error}");
                    }
                    Ok(PostAction::Continue)
                },
            )
            .map_err(|e| rejected(e.to_string()))?;
        self.ipc.clients.insert(name.clone(), (token, path.clone()));
        Ok(json!({"path": path}))
    }
}

/// Creates the directory of a client socket with mode 0700, and returns
/// the path of the socket, without an old socket file.
fn open(dir: &Path) -> io::Result<PathBuf> {
    std::fs::DirBuilder::new()
        .mode(0o700)
        .recursive(true)
        .create(dir)?;
    let path = dir.join("ipc.sock");
    match std::fs::remove_file(&path) {
        Err(err) if err.kind() != io::ErrorKind::NotFound => Err(err),
        _ => Ok(path),
    }
}
