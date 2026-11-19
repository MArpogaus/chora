//! Tests of the hand-over of a dragged window on the fake core of the kit:
//! a release where no window manager holds a claim keeps the window with xp.

use super::super::tests::wm_on_kit_core;
use super::super::*;

/// A window of xp on the desktop that shows, as a client that mapped it.
fn own_window(wm: &mut Wm, surface: &str) {
    let mut win = crate::window::Window::new(surface, Some("probe".into()), Some(surface));
    win.shown = true;
    win.desktop = wm.desktop;
    wm.windows.push(win);
}

/// A release on no claim keeps the dragged window with xp: the window
/// stays in the list, and its commands only move it back to the desktop.
#[test]
fn a_release_on_no_claim_keeps_the_window_with_xp() {
    let core = chora_kit::testing::FakeCore::start();
    let mut wm = wm_on_kit_core(&core, "pointer-hand-over");
    own_window(&mut wm, "s1");
    let commands = wm.hand_over("s1", [10.0, 10.0]);
    assert_eq!(wm.windows.len(), 1, "the window stays with xp");
    assert_eq!(wm.index("s1"), Some(0), "the window is still the first");
    for command in &commands {
        assert!(
            matches!(command, Command::Update { id, .. } if id == "s1"),
            "only the window moves back: {command:?}"
        );
    }
}
