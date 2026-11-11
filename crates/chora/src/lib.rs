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
mod offscreen;
mod output;
mod output_management;
mod process;
mod protocols;
mod render;
mod safety;
mod screencopy;
mod security;
mod shm;
mod space;
mod state;
#[cfg(test)]
mod testing;
mod text;
mod time;
mod transaction;
mod uploads;
mod words;

use smithay::reexports::{calloop::EventLoop, wayland_server::Display};

use crate::state::State;

const USAGE: &str = "usage: chora [--version] \
     [--headless WxH[@SCALE][+X+Y][,...] [--test-injected-drag-gives]] [-- COMMAND [ARGS...]]";

/// The flag of the headless backend, for tests: an injected button press
/// counts as a press of the user for `give_grab`.
const TEST_INJECTED_DRAG_GIVES: &str = "--test-injected-drag-gives";

/// The virtual outputs of `--headless` with its test flag, and the
/// arguments after them. The test flag is no flag without `--headless`.
#[allow(clippy::type_complexity, reason = "the parts of the command line")]
fn headless_outputs(
    args: &[String],
) -> Result<(Option<(Vec<backend::headless::Spec>, bool)>, &[String]), String> {
    match args {
        [flag, specs, rest @ ..] if flag == "--headless" => {
            let specs = backend::headless::parse_specs(specs)?;
            match rest {
                [test, rest @ ..] if test == TEST_INJECTED_DRAG_GIVES => {
                    Ok((Some((specs, true)), rest))
                }
                _ => Ok((Some((specs, false)), rest)),
            }
        }
        [flag] if flag == "--headless" => Err(USAGE.into()),
        _ => Ok((None, args)),
    }
}

/// Turns on the test mode of `--test-injected-drag-gives` when the
/// command line asks for it, with a warning in the log, and gives the
/// outputs back.
fn test_mode<T>(state: &mut State, specs: T, test: bool) -> T {
    if test {
        tracing::warn!(
            "{TEST_INJECTED_DRAG_GIVES}: an injected press gives its drag; for tests only"
        );
        state.injected_drag_gives = true;
    }
    specs
}

/// Runs the compositor with the arguments of the command line, without the
/// program name.
///
/// # Errors
///
/// Wrong arguments, and a failed start of the event loop, the display, the
/// IPC socket or the backend.
pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    words::install();
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
                    quit.set(true);
                    state.quit("the startup command failed and no client said hello");
                }
            })
            .map_err(|e| e.error)?;
    }
    let headless = headless.map(|(specs, test)| test_mode(&mut state, specs, test));
    backend::init(&mut state, headless.as_deref(), command)?;
    input::log_layout(&mut state);
    tracing::info!(wayland = ?state.socket_name, ipc = %socket.display(), "chora runs");
    hang::run(&mut event_loop, &mut state, State::after_events)?;
    if failed.get() {
        return Err("the startup command failed".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{TEST_INJECTED_DRAG_GIVES, headless_outputs};

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|word| (*word).to_owned()).collect()
    }

    /// The test mode is a flag of `--headless` and of nothing else: a
    /// flag is not inherited by the programs that the core starts, and
    /// without `--headless` it is a wrong argument.
    #[test]
    fn the_test_flag_belongs_to_the_headless_backend() {
        let plain = args(&["--headless", "640x480", "--", "true"]);
        let (headless, rest) = headless_outputs(&plain).unwrap();
        assert!(headless.is_some_and(|(_, test)| !test));
        assert_eq!(rest, &plain[2..]);
        let test = args(&[
            "--headless",
            "640x480",
            TEST_INJECTED_DRAG_GIVES,
            "--",
            "true",
        ]);
        let (headless, rest) = headless_outputs(&test).unwrap();
        assert!(headless.is_some_and(|(_, test)| test));
        assert_eq!(rest, &test[3..]);
        let alone = args(&[TEST_INJECTED_DRAG_GIVES]);
        let (headless, rest) = headless_outputs(&alone).unwrap();
        assert!(headless.is_none());
        assert_eq!(rest, &alone[..], "left for the usage error");
    }
}
