//! Unit tests of the close key and of the way out of a pane (co-75,
//! co-76), on the fake core of `tests_construct`.

use std::time::Instant;

use chora_kit::{
    modes::Act,
    testing::{FakeCore, Log, focus_changed, has},
};
use chora_scene::{Command, NodeKind, PropsUpdate, ROOT};
use serde_json::Value;

use super::with_hint_bar;
use crate::{Construct, PANES};

/// A cube stepped into with the terminal pane `s1` of `with_hint_bar` at
/// home in front of it, and the window `s2` of the face manager `xp` on
/// the front face; `s2` has the keyboard when `focused`.
pub(super) fn in_a_face_beside_a_pane(tag: &str, focused: bool) -> (Construct, FakeCore, Log) {
    let (mut construct, core, lines) = with_hint_bar(tag);
    let host = construct.cube.as_mut().unwrap();
    assert!(host.act(Act::StepIn, Instant::now()), "the cube steps in");
    host.step = None;
    host.stand = 0.0;
    construct.step_asides();
    {
        let model = core.model();
        let mut model = model.lock().unwrap();
        for s in ["s1", "s2"] {
            model
                .scene
                .add_surface(s, chora_scene::SurfaceInfo::default());
        }
    }
    construct
        .rt
        .conn
        .apply(vec![Command::Attach {
            surface: "s1".into(),
            parent: PANES.into(),
        }])
        .unwrap();
    let mut xp = core.connect("xp");
    xp.apply(vec![
        Command::Create {
            id: "xp-desk-0".into(),
            kind: NodeKind::Group,
            parent: ROOT.into(),
            props: PropsUpdate::default(),
        },
        Command::Attach {
            surface: "s2".into(),
            parent: "xp-desk-0".into(),
        },
    ])
    .unwrap();
    drop(xp);
    if focused {
        core.model().lock().unwrap().scene.set_focused(Some("s2"));
        chora_kit::runtime::Plugin::event(&mut construct, focus_changed(Some("s2"))).unwrap();
    }
    lines.lock().unwrap().clear();
    (construct, core, lines)
}

/// The surfaces that the recorded requests asked to close.
pub(super) fn closed(lines: &Log) -> Vec<String> {
    (lines.lock().unwrap().iter())
        .filter(|line| line.starts_with("scene.apply "))
        .filter_map(|line| serde_json::from_str::<Value>(&line["scene.apply ".len()..]).ok())
        .flat_map(|params| params["commands"].as_array().cloned().unwrap_or_default())
        .filter_map(|c| serde_json::from_value::<Command>(c).ok())
        .filter_map(|c| match c {
            Command::Close { surface } => Some(surface),
            _ => None,
        })
        .collect()
}

/// co-75 (rc 1, BLOCK): stepped into a face, with a terminal pane at home
/// in front of the cube and out of view, `Super+x` closes the face window
/// that the person sees focused, never the pane: the pane, its program
/// and its work stay (rpc1-1).
#[test]
fn super_x_in_a_face_closes_the_focused_face_window_and_keeps_the_pane() {
    let (mut construct, _core, lines) = in_a_face_beside_a_pane("close-face", true);
    chora_kit::actions::perform::Host::close(&mut construct).unwrap();
    assert_eq!(
        closed(&lines),
        vec!["s2".to_owned()],
        "Super+x closes the focused face window, not the pane s1"
    );
    assert_eq!(construct.panes.len(), 1, "the pane stays");
}

/// co-75: stepped into a face with no window focused there, `Super+x`
/// closes nothing: never a window out of view.
#[test]
fn super_x_in_a_face_without_a_focused_window_closes_nothing() {
    let (mut construct, _core, lines) = in_a_face_beside_a_pane("close-none", false);
    chora_kit::actions::perform::Host::close(&mut construct).unwrap();
    assert_eq!(
        closed(&lines),
        Vec::<String>::new(),
        "nothing out of view closes"
    );
    assert_eq!(construct.panes.len(), 1, "the pane stays");
}

/// co-76 (rc 1, MAJOR): flown into a pane in the walk, with no cube, after
/// the 3 s of its hints something stays on the screen that leads to the
/// key list and out, as in a face: the resting hint bar with `Step back`
/// and `All keys`; a mouse user is never stranded in a pane (rpc1-2).
#[test]
fn a_pane_flown_into_in_the_walk_keeps_its_way_out_after_the_hints_went() {
    let (mut construct, core, _lines) = with_hint_bar("pane-rest");
    construct.cube = None;
    construct.fly_in(0).unwrap();
    assert!(construct.in_pane(), "the camera flies into the pane");
    construct.flown = crate::camera::take_flight(&mut construct.glide);
    assert!(construct.flown.is_some(), "the flight lands in the pane");
    construct.draw_hints().unwrap();
    construct.hints.until = None;
    construct.draw_hints().unwrap();
    let model = core.model();
    assert!(
        has(&model, "construct-hint-keys"),
        "a hint leads to the key list after the 3 s"
    );
    assert!(
        has(&model, "construct-hint-step-back"),
        "a hint leads out of the pane after the 3 s"
    );
}

/// rcc1-1 (review r1 of co-close, MAJOR): flown into a pane from a face,
/// with the cube still in use mode, `Super+x` closes that pane's window:
/// it fills the screen with the keyboard, so it is the one the person
/// sees focused.
#[test]
fn super_x_flown_into_a_pane_from_a_face_closes_that_pane() {
    let (mut construct, core, lines) = in_a_face_beside_a_pane("close-flown", false);
    construct.fly_in(0).unwrap();
    construct.flown = crate::camera::take_flight(&mut construct.glide);
    assert!(construct.in_pane(), "the camera is in the pane");
    core.model().lock().unwrap().scene.set_focused(Some("s1"));
    chora_kit::runtime::Plugin::event(&mut construct, focus_changed(Some("s1"))).unwrap();
    lines.lock().unwrap().clear();
    chora_kit::actions::perform::Host::close(&mut construct).unwrap();
    assert_eq!(
        closed(&lines),
        vec!["s1".to_owned()],
        "Super+x closes the pane flown into"
    );
}
