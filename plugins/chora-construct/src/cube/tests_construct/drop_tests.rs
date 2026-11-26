//! Unit tests of where the keyboard goes when the window of a pane goes
//! to the face manager while the cube stands back (co-drop r1, rcr5-1,
//! rcr5-2), on the fake core of `tests_construct`: a drop onto the face at
//! the front ends as the give key does (`put_in_face`), the person steps
//! in and the window has the keyboard; a drop onto another face, or a
//! window that the face manager could not place, leaves the keyboard with
//! the construct, so no key goes into a window the person cannot see.

use chora_kit::{
    modes::Mode,
    testing::{FakeCore, Log},
};
use chora_scene::{Command, NodeKind, PropsUpdate, ROOT};
use serde_json::{Value, json};

use super::{host_of, with_hint_bar};
use crate::{Construct, LOOK, Moving, PANES};

/// The cube of `with_hint_bar` stood back, its face manager `xp`, with
/// the look node of the construct, and the window `s1` of the pane
/// `foot` in the scene and in the hand of a drag: xp claims the outputs of the two faces, `VIRTUAL-1` (the front)
/// and `VIRTUAL-2`, side by side.
fn dragging_stood_back(tag: &str) -> (Construct, FakeCore, Log) {
    dragging_stood_back_on(tag, 2)
}

/// As [`dragging_stood_back`] with `faces` faces and outputs.
fn dragging_stood_back_on(tag: &str, faces: usize) -> (Construct, FakeCore, Log) {
    let (mut construct, core, lines) = with_hint_bar(tag);
    if faces != 2 {
        let mut host = host_of(faces);
        assert!(host.act(chora_kit::modes::Act::StepBack, std::time::Instant::now()));
        host.stand = 1.0;
        host.step = None;
        construct.cube = Some(host);
    }
    construct.cube.as_mut().unwrap().program = "xp".into();
    core.model().lock().unwrap().selects = true;
    let output = |name: &str, x: i32| {
        json!({"name": name, "position": [x, 0], "size": [1280, 800], "scale": 1.0,
            "camera": chora_scene::Transform::IDENTITY, "work_area": [0, 0, 1280, 800],
            "default_work_area": [0, 0, 0, 0], "virtual": true, "owner": crate::NAME})
    };
    let outputs: Vec<Value> = (0..faces)
        .map(|i| {
            output(
                &format!("VIRTUAL-{}", i + 1),
                2000 * i32::try_from(i).unwrap(),
            )
        })
        .collect();
    core.outputs(Value::Array(outputs));
    (core.model().lock().unwrap().scene).add_surface("s1", chora_scene::SurfaceInfo::default());
    let look = Command::Create {
        id: LOOK.into(),
        kind: NodeKind::Group,
        parent: ROOT.into(),
        props: PropsUpdate::default(),
    };
    let attach = Command::Attach {
        surface: "s1".into(),
        parent: PANES.into(),
    };
    construct.rt.conn.apply(vec![look, attach]).unwrap();
    construct.book.keep("s1");
    let mut xp = core.connect("xp");
    let desk = |i: usize, x: f64| Command::Create {
        id: format!("xp-desk-{i}"),
        kind: NodeKind::Group,
        parent: ROOT.into(),
        props: PropsUpdate {
            claim: Some(Some([x, 0.0, 1280.0, 800.0])),
            ..PropsUpdate::default()
        },
    };
    let desks = (0..faces)
        .map(|i| desk(i, 2000.0 * f64::from(u16::try_from(i).unwrap())))
        .collect();
    xp.apply(desks).unwrap();
    drop(xp);
    construct.moving = Some(Moving {
        name: "foot".into(),
        from: [0.0; 2],
        at: [-200.0, 700.0],
        yaw: 0.0,
    });
    lines.lock().unwrap().clear();
    (construct, core, lines)
}

/// The methods that the construct called on other clients, by
/// `methods.call`, with their parameters.
fn called(lines: &Log) -> Vec<(String, Value)> {
    (lines.lock().unwrap().iter())
        .filter_map(|line| line.strip_prefix("methods.call "))
        .filter_map(|params| serde_json::from_str::<Value>(params).ok())
        .map(|p| (p["method"].as_str().unwrap_or_default().to_owned(), p))
        .collect()
}

/// Whether the construct asked the face manager to give `surface` the
/// keyboard.
fn activated(lines: &Log, surface: &str) -> bool {
    called(lines)
        .iter()
        .any(|(m, p)| m == "activate" && p["params"]["surface"] == surface)
}

fn mode(construct: &Construct) -> Mode {
    construct.cube.as_ref().unwrap().view().mode()
}

/// The construct's own node has the keyboard, and no surface: a key goes
/// to the cube stood back.
fn keys_with_the_construct(core: &FakeCore) -> bool {
    let model = core.model();
    let model = model.lock().unwrap();
    model.scene.focused_node() == Some(LOOK) && model.scene.focused_surface().is_none()
}

/// rcr5-2 (front): stood back, a drop of the pane onto the face at the
/// front puts the window there as the give key does: the person steps in
/// and the face manager gives the window the keyboard, so the bar's keys
/// work and the told `Return Step in` types into no window stood back.
#[test]
fn a_drop_onto_the_front_face_stood_back_steps_in_with_the_window() {
    let (mut construct, _core, lines) = dragging_stood_back("drop-front");
    assert_eq!(mode(&construct), Mode::Navigate, "the cube stands back");
    let went = construct
        .drop_on_a_face("VIRTUAL-1", [640.0, 400.0])
        .unwrap();
    assert!(went, "the window goes to xp");
    let moved = called(&lines);
    assert!(
        moved.iter().any(|(m, p)| m == "window.move"
            && p["client"] == "xp"
            && p["params"]["output"] == "VIRTUAL-1"),
        "xp is asked to place it on the front face: {moved:?}"
    );
    assert_eq!(
        mode(&construct),
        Mode::Use,
        "the person steps into the face"
    );
    assert!(
        activated(&lines, "s1"),
        "the window has the keyboard in the face: {:?}",
        called(&lines)
    );
}

/// rcr5-2 (another face): stood back, a drop onto a face that is not at
/// the front gives the window to xp there, and the keyboard stays with
/// the construct: the cube stands back, no window that the person does
/// not see gets a key, and the window is not noted for the step in.
#[test]
fn a_drop_onto_another_face_stood_back_keeps_the_keyboard() {
    let (mut construct, core, lines) = dragging_stood_back("drop-side");
    let went = construct
        .drop_on_a_face("VIRTUAL-2", [640.0, 400.0])
        .unwrap();
    assert!(went, "the window goes to xp");
    assert_eq!(
        mode(&construct),
        Mode::Navigate,
        "the cube stays stood back"
    );
    assert!(
        !activated(&lines, "s1"),
        "the window on the side gets no keys"
    );
    assert!(
        keys_with_the_construct(&core),
        "the construct takes the keyboard back"
    );
    let noted = construct.cube.as_ref().unwrap().noted_focus();
    assert_ne!(noted.as_deref(), Some("s1"), "the step in gives it no keys");
}

/// The notices that the construct showed, by their summary.
fn told(lines: &Log) -> Vec<String> {
    called(lines)
        .into_iter()
        .filter(|(m, _)| m == chora_kit::notice::SHOW)
        .filter_map(|(_, p)| p["params"]["summary"].as_str().map(String::from))
        .collect()
}

/// rcr5-1 (drop): when xp cannot place the window on the face of the
/// drop, it is on an output that xp chose, maybe one the person does not
/// see: the construct tells it, stays stood back and keeps the keyboard.
#[test]
fn a_drop_that_xp_cannot_place_is_told_and_stays_stood_back() {
    let (mut construct, core, lines) = dragging_stood_back("drop-unplaced");
    core.refuse_next("methods.call");
    let went = construct
        .drop_on_a_face("VIRTUAL-1", [640.0, 400.0])
        .unwrap();
    assert!(went, "the window went to xp all the same");
    assert_eq!(
        called(&lines).first().map(|(m, _)| m.clone()).as_deref(),
        Some("window.move"),
        "the refused call is the place"
    );
    assert_eq!(
        mode(&construct),
        Mode::Navigate,
        "the cube stays stood back"
    );
    assert!(!activated(&lines, "s1"), "the window gets no keys");
    assert!(
        keys_with_the_construct(&core),
        "the construct keeps the keyboard"
    );
    assert_eq!(
        told(&lines).len(),
        1,
        "the person is told: {:?}",
        told(&lines)
    );
}

/// rcr5-1 (give key): the same for the give key of the face manager
/// stood back (`put_in_face`, co-79).
#[test]
fn a_put_that_xp_cannot_place_is_told_and_stays_stood_back() {
    let (mut construct, core, lines) = dragging_stood_back("put-unplaced");
    construct.moving = None;
    core.refuse_next("methods.call");
    construct.put_in_face("s1", "xp").unwrap();
    assert_eq!(
        called(&lines).first().map(|(m, _)| m.clone()).as_deref(),
        Some("window.move"),
        "the refused call is the place"
    );
    assert_eq!(
        mode(&construct),
        Mode::Navigate,
        "the cube stays stood back"
    );
    assert!(!activated(&lines, "s1"), "the window gets no keys");
    assert!(
        keys_with_the_construct(&core),
        "the construct keeps the keyboard"
    );
    assert_eq!(
        told(&lines).len(),
        1,
        "the person is told: {:?}",
        told(&lines)
    );
}

/// co-79 stays: a put that xp places steps in, and the window has the
/// keyboard there.
#[test]
fn a_put_that_xp_places_steps_in_with_the_window() {
    let (mut construct, _core, lines) = dragging_stood_back("put-placed");
    construct.moving = None;
    construct.put_in_face("s1", "xp").unwrap();
    assert_eq!(
        mode(&construct),
        Mode::Use,
        "the person steps into the face"
    );
    assert!(activated(&lines, "s1"), "the window has the keyboard");
    assert!(
        told(&lines).is_empty(),
        "nothing to tell: {:?}",
        told(&lines)
    );
}

/// rcdr1-1: with three faces, the window noted at the step back on face 0
/// is dropped onto face 2 from face 1 and the cube turns back to face 0:
/// `Return` (the step in) gives the keyboard to no window, for the dropped
/// one is on a face that the person does not see.
#[test]
fn a_drop_on_a_third_face_leaves_no_key_for_the_step_in_after_a_turn_back() {
    let (mut construct, _core, lines) = dragging_stood_back_on("drop-three", 3);
    construct
        .cube
        .as_mut()
        .unwrap()
        .note_focus(Some("s1".into()));
    construct.turn_cube(1, Some(0), false).unwrap();
    construct.step_cube().unwrap();
    assert_eq!(construct.cube.as_ref().unwrap().front_face(), (1, 3));
    let went = construct
        .drop_on_a_face("VIRTUAL-3", [640.0, 400.0])
        .unwrap();
    assert!(went, "the window goes to xp");
    construct.turn_cube(-1, Some(0), false).unwrap();
    construct.step_cube().unwrap();
    assert_eq!(construct.cube.as_ref().unwrap().front_face(), (0, 3));
    lines.lock().unwrap().clear();
    construct
        .mode_act(chora_kit::modes::Act::StepIn, crate::modes::By::Person)
        .unwrap();
    assert!(
        !activated(&lines, "s1"),
        "the window on face 2 gets no keys: {:?}",
        called(&lines)
    );
}
