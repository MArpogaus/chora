//! The safety layer: the only built-in behavior (D23).
//!
//! Besides the bindings and the reset, a watchdog watches frames that take
//! longer to render than a refresh of their output. After two seconds of
//! slow frames it removes the effects; after two more, it starts the
//! emergency reset. `CHORA_FRAME_BUDGET_US` sets a fixed budget in
//! microseconds, for tests.

use calloop::{
    LoopHandle,
    signals::{Signal, Signals},
};
use std::time::{Duration, Instant};

use chora_scene::{Effect, Event};

use crate::state::State;

/// SIGUSR1 starts the emergency reset, for example from a text console when
/// the keyboard no longer reaches the compositor.
pub fn reset_on_signal(
    handle: &LoopHandle<'static, State>,
) -> Result<(), Box<dyn std::error::Error>> {
    let signals = Signals::new(&[Signal::SIGUSR1])?;
    handle
        .insert_source(signals, |_, (), state| state.reset())
        .map_err(|e| e.error)?;
    Ok(())
}

/// How long frames stay slow before the watchdog acts.
const SLOW_FOR: Duration = Duration::from_secs(2);
/// How long frames stay fast before removed effects may come back without
/// a reset.
const CALM_FOR: Duration = Duration::from_secs(10);

/// The watchdog of the render times.
#[derive(Default)]
pub struct Watchdog {
    /// The first frame of the current run of slow frames.
    slow_since: Option<Instant>,
    /// The last slow frame.
    last_slow: Option<Instant>,
    /// The effects are gone; slow frames now start the reset.
    dropped: bool,
    /// A fixed budget from `CHORA_FRAME_BUDGET_US`.
    budget: Option<Duration>,
}

impl Watchdog {
    /// A watchdog with the budget of the environment, if any.
    pub fn from_env() -> Self {
        let budget = std::env::var("CHORA_FRAME_BUDGET_US")
            .ok()
            .and_then(|v| v.parse().ok())
            .map(Duration::from_micros);
        Self {
            budget,
            ..Self::default()
        }
    }

    /// Counts a frame that took a time to render, against the refresh
    /// period of its output.
    fn note(&mut self, refresh: Duration, took: Duration, now: Instant) -> Option<Action> {
        if took <= self.budget.unwrap_or(refresh) {
            self.slow_since = None;
            if self.last_slow.is_some_and(|t| now - t >= CALM_FOR) {
                self.dropped = false;
            }
            return None;
        }
        self.last_slow = Some(now);
        let since = *self.slow_since.get_or_insert(now);
        if now - since < SLOW_FOR {
            return None;
        }
        self.slow_since = None;
        self.dropped = !self.dropped;
        Some(if self.dropped {
            Action::DropEffects
        } else {
            Action::Reset
        })
    }
}

/// What the watchdog asks for after a frame.
#[derive(Debug, PartialEq, Eq)]
enum Action {
    DropEffects,
    Reset,
}

impl State {
    /// Notes how long a frame of an output with a refresh period took to
    /// render, and acts on slow frames.
    pub fn frame_rendered(&mut self, refresh: Duration, took: Duration) {
        match self.watchdog.note(refresh, took, Instant::now()) {
            Some(Action::DropEffects) => self.drop_effects(),
            Some(Action::Reset) => {
                watchdog_resets();
                self.reset();
            }
            None => {}
        }
    }

    /// Removes the effects of all nodes and tells the plugins.
    fn drop_effects(&mut self) {
        tracing::warn!("frames are slow; the watchdog removes the effects");
        self.scene.drop_effects();
        self.emit(&Event::EffectsDropped);
    }

    /// The emergency reset: disconnects all clients of the IPC socket, puts
    /// all surfaces in a grid that fills the output and focuses the first
    /// one.
    pub fn reset(&mut self) {
        for conn in self.ipc.conn_ids() {
            self.ipc_close(conn);
        }
        self.node_grab = None;
        let effects = self.scene.reset(self.reset_area());
        self.send_configures(&effects);
        let first = self
            .scene
            .node(chora_scene::ROOT)
            .and_then(|r| r.children.first().cloned());
        self.focus(first.as_deref());
        tracing::info!("emergency reset");
    }

    fn send_configures(&mut self, effects: &[Effect]) {
        for effect in effects {
            if let Effect::Configure {
                surface,
                size,
                states,
            } = effect
            {
                self.configure(surface, *size, states);
            }
        }
    }
}

fn watchdog_resets() {
    tracing::warn!("frames stay slow without effects; the watchdog resets");
}

#[cfg(test)]
mod tests {
    use super::*;

    const REFRESH: Duration = Duration::from_millis(16);
    const SLOW: Duration = Duration::from_millis(20);

    /// Notes slow frames at 60 Hz for a time, and returns the actions.
    fn slow_for(watchdog: &mut Watchdog, start: Instant, ms: u64) -> Vec<Action> {
        (0..ms / 16)
            .filter_map(|i| watchdog.note(REFRESH, SLOW, start + Duration::from_millis(i * 16)))
            .collect()
    }

    #[test]
    fn slow_frames_drop_the_effects_and_then_reset() {
        let mut watchdog = Watchdog::default();
        let actions = slow_for(&mut watchdog, Instant::now(), 4100);
        assert_eq!(actions, vec![Action::DropEffects, Action::Reset]);
    }

    #[test]
    fn a_fast_frame_starts_the_time_again() {
        let mut watchdog = Watchdog::default();
        let start = Instant::now();
        assert!(slow_for(&mut watchdog, start, 1900).is_empty());
        let later = start + Duration::from_millis(1950);
        assert_eq!(
            watchdog.note(REFRESH, Duration::from_millis(5), later),
            None
        );
        assert!(slow_for(&mut watchdog, later, 1900).is_empty());
    }
}
