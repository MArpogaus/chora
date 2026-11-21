//! Unit tests of the construct plugin.

use super::*;
use crate::{
    adopt::*,
    camera::*,
    input::{keys_walk_due, look_pointer, pressed_row, walk_key},
    list::row_of,
};

/// `x` and Return act on a pane that shows, not on one beyond the
/// depth of the view (review 2.5).
#[test]
fn no_pane_ahead_that_is_out_of_sight() {
    let pane = |z| Pane {
        name: "foot".into(),
        source: "s1".into(),
        content: Content::Window { command: None },
        at: [0.0, z],
        yaw: 0.0,
        size: [640.0, 400.0],
        taken: false,
        attached: false,
        placed: false,
    };
    let lens = space::Lens::new(1920.0, 1080.0);
    let pose = Pose::default();
    let panes = [pane(100_000.0)];
    assert_eq!(ahead(&panes, &lens, &pose, 6000.0), None);
    let panes = [pane(100_000.0), pane(2000.0)];
    assert_eq!(ahead(&panes, &lens, &pose, 6000.0), Some(1));
}

/// A window handed over keeps its own size, clamped to the view. A
/// window that finds no free spot in the view must still get its
/// pane (forced), or it is lost to the list (the pane leak of the
/// handover matrix).
#[test]
fn a_handed_over_window_keeps_its_size_clamped_to_the_view() {
    let lens = space::Lens::new(1280.0, 800.0);
    let small = handed_over(Some([640.0, 400.0]), &lens);
    assert_eq!(small.size, Some([640.0, 400.0]));
    let big = handed_over(Some([3000.0, 800.0]), &lens);
    assert_eq!(big.size, Some([1280.0, 800.0]));
    let unknown = handed_over(None, &lens);
    assert_eq!(unknown.size, Some([1280.0, 800.0]));
}

/// The window the camera is in, or on its way into, is the one a
/// give, a drag or a handover has to reckon with; a plain glide is
/// no window.
#[test]
fn the_window_flown_into_includes_the_glide_into_it() {
    let flown = |s: &str| Flown {
        surface: s.into(),
        back: Pose::default(),
        attached: true,
        plugin: None,
    };
    let glide = |into| Glide {
        from: Pose::default(),
        to: Pose::default(),
        start: Instant::now(),
        ms: 380.0,
        into,
        smooth: false,
    };
    assert_eq!(window_flown_into(None, None), None);
    assert_eq!(window_flown_into(Some(&flown("s1")), None), Some("s1"));
    assert_eq!(window_flown_into(None, Some(&glide(None))), None);
    let gliding = glide(Some(flown("s2")));
    assert_eq!(window_flown_into(None, Some(&gliding)), Some("s2"));
}

/// A window that is handed over takes the camera only when the user
/// is not in a window; in one, the pane waits in the space.
#[test]
fn a_handover_does_not_pull_the_user_out_of_a_window() {
    assert!(takes_the_camera(None));
    assert!(!takes_the_camera(Some("s1")));
}

/// A pane keeps its pose in the file, and a construct file gives it
/// back unchanged; a window that the construct did not start has no
/// command and stays out.
#[test]
fn a_pane_saves_its_pose() {
    let pane = |content| Pane {
        name: "foot".into(),
        source: "s1".into(),
        content,
        at: [-750.125, 2300.5],
        yaw: 0.25,
        size: [1200.0, 768.0],
        taken: false,
        attached: false,
        placed: false,
    };
    let started = pane(Content::Window {
        command: Some("foot".into()),
    });
    let saved = started.file().unwrap();
    assert_eq!(saved.kind, Kind::Program);
    assert_eq!(
        (saved.at, saved.yaw, saved.size),
        (Some(started.at), Some(started.yaw), Some(started.size))
    );
    let text = serde_saphyr::to_string(&file::File {
        panes: vec![saved.clone()],
        ..file::File::default()
    })
    .unwrap();
    assert_eq!(file::parse(&text).unwrap().panes, [saved]);
    assert!(pane(Content::Window { command: None }).file().is_none());
    let plugin = pane(Content::Plugin {
        plugin: "mc".into(),
        scene: "mission".into(),
    });
    assert_eq!(plugin.node(), "construct-plugin-foot");
    assert_eq!(plugin.file().unwrap().scene.as_deref(), Some("mission"));
}

/// A pane keeps its yaw while the camera walks and looks around
/// (live bug: the panes turned with the camera), and a move turns
/// its face to the camera.
#[test]
fn a_walk_keeps_the_yaw_and_a_move_refaces() {
    let mut pane = Pane {
        name: "foot".into(),
        source: "s1".into(),
        content: Content::Window { command: None },
        at: [0.0, 0.0],
        yaw: 0.0,
        size: [640.0, 400.0],
        taken: false,
        attached: false,
        placed: false,
    };
    pane.put([0.0, 2000.0], [0.0, 0.0]);
    assert!(pane.yaw.abs() < 1e-12);
    let mut pose = Pose::default();
    for _ in 0..20 {
        [pose.x, pose.z] = space::leash([pose.x, pose.z], [150.0, 40.0], 6000.0);
        turn(&mut pose, None, 30.0, 4.0);
    }
    assert!(pane.yaw.abs() < 1e-12, "the walk turned the pane");
    pane.put([1000.0, 2000.0], [pose.x, pose.z]);
    let face = space::facing([pose.x, pose.z], [1000.0, 2000.0]);
    assert!((pane.yaw - face).abs() < 1e-12);
    assert!(face.abs() > 0.1, "the move turned it: {face}");
}

/// A pane that the person put, by a drag, a pull out of a face or a
/// move, is marked, so that a cube that stands back does not hide it.
#[test]
fn a_move_marks_the_pane_as_placed() {
    let mut pane = Pane {
        name: "foot".into(),
        source: "s1".into(),
        content: Content::Window { command: None },
        at: [0.0, 2000.0],
        yaw: 0.0,
        size: [640.0, 400.0],
        taken: false,
        attached: false,
        placed: false,
    };
    assert!(!pane.placed);
    pane.put([300.0, 780.0], [0.0, 0.0]);
    assert!(pane.placed);
}

/// The shadow uniform gives the lift and the yaw back as fog.frag
/// reads them.
#[test]
fn the_shadow_packs_the_lift_and_the_yaw() {
    for (lift, yaw) in [(0.0, 0.0), (412.7, 3.0), (999.0, -3.1), (5.0, 7.0)] {
        let w = shadow_w(lift, yaw);
        assert!((w.floor() - lift.floor()).abs() < f64::EPSILON);
        let back = (w.fract() / 0.99 - 0.5) * std::f64::consts::TAU;
        let diff = (back - yaw).rem_euclid(std::f64::consts::TAU);
        assert!(
            diff.min(std::f64::consts::TAU - diff) < 1e-9,
            "{lift} {yaw} {back}"
        );
    }
}

/// A drag grabs the world (co-pane r2, rv-6): the scene follows the hand,
/// as the cube does (docs/cube-modes.md). A drag to the right moves the
/// pane ahead to the right on the screen, a drag down moves it down.
#[test]
fn a_drag_takes_the_scene_along_with_the_hand() {
    let lens = space::Lens::new(1280.0, 800.0);
    let (at, size) = ([0.0, 2000.0], [640.0, 400.0]);
    let middle = |pose: &Pose| {
        let m = space::pane_matrix(&lens, pose, at, size, 0.0);
        space::project(&m, 320.0, 200.0)
    };
    let rest = middle(&Pose::default());
    let mut pose = Pose::default();
    turn(&mut pose, None, 40.0, 0.0);
    assert!(middle(&pose)[0] > rest[0] + 1.0, "right with the hand");
    let mut pose = Pose::default();
    turn(&mut pose, None, -40.0, 0.0);
    assert!(middle(&pose)[0] < rest[0] - 1.0, "left with the hand");
    let mut pose = Pose::default();
    turn(&mut pose, None, 0.0, 40.0);
    assert!(middle(&pose)[1] > rest[1] + 1.0, "down with the hand");
    let mut pose = Pose::default();
    turn(&mut pose, None, 0.0, -40.0);
    assert!(middle(&pose)[1] < rest[1] - 1.0, "up with the hand");
}

/// The direction of the look is written in the same words in F5 and in
/// the docs of the walk (rw-2): the scene follows the hand, and a press
/// on the window of a pane starts no look.
#[test]
fn the_docs_say_the_direction_of_the_look_as_f5_does() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let words = |path: &str| {
        let text = std::fs::read_to_string(root.join(path)).unwrap();
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    };
    let sentences = [
        "The drag grabs the world: the scene follows the hand, so a drag to \
         the right turns the view to the left and a drag down tilts it up",
        "A press that lands on the window of a pane gives that window the \
         keyboard and starts no look",
    ];
    for path in [
        "plans/construct-requirements.md",
        "docs/cube-modes.md",
        "docs/configuration.md",
    ] {
        let text = words(path);
        for sentence in sentences {
            assert!(text.contains(sentence), "{path}: {sentence}");
        }
    }
}

/// A walk during a drag stays: the drag turns only (live bug: the
/// camera jumped back to the press when the key `w` was released).
#[test]
fn a_walk_during_a_drag_keeps_its_place() {
    let mut pose = Pose::default();
    turn(&mut pose, None, 50.0, 0.0);
    let mut to = pose;
    to.x += 90.0 * pose.yaw.sin();
    to.z += 90.0 * pose.yaw.cos();
    let mut glide = Glide {
        from: pose,
        to,
        start: Instant::now(),
        ms: STEP_MS,
        into: None,
        smooth: false,
    };
    turn(&mut pose, Some(&mut glide), 10.0, -5.0);
    pose = glide.from.lerp(&glide.to, 1.0);
    turn(&mut pose, None, 10.0, 0.0);
    assert!(
        (pose.x - to.x).abs() < 1e-9 && (pose.z - to.z).abs() < 1e-9,
        "{pose:?}"
    );
    assert!((pose.yaw + 70.0 * LOOK_RATE).abs() < 1e-9);
    assert!((pose.pitch + 5.0 * LOOK_RATE).abs() < 1e-9);
}

/// The offsets of `pane.move` add up, as the camera looks.
#[test]
fn pane_moves_add_up() {
    let near = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9;
    let start = [3150.0, 2000.0];
    let ahead = Pose::default().forward();
    let there = moved(start, ahead, 1800.0, 0.0);
    assert!(
        near(there, [4950.0, 2000.0]),
        "right is +x at yaw 0: {there:?}"
    );
    assert!(near(moved(there, ahead, -1800.0, 0.0), start));
    let twice = moved(moved(start, ahead, 900.0, 100.0), ahead, 900.0, 100.0);
    assert!(near(twice, moved(start, ahead, 1800.0, 200.0)));
    let turned = Pose {
        yaw: std::f64::consts::FRAC_PI_2,
        ..Pose::default()
    };
    assert!(near(
        moved([0.0, 0.0], turned.forward(), 100.0, 0.0),
        [0.0, -100.0]
    ));
}

/// The launcher's token on the window of another process is the
/// program of the launcher, as fuzzel passes it on.
#[test]
fn a_window_of_another_process_is_not_the_launcher() {
    assert!(launchers_own(Some(7), Some(7)));
    assert!(!launchers_own(Some(8), Some(7)));
    assert!(launchers_own(None, Some(7)));
    assert!(launchers_own(Some(8), None));
}

/// A shell that does not exec its command leaves the window with the
/// child of the launcher's pid; a program of the launcher is still
/// another process.
#[test]
fn the_window_of_the_child_of_a_launcher_shell_is_the_launcher() {
    let table = |pid| match pid {
        7 => Some((1, "sh".to_owned())),
        8 => Some((7, "foot".to_owned())),
        9 => Some((8, "foot".to_owned())),
        10 => Some((1, "fuzzel".to_owned())),
        11 => Some((10, "app".to_owned())),
        _ => None,
    };
    assert!(launchers_own_by(Some(8), Some(7), table));
    assert!(!launchers_own_by(Some(9), Some(7), table));
    assert!(!launchers_own_by(Some(11), Some(10), table));
    assert!(!launchers_own_by(Some(8), Some(99), table));
}

/// A program that waits longer than [`LAUNCHED`] goes, and no program
/// that waits is saved; a plugin pane waits and saves.
#[test]
fn a_waiting_program_expires_and_is_not_saved() {
    let pane = |kind| PaneFile {
        kind,
        command: Some("xterm".into()),
        plugin: None,
        scene: None,
        manager: None,
        at: None,
        yaw: None,
        size: None,
        placed: false,
    };
    let old = |kind| Waiting {
        since: Instant::now().checked_sub(LAUNCHED).unwrap(),
        ..Waiting::new(None, pane(kind))
    };
    assert!(!Waiting::new(None, pane(Kind::Program)).stale());
    assert!(old(Kind::Program).stale());
    assert!(!old(Kind::Plugin).stale());
    assert!(
        !old(Kind::Manager).stale(),
        "a manager pane waits for its name"
    );
    assert!(Waiting::new(None, pane(Kind::Program)).file().is_none());
    assert!(old(Kind::Plugin).file().is_some());
}

/// A flight of `camera.glide` starts and ends slowly and passes its
/// middle at half of its time.
#[test]
fn a_camera_glide_is_slow_at_both_ends() {
    assert!(ease_in_out(0.0).abs() < 1e-12 && (ease_in_out(1.0) - 1.0).abs() < 1e-12);
    assert!((ease_in_out(0.5) - 0.5).abs() < 1e-12);
    assert!(ease_in_out(0.1) < 0.01 && ease_in_out(0.9) > 0.99);
    let steps: Vec<f64> = (0..=20).map(|i| ease_in_out(f64::from(i) / 20.0)).collect();
    assert!(steps.windows(2).all(|w| w[1] >= w[0]));
}

/// The notice of a typed key that does nothing shows at most once in 10
/// seconds, so that typed text gives one notice, not one per key.
#[test]
fn the_notice_of_the_walk_keys_waits_ten_seconds() {
    let now = Instant::now();
    assert!(keys_walk_due(None, now));
    assert!(!keys_walk_due(Some(now), now + Duration::from_secs(9)));
    assert!(keys_walk_due(Some(now), now + Duration::from_secs(10)));
}

/// A manager pane of a file names a manager that the setting `managers`
/// lacks: it goes, with the reason, and does not wait.
#[test]
fn a_manager_pane_without_its_manager_goes_with_a_reason() {
    let managers = std::collections::BTreeMap::from([("xp".to_owned(), "chora-xp".to_owned())]);
    assert_eq!(wm::unknown_manager(&managers, "xp"), None);
    let reason = wm::unknown_manager(&managers, "twm").expect("a reason");
    assert!(reason.contains("twm"), "{reason}");
}

/// The window list names each window, and marks only a window that is
/// not in view; it never names an owner or an internal place.
#[test]
fn the_window_list_names_the_windows_in_plain_words() {
    let entry = |pane| Entry {
        name: "foot-2".into(),
        surface: "s2".into(),
        owner: Some("construct".into()),
        pane,
        size: None,
    };
    let shown = row_of(entry("shown"));
    assert_eq!((shown.label.as_str(), shown.note.as_str()), ("foot-2", ""));
    for pane in ["behind", "far", "aside", "covered", "none"] {
        let row = row_of(entry(pane));
        assert_eq!(row.label, "foot-2");
        assert_eq!(row.note, "not in view", "{pane}");
    }
}

/// The floor of both themes is marble, and a mirror image blurs more far
/// from the pane than at the contact line (fog.frag, mirror.frag).
#[test]
fn the_floor_is_marble_and_its_mirror_images_are_rough() {
    for theme in [config::Theme::Light, config::Theme::Dark] {
        let [veins, bumps, scratches, size] = theme.palette().marble;
        assert!(veins > 0.0 && bumps > 0.0 && scratches > 0.0, "{theme:?}");
        assert!(size >= 200.0, "{theme:?}: the veins are long");
    }
    let [near, far, stretch, _] = panes::ROUGH;
    assert!(near < far && stretch >= 1.0);
}

/// The floor renders at half the output on a GPU and at a quarter on a
/// software renderer.
#[test]
fn the_floor_renders_finer_on_a_gpu() {
    assert!((fog_scale(false) - 2.0).abs() < f64::EPSILON);
    assert!((fog_scale(true) - 4.0).abs() < f64::EPSILON);
}

/// The frame of a pane is the buffer of its window: it takes the size of
/// every new buffer, and no size of a window that has none yet or that
/// changed by less than half a pixel.
#[test]
fn the_frame_of_a_pane_takes_the_size_of_the_buffer() {
    let pane = [1280.0, 720.0];
    assert_eq!(follows_buffer(pane, [700.0, 500.0]), Some([700.0, 500.0]));
    assert_eq!(follows_buffer(pane, [1276.0, 704.0]), Some([1276.0, 704.0]));
    assert_eq!(follows_buffer(pane, [0.0, 0.0]), None, "no buffer yet");
    assert_eq!(follows_buffer(pane, [1280.0, 0.0]), None);
    assert_eq!(
        follows_buffer(pane, [1280.2, 719.8]),
        None,
        "under half a pixel"
    );
}

/// Taking the flight into a pane leaves a plain glide of the walk
/// alone.
#[test]
fn taking_the_flight_into_a_pane_keeps_a_plain_glide() {
    let glide = |into| Glide {
        from: Pose::default(),
        to: Pose::default(),
        start: std::time::Instant::now(),
        ms: 380.0,
        into,
        smooth: false,
    };
    let flown = Flown {
        surface: "s1".into(),
        back: Pose::default(),
        attached: false,
        plugin: None,
    };
    let mut plain = Some(glide(None));
    assert!(take_flight(&mut plain).is_none());
    assert!(plain.is_some(), "a plain glide stays");
    let mut flying = Some(glide(Some(flown)));
    assert_eq!(
        take_flight(&mut flying).map(|f| f.surface),
        Some("s1".into())
    );
    assert!(flying.is_none());
}

/// A press on the window list was taken for any button and from
/// `input.inject` (review plugins r1, CON-B-10): only a left press of the
/// person chooses a row.
#[test]
fn only_a_left_press_of_the_person_chooses_a_row() {
    let press = |button, injected| Event::PointerPressed {
        node: format!("{LIST_ROW}3"),
        button,
        position: [0.0; 2],
        local: [0.0; 2],
        modifiers: Vec::new(),
        injected,
    };
    assert_eq!(pressed_row(&press(BTN_LEFT, false), false), Some(3));
    assert_eq!(
        pressed_row(&press(273, false), false),
        None,
        "the right button"
    );
    assert_eq!(pressed_row(&press(BTN_LEFT, true), false), None, "injected");
    assert_eq!(pressed_row(&press(BTN_LEFT, true), true), Some(3), "a test");
}

/// A key of `input.inject` chose a row of the window list, flew to a
/// window or gave one away without a person (review security a r1,
/// CON-B-10): the list acts on a key of the person, or of a test session.
#[test]
fn an_injected_key_does_not_drive_the_window_list() {
    let key = |name: &str, injected| Event::NodeKey {
        node: LOOK.into(),
        key: name.into(),
        code: 28,
        state: PressState::Pressed,
        modifiers: Vec::new(),
        text: None,
        injected,
    };
    for name in ["Return", "Down", "Tab", "Escape"] {
        assert!(
            walk_key(&key(name, false), false).is_some(),
            "{name} of the person"
        );
        assert!(
            walk_key(&key(name, true), false).is_none(),
            "{name} injected"
        );
        assert!(
            walk_key(&key(name, true), true).is_some(),
            "{name} in a test"
        );
    }
}

/// A key of `input.inject` flew to a window, closed a pane, typed the
/// terminal key or turned the scene without a person (review security a
/// r2, MAJOR 1): the walk acts on a key of the person, or of a test
/// session.
#[test]
fn an_injected_key_does_not_drive_the_walk() {
    let key = |name: &str, injected| Event::NodeKey {
        node: LOOK.into(),
        key: name.into(),
        code: 28,
        state: PressState::Pressed,
        modifiers: Vec::new(),
        text: None,
        injected,
    };
    for name in ["Return", "Up", "Left", "Escape", "w", "c", "t"] {
        assert!(
            walk_key(&key(name, false), false).is_some(),
            "{name} of the person"
        );
        assert!(
            walk_key(&key(name, true), false).is_none(),
            "{name} injected"
        );
        assert!(
            walk_key(&key(name, true), true).is_some(),
            "{name} in a test"
        );
    }
}

/// A press of `input.inject` on the look took the keyboard, and with the
/// modifier moved a pane or looked around, with the switch off.
#[test]
fn an_injected_pointer_does_not_drive_the_look() {
    let pressed = |injected| Event::PointerPressed {
        node: LOOK.into(),
        button: BTN_LEFT,
        position: [0.0; 2],
        local: [0.0; 2],
        modifiers: vec!["Super".into()],
        injected,
    };
    let moved = |injected| Event::PointerMoved {
        node: LOOK.into(),
        position: [4.0; 2],
        local: [0.0; 2],
        modifiers: Vec::new(),
        injected,
        time: 0,
    };
    let released = |injected| Event::PointerReleased {
        node: LOOK.into(),
        button: BTN_LEFT,
        position: [4.0; 2],
        local: [0.0; 2],
        modifiers: Vec::new(),
        injected,
        time: 0,
    };
    let cases = [
        ("a press", pressed(false), pressed(true)),
        ("a move", moved(false), moved(true)),
        ("a release", released(false), released(true)),
    ];
    for (name, person, injected) in cases {
        assert!(look_pointer(&person, false), "{name} of the person");
        assert!(!look_pointer(&injected, false), "{name} injected");
        assert!(look_pointer(&injected, true), "{name} in a test");
    }
}

/// A construct on a fake core that answers every request with null, and
/// the methods that it saw.
fn on_fake_core(tag: &str) -> (Construct, chora_kit::testing::Log) {
    let dir = chora_kit::testing::TempDir::new(&format!("chora-construct-{tag}")).keep();
    std::fs::create_dir_all(&dir).unwrap();
    let core = chora_kit::testing::FakeCore::start();
    let methods = core.record(|method, _| Some(method.to_owned()));
    let mut rt = core.runtime(NAME);
    rt.pin_accepts_injected(false);
    let settings =
        settings::File::<Config>::load(dir.join("chora-construct.yaml"), config::DEFAULT).unwrap();
    let shaders = |name: &str, _: &str| Ok(format!("{}/{name}", dir.display()));
    let mut construct = Construct::new(rt, settings, false, &shaders).unwrap();
    construct.active = true;
    (construct, methods)
}

/// An event as the core sends it.
fn wire(event: Value) -> Event {
    serde_json::from_value(event).expect("an event of the core")
}

/// An injected key, button grab, drag move or scroll turned the scene, took
/// the look or walked the camera with the switch off; the same events of
/// the person still do.
#[test]
fn an_injected_input_drives_the_construct_only_when_the_setting_takes_it() {
    let (mut construct, _methods) = on_fake_core("injected-input");
    let key = |injected| {
        json!({"event": "node.key", "node": LOOK, "key": "w", "code": 17,
            "state": "pressed", "modifiers": [], "injected": injected})
    };
    let scroll = |injected| {
        json!({"event": "pointer.scroll", "node": LOOK, "position": [10.0, 10.0],
            "dx": 0.0, "dy": 120.0, "discrete": [0.0, 1.0], "source": "wheel",
            "modifiers": [], "injected": injected})
    };
    let grab = |injected| {
        json!({"event": "input.button", "binding": "Super+BTN_LEFT", "state": "pressed",
            "position": [10.0, 10.0], "output": "X", "local": [10.0, 10.0],
            "injected": injected})
    };
    let walks = |construct: &Construct| construct.glide.is_some();
    let strides = |construct: &Construct| construct.stride.is_some();
    let looks = |construct: &Construct| construct.drag.is_some();
    for (name, event, acted) in [
        (
            "a key",
            key as fn(bool) -> Value,
            strides as fn(&Construct) -> bool,
        ),
        ("a scroll", scroll, walks),
        ("a button grab", grab, looks),
    ] {
        construct.glide = None;
        construct.drag = None;
        construct.stride = None;
        construct.rt.pin_accepts_injected(false);
        construct.event(wire(event(true))).unwrap();
        assert!(!acted(&construct), "{name} injected");
        construct.event(wire(event(false))).unwrap();
        assert!(acted(&construct), "{name} of the person");
        construct.glide = None;
        construct.drag = None;
        construct.stride = None;
        construct.rt.pin_accepts_injected(true);
        construct.event(wire(event(true))).unwrap();
        assert!(acted(&construct), "{name} injected in a test");
    }
}

/// A window pulled out of a face while the camera is still on its way
/// into it (the handover flies the camera in, the drag turns it back):
/// the pane comes at 1:1 or nearer for the pose the camera turns back
/// to, not for the pose it passed when the drag came. Under load the
/// drag comes late in the flight, and the pane stood far behind 1:1
/// (F21, gate-construct-pull.sh "168 px wide for 200").
#[test]
fn a_window_pulled_during_the_flight_into_it_comes_at_1_to_1_where_the_camera_stays() {
    let (mut construct, _methods) = on_fake_core("pulled-in-flight");
    construct.panes.push(Pane {
        name: "probe".into(),
        source: "s1".into(),
        content: Content::Window { command: None },
        at: [0.0, 2000.0],
        yaw: 0.0,
        size: [200.0, 200.0],
        taken: true,
        attached: false,
        placed: true,
    });
    let back = Pose::default();
    construct.pose = Pose { z: 700.0, ..back };
    construct.glide = Some(Glide {
        from: back,
        to: Pose { z: 1100.0, ..back },
        start: chora_kit::clock::now(),
        ms: 380.0,
        into: Some(Flown {
            surface: "s1".into(),
            back,
            attached: false,
            plugin: None,
        }),
        smooth: false,
    });
    let grab = chora_scene::GivenGrab {
        point: [100.0, 20.0],
        velocity: [0.0; 2],
        wobble: None,
    };
    construct.take_pulled("s1", [1480.0, 400.0], &grab).unwrap();
    let rest = construct.target();
    assert_eq!((rest.x, rest.z), (back.x, back.z), "the camera turns back");
    let pane = &construct.panes[0];
    let depth = space::depth(&rest, pane.at);
    assert!(
        depth <= construct.lens.focal + 1e-6,
        "the pane is {depth} deep where the camera stays, behind 1:1 at {}",
        construct.lens.focal
    );
    let faces = space::facing([rest.x, rest.z], pane.at);
    assert!(
        (pane.yaw - faces).abs() < 1e-9,
        "the pane at the right edge turns to the pose of the flight, yaw {}, not to where the \
         camera stays, yaw {faces}",
        pane.yaw
    );
}

/// The line that a failed request leaves in the log (a guard of the move
/// of `report` to the kit).
#[test]
fn report_line() {
    if chora_kit::testing::in_child() {
        chora_kit::runtime::report(NAME, &"no such window");
        return;
    }
    assert_eq!(
        chora_kit::testing::stderr_of("tests::report_line"),
        "chora-construct: no such window\n"
    );
}
