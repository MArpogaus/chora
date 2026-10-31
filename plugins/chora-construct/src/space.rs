//! The pure math of the space: the camera pose, the 4x4 matrix of a pane
//! for the core (D66, as the loader prototype computes it), the leash, the
//! place of a new pane, the fly-in pose and the ray of a click.
//!
//! The world has x to the right, y up and z forward at yaw 0. One unit is
//! one logical pixel. The eye and the centres of the panes sit at y = 0,
//! so a pane that faces the camera turns about the y axis only.
#![allow(clippy::many_single_char_names, reason = "the names of the math")]

/// A 4x4 matrix, row by row.
type Mat = [[f64; 4]; 4];

/// Where the camera stands and looks.
#[derive(Clone, Copy, Debug, PartialEq)]
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
}

impl Lens {
    /// A lens with about 75 degrees of horizontal view, as the prototype.
    #[must_use]
    pub fn new(w: f64, h: f64) -> Self {
        Self {
            w,
            h,
            focal: w * 0.65,
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

/// The right direction (x, z) of a camera that looks from `eye` at `at`.
fn right_towards(eye: [f64; 2], at: [f64; 2]) -> [f64; 2] {
    let yaw = (at[0] - eye[0]).atan2(at[1] - eye[1]);
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

/// The model of a pane of `size` with its centre at `at` that faces the
/// eye (billboard): node point (u, v) to the world.
#[must_use]
fn billboard(at: [f64; 2], size: [f64; 2], eye: [f64; 2]) -> Mat {
    let r = right_towards(eye, at);
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
pub fn pane_matrix(lens: &Lens, pose: &Pose, at: [f64; 2], size: [f64; 2]) -> [f64; 16] {
    let proj = [
        [lens.focal, 0.0, lens.w / 2.0, 0.0],
        [0.0, -lens.focal, lens.h / 2.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
    ];
    let m = mul(
        &proj,
        &mul(&view(pose), &billboard(at, size, [pose.x, pose.z])),
    );
    let mut out = [0.0; 16];
    for (c, column) in out.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        for (r, cell) in column.iter_mut().enumerate() {
            *cell = m[r][c];
        }
    }
    out
}

/// The depth of a point in front of the camera, in camera space.
#[must_use]
pub fn depth(pose: &Pose, at: [f64; 2]) -> f64 {
    let v = view(pose);
    v[2][0] * at[0] + v[2][2] * at[1] + v[2][3]
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

/// The pose in front of a pane at `at` from which it shows at 1:1 and
/// centred: at the focal distance, on the line from `eye`, level.
#[must_use]
pub fn fly_in(lens: &Lens, eye: [f64; 2], at: [f64; 2]) -> Pose {
    let (dx, dz) = (at[0] - eye[0], at[1] - eye[1]);
    let len = dx.hypot(dz).max(f64::EPSILON);
    let (fx, fz) = (dx / len, dz / len);
    Pose {
        x: at[0] - fx * lens.focal,
        z: at[1] - fz * lens.focal,
        yaw: fx.atan2(fz),
        pitch: 0.0,
    }
}

/// The place of a new pane of `width`: `distance` ahead of the camera,
/// or beside the panes in the way, right first, then left (C7). `others`
/// are the centres and widths of the panes.
#[must_use]
pub fn place(pose: &Pose, distance: f64, width: f64, others: &[([f64; 2], f64)]) -> [f64; 2] {
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
/// `at` that faces the camera: its distance along the ray, or `None`.
#[must_use]
pub fn hit(lens: &Lens, pose: &Pose, p: [f64; 2], at: [f64; 2], size: [f64; 2]) -> Option<f64> {
    // The ray in camera space, then undo the pitch and the yaw.
    let (x, y) = (
        (p[0] - lens.w / 2.0) / lens.focal,
        -(p[1] - lens.h / 2.0) / lens.focal,
    );
    let (sp, cp) = pose.pitch.sin_cos();
    let (ry, rz) = (cp * y + sp, -sp * y + cp);
    let (s, c) = pose.yaw.sin_cos();
    let ray = [c * x + s * rz, ry, -s * x + c * rz];
    // The plane of the pane: normal towards the eye.
    let eye = [pose.x, pose.z];
    let r = right_towards(eye, at);
    let n = [-r[1], r[0]]; // forward of the pane, away from the eye
    let denom = ray[0] * n[0] + ray[2] * n[1];
    if denom <= 1e-9 {
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
    };

    fn project(m: &[f64; 16], u: f64, v: f64) -> [f64; 2] {
        let x = m[0] * u + m[4] * v + m[12];
        let y = m[1] * u + m[5] * v + m[13];
        let w = m[3] * u + m[7] * v + m[15];
        [x / w, y / w]
    }

    fn close(a: [f64; 2], b: [f64; 2]) -> bool {
        (a[0] - b[0]).abs() < 1e-6 && (a[1] - b[1]).abs() < 1e-6
    }

    /// The billboard turns to the camera from any side: the pane shows
    /// upright and unmirrored, its left edge left of its right edge.
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
            let m = pane_matrix(&LENS, &pose, at, [1280.0, 800.0]);
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
    /// output at 1:1: the matrix maps each corner to itself.
    #[test]
    fn the_fly_in_pose_shows_the_pane_at_one_to_one() {
        let at = [-900.0, 2400.0];
        let eye = [100.0, -300.0];
        let pose = fly_in(&LENS, eye, at);
        assert!(pose.pitch.abs() < f64::EPSILON);
        assert!((pose.x - at[0]).hypot(pose.z - at[1]) - 832.0 < 1e-9);
        let m = pane_matrix(&LENS, &pose, at, [1280.0, 800.0]);
        for (u, v) in [(0.0, 0.0), (1280.0, 0.0), (0.0, 800.0), (1280.0, 800.0)] {
            assert!(close(project(&m, u, v), [u, v]), "{u} {v}");
        }
        // A smaller window shows centred at 1:1.
        let m = pane_matrix(&LENS, &pose, at, [800.0, 600.0]);
        assert!(close(project(&m, 0.0, 0.0), [240.0, 100.0]));
        assert!(close(project(&m, 800.0, 600.0), [1040.0, 700.0]));
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

    #[test]
    fn a_new_pane_goes_ahead_or_beside_the_panes_in_the_way() {
        let pose = Pose {
            x: 0.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
        };
        assert_eq!(place(&pose, 2000.0, 1000.0, &[]), [0.0, 2000.0]);
        let first = [([0.0, 2000.0], 1000.0)];
        let right = place(&pose, 2000.0, 1000.0, &first);
        assert!((right[0] - 1100.0).abs() < 1e-9 && (right[1] - 2000.0).abs() < 1e-9);
        let two = [first[0], (right, 1000.0)];
        let left = place(&pose, 2000.0, 1000.0, &two);
        assert!((left[0] + 1100.0).abs() < 1e-9, "{left:?}");
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
        assert!((hit(&LENS, &pose, [640.0, 400.0], at, size).unwrap() - 1664.0).abs() < 1e-9);
        assert!(hit(&LENS, &pose, [330.0, 400.0], at, size).is_some());
        assert!(hit(&LENS, &pose, [310.0, 400.0], at, size).is_none());
        assert!(hit(&LENS, &pose, [640.0, 190.0], at, size).is_none());
        let behind = Pose { yaw: 3.0, ..pose };
        assert!(hit(&LENS, &behind, [640.0, 400.0], at, size).is_none());
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
