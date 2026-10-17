//! Client sockets (D33): `clients/NAME/ipc.sock` next to the main socket.
//! A connection through it carries the name NAME. It cannot say `hello`
//! with another name, and it cannot change grants or make client sockets.
//! It names only its own assets as files. The main socket refuses a
//! `hello` with a name that has a client socket.

use std::{os::unix::net::UnixListener, path::Path};

use chora_scene::Command;
use serde::Deserialize;
use serde_json::{Value, json};
use smithay::reexports::calloop::{Interest, Mode, PostAction, generic::Generic};

use super::{RpcError, private_dir, rejected};
use crate::state::State;

/// The most client sockets.
const MAX_SOCKETS: usize = 64;
/// The most connections of one client socket, as of one process.
pub(super) const MAX_PER_SOCKET: usize = 16;
/// The connections that client sockets never take, so that the main
/// socket keeps room.
pub(super) const KEPT_FOR_MAIN: usize = 32;
/// The longest path of a Unix socket, without its final zero.
const MAX_SOCKET_PATH: usize = 107;

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
        self.check_new_socket(conn, name)?;
        let base = (self.ipc.path.as_ref())
            .and_then(|p| p.parent())
            .ok_or_else(|| rejected("the core has no socket directory"))?;
        let path = base.join("clients").join(name).join("ipc.sock");
        if path.as_os_str().len() > MAX_SOCKET_PATH {
            return Err(rejected(format!("{name:?}: the socket path is too long")));
        }
        let listener = open(&path).map_err(|e| rejected(format!("{name:?}: {e}")))?;
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

    /// Why `clients.create` of a name must fail, if it must.
    fn check_new_socket(&self, conn: u64, name: &str) -> Result<(), RpcError> {
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
        // A live connection of the name would share the nodes and grants.
        if self.ipc.live(name) {
            return Err(rejected(format!(
                "{name:?} is connected or in its grace time"
            )));
        }
        if self.ipc.clients.len() >= MAX_SOCKETS {
            return Err(rejected(format!(
                "the core holds {MAX_SOCKETS} client sockets"
            )));
        }
        Ok(())
    }

    /// Removes all client sockets, for the emergency reset (D40). Their
    /// connections close with the others.
    pub fn remove_client_sockets(&mut self) {
        for (_, (token, path)) in self.ipc.clients.drain() {
            self.loop_handle.remove(token);
            let _ = std::fs::remove_file(&path);
            if let Some(dir) = path.parent() {
                let _ = std::fs::remove_dir(dir);
            }
        }
    }

    /// A connection of a client socket names only files that its client
    /// stored with `assets.put`, so that it learns nothing of other files.
    pub(super) fn check_paths(&self, conn: u64, commands: &[Command]) -> Result<(), RpcError> {
        let Some(name) = self.ipc.fixed(conn) else {
            return Ok(());
        };
        let foreign = commands
            .iter()
            .filter_map(|c| match c {
                Command::Create { props, .. }
                | Command::Update { props, .. }
                | Command::Animate { props, .. } => Some(props),
                _ => None,
            })
            .flat_map(|p| {
                let effects = p.effects.iter().flatten().map(|e| &e.shader);
                p.image.iter().chain(p.shader.iter()).chain(effects)
            })
            .find(|path| !self.ipc.uploads.owns(name, Path::new(path)));
        match foreign {
            Some(path) => Err(rejected(format!(
                "{path:?}: a client socket names only its own assets"
            ))),
            None => Ok(()),
        }
    }
}

/// Creates the private directories of a client socket and binds it. A
/// failed bind leaves no directory of the client behind.
fn open(path: &Path) -> std::io::Result<UnixListener> {
    let dir = path.parent().expect("the socket has a directory");
    private_dir(dir.parent().expect("clients/"))?;
    private_dir(dir)?;
    match std::fs::remove_file(path) {
        Err(err) if err.kind() != std::io::ErrorKind::NotFound => return Err(err),
        _ => {}
    }
    let bound = UnixListener::bind(path).and_then(|l| l.set_nonblocking(true).map(|()| l));
    if bound.is_err() {
        let _ = std::fs::remove_dir(dir);
    }
    bound
}
