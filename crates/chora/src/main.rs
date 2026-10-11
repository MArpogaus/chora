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
    // Before any thread starts, Mesa's included: the signal source blocks
    // SIGUSR1 in this thread only, and new threads inherit the mask. A
    // thread without it would receive SIGUSR1 and end the process.
    // Children start with an empty mask, because std resets it.
    safety::reset_on_signal(&event_loop.handle())?;
    let display: Display<State> = Display::new()?;
    let mut state = State::new(display, event_loop.handle(), event_loop.get_signal());
    winit::init(&mut state)?;
    let socket = ipc::listen(&mut state)?;
    tracing::info!(wayland = ?state.socket_name, ipc = %socket.display(), "chora runs");
    if let Some((program, args)) = command.split_first() {
        spawn(program, args, &state.socket_name, &socket);
    }
    event_loop.run(None, &mut state, |state| {
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
/// (D30). A thread waits for it, so that it never stays a zombie.
fn spawn(program: &str, args: &[String], wayland: &OsStr, socket: &Path) {
    let mut command = Command::new(program);
    command
        .args(args)
        .env("WAYLAND_DISPLAY", wayland)
        .env(chora_scene::ipc::SOCKET_ENV, socket);
    for variable in HOST_VARIABLES {
        command.env_remove(variable);
    }
    match command.spawn() {
        Ok(mut child) => {
            std::thread::spawn(move || child.wait());
        }
        Err(err) => tracing::error!(%err, program, "the startup command failed"),
    }
}
