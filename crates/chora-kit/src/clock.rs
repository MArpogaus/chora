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

thread_local! {
    /// The stepped time of the thread of the plugin's loop.
    static STEPPED: Cell<Option<Instant>> = const { Cell::new(None) };
    /// Where the recording started: the time of the plugin then, and the
    /// time of the recording.
    static BASE: Cell<Option<(Instant, f64)>> = const { Cell::new(None) };
}

/// The time of the animations: of the last step of a recording on this
/// thread, else now.
#[must_use]
pub fn now() -> Instant {
    at(Instant::now())
}

/// The time of the animations at a time of the monotonic clock.
fn at(real: Instant) -> Instant {
    STEPPED.get().unwrap_or(real)
}

/// The time that passed since `earlier` on the clock of the animations.
#[must_use]
pub fn since(earlier: Instant) -> Duration {
    now().saturating_duration_since(earlier)
}

/// Stops the clock at `ms` of a recording, or lets it run again with
/// `None`. The first step of a recording stops the clock where it is, and
/// the later steps move it on from there, so the time never goes back.
pub fn set(ms: Option<f64>) {
    let Some(ms) = ms else {
        STEPPED.set(None);
        BASE.set(None);
        return;
    };
    let ms = ms.clamp(0.0, MAX_MS);
    let (start, first) = BASE.get().unwrap_or((Instant::now(), ms));
    BASE.set(Some((start, first)));
    let passed = Duration::from_secs_f64((ms - first).max(0.0) / 1000.0);
    STEPPED.set(Some(start + passed));
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
        let before = Instant::now();
        step(json!({"ms": 1000.0})).unwrap();
        let first = now();
        assert!(
            first >= before,
            "the first step keeps the time of the plugin"
        );
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

    #[test]
    fn a_stepped_clock_runs_again_when_no_step_comes() {
        step(json!({"ms": 0.0})).unwrap();
        let stopped = now();
        let soon = Instant::now() + Duration::from_millis(500);
        assert_eq!(at(soon), stopped, "between the steps of a slow frame");
        let late = Instant::now() + Duration::from_secs(3);
        assert_eq!(at(late), late, "the recording died: the clock runs");
        assert_eq!(
            at(late + Duration::from_secs(1)),
            late + Duration::from_secs(1)
        );
        set(None);
    }

    #[test]
    fn a_new_recording_starts_from_the_time_of_the_plugin() {
        step(json!({"ms": 5000.0})).unwrap();
        step(json!({"ms": 5033.0})).unwrap();
        std::thread::sleep(Duration::from_millis(5));
        let before = Instant::now();
        step(json!({"ms": 0.0})).unwrap();
        assert!(now() >= before, "a time that goes down starts anew");
        set(None);
    }
}
