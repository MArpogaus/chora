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

/// The window `s1` of the pane of `with_hint_bar`, which exists and sits in
/// the panes, so that the core can focus it and answers the query for the
/// window with the keyboard.
fn the_pane_window(construct: &mut crate::Construct, core: &chora_kit::testing::FakeCore) {
    core.model()
        .lock()
        .unwrap()
        .scene
        .add_surface("s1", chora_scene::SurfaceInfo::default());
    construct
        .rt
        .conn
        .apply(vec![chora_scene::Command::Attach {
            surface: "s1".into(),
            parent: crate::PANES.into(),
        }])
        .unwrap();
    let model = core.model();
    core.respond(move |method, params| {
        (method == "scene.query" && params["filter"]["focused"].as_bool() == Some(true)).then(
            || {
                let model = model.lock().unwrap();
                let focused = model.scene.focused_surface();
                serde_json::json!(
                    focused
                        .map(|id| serde_json::json!({ "id": id }))
                        .into_iter()
                        .collect::<Vec<_>>()
                )
            },
        )
    });
}

/// Every text of the key sheet on the screen, one string.
fn sheet_texts(core: &chora_kit::testing::FakeCore) -> String {
    let model = core.model();
    let model = model.lock().unwrap();
    (model.scene.nodes())
        .filter(|n| n.id.starts_with(crate::hud::HELP))
        .filter_map(|n| n.props.text.clone())
        .collect::<Vec<_>>()
        .join("|")
}

/// The sheet opened with the camera flown into the pane of `with_hint_bar`
/// and its window `s1` on the keyboard, by `open`; its texts.
fn sheet_in_a_pane(tag: &str, open: impl FnOnce(&mut crate::Construct)) -> String {
    let (mut construct, core, _lines) = super::with_hint_bar(tag);
    the_pane_window(&mut construct, &core);
    construct.fly_in(0).unwrap();
    construct.flown = crate::camera::take_flight(&mut construct.glide);
    assert!(construct.in_pane(), "the camera is in the pane");
    core.model().lock().unwrap().scene.set_focused(Some("s1"));
    chora_kit::runtime::Plugin::event(&mut construct, focus_changed(Some("s1"))).unwrap();
    open(&mut construct);
    assert!(construct.help, "the sheet is open");
    sheet_texts(&core)
}

/// What the sheet of a pane does not list: it types into the window.
fn assert_only_the_keys_that_work_in_a_window(sheet: &str) {
    for typed in [
        "Fly into the pane ahead",
        "Walk forward, back, sideways",
        "Step into the face",
        "Close the cube, stood back",
        "Show these keys, stood back",
        "Back out one step",
    ] {
        assert!(
            !sheet.contains(typed),
            "{typed:?} reaches the window, not the sheet: {sheet}"
        );
    }
    assert!(
        sheet.contains("Step back out of a pane"),
        "the way out of the pane is listed: {sheet}"
    );
}

/// rpc2-1 (rc 1b pass r2, MINOR): the key sheet opened while a pane's
/// window has the keyboard lists only the keys that work there: the
/// chords the construct holds for the window. A walk key (`Return Fly in`,
/// `W A S D`) or a key of the cube (`Return Step into the face`, `Esc`
/// close the cube) types into the window, so the sheet under "Only the keys
/// that work now are listed." does not list it; the way out of the pane does.
/// The hint `All keys` opens it with the camera still in the pane.
#[test]
fn the_key_sheet_in_a_pane_lists_only_the_keys_that_work_there() {
    let sheet = sheet_in_a_pane("sheet-pane", |c| c.toggle_help().unwrap());
    assert_only_the_keys_that_work_in_a_window(&sheet);
}

/// rpc2-1, by the key: `Super+F1` flies out of the pane first and then
/// shows the sheet, and the window gets the keyboard back when it closes,
/// so it lists the same keys.
#[test]
fn the_key_sheet_opened_by_its_key_in_a_pane_lists_the_same_keys() {
    let sheet = sheet_in_a_pane("sheet-pane-key", |c| {
        c.grabbed_key(&chora_kit::actions::System::Keys).unwrap();
    });
    assert_only_the_keys_that_work_in_a_window(&sheet);
}

/// rpc2-2 (rc 1b pass r2, MINOR): a cube stood back, the camera flown into
/// a pane, the key sheet opened and closed: the window of the pane has the
/// keyboard again, so the step back names `Super Alt 2 Into face` as it
/// does without the sheet (co-79).
#[test]
fn after_the_key_sheet_in_a_pane_the_step_back_names_the_way_into_the_face() {
    let (mut construct, core, _lines) = super::with_hint_bar("sheet-then-back");
    the_pane_window(&mut construct, &core);
    construct.cube.as_mut().unwrap().program = "xp".into();
    construct.fly_in(0).unwrap();
    construct.flown = crate::camera::take_flight(&mut construct.glide);
    assert!(construct.in_pane(), "the camera is in the pane");
    let focus = |construct: &mut crate::Construct, to: Option<&str>| {
        core.model().lock().unwrap().scene.set_focused(to);
        chora_kit::runtime::Plugin::event(construct, focus_changed(to)).unwrap();
    };
    focus(&mut construct, Some("s1"));
    construct.toggle_help().unwrap();
    focus(&mut construct, None);
    construct.toggle_help().unwrap();
    let given = core.model().lock().unwrap().focuses.last().cloned();
    assert_eq!(
        given,
        Some(Some("s1".into())),
        "closing the sheet gives the pane's window the keyboard back"
    );
    focus(&mut construct, Some("s1"));
    construct.step_out().unwrap();
    construct.glide = None;
    construct.draw_hints().unwrap();
    assert!(
        has(&core.model(), "construct-hint-put-in"),
        "the way into the face is named after the sheet, as without it"
    );
}
