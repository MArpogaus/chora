//! The clock of the animations of a plugin: the monotonic clock, or the
//! stepped time of a recording.
//!
//! `choractl record` steps the scene time of the core by a fixed frame
//! time and calls the method `clock.step` of every plugin that exports
//! it, so that the frames of a recording do not depend on the speed of
//! the renderer. A plugin takes the time of its animations from [`now`],
//! exports [`method`] and hands the call to [`step`], then draws one
//! frame.

use std::{
    cell::Cell,
    sync::OnceLock,
    time::{Duration, Instant},
};

use chora_scene::ipc::{ExportMethod, RpcError};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::runtime;

/// The name of the method that steps the clock.
pub const METHOD: &str = "clock.step";

/// The longest recording: a day.
const MAX_MS: f64 = 86_400_000.0;

static START: OnceLock<Instant> = OnceLock::new();
thread_local! {
    /// The stepped time of the thread of the plugin's loop.
    static STEPPED: Cell<Option<Instant>> = const { Cell::new(None) };
}

/// The time of the animations: of the last step of a recording on this
/// thread, else now.
#[must_use]
pub fn now() -> Instant {
    STEPPED.get().unwrap_or_else(Instant::now)
}

/// The time that passed since `earlier` on the clock of the animations.
#[must_use]
pub fn since(earlier: Instant) -> Duration {
    now().saturating_duration_since(earlier)
}

/// Stops the clock at `ms` after the start of the plugin, or lets it run
/// again with `None`.
pub fn set(ms: Option<f64>) {
    let start = *START.get_or_init(Instant::now);
    let at = ms.map(|ms| start + Duration::from_secs_f64(ms.clamp(0.0, MAX_MS) / 1000.0));
    STEPPED.set(at);
}

/// The parameters of `clock.step`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    ms: Option<f64>,
}

/// The export of `clock.step`. It is privileged: it stops the animations
/// of the plugin.
#[must_use]
pub fn method() -> ExportMethod {
    runtime::privileged(runtime::method(
        METHOD,
        "Stops the clock of the animations at MS milliseconds of a recording, so \
         that the next frame shows that time; null lets the clock run again. \
         choractl record calls it for each frame.",
        json!({"ms": {"type": ["number", "null"]}}),
        &["ms"],
    ))
}

/// Takes a call of `clock.step`. The plugin then draws one frame of its
/// animations.
///
/// # Errors
///
/// The parameters are wrong.
pub fn step(params: Value) -> Result<Value, RpcError> {
    let Step { ms } = runtime::params(params)?;
    if ms.is_some_and(|ms| !ms.is_finite()) {
        return Err(RpcError::new(RpcError::INVALID_PARAMS, "ms is a number"));
    }
    set(ms);
    Ok(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stepped_clock_moves_only_with_its_steps() {
        step(json!({"ms": 1000.0})).unwrap();
        let first = now();
        std::thread::sleep(Duration::from_millis(5));
        assert_eq!(now(), first);
        step(json!({"ms": 1000.0 + 1000.0 / 30.0})).unwrap();
        let frame = now().duration_since(first).as_secs_f64();
        assert!((frame - 1.0 / 30.0).abs() < 1e-6, "{frame}");
        assert!(step(json!({"ms": "soon"})).is_err());
        step(json!({"ms": null})).unwrap();
        let running = now();
        std::thread::sleep(Duration::from_millis(5));
        assert!(now() > running);
    }
}
