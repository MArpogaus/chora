//! What happens to the windows of a virtual output that goes: no window
//! stays where no screen shows it.

use serde_json::{Value, json};

use super::{SIDE, SLOT_1};
use crate::{
    state::RESERVATION,
    testing::{Client, Core, wayland::Wayland},
};

/// A client whose group claims an area.
fn manager(core: &mut Core, name: &str, claim: [i32; 4]) -> Client {
    let mut client = core.named(name);
    let group = json!({"commands": [{"create": {"id": name, "kind": "group",
        "parent": "root", "props": {"claim": claim}}}]});
    client.ok(core, "scene.apply", group);
    client
}

fn apply(client: &mut Client, core: &mut Core, commands: &Value) {
    client.ok(core, "scene.apply", json!({"commands": commands}));
}

/// The left half of a screen for "construct", the right half for "desk",
/// and a virtual output of "construct" for "xp", which got the window
/// `s1` from "construct" and shows it there.
struct Session {
    core: Core,
    construct: Option<Client>,
    xp: Option<Client>,
    _desk: Client,
    _wayland: Wayland,
}

fn session() -> Session {
    let mut core = Core::new();
    core.add_output("screen", (128, 96));
    let mut construct = manager(&mut core, "construct", [0, 0, 64, 96]);
    let desk = manager(&mut core, "desk", [64, 0, 64, 96]);
    let add = json!({"spec": "32x32", "virtual": true});
    construct.ok(&mut core, "output.add", add);
    let mut xp = manager(&mut core, "xp", [SLOT_1, SLOT_1, SIDE, SIDE]);
    let mut wayland = Wayland::connect(&mut core);
    let window = wayland.toplevel(&mut core, "window");
    wayland.attach(&mut core, &window, (20, 20));
    let take = json!([{"attach": {"surface": "s1", "parent": "construct"}}]);
    apply(&mut construct, &mut core, &take);
    apply(
        &mut construct,
        &mut core,
        &json!([{"remove": {"id": "s1", "to": "xp"}}]),
    );
    let on_the_face = json!([
        {"attach": {"surface": "s1", "parent": "xp"}},
        {"update": {"id": "s1", "props": {"transform": {"translate": [SLOT_1, SLOT_1]}}}},
    ]);
    apply(&mut xp, &mut core, &on_the_face);
    core.state.pointer_layout = (100.0, 50.0).into();
    Session {
        core,
        construct: Some(construct),
        xp: Some(xp),
        _desk: desk,
        _wayland: wayland,
    }
}

impl Session {
    fn remove_the_output(&mut self) {
        let remove = json!({"output": "VIRTUAL-1"});
        let construct = self.construct.as_mut().expect("the construct");
        construct.ok(&mut self.core, "output.remove", remove);
    }

    fn owner(&self) -> Option<String> {
        let node = self.core.state.scene.node("s1").expect("the window");
        node.owner.clone()
    }

    fn reserved_for(&self) -> Option<String> {
        let node = self.core.state.scene.node("s1").expect("the window");
        node.to.clone()
    }

    fn on_the_screen(&mut self) -> bool {
        let query = json!({"selector": "id:s1"});
        let construct = self.construct.as_mut().expect("the construct");
        let found = construct.ok(&mut self.core, "scene.query", query);
        found[0]["rects"].get("screen").is_some()
    }
}

#[test]
fn a_stranded_window_lands_on_a_screen_after_5_s() {
    let mut session = session();
    session.remove_the_output();
    session.core.advance(RESERVATION / 2);
    assert_eq!(session.owner().as_deref(), Some("xp"), "xp has time");
    session.core.advance(RESERVATION);
    assert_eq!(session.owner(), None, "the core takes it from xp");
    assert_eq!(
        session.reserved_for().as_deref(),
        Some("construct"),
        "the giver comes first, not the manager under the pointer"
    );
    session.core.advance(RESERVATION * 2);
    assert!(session.on_the_screen(), "the core shows it at last");
}

#[test]
fn a_window_that_its_manager_moved_to_a_screen_stays_with_it() {
    let mut session = session();
    session.remove_the_output();
    let moved = json!([{"update": {"id": "s1", "props": {"transform": {"translate": [70, 10]}}}}]);
    let xp = session.xp.as_mut().expect("xp");
    apply(xp, &mut session.core, &moved);
    session.core.advance(RESERVATION * 3);
    assert_eq!(session.owner().as_deref(), Some("xp"));
}

#[test]
fn a_killed_manager_returns_its_windows_to_the_giver() {
    let mut session = session();
    drop(session.xp.take());
    session.core.dispatch();
    assert_eq!(session.owner(), None);
    assert_eq!(
        session.reserved_for().as_deref(),
        Some("construct"),
        "the giver, not the manager under the pointer"
    );
}

#[test]
fn the_reset_removes_virtual_outputs_first() {
    let mut session = session();
    session.core.state.reset();
    let names: Vec<_> = (session.core.state.all_outputs().iter())
        .map(smithay::output::Output::name)
        .collect();
    assert_eq!(names, ["screen"]);
    let placed = |core: &Core| core.state.scene.node("s1").expect("the window").clone();
    let in_the_grid = placed(&session.core);
    assert_eq!(in_the_grid.parent.as_deref(), Some("root"));
    session.core.advance(RESERVATION * 3);
    assert_eq!(placed(&session.core), in_the_grid, "nothing waits for it");
}

/// A random number below `below`, from a seed that it moves on.
fn pick(seed: &mut u64, below: u64) -> u64 {
    *seed = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    (*seed >> 33) % below
}

/// One random step of a session: the output goes or comes back, xp or
/// the construct ends, xp moves the window, or time passes.
fn step(session: &mut Session, seed: &mut u64) {
    let Session {
        core,
        construct,
        xp,
        ..
    } = session;
    let mut ask = |method: &str, params: Value| {
        let _ = construct.as_mut().map(|c| c.send(method, params));
    };
    match pick(seed, 6) {
        0 => ask("output.remove", json!({"output": "VIRTUAL-1"})),
        1 => ask("output.add", json!({"spec": "32x32", "virtual": true})),
        2 => drop(xp.take()),
        3 => {
            let at = [pick(seed, 100), pick(seed, 80)];
            let moved = json!({"commands": [{"update": {"id": "s1",
                "props": {"transform": {"translate": at}}}}]});
            let _ = xp.as_mut().map(|xp| xp.send("scene.apply", moved));
        }
        4 => drop(construct.take()),
        _ => core.advance(RESERVATION / 3),
    }
    core.dispatch();
}

/// The window is with a live client that shows it on an output, or the
/// core shows it on the screen.
fn settled(session: &mut Session) -> bool {
    let state = &session.core.state;
    let node = state
        .scene
        .node("s1")
        .expect("the window stays in the scene");
    let (x, y) = node.props.transform.translation_2d();
    let on_the_screen = (0.0..128.0).contains(&x) && y >= 0.0;
    let on_the_face = x < 0.0 && state.all_outputs().len() == 2;
    match &node.owner {
        Some(owner) => state.ipc.has_client(owner) && (on_the_screen || on_the_face),
        None => node.parent.as_deref() == Some("root") && on_the_screen,
    }
}

#[test]
fn no_window_is_lost_over_200_random_sequences() {
    for sequence in 0..200 {
        let mut session = session();
        let mut seed = sequence;
        for _ in 0..12 {
            step(&mut session, &mut seed);
            let surfaces = session.core.state.scene.nodes();
            let windows = surfaces.filter(|n| n.surface.is_some()).count();
            assert_eq!(windows, 1, "sequence {sequence}: the window is gone");
        }
        session.core.advance(RESERVATION * 3);
        session.core.advance(RESERVATION * 3);
        assert!(settled(&mut session), "sequence {sequence}: a lost window");
    }
}
