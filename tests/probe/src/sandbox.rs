//! The mode `sandbox`: creates a security context, connects through its
//! socket as an app of a sandbox does, and checks that the sandboxed
//! client sees the ordinary globals, but none of the privileged ones.

use std::{
    os::{
        fd::AsFd,
        unix::net::{UnixListener, UnixStream},
    },
    path::PathBuf,
};

use wayland_client::{Connection, Dispatch, QueueHandle, delegate_noop, protocol::wl_registry};
use wayland_protocols::wp::security_context::v1::client::{
    wp_security_context_manager_v1::WpSecurityContextManagerV1,
    wp_security_context_v1::WpSecurityContextV1,
};

use crate::{Probe, Queue};

/// The globals that only clients outside of a sandbox see.
pub const PRIVILEGED: [&str; 11] = [
    "zwlr_layer_shell_v1",
    "ext_session_lock_manager_v1",
    "zwp_input_method_manager_v2",
    "ext_foreign_toplevel_list_v1",
    "zwlr_foreign_toplevel_manager_v1",
    "zwlr_output_manager_v1",
    "zwlr_screencopy_manager_v1",
    "ext_image_copy_capture_manager_v1",
    "ext_output_image_capture_source_manager_v1",
    "ext_foreign_toplevel_image_capture_source_manager_v1",
    "wp_security_context_manager_v1",
];

/// Globals that every client sees.
const ORDINARY: [&str; 4] = ["wl_compositor", "wl_shm", "wl_seat", "xdg_wm_base"];

/// The global names that a connection sees.
#[derive(Default)]
struct Globals(Vec<String>);

/// Connects through a new security context and lists its globals.
pub fn sandbox(queue: &mut Queue, probe: &mut Probe) -> Result<String, String> {
    let missing: Vec<_> = PRIVILEGED
        .iter()
        .filter(|g| !probe.globals.iter().any(|n| n == *g))
        .collect();
    if !missing.is_empty() {
        return Err(format!("the trusted connection misses {missing:?}"));
    }
    let manager = probe
        .security
        .clone()
        .ok_or("no wp_security_context_manager_v1")?;
    let runtime = std::env::var_os("XDG_RUNTIME_DIR").ok_or("no XDG_RUNTIME_DIR")?;
    let path = PathBuf::from(runtime).join(format!("chora-probe-{}.sock", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).map_err(|e| e.to_string())?;
    // The context ends when the write end closes, at the end of this mode.
    let (close_read, _close_write) = rustix::pipe::pipe().map_err(|e| e.to_string())?;
    let qh = queue.handle();
    let context = manager.create_listener(listener.as_fd(), close_read.as_fd(), &qh, ());
    context.set_sandbox_engine("org.chora.probe".into());
    context.set_app_id("org.chora.Probe".into());
    context.set_instance_id("1".into());
    context.commit();
    queue.roundtrip(probe).map_err(|e| e.to_string())?;
    let seen = globals_through(&path);
    let _ = std::fs::remove_file(&path);
    let seen = seen?;
    let leaked: Vec<_> = PRIVILEGED
        .iter()
        .filter(|g| seen.contains(&(**g).to_string()))
        .collect();
    if !leaked.is_empty() {
        return Err(format!("the sandbox sees {leaked:?}"));
    }
    let lost: Vec<_> = ORDINARY
        .iter()
        .filter(|g| !seen.contains(&(**g).to_string()))
        .collect();
    if !lost.is_empty() {
        return Err(format!("the sandbox misses {lost:?}"));
    }
    Ok(format!(
        "the sandbox sees {} globals, none of them privileged",
        seen.len()
    ))
}

/// The globals of a connection through a socket.
fn globals_through(path: &PathBuf) -> Result<Vec<String>, String> {
    let stream = UnixStream::connect(path).map_err(|e| e.to_string())?;
    let conn = Connection::from_socket(stream).map_err(|e| e.to_string())?;
    let mut queue = conn.new_event_queue();
    conn.display().get_registry(&queue.handle(), ());
    let mut globals = Globals::default();
    queue.roundtrip(&mut globals).map_err(|e| e.to_string())?;
    Ok(globals.0)
}

impl Dispatch<wl_registry::WlRegistry, ()> for Globals {
    fn event(
        globals: &mut Self,
        _: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { interface, .. } = event {
            globals.0.push(interface);
        }
    }
}

delegate_noop!(Probe: ignore WpSecurityContextManagerV1);
delegate_noop!(Probe: ignore WpSecurityContextV1);
