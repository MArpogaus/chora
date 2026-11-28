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

/// co-79 r7: right after `Alt+Escape` out of a pane with no cube open, the
/// walk names `Super Alt 2 Into face`, the one key that puts that pane's
/// window into a face; in a pane, flown in (from a face or by its `Fly
/// in` click), the bar names the same key, beside `Step back`.
#[test]
fn out_of_a_pane_and_in_one_the_bar_names_the_key_into_a_face() {
    let (mut construct, core, _lines) = super::with_hint_bar("reach-r7-walk");
    construct.cube = None;
    construct.hints.until = Some(std::time::Instant::now() + crate::hints::WALK);
    construct.core_focus = Some("s1".into());
    construct.draw_hints().unwrap();
    assert!(
        has(&core.model(), "construct-hint-put-in"),
        "stepped out of the pane with no cube, the walk names the key into a face"
    );
    let (mut construct, core, _lines) = super::with_hint_bar("reach-r7-pane");
    construct.cube.as_mut().unwrap().program = "xp".into();
    construct.fly_in(0).unwrap();
    construct.flown = crate::camera::take_flight(&mut construct.glide);
    construct.core_focus = Some("s1".into());
    construct.hints.until = Some(std::time::Instant::now() + crate::hints::FLOWN);
    construct.draw_hints().unwrap();
    assert!(
        has(&core.model(), "construct-hint-step-back"),
        "in the pane the bar names the way out"
    );
    assert!(
        has(&core.model(), "construct-hint-put-in"),
        "in the pane the bar names the key into a face"
    );
}

/// co-79 r7: the key into a face waits for a face manager that cannot
/// place the window yet (a cube just opened by the key, whose xp exports
/// no `window.move` yet), and the construct looks again soon; the wait
/// ends unput once the person stepped into a face.
#[test]
fn the_key_into_a_face_waits_for_the_face_manager_and_ends_at_a_step_in() {
    let (mut construct, _core, _lines) = super::with_hint_bar("reach-r7-wait");
    construct.cube.as_mut().unwrap().program = "xp".into();
    construct.core_focus = Some("s1".into());
    assert_eq!(construct.pane_to_face().as_deref(), Some("s1"));
    construct.put_pane("s1").unwrap();
    assert!(
        construct.cube.as_ref().unwrap().put_later.is_some(),
        "no window.move is exported: the put waits"
    );
    let soon = std::time::Instant::now() + std::time::Duration::from_secs(1);
    assert!(
        construct.cube_deadline().is_some_and(|at| at <= soon),
        "the construct looks again soon"
    );
    let host = construct.cube.as_mut().unwrap();
    assert!(host.act(chora_kit::modes::Act::StepIn, std::time::Instant::now()));
    host.put_later.as_mut().unwrap().at = std::time::Instant::now();
    construct.put_when_ready().unwrap();
    assert!(
        construct.cube.as_ref().unwrap().put_later.is_none(),
        "stepped into a face, the put ends"
    );
}

/// rcrr7-1 (review r7, MAJOR): with the face manager running already
/// and no cube open (xp as a client the construct did not start, or in a
/// manager pane), the give key from a pane gives the window to it, as
/// before r7, and the bar tells that give: the cube that `Into face` and
/// `Super C Cube` open cannot start beside that xp, so neither is told.
#[test]
fn with_the_face_manager_running_the_key_from_a_pane_gives_to_it() {
    let xp = serde_json::json!([{"name": "xp", "methods": []}]);
    let (mut construct, core, lines) = super::with_hint_bar_beside("reach-r8-runs", xp);
    the_pane_window(&mut construct, &core);
    construct.book.keep("s1");
    construct.cube = None;
    let connected = chora_scene::Event::ClientConnected { name: "xp".into() };
    chora_kit::runtime::Plugin::event(&mut construct, connected).unwrap();
    core.model().lock().unwrap().scene.set_focused(Some("s1"));
    construct.core_focus = Some("s1".into());
    construct.hints.until = Some(std::time::Instant::now() + crate::hints::WALK);
    construct.draw_hints().unwrap();
    let model = core.model();
    assert!(
        !has(&model, "construct-hint-put-in") && !has(&model, "construct-hint-cube"),
        "with xp running beside, no cube can open: no key into a face, no cube is told"
    );
    assert!(
        has(&model, "construct-hint-give-to"),
        "the bar tells what the key does there: it gives the window to XP"
    );
    lines.lock().unwrap().clear();
    construct.give_key("xp").unwrap();
    let said = lines.lock().unwrap().join("\n");
    assert!(!said.contains("output.add"), "no cube opens: {said}");
    assert!(
        said.contains(r#""id":"s1","to":"xp""#),
        "the window of the pane goes to the running xp: {said}"
    );
}

/// rcrr7-2 (review r7): the put that waits ends when the person moved
/// on: flown into another pane, or another window noted at the cube
/// (chosen there), so that no later put glides the camera out of where
/// the person went.
#[test]
fn the_wait_ends_when_the_person_moves_on() {
    for moves in ["fly", "note"] {
        let (mut construct, _core, _lines) = super::with_hint_bar(&format!("reach-r8-on-{moves}"));
        construct.cube.as_mut().unwrap().program = "xp".into();
        let size = construct.panes[0].size;
        construct.panes.push(crate::Pane {
            name: "notes".into(),
            source: "s2".into(),
            content: crate::Content::Window { command: None },
            at: [400.0, 700.0],
            yaw: 0.0,
            size,
            taken: false,
            attached: true,
            placed: false,
        });
        construct.core_focus = Some("s1".into());
        construct.put_pane("s1").unwrap();
        assert!(
            waits(&construct),
            "no window.move is exported: the put waits"
        );
        if moves == "fly" {
            construct.fly_in(1).unwrap();
            construct.flown = crate::camera::take_flight(&mut construct.glide);
            assert!(construct.in_pane(), "flown into the other pane");
        } else {
            let host = construct.cube.as_mut().unwrap();
            host.note_focus(Some("s7".into()));
        }
        due(&mut construct);
        construct.put_when_ready().unwrap();
        assert!(!waits(&construct), "moved on ({moves}): the put ends");
    }
}

/// rcrr7-3 (review r7): a second Super+Alt+2 while the cube that the
/// key opened still starts keeps the same put: no give to a face manager
/// that cannot place the window yet, and no refusal while the cube
/// visibly opens.
#[test]
fn a_second_key_while_the_cube_starts_keeps_the_same_put() {
    let (mut construct, core, lines) = super::with_hint_bar("reach-r8-again");
    (core.model().lock().unwrap().scene).add_surface("s1", chora_scene::SurfaceInfo::default());
    let attach = chora_scene::Command::Attach {
        surface: "s1".into(),
        parent: crate::PANES.into(),
    };
    construct.rt.conn.apply(vec![attach]).unwrap();
    construct.book.keep("s1");
    core.model().lock().unwrap().scene.set_focused(Some("s1"));
    construct.cube.as_mut().unwrap().program = "xp".into();
    construct.core_focus = Some("s1".into());
    construct.put_pane("s1").unwrap();
    let host = construct.cube.as_mut().unwrap();
    host.health.fill(crate::faces::Health::Starting);
    assert_eq!(waiting(&construct).as_deref(), Some("s1"));
    lines.lock().unwrap().clear();
    construct.give_key("xp").unwrap();
    let said = lines.lock().unwrap().join("\n");
    assert_eq!(
        waiting(&construct).as_deref(),
        Some("s1"),
        "the second key keeps the put: {said}"
    );
    assert!(
        !said.contains(r#""to":"xp""#) && !said.contains("notify.show"),
        "no give to an xp that still starts, nothing refused: {said}"
    );
}

/// rcrr7-4 (review r7): every end of a put that waits without the put
/// tells the person why in a notice, after the told key: the 20 s, a
/// move on (a step into a face, a fly into a pane, another window noted
/// at the cube), the window gone, the face at the front failed, the cube
/// closed.
#[test]
fn every_end_of_the_wait_without_the_put_is_told() {
    let failure = "the manager stopped after 2 s: it needs foot 1.26 or later.";
    let cases = [
        ("late", "XP did not place the window in time."),
        (
            "step-in",
            "XP did not place the window: you stepped into a face.",
        ),
        ("pane", "XP did not place the window: you flew into a pane."),
        (
            "note",
            "XP did not place the window: another window has the keyboard.",
        ),
        ("gone", "XP did not place the window: the window closed."),
        (
            "failed",
            "XP did not place the window: the manager stopped after 2 s: it needs foot 1.26 or later.",
        ),
        ("closed", "XP did not place the window: the cube closed."),
    ];
    for (end, told) in cases {
        let (mut construct, _core, lines) = super::with_hint_bar(&format!("reach-r8-told-{end}"));
        construct.cube.as_mut().unwrap().program = "xp".into();
        construct.core_focus = Some("s1".into());
        construct.put_pane("s1").unwrap();
        assert!(waits(&construct), "{end}: the put waits");
        let now = std::time::Instant::now();
        let host = construct.cube.as_mut().unwrap();
        match end {
            "late" => host.put_later.as_mut().unwrap().until = now,
            "step-in" => assert!(host.act(chora_kit::modes::Act::StepIn, now)),
            "note" => host.note_focus(Some("s7".into())),
            "failed" => host
                .health
                .fill(crate::faces::Health::Failed(failure.into())),
            "pane" => {
                construct.fly_in(0).unwrap();
                construct.flown = crate::camera::take_flight(&mut construct.glide);
            }
            "gone" => construct.panes[0].attached = false,
            _ => {}
        }
        lines.lock().unwrap().clear();
        due(&mut construct);
        if end == "closed" {
            construct.close_cube().unwrap();
        } else {
            construct.put_when_ready().unwrap();
        }
        assert!(!waits(&construct), "{end}: the put ends");
        let said = lines.lock().unwrap().join("\n");
        assert!(
            said.contains(&format!(r#""summary":"{told}""#)),
            "{end}: the end is told as {told:?}: {said}"
        );
    }
}

/// rcrr7-5 (review r7): the put that waits never waits on the face
/// manager: with an xp that exports `window.move` but answers a call only
/// after a second (as one that starts), the key and each look of the put
/// return at once, so the frame loop goes on.
#[test]
fn the_wait_never_waits_on_the_face_manager() {
    let (mut construct, core, lines) = super::with_hint_bar("reach-r8-slow");
    core.respond(|method, params| match method {
        "methods.list" => Some(serde_json::json!([{"name": "window.move", "client": "xp"}])),
        "methods.call" if params["method"] == "state.get" => {
            std::thread::sleep(std::time::Duration::from_secs(1));
            Some(serde_json::json!({}))
        }
        _ => None,
    });
    construct.cube.as_mut().unwrap().program = "xp".into();
    construct.core_focus = Some("s1".into());
    construct.socket = Some(core.socket().to_owned());
    (core.model().lock().unwrap().scene).add_surface("s1", chora_scene::SurfaceInfo::default());
    let attach = chora_scene::Command::Attach {
        surface: "s1".into(),
        parent: crate::PANES.into(),
    };
    construct.rt.conn.apply(vec![attach]).unwrap();
    construct.book.keep("s1");
    let mut xp = core.connect("xp");
    let desk = chora_scene::Command::Create {
        id: "xp-desk-0".into(),
        kind: chora_scene::NodeKind::Group,
        parent: chora_scene::ROOT.into(),
        props: chora_scene::PropsUpdate {
            claim: Some(Some([0.0, 0.0, 1280.0, 800.0])),
            ..chora_scene::PropsUpdate::default()
        },
    };
    xp.apply(vec![desk]).unwrap();
    let began = std::time::Instant::now();
    construct.put_pane("s1").unwrap();
    for _ in 0..3 {
        due(&mut construct);
        construct.put_when_ready().unwrap();
    }
    let took = began.elapsed();
    assert!(
        took < std::time::Duration::from_millis(100),
        "the key and three looks waited {took:?} on xp"
    );
    assert!(waits(&construct), "xp has not answered: the put waits");
    let until = began + std::time::Duration::from_secs(4);
    while waits(&construct) && std::time::Instant::now() < until {
        std::thread::sleep(std::time::Duration::from_millis(50));
        due(&mut construct);
        construct.put_when_ready().unwrap();
    }
    let said = lines.lock().unwrap().join("\n");
    assert!(
        !waits(&construct) && said.contains("window.move"),
        "once xp answered, a later look puts the window: {said}"
    );
}

/// The window of the put that waits.
fn waiting(construct: &crate::Construct) -> Option<String> {
    let host = construct.cube.as_ref()?;
    host.put_later.as_ref().map(|put| put.surface.clone())
}

/// Whether a put waits (`put_pane`).
fn waits(construct: &crate::Construct) -> bool {
    (construct.cube.as_ref()).is_some_and(|host| host.put_later.is_some())
}

/// The put that waits looks again now.
fn due(construct: &mut crate::Construct) {
    if let Some(put) = (construct.cube.as_mut()).and_then(|host| host.put_later.as_mut()) {
        put.at = std::time::Instant::now();
    }
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
