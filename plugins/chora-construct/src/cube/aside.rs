//! Panes that give way to a cube that steps back (co-21, co-54). The cube
//! steps back away from the eye, and a pane in front of the face, as one
//! pulled out of it or a terminal that stood where the cube opened, stays
//! near the eye: it would cover the cube and read as a face that juts out. Such a pane moves beside
//! the cube, at its depth, while the cube stands back, and goes home when
//! the cube steps in or closes. A pane that the person moves meanwhile
//! stays where they put it.

use std::f64::consts::SQRT_2;

use super::Host;
use crate::{ClientError, Command, Construct, Pose, space};

/// The gap between the cube and a pane beside it, in widths of a face.
const GAP: f64 = 0.15;

/// How many steps a pane that is too wide for the room shrinks its offset
/// in, from the full offset to none.
const SHRINKS: u32 = 20;

/// A pane that gave way: its centre and yaw at home and beside the cube.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Aside {
    home: ([f64; 2], f64),
    beside: ([f64; 2], f64),
}

impl Aside {
    /// The centre and the yaw at `stand`, from home at 0 to beside the
    /// cube at 1.
    fn at(&self, stand: f64) -> ([f64; 2], f64) {
        let k = stand.clamp(0.0, 1.0);
        let ((home, home_yaw), (beside, beside_yaw)) = (self.home, self.beside);
        (
            [
                home[0] + (beside[0] - home[0]) * k,
                home[1] + (beside[1] - home[1]) * k,
            ],
            home_yaw + (beside_yaw - home_yaw) * k,
        )
    }
}

impl Host {
    /// The centre of the cube once it stands back, and the box that it
    /// covers on the screen of `lens` and `pose` then.
    fn stood(&mut self, lens: &space::Lens, pose: &Pose) -> ([f64; 2], Option<[f64; 4]>) {
        let now = (self.stand, self.shift, self.lean);
        self.stand = 1.0;
        self.aim(lens, pose);
        let stood = (self.centre(), self.screen_box(lens, pose));
        (self.stand, self.shift, self.lean) = now;
        stood
    }

    /// How far the inner edge of the first pane beside the cube lies from
    /// its centre, across the view: clear of the cube in every turn.
    fn clearance(&self) -> f64 {
        self.cube.size[0] * (SQRT_2 / 2.0 + GAP)
    }

    /// How far the centre of a pane `width` wide lies from the centre of
    /// the cube, across the view, with `taken` of the room beside the
    /// cube used by the panes there already.
    fn offset(&self, taken: f64, width: f64) -> f64 {
        self.clearance() + taken + width / 2.0
    }

    /// The point `off` to the right of `centre`, to the left for a
    /// negative `side`, across the view of the cube.
    fn beside(&self, centre: [f64; 2], side: f64, off: f64) -> [f64; 2] {
        let (s, c) = self.yaw.sin_cos();
        let off = side.signum() * off;
        [centre[0] + c * off, centre[1] - s * off]
    }

    /// How far `at` lies to the right of `centre`, across the view of the
    /// cube.
    fn across(&self, centre: [f64; 2], at: [f64; 2]) -> f64 {
        let (s, c) = self.yaw.sin_cos();
        (at[0] - centre[0]) * c - (at[1] - centre[1]) * s
    }
}

impl Construct {
    /// `to` inside the leash.
    fn leashed(&self, to: [f64; 2]) -> [f64; 2] {
        let (len, r) = (to[0].hypot(to[1]), self.radius());
        if len > r {
            [to[0] * r / len, to[1] * r / len]
        } else {
            to
        }
    }

    /// Whether a pane of `extent` at `to`, its face to the eye, shows
    /// between the left and the right edge of the screen.
    fn on_screen(&self, to: [f64; 2], extent: [f64; 2]) -> bool {
        let (lens, pose) = (self.lens, self.pose);
        let yaw = space::facing([pose.x, pose.z], to);
        space::screen_box(&lens, &pose, to, extent, yaw)
            .is_some_and(|b| b[0] >= 0.0 && b[2] <= lens.w)
    }

    /// The place of a pane of `extent` beside the cube at `centre`, on
    /// the side `sides[0]` (0 left, 1 right) or else on `sides[1]`, when
    /// the pane shows in full there: out from the cube past the room
    /// that `taken` holds on each side, and inside the leash. A pane that
    /// shows in full on neither side shrinks its offset on the first
    /// until it does. Answers the side and the place.
    fn place_beside(
        &self,
        host: &Host,
        centre: [f64; 2],
        (sides, taken): ([usize; 2], [f64; 2]),
        extent: [f64; 2],
    ) -> (usize, [f64; 2]) {
        let place = |k: usize, shrink: f64| {
            let off = host.offset(taken[k], extent[0]) * shrink;
            self.leashed(host.beside(centre, [-1.0, 1.0][k], off))
        };
        let whole = sides.into_iter().find_map(|k| {
            let to = place(k, 1.0);
            self.on_screen(to, extent).then_some((k, to))
        });
        whole.unwrap_or_else(|| {
            let k = sides[0];
            let shrunk = (0..=SHRINKS).map(|n| place(k, 1.0 - f64::from(n) / f64::from(SHRINKS)));
            (
                k,
                shrunk
                    .into_iter()
                    .find(|to| self.on_screen(*to, extent))
                    .unwrap_or_else(|| place(k, 1.0)),
            )
        })
    }

    /// As the cube starts to step back: each pane in front of it, placed
    /// by the person or not, that would cover it once it stands back, gets
    /// a place beside it, so that no window is lost behind the cube
    /// (co-54). The panes nearest the cube take the places nearest it,
    /// and no place overlaps another.
    pub(crate) fn make_way(&mut self) {
        let (lens, pose) = (self.lens, self.pose);
        let Some((centre, Some(cube))) = (self.cube.as_mut()).map(|h| h.stood(&lens, &pose)) else {
            return;
        };
        let Some(host) = self.cube.as_ref() else {
            return;
        };
        let depth = space::depth(&pose, centre);
        let eye = [pose.x, pose.z];
        let mut covering: Vec<(usize, f64)> = (self.panes.iter().enumerate())
            .filter(|(_, p)| !self.asides.contains_key(&p.name))
            .filter(|(_, p)| space::depth(&pose, p.at) < depth)
            .filter(|(_, p)| crate::frame::overlaps(cube, self.screen_box_of(p)))
            .map(|(i, p)| (i, host.across(centre, p.at)))
            .collect();
        covering.sort_by(|a, b| a.1.abs().total_cmp(&b.1.abs()));
        let mut taken = [0.0; 2];
        let mut asides = Vec::new();
        for (i, across) in covering {
            let pane = &self.panes[i];
            let sides = if across < 0.0 { [0, 1] } else { [1, 0] };
            let (k, to) = self.place_beside(host, centre, (sides, taken), pane.size);
            taken[k] += pane.size[0] + GAP * host.cube.size[0];
            let aside = Aside {
                home: (pane.at, pane.yaw),
                beside: (to, space::facing(eye, to)),
            };
            asides.push((pane.name.clone(), aside));
        }
        self.asides.extend(asides);
    }

    /// Moves the panes that gave way with the stand of the cube now; at
    /// the end of a step in, or without a cube, they are home and no
    /// longer aside.
    pub(crate) fn step_asides(&mut self) -> Vec<Command> {
        let stand = self.cube.as_ref().map_or(0.0, |h| h.stand);
        let home = self
            .cube
            .as_ref()
            .is_none_or(|h| h.step.is_none() && h.stand <= 0.0);
        let panes = &self.panes;
        self.asides
            .retain(|name, _| panes.iter().any(|p| p.name == *name));
        let mut commands = Vec::new();
        for (name, aside) in self.asides.clone() {
            let Some(i) = self.panes.iter().position(|p| p.name == name) else {
                continue;
            };
            (self.panes[i].at, self.panes[i].yaw) = aside.at(stand);
            commands.extend(self.placed(&self.panes[i]));
        }
        if home {
            self.asides.clear();
        }
        commands
    }

    /// Sends the panes that gave way to their places for the stand now.
    pub(crate) fn move_asides(&mut self) -> Result<(), ClientError> {
        let commands = self.step_asides();
        if commands.is_empty() {
            return Ok(());
        }
        self.rt.conn.apply(commands)
    }
}

#[cfg(test)]
mod tests {
    use super::Aside;

    #[test]
    fn a_pane_aside_goes_from_home_to_beside_the_cube_with_the_stand() {
        let aside = Aside {
            home: ([0.0, 900.0], 0.0),
            beside: ([2000.0, 3000.0], 0.5),
        };
        assert_eq!(aside.at(0.0), ([0.0, 900.0], 0.0));
        assert_eq!(aside.at(1.0), ([2000.0, 3000.0], 0.5));
        assert_eq!(aside.at(0.5), ([1000.0, 1950.0], 0.25));
        assert_eq!(aside.at(2.0), aside.at(1.0), "no further than beside");
    }
}
