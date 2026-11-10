//! chora-chat: the opener of the chat, and with `--input` the text field
//! of its view, a layer surface that the host starts (docs/voice.md,
//! "The chat field"). The field sends text to a helper and answers no
//! question.

#![allow(
    clippy::print_stderr,
    reason = "the field is a program of the session: its log is stderr"
)]

mod draw;
mod events;
mod ipc;
mod keys;
mod opener;
mod paste;
mod session;
mod window;

use std::{process::ExitCode, sync::mpsc, thread};

use chora_kit::words::Language;
use clap::Parser;
use rustix::pipe::{PipeFlags, pipe_with};
use wayland_client::Connection;

use crate::{
    ipc::{Ipc, Wake},
    session::Session,
    window::Window,
};

/// The chat with a helper.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// Runs the text field of the chat view for the helper NAME, as a layer
    /// surface; the host starts it.
    #[arg(long, value_name = "NAME", conflicts_with = "name")]
    input: Option<String>,
    /// The helper to chat with; the person picks from the list without it.
    name: Option<String>,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let result = match &args.input {
        Some(helper) => field(helper),
        None => opener::open(args.name.as_deref()).map_err(|e| e.to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("chora-chat: {error}");
            ExitCode::FAILURE
        }
    }
}

/// The language of the words on the screen, from the locale variables.
fn language() -> Language {
    let tag = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .into_iter()
        .filter_map(|name| std::env::var(name).ok())
        .find(|value| !value.is_empty());
    tag.map_or(Language::En, |tag| Language::from_tag(&tag))
}

/// Runs the field until the host quits it.
fn field(helper: &str) -> Result<(), String> {
    let (msgs, inbox) = mpsc::channel();
    let (requests, outbox) = mpsc::channel();
    let (woken, wake) =
        pipe_with(PipeFlags::CLOEXEC | PipeFlags::NONBLOCK).map_err(|e| e.to_string())?;
    let ipc = Ipc::connect(helper, Wake::new(msgs, wake)).map_err(|e| e.to_string())?;
    thread::spawn(move || ipc.serve(&outbox));
    let conn = Connection::connect_to_env().map_err(|e| e.to_string())?;
    let mut queue = conn.new_event_queue();
    let qh = queue.handle();
    conn.display().get_registry(&qh, ());
    let mut window = Window::new(conn, qh.clone(), Session::new(language()), requests);
    queue.roundtrip(&mut window).map_err(|e| e.to_string())?;
    window.attach_seat(&qh);
    queue.roundtrip(&mut window).map_err(|e| e.to_string())?;
    queue.roundtrip(&mut window).map_err(|e| e.to_string())?;
    window.run(&mut queue, &inbox, woken)
}
