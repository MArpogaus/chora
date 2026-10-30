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
