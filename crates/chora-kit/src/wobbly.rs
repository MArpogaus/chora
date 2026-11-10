//! Wobbly windows, with the model and the default parameters of `KWin`'s
//! wobbly windows effect: a 4x4 grid of control points over the window
//! hangs on springs. Each point keeps its distance to its neighbours, and
//! a spring pulls the point under the pointer to its place in the window.
//! The pull lasts after the release, so the window settles where it is.
//! The drawn mesh is a finer grid through the Bézier surface of the
//! control points.
//!
//! The springs act on three axes. A window made with `Wobble::new` and
//! moved with `advance` stays in the plane: z is exactly 0 and the mesh
//! has no depth. A window made with `Wobble::new_3d` and moved with
//! `advance_3d` also has a z. The z of its place is the z of its origin
//! plus the part of its move across the screen that goes along the view
//! (`view[0] * x + view[1] * y`), so a drag toward the camera pulls the
//! window in depth, and so does a move of the window in the space. The
//! control points trail their places in z as they do in x and y. The
//! mesh goes out with a depth for each point, relative to the place of
//! the window and kept within `MAX`, in the z of `chora_scene`: the eye
//! looks along +z, so a smaller z is nearer.

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
    /// The origin of the window at the last step.
    origin: [f64; 2],
    /// The control points in scene coordinates, row by row.
    at: [[f64; 3]; COUNT],
    velocity: [[f64; 3]; COUNT],
    /// The z of the place of the window; 0 in 2D.
    lift: f64,
    /// The x and y of the direction of the view in scene coordinates.
    view: [f64; 2],
    /// Whether the mesh goes out with a depth.
    three_d: bool,
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
            origin,
            at: [[0.0; 3]; COUNT],
            velocity: [[0.0; 3]; COUNT],
            lift: 0.0,
            view: [0.0; 2],
            three_d: false,
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

    /// A flat grid over a window in 3D, with its origin and the grab in
    /// scene coordinates and the x and y of the direction of the view (the
    /// way the eye looks, for example `[0.0, 0.0]` for straight ahead). The
    /// grid draws with a depth. A non-finite value counts as 0.
    #[must_use]
    pub fn new_3d(size: [f64; 2], origin: [f64; 3], grab: [f64; 3], view: [f64; 2]) -> Self {
        let finite = |v: [f64; 3]| v.map(or_zero);
        let (origin, grab) = (finite(origin), finite(grab));
        let mut wobble = Self::new(size, [origin[0], origin[1]], [grab[0], grab[1]]);
        wobble.view = if view.iter().all(|v| v.is_finite()) {
            view
        } else {
            [0.0; 2]
        };
        wobble.three_d = true;
        wobble.lift = wobble.lift_of(origin);
        for point in &mut wobble.at {
            point[2] = wobble.lift;
        }
        wobble
    }

    /// Advances the grid as `advance` does, for a window at an origin in
    /// 3D. Use it with a grid of `new_3d`. A non-finite origin is ignored.
    pub fn advance_3d(&mut self, origin: [f64; 3], now: Instant) {
        if !origin.iter().all(|v| v.is_finite()) {
            return;
        }
        self.lift = self.lift_of(origin);
        self.advance([origin[0], origin[1]], now);
    }

    /// The z of the place of a window at an origin in 3D.
    fn lift_of(&self, origin: [f64; 3]) -> f64 {
        let lift = origin[2] + self.view[0] * origin[0] + self.view[1] * origin[1];
        or_zero(lift)
    }

    /// The springs of the grid as a value for another window manager
    /// that takes the window with its drag (`give_grab`): how far each
    /// control point is from its place for a window at an origin, and its
    /// velocity.
    #[must_use]
    pub fn springs(&self, origin: [f64; 2]) -> serde_json::Value {
        let offsets: Vec<[f64; 3]> = (0..COUNT)
            .map(|i| {
                let rest = self.place(i, origin);
                [0, 1, 2].map(|axis| self.at[i][axis] - rest[axis])
            })
            .collect();
        serde_json::json!({"offsets": offsets, "velocity": self.velocity.to_vec()})
    }

    /// Takes the springs that another window manager sent with a drag,
    /// for a window at an origin. The value comes from another client:
    /// anything but two lists of this grid with numbers within `MAX`
    /// leaves the grid flat.
    pub fn take_springs(&mut self, origin: [f64; 2], springs: &serde_json::Value) {
        let list = |name: &str| {
            let points: Vec<[f64; 3]> = serde_json::from_value(springs[name].clone()).ok()?;
            let sound = points.iter().flatten().all(|v| v.abs() <= MAX);
            (points.len() == COUNT && sound).then_some(points)
        };
        let (Some(offsets), Some(velocity)) = (list("offsets"), list("velocity")) else {
            return;
        };
        for i in 0..COUNT {
            let rest = self.place(i, origin);
            self.at[i] = [0, 1, 2].map(|axis| rest[axis] + offsets[i][axis]);
            self.velocity[i] = velocity[i];
        }
        self.moving = true;
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
        self.follow(origin);
        self.pending += dt.clamp(0.0, 0.1) * 1000.0;
        while self.pending >= STEP_MS {
            self.pending -= STEP_MS;
            self.integrate(origin);
        }
    }

    /// Takes the grid along when the window jumps farther than its size
    /// in one step, as to another output: the springs would else stretch
    /// the mesh over the way of the jump until they catch up.
    fn follow(&mut self, origin: [f64; 2]) {
        let jump = [origin[0] - self.origin[0], origin[1] - self.origin[1]];
        self.origin = origin;
        if jump[0].abs() <= self.size[0] && jump[1].abs() <= self.size[1] {
            return;
        }
        for point in &mut self.at {
            point[0] += jump[0];
            point[1] += jump[1];
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
    /// With a grid of `new_3d` it has a depth for each point.
    #[must_use]
    pub fn mesh(&self, origin: [f64; 2]) -> Mesh {
        let n = CELLS as usize;
        let mut points = Vec::with_capacity((n + 1) * (n + 1));
        let mut depth = Vec::new();
        for row in 0..=n {
            for col in 0..=n {
                let [x, y, z] = self.bezier(col as f64 / n as f64, row as f64 / n as f64);
                points.push([x - origin[0], y - origin[1]]);
                if self.three_d {
                    let z = (z - self.lift).clamp(-MAX, MAX);
                    depth.push(or_zero(z));
                }
            }
        }
        Mesh {
            cols: CELLS,
            rows: CELLS,
            points,
            depth,
        }
    }

    /// The place of a control point in a window at an origin.
    fn place(&self, i: usize, origin: [f64; 2]) -> [f64; 3] {
        let (col, row) = (i % SIDE, i / SIDE);
        let last = (SIDE - 1) as f64;
        [
            origin[0] + self.size[0] * col as f64 / last,
            origin[1] + self.size[1] * row as f64 / last,
            self.lift,
        ]
    }

    /// One step of 10 ms.
    fn integrate(&mut self, origin: [f64; 2]) {
        let ms = STEP_MS;
        let mut acceleration = [[0.0; 3]; COUNT];
        for (i, acc) in acceleration.iter_mut().enumerate() {
            let place = self.place(i, origin);
            if i == self.grab {
                *acc = [
                    (place[0] - self.at[i][0]) * STIFFNESS,
                    (place[1] - self.at[i][1]) * STIFFNESS,
                    (place[2] - self.at[i][2]) * STIFFNESS,
                ];
                continue;
            }
            // Each neighbour pulls the point to keep the distance between
            // their places.
            let near = neighbours(i);
            for j in &near {
                let rest = self.place(*j, origin);
                for axis in [0, 1, 2] {
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
            for axis in [0, 1, 2] {
                let acc = acc[axis].clamp(-MAX, MAX);
                velocity[axis] = acc * ms + velocity[axis] * DRAG;
                acc_sum += acc.abs();
            }
        }
        self.velocity = smooth(&self.velocity);
        let mut vel_sum = 0.0;
        for (at, velocity) in self.at.iter_mut().zip(&mut self.velocity) {
            for axis in [0, 1, 2] {
                velocity[axis] = velocity[axis].clamp(-MAX, MAX);
                at[axis] += velocity[axis] * ms * MOVE_FACTOR;
                vel_sum += velocity[axis].abs();
            }
        }
        self.moving = acc_sum >= STOP_ACCELERATION || vel_sum >= STOP_VELOCITY;
    }

    /// A point of the Bézier surface of the control points at (u, v) from
    /// 0 to 1.
    fn bezier(&self, u: f64, v: f64) -> [f64; 3] {
        let weights = |t: f64| {
            let s = 1.0 - t;
            [s * s * s, 3.0 * s * s * t, 3.0 * s * t * t, t * t * t]
        };
        let (wu, wv) = (weights(u), weights(v));
        let mut point = [0.0; 3];
        for (row, a) in wv.iter().enumerate() {
            for (col, b) in wu.iter().enumerate() {
                // Moved on by the time since the last step.
                let i = row * SIDE + col;
                let p = [0, 1, 2].map(|axis| {
                    self.at[i][axis] + self.velocity[i][axis] * self.pending * MOVE_FACTOR
                });
                point[0] += a * b * p[0];
                point[1] += a * b * p[1];
                point[2] += a * b * p[2];
            }
        }
        point
    }
}

/// The value, or 0 for a value that is not finite.
fn or_zero(value: f64) -> f64 {
    if value.is_finite() { value } else { 0.0 }
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
fn smooth(data: &[[f64; 3]; COUNT]) -> [[f64; 3]; COUNT] {
    let mut out = [[0.0; 3]; COUNT];
    for (i, value) in out.iter_mut().enumerate() {
        #[allow(clippy::cast_possible_wrap)] // a 4x4 grid
        let (col, row) = ((i % SIDE) as isize, (i / SIDE) as isize);
        let mut sum = [0.0; 3];
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
                sum[2] += data[j][2];
                count += 1.0;
            }
        }
        for axis in [0, 1, 2] {
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

    /// A window that jumps much farther than its size, as from one virtual
    /// output to another, takes its grid along: the mesh never spans the
    /// way of the jump.
    #[test]
    fn a_far_jump_takes_the_grid_along() {
        let size = [400.0, 300.0];
        let mut w = Wobble::new(size, [0.0, 0.0], [200.0, 10.0]);
        let far = [-524_288.0, 100.0];
        w.step(far, 1.0 / 60.0);
        assert!(w.bend(far) < 400.0, "the grid lags {} px", w.bend(far));
        let mesh = w.mesh(far);
        let widest = mesh.points.iter().map(|p| p[0].abs()).fold(0.0, f64::max);
        assert!(widest < 2.0 * size[0], "a point at {widest} px");
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

    /// A drag with a change of direction and a release, fed frame by
    /// frame; the hash of the bits of every drawn point of every frame.
    fn recorded_drag_hash() -> u64 {
        let mut w = Wobble::new([400.0, 300.0], [20.0, 30.0], [120.0, 40.0]);
        let start = Instant::now();
        w.last = start;
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for frame in 1_u32..=240 {
            let t = f64::from(frame) * 0.016;
            let origin = [
                20.0 + 700.0 * t.min(0.8) - 300.0 * (t - 0.8).max(0.0),
                30.0 + 90.0 * t.min(0.5),
            ];
            if frame == 120 {
                w.release();
            }
            let at = std::time::Duration::from_micros(u64::from(frame) * 16_000);
            w.advance(origin, start + at);
            for p in w.mesh(origin).points {
                for value in p {
                    hash = (hash ^ value.to_bits()).wrapping_mul(0x0100_0000_01b3);
                }
            }
        }
        hash
    }

    #[test]
    fn the_2d_drag_is_bit_identical_to_the_recorded_one() {
        assert_eq!(recorded_drag_hash(), 789_747_824_616_280_216);
    }

    const FRAME: std::time::Duration = std::time::Duration::from_millis(16);

    fn deepest(mesh: &Mesh) -> f64 {
        mesh.depth.iter().copied().fold(0.0, |a, z| a.max(z.abs()))
    }

    /// Drags a window of 400 by 300 left from x = 500 by 20 pixels a
    /// frame for 20 frames, releases it, and runs until it settles: the
    /// frames spent after the release. `frame` sees the grid and the
    /// origin after each frame of the drag. A grid of `new_3d` is moved
    /// with `advance_3d`, the other with `advance`.
    fn drag_left_and_settle(w: &mut Wobble, mut frame: impl FnMut(&Wobble, [f64; 2])) -> u32 {
        let mut now = w.last;
        let mut origin = [500.0, 0.0];
        let advance = |w: &mut Wobble, origin: [f64; 2], now| {
            if w.three_d {
                w.advance_3d([origin[0], origin[1], 0.0], now);
            } else {
                w.advance(origin, now);
            }
        };
        for _ in 0..20 {
            origin[0] -= 20.0;
            now += FRAME;
            advance(w, origin, now);
            frame(w, origin);
        }
        w.release();
        let mut frames = 0;
        while !w.settled() && frames < 600 {
            now += FRAME;
            advance(w, origin, now);
            frames += 1;
        }
        frames
    }

    /// Drags toward the camera along `-x` with the view `[0.6, 0]`: the
    /// mesh of the frame with the deepest bend, the mesh at rest, and the
    /// frames spent after the release.
    fn drag_along_the_view() -> (Mesh, Mesh, u32) {
        let mut w = Wobble::new_3d(
            [400.0, 300.0],
            [500.0, 0.0, 0.0],
            [700.0, 0.0, 0.0],
            [0.6, 0.0],
        );
        let mut bent = w.mesh([500.0, 0.0]);
        let frames = drag_left_and_settle(&mut w, |w, origin| {
            let mesh = w.mesh(origin);
            if deepest(&mesh) > deepest(&bent) {
                bent = mesh;
            }
        });
        let rest = w.mesh([100.0, 0.0]);
        (bent, rest, frames)
    }

    #[test]
    fn a_drag_toward_the_camera_bends_the_mesh_in_depth_and_it_settles_flat() {
        let (bent, rest, _) = drag_along_the_view();
        assert!(bent.is_valid());
        assert_eq!(bent.depth.len(), 21 * 21);
        assert!(
            bent.depth.iter().copied().fold(0.0, f64::max) > 5.0,
            "the trailing side stays behind: {}",
            deepest(&bent)
        );
        assert!(deepest(&rest) < 2.0, "{}", deepest(&rest));
    }

    #[test]
    fn the_window_own_motion_in_z_bends_the_mesh() {
        let mut w = Wobble::new_3d([400.0, 300.0], [0.0; 3], [200.0, 0.0, 0.0], [0.0, 0.0]);
        let mut now = w.last;
        for step in 1..=10 {
            now += FRAME;
            w.advance_3d([0.0, 0.0, -30.0 * f64::from(step)], now);
        }
        let mesh = w.mesh([0.0, 0.0]);
        assert!(deepest(&mesh) > 5.0, "{}", deepest(&mesh));
        assert!(
            mesh.points
                .iter()
                .all(|p| p[0].is_finite() && p[1].is_finite())
        );
    }

    #[test]
    fn the_depth_stays_bounded_under_extreme_velocity() {
        let mut w = Wobble::new_3d([400.0, 300.0], [0.0; 3], [200.0, 0.0, 0.0], [0.6, 0.0]);
        let mut now = w.last;
        for step in 1..=60 {
            now += FRAME;
            let jump = 1e9 * f64::from(step);
            w.advance_3d([jump, jump, -jump], now);
            let mesh = w.mesh([jump, jump]);
            assert!(mesh.depth.iter().all(|z| z.is_finite() && z.abs() <= MAX));
            assert_eq!(mesh.depth.len(), 21 * 21);
        }
    }

    #[test]
    fn a_wobble_in_3d_settles_as_fast_as_in_2d() {
        let mut flat = Wobble::new([400.0, 300.0], [500.0, 0.0], [700.0, 0.0]);
        let flat_frames = drag_left_and_settle(&mut flat, |_, _| {});
        let (_, _, frames) = drag_along_the_view();
        assert!(frames > 10 && frames < 600, "{frames}");
        assert!(
            frames <= flat_frames + 5,
            "{frames} in 3D, {flat_frames} in 2D"
        );
    }

    #[test]
    fn the_3d_entry_in_the_plane_equals_the_2d_one() {
        let mut flat = Wobble::new([400.0, 300.0], [20.0, 30.0], [120.0, 40.0]);
        let mut deep = Wobble::new_3d(
            [400.0, 300.0],
            [20.0, 30.0, 0.0],
            [120.0, 40.0, 0.0],
            [0.0; 2],
        );
        let mut now = flat.last;
        deep.last = now;
        for frame in 1..=30 {
            now += FRAME;
            let x = 20.0 + 15.0 * f64::from(frame);
            flat.advance([x, 30.0], now);
            deep.advance_3d([x, 30.0, 0.0], now);
        }
        let (a, b) = (flat.mesh([20.0, 30.0]), deep.mesh([20.0, 30.0]));
        assert_eq!(a.points, b.points);
        assert!(b.depth.iter().all(|z| *z == 0.0));
    }

    #[test]
    fn a_non_finite_origin_or_view_leaves_the_mesh_finite() {
        let mut w = Wobble::new_3d([400.0, 300.0], [0.0; 3], [200.0, 0.0, 0.0], [f64::NAN, 0.0]);
        let mut now = w.last;
        for origin in [
            [f64::INFINITY, 0.0, 0.0],
            [0.0, f64::NAN, 0.0],
            [0.0, 0.0, f64::NEG_INFINITY],
        ] {
            now += FRAME;
            w.advance_3d(origin, now);
            let mesh = w.mesh([0.0, 0.0]);
            assert!(mesh.depth.iter().all(|z| z.is_finite()));
            assert!(
                mesh.points
                    .iter()
                    .all(|p| p[0].is_finite() && p[1].is_finite())
            );
        }
    }

    /// The springs of a grid go to another window manager with a drag and
    /// come back the same; a value of another shape, or with numbers out
    /// of range, leaves the grid flat.
    #[test]
    fn the_springs_of_a_drag_go_to_another_grid() {
        let origin = [100.0, 50.0];
        let mut pulled = Wobble::new([200.0, 100.0], origin, [150.0, 60.0]);
        let later = pulled.last + std::time::Duration::from_millis(50);
        pulled.advance([160.0, 50.0], later);
        let springs = pulled.springs([160.0, 50.0]);
        let mut taken = Wobble::new([200.0, 100.0], [160.0, 50.0], [210.0, 60.0]);
        taken.take_springs([160.0, 50.0], &springs);
        assert_eq!(taken.at, pulled.at);
        assert_eq!(taken.velocity, pulled.velocity);
        let mut flat = Wobble::new([200.0, 100.0], origin, [150.0, 60.0]);
        let rest = flat.at;
        for bad in [
            serde_json::json!({"offsets": [[1.0, 0.0, 0.0]], "velocity": [[0.0, 0.0, 0.0]]}),
            serde_json::json!({"offsets": vec![[1e9, 0.0, 0.0]; COUNT],
                "velocity": vec![[0.0; 3]; COUNT]}),
            serde_json::json!("springs"),
        ] {
            flat.take_springs(origin, &bad);
            assert_eq!(flat.at, rest, "{bad}");
        }
    }
}
