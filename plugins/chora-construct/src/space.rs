//! The pure math of the space: the camera pose, the 4x4 matrix of a pane
//! for the core (D66, as the loader prototype computes it), the leash, the
//! place of a new pane, the fly-in pose and the ray of a click.
//!
//! The world has x to the right, y up and z forward at yaw 0. One unit is
//! one logical pixel. The eye and the centres of the panes sit at y = 0,
//! so a pane turns about the y axis only. A pane keeps its own yaw (F8):
//! it faces the camera where it was placed, moved or flown into, and
//! shows at an angle when the camera walks past. The ground lies at
//! y = -`Lens::ground`.
#![allow(clippy::many_single_char_names, reason = "the names of the math")]

/// A 4x4 matrix, row by row.
type Mat = [[f64; 4]; 4];

/// Where the camera stands and looks.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Pose {
    pub x: f64,
    pub z: f64,
    /// The turn about the y axis; 0 looks along +z.
    pub yaw: f64,
    /// Positive looks up.
    pub pitch: f64,
}

impl Pose {
    /// The forward direction on the ground, (x, z).
    #[must_use]
    pub fn forward(&self) -> [f64; 2] {
        [self.yaw.sin(), self.yaw.cos()]
    }

    /// The pose a share `k` from 0 to 1 of the way to `to`, the short way
    /// round for the yaw.
    #[must_use]
    pub fn lerp(&self, to: &Self, k: f64) -> Self {
        let turn = (to.yaw - self.yaw + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
            - std::f64::consts::PI;
        Self {
            x: self.x + (to.x - self.x) * k,
            z: self.z + (to.z - self.z) * k,
            yaw: self.yaw + turn * k,
            pitch: self.pitch + (to.pitch - self.pitch) * k,
        }
    }
}

/// The view of one output: its logical size and the focal length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lens {
    pub w: f64,
    pub h: f64,
    /// In logical pixels: a pane at this distance shows at 1:1.
    pub focal: f64,
    /// The height of the eye above the ground.
    pub ground: f64,
}

impl Lens {
    /// A lens with about 75 degrees of horizontal view, as the prototype.
    #[must_use]
    pub fn new(w: f64, h: f64) -> Self {
        Self {
            w,
            h,
            focal: w * 0.65,
            ground: h * 0.7,
        }
    }
}

fn mul(a: &Mat, b: &Mat) -> Mat {
    let mut m = [[0.0; 4]; 4];
    for (i, row) in m.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = (0..4).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    m
}

/// The yaw of a pane at `at` that faces `eye`: the yaw of a camera at
/// `eye` that looks at it.
#[must_use]
pub fn facing(eye: [f64; 2], at: [f64; 2]) -> f64 {
    (at[0] - eye[0]).atan2(at[1] - eye[1])
}

/// The right direction (x, z) of a pane with `yaw`, seen from its front.
fn right(yaw: f64) -> [f64; 2] {
    [yaw.cos(), -yaw.sin()]
}

/// The camera: world to camera space, with the pitch.
fn view(pose: &Pose) -> Mat {
    let (s, c) = pose.yaw.sin_cos();
    let yaw = [
        [c, 0.0, -s, -c * pose.x + s * pose.z],
        [0.0, 1.0, 0.0, 0.0],
        [s, 0.0, c, -s * pose.x - c * pose.z],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let (sp, cp) = pose.pitch.sin_cos();
    let pitch = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, cp, -sp, 0.0],
        [0.0, sp, cp, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    mul(&pitch, &yaw)
}

/// The model of a pane of `size` with its centre at `at` and its own
/// `yaw`: node point (u, v) to the world.
#[must_use]
fn model(at: [f64; 2], size: [f64; 2], yaw: f64) -> Mat {
    let r = right(yaw);
    let [w, h] = size;
    [
        [r[0], 0.0, 0.0, at[0] - r[0] * w / 2.0],
        [0.0, -1.0, 0.0, h / 2.0],
        [r[1], 0.0, 0.0, at[1] - r[1] * w / 2.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// The matrix for the `transform` of a pane node, column-major: the core
/// divides by w.
#[must_use]
pub fn pane_matrix(lens: &Lens, pose: &Pose, at: [f64; 2], size: [f64; 2], yaw: f64) -> [f64; 16] {
    to_screen(lens, pose, &model(at, size, yaw))
}

/// The matrix of the mirror image of a pane in the ground, for a node of
/// the pane's size divided by `k`: the node's top row is the pane's top
/// row, deepest below the ground.
#[must_use]
#[allow(clippy::too_many_arguments, reason = "the pane and its scale")]
pub fn mirror_matrix(
    lens: &Lens,
    pose: &Pose,
    at: [f64; 2],
    size: [f64; 2],
    yaw: f64,
    k: f64,
) -> [f64; 16] {
    let flip = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, -1.0, 0.0, -2.0 * lens.ground],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let scale = [
        [k, 0.0, 0.0, 0.0],
        [0.0, k, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let pane = model(at, size, yaw);
    to_screen(lens, pose, &mul(&flip, &mul(&pane, &scale)))
}

/// The matrix of the glow node behind a pane: a node of the pane's size
/// plus `reach` on each side, divided by `k`, so that its point (0, 0)
/// lies at (-reach, -reach) of the pane.
#[must_use]
#[allow(clippy::too_many_arguments, reason = "the pane and its glow")]
pub fn glow_matrix(
    lens: &Lens,
    pose: &Pose,
    at: [f64; 2],
    size: [f64; 2],
    yaw: f64,
    reach: f64,
    k: f64,
) -> [f64; 16] {
    let scale = [
        [k, 0.0, 0.0, -reach],
        [0.0, k, 0.0, -reach],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let pane = model(at, size, yaw);
    to_screen(lens, pose, &mul(&pane, &scale))
}

/// The projection of a model to the output, column-major.
fn to_screen(lens: &Lens, pose: &Pose, model: &Mat) -> [f64; 16] {
    let proj = [
        [lens.focal, 0.0, lens.w / 2.0, 0.0],
        [0.0, -lens.focal, lens.h / 2.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
    ];
    let m = mul(&proj, &mul(&view(pose), model));
    let mut out = [0.0; 16];
    for (c, column) in out.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        for (r, cell) in column.iter_mut().enumerate() {
            *cell = m[r][c];
        }
    }
    out
}

/// The point (u, v) of a node on the output, through the column-major
/// matrix of [`pane_matrix`].
#[must_use]
pub fn project(m: &[f64; 16], u: f64, v: f64) -> [f64; 2] {
    let x = m[0] * u + m[4] * v + m[12];
    let y = m[1] * u + m[5] * v + m[13];
    let w = m[3] * u + m[7] * v + m[15];
    [x / w, y / w]
}

/// The depth of a point in front of the camera, in camera space.
#[must_use]
pub fn depth(pose: &Pose, at: [f64; 2]) -> f64 {
    let v = view(pose);
    v[2][0] * at[0] + v[2][2] * at[1] + v[2][3]
}

/// Where a pane at `at` is for the camera: `shown` when its centre is on
/// the output, else `behind` the eye, `far` beyond the depth `far`, or
/// `aside` out of the view (F29).
#[must_use]
pub fn sight(lens: &Lens, pose: &Pose, at: [f64; 2], far: f64) -> &'static str {
    let v = view(pose);
    let cam = |r: usize| v[r][0] * at[0] + v[r][2] * at[1] + v[r][3];
    let d = cam(2);
    if d <= lens.focal * 0.1 {
        return "behind";
    }
    if d > far {
        return "far";
    }
    let x = lens.w / 2.0 + lens.focal * cam(0) / d;
    let y = lens.h / 2.0 - lens.focal * cam(1) / d;
    if (0.0..=lens.w).contains(&x) && (0.0..=lens.h).contains(&y) {
        "shown"
    } else {
        "aside"
    }
}

/// The leash: a step from `from`, slowed in the outer quarter of the
/// radius `r` when it leads outwards, and cut at `r`.
#[must_use]
pub fn leash(from: [f64; 2], step: [f64; 2], r: f64) -> [f64; 2] {
    let d = from[0].hypot(from[1]);
    let outwards = from[0] * step[0] + from[1] * step[1] > 0.0;
    let slow = if outwards {
        ((r - d) / (r * 0.25)).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let to = [from[0] + step[0] * slow, from[1] + step[1] * slow];
    let len = to[0].hypot(to[1]);
    if len > r {
        [to[0] * r / len, to[1] * r / len]
    } else {
        to
    }
}

/// The pose in front of the face of a pane at `at` with `yaw`, from
/// which it shows at 1:1 and centred: at the focal distance, level.
#[must_use]
pub fn fly_in(lens: &Lens, at: [f64; 2], yaw: f64) -> Pose {
    let (fx, fz) = yaw.sin_cos();
    Pose {
        x: at[0] - fx * lens.focal,
        z: at[1] - fz * lens.focal,
        yaw,
        pitch: 0.0,
    }
}

/// Whether the whole pane lies in front of the eye. A corner at or behind
/// it has `w <= 0`, and the core would stretch the quad over the output.
#[must_use]
pub fn in_front(lens: &Lens, pose: &Pose, at: [f64; 2], size: [f64; 2], yaw: f64) -> bool {
    screen_box(lens, pose, at, size, yaw).is_some()
}

/// The box of a pane on the output: left, top, right, bottom; `None`
/// when a corner lies at or behind the eye.
fn screen_box(
    lens: &Lens,
    pose: &Pose,
    at: [f64; 2],
    size: [f64; 2],
    yaw: f64,
) -> Option<[f64; 4]> {
    if depth(pose, at) <= lens.focal * 0.1 {
        return None;
    }
    let m = pane_matrix(lens, pose, at, size, yaw);
    let corners = [
        [0.0, 0.0],
        [size[0], 0.0],
        [0.0, size[1]],
        [size[0], size[1]],
    ];
    let mut b = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    for [u, v] in corners {
        let w = m[3] * u + m[7] * v + m[15];
        if w <= 0.0 {
            return None;
        }
        let [x, y] = project(&m, u, v);
        b = [b[0].min(x), b[1].min(y), b[2].max(x), b[3].max(y)];
    }
    Some(b)
}

/// The place of a new pane of `size` in the view direction (C7): ahead
/// at `distance`, else shifted sideways, nearer or farther, but always
/// with its whole face in the view, clear of the other panes on the
/// output and on the ground. `others` are the centres, sizes and yaws of
/// the panes; the new pane faces the camera. `None` when no spot in the
/// view is free.
#[must_use]
pub fn place(
    lens: &Lens,
    pose: &Pose,
    distance: f64,
    size: [f64; 2],
    others: &[([f64; 2], [f64; 2], f64)],
) -> Option<[f64; 2]> {
    let [fx, fz] = pose.forward();
    let boxes: Vec<[f64; 4]> = (others.iter())
        .filter_map(|(at, size, yaw)| screen_box(lens, pose, *at, *size, *yaw))
        .collect();
    let gap = 8.0;
    let free = |p: [f64; 2]| {
        let Some(b) = screen_box(lens, pose, p, size, facing([pose.x, pose.z], p)) else {
            return false;
        };
        let inside = b[0] >= 0.0 && b[1] >= 0.0 && b[2] <= lens.w && b[3] <= lens.h;
        let clear = boxes.iter().all(|o| {
            b[2] + gap <= o[0] || o[2] + gap <= b[0] || b[3] + gap <= o[1] || o[3] + gap <= b[1]
        });
        let apart = (others.iter())
            .all(|(q, s, _)| (p[0] - q[0]).hypot(p[1] - q[1]) >= f64::midpoint(size[0], s[0]));
        inside && clear && apart
    };
    // ponytail: a fixed grid of 7 depths and 17 shifts of 1/16 of the
    // output; a finer search if panes get packed tight.
    [1.0, 0.8, 1.25, 1.6, 2.0, 2.5, 3.0]
        .into_iter()
        .flat_map(|k| {
            (0..17_i32).map(move |i| {
                let lateral = f64::from((i + 1) / 2) * if i % 2 == 1 { 1.0 } else { -1.0 };
                (distance * k, lateral)
            })
        })
        .map(|(z, side)| {
            let shift = side * lens.w / 16.0 * z / lens.focal;
            [pose.x + fx * z + fz * shift, pose.z + fz * z - fx * shift]
        })
        .find(|&p| free(p))
}

/// The place of a pane of `width` that must come although no spot in
/// the view is free, such as a pane of a construct file: `distance`
/// ahead of the camera, or beside the panes in the way, right first,
/// then left. `others` are the centres and widths of the panes.
#[must_use]
pub fn beside(pose: &Pose, distance: f64, width: f64, others: &[([f64; 2], f64)]) -> [f64; 2] {
    let [fx, fz] = pose.forward();
    let ahead = [pose.x + fx * distance, pose.z + fz * distance];
    let gap = width * 0.1;
    let free = |p: [f64; 2]| {
        others
            .iter()
            .all(|(q, w)| (p[0] - q[0]).hypot(p[1] - q[1]) >= (width + w) / 2.0 + gap)
    };
    // ponytail: a fixed search of 16 slots; a crowded space past them
    // stacks the pane on the last slot.
    let mut spot = ahead;
    for k in 0..16_i32 {
        let side = f64::from((k + 1) / 2) * if k % 2 == 1 { 1.0 } else { -1.0 };
        let shift = side * (width + gap);
        spot = [ahead[0] + fz * shift, ahead[1] - fx * shift];
        if free(spot) {
            break;
        }
    }
    spot
}

/// Where the ray through the screen point `p` hits a pane of `size` at
/// `at` with `yaw`, from either side: its distance along the ray, or
/// `None`.
#[must_use]
#[allow(clippy::too_many_arguments, reason = "the ray and the pane")]
pub fn hit(
    lens: &Lens,
    pose: &Pose,
    p: [f64; 2],
    at: [f64; 2],
    size: [f64; 2],
    yaw: f64,
) -> Option<f64> {
    // The ray in camera space, then undo the pitch and the yaw.
    let (x, y) = (
        (p[0] - lens.w / 2.0) / lens.focal,
        -(p[1] - lens.h / 2.0) / lens.focal,
    );
    let (sp, cp) = pose.pitch.sin_cos();
    let (ry, rz) = (cp * y + sp, -sp * y + cp);
    let (s, c) = pose.yaw.sin_cos();
    let ray = [c * x + s * rz, ry, -s * x + c * rz];
    // The plane of the pane, its normal along its yaw.
    let eye = [pose.x, pose.z];
    let r = right(yaw);
    let n = [-r[1], r[0]];
    let denom = ray[0] * n[0] + ray[2] * n[1];
    if denom.abs() <= 1e-9 {
        return None;
    }
    let t = ((at[0] - eye[0]) * n[0] + (at[1] - eye[1]) * n[1]) / denom;
    let hit = [
        eye[0] + ray[0] * t - at[0],
        ray[1] * t,
        eye[1] + ray[2] * t - at[1],
    ];
    let u = hit[0] * r[0] + hit[2] * r[1];
    (t > 0.0 && u.abs() <= size[0] / 2.0 && hit[1].abs() <= size[1] / 2.0).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LENS: Lens = Lens {
        w: 1280.0,
        h: 800.0,
        focal: 832.0,
        ground: 560.0,
    };

    fn close(a: [f64; 2], b: [f64; 2]) -> bool {
        (a[0] - b[0]).abs() < 1e-6 && (a[1] - b[1]).abs() < 1e-6
    }

    /// A pane that faces the camera, from any side, shows upright and
    /// unmirrored, its left edge left of its right edge.
    #[test]
    fn a_pane_faces_the_camera() {
        let at = [300.0, 2000.0];
        for yaw_deg in [0.0, 70.0, 160.0, -120.0] {
            let yaw = f64::to_radians(yaw_deg);
            // A camera that stands back from the pane and looks at it.
            let pose = Pose {
                x: at[0] - yaw.sin() * 1500.0,
                z: at[1] - yaw.cos() * 1500.0,
                yaw,
                pitch: 0.0,
            };
            let face = facing([pose.x, pose.z], at);
            let m = pane_matrix(&LENS, &pose, at, [1280.0, 800.0], face);
            let [l, r] = [project(&m, 0.0, 400.0), project(&m, 1280.0, 400.0)];
            let [t, b] = [project(&m, 640.0, 0.0), project(&m, 640.0, 800.0)];
            assert!(
                l[0] < r[0] && t[1] < b[1],
                "{yaw_deg}: {l:?} {r:?} {t:?} {b:?}"
            );
            // Facing: both edges equally far, so equally tall on screen.
            assert!((l[0] + r[0] - 1280.0).abs() < 1e-6, "centred: {l:?} {r:?}");
            let scale = (r[0] - l[0]) / 1280.0;
            assert!((scale - 832.0 / 1500.0).abs() < 1e-9, "{scale}");
        }
    }

    /// From the fly-in pose, the pane of the output size covers the
    /// output at 1:1: the matrix maps each corner to itself, also for a
    /// pane that the camera sees at an angle before.
    #[test]
    fn the_fly_in_pose_shows_the_pane_at_one_to_one() {
        let at = [-900.0, 2400.0];
        let yaw = facing([100.0, -300.0], at) + 0.7;
        let pose = fly_in(&LENS, at, yaw);
        assert!(pose.pitch.abs() < f64::EPSILON);
        assert!((pose.x - at[0]).hypot(pose.z - at[1]) - 832.0 < 1e-9);
        let m = pane_matrix(&LENS, &pose, at, [1280.0, 800.0], yaw);
        for (u, v) in [(0.0, 0.0), (1280.0, 0.0), (0.0, 800.0), (1280.0, 800.0)] {
            assert!(close(project(&m, u, v), [u, v]), "{u} {v}");
        }
        // A smaller window shows centred at 1:1.
        let m = pane_matrix(&LENS, &pose, at, [800.0, 600.0], yaw);
        assert!(close(project(&m, 0.0, 0.0), [240.0, 100.0]));
        assert!(close(project(&m, 800.0, 600.0), [1040.0, 700.0]));
    }

    /// The mirror image hangs under the ground: the pane's bottom row
    /// shows as deep below the ground as the pane floats above it, the
    /// top row deeper still.
    #[test]
    fn the_mirror_image_hangs_under_the_ground() {
        let at = [0.0, 2000.0];
        let m = mirror_matrix(&LENS, &Pose::default(), at, [1280.0, 800.0], 0.0, 4.0);
        // The world point (0, y, 2000) shows at 400 - y * 832 / 2000.
        let screen = |y: f64| [640.0, 400.0 - y * 832.0 / 2000.0];
        assert!(close(project(&m, 160.0, 200.0), screen(-1120.0 + 400.0)));
        assert!(close(project(&m, 160.0, 0.0), screen(-1120.0 - 400.0)));
    }

    /// The label of a pane straight ahead sits centred under it.
    #[test]
    fn the_label_place_of_a_pane_ahead_is_centred_under_it() {
        let size = [1280.0, 800.0];
        let m = pane_matrix(&LENS, &Pose::default(), [0.0, 1664.0], size, 0.0);
        let [x, y] = project(&m, size[0] / 2.0, size[1]);
        assert!((x - 640.0).abs() < 1e-9, "{x}");
        assert!((y - 600.0).abs() < 1e-9, "{y}");
    }

    #[test]
    fn the_leash_slows_near_r_and_stops_at_r() {
        let r = 1000.0;
        assert_eq!(leash([0.0, 0.0], [0.0, 100.0], r), [0.0, 100.0]);
        // In the outer quarter, outwards: slower.
        let near = leash([0.0, 875.0], [0.0, 100.0], r);
        assert!((near[1] - 925.0).abs() < 1e-9, "{near:?}");
        // At R, outwards: no move; inwards: the full step.
        assert_eq!(leash([0.0, 1000.0], [0.0, 100.0], r), [0.0, 1000.0]);
        assert_eq!(leash([0.0, 1000.0], [0.0, -100.0], r), [0.0, 900.0]);
        // A step along the border never leaves the circle.
        let side = leash([0.0, 990.0], [300.0, 0.0], r);
        assert!(side[0].hypot(side[1]) <= r + 1e-9, "{side:?}");
        // No number of steps passes R.
        let mut p = [0.0, 0.0];
        for _ in 0..1000 {
            p = leash(p, [37.0, 59.0], r);
        }
        assert!(p[0].hypot(p[1]) <= r + 1e-9);
    }

    /// A new pane goes ahead if it is free, else to a spot in the view;
    /// with the view full, nowhere.
    #[test]
    fn a_new_pane_goes_into_the_view_or_nowhere() {
        let pose = Pose::default();
        let size = [640.0, 400.0];
        assert_eq!(place(&LENS, &pose, 2000.0, size, &[]), Some([0.0, 2000.0]));
        let first = [([0.0, 2000.0], size, 0.0)];
        let spot = place(&LENS, &pose, 2000.0, size, &first).unwrap();
        assert_ne!(spot, [0.0, 2000.0]);
        let b = screen_box(&LENS, &pose, spot, size, facing([0.0, 0.0], spot)).unwrap();
        assert!(
            b[0] >= 0.0 && b[2] <= LENS.w && b[1] >= 0.0 && b[3] <= LENS.h,
            "{b:?}"
        );
        let a = screen_box(&LENS, &pose, first[0].0, size, 0.0).unwrap();
        assert!(
            b[0] >= a[2] || b[2] <= a[0] || b[1] >= a[3] || b[3] <= a[1],
            "{a:?} {b:?}"
        );
        // A pane that fills the view leaves no room.
        let full = [([0.0, 800.0], [1400.0, 900.0], 0.0)];
        assert_eq!(place(&LENS, &pose, 2000.0, size, &full), None);
    }

    #[test]
    fn a_click_hits_the_pane_under_it() {
        let pose = Pose {
            x: 0.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
        };
        let at = [0.0, 1664.0]; // twice the focal length: half size
        let size = [1280.0, 800.0];
        assert!((hit(&LENS, &pose, [640.0, 400.0], at, size, 0.0).unwrap() - 1664.0).abs() < 1e-9);
        assert!(hit(&LENS, &pose, [330.0, 400.0], at, size, 0.0).is_some());
        assert!(hit(&LENS, &pose, [310.0, 400.0], at, size, 0.0).is_none());
        assert!(hit(&LENS, &pose, [640.0, 190.0], at, size, 0.0).is_none());
        let behind = Pose { yaw: 3.0, ..pose };
        assert!(hit(&LENS, &behind, [640.0, 400.0], at, size, 0.0).is_none());
        // A pane turned by 60 degrees shows about half as wide: its far
        // left edge lies at 520, so a click at 544 hits, one at 512
        // misses; facing, both hit.
        let turned = f64::to_radians(60.0);
        assert!(hit(&LENS, &pose, [544.0, 400.0], at, size, turned).is_some());
        assert!(hit(&LENS, &pose, [512.0, 400.0], at, size, turned).is_none());
        assert!(hit(&LENS, &pose, [512.0, 400.0], at, size, 0.0).is_some());
    }

    /// A pane keeps its yaw while the camera walks past it and looks at
    /// it: its matrix is the one of a fixed object, so the near edge
    /// shows taller than the far one.
    #[test]
    fn a_pane_does_not_turn_while_the_camera_walks() {
        let at = [0.0, 2000.0];
        let size = [1280.0, 800.0];
        let yaw = facing([0.0, 0.0], at);
        let past = Pose {
            x: 1500.0,
            yaw: facing([1500.0, 0.0], at),
            ..Pose::default()
        };
        let m = pane_matrix(&LENS, &past, at, size, yaw);
        let height = |u: f64| project(&m, u, size[1])[1] - project(&m, u, 0.0)[1];
        // The right edge is nearer to a camera on the right.
        assert!(
            height(size[0]) > height(0.0) * 1.1,
            "{} {}",
            height(0.0),
            height(size[0])
        );
    }

    /// Walking sideways past a pane puts its near corner level with or
    /// behind the eye while its centre is still ahead: it must not show.
    #[test]
    fn a_pane_with_a_corner_behind_the_eye_does_not_show() {
        let at = [0.0, 2000.0];
        let size = [1280.0, 800.0];
        let yaw = facing([0.0, 0.0], at);
        assert!(in_front(&LENS, &Pose::default(), at, size, yaw));
        let beside = Pose {
            x: 670.0,
            z: 1900.0,
            ..Pose::default()
        };
        assert!(depth(&beside, at) > LENS.focal * 0.1);
        assert!(!in_front(&LENS, &beside, at, size, 0.5));
    }

    #[test]
    fn the_sight_tells_where_a_pane_is() {
        let pose = Pose::default();
        assert_eq!(sight(&LENS, &pose, [0.0, 2000.0], 6000.0), "shown");
        assert_eq!(sight(&LENS, &pose, [0.0, -2000.0], 6000.0), "behind");
        assert_eq!(sight(&LENS, &pose, [0.0, 9000.0], 6000.0), "far");
        assert_eq!(sight(&LENS, &pose, [4000.0, 2000.0], 6000.0), "aside");
        let up = Pose { pitch: 1.0, ..pose };
        assert_eq!(sight(&LENS, &up, [0.0, 2000.0], 6000.0), "aside");
    }

    #[test]
    fn a_glide_turns_the_short_way() {
        let a = Pose {
            x: 0.0,
            z: 0.0,
            yaw: 3.0,
            pitch: 0.0,
        };
        let b = Pose { yaw: -3.0, ..a };
        let mid = a.lerp(&b, 0.5);
        assert!((mid.yaw - (3.0 + (std::f64::consts::TAU - 6.0) / 2.0)).abs() < 1e-9);
    }
}
