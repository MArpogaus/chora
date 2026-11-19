//! Unit tests of the bar, the theme, the layout chip and the key list of
//! tiling, on the fake core of its tests: what it asks the core and when.

use std::time::{Duration, Instant};

use chora_kit::hud;
use serde_json::{Value, json};

use crate::{CHIP_TIME, NAME, Tiling, tests::on_fake_core, theme};

/// The number of requests of the method `method` that the fake core saw.
fn calls(methods: &chora_kit::testing::Log, method: &str) -> usize {
    let prefix = format!("{method} ");
    let seen = methods.lock().unwrap();
    seen.iter()
        .filter(|line| *line == method || line.starts_with(&prefix))
        .count()
}

/// Gives the current output and one window to the tiling.
fn one_window(tiling: &mut Tiling) {
    let area = [0, 0, 400, 300];
    tiling.wm.set_output("X", area, area);
    tiling.wm.add_window("s1");
}

/// The loop wakes at the earliest of its timers, and at none while
/// nothing runs.
#[test]
fn the_loop_wakes_at_the_earliest_timer_and_not_before() {
    let (mut tiling, _) = on_fake_core("overlay-wake");
    let now = Instant::now();
    assert_eq!(tiling.wake(now), None, "nothing runs on a fresh tiling");
    let chip = now + Duration::from_millis(700);
    let check = now + Duration::from_millis(300);
    tiling.chip_until = Some(chip);
    assert_eq!(tiling.wake(now), Some(chip));
    tiling.theme_check = Some(check);
    assert_eq!(tiling.wake(now), Some(check), "the earlier timer wins");
}

/// The bar is taken as told when the layout or the tags change, and not
/// otherwise.
#[test]
fn the_bar_state_changes_only_with_the_tags_or_the_layout() {
    let (mut tiling, _) = on_fake_core("overlay-bar");
    one_window(&mut tiling);
    assert!(tiling.bar_changed(), "the first state is new");
    assert!(!tiling.bar_changed(), "an unchanged state is not");
    tiling.wm.view_tag(2);
    assert!(tiling.bar_changed(), "a shown tag is a change");
    assert!(!tiling.bar_changed());
}

/// Pushing the bar records the state once; nothing is fullscreen, so
/// the bar is not hidden.
#[test]
fn pushing_the_bar_takes_the_state_once() {
    let (mut tiling, _) = on_fake_core("overlay-push");
    one_window(&mut tiling);
    tiling.push_bar();
    assert!(tiling.bar.is_some(), "the state is taken");
    assert!(
        !tiling.bar_changed(),
        "the panel was told the state just pushed"
    );
    tiling.push_bar();
    assert!(!tiling.bar_changed(), "an unchanged state stays taken");
    assert_eq!(tiling.bar_hidden, None, "no fullscreen, no hiding");
}

/// The host is named the theme of the scheme: once with the theme of the
/// flavor, once with the look.
#[test]
fn the_host_is_named_the_theme_of_the_scheme_twice() {
    let (mut tiling, methods) = on_fake_core("overlay-theme");
    tiling.name_theme().unwrap();
    let sent: Vec<Value> = methods
        .lock()
        .unwrap()
        .iter()
        .filter_map(|line| line.strip_prefix("events.emit "))
        .map(|params| serde_json::from_str(params).expect("the parameters"))
        .collect();
    assert_eq!(sent.len(), 2, "one event for the flavor, one for the look");
    assert_eq!(
        sent[0],
        hud::theme_event(NAME, true),
        "the scheme Ink is dark"
    );
    assert_eq!(sent[1], hud::look_theme_event(true));
    tiling.scheme = theme::Scheme::Paper;
    tiling.name_theme().unwrap();
    let last: Value = {
        let seen = methods.lock().unwrap();
        let line = seen.last().expect("the emit of the paper scheme");
        serde_json::from_str(line.strip_prefix("events.emit ").expect("an emit")).unwrap()
    };
    assert_eq!(
        last,
        hud::look_theme_event(false),
        "the paper scheme is light"
    );
}

/// The layout chip shows only with an output and a layout, for its time,
/// and goes when its time is up.
#[test]
fn the_layout_chip_shows_for_its_time_and_then_goes() {
    let (mut tiling, methods) = on_fake_core("overlay-chip");
    tiling.show_chip().unwrap();
    assert_eq!(tiling.chip_until, None, "no output, no chip");
    assert_eq!(calls(&methods, "scene.apply"), 0, "nothing is sent");
    one_window(&mut tiling);
    let before = Instant::now();
    tiling.show_chip().unwrap();
    let until = tiling.chip_until.expect("the chip shows");
    assert!(until >= before + CHIP_TIME, "the chip keeps its time");
    let shown = calls(&methods, "scene.apply");
    assert!(shown >= 1, "the chip is sent to the core");
    tiling.end_chip(before).unwrap();
    assert!(
        tiling.chip_until.is_some(),
        "the chip stays before its time"
    );
    assert_eq!(
        calls(&methods, "scene.apply"),
        shown,
        "nothing is removed early"
    );
    tiling.end_chip(until).unwrap();
    assert_eq!(tiling.chip_until, None, "the chip is gone at its time");
    assert_eq!(
        calls(&methods, "scene.apply"),
        shown + 1,
        "its removal is sent"
    );
}

/// A renderer that answers nothing leaves the chip at its guessed size,
/// and the key list without a fitted panel.
#[test]
fn without_a_renderer_the_key_list_is_not_fitted_and_no_text_is_measured() {
    let (mut tiling, _) = on_fake_core("overlay-fit");
    assert_eq!(format!("{:?}", tiling.key_list_fit([1280.0, 800.0])), "[]");
    assert_eq!(tiling.measured("chip-word"), None);
}

/// The key list opens on a screen, holds the keyboard, and closes again
/// on the second toggle.
#[test]
fn the_key_list_opens_on_a_screen_and_closes_on_the_next_toggle() {
    let (mut tiling, methods) = on_fake_core("overlay-keys");
    tiling.toggle_keys().unwrap();
    assert!(!tiling.keys_open(), "no screen, no key list");
    assert_eq!(calls(&methods, "scene.apply"), 0);
    one_window(&mut tiling);
    tiling.toggle_keys().unwrap();
    assert!(tiling.keys_open(), "the key list shows");
    let shown = calls(&methods, "scene.apply");
    assert!(shown >= 1, "the list is sent to the core");
    tiling.toggle_keys().unwrap();
    assert!(!tiling.keys_open(), "a second toggle hides it");
    assert_eq!(
        calls(&methods, "scene.apply"),
        shown + 1,
        "its removal is sent"
    );
}

/// The system is asked for its colour scheme only when the check is due,
/// and the answer is then awaited, not asked again.
#[test]
fn the_system_is_asked_only_when_the_check_is_due() {
    let (mut tiling, _) = on_fake_core("overlay-system");
    let now = Instant::now();
    tiling.follow_system(now).unwrap();
    assert!(tiling.theme_query.is_none(), "no check, no question");
    tiling.theme_check = Some(now + Duration::from_secs(10));
    tiling.follow_system(now).unwrap();
    assert!(
        tiling.theme_query.is_none(),
        "a check that is not due waits"
    );
    tiling.theme_check = Some(now);
    tiling.follow_system(now).unwrap();
    assert!(tiling.theme_query.is_some(), "a due check asks the system");
    assert_eq!(tiling.theme_check, Some(now + crate::ANSWER_CHECK));
}

/// A tiling on a fake core that answers each scene query with what
/// `answer` gives for its selector; the other requests get null.
fn on_scene(tag: &str, answer: fn(&str) -> Value) -> Tiling {
    let dir = std::env::temp_dir().join(format!("chora-tiling-moved-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let core = chora_kit::testing::FakeCore::start();
    core.respond(move |method, params| {
        (method == "scene.query").then(|| answer(params["selector"].as_str().unwrap_or_default()))
    });
    crate::tests::tiling_on(core.connect(NAME), &dir, &[])
}

/// The renderer's answer to the scene queries of the key list and of the
/// layout chip: a panel 200 wide, two words of which only one sizes the
/// panel, and the chip word of 64 by 24.
fn drawn(selector: &str) -> Value {
    match selector {
        "id:tiling-keys-panel" => json!([{
            "id": "tiling-keys-panel",
            "rects": {"a": [0.0, 0.0, 200.0, 100.0]},
        }]),
        "kind:text" => json!([
            {"id": "tiling-keys-row-l", "rects": {"a": [20.0, 0.0, 150.0, 20.0]}},
            {"id": "tiling-keys-row-x", "rects": {"a": [0.0, 0.0, 900.0, 20.0]}},
        ]),
        "id:tiling-chip-word" => json!([{
            "id": "tiling-chip-word",
            "rects": {"a": [0.0, 0.0, 64.0, 24.0]},
        }]),
        _ => json!([]),
    }
}

/// The key list is fitted to its widest word that sizes the panel: the
/// word ends 170 from the left of the panel, a word that does not size
/// the panel does not count.
#[test]
fn the_key_list_is_fitted_to_the_word_that_sizes_it() {
    let mut tiling = on_scene("fit", drawn);
    let size = [1280.0, 800.0];
    let fit = tiling.key_list_fit(size);
    assert!(!fit.is_empty(), "a drawn panel is fitted");
    assert_eq!(
        format!("{fit:?}"),
        format!("{:?}", crate::keys::fit_list(size, 100.0, 170.0)),
        "the word ends at 170, the other word is left out"
    );
}

/// The layout chip takes the size that the renderer gave its word, and a
/// word the renderer did not draw has no size.
#[test]
fn the_renderer_sizes_the_chip_word_and_names_no_missing_word() {
    let mut tiling = on_scene("measure", drawn);
    assert_eq!(tiling.measured("tiling-chip-word"), Some([64.0, 24.0]));
    assert_eq!(tiling.measured("tiling-chip-missing"), None);
}
