//! The cube role of the construct (D71, `chora_kit::cube`): the listing
//! of the faces, a turn by quarters or to a face, and the event that
//! tells each change of the front face or the turning once.

use chora_kit::cube;
use serde_json::{Value, json};

use super::{ClientError, Construct, Health, Host, RpcError};

impl Host {
    /// The answer of `cube.list`.
    pub(crate) fn listing(&self) -> cube::Listing {
        let faces = (self.outputs.iter().enumerate())
            .map(|(i, output)| cube::Face {
                output: output.clone(),
                manager: super::MANAGER.to_owned(),
                health: match self.health_of(i) {
                    Health::Starting => "starting",
                    Health::Up => "up",
                    Health::Failed(_) => "failed",
                }
                .into(),
            })
            .collect();
        cube::Listing {
            faces,
            front: self.front,
            turning: self.turning(),
            mode: match self.view.mode() {
                chora_kit::modes::Mode::Use => "use",
                chora_kit::modes::Mode::Navigate => "navigate",
            }
            .into(),
        }
    }

    /// The data of the event of the cube role when the front face or
    /// the turning changed since it was told last.
    fn untold(&mut self) -> Option<cube::Turned> {
        let now = cube::Turned {
            front: self.front,
            turning: self.turning(),
        };
        (self.told != Some(now)).then(|| {
            self.told = Some(now);
            now
        })
    }
}

impl Host {
    /// The last data of the event of the cube role when the cube goes: at
    /// rest, unless that was told last.
    pub(crate) fn last_word(&self) -> Option<cube::Turned> {
        let rest = cube::Turned {
            front: self.front,
            turning: false,
        };
        (self.told.is_some() && self.told != Some(rest)).then_some(rest)
    }
}

impl Construct {
    /// Tells the end of a turn when the cube goes during one, so that a
    /// look that follows the event does not wait for it.
    pub(crate) fn tell_end(&mut self, host: &Host) {
        let Some(rest) = host.last_word() else {
            return;
        };
        let event = json!({"event": cube::event(crate::NAME), "data": rest});
        if let Err(err) = self.rt.conn.call::<_, Value>("events.emit", event) {
            eprintln!("chora-construct: {err}");
        }
    }

    /// `cube.turn` of the cube role: by quarter turns or to a face.
    pub(crate) fn turn_method(&mut self, turn: cube::Turn) -> Result<bool, RpcError> {
        let rejected = |e: ClientError| RpcError::new(RpcError::REJECTED, e.to_string());
        let Some(host) = &self.cube else {
            return Err(rejected(crate::modes::no_cube()));
        };
        let (front, faces) = host.front_face();
        let by = (turn.quarters(front, faces))
            .map_err(|e| RpcError::new(RpcError::INVALID_PARAMS, e))?;
        let taken = self.turn_cube(by, turn.ms, false).map_err(rejected)?;
        self.tell_cube();
        Ok(taken)
    }

    /// `cube.list` of the cube role; no faces while no cube is open.
    pub(crate) fn cube_listing(&self) -> cube::Listing {
        self.cube.as_ref().map_or_else(
            || cube::Listing {
                faces: Vec::new(),
                front: 0,
                turning: false,
                mode: "use".into(),
            },
            Host::listing,
        )
    }

    /// Emits the event of the cube role when the front face or the
    /// turning changed; a refusal of the core reaches the log only.
    pub(crate) fn tell_cube(&mut self) {
        let Some(turned) = self.cube.as_mut().and_then(Host::untold) else {
            return;
        };
        let event = json!({"event": cube::event(crate::NAME), "data": turned});
        if let Err(err) = self.rt.conn.call::<_, Value>("events.emit", event) {
            eprintln!("chora-construct: {err}");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::super::TURN;
    use super::*;

    /// The faces name the client of the manager, not its display name.
    #[test]
    fn the_role_lists_the_faces_with_their_manager_and_health() {
        let mut host = super::super::tests::host_of(4);
        host.name = "XP".into();
        host.program = "chora-xp".into();
        host.health[3] = Health::Starting;
        let listing = host.listing();
        assert_eq!(listing.faces.len(), 4);
        assert_eq!(listing.faces[0].output, "VIRTUAL-1");
        assert_eq!(listing.faces[0].manager, "xp");
        host.program = "chora-mc".into();
        let listing = host.listing();
        assert_eq!(listing.faces[0].manager, "mc", "the client of the setting");
        assert_eq!(listing.faces[3].health, "starting");
        assert_eq!((listing.front, listing.turning), (0, false));
        assert_eq!(listing.mode, "use");
        let _ = host.child.as_mut().map(std::process::Child::wait);
    }

    #[test]
    fn the_role_tells_each_change_of_the_front_and_the_turning_once() {
        let mut host = super::super::tests::host_of(4);
        let rest = cube::Turned {
            front: 0,
            turning: false,
        };
        assert_eq!(host.untold(), Some(rest), "the first state is told");
        assert_eq!(host.untold(), None, "an unchanged state is not");
        let now = Instant::now();
        assert!(host.start_turn(host.turns_to(3), TURN, false, now));
        let turning = cube::Turned {
            front: 3,
            turning: true,
        };
        assert_eq!(host.untold(), Some(turning), "a turn to face 3 goes left");
        assert_eq!(
            host.last_word(),
            Some(cube::Turned {
                front: 3,
                turning: false
            }),
            "a cube that goes during a turn tells its end"
        );
        host.turn = None;
        let done = cube::Turned {
            front: 3,
            turning: false,
        };
        assert_eq!(host.untold(), Some(done), "the end of the turn is told");
        assert_eq!(
            host.last_word(),
            None,
            "a cube at rest that goes tells nothing more"
        );
        let _ = host.child.as_mut().map(std::process::Child::wait);
    }
}
