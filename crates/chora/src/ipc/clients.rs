//! Client sockets (D33): `clients/NAME/ipc.sock` next to the main socket.
//! A connection through it carries the name NAME. It cannot say `hello`
//! with another name.
//! It names only its own assets as files. The main socket refuses a
//! `hello` with a name that has a client socket.

use std::{
    os::unix::net::{UnixListener, UnixStream},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

use chora_scene::ipc::Capability;
use smithay::reexports::{
    calloop::RegistrationToken,
    wayland_server::backend::{ClientId, DisconnectReason},
};

use chora_scene::Command;
use serde::Deserialize;
use serde_json::{Value, json};
use smithay::reexports::calloop::{Interest, Mode, PostAction, generic::Generic};

use super::{RpcError, Socket, private_dir, rejected};
use crate::state::{ClientState, Slot, State};

/// The most client sockets.
const MAX_SOCKETS: usize = 64;
/// The most connections of one client socket, as of one process.
pub(super) const MAX_PER_SOCKET: usize = 16;
/// The most Wayland clients of one client socket, as of its IPC socket.
#[allow(clippy::cast_possible_truncation)] // 16
const MAX_PER_SOCKET_U32: u32 = MAX_PER_SOCKET as u32;
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
    /// `clients.create`: opens the client socket and the Wayland socket of
    /// a name, and returns their paths.
    pub(super) fn ipc_client_socket(&mut self, create: &Create) -> Result<Value, RpcError> {
        let name = &create.name;
        self.check_new_socket(name)?;
        let base = (self.ipc.path.as_ref())
            .and_then(|p| p.parent())
            .ok_or_else(|| rejected("the core has no socket directory"))?;
        let dir = base.join("clients").join(name);
        if dir.join(WAYLAND).as_os_str().len() > MAX_SOCKET_PATH {
            return Err(rejected(format!("{name:?}: the socket path is too long")));
        }
        let (ipc, wayland) = open(&dir).map_err(|e| rejected(format!("{name:?}: {e}")))?;
        let fixed = Socket::Client(name.clone());
        let path = dir.join(IPC);
        let ipc = self.loop_handle.insert_source(
            Generic::new(ipc, Interest::READ, Mode::Level),
            move |_, listener, state| {
                if let Some(error) = state.ipc_accept_all(listener, &fixed) {
                    tracing::warn!("{}: {error}", path.display());
                }
                Ok(PostAction::Continue)
            },
        );
        let owner = name.clone();
        let count = Arc::new(AtomicU32::new(0));
        let wayland = self.loop_handle.insert_source(
            Generic::new(wayland, Interest::READ, Mode::Level),
            move |_, listener, state| {
                while let Ok((stream, _)) = listener.accept() {
                    // Beyond the cap, the stream closes at once.
                    if count.load(Ordering::Relaxed) < MAX_PER_SOCKET_U32 {
                        state.insert_named_client(stream, &owner, &count);
                    }
                }
                Ok(PostAction::Continue)
            },
        );
        let (Ok(ipc), Ok(wayland)) = (ipc, wayland) else {
            return Err(rejected("no event source for a client socket"));
        };
        self.ipc.clients.insert(
            name.clone(),
            ClientSocket {
                ipc,
                wayland,
                dir: dir.clone(),
                clients: Vec::new(),
            },
        );
        Ok(json!({"path": dir.join(IPC), "wayland": dir.join(WAYLAND)}))
    }

    /// Inserts a Wayland client of the client socket of a name, with the
    /// grants of the name at this moment.
    fn insert_named_client(&mut self, stream: UnixStream, name: &str, count: &Arc<AtomicU32>) {
        let granted = [Capability::InputInject, Capability::ScreenCapture]
            .into_iter()
            .filter(|c| self.grants.allows(name, *c))
            .collect();
        count.fetch_add(1, Ordering::Relaxed);
        let data = ClientState {
            name: Some(name.into()),
            granted,
            _slot: Some(Slot(count.clone())),
            ..ClientState::default()
        };
        match self.display_handle.insert_client(stream, Arc::new(data)) {
            Ok(client) => {
                if let Some(socket) = self.ipc.clients.get_mut(name) {
                    socket.clients.push(client.id());
                }
            }
            Err(err) => tracing::warn!(?err, "a wayland client of {name:?} failed to connect"),
        }
    }

    /// Disconnects the Wayland clients of the client socket of a name that
    /// got one of the capabilities, or all of them without capabilities,
    /// so that a revoke ends what a client got at its connection.
    pub fn disconnect_wayland(&mut self, name: &str, capabilities: &[Capability]) {
        let Some(socket) = self.ipc.clients.get_mut(name) else {
            return;
        };
        let handle = self.display_handle.backend_handle();
        let mut killed = Vec::new();
        socket.clients.retain(|id| {
            let Ok(data) = handle.get_client_data(id.clone()) else {
                return false;
            };
            let granted = data
                .downcast_ref::<ClientState>()
                .map_or(&[][..], |c| &c.granted);
            let hit = capabilities.is_empty() || capabilities.iter().any(|c| granted.contains(c));
            if hit {
                handle.kill_client(id.clone(), DisconnectReason::ConnectionClosed);
                killed.push(id.clone());
            }
            !hit
        });
        // A killed client stays until its next request; its frames that
        // wait for a tick must not get the screen.
        let gone = |client: Option<ClientId>| client.is_some_and(|id| killed.contains(&id));
        self.copies.wlr.retain(|c| !gone(c.client()));
        self.copies.ext.retain(|w| !gone(w.client()));
    }

    /// Why `clients.create` of a name must fail, if it must.
    fn check_new_socket(&self, name: &str) -> Result<(), RpcError> {
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
        let names: Vec<String> = self.ipc.clients.keys().cloned().collect();
        for name in names {
            self.disconnect_wayland(&name, &[]);
        }
        for (_, socket) in self.ipc.clients.drain() {
            self.loop_handle.remove(socket.ipc);
            self.loop_handle.remove(socket.wayland);
            socket.remove_files();
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

/// The file names of the two sockets in the directory of a client.
const IPC: &str = "ipc.sock";
const WAYLAND: &str = "wayland.sock";

/// The listeners of the IPC and the Wayland socket of a client name, and
/// their directory.
pub(super) struct ClientSocket {
    pub ipc: RegistrationToken,
    wayland: RegistrationToken,
    dir: PathBuf,
    /// The Wayland clients that connected through it.
    clients: Vec<ClientId>,
}

impl ClientSocket {
    /// Removes the sockets and their directory.
    pub fn remove_files(&self) {
        remove(&self.dir);
    }
}

/// Creates the private directories of a client and binds its IPC and its
/// Wayland socket. A failure leaves no socket or directory of the client.
fn open(dir: &Path) -> std::io::Result<(UnixListener, UnixListener)> {
    private_dir(dir.parent().expect("clients/"))?;
    private_dir(dir)?;
    let bound = bind(&dir.join(IPC)).and_then(|ipc| Ok((ipc, bind(&dir.join(WAYLAND))?)));
    if bound.is_err() {
        remove(dir);
    }
    bound
}

fn bind(path: &Path) -> std::io::Result<UnixListener> {
    match std::fs::remove_file(path) {
        Err(err) if err.kind() != std::io::ErrorKind::NotFound => return Err(err),
        _ => {}
    }
    let listener = UnixListener::bind(path)?;
    listener.set_nonblocking(true)?;
    Ok(listener)
}

fn remove(dir: &Path) {
    let _ = std::fs::remove_file(dir.join(IPC));
    let _ = std::fs::remove_file(dir.join(WAYLAND));
    let _ = std::fs::remove_dir(dir);
}

#[cfg(test)]
mod tests;
