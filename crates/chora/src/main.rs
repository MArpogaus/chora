//! The chora Wayland compositor.

mod handlers;
mod input;
mod ipc;
mod render;
mod safety;
mod state;
mod transaction;
mod winit;

use std::{ffi::OsStr, path::Path, process::Command};

use smithay::reexports::{calloop::EventLoop, wayland_server::Display};

use crate::state::State;

const USAGE: &str = "usage: chora [--version] [-- COMMAND [ARGS...]]";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = match args.split_first() {
        None => &[][..],
        Some((first, _)) if first == "--version" => {
            println!("chora {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some((first, rest)) if first == "--" => rest,
        Some(_) => return Err(USAGE.into()),
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let mut event_loop: EventLoop<State> = EventLoop::try_new()?;
    let display: Display<State> = Display::new()?;
    let mut state = State::new(display, event_loop.handle(), event_loop.get_signal());
    winit::init(&mut state)?;
    let socket = ipc::listen(&mut state)?;
    tracing::info!(wayland = ?state.socket_name, ipc = %socket.display(), "chora runs");
    if let Some((program, args)) = command.split_first() {
        spawn(program, args, &state.socket_name, &socket);
    }
    // After the spawn: the signal source blocks SIGUSR1, and children
    // inherit the signal mask.
    safety::reset_on_signal(&state.loop_handle)?;
    event_loop.run(None, &mut state, |state| {
        state.ipc_flush();
        if let Err(err) = state.display_handle.flush_clients() {
            tracing::warn!(?err, "flushing the wayland clients failed");
        }
    })?;
    Ok(())
}

/// Starts the startup command with the environment of this compositor
/// (D30).
fn spawn(program: &str, args: &[String], wayland: &OsStr, socket: &Path) {
    let result = Command::new(program)
        .args(args)
        .env("WAYLAND_DISPLAY", wayland)
        .env(chora_scene::ipc::SOCKET_ENV, socket)
        .spawn();
    if let Err(err) = result {
        tracing::error!(%err, program, "the startup command failed");
    }
}
