//! The walk in pane steps (construct look v2, decision D8): a tap of a
//! walk key glides one pane step in 320 ms; a key held on accelerates
//! over 160 ms to one pane step per 400 ms, and the release stops it over
//! 160 ms. The distance only grows: nothing overshoots.

use std::time::Instant;

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
    match released {
        Some(r) if r <= TAP_MS => tap(t),
        Some(r) if t >= r => stop(t - r, r),
        _ => (held(t), false),
    }
}

fn tap(t: f64) -> (f64, bool) {
    let k = (t / TAP_MS).clamp(0.0, 1.0);
    (1.0 - (1.0 - k).powi(3), k >= 1.0)
}

fn pace() -> f64 {
    1.0 / HOLD_STEP_MS
}

fn held(t: f64) -> f64 {
    if t <= TAP_MS {
        return tap(t).0;
    }
    let u = t - TAP_MS;
    if u < RAMP_MS {
        1.0 + pace() * u * u / (2.0 * RAMP_MS)
    } else {
        1.0 + pace() * RAMP_MS / 2.0 + pace() * (u - RAMP_MS)
    }
}

fn stop(since: f64, released: f64) -> (f64, bool) {
    let speed = pace() * ((released - TAP_MS) / RAMP_MS).clamp(0.0, 1.0);
    let k = (since / RAMP_MS).clamp(0.0, 1.0);
    let reach = speed * RAMP_MS / 2.0;
    (held(released) + reach * (1.0 - (1.0 - k).powi(2)), k >= 1.0)
}

/// A walk of one key: where it began, which way, and when the key went
/// down and up.
pub struct Stride {
    pub key: String,
    dir: [f64; 2],
    from: [f64; 2],
    pressed: Instant,
    released: Option<Instant>,
}

impl Stride {
    /// A walk that begins at `from`, along the unit vector `dir`.
    pub fn new(key: &str, dir: [f64; 2], from: [f64; 2], pressed: Instant) -> Self {
        Self {
            key: key.to_owned(),
            dir,
            from,
            pressed,
            released: None,
        }
    }

    /// Whether the key is still down.
    pub fn held(&self) -> bool {
        self.released.is_none()
    }

    /// The key went up at `now`.
    pub fn release(&mut self, now: Instant) {
        self.released.get_or_insert(now);
    }

    /// Where the walk is at `now`, and whether it ended there.
    pub fn at(&self, now: Instant) -> ([f64; 2], bool) {
        let ms = |t: Instant| t.saturating_duration_since(self.pressed).as_secs_f64() * 1000.0;
        let (steps, done) = steps_at(ms(now), self.released.map(ms));
        let go = steps * PANE_STEP;
        (
            [
                self.from[0] + self.dir[0] * go,
                self.from[1] + self.dir[1] * go,
            ],
            done,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
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
    fn a_stride_walks_along_its_direction_from_where_it_began() {
        let t0 = Instant::now();
        let at = |ms: u64| t0 + std::time::Duration::from_millis(ms);
        let mut walk = Stride::new("w", [0.0, 1.0], [10.0, 20.0], t0);
        assert!(walk.held());
        let (third, done) = walk.at(at(1200));
        assert!(!done && near(third[0], 10.0) && near(third[1], 20.0 + 3.0 * PANE_STEP));
        walk.release(at(1200));
        walk.release(at(1250));
        assert!(!walk.held());
        let (end, done) = walk.at(at(1400));
        assert!(done && near(end[1], 20.0 + 3.2 * PANE_STEP));
    }

    #[test]
    fn a_release_while_the_pace_builds_stops_in_less_distance() {
        let release = TAP_MS + 80.0;
        let (at_release, _) = steps_at(release, Some(release));
        let (end, done) = steps_at(release + RAMP_MS, Some(release));
        assert!(done && end - at_release < 0.2 && end > at_release);
    }
}
