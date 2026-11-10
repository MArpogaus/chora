//! Geometries of panes (D76): the slots of panes in one 3D context of
//! the core. The first is the cube: four side faces at 90 degrees around
//! a centre, with an optional top and bottom, a turn between the faces,
//! and the faces that look away from the eye, which the plugin hides
//! (D19).
//!
//! The space is the one of a 3D context: x to the right, y down and z
//! away from the eye, in logical pixels. The cube has its centre at the
//! origin. A transform maps the point (u, v, z) of a face node to the
//! space, as a 4x4 matrix in the column-major order of the core.
#![allow(clippy::many_single_char_names, reason = "the names of the math")]

use std::{
    f64::consts::{FRAC_PI_2, PI, TAU},
    time::{Duration, Instant},
};

/// A 4x4 matrix in column-major order, as `transform` of a node takes it.
pub type Matrix = [f64; 16];

/// A cube of faces of `size`: the side faces have this size, so the cube
/// is `size[0]` deep; the top and the bottom, with `caps`, are squares of
/// `size[0]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cube {
    /// The width and height of a side face.
    pub size: [f64; 2],
    /// Whether the cube has a top and a bottom face.
    pub caps: bool,
}

/// The index of the top face, with caps.
pub const TOP: usize = 4;
/// The index of the bottom face, with caps.
pub const BOTTOM: usize = 5;

impl Cube {
    /// The number of faces: four sides, and two caps with `caps`.
    #[must_use]
    pub fn faces(&self) -> usize {
        if self.caps { 6 } else { 4 }
    }

    /// The size of the node of face `i`.
    #[must_use]
    pub fn size_of(&self, i: usize) -> [f64; 2] {
        if i >= TOP {
            [self.size[0], self.size[0]]
        } else {
            self.size
        }
    }

    /// The transform of face `i` at `turn`, the angle of the cube about
    /// the y axis. At turn 0, face 0 looks at the eye, at z = -depth / 2;
    /// face `i` comes to the front at [`Cube::turn_of`]`(i)`.
    #[must_use]
    pub fn face(&self, i: usize, turn: f64) -> Matrix {
        let (local, _) = self.local(i);
        times(&yaw(turn), &local)
    }

    /// Whether face `i` at `turn` shows its front to an eye at `eye`.
    #[must_use]
    pub fn faces_eye(&self, i: usize, turn: f64, eye: [f64; 3]) -> bool {
        let m = self.face(i, turn);
        let [w, h] = self.size_of(i);
        let centre = [0, 1, 2].map(|r| m[r] * w / 2.0 + m[4 + r] * h / 2.0 + m[12 + r]);
        let (_, normal) = self.local(i);
        let y = yaw(turn);
        let normal =
            [0, 1, 2].map(|r| y[r] * normal[0] + y[4 + r] * normal[1] + y[8 + r] * normal[2]);
        (0..3)
            .map(|k| normal[k] * (eye[k] - centre[k]))
            .sum::<f64>()
            > 0.0
    }

    /// The turn that brings side face `i` to the front.
    #[must_use]
    pub fn turn_of(i: usize) -> f64 {
        #[allow(clippy::cast_precision_loss, reason = "a face index")]
        let i = i as f64;
        -i * FRAC_PI_2
    }

    /// The transform of face `i` at turn 0, and the way its front looks.
    fn local(&self, i: usize) -> (Matrix, [f64; 3]) {
        let [w, h] = self.size;
        let d = w / 2.0;
        match i {
            TOP => (
                columns(
                    [1.0, 0.0, 0.0],
                    [0.0, 0.0, -1.0],
                    [0.0, 1.0, 0.0],
                    [-d, -h / 2.0, d],
                ),
                [0.0, -1.0, 0.0],
            ),
            BOTTOM => (
                columns(
                    [1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0],
                    [0.0, -1.0, 0.0],
                    [-d, h / 2.0, -d],
                ),
                [0.0, 1.0, 0.0],
            ),
            side => {
                let a = -Self::turn_of(side % 4);
                let front = columns(
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [0.0, 0.0, 1.0],
                    [-d, -h / 2.0, -d],
                );
                (times(&yaw(a), &front), [-a.sin(), 0.0, -a.cos()])
            }
        }
    }
}

/// A matrix of the images of the x, y and z axis and of the origin.
fn columns(x: [f64; 3], y: [f64; 3], z: [f64; 3], t: [f64; 3]) -> Matrix {
    let mut m = [0.0; 16];
    for (c, column) in [x, y, z, t].into_iter().enumerate() {
        m[c * 4..c * 4 + 3].copy_from_slice(&column);
    }
    m[15] = 1.0;
    m
}

/// A turn about the y axis by `a`: the point (0, 0, -1) goes to
/// (-sin a, 0, -cos a).
fn yaw(a: f64) -> Matrix {
    let (s, c) = a.sin_cos();
    columns([c, 0.0, -s], [0.0, 1.0, 0.0], [s, 0.0, c], [0.0; 3])
}

/// The product `a * b` of two column-major matrices.
fn times(a: &Matrix, b: &Matrix) -> Matrix {
    let mut m = [0.0; 16];
    for c in 0..4 {
        for r in 0..4 {
            m[c * 4 + r] = (0..4).map(|k| a[k * 4 + r] * b[c * 4 + k]).sum();
        }
    }
    m
}

/// An eased turn of a cube from one angle to another, the short way.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Turn {
    from: f64,
    by: f64,
    start: Instant,
    duration: Duration,
}

impl Turn {
    /// A turn from `from` to `to` that starts at `start`.
    #[must_use]
    pub fn new(from: f64, to: f64, start: Instant, duration: Duration) -> Self {
        let by = (to - from + PI).rem_euclid(TAU) - PI;
        Self {
            from,
            by,
            start,
            duration,
        }
    }

    /// The angle at `now`, eased out.
    #[must_use]
    pub fn at(&self, now: Instant) -> f64 {
        let k = self.share(now);
        self.from + self.by * (1.0 - (1.0 - k).powi(3))
    }

    /// Whether the turn reached its end at `now`.
    #[must_use]
    pub fn done(&self, now: Instant) -> bool {
        self.share(now) >= 1.0
    }

    fn share(&self, now: Instant) -> f64 {
        let total = self.duration.as_secs_f64();
        if total <= 0.0 {
            return 1.0;
        }
        (now.saturating_duration_since(self.start).as_secs_f64() / total).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CUBE: Cube = Cube {
        size: [1280.0, 800.0],
        caps: false,
    };

    fn apply(m: &Matrix, [u, v, z]: [f64; 3]) -> [f64; 3] {
        [0, 1, 2].map(|r| m[r] * u + m[4 + r] * v + m[8 + r] * z + m[12 + r])
    }

    fn close(a: [f64; 3], b: [f64; 3]) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-6)
    }

    #[test]
    fn four_side_faces_ring_the_centre() {
        assert_eq!(CUBE.faces(), 4);
        assert_eq!(Cube { caps: true, ..CUBE }.faces(), 6);
        let d = 640.0;
        let centres = [[0.0, 0.0, -d], [-d, 0.0, 0.0], [0.0, 0.0, d], [d, 0.0, 0.0]];
        for (i, want) in centres.into_iter().enumerate() {
            let got = apply(&CUBE.face(i, 0.0), [640.0, 400.0, 0.0]);
            assert!(close(got, want), "face {i}: {got:?}");
        }
        let front = CUBE.face(0, 0.0);
        assert!(close(apply(&front, [0.0, 0.0, 0.0]), [-640.0, -400.0, -d]));
        assert!(close(
            apply(&front, [1280.0, 800.0, 0.0]),
            [640.0, 400.0, -d]
        ));
    }

    #[test]
    fn a_turn_of_a_face_brings_it_to_the_front() {
        for i in 0..4 {
            let at = CUBE.face(i, Cube::turn_of(i));
            let front = CUBE.face(0, 0.0);
            for p in [[0.0, 0.0, 0.0], [1280.0, 800.0, 0.0], [300.0, 100.0, 0.0]] {
                assert!(close(apply(&at, p), apply(&front, p)), "face {i}");
            }
        }
    }

    #[test]
    fn faces_that_look_away_do_not_face_the_eye() {
        let eye = [0.0, 0.0, -3000.0];
        let shown = |turn: f64| -> Vec<usize> {
            (0..4).filter(|&i| CUBE.faces_eye(i, turn, eye)).collect()
        };
        assert_eq!(shown(0.0), vec![0]);
        assert_eq!(shown(Cube::turn_of(1)), vec![1]);
        assert_eq!(shown(-FRAC_PI_2 / 3.0).len(), 2);
        let caps = Cube { caps: true, ..CUBE };
        assert_eq!(caps.size_of(TOP), [1280.0, 1280.0]);
        assert!(caps.faces_eye(TOP, 0.0, [0.0, -3000.0, -3000.0]));
        assert!(!caps.faces_eye(BOTTOM, 0.0, [0.0, -3000.0, -3000.0]));
        assert!(caps.faces_eye(BOTTOM, 0.0, [0.0, 3000.0, -3000.0]));
    }

    #[test]
    fn a_turn_goes_the_short_way_and_eases_out() {
        let start = Instant::now();
        let turn = Turn::new(0.1, TAU - 0.1, start, Duration::from_millis(400));
        assert!((turn.at(start) - 0.1).abs() < 1e-9);
        let end = turn.at(start + Duration::from_millis(400));
        assert!((end - (-0.1)).abs() < 1e-9, "{end}");
        let half = turn.at(start + Duration::from_millis(200));
        assert!(half < 0.0 && half > -0.1, "eased past the middle: {half}");
        assert!(!turn.done(start + Duration::from_millis(100)));
        assert!(turn.done(start + Duration::from_millis(400)));
    }
}
