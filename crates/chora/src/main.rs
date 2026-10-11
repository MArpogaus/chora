//! The chora Wayland compositor.

mod backend;
mod cursor;
mod foreign_toplevel;
mod handlers;
mod input;
mod ipc;
mod layers;
mod lock;
mod output;
mod output_management;
mod protocols;
mod render;
mod safety;
mod state;
mod transaction;

use std::{
    ffi::OsStr,
    path::Path,
    process::{Child, Command},
};

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
    let socket = ipc::listen(&mut state)?;
    // The order matters. No thread exists yet, so the startup command gets
    // an empty signal mask; std does not reset the mask everywhere. Then
    // the signal source blocks SIGUSR1 in this thread, and all later
    // threads inherit the block, Mesa's included. A thread without it would
    // receive SIGUSR1, and its default action ends the process.
    let child = command
        .split_first()
        .and_then(|(program, args)| spawn(program, args, &state.socket_name, &socket));
    safety::reset_on_signal(&event_loop.handle())?;
    if let Some(mut child) = child {
        // Waits for the command, so that it never stays a zombie.
        std::thread::spawn(move || child.wait());
    }
    backend::init(&mut state)?;
    tracing::info!(wayland = ?state.socket_name, ipc = %socket.display(), "chora runs");
    event_loop.run(None, &mut state, |state| {
        state.popups.cleanup();
        // The focus follows the protocols, for example after a popup grab
        // ended.
        state.update_focus();
        state.ipc_flush();
        if let Err(err) = state.display_handle.flush_clients() {
            tracing::warn!(?err, "flushing the wayland clients failed");
        }
    })?;
    Ok(())
}

/// Variables of the host session that would let the startup command reach
/// the host compositor instead of this one.
const HOST_VARIABLES: [&str; 6] = [
    "WAYLAND_SOCKET",
    "DISPLAY",
    "SWAYSOCK",
    "I3SOCK",
    "NIRI_SOCKET",
    "HYPRLAND_INSTANCE_SIGNATURE",
];

/// Starts the startup command with the environment of this compositor
/// (D30).
fn spawn(program: &str, args: &[String], wayland: &OsStr, socket: &Path) -> Option<Child> {
    let mut command = Command::new(program);
    command
        .args(args)
        .env("WAYLAND_DISPLAY", wayland)
        .env(chora_scene::ipc::SOCKET_ENV, socket);
    for variable in HOST_VARIABLES {
        command.env_remove(variable);
    }
    command
        .spawn()
        .inspect_err(|err| tracing::error!(%err, program, "the startup command failed"))
        .ok()
}
