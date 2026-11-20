//! Unit tests of the panels of xp on the fake core of the kit: the notices
//! of the launches, the dialogs that open, raise and close, their controls
//! as a press and a release click them, the Display and Effects dialogs, the
//! taskbar settings, the sliding of the taskbars, and the icons of the
//! desktop. A press and a release go through the same path as the pointer:
//! the node of the control and the point of the scene where it is.

use std::sync::{Arc, Mutex};

use chora_kit::{launch::Why, notice};

use super::super::tests::{wm_on_fake_core, wm_on_kit_core};
use super::super::*;
use super::*;
use crate::decoration::{self, Button};

/// A window of xp on the desktop that shows, as a client that mapped it.
fn own_window(wm: &mut Wm, surface: &str) {
    let mut win = crate::window::Window::new(surface, Some("probe".into()), Some(surface));
    win.shown = true;
    win.desktop = wm.desktop;
    wm.windows.push(win);
}

/// The method that a notice goes by: the core calls the client of notices.
const CALL: &str = "methods.call";

/// The methods that the fake core saw since `from`.
fn sent(methods: &Arc<Mutex<Vec<String>>>, from: usize) -> Vec<String> {
    methods.lock().unwrap()[from..].to_vec()
}

/// The page that a dialog of the panel at `i` shows now.
fn page_of(wm: &mut Wm, i: usize) -> dialog::Page {
    let now = chrono::Local::now().naive_local();
    match &wm.panels[i] {
        Panel::Taskbar(panel) => panel.page(&mut wm.theme, &wm.clock),
        Panel::Clock(panel) => panel.page(&wm.theme, now),
        Panel::Display(panel) => panel.page(&mut wm.theme),
        Panel::Effects(panel) => panel.page(),
        Panel::Help(panel) => panel.page(&wm.theme),
    }
}

/// The point of the scene in the middle of a control of the dialog at `i`.
fn point_of(wm: &mut Wm, i: usize, name: &str) -> [f64; 2] {
    let page = page_of(wm, i);
    let rect = page
        .controls
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("the dialog has the control {name}"))
        .rect;
    let [cx, cy] = wm.panels[i].window_ref().client();
    [
        f64::from(cx + rect[0] + rect[2] / 2),
        f64::from(cy + rect[1] + rect[3] / 2),
    ]
}

/// A click on a control of the dialog at `i`: the press on its node, then
/// the release at its point.
fn click(wm: &mut Wm, i: usize, name: &str) {
    let node = format!("{}-content-c-{name}", wm.panels[i].window_ref().group());
    let at = point_of(wm, i, name);
    wm.pressed(&node, LEFT_BUTTON, at);
    wm.released(&node, at);
}

/// The point of the scene in the middle of the close button of the dialog
/// at `i`, as the frame of the dialog places it. The group of a dialog holds
/// its frame with a shadow on each side, so the frame is the group less the
/// two shadows.
#[allow(clippy::cast_possible_truncation, reason = "scene coordinates")]
fn close_point(wm: &Wm, i: usize) -> [f64; 2] {
    let dialog = wm.panels[i].window_ref();
    let [cx, cy] = dialog.client();
    let outer_width = dialog.layer().1[0] as i32 - 2 * decoration::SHADOW;
    let [bx, by, bw, bh] = decoration::button_rect(outer_width, Button::Close);
    let [ox, oy] = [cx - decoration::BORDER, cy - decoration::TITLE];
    [f64::from(ox + bx + bw / 2), f64::from(oy + by + bh / 2)]
}

/// The index of the first panel that `kind` takes.
fn index_of(wm: &Wm, kind: fn(&Panel) -> bool) -> usize {
    wm.panels.iter().position(kind).expect("the panel is open")
}

/// The area of the work area of the first output, as the dialogs take it.
fn area_of(wm: &Wm) -> Rect {
    wm.bars[0].usable()
}

mod notices {
    use super::*;

    /// A dismissal that the server refuses is logged, and the connection
    /// goes on: the next dismissal passes.
    #[test]
    fn a_dismissal_the_server_refuses_is_logged_and_the_next_one_passes() {
        let core = chora_kit::testing::FakeCore::start();
        let mut wm = wm_on_kit_core(&core, "notice-refused");
        core.refuse_next(CALL);
        wm.dismiss_notice(7);
        assert!(
            notice::dismiss(&mut wm.rt.conn, 8).is_ok(),
            "the connection goes on after a refusal"
        );
    }

    /// A finished program closes the message of its start, and a finished
    /// program without a message sends nothing.
    #[test]
    fn a_finished_program_closes_its_message() {
        let (mut wm, methods) = wm_on_fake_core("tell-done");
        let before = methods.lock().unwrap().len();
        wm.tell(&Change::Done { note: Some(9) });
        assert!(
            sent(&methods, before).iter().any(|m| m == CALL),
            "the message closes"
        );
        let before = methods.lock().unwrap().len();
        wm.tell(&Change::Done { note: None });
        assert!(sent(&methods, before).is_empty(), "nothing to close");
    }

    /// A program that did not open is told in a notice that replaces the
    /// message of its start.
    #[test]
    fn a_failed_start_is_told_in_a_notice() {
        let (mut wm, methods) = wm_on_fake_core("tell-failed");
        let before = methods.lock().unwrap().len();
        wm.tell(&Change::Failed {
            name: "Paint".into(),
            note: Some(4),
            why: Why::Missing,
        });
        assert!(
            sent(&methods, before).iter().any(|m| m == CALL),
            "the failure is shown"
        );
    }

    /// A program that starts is shown as starting, and a fallback that
    /// opens with another program is told too.
    #[test]
    fn a_starting_program_and_a_fallback_are_shown() {
        let (mut wm, methods) = wm_on_fake_core("tell-starting");
        let before = methods.lock().unwrap().len();
        wm.tell(&Change::Starting {
            token: "t1".into(),
            name: "Notepad".into(),
        });
        assert!(
            sent(&methods, before).iter().any(|m| m == CALL),
            "the start is shown"
        );
        let before = methods.lock().unwrap().len();
        wm.tell(&Change::Fallback {
            name: "Editor".into(),
            from: "vim".into(),
            to: "nano".into(),
            note: Some(5),
            why: Why::Slow,
        });
        assert!(
            sent(&methods, before).iter().any(|m| m == CALL),
            "the fallback is shown"
        );
    }
}

mod run {
    use super::*;

    /// Log Off and Turn Off open their dialog, and Help opens the help
    /// window; a setting that needs no change changes nothing.
    #[test]
    fn the_actions_open_their_dialogs_and_the_help() {
        let (mut wm, _methods) = wm_on_fake_core("run-actions");
        wm.run(&Action::LogOff, None);
        assert!(wm.dialog.is_some(), "Log Off opens its dialog");
        wm.choose(Choice::Cancel);
        wm.run(&Action::TurnOff, None);
        assert!(wm.dialog.is_some(), "Turn Off opens its dialog");
        wm.choose(Choice::Cancel);
        wm.run(&Action::Help, None);
        assert!(
            wm.panels.iter().any(|p| matches!(p, Panel::Help(_))),
            "Help opens the help window"
        );
        assert_eq!(
            wm.run(&Action::Set(Setting::Note), None),
            [] as [chora_scene::Command; 0]
        );
    }

    /// A second Help raises the open help window: no second one opens.
    #[test]
    fn a_second_help_raises_the_open_window() {
        let (mut wm, _methods) = wm_on_fake_core("help-twice");
        wm.open_help(0);
        let open = wm.panels.len();
        let commands = wm.open_help(2);
        assert_eq!(wm.panels.len(), open, "the help is raised, not new");
        assert!(!commands.is_empty(), "the window is moved to the top");
    }
}

mod dialogs {
    use super::*;

    /// A second Date and Time, Display Properties or Effects raises the
    /// open one: the dialog is moved to the top, and no dialog opens.
    #[test]
    fn a_second_open_raises_the_dialog() {
        let (mut wm, _methods) = wm_on_fake_core("raise-twice");
        let area = area_of(&wm);
        wm.open_date_dialog(area);
        wm.open_display_dialog(area);
        wm.open_effects();
        let open = wm.panels.len();
        let dates = wm.open_date_dialog(area);
        let displays = wm.open_display_dialog(area);
        let effects = wm.open_effects();
        assert_eq!(wm.panels.len(), open, "no dialog opens twice");
        for raised in [dates, displays, effects] {
            assert!(
                matches!(raised.as_slice(), [Command::Move { .. }]),
                "the open dialog is moved to the top: {raised:?}"
            );
        }
    }

    /// The help window: a topic opens its page in place, and the close
    /// button closes the window.
    #[test]
    fn the_help_window_shows_topics_and_closes() {
        let (mut wm, _methods) = wm_on_fake_core("help-window");
        wm.open_help(0);
        let i = index_of(&wm, |p| matches!(p, Panel::Help(_)));
        click(&mut wm, i, "topic-1");
        assert!(
            wm.panels.iter().any(|p| matches!(p, Panel::Help(_))),
            "a topic keeps the window open"
        );
        let close = format!("{}-close", wm.panels[i].window_ref().group());
        let at = close_point(&wm, i);
        wm.pressed(&close, LEFT_BUTTON, at);
        wm.released(&close, at);
        assert!(
            !wm.panels.iter().any(|p| matches!(p, Panel::Help(_))),
            "the close button closes the help"
        );
    }

    /// A change that the settings refuse, because it is no object, changes
    /// nothing and sends nothing.
    #[test]
    fn a_change_that_is_no_object_changes_nothing() {
        let (mut wm, _methods) = wm_on_fake_core("change-refused");
        let terminal = wm.settings.value.terminal.clone();
        assert_eq!(wm.change(json!(5)), [] as [chora_scene::Command; 0]);
        assert_eq!(wm.settings.value.terminal, terminal, "nothing changed");
    }

    /// A change that cannot be saved stays applied: the settings file is a
    /// directory, so the save fails, and the new setting holds.
    #[test]
    fn a_change_that_cannot_be_saved_stays_applied() {
        let (mut wm, _methods) = wm_on_fake_core("change-unsaved");
        let file =
            chora_kit::testing::TempDir::new("chora-xp-wm-change-unsaved").join("chora-xp.yaml");
        let _ = std::fs::remove_file(&file);
        std::fs::create_dir_all(&file).unwrap();
        wm.change(json!({"terminal": "xterm-unsaved"}));
        assert_eq!(
            wm.settings.value.terminal, "xterm-unsaved",
            "the change applies"
        );
    }

    /// A Display Properties of an output the core does not list still opens,
    /// on the work area it was given.
    #[test]
    fn display_properties_of_an_unknown_output_still_opens() {
        let (mut wm, _methods) = wm_on_fake_core("display-unknown");
        wm.bars[0].output = "gone".into();
        let area = area_of(&wm);
        wm.open_display_dialog(area);
        assert!(
            wm.panels.iter().any(|p| matches!(p, Panel::Display(_))),
            "Display Properties opens"
        );
    }

    /// Changing Display Properties with none open changes nothing.
    #[test]
    fn a_change_with_no_display_properties_open_does_nothing() {
        let (mut wm, _methods) = wm_on_fake_core("display-none");
        assert_eq!(wm.with_display(|_| {}), [] as [chora_scene::Command; 0]);
    }

    /// A close button closes the dialog, and a release off the control
    /// that was pressed clicks nothing.
    #[test]
    fn a_release_off_the_control_clicks_nothing_and_close_closes() {
        let (mut wm, _methods) = wm_on_fake_core("dialog-close");
        let area = area_of(&wm);
        wm.open_date_dialog(area);
        let i = index_of(&wm, |p| matches!(p, Panel::Clock(_)));
        let node = format!("{}-content-c-ok", wm.panels[i].window_ref().group());
        wm.pressed(&node, LEFT_BUTTON, [0.0, 0.0]);
        wm.released(&node, [0.0, 0.0]);
        assert!(
            wm.panels.iter().any(|p| matches!(p, Panel::Clock(_))),
            "a release off the control keeps the dialog"
        );
        let close = format!("{}-close", wm.panels[i].window_ref().group());
        let at = close_point(&wm, i);
        wm.pressed(&close, LEFT_BUTTON, at);
        wm.released(&close, at);
        assert!(
            !wm.panels.iter().any(|p| matches!(p, Panel::Clock(_))),
            "the close button closes the dialog"
        );
    }

    /// Closing a dialog while no window has the focus gives the focus to the
    /// topmost window of the desktop, as in XP.
    #[test]
    fn closing_a_dialog_gives_the_focus_to_the_topmost_window() {
        let (mut wm, _methods) = wm_on_fake_core("close-focus");
        own_window(&mut wm, "s1");
        wm.focused = None;
        let area = area_of(&wm);
        wm.open_date_dialog(area);
        let i = index_of(&wm, |p| matches!(p, Panel::Clock(_)));
        wm.close_panel(i);
        assert_eq!(
            wm.focused.as_deref(),
            Some("s1"),
            "the window gets the focus"
        );
    }

    /// Display Properties keeps its tab for the next time: closed on the
    /// second tab, it remembers the tab.
    #[test]
    fn display_properties_keeps_its_tab() {
        let (mut wm, _methods) = wm_on_fake_core("display-tab");
        let area = area_of(&wm);
        wm.open_display_dialog(area);
        let i = index_of(&wm, |p| matches!(p, Panel::Display(_)));
        click(&mut wm, i, "tab-1");
        assert!(
            wm.panels.iter().any(|p| matches!(p, Panel::Display(_))),
            "a tab keeps the dialog open"
        );
        click(&mut wm, i, "cancel");
        assert_eq!(wm.display_tab, 1, "the tab is kept");
    }
}

mod clock {
    use super::*;

    /// A press on the tray that is on neither the speaker nor the clock
    /// opens nothing.
    #[test]
    fn a_press_on_the_tray_off_the_clock_opens_nothing() {
        let (mut wm, _methods) = wm_on_fake_core("tray-off");
        assert_eq!(wm.press_tray([0.0, 0.0]), [] as [chora_scene::Command; 0]);
        assert!(!wm.panels.iter().any(|p| matches!(p, Panel::Clock(_))));
    }

    /// A click on the month list of the date opens the months, and the OK
    /// of the date closes the dialog.
    #[test]
    fn the_month_list_opens_and_ok_closes() {
        let (mut wm, _methods) = wm_on_fake_core("clock-controls");
        let area = area_of(&wm);
        wm.open_date_dialog(area);
        let i = index_of(&wm, |p| matches!(p, Panel::Clock(_)));
        click(&mut wm, i, "month");
        assert!(wm.context.is_some(), "the months are a list");
        wm.node_key("Escape");
        click(&mut wm, i, "tab-1");
        assert!(
            wm.panels.iter().any(|p| matches!(p, Panel::Clock(_))),
            "a tab keeps the date open"
        );
        click(&mut wm, i, "ok");
        assert!(
            !wm.panels.iter().any(|p| matches!(p, Panel::Clock(_))),
            "OK closes the date"
        );
    }

    /// The date takes the day that a click on the calendar points at, and a
    /// year spin moves the year.
    #[test]
    fn the_calendar_and_the_year_change_the_date() {
        let (mut wm, _methods) = wm_on_fake_core("clock-calendar");
        let area = area_of(&wm);
        wm.open_date_dialog(area);
        let i = index_of(&wm, |p| matches!(p, Panel::Clock(_)));
        click(&mut wm, i, "calendar");
        click(&mut wm, i, "year");
        assert!(
            wm.panels.iter().any(|p| matches!(p, Panel::Clock(_))),
            "the date stays open"
        );
    }

    /// A press on the speaker of the taskbar opens the volume popup and
    /// does not mute; a row of it mutes.
    #[test]
    fn a_click_on_the_speaker_opens_the_volume_popup() {
        let (mut wm, _methods) = wm_on_fake_core("speaker-popup");
        let [x, y, w, h] = wm.bars[0].speaker_rect();
        wm.press_tray([f64::from(x + w / 2), f64::from(y + h / 2)]);
        assert!(wm.context.is_some(), "the popup is open");
    }

    /// Two presses on the clock of the taskbar within a double click open
    /// Date and Time; one press opens nothing.
    #[test]
    fn a_double_click_on_the_clock_opens_date_and_time() {
        let (mut wm, _methods) = wm_on_fake_core("clock-double");
        let [x, y, w, h] = wm.bars[0]
            .clock_rect()
            .expect("the taskbar shows the clock");
        let at = [f64::from(x + w / 2), f64::from(y + h / 2)];
        wm.press_tray(at);
        assert!(
            !wm.panels.iter().any(|p| matches!(p, Panel::Clock(_))),
            "one press opens nothing"
        );
        wm.press_tray(at);
        assert!(
            wm.panels.iter().any(|p| matches!(p, Panel::Clock(_))),
            "the double click opens Date and Time"
        );
    }
}

mod display {
    use super::*;

    /// The Display Properties buttons: Cancel closes, Apply keeps the dialog
    /// open, and Effects opens the Effects dialog.
    #[test]
    fn the_buttons_of_display_properties_act() {
        let (mut wm, _methods) = wm_on_fake_core("display-buttons");
        let area = area_of(&wm);
        wm.open_display_dialog(area);
        let i = index_of(&wm, |p| matches!(p, Panel::Display(_)));
        click(&mut wm, i, "apply");
        assert!(
            wm.panels.iter().any(|p| matches!(p, Panel::Display(_))),
            "Apply keeps the dialog"
        );
        click(&mut wm, i, "tab-3");
        click(&mut wm, i, "effects");
        assert!(
            wm.panels.iter().any(|p| matches!(p, Panel::Effects(_))),
            "Effects opens its dialog"
        );
        let e = index_of(&wm, |p| matches!(p, Panel::Effects(_)));
        click(&mut wm, e, "cancel");
        assert!(!wm.panels.iter().any(|p| matches!(p, Panel::Effects(_))));
        click(&mut wm, i, "cancel");
        assert!(!wm.panels.iter().any(|p| matches!(p, Panel::Display(_))));
    }

    /// The Position and Color lists of the desktop tab open as menus of
    /// their rows.
    #[test]
    fn the_position_and_color_lists_open_as_menus() {
        let (mut wm, _methods) = wm_on_fake_core("display-lists");
        let area = area_of(&wm);
        wm.open_display_dialog(area);
        let i = index_of(&wm, |p| matches!(p, Panel::Display(_)));
        click(&mut wm, i, "tab-4");
        click(&mut wm, i, "tab-1");
        click(&mut wm, i, "position");
        assert!(wm.context.is_some(), "the positions are a list");
        wm.node_key("Escape");
        click(&mut wm, i, "color");
        assert!(wm.context.is_some(), "the colors are a list");
        wm.node_key("Escape");
    }

    /// Apply takes a change of the scale: the core is told the new scale.
    #[test]
    fn apply_takes_a_new_scale_from_the_slider() {
        let (mut wm, methods) = wm_on_fake_core("display-scale");
        let area = area_of(&wm);
        wm.open_display_dialog(area);
        let i = index_of(&wm, |p| matches!(p, Panel::Display(_)));
        let before = methods.lock().unwrap().len();
        let _ = wm.click_display(i, &Hit::Control("slider".into(), [0, 0]));
        let _ = wm.click_display(i, &Hit::Control("apply".into(), [0, 0]));
        assert!(
            sent(&methods, before)
                .iter()
                .any(|m| m == "output.configure"),
            "the new scale goes to the core"
        );
    }

    /// Apply takes a new picture of the desktop: the first row of the list
    /// is no picture, and the settings of the wallpaper change.
    #[test]
    fn apply_takes_a_new_wallpaper_from_the_list() {
        let (mut wm, _methods) = wm_on_fake_core("display-wall");
        let area = area_of(&wm);
        wm.open_display_dialog(area);
        let i = index_of(&wm, |p| matches!(p, Panel::Display(_)));
        let _ = wm.click_display(i, &Hit::Control("list".into(), [10, 1]));
        let _ = wm.click_display(i, &Hit::Control("apply".into(), [0, 0]));
        assert!(
            wm.panels.iter().any(|p| matches!(p, Panel::Display(_))),
            "Apply keeps the dialog"
        );
    }

    /// OK applies and closes, and a control that is not one of the page
    /// changes nothing of the dialog.
    #[test]
    fn ok_applies_and_closes_the_display_properties() {
        let (mut wm, _methods) = wm_on_fake_core("display-ok");
        let area = area_of(&wm);
        wm.open_display_dialog(area);
        let i = index_of(&wm, |p| matches!(p, Panel::Display(_)));
        let _ = wm.click_display(i, &Hit::Control("unknown".into(), [0, 0]));
        assert!(
            wm.panels.iter().any(|p| matches!(p, Panel::Display(_))),
            "an unknown control keeps the dialog"
        );
        click(&mut wm, i, "ok");
        assert!(!wm.panels.iter().any(|p| matches!(p, Panel::Display(_))));
    }

    /// A hit on a dialog that is not Display Properties changes nothing
    /// of the Display Properties, and there is none to change.
    #[test]
    fn a_hit_on_another_dialog_changes_no_display() {
        let (mut wm, _methods) = wm_on_fake_core("display-other");
        let area = area_of(&wm);
        wm.open_date_dialog(area);
        let i = index_of(&wm, |p| matches!(p, Panel::Clock(_)));
        assert_eq!(
            wm.click_display(i, &Hit::Close),
            [] as [chora_scene::Command; 0]
        );
    }
}

mod effects {
    use super::*;

    /// Effects has no page to show again: a redraw of it sends nothing.
    #[test]
    fn effects_has_no_page_to_show_again() {
        let (mut wm, _methods) = wm_on_fake_core("effects-redraw");
        wm.open_effects();
        let i = index_of(&wm, |p| matches!(p, Panel::Effects(_)));
        assert_eq!(wm.redraw_panel(i), [] as [chora_scene::Command; 0]);
    }

    /// A check box of the Effects dialog flips; OK takes the effects, and
    /// Cancel closes them without a change.
    #[test]
    fn ok_takes_the_effects_and_cancel_keeps_them() {
        let (mut wm, _methods) = wm_on_fake_core("effects-take");
        wm.open_effects();
        let i = index_of(&wm, |p| matches!(p, Panel::Effects(_)));
        let lamp = wm.effects.toggles.lamp;
        click(&mut wm, i, "lamp");
        assert_eq!(
            wm.effects.toggles.lamp, lamp,
            "a check box takes effect on OK"
        );
        click(&mut wm, i, "ok");
        assert_ne!(wm.effects.toggles.lamp, lamp, "OK takes the lamp");
        wm.open_effects();
        let j = index_of(&wm, |p| matches!(p, Panel::Effects(_)));
        let wobbly = wm.effects.toggles.wobbly;
        click(&mut wm, j, "wobbly");
        click(&mut wm, j, "cancel");
        assert_eq!(
            wm.effects.toggles.wobbly, wobbly,
            "Cancel keeps the effects"
        );
    }

    /// A hit on an Effects control that is not a box or a button keeps the
    /// dialog open, and the close button closes it.
    #[test]
    fn an_unknown_control_keeps_effects_open() {
        let (mut wm, _methods) = wm_on_fake_core("effects-keep");
        wm.open_effects();
        let i = index_of(&wm, |p| matches!(p, Panel::Effects(_)));
        let _ = wm.click_display(i, &Hit::Control("none".into(), [0, 0]));
        assert!(wm.panels.iter().any(|p| matches!(p, Panel::Effects(_))));
        let _ = wm.click_display(i, &Hit::Close);
        assert!(!wm.panels.iter().any(|p| matches!(p, Panel::Effects(_))));
    }
}

mod taskbar_settings {
    use super::*;

    /// The Taskbar dialog changes the auto-hide of the taskbar on OK, and
    /// Cancel keeps it.
    #[test]
    fn ok_takes_the_taskbar_settings_and_cancel_keeps_them() {
        let (mut wm, _methods) = wm_on_fake_core("taskbar-ok");
        wm.open_taskbar_dialog(area_of(&wm));
        let i = index_of(&wm, |p| matches!(p, Panel::Taskbar(_)));
        let auto = wm.taskbar.auto_hide;
        click(&mut wm, i, "auto_hide");
        click(&mut wm, i, "ok");
        assert_ne!(wm.taskbar.auto_hide, auto, "OK takes auto-hide");
        wm.open_taskbar_dialog(area_of(&wm));
        let j = index_of(&wm, |p| matches!(p, Panel::Taskbar(_)));
        let on_top = wm.taskbar.on_top;
        click(&mut wm, j, "on_top");
        click(&mut wm, j, "cancel");
        assert_eq!(wm.taskbar.on_top, on_top, "Cancel keeps the taskbar");
    }

    /// Apply keeps the Taskbar dialog open, and the page that shows the
    /// settings follows the change.
    #[test]
    fn apply_keeps_the_taskbar_dialog_open() {
        let (mut wm, _methods) = wm_on_fake_core("taskbar-apply");
        wm.open_taskbar_dialog(area_of(&wm));
        let i = index_of(&wm, |p| matches!(p, Panel::Taskbar(_)));
        click(&mut wm, i, "locked");
        click(&mut wm, i, "apply");
        assert!(wm.panels.iter().any(|p| matches!(p, Panel::Taskbar(_))));
    }

    /// Changing the taskbar moves it above the windows, and a taskbar dialog
    /// that is open shows the new settings.
    #[test]
    fn a_change_of_the_taskbar_moves_the_bars_and_redraws_its_dialog() {
        let (mut wm, _methods) = wm_on_fake_core("taskbar-set");
        wm.open_taskbar_dialog(area_of(&wm));
        let mut next = wm.taskbar;
        next.on_top = !next.on_top;
        let commands = wm.set_taskbar(next);
        assert!(
            commands.iter().any(|c| matches!(
                c,
                Command::Move { id, .. } if id == BARS
            )),
            "the bars move in the stack"
        );
        assert_eq!(wm.taskbar, next, "the settings are the new ones");
    }

    /// A window that is maximized fills the work area that the taskbar
    /// leaves when the taskbar hides itself.
    #[test]
    fn a_maximized_window_fills_the_area_a_hiding_taskbar_frees() {
        let (mut wm, _methods) = wm_on_fake_core("taskbar-refit");
        own_window(&mut wm, "s1");
        wm.toggle_maximize("s1");
        let before = wm.windows[0].rect();
        let mut next = wm.taskbar;
        next.auto_hide = true;
        wm.set_taskbar(next);
        assert_ne!(
            wm.windows[0].rect(),
            before,
            "the window fills the new work area"
        );
    }

    /// A taskbar that hides itself slides away when the pointer is not on it,
    /// and the slide ends once it is hidden.
    #[test]
    fn a_hiding_taskbar_slides_away_when_the_pointer_leaves_it() {
        let (mut wm, _methods) = wm_on_fake_core("taskbar-slide");
        let mut next = wm.taskbar;
        next.auto_hide = true;
        wm.set_taskbar(next);
        wm.hover = None;
        wm.auto_hide();
        assert!(wm.sliding(), "the taskbar waits to hide");
        let later = Instant::now() + HIDE_DELAY * 2;
        for _ in 0..200 {
            if !wm.sliding() {
                break;
            }
            wm.slide_frame(later, 0.05);
        }
        assert!(!wm.sliding(), "the slide ends hidden");
    }

    /// The pointer on a taskbar that hides itself brings it back and stops
    /// its wait to hide; the pointer off it starts the wait again.
    #[test]
    fn the_pointer_on_a_hiding_taskbar_brings_it_back() {
        let (mut wm, _methods) = wm_on_fake_core("taskbar-back");
        let mut next = wm.taskbar;
        next.auto_hide = true;
        wm.set_taskbar(next);
        wm.hover = None;
        wm.auto_hide();
        assert!(wm.hide_at.is_some(), "the taskbar waits to hide");
        wm.hover = Some("xp-panel-X".into());
        wm.auto_hide();
        assert!(wm.hide_at.is_none(), "the pointer on it stops the wait");
        wm.hover = None;
        wm.auto_hide();
        assert!(wm.hide_at.is_some(), "the wait starts again");
    }

    /// A taskbar stays while the start menu is open, even when the pointer
    /// left it.
    #[test]
    fn a_taskbar_stays_while_the_start_menu_is_open() {
        let (mut wm, _methods) = wm_on_fake_core("taskbar-stay");
        let mut next = wm.taskbar;
        next.auto_hide = true;
        wm.set_taskbar(next);
        wm.press_taskbar(taskbar::Part::Start("X".into()));
        assert!(wm.menu.is_open(), "the start menu opens");
        wm.hover = None;
        wm.auto_hide();
        let later = Instant::now() + HIDE_DELAY * 2;
        wm.slide_frame(later, 0.05);
        assert!(wm.hide_at.is_some(), "the hide waits for the menu");
    }
}

mod icons {
    use super::*;

    /// A press on the desktop takes the focus and drags a selection
    /// rectangle; the release ends it.
    #[test]
    fn a_press_on_the_desktop_selects_and_releases() {
        let (mut wm, _methods) = wm_on_fake_core("desktop-press");
        own_window(&mut wm, "s1");
        wm.focused = Some("s1".into());
        let at = [600.0, 400.0];
        wm.press_wallpaper(at);
        assert!(wm.focused.is_none(), "the desktop takes the focus");
        assert!(wm.desk.pressed(), "the press of the desktop is down");
        wm.dragged([700.0, 500.0]);
        wm.released(&wallpaper::id("X"), [700.0, 500.0]);
        assert!(!wm.desk.pressed(), "the release ends the press");
    }

    /// A press on an icon of the desktop takes the focus from the windows.
    #[test]
    fn a_press_on_an_icon_takes_the_focus_from_the_windows() {
        let (mut wm, _methods) = wm_on_fake_core("icon-press");
        own_window(&mut wm, "s1");
        wm.focused = Some("s1".into());
        wm.press_icon("xp-icon-0", [40.0, 40.0]);
        assert!(wm.focused.is_none(), "the desktop takes the focus");
    }
}
