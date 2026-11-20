//! Unit tests of the hand on the cube and of the start of the cube's
//! window manager (`reopen_left_cube`, `manager_runs`, `face_manager`),
//! on a fake core that answers the requests they make.

use std::time::{Duration, Instant};

use chora_kit::{
    modes::Act,
    testing::{FakeCore, Log},
};
use chora_scene::{OutputInfo, Transform};
use serde_json::{Value, json};

use super::{FACES, Health, Turn, tests::host_of};
use crate::{Config, Construct, Flown, NAME, Pose, config, settings};

/// A construct on a fake core that answers each request with `answer`,
/// which gets the method and the parameters; the methods that it saw.
///
/// These cells do not build the pane tree that the cube hangs on: the fake
/// core takes the commands as they come, and the test reads them.
fn on_core(tag: &str, answer: fn(&str, &Value) -> Option<Value>) -> (Construct, Log) {
    let (construct, methods, _core) = on_fake_core(tag, answer);
    (construct, methods)
}

/// As [`on_core`], with the fake core, whose model keeps the emits.
fn on_fake_core(
    tag: &str,
    answer: fn(&str, &Value) -> Option<Value>,
) -> (Construct, Log, FakeCore) {
    let dir = chora_kit::testing::TempDir::new(&format!("chora-construct-moved-{tag}")).keep();
    std::fs::create_dir_all(&dir).unwrap();
    let core = FakeCore::start();
    core.respond(answer);
    core.respond(|method, _| (method == "scene.apply").then_some(Value::Null));
    let methods = core.record(|method, _| Some(method.to_owned()));
    let mut rt = core.runtime(NAME);
    rt.pin_accepts_injected(false);
    let settings =
        settings::File::<Config>::load(dir.join("chora-construct.yaml"), config::DEFAULT).unwrap();
    let shaders = |name: &str, _: &str| Ok(format!("{}/{name}", dir.display()));
    let mut construct = Construct::new(rt, settings, false, &shaders).unwrap();
    construct.active = true;
    (construct, methods, core)
}

/// How many requests of the method `method` the fake core saw.
fn count(methods: &Log, method: &str) -> usize {
    methods
        .lock()
        .unwrap()
        .iter()
        .filter(|m| *m == method)
        .count()
}

/// The virtual outputs of the names `names`, owned by this plugin.
fn virtual_outputs(names: &[&str]) -> Value {
    let outputs: Vec<OutputInfo> = names
        .iter()
        .map(|name| OutputInfo {
            name: (*name).into(),
            position: [0, 0],
            size: [1280, 800],
            scale: 1.0,
            camera: Transform::default(),
            work_area: [0, 0, 1280, 800],
            default_work_area: [0; 4],
            is_virtual: Some(true),
            owner: Some(NAME.into()),
        })
        .collect();
    json!(outputs)
}

/// A press with the modifier on a cube at rest in use mode takes the
/// cube; without a cube, or outside its use mode, the hand does nothing.
#[test]
fn the_hand_takes_a_cube_at_rest_in_use_mode() {
    let (mut construct, _) = on_core("hand-take", |_, _| None);
    assert!(!construct.hand_press([10.0, 10.0]), "no cube, no hand");
    construct.cube = Some(host_of(FACES));
    assert!(
        construct.hand_press([10.0, 10.0]),
        "a cube at rest takes the hand"
    );
    let held = construct.cube.as_ref().expect("the cube");
    assert!(held.press.is_some(), "the cube holds the press");

    let (mut construct, _) = on_core("hand-navigate", |_, _| None);
    let mut navigate = host_of(FACES);
    navigate.view.act(Act::StepBack);
    construct.cube = Some(navigate);
    assert!(
        !construct.hand_press([10.0, 10.0]),
        "in navigate mode the hand waits"
    );

    let (mut construct, _) = on_core("hand-turning", |_, _| None);
    let mut turning = host_of(FACES);
    turning.turn = Some(Turn::new(0.0, 1.0, Instant::now(), Duration::from_secs(1)));
    construct.cube = Some(turning);
    assert!(
        !construct.hand_press([10.0, 10.0]),
        "a cube that turns is not taken"
    );
}

/// A press on the cube while the camera is in a window does nothing: the
/// window has the input, not the cube.
#[test]
fn the_hand_waits_while_the_camera_is_in_a_window() {
    let (mut construct, _) = on_core("hand-window", |_, _| None);
    construct.cube = Some(host_of(FACES));
    construct.flown = Some(Flown {
        surface: "s1".into(),
        back: Pose::default(),
        attached: true,
        plugin: None,
    });
    assert!(!construct.hand_press([10.0, 10.0]));
    assert!(construct.cube.as_ref().expect("the cube").press.is_none());
}

/// With a pane and a cube open, the construct holds no unbound Super chord:
/// the window manager of the faces swallows for its own windows, and a
/// chord that it binds later must not be lost to the construct.
#[test]
fn a_pane_and_a_cube_hold_no_unbound_super_chord() {
    let (mut construct, _) = on_core("swallow-both", |_, _| None);
    construct.flown = Some(Flown {
        surface: "s1".into(),
        back: Pose::default(),
        attached: true,
        plugin: None,
    });
    construct.regrab().unwrap();
    assert!(
        construct.rt.action("Super+q").is_some(),
        "a pane alone swallows"
    );
    construct.cube = Some(host_of(FACES));
    construct.regrab().unwrap();
    assert!(construct.in_pane(), "the camera stays in the pane");
    assert!(
        construct.rt.action("Super+q").is_none(),
        "with a cube open the face manager swallows, not the construct"
    );
}

/// A press on a pane in the walk gives its window the keyboard where the
/// pane stands; the camera is in no pane. The construct then swallows the
/// unbound Super chords and the tap of Super for that window, and keeps
/// its own chords, as in a pane that the camera flew into (co-25, persona
/// Max of p2-r2); the walk that takes the keyboard back lets them go.
#[test]
fn a_pane_window_with_the_keyboard_from_a_press_holds_the_super_chords() {
    use chora_kit::runtime::Plugin as _;
    let (mut construct, _) = on_core("swallow-pressed", |_, _| None);
    construct.panes.push(crate::Pane {
        name: "probe".into(),
        source: "s1".into(),
        content: crate::Content::Window { command: None },
        at: [0.0, 1500.0],
        yaw: 0.0,
        size: [640.0, 400.0],
        taken: false,
        attached: true,
        placed: false,
    });
    construct.regrab().unwrap();
    assert!(
        construct.rt.action("Super+q").is_none(),
        "the walk holds none"
    );
    let focus = chora_kit::testing::focus_changed;
    construct.event(focus(Some("s1"))).unwrap();
    assert!(!construct.in_pane(), "the camera is in no pane");
    for chord in ["Super+q", "Super+d", "Super+Tab", "Super+Super_L"] {
        assert!(
            construct.rt.action(chord).is_some(),
            "{chord} reaches the pressed window"
        );
    }
    assert!(
        construct.rt.action("Super+F1").is_some(),
        "the key list keeps its key"
    );
    construct.event(focus(None)).unwrap();
    for chord in ["Super+q", "Super+Super_L", "Super+F1"] {
        assert!(
            construct.rt.action(chord).is_none(),
            "{chord} stays held after the walk took the keyboard"
        );
    }
}

/// A held cube turns with the hand; a hand that holds nothing moves nothing
/// and releases nothing.
#[test]
fn the_held_cube_turns_with_the_hand() {
    let (mut construct, _) = on_core("hand-drag", |_, _| None);
    assert!(
        !construct.hand_moved([0.0, 0.0]).unwrap(),
        "no cube, nothing moves"
    );
    assert!(
        !construct.hand_released().unwrap(),
        "no cube, nothing lets go"
    );
    construct.cube = Some(host_of(FACES));
    assert!(
        !construct.hand_moved([400.0, 100.0]).unwrap(),
        "no press, nothing moves"
    );
    assert!(construct.hand_press([100.0, 100.0]));
    let before = construct.cube.as_ref().expect("the cube").angle;
    assert!(
        construct.hand_moved([400.0, 100.0]).unwrap(),
        "the held cube moves"
    );
    let moved = construct.cube.as_ref().expect("the cube").angle;
    assert!(
        (moved - before).abs() > 1e-9,
        "the cube turns with the hand"
    );
}

/// A release lets go of the held cube, which settles on a face; a second
/// release has no hand to let go.
#[test]
fn the_release_settles_the_held_cube() {
    let (mut construct, _) = on_core("hand-release", |_, _| None);
    construct.cube = Some(host_of(FACES));
    assert!(construct.hand_press([100.0, 100.0]));
    assert!(construct.hand_moved([400.0, 100.0]).unwrap());
    assert!(construct.hand_released().unwrap(), "the release is taken");
    let host = construct.cube.as_ref().expect("the cube");
    assert!(host.press.is_none(), "the hand let go");
    assert!(host.turn.is_some(), "the cube settles on a face");
    assert!(
        !construct.hand_released().unwrap(),
        "a second release has no hand"
    );
}

/// The manager runs when a client of its name is connected, and not
/// otherwise.
#[test]
fn the_manager_runs_when_its_client_is_connected() {
    let (mut connected, _) = on_core("start-connected", |method, _| match method {
        "clients.list" => Some(json!([{"name": "other"}, {"name": "cube-test"}])),
        _ => None,
    });
    connected.settings.value.cube_manager = "chora-cube-test --quiet".into();
    assert!(connected.manager_runs().unwrap());

    let (mut absent, _) = on_core("start-absent", |method, _| match method {
        "clients.list" => Some(json!([{"name": "other"}])),
        _ => None,
    });
    absent.settings.value.cube_manager = "chora-cube-test".into();
    assert!(!absent.manager_runs().unwrap());
}

/// Without a cube, the manager of the faces is the client of the setting
/// program, without its path and arguments; with an open cube, the
/// client of the cube's own program.
#[test]
fn the_face_manager_is_the_client_of_the_cube_or_of_the_setting() {
    let (mut construct, _) = on_core("start-face", |_, _| None);
    construct.settings.value.cube_manager = "/usr/bin/chora-cube-test --x".into();
    assert_eq!(construct.face_manager(), "cube-test");
    let mut open = host_of(FACES);
    open.program = "chora-open".into();
    construct.cube = Some(open);
    assert_eq!(
        construct.face_manager(),
        "open",
        "the open cube names the manager"
    );
}

/// Virtual outputs of this plugin that a running manager still shows
/// reopen the cube on its faces; no output is removed. The core echoes the
/// binding it grabs, which is what the grab of the keys expects.
#[test]
#[allow(
    clippy::disallowed_methods,
    reason = "the test answers the requests of its fake core"
)]
fn a_cube_left_by_a_running_manager_reopens_on_its_faces() {
    let (mut construct, methods) = on_core("start-reopen", |method, _| match method {
        "output.list" => Some(virtual_outputs(&[
            "VIRTUAL-1",
            "VIRTUAL-2",
            "VIRTUAL-3",
            "VIRTUAL-4",
        ])),
        "clients.list" => Some(json!([{"name": "cube-test"}])),
        _ => None,
    });
    construct.settings.value.cube_manager = "chora-cube-test".into();
    construct.reopen_left_cube().unwrap();
    let host = construct.cube.as_ref().expect("the cube reopens");
    assert_eq!(
        host.outputs,
        ["VIRTUAL-1", "VIRTUAL-2", "VIRTUAL-3", "VIRTUAL-4"],
        "the faces are the left outputs"
    );
    assert!(host.health.iter().all(|h| matches!(h, Health::Up)));
    assert_eq!(count(&methods, "output.remove"), 0, "the outputs stay");
    assert!(
        count(&methods, "scene.apply") >= 1,
        "the faces are sent to the core"
    );
}

/// Without a running manager, the left outputs go: no cube reopens.
#[test]
#[allow(
    clippy::disallowed_methods,
    reason = "the test answers the requests of its fake core"
)]
fn left_outputs_go_when_their_manager_is_gone() {
    let (mut construct, methods) = on_core("start-gone", |method, _| match method {
        "output.list" => Some(virtual_outputs(&["VIRTUAL-1", "VIRTUAL-2"])),
        "clients.list" => Some(json!([])),
        _ => None,
    });
    construct.settings.value.cube_manager = "chora-cube-test".into();
    construct.reopen_left_cube().unwrap();
    assert!(construct.cube.is_none(), "no manager, no cube");
    assert_eq!(
        count(&methods, "output.remove"),
        2,
        "each left output is removed"
    );
}

/// A cube reopens only from one to the number of faces of outputs; none,
/// or more than the faces, go, and so does every output while the
/// construct is an overlay.
#[test]
#[allow(
    clippy::disallowed_methods,
    reason = "the test answers the requests of its fake core"
)]
fn outputs_reopen_only_as_a_cube_of_the_faces_and_not_as_an_overlay() {
    let (mut none, methods) = on_core("start-none", |method, _| match method {
        "output.list" => Some(json!([])),
        "clients.list" => Some(json!([{"name": "cube-test"}])),
        _ => None,
    });
    none.settings.value.cube_manager = "chora-cube-test".into();
    none.reopen_left_cube().unwrap();
    assert!(none.cube.is_none(), "no left output, no cube");
    assert_eq!(count(&methods, "output.remove"), 0);

    let (mut too_many, methods) = on_core("start-many", |method, _| match method {
        "output.list" => Some(virtual_outputs(&[
            "VIRTUAL-1",
            "VIRTUAL-2",
            "VIRTUAL-3",
            "VIRTUAL-4",
            "VIRTUAL-5",
        ])),
        "clients.list" => Some(json!([{"name": "cube-test"}])),
        _ => None,
    });
    too_many.settings.value.cube_manager = "chora-cube-test".into();
    too_many.reopen_left_cube().unwrap();
    assert!(too_many.cube.is_none(), "more outputs than faces, no cube");
    assert_eq!(count(&methods, "output.remove"), 5);

    let (mut overlay, methods) = on_core("start-overlay", |method, _| match method {
        "output.list" => Some(virtual_outputs(&[
            "VIRTUAL-1",
            "VIRTUAL-2",
            "VIRTUAL-3",
            "VIRTUAL-4",
        ])),
        "clients.list" => Some(json!([{"name": "cube-test"}])),
        _ => None,
    });
    overlay.overlay = true;
    overlay.settings.value.cube_manager = "chora-cube-test".into();
    overlay.reopen_left_cube().unwrap();
    assert!(
        overlay.cube.is_none(),
        "an overlay does not reopen the cube"
    );
    assert_eq!(count(&methods, "output.remove"), FACES);
}

/// wm7a-m3: the cube closed during a turn tells `construct.cube` once more,
/// at rest, so that a look that follows the event keeps no `turning: true`;
/// a cube closed at rest tells nothing more.
#[test]
fn a_cube_closed_in_a_turn_tells_its_rest_last() {
    let (mut construct, _methods, core) = on_fake_core("closed-in-turn", |_, _| None);
    let told = || {
        let emits = core.model().lock().unwrap().emits.clone();
        (emits.into_iter())
            .filter(|e| e["event"] == "construct.cube")
            .map(|e| e["data"].clone())
            .collect::<Vec<_>>()
    };
    let mut host = host_of(FACES);
    host.turn = Some(Turn::new(0.0, 1.0, Instant::now(), Duration::from_secs(4)));
    host.front = 1;
    construct.cube = Some(host);
    construct.tell_cube();
    assert_eq!(told(), [json!({"front": 1, "turning": true})], "the turn");
    construct.close_cube().unwrap();
    assert_eq!(
        told().last(),
        Some(&json!({"front": 1, "turning": false})),
        "the last word at rest"
    );
    let mut host = host_of(FACES);
    host.front = 2;
    construct.cube = Some(host);
    construct.tell_cube();
    let before = told().len();
    construct.close_cube().unwrap();
    assert_eq!(
        told().len(),
        before,
        "a cube at rest that goes tells nothing more"
    );
}
