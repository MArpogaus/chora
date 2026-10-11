//! The magic lamp: a minimized window narrows toward its task button and
//! flows into it, row by row, as a genie into its lamp. The plugin computes
//! the grid for each frame; restoring plays the frames backwards.

use std::time::{Duration, Instant};

use chora_scene::Mesh;

/// The grid: few columns, because each row stays a straight line, and many
/// rows for the curve of the sides.
const COLS: u32 = 4;
const ROWS: u32 = 24;
/// How far the bottom rows run ahead of the top rows, as a share of the
/// way.
const LAG: f64 = 0.6;
/// The share of the time in which the sides bend into the funnel.
const BEND: f64 = 0.4;

/// A running minimize or restore.
pub struct Lamp {
    /// The window, in scene coordinates: x, y, width and height.
    from: [f64; 4],
    /// The task button.
    to: [f64; 4],
    start: Instant,
    duration: Duration,
    /// Plays backwards: the window comes out of the button.
    pub restore: bool,
}

impl Lamp {
    /// A minimize, or with `restore` the way back, starting now.
    pub fn new(from: [f64; 4], to: [f64; 4], duration: Duration, restore: bool) -> Self {
        Self {
            from,
            to,
            start: Instant::now(),
            duration,
            restore,
        }
    }

    /// Turns a running lamp around where it is: a restore of a window that
    /// is half way into its button starts half way out.
    pub fn flip(&mut self, now: Instant) {
        let t = self.time(now);
        self.restore = !self.restore;
        self.start = now
            .checked_sub(self.duration.mul_f64(1.0 - t))
            .unwrap_or(now);
    }

    /// The time from 0 to 1.
    pub fn time(&self, now: Instant) -> f64 {
        let elapsed = now.saturating_duration_since(self.start).as_secs_f64();
        (elapsed / self.duration.as_secs_f64().max(1e-3)).min(1.0)
    }

    /// The grid at a time, in the node coordinates of the window.
    pub fn mesh(&self, time: f64) -> Mesh {
        let t = if self.restore { 1.0 - time } else { time };
        let [x, y, _, _] = self.from;
        let points = genie(self.from, self.to, t)
            .into_iter()
            .map(|[px, py]| [px - x, py - y])
            .collect();
        Mesh {
            cols: COLS,
            rows: ROWS,
            points,
        }
    }
}

/// The grid points of the genie in scene coordinates at a progress from 0
/// (the window) to 1 (inside the button).
pub fn genie(from: [f64; 4], to: [f64; 4], progress: f64) -> Vec<[f64; 2]> {
    let [left0, top0, width, height] = from;
    let [left1, top1, width1, height1] = to;
    let bend = smooth((progress / BEND).min(1.0));
    let flow = smooth(((progress - 0.15) / 0.85).clamp(0.0, 1.0));
    let funnel = (top1 - top0).abs().max(1.0);
    let mut points = Vec::with_capacity(((COLS + 1) * (ROWS + 1)) as usize);
    for row in 0..=ROWS {
        let down = f64::from(row) / f64::from(ROWS);
        // The bottom rows start first and reach the button first.
        let way = smooth((flow * (1.0 + LAG) - (1.0 - down) * LAG).clamp(0.0, 1.0));
        let start = top0 + down * height;
        let row_y = start + (top1 + down * height1 - start) * way;
        let narrow = smooth(((row_y - top0) / funnel).clamp(0.0, 1.0)) * bend;
        let left = left0 + (left1 - left0) * narrow;
        let right = left0 + width + (left1 + width1 - left0 - width) * narrow;
        for col in 0..=COLS {
            let across = f64::from(col) / f64::from(COLS);
            points.push([left + (right - left) * across, row_y]);
        }
    }
    points
}

/// A smooth step from 0 to 1.
fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOW: [f64; 4] = [100.0, 50.0, 600.0, 400.0];
    const BUTTON: [f64; 4] = [300.0, 773.0, 160.0, 25.0];

    #[test]
    #[allow(clippy::float_cmp)] // the corners are exact
    fn the_genie_starts_flat_and_ends_in_the_button() {
        let start = genie(WINDOW, BUTTON, 0.0);
        assert_eq!(start[0], [100.0, 50.0]);
        assert_eq!(*start.last().unwrap(), [700.0, 450.0]);
        for [x, y] in genie(WINDOW, BUTTON, 1.0) {
            assert!(
                (300.0..=460.0).contains(&x) && (773.0..=798.0).contains(&y),
                "{x} {y}"
            );
        }
    }

    #[test]
    fn half_way_the_window_narrows_toward_the_bottom() {
        let mid = genie(WINDOW, BUTTON, 0.5);
        let width = |row: usize| {
            let stride = (COLS + 1) as usize;
            mid[row * stride + COLS as usize][0] - mid[row * stride][0]
        };
        assert!(width(ROWS as usize) < width(0));
        assert!(width(ROWS as usize) < 400.0);
    }

    #[test]
    fn a_restore_plays_backwards() {
        let lamp = Lamp::new(WINDOW, BUTTON, Duration::from_millis(400), true);
        let mesh = lamp.mesh(1.0);
        let flat = Mesh::flat(COLS, ROWS, [600.0, 400.0]).points;
        for (a, b) in mesh.points.iter().zip(&flat) {
            assert!((a[0] - b[0]).abs() < 1e-6 && (a[1] - b[1]).abs() < 1e-6);
        }
    }
}
