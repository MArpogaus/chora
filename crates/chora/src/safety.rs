//! The safety layer: the only built-in behavior (D23).
//!
//! Besides the bindings and the reset, a watchdog watches the frames of
//! each output. After two seconds of slow frames it removes the effects;
//! when there were some and frames stay slow for two more seconds, it
//! starts the emergency reset, at most once a minute. `CHORA_FRAME_BUDGET_US` sets a fixed budget in
//! microseconds, for tests.

use calloop::{
    LoopHandle,
    signals::{Signal, Signals},
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

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
/// The shortest time between two resets of the watchdog.
const RESET_PAUSE: Duration = Duration::from_secs(60);

/// The watchdog of the render times.
#[derive(Default)]
pub struct Watchdog {
    /// The slow frames of each output.
    outputs: HashMap<String, Run>,
    /// The watchdog removed effects; slow frames now start the reset.
    dropped: bool,
    last_reset: Option<Instant>,
    /// A fixed budget from `CHORA_FRAME_BUDGET_US`.
    budget: Option<Duration>,
}

/// The slow frames of one output.
#[derive(Default)]
struct Run {
    /// The first frame of the current run of slow frames.
    since: Option<Instant>,
    /// The last slow frame.
    last: Option<Instant>,
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

    /// Counts a frame of an output that took a time, against a budget.
    fn note(
        &mut self,
        output: &str,
        budget: Duration,
        took: Duration,
        now: Instant,
    ) -> Option<Action> {
        let run = self.outputs.entry(output.into()).or_default();
        if took <= self.budget.unwrap_or(budget) {
            run.since = None;
            if run.last.is_some_and(|t| now - t >= CALM_FOR) {
                self.dropped = false;
            }
            return None;
        }
        run.last = Some(now);
        let since = *run.since.get_or_insert(now);
        if now - since < SLOW_FOR {
            return None;
        }
        run.since = None;
        if !self.dropped {
            return Some(Action::DropEffects);
        }
        // Without effects, the plugins may still overload the scene; a
        // reset helps once, but not again and again, for example on a slow
        // GPU.
        if self.last_reset.is_some_and(|t| now - t < RESET_PAUSE) {
            return None;
        }
        self.last_reset = Some(now);
        self.dropped = false;
        Some(Action::Reset)
    }
}

/// What the watchdog asks for after a frame.
#[derive(Debug, PartialEq, Eq)]
enum Action {
    DropEffects,
    Reset,
}

impl State {
    /// Notes how long a frame of an output took against a budget, and acts
    /// on slow frames: the nested backend measures the render, the tty
    /// backend the time from the render to the vblank.
    pub fn frame_rendered(&mut self, output: &str, budget: Duration, took: Duration) {
        self.note_left_out(output);
        match self.watchdog.note(output, budget, took, Instant::now()) {
            Some(Action::DropEffects) => self.drop_effects(),
            Some(Action::Reset) => {
                watchdog_resets();
                self.reset();
            }
            None => {}
        }
    }

    /// Tells the plugins when the layer budget of a frame left other nodes
    /// out than the frame before.
    fn note_left_out(&mut self, output: &str) {
        let nodes = self.assets.layers.left_out.clone();
        let before = self
            .assets
            .left_out
            .get(output)
            .map_or(&[][..], Vec::as_slice);
        if before == nodes.as_slice() {
            return;
        }
        if nodes.is_empty() {
            self.assets.left_out.remove(output);
        } else {
            self.assets
                .left_out
                .insert(output.to_owned(), nodes.clone());
        }
        self.emit(&Event::LayersDropped {
            output: output.to_owned(),
            nodes,
        });
    }

    /// Removes the effects of all nodes and tells the plugins. Without
    /// effects, the slow frames have another cause, and no reset follows.
    fn drop_effects(&mut self) {
        if !self.scene.drop_effects() {
            return;
        }
        tracing::warn!("frames are slow; the watchdog removes the effects");
        self.watchdog.dropped = true;
        self.emit(&Event::EffectsDropped);
    }

    /// The emergency reset: disconnects all clients of the IPC socket,
    /// revokes all grants, puts all surfaces in a grid that fills the output
    /// and focuses the first one.
    pub fn reset(&mut self) {
        for conn in self.ipc.conn_ids() {
            self.ipc_close(conn);
        }
        self.grants.clear();
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

    /// Notes slow frames of an output at 60 Hz for a time, from a start,
    /// and returns the actions; a drop of effects removes some if `found`.
    fn slow_for(watchdog: &mut Watchdog, start: Instant, ms: u64, found: bool) -> Vec<Action> {
        let mut actions = Vec::new();
        for i in 0..ms / 16 {
            let now = start + Duration::from_millis(i * 16);
            if let Some(action) = watchdog.note("out", REFRESH, SLOW, now) {
                if action == Action::DropEffects {
                    watchdog.dropped = found;
                }
                actions.push(action);
            }
        }
        actions
    }

    #[test]
    fn slow_frames_drop_the_effects_and_then_reset_once() {
        let mut watchdog = Watchdog::default();
        let start = Instant::now();
        let actions = slow_for(&mut watchdog, start, 4100, true);
        assert_eq!(actions, vec![Action::DropEffects, Action::Reset]);
        // Slow again soon after: a drop, but no second reset.
        let later = start + Duration::from_millis(4200);
        let actions = slow_for(&mut watchdog, later, 4100, true);
        assert_eq!(actions, vec![Action::DropEffects]);
    }

    #[test]
    fn without_effects_slow_frames_never_reset() {
        let mut watchdog = Watchdog::default();
        let actions = slow_for(&mut watchdog, Instant::now(), 20_000, false);
        assert!(actions.iter().all(|a| *a == Action::DropEffects));
    }

    #[test]
    fn a_fast_frame_starts_the_time_again() {
        let mut watchdog = Watchdog::default();
        let start = Instant::now();
        assert_eq!(slow_for(&mut watchdog, start, 1900, true), Vec::new());
        let later = start + Duration::from_millis(1950);
        assert_eq!(
            watchdog.note("out", REFRESH, Duration::from_millis(5), later),
            None
        );
        assert_eq!(slow_for(&mut watchdog, later, 1900, true), Vec::new());
    }
}
