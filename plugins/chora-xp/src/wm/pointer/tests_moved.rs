//! Tests of the pointer of xp on the fake core of the kit: the presses on the
//! parts of windows, the taskbar and the desktop, the drags, the releases, and
//! the hand-over of a dragged window. A press and a release go through the
//! path of the pointer: the node and the point are those of the scene.

use super::super::tests::{wm_on_fake_core, wm_on_kit_core};
use super::super::*;
use crate::decoration::{self, Button};

/// A window of xp on the desktop that shows, as a client that mapped it.
fn own_window(wm: &mut Wm, surface: &str) {
    let mut win = crate::window::Window::new(surface, Some("probe".into()), Some(surface));
    win.shown = true;
    win.desktop = wm.desktop;
    wm.windows.push(win);
}

/// A window of xp at a place and a size, framed as a client draws it.
fn placed(wm: &mut Wm, surface: &str, pos: [i32; 2], size: [i32; 2]) {
    own_window(wm, surface);
    let win = wm.windows.last_mut().unwrap();
    win.pos = pos;
    win.size = Some(size);
    win.framed = true;
}

/// The point of the scene in the middle of a button of the window `surface`.
fn button_at(wm: &Wm, surface: &str, button: Button) -> [f64; 2] {
    let win = wm.windows.iter().find(|w| w.surface == surface).unwrap();
    let [bx, by, bw, bh] = decoration::button_rect(win.outer()[0], button);
    [
        f64::from(win.pos[0] + bx + bw / 2),
        f64::from(win.pos[1] + by + bh / 2),
    ]
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

mod parts {
    use super::*;

    /// A press on the frame of a window activates it; a right press on the
    /// frame does nothing, and a press on a window that is gone is nothing.
    #[test]
    fn a_press_on_the_frame_activates_the_window_and_a_right_press_does_not() {
        let (mut wm, _methods) = wm_on_fake_core("pointer-frame");
        placed(&mut wm, "s1", [100, 100], [300, 200]);
        wm.press_decoration(&Part::Frame("s1".into()), LEFT_BUTTON, [150.0, 150.0]);
        assert_eq!(wm.focused.as_deref(), Some("s1"), "the frame activates s1");
        wm.press_decoration(&Part::Frame("s1".into()), RIGHT_BUTTON, [150.0, 150.0]);
        assert!(wm.drag.is_none(), "a right press starts no drag");
        wm.press_decoration(&Part::Frame("gone".into()), LEFT_BUTTON, [0.0, 0.0]);
        assert!(wm.drag.is_none(), "a window that is gone starts no drag");
    }

    /// A press on the icon of a title bar opens the system menu below the
    /// icon, and a second press on it closes the window, as on XP.
    #[test]
    fn the_icon_of_a_title_bar_opens_the_system_menu_and_a_double_click_closes() {
        let (mut wm, _methods) = wm_on_fake_core("pointer-icon");
        placed(&mut wm, "s1", [100, 100], [300, 200]);
        wm.windows[0].icon = Some("icon.png".into());
        let [ix, iy, iw, ih] = decoration::ICON;
        let at = [f64::from(100 + ix + iw / 2), f64::from(100 + iy + ih / 2)];
        wm.press_decoration(&Part::Title("s1".into()), LEFT_BUTTON, at);
        assert!(wm.context.is_some(), "the icon opens the system menu");
        wm.node_key("Escape");
        let closed = wm.press_icon_of("s1");
        assert!(
            closed
                .iter()
                .any(|c| matches!(c, Command::Close { surface } if surface == "s1")),
            "a double click on the icon closes s1: {closed:?}"
        );
    }

    /// A system menu of a window that is gone opens nothing; a title bar of a
    /// maximized window does not start a drag; a taskbar part that is not a
    /// start or a pager part of an output is nothing.
    #[test]
    fn a_part_of_nothing_or_of_a_maximized_window_does_nothing() {
        let (mut wm, _methods) = wm_on_fake_core("pointer-nothing");
        assert_eq!(
            wm.system_below_icon("gone"),
            [] as [chora_scene::Command; 0]
        );
        placed(&mut wm, "s1", [100, 100], [300, 200]);
        wm.toggle_maximize("s1");
        assert_eq!(
            wm.press_title("s1", [150.0, 110.0]),
            [] as [chora_scene::Command; 0]
        );
        assert!(wm.drag.is_none(), "a maximized window is not dragged");
        assert_eq!(
            wm.press_taskbar(taskbar::Part::Start("nowhere".into())),
            [] as [chora_scene::Command; 0]
        );
        assert_eq!(
            wm.press_taskbar(taskbar::Part::Speaker("X".into())),
            [] as [chora_scene::Command; 0]
        );
    }

    /// The tray, the menu, the desktop icons and the wallpaper take their own
    /// presses: the tray a press on its speaker or clock, the wallpaper the
    /// selection of the desktop.
    #[test]
    fn the_left_press_goes_to_the_part_it_is_on() {
        let (mut wm, _methods) = wm_on_fake_core("pointer-left");
        assert_eq!(
            wm.press_left("xp-tray-X", [0.0, 0.0]),
            [] as [chora_scene::Command; 0]
        );
        wm.press_taskbar(taskbar::Part::Start("X".into()));
        assert!(wm.menu.is_open(), "the start menu opens");
        wm.press_left("xp-menu-X", [0.0, 0.0]);
        assert!(wm.menu.is_open(), "a press on the menu keeps it open");
        wm.press_left(&wallpaper::id("X"), [600.0, 400.0]);
        assert!(wm.desk.pressed(), "the desktop is pressed");
        wm.release_icons([600.0, 400.0]);
        wm.press_left("xp-icon-0", [40.0, 40.0]);
        assert!(
            wm.focused.is_none(),
            "an icon takes the focus from the windows"
        );
    }

    /// The release of a button of a window acts when the pointer goes up on
    /// the same button, and a release of a button of a window that is gone
    /// does nothing.
    #[test]
    fn a_release_on_a_window_button_maximizes_it() {
        let (mut wm, _methods) = wm_on_fake_core("pointer-button");
        placed(&mut wm, "s1", [100, 100], [300, 200]);
        let at = button_at(&wm, "s1", Button::Maximize);
        wm.press_decoration(
            &Part::Button("s1".into(), Button::Maximize),
            LEFT_BUTTON,
            at,
        );
        wm.released("xp-max-s1", at);
        assert!(wm.windows[0].maximized(), "the release maximizes s1");
        assert!(
            wm.release_button("gone", Button::Close, at).is_empty(),
            "a button of a window that is gone"
        );
    }

    /// A drop of desktop icons on a window goes back to the desktop, and
    /// the desktop stops being pressed.
    #[test]
    fn icons_dropped_on_a_window_go_back_and_the_press_ends() {
        let (mut wm, _methods) = wm_on_fake_core("pointer-icons-window");
        placed(&mut wm, "s1", [100, 100], [300, 200]);
        wm.press_wallpaper([600.0, 400.0]);
        wm.dragged([650.0, 450.0]);
        wm.released(&wallpaper::id("X"), [200.0, 200.0]);
        assert!(!wm.desk.pressed(), "the release ends the press");
    }

    /// The taskbar of a dialog that is dragged moves the dialog, and its
    /// release ends the drag without a click.
    #[test]
    fn a_dialog_dragged_by_its_title_moves_and_its_release_ends_it() {
        let (mut wm, _methods) = wm_on_fake_core("pointer-dialog-drag");
        let area = wm.bars[0].usable();
        wm.open_date_dialog(area);
        let i = wm
            .panels
            .iter()
            .position(|p| matches!(p, Panel::Clock(_)))
            .unwrap();
        let [cx, cy] = wm.panels[i].window_ref().client();
        let start = [f64::from(cx + 40), f64::from(cy - 10)];
        let title = format!("{}-title", wm.panels[i].window_ref().group());
        wm.pressed(&title, LEFT_BUTTON, start);
        wm.dragged([start[0] + 30.0, start[1] + 20.0]);
        let [moved_x, moved_y] = wm.panels[i].window_ref().client();
        assert_eq!([moved_x - cx, moved_y - cy], [30, 20], "the dialog follows");
        wm.released(&title, [start[0] + 30.0, start[1] + 20.0]);
        assert!(
            wm.panels.iter().any(|p| matches!(p, Panel::Clock(_))),
            "the drag ends and the dialog stays"
        );
    }

    /// A drag that the core kept moves the window with xp; one that does not
    /// move changes nothing, and a window that is gone is not moved.
    #[test]
    fn a_drag_moves_the_window_only_when_it_moves() {
        let (mut wm, _methods) = wm_on_fake_core("pointer-drag");
        placed(&mut wm, "s1", [100, 100], [300, 200]);
        wm.press_node("xp-title-s1", LEFT_BUTTON, [150.0, 110.0]);
        assert!(wm.drag.is_some(), "the drag began");
        assert!(wm.dragged([150.0, 110.0]).is_empty(), "no move, no command");
        wm.drag.as_mut().unwrap().kept = true;
        wm.dragged([200.0, 160.0]);
        assert_eq!(wm.windows[0].pos, [150, 150], "a kept drag moves in xp");
        wm.windows.clear();
        assert!(
            wm.dragged([250.0, 200.0]).is_empty(),
            "a window that is gone is not moved"
        );
    }

    /// A drag of the desktop draws its selection, and a drag with nothing
    /// pressed changes nothing.
    #[test]
    fn a_drag_of_the_desktop_draws_the_selection() {
        let (mut wm, _methods) = wm_on_fake_core("pointer-desktop-drag");
        assert!(wm.dragged([1.0, 1.0]).is_empty(), "nothing is dragged");
        wm.press_wallpaper([600.0, 400.0]);
        wm.dragged([650.0, 450.0]);
        assert!(wm.desk.pressed(), "the selection is drawn");
    }

    /// A release off every output hands the window over, and with no window
    /// manager there the window stays with xp.
    #[test]
    fn a_release_outside_the_taskbars_keeps_a_window_without_a_claim() {
        let (mut wm, _methods) = wm_on_fake_core("pointer-release-outside");
        placed(&mut wm, "s1", [100, 100], [300, 200]);
        wm.press_node("xp-title-s1", LEFT_BUTTON, [150.0, 110.0]);
        wm.released("xp-title-s1", [5000.0, 5000.0]);
        assert!(wm.drag.is_none(), "the drag ends");
        assert_eq!(wm.windows.len(), 1, "s1 stays with xp");
    }

    /// The launch check, with nothing started, says nothing to the user.
    #[test]
    fn nothing_is_told_without_a_started_program() {
        let (mut wm, methods) = wm_on_fake_core("pointer-launches");
        let before = methods.lock().unwrap().len();
        wm.check_launches();
        assert_eq!(methods.lock().unwrap().len(), before, "nothing is sent");
    }
}
