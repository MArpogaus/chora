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
    desk: Option<Client>,
    wayland: Wayland,
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
        desk: Some(desk),
        wayland,
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

/// The owner of a virtual output aims its camera at a window that another
/// window manager keeps off the screens, and removes the output: the
/// window stays with its manager.
#[test]
fn a_camera_on_the_window_of_another_manager_takes_nothing() {
    let mut session = session();
    let second = session.wayland.toplevel(&mut session.core, "second");
    session.wayland.attach(&mut session.core, &second, (20, 20));
    let far = json!([
        {"attach": {"surface": "s2", "parent": "desk"}},
        {"update": {"id": "s2", "props": {"transform": {"translate": [5000, 5000]}}}},
    ]);
    let desk = session.desk.as_mut().expect("desk");
    apply(desk, &mut session.core, &far);
    let aim = json!([{"camera": {"output": "VIRTUAL-1",
        "transform": {"translate": [-5000, -5000]}}}]);
    let construct = session.construct.as_mut().expect("the construct");
    apply(construct, &mut session.core, &aim);
    session.remove_the_output();
    session.core.advance(RESERVATION * 2);
    let owner = |core: &Core| core.state.scene.node("s2").expect("s2").owner.clone();
    assert_eq!(owner(&session.core).as_deref(), Some("desk"));
}

#[test]
fn the_owner_hears_that_the_core_took_its_stranded_window() {
    let mut session = session();
    let xp = session.xp.as_mut().expect("xp");
    let subscribe = json!({"events": ["surface.detached"]});
    xp.ok(&mut session.core, "events.subscribe", subscribe);
    session.remove_the_output();
    session.core.advance(RESERVATION * 2);
    let xp = session.xp.as_mut().expect("xp");
    assert_eq!(xp.event_names(&mut session.core), ["surface.detached"]);
}

/// A window manager that took a free window got it from nobody: when it
/// goes, the client that gave the window to an earlier one is no giver.
#[test]
fn a_giver_counts_only_for_the_manager_that_it_gave_to() {
    let mut session = session();
    let xp = session.xp.as_mut().expect("xp");
    apply(xp, &mut session.core, &json!([{"remove": {"id": "s1"}}]));
    let mut late = manager(&mut session.core, "late", [0, 200, 10, 10]);
    let take = json!([{"attach": {"surface": "s1", "parent": "late"}}]);
    apply(&mut late, &mut session.core, &take);
    drop(late);
    session.core.dispatch();
    assert_eq!(
        session.reserved_for().as_deref(),
        Some("desk"),
        "the manager under the pointer, since nobody gave it to late"
    );
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

fn ask(client: &mut Option<Client>, method: &str, params: Value) {
    let _ = client.as_mut().map(|c| c.send(method, params));
}

fn change(client: &mut Option<Client>, commands: Value) {
    let params = serde_json::Map::from_iter([("commands".to_owned(), commands)]);
    ask(client, "scene.apply", Value::Object(params));
}

/// One random step of a session: the output goes or comes back under its
/// name, a window manager or the construct ends, xp moves, hides or gives
/// the window, desk takes or gives it, the construct aims the camera of
/// the output elsewhere, or time passes.
fn step(session: &mut Session, seed: &mut u64) {
    let Session {
        core,
        construct,
        xp,
        desk,
        ..
    } = session;
    let at = [pick(seed, 100), pick(seed, 80)];
    let give = |to: &str| json!([{"remove": {"id": "s1", "to": to}}]);
    match pick(seed, 12) {
        0 => ask(construct, "output.remove", json!({"output": "VIRTUAL-1"})),
        1 => ask(
            construct,
            "output.add",
            json!({"spec": "32x32", "virtual": true}),
        ),
        2 => drop(xp.take()),
        3 => change(
            xp,
            json!([{"update": {"id": "s1",
            "props": {"transform": {"translate": at}}}}]),
        ),
        4 => drop(construct.take()),
        5 => change(xp, give("desk")),
        6 => change(
            desk,
            json!([{"attach": {"surface": "s1", "parent": "desk"}},
            {"update": {"id": "s1", "props": {"transform": {"translate": [70, 10]}}}}]),
        ),
        7 => change(desk, give("construct")),
        8 => drop(desk.take()),
        9 => change(
            construct,
            json!([{"camera": {"output": "VIRTUAL-1",
            "transform": {"translate": at}}}]),
        ),
        10 => change(
            xp,
            json!([{"update": {"id": "s1", "props": {"visible": false}}}]),
        ),
        _ => core.advance(RESERVATION / 3),
    }
    core.dispatch();
}

/// The window is with a live client that keeps it on the screen, in the
/// slot of a virtual output that lives, or hidden; or the core shows it
/// on the screen.
fn settled(session: &mut Session) -> bool {
    let state = &session.core.state;
    let node = state
        .scene
        .node("s1")
        .expect("the window stays in the scene");
    let (x, y) = node.props.transform.translation_2d();
    let on_the_screen = (0.0..128.0).contains(&x) && (0.0..96.0).contains(&y);
    let slot = f64::from(SLOT_1);
    let in_the_slot = (slot..slot + f64::from(SIDE)).contains(&x);
    let on_the_face = in_the_slot && state.all_outputs().len() == 2;
    match &node.owner {
        Some(owner) => {
            state.ipc.has_client(owner) && (on_the_screen || on_the_face || !node.props.visible)
        }
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
