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
const KWIN: Level = Level {
    stiffness: 0.15,
    drag: 0.80,
    move_factor: 0.10,
};
/// The level of a sail: looser springs, so that a pane that swings
/// through a space bends far and still rests within a second.
const SAIL: Level = Level {
    stiffness: 0.045,
    drag: 0.91,
    move_factor: 0.10,
};
/// The depth of the bow of a sail per pixel that a point trails its
/// place, and the trail below which the bow starts softly.
const BOW: f64 = 1.6;
const BOW_SOFT: f64 = 12.0;
/// The cap of velocity and acceleration, per axis.
const MAX: f64 = 1000.0;
/// The grid rests when the summed accelerations and velocities fall below
/// these.
const STOP_ACCELERATION: f64 = 0.5;
const STOP_VELOCITY: f64 = 0.5;
/// The integration step, in milliseconds.
const STEP_MS: f64 = 10.0;

/// The springs of a grid.
#[derive(Clone, Copy)]
struct Level {
    stiffness: f64,
    /// The share of the velocity kept per step.
    drag: f64,
    /// The scale from velocity to movement.
    move_factor: f64,
}

/// The state of the grid of one window.
pub struct Wobble {
    level: Level,
    /// The depth of the bow per pixel of trail; 0 without a sail.
    bow: f64,
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
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let index = |axis: usize| {
            ((grab[axis] - origin[axis]) / cell(axis))
                .round()
                .clamp(0.0, (SIDE - 1) as f64) as usize
        };
        let mut wobble = Self {
            level: KWIN,
            bow: 0.0,
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
            last: crate::clock::now(),
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
