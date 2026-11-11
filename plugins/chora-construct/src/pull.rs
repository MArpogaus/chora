//! The pull-out (E3): a window that another window manager gives with
//! its drag, as `chora-xp` does when a title bar leaves a face of the
//! cube, becomes a pane under the pointer and goes on with the drag and
//! with the wobble, now with a depth. A pane that a drag drops over a
//! resting face of the cube goes back: its window goes to the window
//! manager whose claim holds the drop point on the output of that face,
//! which puts it there. So a
//! move from face to face stays inside that manager for the user.

use std::time::Duration;

use chora_kit::effects::Target;
use chora_scene::{
    GivenGrab, OutputInfo,
    ipc::{MethodCall, Query},
};
use serde_json::{Value, json};

use super::{
    BTN_LEFT, ClientError, Construct, Event, FLY_MS, Moving, NAME, PressState, cube, float, give,
    space,
};

/// How long the window manager of a face may take to place a window.
const PLACE_WAIT: Duration = Duration::from_secs(2);

/// The share of the depth of the front face at which a pulled pane
/// comes, toward the eye.
const PULL: f64 = 0.9;

/// The ids of the faces of the cube start with this.
const FACE: &str = "construct-cube-face-";

/// The face of the cube that shows at a point of a screen, among the
/// nodes of a query: its virtual output, and the point in the logical
/// pixels of that output. A pane that is dragged covers the face under
/// the pointer, so the core cannot name the face; the box of the face on
/// the screen maps to its output. A face counts while it forwards, so
/// not during a turn.
pub(crate) fn face_under(
    nodes: &[Value],
    screen: &str,
    at: [f64; 2],
    rests: bool,
) -> Option<(String, [f64; 2])> {
    let number = |value: &Value, i: usize| value.get(i).and_then(Value::as_f64);
    nodes.iter().find_map(|node| {
        let live = node["visible"] == true && (node["forward"] == true || rests);
        let face = node["id"].as_str()?.starts_with(FACE) && live;
        let area = node["rects"].get(screen).filter(|_| face)?;
        let [x, y, w, h] = [0, 1, 2, 3].map(|i| number(area, i));
        let (x, y, w, h) = (x?, y?, w?, h?);
        let inside = (x..x + w).contains(&at[0]) && (y..y + h).contains(&at[1]);
        let size = [number(&node["size"], 0)?, number(&node["size"], 1)?];
        let output = node["uniforms"]["win"]["output"].as_str()?;
        (inside && w > 0.0 && h > 0.0).then(|| {
            let point = [(at[0] - x) / w * size[0], (at[1] - y) / h * size[1]];
            (output.to_owned(), point)
        })
    })
}

/// Whether an output is a face of the cube: a virtual output that the
/// construct added. The release of a drag over a face names that output
/// and the point in it, since a face forwards the pointer (D69).
pub(crate) fn is_a_face(outputs: &[OutputInfo], output: &str) -> bool {
    let own = |o: &&OutputInfo| o.is_virtual == Some(true) && o.owner.as_deref() == Some(NAME);
    outputs.iter().filter(own).any(|o| o.name == output)
}

/// What an event does to the drag of a window pulled from a floating
/// plane, whose press stays on the plane: the event names the plane or a
/// face.
#[derive(Debug, PartialEq)]
enum PullStep {
    Move([f64; 2]),
    Drop([f64; 2]),
    /// Injected input that is not the hand of the person ends the drag in
    /// the space, and gives the window to no one.
    End,
}

fn pull_step(event: &Event, accepts_injected: bool) -> Option<PullStep> {
    let (node, position, injected, released) = match event {
        Event::PointerMoved {
            node,
            position,
            injected,
            ..
        } => (node, *position, *injected, false),
        Event::PointerReleased {
            node,
            button,
            position,
            injected,
            ..
        } if *button == BTN_LEFT => (node, *position, *injected, true),
        _ => return None,
    };
    cube::face_of(node)?;
    Some(match (injected && !accepts_injected, released) {
        (true, _) => PullStep::End,
        (false, false) => PullStep::Move(position),
        (false, true) => PullStep::Drop(position),
    })
}

/// The depth of a pulled pane for the front face at `face`, by a lens of
/// `focal`: a little in front of the face, else at 1:1; never nearer than
/// a quarter of the focal length, and never a depth that is no number.
fn pull_depth(face: Option<f64>, focal: f64) -> f64 {
    let d = face.map_or(focal, |d| d * PULL);
    if d.is_finite() {
        d.max(focal * 0.25)
    } else {
        focal
    }
}

impl Construct {
    /// Takes the press of `move` that comes with a window and its drag,
    /// and the release of a moved pane that the space would not take: a
    /// drop over a face of the cube, and a release while the camera is in
    /// a window or on its way into one, which must still end the drag. A
    /// press of `move` without a drag, of a window that draws its own
    /// title bar, moves no pane while the camera is in a window. Any
    /// other event goes back to the caller.
    pub(crate) fn pulled(&mut self, event: Event) -> Result<Option<Event>, ClientError> {
        let released = match &event {
            Event::InputButton {
                state: PressState::Released,
                position,
                output,
                local,
                ..
            } => Some((output.clone(), *local, self.local(*position))),
            _ => None,
        };
        if let Some((output, point, at)) = released {
            if self.drop_on_a_face(&output, point)? {
                return Ok(None);
            }
            if self.moving.is_some() && self.inside().is_some() {
                return self.release(at, None).map(|()| None);
            }
        }
        match event {
            Event::InputButton {
                binding,
                state: PressState::Pressed,
                position,
                surface: Some(surface),
                grab: Some(grab),
                ..
            } if binding == give::MOVE => {
                let at = self.local(position);
                self.take_pulled(&surface, at, &grab).map(|()| None)
            }
            Event::InputButton {
                binding,
                state: PressState::Pressed,
                grab: None,
                ..
            } if binding == give::MOVE && self.inside().is_some() => Ok(None),
            event => Ok(Some(event)),
        }
    }

    /// The drop of a moved pane of an own window over a resting face of
    /// the cube: the release names the output of the face and the point
    /// in it, or, under the dragged pane, the screen and the point there.
    /// The front face counts first, as the box of a face on the screen
    /// holds its whole quad only face on.
    /// The window goes to the window manager whose claim holds that point
    /// of the output of the face, as a drop on a screen does (D55), and
    /// that manager is asked to put it at the drop point. Returns whether
    /// the window went; else the drop of the space takes the pane.
    #[allow(clippy::cast_possible_truncation, reason = "a point of an output")]
    fn drop_on_a_face(&mut self, output: &str, point: [f64; 2]) -> Result<bool, ClientError> {
        let dragged = self.moving.as_ref().and_then(|moving| {
            let pane = self.panes.iter().find(|p| p.name == moving.name)?;
            pane.window().filter(|_| pane.attached).map(String::from)
        });
        let Some(surface) = dragged else {
            return Ok(false);
        };
        let outputs = self.rt.outputs()?;
        let face = if is_a_face(&outputs, output) {
            Some((output.to_owned(), point))
        } else {
            let shaders = Query {
                selector: format!("owner:{NAME} kind:shader"),
                filter: None,
            };
            let mut nodes: Vec<Value> = self.rt.conn.call("scene.query", shaders)?;
            let front = self.cube.as_ref().map(cube::Host::front_output);
            nodes
                .sort_by_key(|node| node["uniforms"]["win"]["output"].as_str() != front.as_deref());
            let rests = self.cube.as_ref().is_some_and(cube::Host::rests);
            face_under(&nodes, output, point, rests)
        };
        let Some((output, point)) = face else {
            return Ok(false);
        };
        let Some(corner) = outputs.iter().find(|o| o.name == output) else {
            return Ok(false);
        };
        let [x, y] = corner.position.map(f64::from);
        let holder = (self.rt.conn).claimant_at([x + point[0], y + point[1]]);
        let Some(to) = holder.map_err(ClientError::Rpc)? else {
            return Ok(false);
        };
        if let Err(err) = self.give(&surface, &to) {
            eprintln!("chora-construct: {surface} stays a pane: {err}");
            return Ok(false);
        }
        if let Some(moving) = self.moving.take() {
            self.effects.release(&moving.name);
        }
        let at = point.map(|v| v.round() as i32);
        let place = MethodCall {
            client: to.clone(),
            method: "window.move".into(),
            params: json!({"surface": surface, "output": output, "at": at}),
        };
        let placed: Result<Value, _> =
            (self.rt.conn).call_within("methods.call", place, PLACE_WAIT);
        if let Err(err) = placed {
            eprintln!(
                "chora-construct: {surface} is a window of {to}, not at the drop point: {err}"
            );
        }
        Ok(true)
    }

    /// Pulls the window of a floating plane out of the stood-back cube
    /// (K12): its window manager gives it to the construct, which goes
    /// on with the drag of the person, held at the point of the window
    /// under the pointer. A refused give leaves the window on its face.
    pub(crate) fn pull_plane(&mut self, surface: &str, at: [f64; 2]) -> Result<(), ClientError> {
        let Some(screen) = self.output.as_ref().map(|o| o.name.clone()) else {
            return Ok(());
        };
        let face = (self.cube.as_ref()).and_then(|host| {
            let faces = 0..host.outputs().len();
            faces.into_iter().find_map(|i| {
                host.floats_of(i)?
                    .iter()
                    .find(|w| w.surface == surface)
                    .map(|w| (i, w.rect))
            })
        });
        let Some((face, rect)) = face else {
            return Ok(());
        };
        let plane: Vec<Value> = (self.rt.conn).call(
            "scene.query",
            Query::new(format!("id:{}", float::plane_id(face, surface))),
        )?;
        let shown = plane.first().and_then(|node| {
            let r = node["rects"].get(&screen)?;
            Some([0, 1, 2, 3].map(|i| r.get(i).and_then(Value::as_f64).unwrap_or(0.0)))
        });
        let point = shown.filter(|r| r[2] > 0.0 && r[3] > 0.0).map_or(
            [rect[2] / 2.0, rect[3] / 2.0],
            |r| {
                [
                    (at[0] - r[0]) / r[2] * rect[2],
                    (at[1] - r[1]) / r[3] * rect[3],
                ]
            },
        );
        let call = MethodCall {
            client: cube::MANAGER.into(),
            method: give::METHOD.into(),
            params: json!({"surface": surface, "to": NAME}),
        };
        let given: Result<Value, _> = (self.rt.conn).call_within("methods.call", call, PLACE_WAIT);
        if let Err(err) = given {
            eprintln!("chora-construct: {surface} stays on its face: {err}");
            return Ok(());
        }
        let grab = GivenGrab {
            point,
            velocity: [0.0; 2],
            wobble: None,
        };
        self.take_pulled(surface, at, &grab)
    }

    /// The moves and the release of the drag of a window pulled from a
    /// floating plane: the press stays on the plane, so they name it, or
    /// a face. The release drops the pane over a face, which gives the
    /// window back, or in the space. None for any other event.
    pub(crate) fn plane_drag(&mut self, event: &Event) -> Option<Result<(), ClientError>> {
        self.moving.as_ref()?;
        Some(match pull_step(event, self.rt.accepts_injected())? {
            PullStep::Move(at) => self.moved(at),
            PullStep::Drop(at) => self.drop_pulled(at),
            PullStep::End => self.end_pull(),
        })
    }

    /// Ends the drag of a pulled window where its pane is, in the space:
    /// the window goes to no one.
    fn end_pull(&mut self) -> Result<(), ClientError> {
        if let Some(moving) = self.moving.take() {
            self.effects.release(&moving.name);
        }
        self.frame()
    }

    fn drop_pulled(&mut self, at: [f64; 2]) -> Result<(), ClientError> {
        let screen = self
            .output
            .as_ref()
            .map(|o| o.name.clone())
            .unwrap_or_default();
        if self.drop_on_a_face(&screen, at)? {
            return Ok(());
        }
        self.release(at, None)
    }

    /// The depth of a pulled pane: a little in front of the front face of
    /// the cube, so that the face never covers it, else at 1:1
    /// ([`pull_depth`]).
    fn pull_depth(&self) -> f64 {
        let face = self.cube.as_ref().map(|c| c.front_depth(&self.pose));
        pull_depth(face, self.lens.focal)
    }

    /// The press of `move` that comes with a window and its drag, at a
    /// point of the output. The window has its pane from the handover,
    /// or gets it now; a window that gets no pane is attached hidden, so
    /// it is not lost, and the drag ends for it. The camera flies to a
    /// window that is handed over: for a window that comes in the middle
    /// of a drag it turns back and stays where the user stands, and the
    /// keyboard stays where the flight left it. The pane
    /// comes under the pointer, with the point of the window that the
    /// giver names there, in front of the front face of the cube, and the moves and the release of
    /// `move` drag and drop it as a drag with the modifier does. A pane
    /// of an own window wobbles on in 3D, from the springs of the giver
    /// when it sent some.
    pub(crate) fn take_pulled(
        &mut self,
        surface: &str,
        at: [f64; 2],
        grab: &GivenGrab,
    ) -> Result<(), ClientError> {
        let of_window =
            |construct: &Self| (construct.panes.iter()).position(|p| p.window() == Some(surface));
        if of_window(self).is_none() {
            self.take_detached(surface, false)?;
        }
        if self.inside().as_deref() == Some(surface) {
            if self.flown.is_some() {
                self.fly_out()?;
            } else if let Some(into) = self.glide.take().and_then(|g| g.into) {
                self.glide_to(into.back, FLY_MS, None);
            }
        }
        let Some(i) = of_window(self) else {
            return Ok(());
        };
        let boxed = |construct: &Self| {
            let pane = &construct.panes[i];
            space::screen_box(
                &construct.lens,
                &construct.pose,
                pane.at,
                pane.size,
                pane.yaw,
            )
        };
        let d = self.pull_depth();
        let to = space::under_pointer(
            &self.lens,
            &self.pose,
            (at, grab.point),
            self.panes[i].size,
            d,
        );
        self.move_pane(i, to)?;
        if boxed(self).is_none() {
            return Ok(());
        }
        let pane = &self.panes[i];
        self.moving = Some(Moving {
            name: pane.name.clone(),
            from: at,
            at: pane.at,
            yaw: pane.yaw,
        });
        if pane.attached {
            let target = Target {
                node: pane.node(),
                origin: [0.0; 2],
                size: pane.size,
                set_size: true,
            };
            let held = [0, 1].map(|axis| grab.point[axis].clamp(0.0, pane.size[axis].max(0.0)));
            let name = pane.name.clone();
            (self.effects).grab_3d(&name, &target, [0.0; 3], [held[0], held[1], 0.0], [0.0; 2]);
            if let Some(springs) = &grab.wobble {
                self.effects.take_springs(&name, [0.0; 2], springs);
            }
        }
        self.moved(at)
    }
}

#[cfg(test)]
mod tests {
    use chora_scene::{OutputInfo, Transform};

    use serde_json::json;

    use super::{Event, PullStep, face_under, is_a_face, pull_depth, pull_step};

    /// Under a dragged pane the core names the screen: the face is the
    /// one whose box on the screen holds the point, and the point maps
    /// into its output. A hidden or turning face, another node and a
    /// point beside the face are no face; a face of a cube that rests
    /// stood back is one.
    #[test]
    fn a_drop_on_the_box_of_a_face_names_its_output_and_the_point_in_it() {
        let face = |id: &str, visible: bool, forward: bool| {
            json!({"id": id, "visible": visible, "forward": forward,
                "size": [1280.0, 800.0], "uniforms": {"win": {"output": "VIRTUAL-2"}},
                "rects": {"DP-1": [320.0, 200.0, 640.0, 400.0]}})
        };
        let front = [face("construct-cube-face-1", true, true)];
        let hit = face_under(&front, "DP-1", [640.0, 400.0], false);
        assert_eq!(hit, Some(("VIRTUAL-2".into(), [640.0, 400.0])));
        assert_eq!(
            face_under(&front, "DP-1", [100.0, 400.0], false),
            None,
            "beside"
        );
        assert_eq!(
            face_under(&front, "DP-2", [640.0, 400.0], false),
            None,
            "a screen"
        );
        for hidden in [(false, true), (true, false)] {
            let face = [face("construct-cube-face-1", hidden.0, hidden.1)];
            assert_eq!(face_under(&face, "DP-1", [640.0, 400.0], false), None);
        }
        let stood = [face("construct-cube-face-1", true, false)];
        let hit = face_under(&stood, "DP-1", [640.0, 400.0], true);
        assert_eq!(
            hit,
            Some(("VIRTUAL-2".into(), [640.0, 400.0])),
            "stood back"
        );
        let pane = [face("construct-pane-s1-source", true, true)];
        assert_eq!(face_under(&pane, "DP-1", [640.0, 400.0], false), None);
    }

    fn moved(injected: bool) -> Event {
        Event::PointerMoved {
            node: "construct-cube-float-0-s1".into(),
            position: [10.0, 20.0],
            local: [0.0; 2],
            modifiers: Vec::new(),
            injected,
        }
    }

    fn released(button: u32, injected: bool) -> Event {
        Event::PointerReleased {
            node: "construct-cube-face-0".into(),
            button,
            position: [30.0, 40.0],
            local: [0.0; 2],
            modifiers: Vec::new(),
            injected,
        }
    }

    /// Only the hand of the person goes on with a pull and drops it: an
    /// injected move or release ends it in the space outside a test
    /// session, and another button does nothing.
    #[test]
    fn injected_input_ends_a_pull_in_the_space() {
        let left = super::BTN_LEFT;
        assert_eq!(
            pull_step(&moved(false), false),
            Some(PullStep::Move([10.0, 20.0]))
        );
        assert_eq!(pull_step(&moved(true), false), Some(PullStep::End));
        assert_eq!(
            pull_step(&moved(true), true),
            Some(PullStep::Move([10.0, 20.0]))
        );
        assert_eq!(
            pull_step(&released(left, false), false),
            Some(PullStep::Drop([30.0, 40.0]))
        );
        assert_eq!(pull_step(&released(left, true), false), Some(PullStep::End));
        assert_eq!(
            pull_step(&released(left, true), true),
            Some(PullStep::Drop([30.0, 40.0]))
        );
        assert_eq!(
            pull_step(&released(273, false), false),
            None,
            "the right button"
        );
    }

    #[test]
    fn a_pulled_pane_is_never_behind_the_eye_or_at_no_depth() {
        assert!((pull_depth(Some(1000.0), 800.0) - 900.0).abs() < 1e-9);
        assert!((pull_depth(None, 800.0) - 800.0).abs() < 1e-9);
        assert!((pull_depth(Some(-50.0), 800.0) - 200.0).abs() < 1e-9);
        assert!((pull_depth(Some(f64::NAN), 800.0) - 800.0).abs() < 1e-9);
    }

    fn output(name: &str, owner: Option<&str>) -> OutputInfo {
        OutputInfo {
            name: name.into(),
            position: [0, 0],
            size: [1280, 800],
            scale: 1.0,
            camera: Transform::IDENTITY,
            work_area: [0, 0, 1280, 800],
            default_work_area: [0; 4],
            is_virtual: owner.map(|_| true),
            owner: owner.map(String::from),
        }
    }

    /// A pane dropped over a face goes back to the manager of the cube:
    /// only a virtual output that the construct added is a face.
    #[test]
    fn only_an_own_virtual_output_is_a_face() {
        let outputs = [
            output("DP-1", None),
            output("VIRTUAL-1", Some("construct")),
            output("VIRTUAL-2", Some("other")),
        ];
        assert!(is_a_face(&outputs, "VIRTUAL-1"));
        assert!(!is_a_face(&outputs, "VIRTUAL-2"), "of another client");
        assert!(!is_a_face(&outputs, "DP-1"), "a screen");
        assert!(!is_a_face(&outputs, "VIRTUAL-9"), "gone");
    }
}
