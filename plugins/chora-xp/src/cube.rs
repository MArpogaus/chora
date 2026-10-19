//! The desktop cube: four desktops on the side faces of a cube that turns
//! about its vertical axis, seen in perspective.
//!
//! Each face is a projective 4x4 transform of the scene: move the center of
//! the output to the origin, push the face back onto the cube, turn the
//! cube about the vertical axis, divide by the depth with a focal distance,
//! and move back to the center. z points away from the viewer. Windows
//! lift off their face toward the viewer and land again.

use std::{
    f64::consts::{FRAC_PI_2, PI},
    time::{Duration, Instant},
};

use chora_scene::{Easing, Transform};

use crate::theme::Rect;

/// The number of desktops, one per side face.
pub const DESKTOPS: usize = 4;
/// The focal distance as a share of the output width.
const FOCAL: f64 = 1.2;
/// How far the cube moves back half way, as a share of half the width.
const PUSH: f64 = 1.1;
/// How far windows fly off their face half way, in pixels.
const LIFT: f64 = 160.0;

/// The output that shows the cube.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    /// The center of the output in scene coordinates.
    pub center: [f64; 2],
    /// Half the width of the output: the distance of each face from the
    /// axis of the cube.
    pub half: f64,
    /// The focal distance of the perspective.
    pub focal: f64,
}

impl View {
    /// The view of a work area.
    pub fn of([x, y, w, h]: Rect) -> Self {
        let (w, h) = (f64::from(w), f64::from(h));
        Self {
            center: [f64::from(x) + w / 2.0, f64::from(y) + h / 2.0],
            half: w / 2.0,
            focal: w * FOCAL,
        }
    }
}

/// A running turn of the cube.
pub struct Turn {
    /// The desktop at the front when the turn started.
    pub from: usize,
    /// Quarter turns: positive to the right, to the next desktop.
    pub steps: i32,
    /// The output.
    pub view: View,
    start: Instant,
    duration: Duration,
    /// The angle at the start, after a turn by the pointer.
    begin: f64,
    /// The angle while the pointer turns the cube.
    held: Option<f64>,
    /// The pointer turned the cube: its phase follows the angle.
    by_pointer: bool,
}

/// A face that shows: its desktop, its turn in radians and its depth.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Face {
    /// The desktop.
    pub desktop: usize,
    /// The angle about the vertical axis; 0 faces the viewer.
    pub angle: f64,
    /// The depth of the center of the face; larger is farther.
    pub depth: f64,
}

impl Turn {
    /// A turn from a desktop by some quarter turns, starting now.
    pub fn new(from: usize, steps: i32, view: View, duration: Duration) -> Self {
        Self {
            from,
            steps,
            view,
            start: Instant::now(),
            duration,
            begin: 0.0,
            held: None,
            by_pointer: false,
        }
    }

    /// A turn that the pointer holds, from a desktop.
    pub fn held(from: usize, view: View) -> Self {
        Self {
            held: Some(0.0),
            by_pointer: true,
            ..Self::new(from, 0, view, Duration::ZERO)
        }
    }

    /// The pointer turned the cube by some pixels to the right: a drag
    /// over the width of the output is a quarter turn.
    pub fn hold(&mut self, dx: f64) {
        self.held = Some(-dx / (2.0 * self.view.half) * FRAC_PI_2);
    }

    /// The pointer lets go: the cube turns on to the nearest face, in the
    /// time of a quarter turn for each quarter.
    pub fn release(&mut self, quarter: Duration) {
        let Some(angle) = self.held.take() else {
            return;
        };
        #[allow(clippy::cast_possible_truncation)] // a few turns
        let steps = (angle / FRAC_PI_2).round() as i32;
        let rest = (f64::from(steps) * FRAC_PI_2 - angle).abs() / FRAC_PI_2;
        self.steps = steps;
        self.begin = angle;
        self.start = Instant::now();
        // At least a fifth of a quarter turn, so that a turn just short
        // of a face still shows.
        self.duration = quarter.mul_f64(rest.max(0.2));
    }

    /// Whether the turn reached its end at a time.
    pub fn done(&self, t: f64) -> bool {
        self.held.is_none() && t >= 1.0
    }

    /// The angle of the cube at a time.
    fn angle(&self, t: f64) -> f64 {
        self.held.unwrap_or_else(|| {
            let end = f64::from(self.steps) * FRAC_PI_2;
            self.begin + (end - self.begin) * Easing::EaseInOut.at(t)
        })
    }

    /// How far the cube is between its faces at a time, from 0 to 1: it
    /// moves back and its windows fly off by this share.
    pub fn phase(&self, t: f64) -> f64 {
        if self.by_pointer {
            (2.0 * self.angle(t)).sin().abs()
        } else {
            (PI * t).sin()
        }
    }

    /// The desktop at the front after the turn.
    pub fn target(&self) -> usize {
        offset(self.from, self.steps)
    }

    /// The time from 0 to 1.
    pub fn time(&self, now: Instant) -> f64 {
        let elapsed = now.saturating_duration_since(self.start).as_secs_f64();
        (elapsed / self.duration.as_secs_f64().max(1e-3)).min(1.0)
    }

    /// How far the cube moved back at a time.
    pub fn push(&self, t: f64) -> f64 {
        self.view.half * PUSH * self.phase(t)
    }

    /// How far a window flew off its face at a phase; higher windows of
    /// the stack fly farther.
    pub fn lift(phase: f64, index: usize, count: usize) -> f64 {
        #[allow(clippy::cast_precision_loss)] // a few windows
        let share = index as f64 / count.saturating_sub(1).max(1) as f64;
        LIFT * (0.6 + 0.4 * share) * phase
    }

    /// The faces that turn toward the viewer at a time, the farthest first.
    pub fn faces(&self, t: f64) -> Vec<Face> {
        let turned = self.angle(t);
        let mut faces: Vec<Face> = (0..DESKTOPS)
            .map(|desktop| {
                #[allow(clippy::cast_precision_loss)] // below 4
                let place = ((desktop + DESKTOPS - self.from) % DESKTOPS) as f64;
                let angle = turned - place * FRAC_PI_2;
                Face {
                    desktop,
                    angle,
                    depth: self.view.half * (1.0 - angle.cos()) + self.push(t),
                }
            })
            .filter(|f| f.angle.cos() > 0.01)
            .collect();
        faces.sort_by(|a, b| b.depth.total_cmp(&a.depth));
        faces
    }
}

/// The desktop some steps away, around the cube.
pub fn offset(desktop: usize, steps: i32) -> usize {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss
    )] // below 4
    let n = (desktop as i32 + steps).rem_euclid(DESKTOPS as i32) as usize;
    n
}

/// The transform of a face turned by an angle, with the cube pushed back
/// and a window lifted toward the viewer, in scene coordinates.
pub fn transform(view: &View, angle: f64, push: f64, lift: f64) -> Transform {
    let [cx, cy] = view.center;
    let h = view.half;
    Transform::translate(cx, cy, 0.0)
        .then(&perspective(view.focal))
        .then(&Transform::translate(0.0, 0.0, h + push))
        .then(&rotate_y(angle))
        .then(&Transform::translate(0.0, 0.0, -h - lift))
        .then(&Transform::translate(-cx, -cy, 0.0))
}

/// A turn about the vertical axis: a positive angle moves the front face to
/// the left.
pub fn rotate_y(angle: f64) -> Transform {
    let (sin, cos) = angle.sin_cos();
    let mut m = *Transform::IDENTITY.matrix();
    m[0] = cos;
    m[2] = -sin;
    m[8] = sin;
    m[10] = cos;
    Transform::from_matrix(m)
}

/// The perspective division by the depth: w = 1 + z / focal.
pub fn perspective(focal: f64) -> Transform {
    let mut m = *Transform::IDENTITY.matrix();
    m[11] = 1.0 / focal;
    Transform::from_matrix(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Where a transform puts a point of the plane z = 0, as the core does.
    fn project(transform: &Transform, px: f64, py: f64) -> [f64; 2] {
        let m = transform.matrix();
        let depth = m[3] * px + m[7] * py + m[15];
        [
            (m[0] * px + m[4] * py + m[12]) / depth,
            (m[1] * px + m[5] * py + m[13]) / depth,
        ]
    }

    fn view() -> View {
        View::of([0, 0, 1280, 800])
    }

    #[test]
    fn a_flat_face_changes_nothing() {
        let t = transform(&view(), 0.0, 0.0, 0.0);
        let p = project(&t, 100.0, 700.0);
        assert!(
            (p[0] - 100.0).abs() < 1e-9 && (p[1] - 700.0).abs() < 1e-9,
            "{p:?}"
        );
    }

    #[test]
    fn a_turned_face_shrinks_in_perspective() {
        let t = transform(&view(), 0.6, 300.0, 0.0);
        let [left_top, left_bottom] = [project(&t, 0.0, 0.0), project(&t, 0.0, 800.0)];
        let [right_top, right_bottom] = [project(&t, 1280.0, 0.0), project(&t, 1280.0, 800.0)];
        // The right edge comes closer, so it is longer than the left edge.
        assert!(right_bottom[1] - right_top[1] > left_bottom[1] - left_top[1]);
        assert!(right_top[0] < 1280.0 && left_top[0] > 0.0);
    }

    #[test]
    fn a_lifted_window_grows() {
        let flat = transform(&view(), 0.0, 0.0, 0.0);
        let lifted = transform(&view(), 0.0, 0.0, 100.0);
        let size = |t: &Transform| project(t, 1000.0, 0.0)[0] - project(t, 280.0, 0.0)[0];
        assert!(size(&lifted) > size(&flat));
    }

    #[test]
    fn a_turn_shows_two_faces_and_ends_at_the_neighbor() {
        let turn = Turn::new(3, 1, view(), Duration::from_millis(800));
        assert_eq!(turn.target(), 0);
        let faces = turn.faces(0.5);
        let desktops: Vec<usize> = faces.iter().map(|f| f.desktop).collect();
        assert_eq!(desktops.len(), 2, "{faces:?}");
        assert!(desktops.contains(&3) && desktops.contains(&0));
        let end = turn.faces(1.0);
        assert_eq!(end.len(), 1);
        assert_eq!(end[0].desktop, 0);
        assert!(end[0].angle.abs() < 1e-9);
        assert_eq!(offset(0, -1), 3);
    }

    #[test]
    fn a_turn_by_the_pointer_ends_at_the_nearest_face() {
        let mut turn = Turn::held(0, view());
        // Two thirds of the width to the left: the right face comes.
        turn.hold(-0.66 * 1280.0);
        let faces = turn.faces(0.0);
        assert!(faces.iter().any(|f| f.desktop == 1), "{faces:?}");
        assert!(!turn.done(1.0));
        turn.release(Duration::from_millis(800));
        assert_eq!(turn.target(), 1);
        let end = turn.faces(1.0);
        assert_eq!(end.len(), 1);
        assert_eq!(end[0].desktop, 1);
        // A short drag goes back.
        let mut back = Turn::held(2, view());
        back.hold(100.0);
        back.release(Duration::from_millis(800));
        assert_eq!(back.target(), 2);
    }

    #[test]
    fn a_click_without_a_drag_keeps_the_cube_still() {
        let mut turn = Turn::held(1, view());
        turn.release(Duration::from_millis(800));
        assert_eq!(turn.target(), 1);
        assert!(turn.phase(0.5).abs() < 1e-9, "{}", turn.phase(0.5));
    }
}
