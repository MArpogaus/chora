//! The log of the core: `tracing` to stderr (journald or the terminal)
//! and to the file `chora.log` of `chora_scene::logs`. `RUST_LOG`
//! filters; the default is info.

use std::{io::IsTerminal, sync::Mutex};

use tracing::{Level, Metadata};
use tracing_subscriber::{
    EnvFilter, Layer, filter::filter_fn, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

/// The target of the last words of a panic. The panic hook writes them to
/// stderr itself, so only the file takes this target.
const PANIC: &str = "chora::panic";

/// Below info only the targets of chora log: smithay writes the keysym of
/// each key press at trace, which is typed text.
pub fn ours_or_info(meta: &Metadata<'_>) -> bool {
    *meta.level() <= Level::INFO || meta.target().starts_with("chora")
}

/// Starts the log with `RUST_LOG` or the default filter.
pub fn init() {
    let file = chora_scene::logs::File::open("chora");
    let failed = file.as_ref().err().map(ToString::to_string);
    let file = file
        .ok()
        .map(|f| fmt::layer().with_ansi(false).with_writer(Mutex::new(f)));
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(filter_fn(ours_or_info))
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(std::io::stderr().is_terminal())
                .with_filter(filter_fn(|meta| meta.target() != PANIC)),
        )
        .with(file)
        .init();
    if let Some(err) = failed {
        no_file(&err);
    }
}

fn no_file(err: &str) {
    tracing::warn!("no log file: {err}");
}

/// Writes the last words of a panic to the log file.
pub fn panic(words: &str) {
    tracing::error!(target: PANIC, "{words}");
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    struct Sink(Arc<Mutex<Vec<u8>>>);

    impl std::io::Write for Sink {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn key_press_of_smithay() {
        tracing::trace!(target: "smithay::input::keyboard", sym = "h", "Calling input filter");
    }

    fn trace_of_chora() {
        tracing::trace!(target: "chora::input", "a trace line of chora");
    }

    fn info_of_smithay() {
        tracing::info!(target: "smithay::backend", "an info line of smithay");
    }

    /// `RUST_LOG=trace` keeps the trace lines of chora and drops the
    /// keysyms that smithay logs for each key press.
    #[test]
    fn a_dependency_logs_no_key_press_at_trace() {
        let lines = Arc::default();
        let sink = Arc::clone(&lines);
        let subscriber = fmt()
            .with_ansi(false)
            .with_max_level(Level::TRACE)
            .with_writer(move || Sink(Arc::clone(&sink)))
            .finish()
            .with(filter_fn(ours_or_info));
        tracing::subscriber::with_default(subscriber, || {
            key_press_of_smithay();
            trace_of_chora();
            info_of_smithay();
        });
        let log = String::from_utf8_lossy(&lines.lock().unwrap()).into_owned();
        assert!(log.contains("a trace line of chora"), "{log}");
        assert!(log.contains("an info line of smithay"), "{log}");
        assert!(!log.contains("sym="), "{log}");
    }
}
