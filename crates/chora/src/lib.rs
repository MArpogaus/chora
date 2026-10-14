//! The chora Wayland compositor as a library: the binary runs it, and the
//! unit tests reach its parts.

mod backend;
mod capture;
mod captured;
mod copies;
mod cursor;
mod foreign_toplevel;
mod gl;
mod grants;
mod handlers;
mod image_capture;
mod images;
mod inject;
mod input;
mod ipc;
mod layers;
mod lock;
mod mesh;
mod methods;
mod output;
mod output_management;
mod process;
mod protocols;
mod render;
mod safety;
mod screencopy;
mod security;
mod shm;
mod state;
mod transaction;
mod uploads;

use smithay::reexports::{calloop::EventLoop, wayland_server::Display};

use crate::state::State;

const USAGE: &str = "usage: chora [--version] [-- COMMAND [ARGS...]]";

/// Runs the compositor with the arguments of the command line, without the
/// program name.
///
/// # Errors
///
/// Wrong arguments, and a failed start of the event loop, the display, the
/// IPC socket or the backend.
pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
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
    state.listen()?;
    let socket = ipc::listen(&mut state)?;
    // The order matters. No thread exists yet, so the startup command gets
    // an empty signal mask; std does not reset the mask everywhere. Then
    // the signal source blocks SIGUSR1 in this thread, and all later
    // threads inherit the block, Mesa's included. A thread without it would
    // receive SIGUSR1, and its default action ends the process.
    let child = (!command.is_empty())
        .then(|| {
            process::spawn(
                command,
                (None, std::process::Stdio::inherit(), false),
                &state.socket_name,
                &socket,
            )
        })
        .and_then(|started| {
            started
                .inspect_err(|err| tracing::error!(%err, ?command, "the startup command failed"))
                .ok()
        });
    safety::reset_on_signal(&event_loop.handle())?;
    if let Some(child) = child {
        process::reap(child);
    }
    backend::init(&mut state)?;
    input::log_layout(&mut state);
    tracing::info!(wayland = ?state.socket_name, ipc = %socket.display(), "chora runs");
    event_loop.run(None, &mut state, State::after_events)?;
    Ok(())
}
