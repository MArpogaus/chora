//! The walk in pane steps (construct look v2, decision D8): a tap of a
//! walk key glides one pane step in 320 ms; a key held on accelerates
//! over 160 ms to one pane step per 400 ms, and the release stops it over
//! 160 ms. The distance only grows: nothing overshoots.
#![allow(dead_code, reason = "the walk keys use it in the next commit")]

/// One pane step in logical pixels.
pub const PANE_STEP: f64 = 800.0;
/// The glide of a tap, in milliseconds.
pub const TAP_MS: f64 = 320.0;
/// The time a held key takes to reach its pace, and the time the release
/// takes to stop it.
pub const RAMP_MS: f64 = 160.0;
/// The time of one pane step at the pace of a held key.
pub const HOLD_STEP_MS: f64 = 400.0;

/// How far a walk has gone `t` ms after the press, in pane steps, and
/// whether it stopped there; `released` is the time of the release.
pub fn steps_at(t: f64, released: Option<f64>) -> (f64, bool) {
    let _ = (t, released);
    (0.0, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn a_tap_glides_one_pane_step_in_320_ms_and_stops_there() {
        for release in [0.0, 100.0, 320.0] {
            let (end, done) = steps_at(TAP_MS, Some(release));
            assert!(near(end, 1.0) && done, "released at {release}");
            let (later, done) = steps_at(900.0, Some(release));
            assert!(near(later, 1.0) && done, "no overshoot after {release}");
        }
        assert!(near(steps_at(0.0, Some(0.0)).0, 0.0));
        let (half, done) = steps_at(160.0, Some(0.0));
        assert!(half > 0.5 && half < 1.0 && !done);
    }

    #[test]
    fn a_held_key_reaches_the_third_pane_in_1200_ms() {
        assert!(near(steps_at(TAP_MS, None).0, 1.0));
        assert!(near(steps_at(1200.0, None).0, 3.0));
        let pace = steps_at(1600.0, None).0 - steps_at(1200.0, None).0;
        assert!(near(pace, 1.0), "one pane step per 400 ms, not {pace}");
    }

    #[test]
    fn a_held_key_speeds_up_over_160_ms_and_never_goes_back() {
        let mut last = 0.0;
        for ms in (0..3000).step_by(8) {
            let (steps, _) = steps_at(f64::from(ms), None);
            assert!(steps >= last, "went back at {ms}");
            last = steps;
        }
        let accel = steps_at(TAP_MS + RAMP_MS, None).0 - 1.0;
        assert!(near(accel, 0.2), "accelerates over 0.2 steps, not {accel}");
    }

    #[test]
    fn the_release_of_a_held_key_stops_it_over_160_ms_without_overshoot() {
        let release = 1000.0;
        let (at_release, done) = steps_at(release, Some(release));
        assert!(!done);
        let (end, done) = steps_at(release + RAMP_MS, Some(release));
        assert!(done);
        assert!(near(end - at_release, 0.2), "stops within 0.2 steps");
        assert!(near(steps_at(release + 5000.0, Some(release)).0, end));
        let mut last = at_release;
        for ms in (0..=160).step_by(8) {
            let (steps, _) = steps_at(release + f64::from(ms), Some(release));
            assert!(steps >= last && steps <= end + 1e-9, "at {ms}");
            last = steps;
        }
    }

    #[test]
    fn a_release_while_the_pace_builds_stops_in_less_distance() {
        let release = TAP_MS + 80.0;
        let (at_release, _) = steps_at(release, Some(release));
        let (end, done) = steps_at(release + RAMP_MS, Some(release));
        assert!(done && end - at_release < 0.2 && end > at_release);
    }
}
