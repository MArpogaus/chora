//! The pull-out (E3): a window that another window manager gives with
//! its drag, as `chora-xp` does when a title bar leaves a face of the
//! cube, becomes a pane under the pointer and goes on with the drag and
//! with the wobble, now with a depth. A pane that a drag drops over a
//! face of the cube goes back: its window is a window of the manager of
//! the cube again, on the output of that face, at the drop point, which
//! the release of the drag names. So a
//! move from face to face stays inside that manager for the user.

use std::time::Duration;

use chora_kit::effects::Target;
use chora_scene::{
    GivenGrab, OutputInfo,
    ipc::{MethodCall, Query},
};
use serde_json::{Value, json};

use super::{ClientError, Construct, Event, Moving, NAME, PressState, cube, give, space};

/// How long the manager of the cube may take to place a window.
const PLACE_WAIT: Duration = Duration::from_secs(2);

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
) -> Option<(String, [f64; 2])> {
    let number = |value: &Value, i: usize| value.get(i).and_then(Value::as_f64);
    nodes.iter().find_map(|node| {
        let live = node["visible"] == true && node["forward"] == true;
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

/// Where a point of a window shows on the output, for a pane with the
/// screen box `shown` (left, top, right, bottom) and the size of the
/// window. The point comes from another client: it is kept inside the
/// window, and a window without a size is held at its middle.
pub(crate) fn held_at(shown: [f64; 4], size: [f64; 2], point: [f64; 2]) -> [f64; 2] {
    let part = |axis: usize| {
        if size[axis] > 0.0 && point[axis].is_finite() {
            (point[axis] / size[axis]).clamp(0.0, 1.0)
        } else {
            0.5
        }
    };
    [
        shown[0] + (shown[2] - shown[0]) * part(0),
        shown[1] + (shown[3] - shown[1]) * part(1),
    ]
}

impl Construct {
    /// Takes the press of `move` that comes with a window and its drag;
    /// any other event goes back to the caller.
    pub(crate) fn pulled(&mut self, event: Event) -> Result<Option<Event>, ClientError> {
        let over = match &event {
            Event::InputButton {
                state: PressState::Released,
                output,
                local,
                ..
            } => Some((output.clone(), *local)),
            _ => None,
        };
        if let Some((output, point)) = over
            && self.drop_on_a_face(&output, point)?
        {
            return Ok(None);
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
            event => Ok(Some(event)),
        }
    }

    /// The drop of a moved pane of an own window over a face of the cube:
    /// the release names the output of the face and the point in it, or,
    /// under the dragged pane, the screen and the point there. The window
    /// goes to the manager of the cube, which places it on the output of
    /// the face at that point. Returns whether the
    /// drop went there; a refusal leaves the pane to the drop of the
    /// space.
    #[allow(clippy::cast_possible_truncation, reason = "a point of an output")]
    fn drop_on_a_face(&mut self, output: &str, point: [f64; 2]) -> Result<bool, ClientError> {
        let dragged = self.moving.as_ref().and_then(|moving| {
            let pane = self.panes.iter().find(|p| p.name == moving.name)?;
            pane.window().filter(|_| pane.attached).map(String::from)
        });
        let Some(surface) = dragged else {
            return Ok(false);
        };
        let face = if is_a_face(&self.rt.outputs()?, output) {
            Some((output.to_owned(), point))
        } else {
            let shaders = Query {
                selector: format!("owner:{NAME} kind:shader"),
                filter: None,
            };
            let nodes: Vec<Value> = self.rt.conn.call("scene.query", shaders)?;
            face_under(&nodes, output, point)
        };
        let Some((output, point)) = face else {
            return Ok(false);
        };
        let output = output.as_str();
        if let Err(err) = self.give(&surface, cube::MANAGER) {
            eprintln!("chora-construct: {surface} stays a pane: {err}");
            return Ok(false);
        }
        if let Some(moving) = self.moving.take() {
            self.effects.release(&moving.name);
        }
        let at = point.map(|v| v.round() as i32);
        let place = MethodCall {
            client: cube::MANAGER.into(),
            method: "window.move".into(),
            params: json!({"surface": surface, "output": output, "at": at}),
        };
        let placed: Result<Value, _> =
            (self.rt.conn).call_within("methods.call", place, PLACE_WAIT);
        if let Err(err) = placed {
            eprintln!("chora-construct: {surface} is not at the drop point: {err}");
        }
        Ok(true)
    }

    /// The press of `move` that comes with a window and its drag, at a
    /// point of the output. The window has its pane from the handover, or
    /// gets it now; a window without room for a pane stays hidden and is
    /// not lost. The camera flies to a window that is handed over; for a
    /// window that comes in the middle of a drag it turns back and stays
    /// where the user stands. The pane moves so that the point of the window that the
    /// giver names is under the pointer, and the moves and the release of
    /// `move` then drag and drop it as a drag with the modifier does. A
    /// pane of an own window wobbles on in 3D, from the springs of the
    /// giver when it sent some.
    pub(crate) fn take_pulled(
        &mut self,
        surface: &str,
        at: [f64; 2],
        grab: &GivenGrab,
    ) -> Result<(), ClientError> {
        let of_window =
            |construct: &Self| (construct.panes.iter()).position(|p| p.window() == Some(surface));
        if of_window(self).is_none() {
            self.detached(surface)?;
        }
        if self.inside().as_deref() == Some(surface) {
            self.leave_window()?;
        }
        let Some(pane) = of_window(self).map(|i| &self.panes[i]) else {
            return Ok(());
        };
        let shown = space::screen_box(&self.lens, &self.pose, pane.at, pane.size, pane.yaw);
        let Some(shown) = shown else {
            return Ok(());
        };
        let from = held_at(shown, pane.size, grab.point);
        self.moving = Some(Moving {
            name: pane.name.clone(),
            from,
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

    use super::{face_under, held_at, is_a_face};

    /// Under a dragged pane the core names the screen: the face is the
    /// one whose box on the screen holds the point, and the point maps
    /// into its output. A hidden or turning face, another node and a
    /// point beside the face are no face.
    #[test]
    fn a_drop_on_the_box_of_a_face_names_its_output_and_the_point_in_it() {
        let face = |id: &str, visible: bool, forward: bool| {
            json!({"id": id, "visible": visible, "forward": forward,
                "size": [1280.0, 800.0], "uniforms": {"win": {"output": "VIRTUAL-2"}},
                "rects": {"DP-1": [320.0, 200.0, 640.0, 400.0]}})
        };
        let front = [face("construct-cube-face-1", true, true)];
        let hit = face_under(&front, "DP-1", [640.0, 400.0]);
        assert_eq!(hit, Some(("VIRTUAL-2".into(), [640.0, 400.0])));
        assert_eq!(face_under(&front, "DP-1", [100.0, 400.0]), None, "beside");
        assert_eq!(face_under(&front, "DP-2", [640.0, 400.0]), None, "a screen");
        for hidden in [(false, true), (true, false)] {
            let face = [face("construct-cube-face-1", hidden.0, hidden.1)];
            assert_eq!(face_under(&face, "DP-1", [640.0, 400.0]), None);
        }
        let pane = [face("construct-pane-s1-source", true, true)];
        assert_eq!(face_under(&pane, "DP-1", [640.0, 400.0]), None);
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

    /// The pointer holds the pulled window where the giver held it: the
    /// point of the window, scaled to the box that the pane shows in, and
    /// never outside the window.
    #[test]
    fn a_pulled_window_is_held_at_the_point_of_the_giver() {
        let shown = [100.0, 50.0, 500.0, 250.0];
        let size = [800.0, 400.0];
        assert_eq!(held_at(shown, size, [0.0, 0.0]), [100.0, 50.0]);
        assert_eq!(held_at(shown, size, [400.0, 100.0]), [300.0, 100.0]);
        assert_eq!(held_at(shown, size, [9000.0, -5.0]), [500.0, 50.0]);
        assert_eq!(held_at(shown, [0.0, 0.0], [3.0, 3.0]), [300.0, 150.0]);
        assert_eq!(held_at(shown, size, [f64::NAN, 100.0]), [300.0, 100.0]);
    }
}
