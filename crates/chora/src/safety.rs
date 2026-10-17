//! The safety layer: the only built-in behavior (D23).
//!
//! Besides the bindings and the reset, a watchdog watches the frames of
//! each output. After two seconds of slow frames it blames the client whose
//! visible nodes cost the most and removes its effects; when frames stay
//! slow for two more seconds, it disconnects that client, at most once a
//! minute. The emergency reset is for the binding and SIGUSR1 only.
//! `CHORA_FRAME_BUDGET_US` sets a fixed budget in microseconds, for tests.

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
/// The shortest time between two disconnects of the watchdog.
const DISCONNECT_PAUSE: Duration = Duration::from_secs(60);
/// The least load of a client that the watchdog disconnects, as
/// `Scene::load_by_owner` counts it: on a slow GPU, a window manager with
/// a few nodes must not go.
const MIN_LOAD: u64 = 500;

/// The watchdog of the render times.
#[derive(Default)]
pub struct Watchdog {
    /// The slow frames of each output.
    outputs: HashMap<String, Run>,
    /// The client that the watchdog blamed; if frames stay slow, it goes.
    suspect: Option<String>,
    last_disconnect: Option<Instant>,
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
                self.suspect = None;
            }
            return None;
        }
        run.last = Some(now);
        let since = *run.since.get_or_insert(now);
        if now - since < SLOW_FOR {
            return None;
        }
        run.since = None;
        let Some(client) = self.suspect.take() else {
            return Some(Action::Blame);
        };
        // A disconnect helps once, but not again and again, for example on
        // a slow GPU.
        if self
            .last_disconnect
            .is_some_and(|t| now - t < DISCONNECT_PAUSE)
        {
            self.suspect = Some(client);
            return None;
        }
        self.last_disconnect = Some(now);
        Some(Action::Disconnect(client))
    }
}

/// What the watchdog asks for after a frame.
#[derive(Debug, PartialEq, Eq)]
enum Action {
    /// Remove the effects of the client whose nodes cost the most.
    Blame,
    Disconnect(String),
}

impl State {
    /// Notes how long a frame of an output took against a budget, and acts
    /// on slow frames: the nested backend measures the render, the tty
    /// backend the time from the render to the vblank.
    pub fn frame_rendered(&mut self, output: &str, budget: Duration, took: Duration) {
        self.note_left_out(output);
        match self.watchdog.note(output, budget, took, self.time.now()) {
            Some(Action::Blame) => self.blame(),
            Some(Action::Disconnect(client)) => self.disconnect_client(&client),
            None => {}
        }
    }

    /// Tells the plugins when the layer budget of a frame left other nodes
    /// out than the frame before.
    fn note_left_out(&mut self, output: &str) {
        let nodes = self
            .assets
            .frame_left_out
            .get(output)
            .cloned()
            .unwrap_or_default();
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

    /// Blames the client whose visible nodes cost the most: removes its
    /// effects and tells the plugins. If it has enough load, it goes when
    /// frames stay slow.
    fn blame(&mut self) {
        let load = self.scene.load_by_owner();
        let Some((client, load)) = load.into_iter().max_by_key(|(_, load)| *load) else {
            return;
        };
        tracing::warn!(%client, load, "frames are slow; the watchdog blames a client");
        if self.scene.drop_effects(&client) {
            self.emit(&Event::EffectsDropped {
                client: client.clone(),
            });
        }
        if load >= MIN_LOAD {
            self.watchdog.suspect = Some(client);
        }
    }

    /// Disconnects all connections of a client name and removes its nodes,
    /// also those of a client with `keep_nodes` or `resume`: they are the
    /// load.
    fn disconnect_client(&mut self, client: &str) {
        tracing::warn!(%client, "frames stay slow; the watchdog disconnects a client");
        let conns: Vec<u64> = self
            .ipc
            .conn_ids()
            .into_iter()
            .filter(|c| self.ipc.name(*c) == Some(client))
            .collect();
        for conn in conns {
            self.ipc_close(conn);
        }
        self.drop_nodes(client);
    }

    /// The emergency reset: disconnects all clients of the IPC socket,
    /// revokes all grants, puts all surfaces in a grid that fills the output
    /// and focuses the first one.
    pub fn reset(&mut self) {
        // The transactions of the clients end without effect, so that a
        // close does not start the next one.
        for conn in self.ipc.conn_ids() {
            let held = self.transactions.drop_conn(conn);
            self.release(&held);
        }
        for conn in self.ipc.conn_ids() {
            self.ipc_close(conn);
        }
        self.end_graces();
        self.grants.clear();
        self.node_grab = None;
        self.button_grab = None;
        self.held.grabs.clear();
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

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::testing::{Client, Core};

    const REFRESH: Duration = Duration::from_millis(16);
    const SLOW: Duration = Duration::from_millis(20);

    /// Notes slow frames of an output at 60 Hz for a time, from a start,
    /// and returns the actions; a blame makes `suspect` the suspect.
    fn slow_for(
        watchdog: &mut Watchdog,
        start: Instant,
        ms: u64,
        suspect: Option<&str>,
    ) -> Vec<Action> {
        let mut actions = Vec::new();
        for i in 0..ms / 16 {
            let now = start + Duration::from_millis(i * 16);
            if let Some(action) = watchdog.note("out", REFRESH, SLOW, now) {
                if action == Action::Blame {
                    watchdog.suspect = suspect.map(Into::into);
                }
                actions.push(action);
            }
        }
        actions
    }

    #[test]
    fn slow_frames_blame_a_client_and_then_disconnect_it_once() {
        let mut watchdog = Watchdog::default();
        let start = Instant::now();
        let actions = slow_for(&mut watchdog, start, 4100, Some("heavy"));
        assert_eq!(actions, [Action::Blame, Action::Disconnect("heavy".into())]);
        // Slow again soon after: a blame, but no second disconnect.
        let later = start + Duration::from_millis(4200);
        let actions = slow_for(&mut watchdog, later, 4100, Some("heavy"));
        assert_eq!(actions, [Action::Blame]);
    }

    #[test]
    fn without_a_suspect_slow_frames_never_disconnect() {
        let mut watchdog = Watchdog::default();
        let actions = slow_for(&mut watchdog, Instant::now(), 20_000, None);
        assert!(actions.iter().all(|a| *a == Action::Blame));
    }

    #[test]
    fn a_fast_frame_starts_the_time_again() {
        let mut watchdog = Watchdog::default();
        let start = Instant::now();
        assert_eq!(slow_for(&mut watchdog, start, 1900, Some("a")), []);
        let later = start + Duration::from_millis(1950);
        assert_eq!(
            watchdog.note("out", REFRESH, Duration::from_millis(5), later),
            None
        );
        assert_eq!(slow_for(&mut watchdog, later, 1900, Some("a")), []);
    }

    /// Renders slow frames for a time on the clock of the core.
    fn slow_frames(core: &mut Core, ms: u64) {
        for _ in 0..ms / 16 {
            core.state.time.advance(REFRESH);
            core.state.frame_rendered("out", REFRESH, SLOW);
        }
        core.dispatch();
    }

    fn effect(id: &str) -> Value {
        json!({"create": {"id": id, "kind": "group", "parent": "root",
            "props": {"size": [10, 10], "effects": [{"shader": "/e.frag"}]}}})
    }

    fn effects(core: &Core, id: &str) -> usize {
        core.state.scene.node(id).unwrap().props.effects.len()
    }

    /// A core with a window manager that listens, a loader with one effect,
    /// and a client with `keep_nodes` and 20 effects.
    fn three_clients() -> (Core, Client, Client, Client) {
        three_clients_with(&json!({"name": "heavy", "keep_nodes": true}))
    }

    /// `three_clients` with another `hello` of the heavy client.
    fn three_clients_with(hello: &Value) -> (Core, Client, Client, Client) {
        let mut core = Core::new();
        let mut wm = core.named("wm");
        wm.ok(
            &mut core,
            "events.subscribe",
            json!({"events": ["effects.dropped"]}),
        );
        let mut loader = core.named("loader");
        let glass = json!({"commands": [effect("glass")]});
        loader.ok(&mut core, "scene.apply", glass);
        let mut heavy = core.connect();
        heavy.ok(&mut core, "hello", hello.clone());
        let commands: Vec<Value> = (0..20).map(|i| effect(&format!("h{i}"))).collect();
        heavy.ok(&mut core, "scene.apply", json!({"commands": commands}));
        wm.take_events();
        (core, wm, loader, heavy)
    }

    #[test]
    fn the_watchdog_drops_the_effects_of_the_client_that_costs_the_most() {
        let (mut core, mut wm, _loader, _heavy) = three_clients();
        slow_frames(&mut core, 2100);
        let events = wm.events_now(&mut core);
        assert_eq!(events.len(), 1, "{events:?}");
        assert_eq!(events[0]["params"]["event"], "effects.dropped");
        assert_eq!(events[0]["params"]["client"], "heavy");
        assert_eq!(effects(&core, "h0"), 0);
        assert_eq!(effects(&core, "glass"), 1, "other clients keep theirs");
    }

    #[test]
    fn frames_that_stay_slow_disconnect_only_that_client() {
        let (mut core, mut wm, mut loader, mut heavy) = three_clients();
        slow_frames(&mut core, 4200);
        assert!(heavy.closed(&mut core));
        let gone = core.state.scene.node("h0").is_none();
        assert!(gone, "its nodes go, also with keep_nodes");
        assert!(!loader.closed(&mut core));
        assert!(!wm.closed(&mut core));
        assert_eq!(effects(&core, "glass"), 1);
    }

    #[test]
    fn the_watchdog_ends_the_grace_time_of_a_client() {
        let hello = json!({"name": "heavy", "resume": true});
        let (mut core, _wm, _loader, mut heavy) = three_clients_with(&hello);
        slow_frames(&mut core, 4200);
        assert!(heavy.closed(&mut core));
        assert!(core.state.scene.node("h0").is_none(), "no grace time");
    }
}
