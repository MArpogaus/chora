//! The log of a plugin: `tracing` to stderr, where the core collects it.
//! `RUST_LOG` filters; the default is info. The target of a line is the
//! module path of the plugin crate (`chora_construct::cube`), so the
//! journal filters by plugin.

use std::{io::IsTerminal, sync::Once};

use tracing_subscriber::EnvFilter;

static START: Once = Once::new();

/// Starts the log of a plugin to stderr with `RUST_LOG` or "info" as the
/// filter. The first call starts it; later calls, and a log that already
/// runs, are no-ops: a subscriber that the host set stays.
pub fn init() {
    START.call_once(|| {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(std::io::stderr)
            .with_ansi(std::io::stderr().is_terminal())
            .with_target(true)
            .try_init();
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_second_init_does_not_panic() {
        super::init();
        super::init();
        tracing::info!("after the log started");
    }
}
