//! Unit tests of the reach of C2 from a face (co-79): the hint bar of a
//! face names the pull key where it acts, and the close key while the
//! camera flies back out of a pane (rcc2-2), on the fake core of
//! `tests_construct`.

use chora_kit::testing::{focus_changed, has};

use super::close_tests::{closed, in_a_face_beside_a_pane};

/// co-79 (rc 2, MAJOR, demo C2 unreached four rounds): stepped into a
/// face, the hint bar names `Super Alt 4 Pull out` wherever the key acts,
/// as the cube stood back does (co-70): with the keyboard in a window of
/// the face, for the time of the hints and after it, beside the way out.
/// With the keyboard in the terminal pane in front of the face (the
/// demo's state after `Alt+Escape` out of the pane: mode use, holder
/// construct s1, no face window), the key gives nothing, so it is not
/// told (co-68); the bar names the way out, `Step back`.
#[test]
fn the_pull_key_is_named_in_a_face_where_it_acts() {
    let (mut construct, core, _lines) = in_a_face_beside_a_pane("reach-face", false);
    core.model().lock().unwrap().scene.set_focused(Some("s1"));
    chora_kit::runtime::Plugin::event(&mut construct, focus_changed(Some("s1"))).unwrap();
    for timed in [true, false] {
        construct.hints.until = timed.then(|| std::time::Instant::now() + crate::hints::FLOWN);
        construct.draw_hints().unwrap();
        let model = core.model();
        assert!(
            has(&model, "construct-hint-step-back"),
            "the face names its way out (timed {timed})"
        );
        assert!(
            !construct.pull_acts() && !has(&model, "construct-hint-pull-out"),
            "the keyboard is in the pane, the construct's own window: the key gives nothing, \
             it is not named (timed {timed})"
        );
    }
    core.model().lock().unwrap().scene.set_focused(Some("s2"));
    chora_kit::runtime::Plugin::event(&mut construct, focus_changed(Some("s2"))).unwrap();
    for timed in [true, false] {
        construct.hints.until = timed.then(|| std::time::Instant::now() + crate::hints::FLOWN);
        construct.draw_hints().unwrap();
        let model = core.model();
        assert!(construct.pull_acts(), "the key pulls the face window out");
        assert!(
            has(&model, "construct-hint-pull-out"),
            "the keyboard is in a window of the face: the key pulls it out, so the face \
             names it (timed {timed})"
        );
        assert!(
            has(&model, "construct-hint-step-back"),
            "the way out stays named (timed {timed})"
        );
    }
}

/// rcc2-2 (review r2 of co-close): flown into the pane from the face and
/// on the way back out (`Alt+Escape`, the glide not yet at its end),
/// `Super+x` closes nothing: the camera left the pane at the key, as the
/// face it lands in closes no pane (co-75), so the pane being left keeps
/// its program.
#[test]
fn super_x_while_flying_out_of_a_pane_into_a_face_closes_nothing() {
    let (mut construct, core, lines) = in_a_face_beside_a_pane("close-flying-out", false);
    construct.fly_in(0).unwrap();
    construct.flown = crate::camera::take_flight(&mut construct.glide);
    core.model().lock().unwrap().scene.set_focused(Some("s1"));
    chora_kit::runtime::Plugin::event(&mut construct, focus_changed(Some("s1"))).unwrap();
    construct.fly_out().unwrap();
    assert!(construct.glide.is_some(), "the camera glides back out");
    assert!(!construct.in_pane(), "the camera is no longer in the pane");
    lines.lock().unwrap().clear();
    chora_kit::actions::perform::Host::close(&mut construct).unwrap();
    assert_eq!(
        closed(&lines),
        Vec::<String>::new(),
        "Super+x on the way out of the pane closes nothing"
    );
}

/// co-79 (pane path): stood back with the keyboard in the window of a
/// pane, as after `Alt+Escape` out of a program started from Programs,
/// the give key of the face manager puts that window into the face, so
/// the bar names it beside the cube's hints; the pull key gives nothing
/// there and is not named (co-68). With the keyboard in a window of the
/// face it is the other way round.
#[test]
fn stood_back_out_of_a_pane_the_bar_names_the_way_into_the_face() {
    let (mut construct, core, _lines) = super::with_hint_bar("reach-put");
    construct.cube.as_mut().unwrap().program = "xp".into();
    construct.core_focus = Some("s1".into());
    construct.draw_hints().unwrap();
    assert_eq!(
        construct.put_window().as_deref(),
        Some("s1"),
        "the key puts the pane's window"
    );
    assert!(
        has(&core.model(), "construct-hint-put-in"),
        "the bar names the way into the face"
    );
    assert!(
        !construct.pull_acts() && !has(&core.model(), "construct-hint-pull-out"),
        "the pull key gives nothing there and is not named"
    );
    construct.core_focus = Some("s7".into());
    construct.draw_hints().unwrap();
    assert_eq!(
        construct.put_window(),
        None,
        "a window of the face is in the face already"
    );
    assert!(
        !has(&core.model(), "construct-hint-put-in"),
        "the way into the face is not named"
    );
    assert!(
        has(&core.model(), "construct-hint-pull-out"),
        "the pull key is named"
    );
}

/// co-79 r6: walking with no cube open, the window of a pane with the
/// keyboard (the state after `Alt+Escape` out of the pane of the Terminal
/// row) has `Super C Cube` named, the way to a face; with the keyboard in
/// no pane window it is not, as the walk names the cube nowhere else.
#[test]
fn stepped_out_of_a_pane_without_a_cube_the_walk_names_the_cube() {
    let (mut construct, core, _lines) = super::with_hint_bar("reach-walk");
    construct.cube = None;
    construct.hints.until = Some(std::time::Instant::now() + crate::hints::WALK);
    construct.core_focus = Some("s1".into());
    assert_eq!(construct.cube_window().as_deref(), Some("s1"));
    construct.draw_hints().unwrap();
    assert!(
        has(&core.model(), "construct-hint-cube"),
        "the walk names the way to a cube"
    );
    construct.core_focus = None;
    construct.draw_hints().unwrap();
    assert_eq!(construct.cube_window(), None);
    assert!(
        !has(&core.model(), "construct-hint-cube"),
        "with no pane window holding the keys the cube is not named"
    );
}
