//! Log lines with timestamps through `tracing`, to stderr. One call site
//! per level keeps the macros out of the callers: clippy counts their
//! expansion into the cognitive complexity of each caller.

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

/// The log lines at info and above, for the tests.
#[cfg(test)]
pub mod capture {
    use std::{
        io::Write,
        sync::{Arc, Mutex, PoisonError},
    };

    use tracing::subscriber::DefaultGuard;

    /// The text that the subscriber of [`info`] wrote.
    #[derive(Clone, Default)]
    pub struct Lines(Arc<Mutex<Vec<u8>>>);

    impl Lines {
        pub fn text(&self) -> String {
            String::from_utf8_lossy(&self.0.lock().unwrap_or_else(PoisonError::into_inner))
                .into_owned()
        }
    }

    impl Write for Lines {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// Captures the lines at info and above of this thread while the
    /// guard lives, as the default level of the service writes them.
    pub fn info() -> (DefaultGuard, Lines) {
        let lines = Lines::default();
        let writer = lines.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_writer(move || writer.clone())
            .finish();
        (tracing::subscriber::set_default(subscriber), lines)
    }
}
