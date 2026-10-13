//! Wobbly windows: the points of a grid over the window spring toward
//! their places in the window, which follows the pointer. The point under
//! the pointer stays under it; points far from it have weaker springs, so
//! they lag behind and the window bends. After the release all springs
//! are equal, and the window shakes until it settles.

use chora_scene::Mesh;

/// Cells along each side of the grid.
const CELLS: u32 = 8;
/// The spring constant at the grab point and far from it, in 1/s².
const NEAR: f64 = 1600.0;
const FAR: f64 = 110.0;
/// The distance over which the springs weaken, in pixels.
const FALLOFF: f64 = 260.0;
/// The spring constant after the release.
const LOOSE: f64 = 260.0;
/// The damping ratio: below 1 the window overshoots and shakes.
const DAMPING: f64 = 0.28;
/// A point is still when it is this close to its place, in pixels, and
/// slower than ten times that per second.
const STILL: f64 = 0.4;

/// The state of the grid of one window.
pub struct Wobble {
    /// The places of the points in the window, in node coordinates.
    rest: Vec<[f64; 2]>,
    /// The points in scene coordinates.
    at: Vec<[f64; 2]>,
    /// Their velocities in pixels per second.
    speed: Vec<[f64; 2]>,
    /// The spring constant of each point.
    spring: Vec<f64>,
    /// The point under the pointer while the button is down.
    grab: Option<usize>,
}

impl Wobble {
    /// A flat grid over a window of a size at an origin, grabbed at a point
    /// in scene coordinates.
    pub fn new(size: [f64; 2], origin: [f64; 2], grab: [f64; 2]) -> Self {
        let rest = Mesh::flat(CELLS, CELLS, size).points;
        let at: Vec<[f64; 2]> = rest.iter().map(|p| add(*p, origin)).collect();
        let local = sub(grab, origin);
        let nearest = (0..rest.len())
            .min_by(|a, b| distance(rest[*a], local).total_cmp(&distance(rest[*b], local)))
            .unwrap_or(0);
        let spring = rest
            .iter()
            .map(|p| FAR + (NEAR - FAR) * (-distance(*p, local) / FALLOFF).exp())
            .collect();
        Self {
            speed: vec![[0.0; 2]; rest.len()],
            rest,
            at,
            spring,
            grab: Some(nearest),
        }
    }

    /// Lets go of the grab point: all springs become equal.
    pub fn release(&mut self) {
        self.grab = None;
        self.spring.fill(LOOSE);
    }

    /// Moves the points by a time step in seconds, for a window at an
    /// origin. Short sub-steps keep the stiff springs stable.
    pub fn step(&mut self, origin: [f64; 2], dt: f64) {
        let dt = dt.clamp(0.0, 0.05);
        let steps = 8;
        let h = dt / f64::from(steps);
        for _ in 0..steps {
            for i in 0..self.at.len() {
                self.step_point(i, origin, h);
            }
        }
    }

    fn step_point(&mut self, i: usize, origin: [f64; 2], h: f64) {
        let place = add(self.rest[i], origin);
        if self.grab == Some(i) {
            self.at[i] = place;
            self.speed[i] = [0.0; 2];
            return;
        }
        let k = self.spring[i];
        let damping = 2.0 * DAMPING * k.sqrt();
        for axis in [0, 1] {
            let force = k * (place[axis] - self.at[i][axis]) - damping * self.speed[i][axis];
            self.speed[i][axis] += force * h;
            self.at[i][axis] += self.speed[i][axis] * h;
            // A huge jump of the window must not throw the grid away.
            self.at[i][axis] = self.at[i][axis].clamp(place[axis] - 600.0, place[axis] + 600.0);
        }
    }

    /// Whether every point rests at its place, after the release.
    pub fn settled(&self, origin: [f64; 2]) -> bool {
        self.grab.is_none()
            && self
                .at
                .iter()
                .zip(&self.rest)
                .zip(&self.speed)
                .all(|((a, r), v)| {
                    distance(*a, add(*r, origin)) < STILL && v[0].hypot(v[1]) < 10.0 * STILL
                })
    }

    /// The largest distance of a point from its place, in pixels.
    #[cfg(test)]
    pub fn bend(&self, origin: [f64; 2]) -> f64 {
        self.at
            .iter()
            .zip(&self.rest)
            .map(|(a, r)| distance(*a, add(*r, origin)))
            .fold(0.0, f64::max)
    }

    /// The grid in the node coordinates of a window at an origin.
    pub fn mesh(&self, origin: [f64; 2]) -> Mesh {
        Mesh {
            cols: CELLS,
            rows: CELLS,
            points: self.at.iter().map(|p| sub(*p, origin)).collect(),
            depth: Vec::new(),
        }
    }
}

fn add(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] + b[0], a[1] + b[1]]
}

fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::float_cmp)] // the grab point sits exactly at its place
    fn a_drag_bends_the_far_side_and_the_grab_point_follows() {
        let size = [400.0, 300.0];
        let mut w = Wobble::new(size, [0.0, 0.0], [200.0, 10.0]);
        // The window jumps 100 pixels to the right with the pointer.
        w.step([100.0, 0.0], 1.0 / 60.0);
        let mesh = w.mesh([100.0, 0.0]);
        let grab = w.grab.unwrap();
        assert_eq!(mesh.points[grab], w.rest[grab]);
        let corner = mesh.points.len() - 1;
        assert!(
            mesh.points[corner][0] < w.rest[corner][0] - 50.0,
            "{:?}",
            mesh.points[corner]
        );
        assert!(w.bend([100.0, 0.0]) > 50.0);
    }

    #[test]
    fn the_grid_settles_after_the_release() {
        let mut w = Wobble::new([400.0, 300.0], [0.0, 0.0], [200.0, 10.0]);
        w.step([80.0, 40.0], 1.0 / 60.0);
        w.release();
        let mut frames = 0;
        while !w.settled([80.0, 40.0]) {
            w.step([80.0, 40.0], 1.0 / 60.0);
            frames += 1;
            assert!(frames < 600, "still wobbling after 10 s");
        }
        assert!(frames > 10, "it settled at once");
    }
}
