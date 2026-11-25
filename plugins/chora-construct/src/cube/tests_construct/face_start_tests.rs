//! Unit tests of a program that starts in a face while the key sheet is
//! open (co-78, rcr1-1), on the fake core of `tests_construct`.

use chora_kit::testing::focus_changed;

use super::close_tests::in_a_face_beside_a_pane;

/// rcr1-1 (co-reach r1, MINOR): Super+t in a face, with the key sheet
/// open over it (the construct holds the key then), starts the terminal
/// in the face, as it does with the sheet shut: no pane waits for it, so
/// the face manager gets the window and the demo's face is not empty.
#[test]
fn super_t_with_the_key_sheet_over_a_face_starts_no_pane() {
    let (mut construct, _core, _lines) = in_a_face_beside_a_pane("sheet-terminal", false);
    construct.toggle_help().unwrap();
    assert!(construct.help, "the key sheet is open");
    assert!(construct.waiting.is_empty(), "nothing waits to start with");
    construct.terminal_beside().unwrap();
    assert!(
        construct.waiting.is_empty(),
        "the terminal of Super+t over a face is no pane of the construct: {} waits for a pane",
        construct.waiting.len()
    );
}

/// co-78 (rc 2, MINOR): a window that opens in the face while the key
/// sheet is open takes the keyboard from the sheet; the construct takes it
/// back, so that Escape reaches the sheet, and notes the window, so that
/// the close of the sheet gives it the keyboard again.
#[test]
fn a_window_that_takes_the_keyboard_from_the_key_sheet_in_a_face_is_noted() {
    let (mut construct, core, _lines) = in_a_face_beside_a_pane("sheet-new-window", false);
    construct.toggle_help().unwrap();
    core.model().lock().unwrap().focuses.clear();
    core.model().lock().unwrap().scene.set_focused(Some("s2"));
    chora_kit::runtime::Plugin::event(&mut construct, focus_changed(Some("s2"))).unwrap();
    assert!(
        core.model().lock().unwrap().focuses.contains(&None),
        "the keyboard goes back to the sheet: {:?}",
        core.model().lock().unwrap().focuses
    );
    assert_eq!(
        construct
            .cube
            .as_mut()
            .and_then(crate::cube::Host::take_focus),
        Some("s2".into()),
        "the window is noted for the close of the sheet"
    );
}
