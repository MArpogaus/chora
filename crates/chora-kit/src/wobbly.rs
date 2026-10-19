//! Wobbly windows, with the model and the default parameters of `KWin`'s
//! wobbly windows effect: a 4x4 grid of control points over the window
//! hangs on springs. Each point keeps its distance to its neighbours, and
//! a spring pulls the point under the pointer to its place in the window.
//! The pull lasts after the release, so the window settles where it is.
//! The drawn mesh is a finer grid through the Bézier surface of the
//! control points.

// The grids have at most 21 points a side, so `usize` to `f64` is exact.
#![allow(clippy::cast_precision_loss)]

use std::time::Instant;

use chora_scene::Mesh;

/// Control points along each side.
const SIDE: usize = 4;
const COUNT: usize = SIDE * SIDE;
/// Cells along each side of the drawn mesh.
const CELLS: u32 = 20;
/// `KWin`'s default level (`set_0`): spring stiffness, the share of the
/// velocity kept per step, and the scale from velocity to movement.
const STIFFNESS: f64 = 0.15;
const DRAG: f64 = 0.80;
const MOVE_FACTOR: f64 = 0.10;
/// The cap of velocity and acceleration, per axis.
const MAX: f64 = 1000.0;
/// The grid rests when the summed accelerations and velocities fall below
/// these.
const STOP_ACCELERATION: f64 = 0.5;
const STOP_VELOCITY: f64 = 0.5;
/// The integration step, in milliseconds.
const STEP_MS: f64 = 10.0;

/// The state of the grid of one window.
pub struct Wobble {
    /// The width and height of the window.
    size: [f64; 2],
    /// The control points in scene coordinates, row by row.
    at: [[f64; 2]; COUNT],
    velocity: [[f64; 2]; COUNT],
    /// The point pulled to its place: the one nearest to the grab.
    grab: usize,
    /// Whether the pointer still holds the window.
    held: bool,
    /// Whether the last step moved anything.
    moving: bool,
    /// The time of the last call of `advance`.
    last: Instant,
    /// The time since the last whole step, in milliseconds.
    pending: f64,
}

impl Wobble {
    /// A flat grid over a window of a size at an origin, grabbed at a point
    /// in scene coordinates.
    #[must_use]
    pub fn new(size: [f64; 2], origin: [f64; 2], grab: [f64; 2]) -> Self {
        let cell = |axis: usize| size[axis] / (SIDE - 1) as f64;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped
        let index = |axis: usize| {
            ((grab[axis] - origin[axis]) / cell(axis))
                .round()
                .clamp(0.0, (SIDE - 1) as f64) as usize
        };
        let mut wobble = Self {
            size,
            at: [[0.0; 2]; COUNT],
            velocity: [[0.0; 2]; COUNT],
            grab: index(1) * SIDE + index(0),
            held: true,
            moving: true,
            last: Instant::now(),
            pending: 0.0,
        };
        for i in 0..COUNT {
            wobble.at[i] = wobble.place(i, origin);
        }
        wobble
    }

    /// Lets go of the window; it shakes until it settles.
    pub fn release(&mut self) {
        self.held = false;
    }

    /// Advances the grid to a time, for a window at an origin: in each
    /// frame, and when the pointer moves the window, so that the drawn
    /// mesh lags the pointer by less than one step.
    pub fn advance(&mut self, origin: [f64; 2], now: Instant) {
        let dt = now.saturating_duration_since(self.last).as_secs_f64();
        self.last = now;
        self.step(origin, dt);
    }

    /// Advances the grid by a time in seconds, in whole steps of 10 ms, as
    /// in the `KWin` model. The rest waits for the next call. The drawn
    /// mesh moves the points on by it.
    fn step(&mut self, origin: [f64; 2], dt: f64) {
        self.pending += dt.clamp(0.0, 0.1) * 1000.0;
        while self.pending >= STEP_MS {
            self.pending -= STEP_MS;
            self.integrate(origin);
        }
    }

    /// Whether the grid rests, after the release.
    #[must_use]
    pub fn settled(&self) -> bool {
        !self.held && !self.moving
    }

    /// The largest distance of a control point from its place, in pixels.
    #[cfg(test)]
    pub fn bend(&self, origin: [f64; 2]) -> f64 {
        (0..COUNT)
            .map(|i| {
                let p = self.place(i, origin);
                (self.at[i][0] - p[0]).hypot(self.at[i][1] - p[1])
            })
            .fold(0.0, f64::max)
    }

    /// The drawn mesh in the node coordinates of a window at an origin.
    #[must_use]
    pub fn mesh(&self, origin: [f64; 2]) -> Mesh {
        let n = CELLS as usize;
        let mut points = Vec::with_capacity((n + 1) * (n + 1));
        for row in 0..=n {
            for col in 0..=n {
                let [x, y] = self.bezier(col as f64 / n as f64, row as f64 / n as f64);
                points.push([x - origin[0], y - origin[1]]);
            }
        }
        Mesh {
            cols: CELLS,
            rows: CELLS,
            points,
            depth: Vec::new(),
        }
    }

    /// The place of a control point in a window at an origin.
    fn place(&self, i: usize, origin: [f64; 2]) -> [f64; 2] {
        let (col, row) = (i % SIDE, i / SIDE);
        let last = (SIDE - 1) as f64;
        [
            origin[0] + self.size[0] * col as f64 / last,
            origin[1] + self.size[1] * row as f64 / last,
        ]
    }

    /// One step of 10 ms.
    fn integrate(&mut self, origin: [f64; 2]) {
        let ms = STEP_MS;
        let mut acceleration = [[0.0; 2]; COUNT];
        for (i, acc) in acceleration.iter_mut().enumerate() {
            let place = self.place(i, origin);
            if i == self.grab {
                *acc = [
                    (place[0] - self.at[i][0]) * STIFFNESS,
                    (place[1] - self.at[i][1]) * STIFFNESS,
                ];
                continue;
            }
            // Each neighbour pulls the point to keep the distance between
            // their places.
            let near = neighbours(i);
            for j in &near {
                let rest = self.place(*j, origin);
                for axis in [0, 1] {
                    let stretch =
                        (self.at[*j][axis] - self.at[i][axis]) - (rest[axis] - place[axis]);
                    acc[axis] += stretch * STIFFNESS;
                }
            }
            for value in acc.iter_mut() {
                *value /= near.len() as f64;
            }
        }
        let acceleration = smooth(&acceleration);
        let mut acc_sum = 0.0;
        for (velocity, acc) in self.velocity.iter_mut().zip(acceleration) {
            for axis in [0, 1] {
                let acc = acc[axis].clamp(-MAX, MAX);
                velocity[axis] = acc * ms + velocity[axis] * DRAG;
                acc_sum += acc.abs();
            }
        }
        self.velocity = smooth(&self.velocity);
        let mut vel_sum = 0.0;
        for (at, velocity) in self.at.iter_mut().zip(&mut self.velocity) {
            for axis in [0, 1] {
                velocity[axis] = velocity[axis].clamp(-MAX, MAX);
                at[axis] += velocity[axis] * ms * MOVE_FACTOR;
                vel_sum += velocity[axis].abs();
            }
        }
        self.moving = acc_sum >= STOP_ACCELERATION || vel_sum >= STOP_VELOCITY;
    }

    /// A point of the Bézier surface of the control points at (u, v) from
    /// 0 to 1.
    fn bezier(&self, u: f64, v: f64) -> [f64; 2] {
        let weights = |t: f64| {
            let s = 1.0 - t;
            [s * s * s, 3.0 * s * s * t, 3.0 * s * t * t, t * t * t]
        };
        let (wu, wv) = (weights(u), weights(v));
        let mut point = [0.0; 2];
        for (row, a) in wv.iter().enumerate() {
            for (col, b) in wu.iter().enumerate() {
                // Moved on by the time since the last step.
                let i = row * SIDE + col;
                let p = [0, 1].map(|axis| {
                    self.at[i][axis] + self.velocity[i][axis] * self.pending * MOVE_FACTOR
                });
                point[0] += a * b * p[0];
                point[1] += a * b * p[1];
            }
        }
        point
    }
}

/// The left, right, upper and lower neighbours of a control point.
fn neighbours(i: usize) -> Vec<usize> {
    let (col, row) = (i % SIDE, i / SIDE);
    let mut near = Vec::with_capacity(4);
    if col > 0 {
        near.push(i - 1);
    }
    if col + 1 < SIDE {
        near.push(i + 1);
    }
    if row > 0 {
        near.push(i - SIDE);
    }
    if row + 1 < SIDE {
        near.push(i + SIDE);
    }
    near
}

/// Blends each value with its ring of up to 8 neighbours; the value itself
/// weighs as much as all of them together, as in `KWin`.
fn smooth(data: &[[f64; 2]; COUNT]) -> [[f64; 2]; COUNT] {
    let mut out = [[0.0; 2]; COUNT];
    for (i, value) in out.iter_mut().enumerate() {
        #[allow(clippy::cast_possible_wrap)] // a 4x4 grid
        let (col, row) = ((i % SIDE) as isize, (i / SIDE) as isize);
        let mut sum = [0.0; 2];
        let mut count = 0.0;
        for dr in -1..=1 {
            for dc in -1..=1 {
                let (c, r) = (col + dc, row + dr);
                let side = SIDE.cast_signed();
                if (dr, dc) == (0, 0) || !(0..side).contains(&c) || !(0..side).contains(&r) {
                    continue;
                }
                #[allow(clippy::cast_sign_loss)] // checked above
                let j = (r * side + c) as usize;
                sum[0] += data[j][0];
                sum[1] += data[j][1];
                count += 1.0;
            }
        }
        for axis in [0, 1] {
            value[axis] = (sum[axis] + count * data[i][axis]) / (2.0 * count);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_drag_bends_the_far_side_more_than_the_grab_point() {
        let size = [400.0, 300.0];
        let mut w = Wobble::new(size, [0.0, 0.0], [200.0, 10.0]);
        // The window jumps 100 pixels to the right with the pointer.
        w.step([100.0, 0.0], 1.0 / 60.0);
        let grab_lag = 100.0 + w.place(w.grab, [0.0, 0.0])[0] - w.at[w.grab][0];
        let corner_lag = 100.0 + w.place(COUNT - 1, [0.0, 0.0])[0] - w.at[COUNT - 1][0];
        assert!(corner_lag > grab_lag + 10.0, "{corner_lag} vs {grab_lag}");
        assert!(w.bend([100.0, 0.0]) > 20.0);
        // The drawn mesh has the fine grid.
        let mesh = w.mesh([100.0, 0.0]);
        assert_eq!(mesh.points.len(), 21 * 21);
    }

    #[test]
    fn the_grid_settles_where_the_window_is() {
        let mut w = Wobble::new([400.0, 300.0], [0.0, 0.0], [200.0, 10.0]);
        w.step([80.0, 40.0], 1.0 / 60.0);
        w.release();
        let mut frames = 0;
        while !w.settled() {
            w.step([80.0, 40.0], 1.0 / 60.0);
            frames += 1;
            assert!(frames < 600, "still wobbling after 10 s");
        }
        assert!(frames > 10, "it settled at once");
        assert!(w.bend([80.0, 40.0]) < 2.0, "{}", w.bend([80.0, 40.0]));
    }

    #[test]
    fn a_flat_grid_maps_to_the_window() {
        let w = Wobble::new([400.0, 300.0], [10.0, 20.0], [0.0, 0.0]);
        let mesh = w.mesh([10.0, 20.0]);
        let last = mesh.points.len() - 1;
        assert!((mesh.points[0][0]).abs() < 1e-9 && (mesh.points[0][1]).abs() < 1e-9);
        assert!((mesh.points[last][0] - 400.0).abs() < 1e-9);
        assert!((mesh.points[last][1] - 300.0).abs() < 1e-9);
    }

    /// A drag of a window of 400 by 300, grabbed at the top middle, at
    /// 1200 pixels a second for half a second, with a move of the pointer
    /// every `ms` milliseconds; the largest bend and the drawn point under
    /// the pointer at the end.
    fn drag(ms: usize) -> (f64, f64) {
        let grab = [200.0, 0.0];
        let mut w = Wobble::new([400.0, 300.0], [0.0, 0.0], grab);
        let start = Instant::now();
        w.last = start;
        let mut bend: f64 = 0.0;
        let mut origin = [0.0, 0.0];
        for t in (ms..=500).step_by(ms) {
            origin = [1.2 * t as f64, 0.0];
            let at = std::time::Duration::from_millis(u64::try_from(t).unwrap_or(0));
            w.advance(origin, start + at);
            bend = bend.max(w.bend(origin));
        }
        let drawn = w.bezier(0.5, 0.0);
        (bend, origin[0] + grab[0] - drawn[0])
    }

    #[test]
    fn the_grid_moves_the_same_at_any_rate_of_the_pointer() {
        // Within one step of 10 ms, in which the window moves 12 pixels.
        let (steady, lag) = drag(10);
        for ms in [1, 2, 8] {
            let (bend, _) = drag(ms);
            assert!(
                (bend - steady).abs() < 13.0,
                "{bend} at {ms} ms, {steady} at 10 ms"
            );
        }
        // The drawn point under the pointer lags only a little.
        assert!(lag > 1.0 && lag < 15.0, "the drawn point lags {lag} pixels");
    }

    #[test]
    fn the_drawn_window_moves_evenly_from_frame_to_frame() {
        // A drag at 1200 pixels a second with a move every 4 ms, drawn
        // every 16 ms.
        let grab = [200.0, 0.0];
        let mut w = Wobble::new([400.0, 300.0], [0.0, 0.0], grab);
        let start = Instant::now();
        w.last = start;
        let mut drawn = Vec::new();
        for t in (4_u32..=496).step_by(4) {
            let origin = [1.2 * f64::from(t), 0.0];
            w.advance(
                origin,
                start + std::time::Duration::from_millis(u64::from(t)),
            );
            if t % 16 == 0 && t > 200 {
                drawn.push(w.bezier(0.5, 0.0)[0]);
            }
        }
        let moves: Vec<f64> = drawn.windows(2).map(|p| p[1] - p[0]).collect();
        let spread = moves.iter().copied().fold(f64::MIN, f64::max)
            - moves.iter().copied().fold(f64::MAX, f64::min);
        assert!(spread < 6.0, "the drawn window moves {moves:?}");
    }

    /// The mean lag of the drawn point under the pointer in a steady drag
    /// at 1200 pixels a second, with a move every `ms` milliseconds.
    fn lag(ms: usize) -> f64 {
        let grab = [200.0, 0.0];
        let mut w = Wobble::new([400.0, 300.0], [0.0, 0.0], grab);
        let start = Instant::now();
        w.last = start;
        let (mut sum, mut n) = (0.0, 0.0);
        for t in (ms..=600).step_by(ms) {
            let origin = [1.2 * t as f64, 0.0];
            let at = std::time::Duration::from_millis(u64::try_from(t).unwrap_or(0));
            w.advance(origin, start + at);
            if t > 300 {
                sum += origin[0] + grab[0] - w.bezier(0.5, 0.0)[0];
                n += 1.0;
            }
        }
        sum / n
    }

    #[test]
    fn the_drawn_point_lags_the_same_at_any_rate_of_the_pointer() {
        let (slow, fast) = (lag(8), lag(1));
        assert!(
            (fast - slow).abs() < 5.0,
            "lag {slow:.1} px at 125 Hz, {fast:.1} px at 1000 Hz"
        );
    }
}
