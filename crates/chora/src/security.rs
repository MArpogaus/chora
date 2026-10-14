//! `wp-security-context-v1`: a sandbox such as Flatpak connects its apps
//! through a socket of its own, and the core hides the privileged protocols
//! from those clients (D33).
//!
//! Hidden are the layer shell, the session lock, the input method, both
//! foreign toplevel protocols, output management, both screencopy
//! protocols with their sources, and the security context manager itself,
//! so that a sandboxed client cannot nest a context of its own.

use std::{os::unix::net::UnixStream, sync::Arc};

use smithay::{
    reexports::wayland_server::Client,
    wayland::security_context::{
        SecurityContext, SecurityContextHandler, SecurityContextListenerSource,
    },
};

use crate::state::{ClientState, State};

/// Whether a client connected outside of a sandbox, and so sees the
/// privileged protocols.
pub fn trusted(client: &Client) -> bool {
    client
        .get_data::<ClientState>()
        .is_none_or(|c| c.security_context.is_none())
}

impl SecurityContextHandler for State {
    fn context_created(&mut self, source: SecurityContextListenerSource, context: SecurityContext) {
        let inserted = self
            .loop_handle
            .insert_source(source, move |stream, (), state| {
                connect(state, stream, &context);
            });
        if let Err(err) = inserted {
            tracing::warn!(?err, "no socket for a sandbox");
        }
    }
}

/// Inserts a client that connected through the socket of a sandbox.
fn connect(state: &mut State, stream: UnixStream, context: &SecurityContext) {
    let data = ClientState {
        security_context: Some(context.clone()),
        ..ClientState::default()
    };
    if let Err(err) = state.display_handle.insert_client(stream, Arc::new(data)) {
        tracing::warn!(?err, "a sandboxed client failed to connect");
    }
}
