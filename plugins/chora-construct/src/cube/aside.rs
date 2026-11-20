//! Panes that give way to a cube that steps back (co-21). The cube steps
//! back away from the eye, and a pane that the person placed in front of
//! the face, as one pulled out of it, stays near the eye: it would cover
//! the cube and read as a face that juts out. Such a pane moves beside
//! the cube, at its depth, while the cube stands back, and goes home when
//! the cube steps in or closes. A pane that the person moves meanwhile
//! stays where they put it.

use std::f64::consts::SQRT_2;

use super::Host;
use crate::{ClientError, Command, Construct, Pose, space};

/// The gap between the cube and a pane beside it, in widths of a face.
const GAP: f64 = 0.15;

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

    /// The centre of a pane `width` wide beside the cube at `centre`, to
    /// the right for a positive `side`: clear of the cube in every turn.
    fn beside(&self, centre: [f64; 2], side: f64, width: f64) -> [f64; 2] {
        let (s, c) = self.yaw.sin_cos();
        let off = side.signum() * (self.cube.size[0] * (SQRT_2 / 2.0 + GAP) + width / 2.0);
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
    /// As the cube starts to step back: each pane that the person placed
    /// in front of it, and that would cover it once it stands back, gets
    /// a place beside it.
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
        let mut asides = Vec::new();
        for pane in self.panes.iter().filter(|p| p.placed) {
            if self.asides.contains_key(&pane.name)
                || space::depth(&pose, pane.at) >= depth
                || !crate::frame::overlaps(cube, self.screen_box_of(pane))
            {
                continue;
            }
            let side = if host.across(centre, pane.at) < 0.0 {
                -1.0
            } else {
                1.0
            };
            let to = host.beside(centre, side, pane.size[0]);
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
