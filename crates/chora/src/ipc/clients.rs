//! Client sockets (D33): `clients/NAME/ipc.sock` next to the main socket.
//! A connection through it carries the name NAME. It cannot say `hello`
//! with another name.
//! It names only its own assets as files. The main socket refuses a
//! `hello` with a name that has a client socket.

use std::{
    collections::BTreeSet,
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

use super::{RpcError, Socket, private_dir, rejected, socket_rule};
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
    /// What a connection through the socket may do (D33, D40); the list
    /// of `clients.yaml`. Without it, a new socket gets none and an
    /// existing socket keeps its own.
    #[serde(default)]
    capabilities: Option<Vec<Capability>>,
}

impl State {
    /// `clients.create`: opens the client socket and the Wayland socket of
    /// a name with its capabilities, and returns their paths. The same
    /// capabilities keep the socket; other capabilities recreate it and
    /// end its Wayland clients (D33, D40).
    pub(super) fn ipc_client_socket(&mut self, create: &Create) -> Result<Value, RpcError> {
        let name = &create.name;
        let base = (self.ipc.path.as_ref())
            .and_then(|p| p.parent())
            .ok_or_else(|| rejected("the core has no socket directory"))?;
        let dir = base.join("clients").join(name);
        let wanted: Option<BTreeSet<Capability>> =
            (create.capabilities.as_ref()).map(|caps| caps.iter().copied().collect());
        let wanted = self.settle_socket(name, &dir, wanted)?;
        if dir.join(WAYLAND).as_os_str().len() > MAX_SOCKET_PATH {
            return Err(RpcError::new(
                RpcError::LIMIT_REACHED,
                format!("{name:?}: the socket path is too long"),
            ));
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
        let wayland_count = count.clone();
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
                wayland_count,
                dir: dir.clone(),
                clients: Vec::new(),
                capabilities: wanted.clone(),
            },
        );
        // A connection through the socket has exactly these (D33).
        self.grants.set_exact(name.clone(), wanted);
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
                let handle = self.display_handle.backend_handle();
                if let Some(socket) = self.ipc.clients.get_mut(name) {
                    // Clients that went leave the list here.
                    socket
                        .clients
                        .retain(|id| handle.get_client_data(id.clone()).is_ok());
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

    /// The capabilities for a new socket of a name. An existing socket
    /// with the same capabilities, or with none asked for, stays and
    /// answers `socket_exists`; other capabilities close it first (D40).
    fn settle_socket(
        &mut self,
        name: &str,
        dir: &Path,
        wanted: Option<BTreeSet<Capability>>,
    ) -> Result<BTreeSet<Capability>, RpcError> {
        if let Some(socket) = self.ipc.clients.get(name) {
            if wanted.as_ref().is_none_or(|w| *w == socket.capabilities) {
                // The same file gives back what the revoke binding took.
                if let Some(wanted) = wanted {
                    self.grants.set_exact(name.to_owned(), wanted);
                }
                return Err(socket_exists(name, dir));
            }
        } else {
            self.check_new_socket(name)?;
        }
        let wanted = wanted.unwrap_or_default();
        if self.ipc.clients.contains_key(name) {
            // The old connections keep running; they get the new
            // capabilities first, also when the new socket fails to open.
            self.grants.set_exact(name.to_owned(), wanted.clone());
            if !wanted.contains(&Capability::GrabAny) {
                self.revoke_free_grabs(name);
            }
            self.close_socket(name);
        }
        Ok(wanted)
    }

    /// Why `clients.create` of a name without a socket must fail, if it
    /// must.
    fn check_new_socket(&self, name: &str) -> Result<(), RpcError> {
        if !crate::methods::valid_name(name) {
            return Err(rejected(format!(
                "{name:?}: a client name for a socket has lowercase letters, digits, _, - and ., from a letter"
            )));
        }
        // A live connection of the name would share the nodes and grants.
        if self.ipc.live(name) {
            return Err(RpcError::new(
                RpcError::NAME_TAKEN,
                format!("{name:?} is connected or in its grace time"),
            )
            .with_data(serde_json::json!({"reason": "connected"})));
        }
        if self.ipc.clients.len() >= MAX_SOCKETS {
            return Err(RpcError::new(
                RpcError::LIMIT_REACHED,
                format!("the core holds {MAX_SOCKETS} client sockets"),
            ));
        }
        Ok(())
    }

    /// Removes all client sockets, for the emergency reset (D40). Their
    /// connections close with the others.
    pub fn remove_client_sockets(&mut self) {
        let names: Vec<String> = self.ipc.clients.keys().cloned().collect();
        for name in names {
            self.close_socket(&name);
        }
    }

    /// Ends the Wayland clients of a name, unregisters its listeners and
    /// removes its socket files; also the first half of a recreate.
    fn close_socket(&mut self, name: &str) {
        self.disconnect_wayland(name, &[]);
        if let Some(socket) = self.ipc.clients.remove(name) {
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
            Some(path) => Err(socket_rule(format!(
                "{path:?}: a client socket names only its own assets"
            ))),
            None => Ok(()),
        }
    }

    /// The scene rules of a connection of a client socket. The error names
    /// the rule and carries the index of the command, as the scene does.
    pub(super) fn confine(&mut self, conn: u64, commands: &[Command]) -> Result<(), RpcError> {
        let Some(name) = self.ipc.fixed(conn).map(str::to_owned) else {
            return Ok(());
        };
        for (index, command) in commands.iter().enumerate() {
            self.confine_one(&name, command)
                .map_err(|e| socket_rule(e).with_data(json!({"command": index})))?;
        }
        Ok(())
    }

    fn confine_one(&mut self, name: &str, command: &Command) -> Result<(), String> {
        match command {
            Command::Create { parent, .. }
            | Command::Move {
                parent: Some(parent),
                ..
            } if self.on_screen(parent) => Err(format!(
                "a client socket has no screen:* parents, and {parent:?} is on a screen"
            )),
            Command::Camera { .. } => Err("a client socket has no camera commands".into()),
            Command::FocusNode { node: Some(_) } if !self.pressed_by(name) => Err(
                "a client socket takes the keyboard with focus_node only right after a press on its own node or window".into(),
            ),
            Command::Attach { surface, .. }
            | Command::Focus {
                surface: Some(surface),
            }
            | Command::Close { surface }
            | Command::Configure { surface, .. }
                if !self.own_surface(name, surface) =>
            {
                Err(format!(
                    "a client socket acts only on its own surfaces, and {surface:?} is of another client"
                ))
            }
            Command::Focus { .. } if !self.pressed_by(name) => Err(
                "a client socket focuses its window only right after a press on its own node or window".into(),
            ),
            _ => Ok(()),
        }
    }

    /// Whether a surface came through the Wayland socket of a client name
    /// or belongs to it. A missing node is for the scene to refuse.
    fn own_surface(&self, name: &str, surface: &str) -> bool {
        self.scene.node(surface).is_none_or(|n| own_node(n, name))
    }

    /// Whether a client name holds the focus of a node already, or the
    /// last press of the user hit it and is not used yet. As with an
    /// activation token, the press counts once.
    fn pressed_by(&mut self, name: &str) -> bool {
        self.scene.focused_node_owner() == Some(name)
            || self.pressed.take_if(|p| p == name).is_some()
    }

    /// Whether a node is an overlay root of an output or below one.
    fn on_screen(&self, id: &str) -> bool {
        std::iter::successors(Some(id), |n| self.scene.node(n)?.parent.as_deref())
            .any(|n| n.starts_with(chora_scene::OVERLAY))
    }
}

/// Whether a client name owns a node, or the node is a window of its
/// Wayland socket.
pub(super) fn own_node(node: &chora_scene::Node, name: &str) -> bool {
    node.owner.as_deref() == Some(name)
        || node.surface.as_ref().and_then(|s| s.client.as_deref()) == Some(name)
}

/// The client name of the client socket that a window came through.
pub fn window_client(scene: &chora_scene::Scene, surface: &str) -> Option<String> {
    scene.node(surface)?.surface.as_ref()?.client.clone()
}

/// The file names of the two sockets in the directory of a client.
const IPC: &str = "ipc.sock";
const WAYLAND: &str = "wayland.sock";

/// The listeners of the IPC and the Wayland socket of a client name, and
/// their directory.
pub(super) struct ClientSocket {
    pub ipc: RegistrationToken,
    wayland: RegistrationToken,
    /// The Wayland clients connected right now; the accept loop owns the
    /// other half of the counter.
    #[allow(dead_code)] // the cap test reads it through the state
    pub wayland_count: Arc<std::sync::atomic::AtomicU32>,
    dir: PathBuf,
    /// The Wayland clients that connected through it.
    pub clients: Vec<ClientId>,
    /// What `clients.create` gave the socket (D33); `clients up`
    /// recreates it when these change.
    capabilities: BTreeSet<Capability>,
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

/// The answer of `clients.create` for a name whose socket already has the
/// capabilities; the `path` lets `choractl clients create` name the socket.
fn socket_exists(name: &str, dir: &Path) -> RpcError {
    RpcError::new(
        RpcError::NAME_TAKEN,
        format!("{name:?} has a client socket already"),
    )
    .with_data(serde_json::json!({"reason": "socket_exists", "path": dir.join(IPC)}))
}

#[cfg(test)]
mod tests;
