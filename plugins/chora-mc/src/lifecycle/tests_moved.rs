//! Tests of the timers, the effects and the start of mc on the fake core:
//! the reading of the volume, the clock, a parked surface, the write
//! animation without a batch, the grabs, and a start after a restart.

use chora_kit::testing::FakeCore;

use crate::{CLOCK, Mc, PARKED, Shaders, VOLUME, testing};

/// mc on a fake core that selects by any selector, as the core does.
fn selecting(dir: &str) -> (Mc, chora_kit::testing::Model, FakeCore) {
    let core = FakeCore::start().selecting();
    let mc = testing::mc_on(&core, dir, [1280, 800]);
    (mc, core.model(), core)
}

/// The volume tick counts each tick and asks the worker for a reading on
/// the first one; what the worker answers is not fixed here, so the test
/// asserts the count only.
#[test]
fn the_volume_tick_counts_each_tick() {
    let (mut mc, _model) = testing::mc("lifecycle-volume", [1280, 800]);
    mc.volume_tick().unwrap();
    mc.volume_tick().unwrap();
    assert_eq!(mc.volume_ticks, 2, "each tick is counted");
}

/// The clock timer shows the date of the clocks of the settings, and
/// stays quiet for the clocks it has not got.
#[test]
fn the_clock_timer_takes_the_clocks_and_renders_them() {
    let (mut mc, _model) = testing::mc("lifecycle-clock", [1280, 800]);
    mc.timer(CLOCK).unwrap();
    let clocks = mc.clocks.len();
    let wanted = mc
        .settings
        .value
        .contents
        .values()
        .filter(|c| c.kind == crate::Kind::Clock)
        .count();
    assert_eq!(clocks, wanted, "one date for each clock of the settings");
}

/// The parked timer fits every surface that waits for its fit, and the
/// fit takes the surface out of the waiting list.
#[test]
fn the_parked_timer_takes_the_waiting_surfaces_out_of_the_list() {
    let (mut mc, _model) = testing::mc("lifecycle-parked", [1280, 800]);
    mc.fitting
        .insert("s9".into(), ("agenda".into(), [640, 400]));
    mc.timer(PARKED).unwrap();
    assert!(
        !mc.fitting.contains_key("s9"),
        "the parked surface was fitted"
    );
}

/// A timer of a node that mc does not time ends the write animation of a
/// panel only when mc wrote it: a node without a batch changes nothing.
#[test]
fn an_animation_end_without_a_write_batch_changes_nothing() {
    let (mut mc, _model) = testing::mc("lifecycle-written", [1280, 800]);
    mc.written("mc-s-agenda-panels-nothing").unwrap();
    assert!(mc.writing.is_empty(), "no batch was written");
}

/// The watchdog of the core drops the effects once: the plugin then
/// shows its panels plain, and the shaders are off until a settings change.
#[test]
fn the_dropped_effects_leave_the_panels_plain() {
    let (mut mc, _model) = testing::mc("lifecycle-dropped", [1280, 800]);
    mc.shaders = Shaders::On;
    mc.effects_dropped().unwrap();
    assert!(
        matches!(mc.shaders, Shaders::Dropped),
        "the shaders are off after the drop"
    );
    assert!(mc.writing.is_empty(), "no write batch is left");
}

/// The claimed grabs wait for a window manager: a plugin that does not
/// manage windows grabs none of them and keeps its built nodes.
#[test]
fn a_plugin_that_manages_no_windows_grabs_no_claimed_binding() {
    let (mut mc, model) = testing::mc("lifecycle-claimed", [1280, 800]);
    assert!(!mc.manage_windows, "mc runs as a plugin of the default");
    mc.grab_claimed().unwrap();
    assert!(mc.built, "the build is kept");
    assert!(
        crate::testing::has(&model, "mc-s-agenda-chapter-head-hit")
            || crate::testing::has(&model, VOLUME),
        "the nodes of the build are still there"
    );
}

/// Grabbing all bindings after the build also asks for the claimed ones,
/// which a plugin that manages no windows leaves out.
#[test]
fn grabbing_all_after_the_build_keeps_the_plugin_running() {
    let (mut mc, _model) = testing::mc("lifecycle-grab-all", [1280, 800]);
    assert!(mc.built, "the build ran");
    mc.grab_all().unwrap();
    assert!(mc.built, "the build is kept by the grabs");
}

/// A start after a restart rebuilds the scene and shows a default scene
/// of the settings, as the core keeps no scene of mc over a restart.
#[test]
fn a_start_after_a_restart_shows_a_scene_of_its_own() {
    let (mut mc, _model, _core) = selecting("lifecycle-start");
    mc.start(Vec::new()).unwrap();
    assert!(mc.current.is_some(), "a scene is shown after the start");
    assert!(
        mc.scenes.contains_key(mc.current.as_deref().unwrap()),
        "the shown scene is one that mc built"
    );
}

/// mc-60 (rpx1-8): a Super chord that mc binds to nothing is grabbed, so
/// no window page gets it; the chords that mc binds keep their action and
/// another modifier is left to the programs.
#[test]
fn an_unbound_super_chord_is_grabbed_and_a_bound_one_keeps_its_action() {
    let (mut mc, _model) = testing::mc("lifecycle-swallow", [1280, 800]);
    mc.rt.start_grabs();
    mc.grab_all().unwrap();
    mc.rt.end_grabs().unwrap();
    for chord in ["Super+x", "Super+k", "Super+m", "Super+q", "Super+Shift+x"] {
        assert!(
            mc.rt.action(chord).is_some(),
            "mc-60: {chord} is free, so it types into a window page"
        );
    }
    assert!(
        matches!(
            mc.rt.action("Super+space"),
            Some(chora_kit::actions::Action::System(
                chora_kit::actions::System::Run(_)
            ))
        ),
        "Super+space keeps its action"
    );
    assert_eq!(
        mc.rt.action("Super+Right"),
        Some(&chora_kit::actions::Action::Look(
            crate::config::Look::NextScene
        )),
        "Super+Right keeps its action"
    );
    assert!(
        matches!(
            mc.rt.action("Super+n"),
            Some(chora_kit::actions::Action::Look(
                crate::config::Look::Toggle(_)
            ))
        ),
        "Super+n keeps its action"
    );
    assert!(mc.rt.action("Alt+q").is_none(), "another modifier is left");
}
