//! The cube (D69 step 10, D76): the construct adds virtual outputs,
//! starts one window manager on them as its child and shows each output
//! on a face of a cube in its 3D context. A face forwards the pointer
//! into its output, so the windows on it take clicks; it forwards
//! nothing while the cube turns, and hides while it looks away. The
//! cube steps back during a turn, so that its edge stays in view. When
//! the window manager ends, or does not connect in time, the outputs and
//! the cube go.

use std::{
    collections::BTreeSet,
    f64::consts::{FRAC_PI_2, SQRT_2},
    process::{Child, Stdio},
    time::{Duration, Instant},
};

use chora_kit::{
    float::{self, Floating},
    geometry::{Cube, Matrix, Turn},
    modes::{Act, View},
};
use chora_scene::{Border, Event, InputMode, NodeKind, OutputInfo, OutputTexture};
use serde_json::{Value, json};

use super::{
    BTreeMap, ClientError, Command, Construct, FLY_MS, PANES, Pose, PropsUpdate, RpcError,
    Transform, Uniform, create, update,
};

/// The group of the cube in the camera group, and the prefix of a face.
pub(crate) const CUBE: &str = "construct-cube";
/// The prefix of every node of the cube: its faces and the planes of
/// its floating windows.
pub(crate) const CUBE_PREFIX: &str = "construct-cube-";
const FACE: &str = "construct-cube-face-";
/// The lit edge of the face under the pointer in navigate mode, and how
/// far it lies in front of the face.
const EDGE: &str = "construct-cube-edge";
const EDGE_WIDTH: f64 = 2.0;
const EDGE_LIFT: f64 = 1.0;
/// How far the pointer moves at most between the press and the release
/// of a click on a face.
const CLICK: f64 = 8.0;
/// The client name of the window manager of the cube.
pub(crate) const MANAGER: &str = "xp";
/// How long a quarter turn takes, by default.
const TURN: Duration = Duration::from_millis(500);
/// How long the window manager may take to connect.
const CONNECT: Duration = Duration::from_secs(10);
/// The most faces: the side faces of a cube.
const FACES: usize = 4;
/// How far the cube stands back in navigate mode, in focal lengths: the
/// front face shows at 1 / (1 + STAND) of its size.
const STAND: f64 = 0.6;

/// The cube that the construct hosts: its outputs in face order, the
/// process of the window manager, where it stands and how it is turned.
pub(crate) struct Host {
    outputs: Vec<String>,
    /// The window manager, and until it connects, the time it has.
    child: Child,
    connect_by: Option<Instant>,
    cube: Cube,
    /// The centre on the ground, (x, z), and the yaw of the cube: face 0
    /// looks back along it.
    at: [f64; 2],
    yaw: f64,
    /// The face at the front, the angle and the running turn.
    front: usize,
    angle: f64,
    turn: Option<Turn>,
    /// Whether each face shows, as last sent.
    shown: Vec<bool>,
    /// The mode of the view (K4), how far the cube stands back in
    /// navigate mode, the part of that way it stands at now, 0 to 1, and
    /// the running step.
    view: View,
    far: f64,
    stand: f64,
    step: Option<Turn>,
    /// The pose that shows the front face at 1:1, and the window that
    /// had the keyboard before the step back.
    home: Pose,
    focus: Option<(String, usize)>,
    /// A turn that a person asked for during a turn took the place of
    /// the end of the running one; one such turn at most.
    queued: bool,
    /// The window switcher of the faces shows, and has the binding of
    /// the step back.
    pub(crate) switching: bool,
    /// The floating windows of each face, the lowest first, while they
    /// float (`float.rs`), and when their planes go after they landed.
    floats: Vec<Option<Vec<Floating>>>,
    landing: Option<Instant>,
    /// The face under the pointer, the press of the pointer on a face
    /// in navigate mode, and the binding that steps back while the
    /// construct holds it.
    hover: Option<usize>,
    press: Option<Press>,
    step_back_key: Option<String>,
    /// The faces whose window manager owes an answer, when the construct
    /// asks again, and the faces that stay flat until the cube rests.
    owing: BTreeSet<usize>,
    again: Option<Instant>,
    denied: BTreeSet<usize>,
}

/// A press on a face in navigate mode: the face, where the pointer was
/// at the press, its x at the last turn of the drag, and whether the
/// pointer went farther than a click does.
#[derive(Clone, Copy)]
struct Press {
    face: usize,
    start: [f64; 2],
    from: f64,
    dragged: bool,
}

/// What a move of the pointer during a press on a face does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Dragged {
    Nothing,
    Turn(i32),
}

/// A move of the pointer to `at` during a press: after each `third` of
/// the width of the view from the last turn, the cube turns by one face;
/// a drag to the left brings the next face. The drag goes on from `at`
/// only after [`turned_at`], when the cube took the turn.
fn drag_of(press: &mut Option<Press>, at: [f64; 2], third: f64) -> Dragged {
    let Some(press) = press else {
        return Dragged::Nothing;
    };
    let far = (at[0] - press.start[0]).hypot(at[1] - press.start[1]);
    press.dragged |= far > CLICK;
    let moved = at[0] - press.from;
    if moved.abs() < third {
        return Dragged::Nothing;
    }
    Dragged::Turn(if moved < 0.0 { 1 } else { -1 })
}

/// The cube took the turn of a drag at `x`: the next third counts from
/// there.
fn turned_at(press: &mut Option<Press>, x: f64) {
    if let Some(press) = press {
        press.from = x;
    }
}

fn face_id(i: usize) -> String {
    format!("{FACE}{i}")
}

/// The face of a node id of a face.
pub(crate) fn face_index(node: &str) -> Option<usize> {
    node.strip_prefix(FACE)?.parse().ok()
}

/// The transform of the cube group in the camera group: the space of the
/// cube, with y down, at `at` on the ground, turned by `yaw`.
fn placement([x, z]: [f64; 2], yaw: f64) -> Matrix {
    let (s, c) = yaw.sin_cos();
    [
        c, 0.0, -s, 0.0, 0.0, -1.0, 0.0, 0.0, s, 0.0, c, 0.0, x, 0.0, z, 1.0,
    ]
}

/// The face that is at the front after `by` quarter turns from `front`,
/// of `n` faces.
pub(crate) fn next_front(front: usize, by: i32, n: usize) -> usize {
    let n = i32::try_from(n).unwrap_or(1).max(1);
    let front = i32::try_from(front).unwrap_or(0).rem_euclid(n);
    #[allow(clippy::cast_sign_loss, reason = "rem_euclid of a positive n")]
    let next = (front + by.rem_euclid(n)).rem_euclid(n) as usize;
    next
}

/// How far the cube is in a turn at `angle`: 0 at rest on a face, 1 at
/// 45 degrees past it.
fn turned(angle: f64) -> f64 {
    let off = (angle / FRAC_PI_2 - (angle / FRAC_PI_2).round()).abs() * FRAC_PI_2;
    (2.0 * off).sin()
}

/// How far the cube steps back at `angle`: nothing at rest, and at 45
/// degrees past a face enough that the edge, sqrt(2) times as near as a
/// face, stays as far as a face at rest, with a margin.
fn step_back(angle: f64, depth: f64) -> f64 {
    (depth / 2.0 * (SQRT_2 - 1.0) * 1.1) * turned(angle)
}

impl Host {
    /// Where the eye at `eye` on the ground is in the space of the cube.
    fn eye(&self, eye: [f64; 2]) -> [f64; 3] {
        let at = self.centre();
        let rel = [eye[0] - at[0], eye[1] - at[1]];
        let (s, c) = self.yaw.sin_cos();
        [rel[0] * c - rel[1] * s, 0.0, rel[0] * s + rel[1] * c]
    }

    /// The centre now: the place, stepped back along the view of the open.
    fn centre(&self) -> [f64; 2] {
        let back = step_back(self.angle, self.cube.size[0]) + self.stand * self.far;
        let (s, c) = self.yaw.sin_cos();
        [self.at[0] + s * back, self.at[1] + c * back]
    }

    /// The updates of the cube at the angle now: its place, the transform
    /// of each face, whether it shows, sent on a change, and whether it
    /// forwards, which it does only while the cube rests.
    pub(crate) fn faces(&mut self, eye: [f64; 2]) -> Vec<Command> {
        let eye = self.eye(eye);
        let mut commands = vec![update(
            CUBE,
            PropsUpdate {
                transform: Some(Transform::from_matrix(placement(self.centre(), self.yaw))),
                ..PropsUpdate::default()
            },
        )];
        for i in 0..self.outputs.len() {
            let shows = self.cube.faces_eye(i, self.angle, eye);
            let changed = self.shown.get(i) != Some(&shows);
            if let Some(slot) = self.shown.get_mut(i) {
                *slot = shows;
            }
            commands.push(update(
                &face_id(i),
                PropsUpdate {
                    transform: Some(Transform::from_matrix(self.cube.face(i, self.angle))),
                    visible: changed.then_some(shows),
                    forward: Some(self.view.forwards() && self.stand <= 0.0),
                    ..PropsUpdate::default()
                },
            ));
            commands.extend(self.planes(i, shows));
        }
        commands.push(self.edge());
        commands
    }

    /// The update of the lit edge: over the face under the pointer while
    /// the cube rests in navigate mode.
    fn edge(&self) -> Command {
        let rests = self.turn.is_none() && !self.view.forwards() && self.step.is_none();
        let face = self
            .hover
            .filter(|i| rests && self.shown.get(*i) == Some(&true));
        let placed = face.map(|i| {
            let plane = self.cube.lifted(i, self.angle, [0.0, 0.0], EDGE_LIFT);
            (Transform::from_matrix(plane), self.cube.size_of(i))
        });
        update(
            EDGE,
            PropsUpdate {
                transform: placed.map(|(transform, _)| transform),
                size: placed.map(|(_, size)| size),
                visible: Some(face.is_some()),
                ..PropsUpdate::default()
            },
        )
    }

    /// The pointer is over a face, or over none.
    pub(crate) fn set_hover(&mut self, face: Option<usize>) -> bool {
        let changed = self.hover != face;
        self.hover = face;
        changed
    }

    /// The face under the pointer.
    pub(crate) fn hover(&self) -> Option<usize> {
        self.hover
    }

    /// A press on a face in navigate mode at a point of the screen.
    pub(crate) fn press(&mut self, face: usize, at: [f64; 2]) {
        self.press = Some(Press {
            face,
            start: at,
            from: at[0],
            dragged: false,
        });
    }

    /// The pointer moved to `at` during a press: after each `third` of
    /// the width of the view the cube turns by one face, with the drag.
    pub(crate) fn drag(&mut self, at: [f64; 2], third: f64) -> Dragged {
        drag_of(&mut self.press, at, third)
    }

    /// The cube took the turn of the drag at `x`.
    pub(crate) fn drag_turned(&mut self, x: f64) {
        turned_at(&mut self.press, x);
    }

    /// Forgets the press: after injected input during it, which is not
    /// the person.
    pub(crate) fn forget_press(&mut self) {
        self.press = None;
    }

    /// The release of a press: answers the face of a click, a press that
    /// did not move.
    pub(crate) fn release(&mut self) -> Option<usize> {
        let press = self.press.take()?;
        (!press.dragged).then_some(press.face)
    }

    /// The binding that steps back, while the construct holds it.
    pub(crate) fn step_back_key(&self) -> Option<&str> {
        self.step_back_key.as_deref()
    }

    /// Sets the binding that steps back; answers whether it changed.
    pub(crate) fn set_step_back_key(&mut self, key: Option<String>) -> bool {
        let changed = self.step_back_key != key;
        self.step_back_key = key;
        changed
    }

    /// The updates of the planes of the floating windows of face `i`:
    /// each at the place of its window, off the face by its lift, which
    /// grows with the stand back and with the turn.
    fn planes(&self, i: usize, shows: bool) -> Vec<Command> {
        let windows = self.floats_of(i).unwrap_or_default();
        let phase = self.stand.max(turned(self.angle));
        let plane = |(k, window): (usize, &Floating)| {
            let lift = float::lift(phase, k, windows.len());
            let at = [window.rect[0], window.rect[1]];
            update(
                &super::float::plane_id(i, &window.surface),
                PropsUpdate {
                    transform: Some(Transform::from_matrix(
                        self.cube.lifted(i, self.angle, at, lift),
                    )),
                    visible: Some(shows),
                    ..PropsUpdate::default()
                },
            )
        };
        windows.iter().enumerate().map(plane).collect()
    }

    /// The updates of the planes of a face as it shows now.
    pub(crate) fn plane_updates(&self, face: usize) -> Vec<Command> {
        self.planes(face, self.shown.get(face) == Some(&true))
    }

    /// The outputs in face order.
    pub(crate) fn outputs(&self) -> &[String] {
        &self.outputs
    }

    /// Whether windows float now: the cube stands back, steps or turns.
    /// Not before the window manager connected.
    pub(crate) fn wants_float(&self) -> bool {
        let moves = self.turn.is_some() || self.step.is_some() || self.stand > 0.0;
        self.connect_by.is_none() && (moves || !self.view.forwards())
    }

    /// Whether the windows of a face float now: windows float, the face
    /// shows, and its planes were not refused.
    pub(crate) fn face_floats(&self, face: usize) -> bool {
        let shows = self.shown.get(face) == Some(&true);
        self.wants_float() && shows && !self.denied.contains(&face)
    }

    /// The floating windows of a face; none while it is flat.
    pub(crate) fn floats_of(&self, face: usize) -> Option<&[Floating]> {
        self.floats.get(face)?.as_deref()
    }

    /// The faces whose windows float.
    pub(crate) fn floating(&self) -> Vec<usize> {
        let faces = 0..self.floats.len();
        faces.filter(|i| self.floats_of(*i).is_some()).collect()
    }

    /// When the planes of the landed windows go.
    pub(crate) fn landing(&self) -> Option<Instant> {
        self.landing
    }

    /// The windows land at a time, or float on.
    pub(crate) fn land_at(&mut self, at: Option<Instant>) {
        self.landing = at;
    }

    /// The windows landed and the cube rests: answers the planes that
    /// go. Every face may float again.
    pub(crate) fn landed(&mut self) -> Vec<String> {
        self.landing = None;
        self.denied.clear();
        let faces = 0..self.floats.len();
        faces.flat_map(|i| self.set_floats(i, None)).collect()
    }

    /// Sets the floating windows of a face, or none for a flat face;
    /// answers the planes of before.
    pub(crate) fn set_floats(
        &mut self,
        face: usize,
        windows: Option<Vec<Floating>>,
    ) -> Vec<String> {
        let Some(slot) = self.floats.get_mut(face) else {
            return Vec::new();
        };
        let old = std::mem::replace(slot, windows).unwrap_or_default();
        let id = |w: Floating| super::float::plane_id(face, &w.surface);
        old.into_iter().map(id).collect()
    }

    /// Notes whether the window manager owes an answer for a face.
    /// Answers whether that is new.
    pub(crate) fn owes(&mut self, face: usize, owes: bool) -> bool {
        if owes {
            self.owing.insert(face)
        } else {
            self.owing.remove(&face)
        }
    }

    /// Whether the window manager owes an answer for a face.
    pub(crate) fn owed(&self, face: usize) -> bool {
        self.owing.contains(&face)
    }

    /// When the construct asks the window manager again.
    pub(crate) fn asks_again(&self) -> Option<Instant> {
        self.again
    }

    /// Sets when the construct asks again; none after every answer.
    pub(crate) fn ask_again(&mut self, at: Option<Instant>) {
        self.again = at;
    }

    /// A face stays flat until the cube rests.
    pub(crate) fn deny(&mut self, face: usize) {
        self.denied.insert(face);
    }

    /// Whether the cube waits for a time: the planes of landed windows
    /// go, or the construct asks the window manager again.
    fn waits(&self) -> bool {
        self.landing.is_some() || self.again.is_some()
    }

    /// The mode of the view.
    pub(crate) fn view(&self) -> View {
        self.view
    }

    /// The pose that shows the front face at 1:1.
    pub(crate) fn home(&self) -> Pose {
        self.home
    }

    /// Does a step back or a step in: the cube starts on its way. Answers
    /// whether the mode changed.
    pub(crate) fn act(&mut self, act: Act, now: Instant) -> bool {
        let changed = self.view.act(act);
        if changed {
            self.press = None;
            let to = if act == Act::StepBack { 1.0 } else { 0.0 };
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "380 ms"
            )]
            let time = Duration::from_millis(FLY_MS as u64);
            self.step = Some(Turn::new(self.stand, to, now, time));
        }
        changed
    }

    /// Notes the window that had the keyboard before the step back, with
    /// the face at the front then.
    pub(crate) fn note_focus(&mut self, window: Option<String>) {
        self.focus = window.map(|window| (window, self.front));
    }

    /// The window that gets the keyboard again at the step in: the noted
    /// one, when its face is at the front again.
    pub(crate) fn take_focus(&mut self) -> Option<String> {
        let (window, front) = self.focus.take()?;
        (front == self.front).then_some(window)
    }

    /// The output of the face at the front.
    pub(crate) fn front_output(&self) -> String {
        self.outputs.get(self.front).cloned().unwrap_or_default()
    }

    /// Starts a turn by `by` faces. During a turn, a turn that a person
    /// asked for takes the place of the end of the running one, once
    /// (`queue`); any other is ignored. Answers whether the cube turns
    /// to a new face.
    fn start_turn(&mut self, by: i32, duration: Duration, queue: bool, now: Instant) -> bool {
        let fresh = self.view.act(Act::Turn(by));
        let queues = queue && by != 0 && self.turn.is_some() && !self.queued;
        if !fresh && !queues {
            return false;
        }
        self.queued = !fresh;
        self.front = next_front(self.front, by, self.outputs.len());
        let to = Cube::turn_of(self.front);
        self.turn = Some(Turn::new(self.angle, to, now, duration));
        true
    }

    /// The quarter turns from the front face to `face`, the short way.
    pub(crate) fn turns_to(&self, face: usize) -> i32 {
        let n = i32::try_from(self.outputs.len()).unwrap_or(1).max(1);
        let face = i32::try_from(face).unwrap_or(0);
        let by = (face - i32::try_from(self.front).unwrap_or(0)).rem_euclid(n);
        if by * 2 > n { by - n } else { by }
    }
}

impl Construct {
    /// Opens the cube: `faces` virtual outputs of the size of the view at
    /// its scale, one window manager on all of them, and a face for each,
    /// ahead of the camera, which levels, so that the front face shows its
    /// output at 1:1. Refused while another client `xp` runs.
    pub(crate) fn open_cube(&mut self, faces: usize) -> Result<Value, ClientError> {
        let refuse = |m: String| Err(ClientError::Rpc(RpcError::new(RpcError::REJECTED, m)));
        if self.cube.is_some() {
            return refuse("a cube is open; cube.close first".into());
        }
        if !(1..=FACES).contains(&faces) {
            return refuse(format!("faces: {faces} is not 1 to {FACES}"));
        }
        if self.manager_runs()? {
            return refuse(format!("a client {MANAGER} runs already"));
        }
        let size = [self.lens.w, self.lens.h];
        let outputs = self.add_outputs(faces, size)?;
        let flags: Vec<String> = outputs.iter().map(|o| format!("--output {o}")).collect();
        let command = format!(
            "exec {} {}",
            self.settings.value.cube_manager,
            flags.join(" ")
        );
        let spawned = std::process::Command::new("sh")
            .args(["-c", &command])
            .stdin(Stdio::null())
            .spawn();
        let child = match spawned {
            Ok(child) => child,
            Err(err) => {
                self.remove_outputs(&outputs);
                return refuse(format!("cannot start {command:?}: {err}"));
            }
        };
        self.glide = None;
        self.pose.pitch = 0.0;
        let cube = Cube { size, caps: false };
        let mut host = self.host(outputs.clone(), child, cube);
        let mut commands = vec![create(CUBE, NodeKind::Group, PANES, PropsUpdate::default())];
        commands.extend(self.face_nodes(&outputs, &cube));
        commands.push(self.edge_node());
        commands.extend(host.faces([self.pose.x, self.pose.z]));
        if let Some(err) = self.rt.conn.try_apply(commands)? {
            end(&mut host.child);
            self.remove_outputs(&outputs);
            return Err(ClientError::Rpc(err));
        }
        self.cube = Some(host);
        self.open_modes()?;
        self.frame()?;
        Ok(json!({"outputs": outputs}))
    }

    /// The host of a new cube ahead of the camera, in use mode.
    fn host(&self, outputs: Vec<String>, child: Child, cube: Cube) -> Host {
        let [fx, fz] = self.pose.forward();
        let reach = self.lens.focal + cube.size[0] / 2.0;
        Host {
            shown: vec![false; outputs.len()],
            floats: vec![None; outputs.len()],
            outputs,
            child,
            connect_by: Some(Instant::now() + CONNECT),
            cube,
            at: [self.pose.x + fx * reach, self.pose.z + fz * reach],
            yaw: self.pose.yaw,
            front: 0,
            angle: 0.0,
            turn: None,
            view: View::default(),
            far: STAND * self.lens.focal,
            stand: 0.0,
            step: None,
            home: self.pose,
            focus: None,
            queued: false,
            switching: false,
            landing: None,
            hover: None,
            press: None,
            step_back_key: None,
            owing: BTreeSet::new(),
            again: None,
            denied: BTreeSet::new(),
        }
    }

    /// Whether a client of the name of the window manager is connected.
    fn manager_runs(&mut self) -> Result<bool, ClientError> {
        let clients: Value = self.rt.conn.call("clients.list", Value::Null)?;
        let list = clients.as_array().cloned().unwrap_or_default();
        Ok(list.iter().any(|c| c["name"].as_str() == Some(MANAGER)))
    }

    /// The window manager of the cube connected, and no longer has to;
    /// or it went, and the cube goes with it.
    pub(crate) fn cube_client(&mut self, event: &Event) -> Result<(), ClientError> {
        if matches!(event, Event::ClientDisconnected { .. }) {
            return self.close_cube();
        }
        if let Some(host) = &mut self.cube {
            host.connect_by = None;
        }
        Ok(())
    }

    /// The time by which the window manager must connect.
    pub(crate) fn cube_deadline(&self) -> Option<Instant> {
        self.cube.as_ref().and_then(|c| c.connect_by)
    }

    /// Closes the cube when its window manager did not connect in time.
    pub(crate) fn check_cube(&mut self) -> Result<(), ClientError> {
        if self.cube_deadline().is_some_and(|t| t <= Instant::now()) {
            eprintln!("chora-construct: the window manager of the cube did not connect");
            return self.close_cube();
        }
        Ok(())
    }

    /// Turns the cube by `by` quarter turns in `ms`, a quarter turn in
    /// 500 ms by default. A turn while one runs is ignored; with `queue`,
    /// for the keys, the chips and the drag of a person, one such turn
    /// goes on from the running one. Answers whether the cube took the
    /// turn.
    pub(crate) fn turn_cube(
        &mut self,
        by: i32,
        ms: Option<u64>,
        queue: bool,
    ) -> Result<bool, ClientError> {
        let Some(host) = &mut self.cube else {
            return Err(ClientError::Rpc(RpcError::new(
                RpcError::REJECTED,
                "no cube is open; cube.open first",
            )));
        };
        let duration = ms.map_or(TURN, Duration::from_millis);
        let taken = host.start_turn(by, duration, queue, Instant::now());
        if taken {
            self.draw_modes()?;
        }
        Ok(taken)
    }

    /// One step of the turn of the cube, then the frame.
    pub(crate) fn step_cube(&mut self) -> Result<(), ClientError> {
        let Some(host) = &mut self.cube else {
            return Ok(());
        };
        let now = Instant::now();
        if let Some(turn) = host.turn {
            host.angle = turn.at(now);
            if turn.done(now) {
                host.turn = None;
                host.queued = false;
                host.view.turn_ended();
            }
        }
        if let Some(step) = host.step {
            host.stand = step.at(now);
            if step.done(now) {
                host.stand = host.stand.round();
                host.step = None;
            }
        }
        self.frame()?;
        self.sync_float()
    }

    /// The updates of the faces of the cube for the pose now.
    pub(crate) fn cube_faces(&mut self) -> Vec<Command> {
        let eye = [self.pose.x, self.pose.z];
        self.cube.as_mut().map(|h| h.faces(eye)).unwrap_or_default()
    }

    /// Whether the cube turns or steps and needs frames.
    pub(crate) fn cube_turns(&self) -> bool {
        (self.cube.as_ref()).is_some_and(|c| c.turn.is_some() || c.step.is_some() || c.waits())
    }

    /// Closes the cube: its window manager ends, and its outputs and the
    /// cube go.
    pub(crate) fn close_cube(&mut self) -> Result<(), ClientError> {
        let Some(mut host) = self.cube.take() else {
            return Ok(());
        };
        end(&mut host.child);
        self.remove_outputs(&host.outputs);
        self.rt.conn.try_apply(vec![Command::Remove {
            id: CUBE.into(),
            to: None,
        }])?;
        self.close_modes()
    }

    /// The shader nodes of the faces, hidden until the first frame: each
    /// reads its output.
    fn face_nodes(&self, outputs: &[String], cube: &Cube) -> Vec<Command> {
        (outputs.iter().enumerate())
            .map(|(i, output)| {
                let win = Uniform::Output(OutputTexture {
                    output: output.clone(),
                });
                create(
                    &face_id(i),
                    NodeKind::Shader,
                    CUBE,
                    PropsUpdate {
                        size: Some(cube.size_of(i)),
                        shader: Some(self.source_shader.clone()),
                        uniforms: Some(BTreeMap::from([("win".into(), win)])),
                        visible: Some(false),
                        ..PropsUpdate::default()
                    },
                )
            })
            .collect()
    }

    /// The lit edge of a face, hidden: a border in the accent colour that
    /// passes the pointer to the face behind it.
    fn edge_node(&self) -> Command {
        let [r, g, b] = self.settings.value.style.accent;
        create(
            EDGE,
            NodeKind::Rect,
            CUBE,
            PropsUpdate {
                size: Some([1.0, 1.0]),
                color: Some([0.0; 4]),
                border: Some(Some(Border {
                    width: EDGE_WIDTH,
                    color: [r, g, b, 1.0],
                })),
                visible: Some(false),
                input: Some(InputMode::Pass),
                ..PropsUpdate::default()
            },
        )
    }

    /// Removes the virtual outputs that the construct added before it
    /// started again.
    pub(crate) fn remove_left_outputs(&mut self) -> Result<(), ClientError> {
        let outputs: Vec<OutputInfo> = self.rt.conn.call("output.list", Value::Null)?;
        let left: Vec<String> = (outputs.into_iter())
            .filter(|o| o.is_virtual == Some(true) && o.owner.as_deref() == Some(super::NAME))
            .map(|o| o.name)
            .collect();
        self.remove_outputs(&left);
        Ok(())
    }

    /// Adds `faces` virtual outputs of `size` at the scale of the view;
    /// none when one fails.
    fn add_outputs(&mut self, faces: usize, size: [f64; 2]) -> Result<Vec<String>, ClientError> {
        let scale = self.output.as_ref().map_or(1.0, |o| o.scale);
        let spec = format!(
            "{}x{}@{scale}",
            (size[0] * scale).round(),
            (size[1] * scale).round()
        );
        let mut outputs = Vec::new();
        for _ in 0..faces {
            let added: Result<OutputInfo, _> =
                (self.rt.conn).call("output.add", json!({"spec": spec, "virtual": true}));
            match added {
                Ok(output) => outputs.push(output.name),
                Err(err) => {
                    self.remove_outputs(&outputs);
                    return Err(err);
                }
            }
        }
        Ok(outputs)
    }

    fn remove_outputs(&mut self, outputs: &[String]) {
        for output in outputs {
            let removed: Result<Value, _> =
                (self.rt.conn).call("output.remove", json!({"output": output}));
            if let Err(err) = removed {
                eprintln!("chora-construct: output.remove {output}: {err}");
            }
        }
    }
}

/// Ends a window manager of the cube and reaps it.
fn end(child: &mut Child) {
    if let Err(err) = child.kill() {
        eprintln!("chora-construct: end the window manager of the cube: {err}");
    }
    if let Err(err) = child.wait() {
        eprintln!("chora-construct: wait for the window manager of the cube: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRESS: Press = Press {
        face: 0,
        start: [600.0, 300.0],
        from: 600.0,
        dragged: false,
    };

    #[test]
    fn a_drag_turns_once_per_third_that_the_cube_took() {
        let mut held = Some(PRESS);
        assert_eq!(drag_of(&mut held, [500.0, 300.0], 400.0), Dragged::Nothing);
        assert_eq!(drag_of(&mut held, [190.0, 300.0], 400.0), Dragged::Turn(1));
        assert_eq!(
            drag_of(&mut held, [180.0, 300.0], 400.0),
            Dragged::Turn(1),
            "the cube did not take it: the drag asks again"
        );
        turned_at(&mut held, 180.0);
        assert_eq!(drag_of(&mut held, [100.0, 300.0], 400.0), Dragged::Nothing);
        assert_eq!(drag_of(&mut held, [-230.0, 300.0], 400.0), Dragged::Turn(1));
        let mut back = Some(PRESS);
        assert_eq!(
            drag_of(&mut back, [1010.0, 300.0], 400.0),
            Dragged::Turn(-1)
        );
    }

    #[test]
    fn a_click_stays_within_eight_pixels_on_both_axes() {
        let dragged = |at: [f64; 2]| {
            let mut held = Some(PRESS);
            drag_of(&mut held, at, 400.0);
            held.is_some_and(|p| p.dragged)
        };
        assert!(!dragged([603.0, 297.0]));
        assert!(dragged([600.0, 320.0]), "up and down counts too");
        assert!(dragged([609.0, 300.0]));
        assert!(dragged([606.0, 306.0]), "the distance, not one axis");
    }
}
