//! Unit tests of the construct plugin.

use super::*;
use crate::{adopt::*, camera::*};

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

/// A shader goes to the runtime directory only, never to the shared
/// temporary directory.
#[test]
fn a_shader_needs_the_runtime_dir() {
    assert!(shader_file(None, "d", "x.frag", "void main() {}").is_err());
    assert!(shader_file(Some("rel".into()), "d", "x.frag", "void main() {}").is_err());
    let dir = std::env::temp_dir().join(format!("chora-shader-test-{}", std::process::id()));
    let path = shader_file(Some(dir.clone().into()), "d", "x.frag", "void main() {}").unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "void main() {}");
    std::fs::remove_dir_all(dir).unwrap();
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
    };
    turn(&mut pose, Some(&mut glide), 10.0, -5.0);
    pose = glide.from.lerp(&glide.to, 1.0);
    turn(&mut pose, None, 10.0, 0.0);
    assert!(
        (pose.x - to.x).abs() < 1e-9 && (pose.z - to.z).abs() < 1e-9,
        "{pose:?}"
    );
    assert!((pose.yaw - 70.0 * LOOK_RATE).abs() < 1e-9);
    assert!((pose.pitch - 5.0 * LOOK_RATE).abs() < 1e-9);
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
