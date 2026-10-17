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

use chora_scene::ipc::Capability;
use smithay::{
    reexports::{
        calloop::LoopHandle,
        wayland_protocols::{
            ext::idle_notify::v1::server::ext_idle_notifier_v1::ExtIdleNotifierV1,
            wp::idle_inhibit::zv1::server::zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1,
        },
        wayland_server::{DataInit, DisplayHandle, New, Resource, protocol::wl_surface::WlSurface},
    },
    wayland::{GlobalData, GlobalDispatch2, idle_notify::IdleNotifierState},
};

use crate::state::{ClientState, State};

/// Whether a client connected outside of a sandbox, and so sees the
/// privileged protocols.
pub fn trusted(client: &Client) -> bool {
    client
        .get_data::<ClientState>()
        .is_none_or(|c| c.security_context.is_none())
}

/// Whether a client sees a privileged protocol that a capability covers,
/// or that none covers with `None`. A client of a client socket sees it
/// only with the grant of its name at its connection (D33).
fn allowed(client: &Client, capability: Option<Capability>) -> bool {
    let Some(data) = client.get_data::<ClientState>() else {
        return true;
    };
    match (&data.security_context, &data.name) {
        (Some(_), _) => false,
        (None, Some(_)) => capability.is_some_and(|c| data.granted.contains(&c)),
        (None, None) => true,
    }
}

/// Offers idle inhibit, which Smithay offers to all clients, only outside
/// client sockets.
pub fn idle_inhibit_global(dh: &DisplayHandle) {
    dh.create_global::<State, ZwpIdleInhibitManagerV1, _>(1, OutsideClientSockets);
}

/// The state of idle notify, with a global that clients of client sockets
/// do not see in place of the global of Smithay.
pub fn idle_notifier(
    dh: &DisplayHandle,
    loop_handle: &LoopHandle<'static, State>,
) -> IdleNotifierState<State> {
    let state = IdleNotifierState::new(dh, loop_handle.clone());
    dh.remove_global::<State>(state.global());
    dh.create_global::<State, ExtIdleNotifierV1, _>(2, OutsideClientSockets);
    state
}

/// Whether a surface belongs to a client of a client socket. Such a
/// client gets no overlay layer and no exclusive keyboard, so it cannot
/// fake a lock screen or keep the keyboard.
pub fn named(dh: &DisplayHandle, surface: &WlSurface) -> bool {
    dh.get_client(surface.id())
        .ok()
        .and_then(|c| c.get_data::<ClientState>().map(|d| d.name.is_some()))
        .unwrap_or(false)
}

/// Whether a client came through the main socket or a sandbox, not a
/// client socket. A video player in a sandbox may inhibit idle.
fn outside_client_sockets(client: &Client) -> bool {
    client
        .get_data::<ClientState>()
        .is_none_or(|c| c.name.is_none())
}

/// The data of a global that only clients outside client sockets see. It binds the
/// global as Smithay does, for globals that Smithay offers to all.
pub struct OutsideClientSockets;

impl GlobalDispatch2<ZwpIdleInhibitManagerV1, State> for OutsideClientSockets {
    fn bind(
        &self,
        _: &mut State,
        _: &DisplayHandle,
        _: &Client,
        manager: New<ZwpIdleInhibitManagerV1>,
        data_init: &mut DataInit<'_, State>,
    ) {
        data_init.init(manager, GlobalData);
    }

    fn can_view(&self, client: &Client) -> bool {
        outside_client_sockets(client)
    }
}

impl GlobalDispatch2<ExtIdleNotifierV1, State> for OutsideClientSockets {
    fn bind(
        &self,
        _: &mut State,
        _: &DisplayHandle,
        _: &Client,
        notifier: New<ExtIdleNotifierV1>,
        data_init: &mut DataInit<'_, State>,
    ) {
        data_init.init(notifier, GlobalData);
    }

    fn can_view(&self, client: &Client) -> bool {
        outside_client_sockets(client)
    }
}

/// Whether a client sees screencopy and image capture.
pub fn may_capture(client: &Client) -> bool {
    allowed(client, Some(Capability::ScreenCapture))
}

/// Whether a client sees the input method.
pub fn may_inject(client: &Client) -> bool {
    allowed(client, Some(Capability::InputInject))
}

/// Whether a client sees the protocols that no grant covers: output
/// management, the session lock, both foreign toplevel protocols and the
/// security context manager. A client of a client socket never sees them.
pub fn unrestricted(client: &Client) -> bool {
    allowed(client, None)
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
