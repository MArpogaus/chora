//! The safety layer: the only built-in behavior (D23).
//!
//! Besides the bindings, the reset and the injection mark (D23), a
//! watchdog watches the frames of each output. After two seconds of
//! slow frames it removes the effects of the client whose visible nodes
//! cost the most. It disconnects that client, at most once a minute,
//! only when its own load reaches the owner budget (D57); below it the
//! slowdown is shared and the watchdog removes the effects of all
//! clients and disconnects none. The emergency reset is for the
//! binding and SIGUSR1 only. `CHORA_FRAME_BUDGET_US` sets a fixed
//! budget in microseconds, for tests, `CHORA_OWNER_LOAD_BUDGET` the
//! owner budget.

use calloop::{
    LoopHandle,
    signals::{Signal, Signals},
};
use std::{
    collections::{BTreeMap, HashMap},
    time::{Duration, Instant},
};

use chora_scene::{Effect, Event};

use crate::state::State;

/// SIGUSR1 starts the emergency reset, for example from a text console.
/// It works while the event loop dispatches: a hung shader blocks the
/// loop and needs a kill.
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
/// The shortest time between two disconnects of the same client.
const DISCONNECT_PAUSE: Duration = Duration::from_secs(60);
/// The load one owner may put on a frame before the watchdog may
/// disconnect it, as `Scene::load_by_owner` counts it (D57): on a slow
/// GPU, a window manager with a few nodes must not go, and two clients
/// that are each half the fault both keep their connection.
/// `CHORA_OWNER_LOAD_BUDGET` tunes it.
const OWNER_BUDGET: u64 = 4000;

/// The watchdog of the render times.
pub struct Watchdog {
    /// The slow frames of each output.
    outputs: HashMap<String, Run>,
    /// The client that the watchdog blamed; if frames stay slow, it goes.
    suspect: Option<String>,
    /// When each client went last.
    last_disconnect: HashMap<String, Instant>,
    /// A fixed budget from `CHORA_FRAME_BUDGET_US`.
    budget: Option<Duration>,
    /// What one owner may cost before it may be disconnected.
    owner_budget: u64,
}

impl Default for Watchdog {
    fn default() -> Self {
        Self {
            outputs: HashMap::new(),
            suspect: None,
            last_disconnect: HashMap::new(),
            budget: None,
            owner_budget: OWNER_BUDGET,
        }
    }
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
    /// A watchdog with the budgets of the environment, if any.
    pub fn from_env() -> Self {
        let budget = std::env::var("CHORA_FRAME_BUDGET_US")
            .ok()
            .and_then(|v| v.parse().ok())
            .map(Duration::from_micros);
        let owner_budget = std::env::var("CHORA_OWNER_LOAD_BUDGET")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(OWNER_BUDGET);
        Self {
            budget,
            owner_budget,
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
        // a slow GPU. Another client must not wait for the pause: while the
        // watchdog holds back, its nodes starve the core.
        if self
            .last_disconnect
            .get(&client)
            .is_some_and(|t| now - *t < DISCONNECT_PAUSE)
        {
            self.suspect = Some(client);
            return None;
        }
        self.last_disconnect.insert(client.clone(), now);
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

    /// Removes the effects of the client whose visible nodes cost the
    /// most, and makes it the suspect, so that it goes when frames stay
    /// slow. Below the owner budget no client alone causes the slowdown:
    /// then the effects of all clients go and no suspect is made (D57).
    fn blame(&mut self) {
        let load = self.scene.load_by_owner();
        let Some((top, max)) = load.iter().max_by_key(|(_, load)| *load) else {
            return;
        };
        if *max < self.watchdog.owner_budget {
            self.drop_all_effects(*max, &load);
            return;
        }
        tracing::warn!(client = %top, load = *max, "frames are slow; the watchdog blames a client");
        if self.scene.drop_effects(top) {
            self.emit(&Event::EffectsDropped {
                client: top.clone(),
            });
        }
        self.watchdog.suspect = Some(top.clone());
    }

    /// No client alone reaches the owner budget: the slowdown is shared,
    /// so the effects of all clients go and no suspect is made (D57).
    fn drop_all_effects(&mut self, top_load: u64, load: &BTreeMap<String, u64>) {
        tracing::warn!(
            load = top_load,
            "frames are slow; no client alone reaches the owner budget; the watchdog drops all effects"
        );
        for client in load.keys() {
            if self.scene.drop_effects(client) {
                self.emit(&Event::EffectsDropped {
                    client: client.clone(),
                });
            }
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

    /// The revoke binding (D23): takes the capabilities away from every
    /// client, as a full `grants.revoke` does. It also drops the unused
    /// activation tokens of no client. The connections stay.
    pub fn revoke_all(&mut self) {
        self.forget_unused_tokens(std::time::Duration::ZERO);
        let names = self.grants.revoke_all();
        if names.is_empty() {
            return;
        }
        tracing::warn!(clients = ?names, "the revoke binding took the capabilities of every client");
        for name in names {
            // Only the clients that a capability gave a protocol go.
            self.disconnect_wayland(&name, &chora_scene::ipc::Capability::ALL);
            self.revoke_free_grabs(&name);
        }
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
        self.remove_client_sockets();
        self.grants.revoke_all();
        self.inject_mark = None;
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

    /// The Linux key codes of Ctrl, Alt and i.
    const CTRL: u32 = 29;
    const ALT: u32 = 56;
    const I: u32 = 23;

    /// Presses or releases a key on the real keyboard path.
    fn press(core: &mut Core, linux: u32, down: bool) {
        use smithay::{
            backend::input::{InputTime, KeyState},
            input::keyboard::Keycode,
        };
        crate::input::key(
            &mut core.state,
            Keycode::new(linux + 8),
            if down {
                KeyState::Pressed
            } else {
                KeyState::Released
            },
            InputTime::now(),
            false,
        );
    }

    #[test]
    fn the_revoke_binding_takes_the_capabilities_of_all_clients() {
        let mut core = Core::new();
        let mut ctl = core.admin();
        let agent = json!({"client": "agent", "capabilities": ["input.inject", "input.grab-any"]});
        ctl.ok(&mut core, "grants.set", agent);
        let eye = json!({"client": "eye", "capabilities": ["screen.capture"]});
        ctl.ok(&mut core, "grants.set", eye);
        let mut agent = core.named("agent");
        agent.ok(&mut core, "input.inject", json!({"events": []}));
        // An injected shortcut never reaches the safety layer (D23).
        let shortcut = json!({"events": [{"shortcut": {"keys": "Ctrl+Alt+i"}}]});
        agent.ok(&mut core, "input.inject", shortcut);
        let list = ctl.ok(&mut core, "grants.list", Value::Null);
        assert_eq!(
            list["agent"],
            json!(["input.inject", "input.grab-any"]),
            "{list}"
        );
        // A waiting token of no client goes with the grants.
        ctl.ok(&mut core, "activation.token", Value::Null);
        assert_eq!(core.state.activation_state.tokens().count(), 1);
        // The fixed binding: Ctrl+Alt+i.
        press(&mut core, CTRL, true);
        press(&mut core, ALT, true);
        press(&mut core, I, true);
        press(&mut core, I, false);
        assert_eq!(ctl.ok(&mut core, "grants.list", Value::Null), json!({}));
        assert_eq!(core.state.activation_state.tokens().count(), 0);
        let (_, message) = agent.fails(&mut core, "input.inject", json!({"events": []}));
        assert!(message.contains("needs a grant"), "{message}");
        assert!(!ctl.closed(&mut core), "the connections stay");
        assert!(!agent.closed(&mut core), "the connections stay");
    }

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
    fn the_pause_covers_only_the_client_that_went() {
        let mut watchdog = Watchdog::default();
        let start = Instant::now();
        assert_eq!(
            slow_for(&mut watchdog, start, 4100, Some("a")),
            [Action::Blame, Action::Disconnect("a".into())]
        );
        // Right after: the same client stays, but a new suspect goes at once.
        let later = start + Duration::from_millis(4200);
        assert_eq!(
            slow_for(&mut watchdog, later, 4100, Some("a")),
            [Action::Blame]
        );
        // Frames get fast again, so the suspect is forgotten; the pause of
        // "a" stays in place.
        let calm = later + Duration::from_secs(15);
        for i in 0..1000 {
            watchdog.note(
                "out",
                REFRESH,
                Duration::from_millis(1),
                calm + Duration::from_millis(i),
            );
        }
        assert_eq!(
            slow_for(
                &mut watchdog,
                calm + Duration::from_millis(1000),
                4100,
                Some("b")
            ),
            [Action::Blame, Action::Disconnect("b".into())]
        );
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
    /// and a client with `keep_nodes` and 80 effects (load 4080, over the
    /// default owner budget).
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
        let commands: Vec<Value> = (0..80).map(|i| effect(&format!("h{i}"))).collect();
        heavy.ok(&mut core, "scene.apply", json!({"commands": commands}));
        wm.take_events();
        (core, wm, loader, heavy)
    }

    /// A named client with `effects` effect nodes and `plain` plain ones:
    /// a load of 51 * effects + plain, as `Scene::load_by_owner` counts.
    fn stress_client(core: &mut Core, name: &str, effects: u32, plain: u32) -> Client {
        let mut client = core.named(name);
        let mut commands: Vec<Value> = (0..effects)
            .map(|i| effect(&format!("{name}e{i}")))
            .collect();
        commands.extend((0..plain).map(|i| {
            json!({"create": {"id": format!("{name}p{i}"), "kind": "group",
                "parent": "root", "props": {"size": [10, 10]}}})
        }));
        client.ok(core, "scene.apply", json!({"commands": commands}));
        client
    }

    /// The battle finding: two stress clients make the frames slow
    /// together. Neither alone reaches the owner budget (4000), so the
    /// watchdog drops effects and disconnects neither (D57).
    #[test]
    fn a_shared_slowdown_disconnects_neither_owner() {
        let mut core = Core::new();
        let mut wm = core.named("wm");
        wm.ok(
            &mut core,
            "events.subscribe",
            json!({"events": ["effects.dropped"]}),
        );
        let mut snake = stress_client(&mut core, "snake", 50, 450); // load 3000
        let mut budget = stress_client(&mut core, "budget", 40, 460); // load 2500
        let loads: Vec<u64> = core.state.scene.load_by_owner().values().copied().collect();
        assert_eq!(loads, [2500, 3000], "alphabetical owners");
        wm.take_events();
        slow_frames(&mut core, 8000);
        assert!(!snake.closed(&mut core), "3000 is under the budget");
        assert!(!budget.closed(&mut core), "2500 is under the budget");
        assert_eq!(effects(&core, "snakee0"), 0);
        assert_eq!(effects(&core, "budgete0"), 0);
        let events = wm.events_now(&mut core);
        let clients: Vec<&str> = events
            .iter()
            .map(|e| e["params"]["client"].as_str().unwrap())
            .collect();
        assert_eq!(clients, ["budget", "snake"], "{events:?}");
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
