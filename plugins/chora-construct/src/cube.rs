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
    io::{BufRead, BufReader},
    process::{Child, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use chora_kit::{
    float::{self, Floating},
    geometry::{Cube, Matrix, Turn},
    modes::{Act, View},
    words::Language,
};
use chora_scene::{Border, Event, InputMode, NodeKind, OutputInfo, OutputTexture};
use serde_json::{Value, json};

use super::{
    BTreeMap, ClientError, Command, Construct, FLY_MS, PANES, Pose, PropsUpdate, RpcError,
    Transform, Uniform, create, faces::Health, update,
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
pub(crate) const TURN: Duration = Duration::from_millis(360);
/// How long the cube takes to settle on a face after a drag.
pub(crate) const SETTLE: Duration = Duration::from_millis(320);
/// How fast a flick is, in widths of the view per second.
pub(crate) const FLICK: f64 = 1.5;
/// The plain fill of a face whose window manager is not up, and how far
/// it lies in front of the face.
const FILL: &str = "construct-cube-fill-";
const FILL_LIFT: f64 = 0.5;
/// How often the construct looks whether a starting manager exited.
const POLL: Duration = Duration::from_millis(100);
/// How far the cube turns off square toward the next face when it
/// stands back, in radians: 14 degrees.
#[allow(dead_code, reason = "a stub until the next commit")]
pub(crate) const REST: f64 = 14.0 * std::f64::consts::PI / 180.0;
/// The mirror image of a face in the floor.
#[allow(dead_code, reason = "a stub until the next commit")]
const MIRROR: &str = "construct-cube-mirror-";
/// The most faces: the side faces of a cube.
pub(crate) const FACES: usize = 4;
/// The most that a window lifts off its face to clear the planes of a
/// neighbour face, and the room it keeps from them.
const CLEAR_CAP: f64 = 2.0 * float::LIFT;
const CLEAR_GAP: f64 = 8.0;
/// How far the cube stands back in navigate mode, in focal lengths: the
/// front face shows at 1 / (1 + STAND) of its size.
const STAND: f64 = 0.6;

/// The cube that the construct hosts: its outputs in face order, the
/// process of the window manager, where it stands and how it is turned.
pub(crate) struct Host {
    outputs: Vec<String>,
    /// The window manager, how it was started, when, the last line it
    /// wrote to its error output, and the health of each face.
    child: Child,
    command: String,
    program: String,
    name: String,
    since: Instant,
    tail: Tail,
    health: Vec<Health>,
    cube: Cube,
    /// The centre on the ground, (x, z), and the yaw of the cube: face 0
    /// looks back along it.
    at: [f64; 2],
    yaw: f64,
    /// The face at the front, the angle and the running turn.
    front: usize,
    angle: f64,
    turn: Option<Turn>,
    /// Whether each face shows, as last sent, and the eye in the space of
    /// the cube then.
    shown: Vec<bool>,
    seen_from: [f64; 3],
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
    live: Option<Live>,
    step_back_key: Option<String>,
    /// The faces whose window manager owes an answer, when the construct
    /// asks again, and the faces that stay flat until the cube rests.
    owing: BTreeSet<usize>,
    again: Option<Instant>,
    denied: BTreeSet<usize>,
}

/// The last line that a window manager wrote to its error output.
type Tail = Arc<Mutex<String>>;

/// Starts `command` with its error output piped: each line goes on to
/// the log of the construct, and the last one is the reason if the
/// manager fails.
fn launch(command: &str) -> std::io::Result<(Child, Tail)> {
    let mut child = super::wm::child_command(command)
        .stderr(Stdio::piped())
        .spawn()?;
    let tail = Tail::default();
    if let Some(pipe) = child.stderr.take() {
        let last = Arc::clone(&tail);
        std::thread::spawn(move || {
            for line in BufReader::new(pipe).lines().map_while(Result::ok) {
                eprintln!("chora-construct: cube manager: {line}");
                if let (false, Ok(mut kept)) = (line.trim().is_empty(), last.lock()) {
                    *kept = line;
                }
            }
        });
    }
    Ok((child, tail))
}

/// The id of the plain fill of face `i`.
fn fill_id(i: usize) -> String {
    format!("{FILL}{i}")
}

/// A press on a face in navigate mode: the face, the window whose
/// floating plane took the press, where the pointer was at the press, and
/// whether the pointer went farther than a click does.
#[derive(Clone)]
struct Press {
    face: usize,
    window: Option<String>,
    start: [f64; 2],
    dragged: bool,
}

/// What a move of the pointer during a press on a face does.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Dragged {
    Nothing,
    /// The cube follows the hand: the drag so far in x.
    Yaw(f64),
    /// The drag on the floating plane of a window pulls the window out.
    Pull(String),
}

/// A move of the pointer to `at` during a press: once it went farther
/// than a click, the cube follows the hand from the press; a press on the
/// floating plane of a window pulls the window out instead.
fn drag_of(press: &mut Option<Press>, at: [f64; 2]) -> Dragged {
    let Some(press) = press else {
        return Dragged::Nothing;
    };
    let far = (at[0] - press.start[0]).hypot(at[1] - press.start[1]);
    press.dragged |= far > CLICK;
    if !press.dragged {
        return Dragged::Nothing;
    }
    match press.window.clone() {
        Some(window) => Dragged::Pull(window),
        None => Dragged::Yaw(at[0] - press.start[0]),
    }
}

/// The cube while the hand holds it: the angle at the press, the width
/// of the view, and the last positions of the hand for its speed.
struct Live {
    base: f64,
    width: f64,
    samples: Vec<(Instant, f64)>,
}

/// How far back the speed of the hand looks, and the least time that it
/// divides by.
const SAMPLED: Duration = Duration::from_millis(120);
const LEAST: f64 = 0.005;

impl Live {
    /// The speed of the hand at `now` in widths per second, to the right
    /// positive; none when it rested.
    fn velocity(&self, now: Instant) -> f64 {
        let recent: Vec<&(Instant, f64)> = (self.samples.iter())
            .filter(|(at, _)| now.saturating_duration_since(*at) <= SAMPLED)
            .collect();
        let (Some(first), Some(last)) = (recent.first(), recent.last()) else {
            return 0.0;
        };
        let seconds = last.0.saturating_duration_since(first.0).as_secs_f64();
        (last.1 - first.1) / self.width / seconds.max(LEAST)
    }
}

/// The yaw of the cube for a drag of `dx` over a view `width` wide: the
/// whole width is a quarter turn.
pub(crate) fn yaw_of(dx: f64, width: f64) -> f64 {
    dx / width * FRAC_PI_2
}

/// By how many faces the cube turns when the hand lets go with the cube
/// at `yaw` off its face, moving at `velocity` widths per second (to the
/// right is positive): to the nearest face, and on to the next one when
/// the hand flicks.
pub(crate) fn settle_by(yaw: f64, velocity: f64) -> i32 {
    let nearest = (-yaw / FRAC_PI_2).round();
    let towards = if velocity < 0.0 { 1.0 } else { -1.0 };
    let by = if velocity.abs() >= FLICK && nearest * towards < 1.0 {
        towards
    } else {
        nearest
    };
    #[allow(clippy::cast_possible_truncation, reason = "a few faces")]
    let by = by as i32;
    by
}

/// The face that the label names for the cube at `angle`: the nearest
/// one.
pub(crate) fn nearest_face(angle: f64, faces: usize) -> usize {
    #[allow(clippy::cast_possible_truncation, reason = "a few faces")]
    let nearest = (-angle / FRAC_PI_2).round() as i32;
    next_front(0, nearest, faces)
}

/// How much of the front dot each of `faces` dots has at `angle`: the
/// face before the angle has the rest of what the next one has not yet.
pub(crate) fn dot_weights(angle: f64, faces: usize) -> Vec<f64> {
    let place = -angle / FRAC_PI_2;
    let before = place.floor();
    #[allow(clippy::cast_possible_truncation, reason = "a few faces")]
    let before_face = next_front(0, before as i32, faces);
    let after_face = next_front(before_face, 1, faces);
    let mut weights = vec![0.0; faces];
    weights[before_face] += 1.0 - (place - before);
    weights[after_face] += place - before;
    weights
}

pub(crate) fn face_id(i: usize) -> String {
    format!("{FACE}{i}")
}

/// The id of the mirror image of face `i`.
#[allow(dead_code, reason = "a stub until the next commit")]
pub(crate) fn mirror_id(i: usize) -> String {
    format!("{MIRROR}{i}")
}

/// The face of a node id of a face.
pub(crate) fn face_index(node: &str) -> Option<usize> {
    node.strip_prefix(FACE)?.parse().ok()
}

/// The face of a node id of a face or of a floating plane on it.
pub(crate) fn face_of(node: &str) -> Option<usize> {
    face_index(node).or_else(|| super::float::plane_face(node))
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

/// The product of two column-major matrices: `b` first, then `a`.
#[allow(dead_code, reason = "a stub until the next commit")]
fn mul4(a: &Matrix, b: &Matrix) -> Matrix {
    let mut m = [0.0; 16];
    for c in 0..4 {
        for r in 0..4 {
            m[c * 4 + r] = (0..4).map(|k| a[k * 4 + r] * b[c * 4 + k]).sum();
        }
    }
    m
}

/// A floating window and its lift at full float.
type Lifted<'a> = (&'a Floating, f64);

/// The lift of a window of `width` wide faces at full float, `base` by
/// its place in the stack, grown until its plane clears the planes of
/// the windows across its left and its right edge, each with its own
/// base lift; at most `CLEAR_CAP`.
fn cleared(window: &Floating, base: f64, width: f64, (left, right): (&[Lifted], &[Lifted])) -> f64 {
    let [_, y, _, h] = window.rect;
    let level = |q: &Floating| q.rect[1] < y + h && y < q.rect[1] + q.rect[3];
    let past_left = |q: &Floating| -q.rect[0];
    let past_right = |q: &Floating| q.rect[0] + q.rect[2] - width;
    let reach = |reach: f64, others: &[Lifted], past: &dyn Fn(&Floating) -> f64| {
        (others.iter())
            .filter(|(q, lift)| level(q) && reach > *lift && past(q) > base)
            .map(|(q, _)| past(q) + CLEAR_GAP)
            .fold(base, f64::max)
    };
    let grown = reach(past_right(window), right, &past_left).max(reach(
        past_left(window),
        left,
        &past_right,
    ));
    grown.min(CLEAR_CAP.max(base))
}

impl Host {
    /// Where the eye at `eye` on the ground is in the space of the cube.
    fn eye(&self, eye: [f64; 2]) -> [f64; 3] {
        let at = self.centre();
        let rel = [eye[0] - at[0], eye[1] - at[1]];
        let (s, c) = self.yaw.sin_cos();
        [rel[0] * c - rel[1] * s, 0.0, rel[0] * s + rel[1] * c]
    }

    /// How far the front face is from the eye of `pose`, along the view.
    pub(crate) fn front_depth(&self, pose: &Pose) -> f64 {
        super::space::depth(pose, self.centre()) - self.cube.size[0] / 2.0
    }

    /// Whether the front face covers a pane at `at` for the eye of
    /// `pose`: the cube rests in use mode, where the face fills the view,
    /// and the pane is behind the face.
    pub(crate) fn covers(&self, pose: &Pose, at: [f64; 2]) -> bool {
        let rests = self.turn.is_none() && self.step.is_none() && self.stand <= 0.0;
        rests && self.view.forwards() && super::space::depth(pose, at) > self.front_depth(pose)
    }

    /// The angle at which the faces show: the angle of the turn, and the
    /// rest pose of a cube that stands back.
    #[allow(dead_code, reason = "a stub until the next commit")]
    pub(crate) fn drawn_angle(&self) -> f64 {
        self.angle
    }

    /// The shadows of the cube on the floor, in the form of the shadows of
    /// panes: centre x and z, half width, and the lift with the yaw.
    #[allow(
        dead_code,
        clippy::unused_self,
        reason = "a stub until the next commit"
    )]
    pub(crate) fn shadows(&self, _ground: f64) -> Vec<[f64; 4]> {
        Vec::new()
    }

    /// The place of face `i` in the camera group: the cube group over the
    /// face.
    #[allow(dead_code, reason = "a stub until the next commit")]
    pub(crate) fn world(&self, i: usize) -> Matrix {
        mul4(
            &placement(self.centre(), self.yaw),
            &self.cube.face(i, self.drawn_angle()),
        )
    }

    /// The place of the mirror image of face `i` in the floor, `ground`
    /// below the eye, for a node of the face's size divided by its scale.
    #[allow(dead_code, reason = "a stub until the next commit")]
    pub(crate) fn mirror_world(&self, i: usize, _ground: f64) -> Matrix {
        self.world(i)
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
    pub(crate) fn faces(&mut self, eye: [f64; 2], _ground: f64) -> Vec<Command> {
        let eye = self.eye(eye);
        self.seen_from = eye;
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
            let up = self.health.get(i) == Some(&Health::Up);
            commands.push(update(
                &face_id(i),
                PropsUpdate {
                    transform: Some(Transform::from_matrix(self.cube.face(i, self.angle))),
                    visible: changed.then_some(shows),
                    forward: Some(up && self.view.forwards() && self.stand <= 0.0),
                    ..PropsUpdate::default()
                },
            ));
            commands.push(update(
                &fill_id(i),
                PropsUpdate {
                    transform: Some(Transform::from_matrix(self.cube.lifted(
                        i,
                        self.angle,
                        [0.0, 0.0],
                        FILL_LIFT,
                    ))),
                    visible: Some(shows && !up),
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
        let rests = self.rests() && !self.view.forwards();
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

    /// The health of face `i`.
    pub(crate) fn health_of(&self, face: usize) -> Health {
        self.health.get(face).cloned().unwrap_or(Health::Up)
    }

    /// Whether a face other than `face` works.
    pub(crate) fn others_up(&self, face: usize) -> bool {
        let up = |(i, h): (usize, &Health)| i != face && *h == Health::Up;
        self.health.iter().enumerate().any(up)
    }

    /// Whether every face works.
    pub(crate) fn works(&self) -> bool {
        self.health.iter().all(|h| *h == Health::Up)
    }

    /// Whether the face at the front failed to start.
    pub(crate) fn front_failed(&self) -> bool {
        matches!(self.health.get(self.front), Some(Health::Failed(_)))
    }

    /// The display name of the window manager of the faces.
    pub(crate) fn manager_name(&self) -> &str {
        &self.name
    }

    /// The window manager connected: every face works.
    fn connected(&mut self) {
        self.health.fill(Health::Up);
    }

    /// The face fails, with a reason.
    fn fail(&mut self, reason: &str) {
        self.health.fill(Health::Failed(reason.to_owned()));
    }

    /// When the construct looks again at a manager that starts: at the
    /// end of its time, or soon, to see whether it exited.
    fn start_deadline(&self, now: Instant) -> Option<Instant> {
        let starting = self.health.contains(&Health::Starting);
        starting.then(|| (self.since + crate::faces::LIMIT).min(now + POLL))
    }

    /// Looks whether the manager that starts exited or is late; then its
    /// process ends, the faces fail, and the reason comes back.
    fn check_start(&mut self, language: Language, now: Instant) -> Option<String> {
        if !self.health.contains(&Health::Starting) {
            return None;
        }
        let exited = matches!(self.child.try_wait(), Ok(Some(_)));
        if exited {
            std::thread::sleep(Duration::from_millis(50));
        }
        let last = self.tail.lock().map(|t| t.clone()).unwrap_or_default();
        let waited = now.saturating_duration_since(self.since);
        let reason = crate::faces::failure(language, &self.program, waited, exited, &last)?;
        end(&mut self.child);
        self.fail(&reason);
        Some(reason)
    }

    /// Starts the window manager again after it failed. Answers why it
    /// could not start.
    fn restart(&mut self, now: Instant) -> Result<(), String> {
        end(&mut self.child);
        let (child, tail) =
            launch(&self.command).map_err(|e| format!("cannot start {:?}: {e}", self.command))?;
        self.child = child;
        self.tail = tail;
        self.since = now;
        self.health.fill(Health::Starting);
        Ok(())
    }

    /// Whether the cube rests: no turn and no step runs.
    pub(crate) fn rests(&self) -> bool {
        self.turn.is_none() && self.step.is_none() && self.live.is_none()
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

    /// A press on a face, or on the floating plane of `window` on it, in
    /// navigate mode at a point of the screen. A press while the cube
    /// turns does nothing.
    pub(crate) fn press(&mut self, face: usize, window: Option<String>, at: [f64; 2]) {
        self.press = self.rests().then_some(Press {
            face,
            window,
            start: at,
            dragged: false,
        });
    }

    /// The pointer moved to `at` during a press: once it went farther
    /// than a click, the cube follows the hand, a view `width` wide being
    /// a quarter turn, without a threshold and without lag.
    pub(crate) fn drag(&mut self, at: [f64; 2], width: f64, now: Instant) -> Dragged {
        let dragged = drag_of(&mut self.press, at);
        if let Dragged::Yaw(dx) = dragged {
            let live = self.live.get_or_insert_with(|| Live {
                base: self.angle,
                width,
                samples: Vec::new(),
            });
            live.samples
                .retain(|(t, _)| now.saturating_duration_since(*t) <= SAMPLED);
            live.samples.push((now, at[0]));
            let reach = 2.0 * width;
            self.angle = live.base + yaw_of(dx.clamp(-reach, reach), width);
            self.view.act(Act::Turn(1));
        }
        dragged
    }

    /// The hand lets go: the cube settles on the nearest face over 320 ms,
    /// or goes on to the next one when the hand flicked.
    fn settle(&mut self, now: Instant) {
        let Some(live) = self.live.take() else {
            return;
        };
        let by = settle_by(self.angle - live.base, live.velocity(now));
        self.front = next_front(self.front, by, self.outputs.len());
        let to = Cube::turn_of(self.front);
        self.turn = Some(Turn::new(self.angle, to, now, SETTLE));
    }

    /// Forgets the press: after injected input during it, which is not
    /// the person. A cube that the hand held settles.
    pub(crate) fn forget_press(&mut self, now: Instant) {
        self.press = None;
        self.settle(now);
    }

    /// The release of a press: the cube settles if the hand held it, and
    /// the face of a click, a press that did not move, comes back.
    pub(crate) fn release(&mut self, now: Instant) -> Option<usize> {
        let press = self.press.take()?;
        self.settle(now);
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

    /// The share of their full lift at which the windows of face `i`
    /// float now for an eye at `eye` in the space of the cube: from the
    /// stand back and the turn, times how squarely the face looks at the
    /// eye, so that a face at a grazing angle floats them flat and no
    /// plane shows as a sliver beside the cube.
    fn float_share(&self, i: usize, eye: [f64; 3]) -> f64 {
        let facing = self.cube.facing(i, self.angle, eye).clamp(0.0, 1.0);
        self.stand.max(turned(self.angle)) * facing
    }

    /// The lifts at full float of the windows of face `i`: each starts at
    /// the lift of its place in the stack and grows until it clears the
    /// planes across the shared edges (`cleared`).
    fn lifts(&self, i: usize) -> Vec<f64> {
        let based = |face: usize| -> Vec<Lifted> {
            let windows = self.floats_of(face).unwrap_or_default();
            let count = windows.len();
            let lifted = |(k, w)| (w, float::lift(1.0, k, count));
            windows.iter().enumerate().map(lifted).collect()
        };
        let own = based(i);
        let sides = (i < FACES).then(|| (based((i + 1) % FACES), based((i + FACES - 1) % FACES)));
        let (left, right) = sides.unwrap_or_default();
        let width = self.cube.size_of(i)[0];
        (own.iter())
            .map(|(w, base)| cleared(w, *base, width, (&left, &right)))
            .collect()
    }

    /// The updates of the planes of the floating windows of face `i`:
    /// each at the place of its window, off the face by its lift, which
    /// grows with the stand back and with the turn.
    fn planes(&self, i: usize, shows: bool) -> Vec<Command> {
        let windows = self.floats_of(i).unwrap_or_default();
        let phase = self.float_share(i, self.seen_from);
        let lifts = self.lifts(i);
        let plane = |(k, window): (usize, &Floating)| {
            let lift = lifts.get(k).copied().unwrap_or_default() * phase;
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
        self.health.iter().all(|h| *h == Health::Up) && (moves || !self.view.forwards())
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
            self.settle(now);
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

    /// The face at the front and the number of faces.
    pub(crate) fn front_face(&self) -> (usize, usize) {
        (self.front, self.outputs.len())
    }

    /// The face that the label names: the nearest one while the cube
    /// turns, else the one at the front.
    pub(crate) fn label_face(&self) -> usize {
        nearest_face(self.angle, self.outputs.len())
    }

    /// Whether the cube turns, by a key or by the hand.
    pub(crate) fn turning(&self) -> bool {
        self.turn.is_some() || self.live.is_some()
    }

    /// How much of the accent each dot of the label has now.
    pub(crate) fn dot_weights(&self) -> Vec<f64> {
        dot_weights(self.angle, self.outputs.len())
    }

    /// Whether the hints stay hidden: while the cube steps, while a
    /// turn of use mode runs, and while the planes of windows land.
    pub(crate) fn hides_hints(&self) -> bool {
        let turns_in_use = self.turn.is_some() && self.view.mode() == chora_kit::modes::Mode::Use;
        self.step.is_some() || self.waits() || turns_in_use
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
    /// output at 1:1. From inside a pane, or on the way into one, the
    /// camera first leaves it, to where it stood before; a running glide
    /// ends at its goal. Refused while another client `xp` runs.
    pub(crate) fn open_cube(&mut self, faces: usize) -> Result<Value, ClientError> {
        let refuse = |m: String| Err(ClientError::Rpc(RpcError::new(RpcError::REJECTED, m)));
        if self.cube.is_some() {
            return refuse("a cube is open; cube.close first".into());
        }
        if !(1..=FACES).contains(&faces) {
            return refuse(format!("faces: {faces} is not 1 to {FACES}"));
        }
        if self.has_manager(MANAGER) || self.manager_runs()? {
            return refuse(format!("a client {MANAGER} runs already"));
        }
        if self.inside().is_some() {
            self.leave_window()?;
        }
        if let Some(glide) = self.glide.take() {
            self.pose = glide.to;
        }
        let size = [self.lens.w, self.lens.h];
        let outputs = self.add_outputs(faces, size)?;
        let flags: Vec<String> = outputs.iter().map(|o| format!("--output {o}")).collect();
        let manager = self.settings.value.cube_manager.clone();
        let command = format!("exec {manager} {}", flags.join(" "));
        let spawned = launch(&command);
        let (child, tail) = match spawned {
            Ok(started) => started,
            Err(err) => {
                self.remove_outputs(&outputs);
                return refuse(format!("cannot start {command:?}: {err}"));
            }
        };
        self.glide = None;
        self.pose.pitch = 0.0;
        let cube = Cube { size, caps: false };
        let mut host = self.host(outputs.clone(), (child, tail), (&manager, &command), cube);
        let mut commands = vec![create(CUBE, NodeKind::Group, PANES, PropsUpdate::default())];
        commands.extend(self.face_nodes(&outputs, &cube));
        commands.push(self.edge_node());
        commands.extend(host.faces([self.pose.x, self.pose.z], self.lens.ground));
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
    fn host(
        &self,
        outputs: Vec<String>,
        (child, tail): (Child, Tail),
        (manager, command): (&str, &str),
        cube: Cube,
    ) -> Host {
        let [fx, fz] = self.pose.forward();
        let reach = self.lens.focal + cube.size[0] / 2.0;
        let faces = outputs.len();
        let mut host = Host {
            shown: vec![false; faces],
            seen_from: [0.0; 3],
            floats: vec![None; faces],
            outputs,
            child,
            command: command.to_owned(),
            program: super::names::program_of(manager).to_owned(),
            name: super::names::cube_manager_name(manager),
            since: Instant::now(),
            tail,
            health: vec![Health::Starting; faces],
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
            live: None,
            step_back_key: None,
            owing: BTreeSet::new(),
            again: None,
            denied: BTreeSet::new(),
        };
        host.seen_from = host.eye([self.pose.x, self.pose.z]);
        host
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
            host.connected();
        }
        self.frame()
    }

    /// The time by which the construct looks again at a window manager.
    pub(crate) fn cube_deadline(&self) -> Option<Instant> {
        let cube = self
            .cube
            .as_ref()
            .and_then(|c| c.start_deadline(Instant::now()));
        let panes = self.manager_deadline();
        cube.into_iter().chain(panes).min()
    }

    /// The faces fail when their window manager exited or did not connect
    /// in time; the cube stays, and the card says why.
    pub(crate) fn check_cube(&mut self) -> Result<(), ClientError> {
        self.check_managers()?;
        let language = Language::of_session();
        let failed = (self.cube.as_mut()).and_then(|c| c.check_start(language, Instant::now()));
        if let Some(reason) = failed {
            eprintln!("chora-construct: the window manager of the cube failed: {reason}");
            self.frame()?;
        }
        Ok(())
    }

    /// Tries again: the window manager of the faces that failed starts
    /// anew.
    pub(crate) fn retry_cube(&mut self) -> Result<(), ClientError> {
        let Some(host) = &mut self.cube else {
            return Ok(());
        };
        if let Err(reason) = host.restart(Instant::now()) {
            eprintln!("chora-construct: the window manager of the cube: {reason}");
            host.fail(&reason);
        }
        self.frame()
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
        let taken = host.start_turn(by, duration, queue, chora_kit::clock::now());
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
        let now = chora_kit::clock::now();
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
        let ground = self.lens.ground;
        (self.cube.as_mut())
            .map(|h| h.faces(eye, ground))
            .unwrap_or_default()
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
        let [r, g, b, _] = self.styled().glass.strong;
        (outputs.iter().enumerate())
            .flat_map(|(i, output)| {
                let win = Uniform::Output(OutputTexture {
                    output: output.clone(),
                });
                [
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
                    ),
                    create(
                        &fill_id(i),
                        NodeKind::Rect,
                        CUBE,
                        PropsUpdate {
                            size: Some(cube.size_of(i)),
                            color: Some([r, g, b, 1.0]),
                            visible: Some(false),
                            input: Some(InputMode::Pass),
                            ..PropsUpdate::default()
                        },
                    ),
                ]
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
    pub(crate) fn add_outputs(
        &mut self,
        faces: usize,
        size: [f64; 2],
    ) -> Result<Vec<String>, ClientError> {
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

    pub(crate) fn remove_outputs(&mut self, outputs: &[String]) {
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
pub(crate) fn end(child: &mut Child) {
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
        window: None,
        start: [600.0, 300.0],
        dragged: false,
    };

    #[test]
    fn a_drag_beyond_a_click_is_the_yaw_from_the_press() {
        let mut held = Some(PRESS.clone());
        assert_eq!(drag_of(&mut held, [605.0, 300.0]), Dragged::Nothing);
        assert_eq!(drag_of(&mut held, [560.0, 300.0]), Dragged::Yaw(-40.0));
        assert_eq!(
            drag_of(&mut held, [602.0, 300.0]),
            Dragged::Yaw(2.0),
            "once it is a drag it stays one, from the press and not from the last move"
        );
        let mut none = None;
        assert_eq!(drag_of(&mut none, [0.0, 0.0]), Dragged::Nothing);
    }

    #[test]
    fn a_drag_on_a_floating_plane_pulls_its_window_instead_of_turning() {
        let mut held = Some(Press {
            window: Some("s1".into()),
            ..PRESS.clone()
        });
        assert_eq!(drag_of(&mut held, [605.0, 300.0]), Dragged::Nothing);
        assert_eq!(
            drag_of(&mut held, [100.0, 300.0]),
            Dragged::Pull("s1".into())
        );
    }

    fn floating(rect: [f64; 4]) -> Floating {
        Floating {
            node: "xp-win-s".into(),
            surface: "s".into(),
            rect,
        }
    }

    #[test]
    fn a_window_over_a_shared_edge_floats_until_it_clears_the_neighbour() {
        let width = 1280.0;
        let over_right = floating([1180.0, 100.0, 300.0, 200.0]);
        let over_left = floating([-150.0, 150.0, 300.0, 200.0]);
        let lift = cleared(&over_right, 100.0, width, (&[], &[(&over_left, 100.0)]));
        assert!(lift > 150.0, "past the reach of the neighbour: {lift}");
        let low = floating([-150.0, 500.0, 300.0, 200.0]);
        let apart = cleared(&over_right, 100.0, width, (&[], &[(&low, 100.0)]));
        assert!((apart - 100.0).abs() < 1e-9, "no overlap in height");
        let short = floating([-50.0, 150.0, 300.0, 200.0]);
        let clear = cleared(&over_right, 100.0, width, (&[], &[(&short, 100.0)]));
        assert!(
            (clear - 100.0).abs() < 1e-9,
            "the neighbour does not reach the plane"
        );
        let far = floating([-900.0, 150.0, 1000.0, 200.0]);
        let capped = cleared(&over_right, 100.0, width, (&[], &[(&far, 100.0)]));
        assert!(
            (capped - CLEAR_CAP).abs() < 1e-9,
            "at most the cap: {capped}"
        );
        let mirrored = cleared(&over_left, 100.0, width, (&[(&over_right, 100.0)], &[]));
        assert!(mirrored > 200.0, "across the left edge: {mirrored}");
    }

    #[test]
    fn a_face_at_a_grazing_angle_floats_its_windows_flat() {
        let mut host = host_at_rest();
        let eye = host.eye([0.0, 0.0]);
        let first = (1..90)
            .map(|k| -f64::from(k) * FRAC_PI_2 / 90.0)
            .find(|angle| host.cube.faces_eye(1, *angle, eye))
            .expect("face 1 comes into view");
        host.angle = first;
        let share = host.float_share(1, host.eye([0.0, 0.0]));
        assert!(share < 0.2, "a sliver beside the cube: {share} at {first}");
        host.angle = 0.0;
        host.stand = 1.0;
        let front = host.float_share(0, host.eye([0.0, 0.0]));
        assert!(front > 0.9, "the front face stood back: {front}");
        let _ = host.child.wait();
    }

    #[test]
    fn a_click_stays_within_eight_pixels_on_both_axes() {
        let dragged = |at: [f64; 2]| {
            let mut held = Some(PRESS.clone());
            drag_of(&mut held, at);
            held.is_some_and(|p| p.dragged)
        };
        assert!(!dragged([603.0, 297.0]));
        assert!(dragged([600.0, 320.0]), "up and down counts too");
        assert!(dragged([609.0, 300.0]));
        assert!(dragged([606.0, 306.0]), "the distance, not one axis");
    }

    fn host_at_rest() -> Host {
        host_of(1)
    }

    fn host_of(faces: usize) -> Host {
        let child = std::process::Command::new("true").spawn().expect("true");
        let cube = Cube {
            size: [1280.0, 800.0],
            caps: false,
        };
        Host {
            outputs: (1..=faces).map(|i| format!("VIRTUAL-{i}")).collect(),
            child,
            command: String::new(),
            program: String::new(),
            name: String::new(),
            since: Instant::now(),
            tail: Tail::default(),
            health: vec![Health::Up; faces],
            cube,
            at: [0.0, 832.0 + 640.0],
            yaw: 0.0,
            front: 0,
            angle: 0.0,
            turn: None,
            shown: vec![true; faces],
            seen_from: [0.0; 3],
            view: View::default(),
            far: 500.0,
            stand: 0.0,
            step: None,
            home: Pose::default(),
            focus: None,
            queued: false,
            switching: false,
            floats: vec![None; faces],
            landing: None,
            hover: None,
            press: None,
            live: None,
            step_back_key: None,
            owing: BTreeSet::new(),
            again: None,
            denied: BTreeSet::new(),
        }
    }

    #[test]
    fn the_resting_front_face_covers_the_panes_behind_it_in_use_mode() {
        let mut host = host_at_rest();
        let pose = Pose::default();
        assert!((host.front_depth(&pose) - 832.0).abs() < 1e-9);
        assert!(host.covers(&pose, [0.0, 2000.0]), "behind the face");
        assert!(!host.covers(&pose, [0.0, 700.0]), "in front of the face");
        host.stand = 1.0;
        assert!(!host.covers(&pose, [0.0, 2000.0]), "stood back");
        let _ = host.child.wait();
    }

    const QUARTER: f64 = FRAC_PI_2;

    fn degrees(d: f64) -> f64 {
        d.to_radians()
    }

    #[test]
    fn the_cube_follows_the_hand_by_the_width_of_the_view() {
        assert!(
            (yaw_of(1280.0, 1280.0) - QUARTER).abs() < 1e-9,
            "a width is 90 degrees"
        );
        assert!((yaw_of(-640.0, 1280.0) + QUARTER / 2.0).abs() < 1e-9);
        assert!(
            yaw_of(1.0, 1280.0) > 0.0,
            "no threshold: one pixel turns it"
        );
        assert!(yaw_of(-1.0, 1280.0) < 0.0);
    }

    #[test]
    fn a_release_settles_on_the_nearest_face_and_a_flick_goes_on() {
        assert_eq!(settle_by(degrees(-30.0), 0.0), 0, "30 degrees returns");
        assert_eq!(settle_by(degrees(-44.0), 0.0), 0);
        assert_eq!(settle_by(degrees(-50.0), 0.0), 1, "50 degrees goes on");
        assert_eq!(
            settle_by(degrees(50.0), 0.0),
            -1,
            "to the right, the face before"
        );
        assert_eq!(settle_by(degrees(-10.0), -1.6), 1, "a flick to the left");
        assert_eq!(settle_by(degrees(-10.0), -1.4), 0, "too slow for a flick");
        assert_eq!(settle_by(degrees(10.0), 1.6), -1, "a flick to the right");
        assert_eq!(
            settle_by(degrees(-50.0), -2.0),
            1,
            "a flick past 45 degrees goes one face"
        );
    }

    #[test]
    fn the_label_names_the_nearest_face_and_the_next_dot_fills_with_the_turn() {
        assert_eq!(nearest_face(degrees(-44.0), 4), 0);
        assert_eq!(nearest_face(degrees(-46.0), 4), 1);
        assert_eq!(nearest_face(degrees(30.0), 4), 0);
        assert_eq!(
            nearest_face(degrees(50.0), 4),
            3,
            "to the right of face 0 is face 3"
        );
        let close = |a: Vec<f64>, b: [f64; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-9);
        assert!(close(dot_weights(0.0, 4), [1.0, 0.0, 0.0, 0.0]));
        assert!(close(dot_weights(-QUARTER, 4), [0.0, 1.0, 0.0, 0.0]));
        assert!(close(dot_weights(-QUARTER / 2.0, 4), [0.5, 0.5, 0.0, 0.0]));
        assert!(close(dot_weights(QUARTER / 2.0, 4), [0.5, 0.0, 0.0, 0.5]));
    }

    #[test]
    fn a_turn_by_key_takes_360_ms_and_the_settle_of_a_drag_320_ms() {
        assert_eq!(TURN, Duration::from_millis(360));
        assert_eq!(SETTLE, Duration::from_millis(320));
        assert!((FLICK - 1.5).abs() < 1e-9);
    }

    #[test]
    fn a_drag_on_a_face_turns_the_cube_with_the_hand_and_settles_on_release() {
        let at = Instant::now();
        let ms = Duration::from_millis;
        let mut host = host_of(4);
        host.press(0, None, [600.0, 300.0]);
        host.drag([560.0, 300.0], 1280.0, at);
        assert!(
            (host.angle - yaw_of(-40.0, 1280.0)).abs() < 1e-9,
            "the cube follows at once: {}",
            host.angle
        );
        host.drag([100.0, 300.0], 1280.0, at + ms(400));
        assert!(
            (host.angle - yaw_of(-500.0, 1280.0)).abs() < 1e-9,
            "from the press, no lag"
        );
        let released = at + ms(900);
        assert_eq!(host.release(released), None, "a drag is no click");
        assert_eq!(host.front, 0, "35 degrees returns to the face of before");
        let turn = host.turn.expect("the cube settles");
        assert!(!turn.done(released + ms(300)));
        assert!(turn.done(released + ms(320)), "within 320 ms");
        assert!(turn.at(released + ms(320)).abs() < 1e-9, "back on face 0");
    }

    #[test]
    fn a_drag_past_45_degrees_settles_on_the_next_face() {
        let at = Instant::now();
        let ms = Duration::from_millis;
        let mut host = host_of(4);
        host.press(0, None, [600.0, 300.0]);
        host.drag([-111.0, 300.0], 1280.0, at);
        let released = at + ms(900);
        host.release(released);
        assert_eq!(host.front, 1, "50 degrees goes on to the next face");
        let turn = host.turn.expect("the cube settles");
        assert!(turn.done(released + ms(320)));
        assert!((turn.at(released + ms(320)) - Cube::turn_of(1)).abs() < 1e-9);
    }

    #[test]
    fn a_flick_goes_on_to_the_next_face_and_a_click_does_not_turn() {
        let at = Instant::now();
        let ms = Duration::from_millis;
        let mut host = host_of(4);
        host.press(0, None, [600.0, 300.0]);
        host.drag([540.0, 300.0], 1280.0, at + ms(10));
        host.drag([440.0, 300.0], 1280.0, at + ms(60));
        host.release(at + ms(70));
        assert_eq!(
            host.front, 1,
            "160 px in 50 ms is faster than 1.5 widths a second"
        );
        let mut host = host_of(4);
        host.press(0, None, [600.0, 300.0]);
        host.drag([604.0, 300.0], 1280.0, at);
        assert_eq!(
            host.release(at + ms(50)),
            Some(0),
            "a press that stays is a click"
        );
        assert!(host.turn.is_none() && host.angle == 0.0);
    }

    fn lens_of(host: &Host) -> super::super::space::Lens {
        super::super::space::Lens::new(host.cube.size[0], host.cube.size[1])
    }

    fn visibility(commands: &[Command], id: &str) -> Option<bool> {
        commands.iter().find_map(|c| match c {
            Command::Update { id: i, props } if i == id => props.visible,
            _ => None,
        })
    }

    fn at_point(m: &Matrix, [u, v]: [f64; 2]) -> [f64; 3] {
        [0, 1, 2].map(|r| m[r] * u + m[4 + r] * v + m[12 + r])
    }

    #[test]
    fn a_cube_that_stands_back_rests_14_degrees_off_square_toward_the_next_face() {
        let mut host = host_of(4);
        assert!(
            (REST - degrees(14.0)).abs() < 1e-12,
            "the rest pose is 14 degrees"
        );
        assert!(host.drawn_angle().abs() < 1e-12, "in use mode it is square");
        host.stand = 1.0;
        assert!(
            ((host.drawn_angle() - host.angle).abs() - REST).abs() < 1e-9,
            "stood back it is turned by 14 degrees: {}",
            host.drawn_angle()
        );
        let eye = host.eye([0.0, 0.0]);
        assert!(host.cube.faces_eye(0, host.drawn_angle(), eye), "the front");
        assert!(
            host.cube.faces_eye(1, host.drawn_angle(), eye),
            "the next face"
        );
        assert!(
            !host.cube.faces_eye(3, host.drawn_angle(), eye),
            "not the one before"
        );
        host.stand = 0.5;
        assert!(
            ((host.drawn_angle() - host.angle).abs() - REST / 2.0).abs() < 1e-9,
            "the step glides the offset away with the stand"
        );
    }

    #[test]
    fn the_rest_pose_goes_with_a_turn_by_key_or_by_hand() {
        let mut host = host_of(4);
        host.stand = 1.0;
        let rest = host.drawn_angle() - host.angle;
        host.press(0, None, [600.0, 300.0]);
        host.drag([560.0, 300.0], 1280.0, Instant::now());
        assert!((host.drawn_angle() - host.angle - rest).abs() < 1e-9);
        assert!(host.angle < 0.0, "the hand still turns the cube");
        host.angle = Cube::turn_of(1);
        assert!((host.drawn_angle() - host.angle - rest).abs() < 1e-9);
        let eye = host.eye([0.0, 0.0]);
        assert!(
            host.cube.faces_eye(2, host.drawn_angle(), eye),
            "the next face of face 1 shows"
        );
        let _ = host.child.wait();
    }

    #[test]
    fn a_cube_that_stepped_in_shows_its_face_at_one_to_one() {
        let host = host_of(4);
        let lens = lens_of(&host);
        let m = super::super::space::screen_matrix(&lens, &Pose::default(), &host.world(0));
        let [x0, y0] = super::super::space::project(&m, 0.0, 0.0);
        let [x1, y1] = super::super::space::project(&m, lens.w, lens.h);
        for (got, want) in [(x0, 0.0), (y0, 0.0), (x1, lens.w), (y1, lens.h)] {
            assert!((got - want).abs() < 1e-6, "{got} is not {want}");
        }
    }

    #[test]
    fn the_cube_has_the_floor_shadow_of_a_pane() {
        let mut host = host_of(4);
        host.stand = 1.0;
        let ground = 560.0;
        let shadows = host.shadows(ground);
        assert_eq!(shadows.len(), 2, "two ovals across each other");
        let centre = host.centre();
        for s in &shadows {
            assert!((s[0] - centre[0]).abs() < 1e-9 && (s[1] - centre[1]).abs() < 1e-9);
            assert!((s[2] - 640.0).abs() < 1e-9, "half the width of a face");
            assert!(s[3] >= 0.0 && s[3] < 1e6);
        }
        let yaw = |w: f64| (w.fract() / 0.99 - 0.5) * std::f64::consts::TAU;
        let across = (yaw(shadows[1][3]) - yaw(shadows[0][3])).rem_euclid(std::f64::consts::PI);
        assert!(
            (across - FRAC_PI_2).abs() < 1e-2,
            "a quarter apart: {across}"
        );
        let face = host.world(0);
        let towards = face[8].atan2(face[10]);
        let off = (yaw(shadows[0][3]) - towards).rem_euclid(FRAC_PI_2);
        assert!(
            off < 1e-2 || FRAC_PI_2 - off < 1e-2,
            "turned with the cube: {off}"
        );
    }

    #[test]
    fn the_mirror_image_of_a_face_hangs_under_the_floor() {
        let host = host_of(4);
        let ground = 560.0;
        let (face, mirror) = (host.world(0), host.mirror_world(0, ground));
        let k = 2.0;
        for p in [[0.0, 0.0], [1280.0, 0.0], [300.0, 800.0], [1280.0, 800.0]] {
            let (a, b) = (at_point(&face, p), at_point(&mirror, [p[0] / k, p[1] / k]));
            assert!((a[0] - b[0]).abs() < 1e-6 && (a[2] - b[2]).abs() < 1e-6);
            assert!((b[1] - (-2.0 * ground - a[1])).abs() < 1e-6, "{a:?} {b:?}");
        }
    }

    #[test]
    fn the_mirror_images_of_the_faces_that_show_come_with_the_step_back() {
        let mut host = host_of(4);
        host.shown = vec![false; 4];
        let in_use = host.faces([0.0, 0.0], 560.0);
        assert_ne!(
            visibility(&in_use, &mirror_id(0)),
            Some(true),
            "in use mode"
        );
        host.stand = 1.0;
        let back = host.faces([0.0, 0.0], 560.0);
        assert_eq!(visibility(&back, &mirror_id(0)), Some(true));
        assert_eq!(
            visibility(&back, &mirror_id(1)),
            Some(true),
            "the next face"
        );
        assert_ne!(
            visibility(&back, &mirror_id(3)),
            Some(true),
            "a face that looks away"
        );
    }
}
