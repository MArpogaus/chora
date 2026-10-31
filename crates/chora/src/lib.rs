//! The chora Wayland compositor as a library: the binary runs it, and the
//! unit tests reach its parts.

mod backend;
mod capture;
mod copies;
mod cursor;
mod devices;
mod foreign_toplevel;
mod gl;
mod grants;
mod handlers;
mod hang;
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
#[cfg(test)]
mod testing;
mod text;
mod time;
mod transaction;
mod uploads;

use smithay::reexports::{calloop::EventLoop, wayland_server::Display};

use crate::state::State;

const USAGE: &str =
    "usage: chora [--version] [--headless WxH[@SCALE][+X+Y][,...]] [-- COMMAND [ARGS...]]";

/// The virtual outputs of `--headless`, and the arguments after them.
fn headless_outputs(
    args: &[String],
) -> Result<(Option<Vec<backend::headless::Spec>>, &[String]), String> {
    match args {
        [flag, specs, rest @ ..] if flag == "--headless" => {
            Ok((Some(backend::headless::parse_specs(specs)?), rest))
        }
        [flag] if flag == "--headless" => Err(USAGE.into()),
        _ => Ok((None, args)),
    }
}

/// The core's last words: a faulting signal writes the fault address and
/// a backtrace to stderr before the default action ends the process. A
/// crash without a panic then leaves something in the log. Best effort:
/// `Backtrace::force_capture` is not async signal safe, so a second fault
/// inside it stays unlogged.
#[allow(unsafe_code)] // a signal handler: libc calls, nothing else
fn install_last_words() {
    extern "C" fn words(sig: libc::c_int, info: *mut libc::siginfo_t, _ctx: *mut libc::c_void) {
        let name = match sig {
            libc::SIGSEGV => "SIGSEGV",
            libc::SIGBUS => "SIGBUS",
            libc::SIGILL => "SIGILL",
            libc::SIGABRT => "SIGABRT",
            _ => "signal",
        };
        // SAFETY: the kernel passes a valid siginfo for a faulting signal.
        let address = if info.is_null() {
            0
        } else {
            unsafe { (*info).si_addr() as usize }
        };
        let text = format!(
            "chora: {name} at address {address:#x}, backtrace follows\n{}\n",
            std::backtrace::Backtrace::force_capture()
        );
        // SAFETY: write to a fixed fd, then the default action of the
        // signal, as the handler contract allows.
        unsafe {
            libc::write(2, text.as_ptr().cast(), text.len());
            libc::signal(sig, libc::SIG_DFL);
            libc::raise(sig);
        }
    }
    for sig in [libc::SIGSEGV, libc::SIGBUS, libc::SIGILL, libc::SIGABRT] {
        // SAFETY: an all-zero sigaction with a plain handler is valid.
        unsafe {
            let mut act: libc::sigaction = std::mem::zeroed();
            act.sa_flags = libc::SA_SIGINFO;
            let handler: extern "C" fn(libc::c_int, *mut libc::siginfo_t, *mut libc::c_void) =
                words;
            act.sa_sigaction = handler as usize;
            libc::sigaction(sig, &raw const act, std::ptr::null_mut());
        }
    }
}

/// Runs the compositor with the arguments of the command line, without the
/// program name.
///
/// # Errors
///
/// Wrong arguments, and a failed start of the event loop, the display, the
/// IPC socket or the backend.
pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    install_last_words();
    let (headless, args) = headless_outputs(args)?;
    let command = match args.split_first() {
        None => &[][..],
        #[allow(clippy::print_stdout, reason = "the answer to --version")]
        Some((first, _)) if first == "--version" => {
            println!("chora {}", chora_scene::ipc::VERSION);
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
    // The order matters. Only the images worker exists, and it blocks
    // SIGUSR1 itself, so the startup command gets an empty signal mask;
    // std does not reset the mask everywhere. Then the signal source
    // blocks SIGUSR1 in this thread, and all later threads inherit the
    // block, Mesa's included. A thread without it would receive SIGUSR1,
    // and its default action ends the process.
    let child = (!command.is_empty())
        .then(|| process::start(command, &state.socket_name, &socket))
        .and_then(|started| {
            started
                .inspect_err(|err| tracing::error!(%err, ?command, "the startup command failed"))
                .ok()
        });
    safety::reset_on_signal(&event_loop.handle())?;
    let failed = std::rc::Rc::new(std::cell::Cell::new(false));
    if let Some(child) = child {
        let (ended, statuses) = calloop::channel::channel();
        process::reap(child, command, ended);
        let quit = failed.clone();
        event_loop
            .handle()
            .insert_source(statuses, move |event, (), state| {
                if let calloop::channel::Event::Msg(status) = event
                    && process::quits_after(state, status)
                {
                    tracing::error!(
                        "the startup command failed and no client said hello; chora quits"
                    );
                    quit.set(true);
                    state.loop_signal.stop();
                }
            })
            .map_err(|e| e.error)?;
    }
    backend::init(&mut state, headless.as_deref())?;
    input::log_layout(&mut state);
    tracing::info!(wayland = ?state.socket_name, ipc = %socket.display(), "chora runs");
    hang::run(&mut event_loop, &mut state, State::after_events)?;
    if failed.get() {
        return Err("the startup command failed".into());
    }
    Ok(())
}
