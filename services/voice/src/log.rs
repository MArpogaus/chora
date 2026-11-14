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

/// The log lines at info and above, for the tests. One global
/// subscriber writes into the buffer of the thread that logs: a
/// subscriber per test races with the interest cache of `tracing`.
#[cfg(test)]
pub mod capture {
    use std::{cell::RefCell, io::Write, rc::Rc, sync::Once};

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

    /// Captures the lines at info and above of this thread while the
    /// guard lives, as the default level of the service writes them.
    pub fn info() -> (Guard, Lines) {
        static GLOBAL: Once = Once::new();
        GLOBAL.call_once(|| {
            let subscriber = tracing_subscriber::fmt()
                .with_max_level(tracing::Level::INFO)
                .with_ansi(false)
                .with_writer(|| ThreadWriter)
                .finish();
            let _ = tracing::subscriber::set_global_default(subscriber);
        });
        let lines = Lines::default();
        LINES.with(|l| *l.borrow_mut() = Some(lines.clone()));
        (Guard, lines)
    }
}
