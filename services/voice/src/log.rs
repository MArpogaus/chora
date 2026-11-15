//! Log lines with timestamps through `tracing`, to stderr (journald)
//! and to the file `voice.log` of `chora_scene::logs`. One call site per
//! level keeps the macros out of the callers: clippy counts their
//! expansion into the cognitive complexity of each caller.

use std::{io::IsTerminal, sync::Mutex};

use chora_scene::logs;
use tracing::{Level, Metadata, Subscriber};
use tracing_subscriber::{
    EnvFilter,
    filter::filter_fn,
    fmt::{self, MakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// The filter without `RUST_LOG`: tract logs its kernels at info, rmcp
/// each connection.
const DEFAULT: &str = "info,tract_linalg=warn,rmcp=warn";

/// Below info only the service logs: a dependency such as rmcp writes
/// the arguments of a tool call at debug, and they carry what the person
/// said.
fn ours_or_info(meta: &Metadata<'_>) -> bool {
    *meta.level() <= Level::INFO || meta.target().starts_with("chora_voice")
}

/// The log of `init`: `filter` and [`ours_or_info`] for every line, to
/// `out` and to `file`.
fn subscriber<W>(
    filter: EnvFilter,
    out: W,
    ansi: bool,
    file: Option<logs::File>,
) -> impl Subscriber + Send + Sync + 'static
where
    W: for<'w> MakeWriter<'w> + Send + Sync + 'static,
{
    tracing_subscriber::registry()
        .with(filter)
        .with(filter_fn(ours_or_info))
        .with(fmt::layer().with_writer(out).with_ansi(ansi))
        .with(file.map(|f| fmt::layer().with_ansi(false).with_writer(Mutex::new(f))))
}

/// Starts the log with `RUST_LOG` or the default filter; to the file
/// `voice.log` only for the agent (`agent`), so a CLI call leaves the
/// log of a running agent alone.
pub fn init(agent: bool) {
    let file = agent.then(|| logs::File::open("voice"));
    let failed = file
        .as_ref()
        .and_then(|f| f.as_ref().err())
        .map(ToString::to_string);
    subscriber(
        EnvFilter::try_from_default_env().unwrap_or_else(|_| DEFAULT.into()),
        std::io::stderr,
        std::io::stderr().is_terminal(),
        file.and_then(Result::ok),
    )
    .init();
    if let Some(err) = failed {
        warn(&format!("no log file: {err}"));
    }
    std::panic::set_hook(Box::new(|panic| error(&format!("panic: {panic}"))));
}

pub fn debug(line: &str) {
    tracing::debug!("{line}");
}

pub fn info(line: &str) {
    tracing::info!("{line}");
}

pub fn warn(line: &str) {
    tracing::warn!("{line}");
}

pub fn error(line: &str) {
    tracing::error!("{line}");
}

/// The log lines of every level, for the tests. One global
/// subscriber writes into the buffer of the thread that logs: a
/// subscriber per test races with the interest cache of `tracing`.
#[cfg(test)]
pub mod capture {
    use std::{cell::RefCell, io::Write, rc::Rc, sync::Once};

    use tracing_subscriber::EnvFilter;

    thread_local! {
        static LINES: RefCell<Option<Lines>> = const { RefCell::new(None) };
    }

    /// The text that the lines of this thread wrote.
    #[derive(Clone, Default)]
    pub struct Lines(Rc<RefCell<Vec<u8>>>);

    impl Lines {
        pub fn text(&self) -> String {
            String::from_utf8_lossy(&self.0.borrow()).into_owned()
        }
    }

    /// Ends the capture of this thread when dropped.
    pub struct Guard;

    impl Drop for Guard {
        fn drop(&mut self) {
            LINES.with(|lines| lines.borrow_mut().take());
        }
    }

    struct ThreadWriter;

    impl Write for ThreadWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            LINES.with(|lines| {
                if let Some(lines) = lines.borrow().as_ref() {
                    lines.0.borrow_mut().extend_from_slice(buf);
                }
            });
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// Captures the lines of every level of this thread while the guard
    /// lives, as `RUST_LOG=trace` writes them.
    pub fn all() -> (Guard, Lines) {
        static GLOBAL: Once = Once::new();
        GLOBAL.call_once(|| {
            let log = super::subscriber(EnvFilter::new("trace"), || ThreadWriter, false, None);
            let _ = tracing::subscriber::set_global_default(log);
        });
        let lines = Lines::default();
        LINES.with(|l| *l.borrow_mut() = Some(lines.clone()));
        (Guard, lines)
    }
}
