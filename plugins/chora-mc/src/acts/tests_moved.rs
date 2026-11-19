//! Tests of the actions of mc on the fake core: the drag of a window, the
//! looks that turn the scenes and the regions, and the host actions.

use chora_kit::actions::System;
use chora_kit::actions::perform::{Failure, Host};

use crate::config::Look;
use crate::region::Region;
use crate::testing;
use crate::{Action, Mc, PressState};

const DRAG: &str = "Super+BTN_LEFT";

/// Binds `binding` to `action` in the runtime of mc, as the grabs of the
/// settings do on the real core.
fn bind(mc: &mut Mc, binding: &str, action: Action) {
    mc.rt.grab(binding.parse().unwrap(), action).unwrap();
}

/// mc with the drag bound, as the default settings bind it.
fn dragging(dir: &str) -> (Mc, chora_kit::testing::Model) {
    let (mut mc, model) = testing::mc(dir, [1280, 800]);
    bind(&mut mc, DRAG, Action::System(System::Drag));
    (mc, model)
}

/// A press on the drag binding takes the window that mc shows.
#[test]
fn a_press_on_the_drag_takes_the_shown_window() {
    let (mut mc, _model) = dragging("acts-press-shown");
    mc.show("agenda", false).unwrap();
    mc.windows.insert("agenda".into(), "s1".into());
    mc.drag(DRAG, PressState::Pressed, [10.0, 10.0], None);
    assert_eq!(mc.dragged.as_deref(), Some("s1"));
}

/// A press on a window of mc takes that window, not the shown one.
#[test]
fn a_press_on_a_window_of_mc_takes_that_window() {
    let (mut mc, _model) = dragging("acts-press-on");
    mc.windows.insert("agenda".into(), "s1".into());
    mc.windows.insert("mission".into(), "s2".into());
    mc.drag(DRAG, PressState::Pressed, [10.0, 10.0], Some("s2".into()));
    assert_eq!(mc.dragged.as_deref(), Some("s2"));
}

/// A press on a surface that mc does not hold takes nothing: mc moves no
/// window it does not own.
#[test]
fn a_press_on_a_foreign_surface_takes_nothing() {
    let (mut mc, _model) = dragging("acts-press-foreign");
    mc.windows.insert("agenda".into(), "s1".into());
    mc.drag(
        DRAG,
        PressState::Pressed,
        [10.0, 10.0],
        Some("other".into()),
    );
    assert_eq!(mc.dragged, None);
}

/// A press on a binding that is not the drag takes nothing.
#[test]
fn a_press_on_another_binding_takes_nothing() {
    let (mut mc, _model) = testing::mc("acts-press-other", [1280, 800]);
    mc.show("agenda", false).unwrap();
    mc.windows.insert("agenda".into(), "s1".into());
    mc.drag("Super+x", PressState::Pressed, [10.0, 10.0], None);
    assert_eq!(mc.dragged, None);
}

/// A release without a press gives nothing and keeps nothing.
#[test]
fn a_release_without_a_press_gives_nothing() {
    let (mut mc, model) = dragging("acts-release-none");
    mc.drag(DRAG, PressState::Released, [10.0, 10.0], None);
    assert_eq!(mc.dragged, None);
    assert!(
        !model
            .lock()
            .unwrap()
            .calls
            .iter()
            .any(|c| c == "window.give")
    );
}

/// A release on no claim of another window manager keeps the window with
/// mc and ends the drag.
#[test]
fn a_release_on_no_claim_keeps_the_window_and_ends_the_drag() {
    let (mut mc, model) = dragging("acts-release-keep");
    mc.show("agenda", false).unwrap();
    mc.windows.insert("agenda".into(), "s1".into());
    mc.drag(DRAG, PressState::Pressed, [10.0, 10.0], None);
    assert_eq!(mc.dragged.as_deref(), Some("s1"));
    mc.drag(DRAG, PressState::Released, [10.0, 10.0], None);
    assert_eq!(mc.dragged, None);
    assert!(
        !model
            .lock()
            .unwrap()
            .calls
            .iter()
            .any(|c| c == "window.give")
    );
    assert!(mc.windows.contains_key("agenda"));
}

/// Next and previous walk the order of the scenes and wrap at the ends.
#[test]
fn next_and_previous_walk_the_scenes_and_wrap() {
    let (mut mc, _model) = testing::mc("acts-turn-walk", [1280, 800]);
    let order = mc.order();
    assert!(order.len() > 2, "the built-in scenes: {order:?}");
    mc.show(&order[0], false).unwrap();
    mc.perform(Action::Look(Look::NextScene));
    assert_eq!(mc.current.as_deref(), Some(order[1].as_str()));
    mc.perform(Action::Look(Look::PreviousScene));
    assert_eq!(mc.current.as_deref(), Some(order[0].as_str()));
    mc.perform(Action::Look(Look::PreviousScene));
    assert_eq!(mc.current.as_deref(), order.last().map(String::as_str));
    mc.perform(Action::Look(Look::NextScene));
    assert_eq!(mc.current.as_deref(), Some(order[0].as_str()));
}

/// A scene look shows the named scene.
#[test]
fn a_scene_look_shows_the_named_scene() {
    let (mut mc, _model) = testing::mc("acts-turn-scene", [1280, 800]);
    mc.perform(Action::Look(Look::Scene("mission".into())));
    assert_eq!(mc.current.as_deref(), Some("mission"));
}

/// A scene look of a scene that does not exist keeps the current scene.
#[test]
fn a_scene_look_of_a_missing_scene_keeps_the_current_one() {
    let (mut mc, _model) = testing::mc("acts-turn-missing", [1280, 800]);
    mc.show("agenda", false).unwrap();
    mc.perform(Action::Look(Look::Scene("no-such-scene".into())));
    assert_eq!(mc.current.as_deref(), Some("agenda"));
}

/// A toggle of a region hides the region that the scene shows, and a second
/// toggle shows it again.
#[test]
fn a_toggle_hides_a_shown_region_and_a_second_one_shows_it() {
    let (mut mc, _model) = testing::mc("acts-turn-toggle", [1280, 800]);
    mc.show("agenda", false).unwrap();
    mc.perform(Action::Look(Look::Toggle(Region::Right)));
    assert_eq!(mc.toggled.get(&Region::Right), Some(&false));
    mc.perform(Action::Look(Look::Toggle(Region::Right)));
    assert_eq!(mc.toggled.get(&Region::Right), Some(&true));
}

/// A solo of a content takes the region, and a second solo of it gives the
/// region back.
#[test]
fn a_solo_takes_the_region_and_a_second_one_gives_it_back() {
    let (mut mc, _model) = testing::mc("acts-turn-solo", [1280, 800]);
    mc.show("agenda", false).unwrap();
    let solo = || {
        Action::Look(Look::Solo {
            content: "notes".into(),
            region: Region::Right,
        })
    };
    mc.perform(solo());
    assert!(mc.solo.is_some(), "the solo holds the region");
    mc.perform(solo());
    assert!(mc.solo.is_none(), "the second solo gives it back");
}

/// The keys action flips the key list between its two states.
#[test]
fn the_keys_action_flips_the_key_list() {
    let (mut mc, _model) = testing::mc("acts-host-keys", [1280, 800]);
    let before = mc.keys;
    mc.keys().unwrap();
    assert_ne!(mc.keys, before);
    mc.keys().unwrap();
    assert_eq!(mc.keys, before);
}

/// The chat action asks the assistant to show its chat.
#[test]
fn the_chat_action_asks_the_assistant_to_show_its_chat() {
    let (mut mc, model) = testing::mc("acts-host-chat", [1280, 800]);
    mc.chat().unwrap();
    assert!(model.lock().unwrap().calls.iter().any(|c| c == "chat.show"));
}

/// A missing shared script is told in a notice of the notification server.
#[test]
fn a_missing_shared_script_is_told_in_a_notice() {
    let (mut mc, model) = testing::mc("acts-host-noscript", [1280, 800]);
    mc.failed(Failure::NoScript("agenda".into())).unwrap();
    assert!(
        model
            .lock()
            .unwrap()
            .calls
            .iter()
            .any(|c| c == "notify.show")
    );
}

/// A window that did not start is not told: the failure is silent in mc.
#[test]
fn a_window_that_did_not_start_is_not_told() {
    let (mut mc, model) = testing::mc("acts-host-notstarted", [1280, 800]);
    mc.failed(Failure::NotStarted("agenda".into())).unwrap();
    assert!(
        !model
            .lock()
            .unwrap()
            .calls
            .iter()
            .any(|c| c == "notify.show")
    );
}

/// The system action of a binding runs what the binding names.
#[test]
fn a_bound_system_action_runs_through_the_host() {
    let (mut mc, model) = testing::mc("acts-perform-chat", [1280, 800]);
    mc.perform(Action::System(System::Chat));
    assert!(model.lock().unwrap().calls.iter().any(|c| c == "chat.show"));
}
