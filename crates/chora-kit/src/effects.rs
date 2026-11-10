//! The driver of the window effects for a window manager: wobbly windows
//! while a window is dragged, and the magic lamp of minimize and restore.
//!
//! The driver turns the hooks of the manager (a grab, a move, a release, a
//! minimize, a restore and a frame) into `mesh` updates of the nodes that
//! the manager owns itself, so that no request to another client is
//! needed. A manager opts in with the setting `effects: [wobbly, lamp]`
//! and sends the commands that the hooks return.

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use chora_scene::{Command, Mesh, PropsUpdate};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{lamp::Lamp, wobbly::Wobble};

/// How long the driver goes without effects after the watchdog of the
/// core dropped them. Slow frames within ten seconds after a drop reset the
/// scene.
const CALM: Duration = Duration::from_secs(15);

/// An effect that a window manager can turn on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// A dragged window bends like jelly.
    Wobbly,
    /// A minimized window pours into its task button.
    Lamp,
}

/// The node that an effect bends: a window of the manager.
#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    /// The node that gets the mesh: a surface, or the group of a window.
    pub node: String,
    /// The top-left corner of the window in the scene.
    pub origin: [f64; 2],
    /// The width and height of the window.
    pub size: [f64; 2],
    /// Whether the mesh update also sets the `size` of the node, as for a
    /// group; a surface has its size from its content.
    pub set_size: bool,
}

/// The way of a lamp.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pour {
    /// The window in the scene: x, y, width and height.
    pub bounds: [f64; 4],
    /// The task button in the scene.
    pub button: [f64; 4],
    /// How long the lamp takes.
    pub duration: Duration,
}

/// What a frame did.
#[derive(Debug, Default, PartialEq)]
pub struct Frame {
    /// The updates of the meshes.
    pub commands: Vec<Command>,
    /// The keys whose lamp ended and left the driver; the manager hides
    /// or shows their windows.
    pub lamps_done: Vec<String>,
}

/// The running effects of one window manager.
#[derive(Default)]
pub struct Driver {
    wobbles: HashMap<String, Wobble>,
    /// The origins in 3D of the wobbles of `grab_3d`, as last moved.
    origins: HashMap<String, [f64; 3]>,
    lamps: HashMap<String, Lamp>,
    wobbly: bool,
    lamp: bool,
    calm_until: Option<Instant>,
}

impl Driver {
    /// A driver with the effects of the setting; none for an empty list.
    #[must_use]
    pub fn new(kinds: &[Kind]) -> Self {
        let mut driver = Self::default();
        driver.set(kinds);
        driver
    }

    /// Takes the effects of the setting.
    pub fn set(&mut self, kinds: &[Kind]) {
        self.wobbly = kinds.contains(&Kind::Wobbly);
        self.lamp = kinds.contains(&Kind::Lamp);
    }

    /// Whether effects may run: not for a while after the watchdog of the
    /// core dropped them.
    #[must_use]
    pub fn calm(&self) -> bool {
        self.calm_until.is_none_or(|until| Instant::now() >= until)
    }

    /// Whether a running effect needs frames.
    #[must_use]
    pub fn animating(&self) -> bool {
        !self.wobbles.is_empty() || !self.lamps.is_empty()
    }

    /// A window is grabbed at a point of the scene: it starts to wobble.
    pub fn grab(&mut self, key: &str, target: &Target, point: [f64; 2]) {
        if self.wobbly && self.calm() {
            let wobble = Wobble::new(target.size, target.origin, point);
            self.wobbles.insert(key.into(), wobble);
        }
    }

    /// A window in 3D is grabbed: it starts to wobble with a depth. The
    /// origin and the grab are in the coordinates of the space of the
    /// window, and `view` is the x and y of the direction of the view, as
    /// [`Wobble::new_3d`] takes them. The frames of the wobble then use
    /// the origin of [`Self::motion_3d`], not the one of the target.
    #[allow(clippy::too_many_arguments, reason = "the window, and the grab in 3D")]
    pub fn grab_3d(
        &mut self,
        key: &str,
        target: &Target,
        origin: [f64; 3],
        grab: [f64; 3],
        view: [f64; 2],
    ) {
        if self.wobbly && self.calm() {
            let wobble = Wobble::new_3d(target.size, origin, grab, view);
            self.wobbles.insert(key.into(), wobble);
            self.origins.insert(key.into(), origin);
        }
    }

    /// A window of [`Self::grab_3d`] moved to an origin in 3D: its mesh
    /// with a depth, with the grid advanced to now. `None` when it does
    /// not wobble.
    pub fn motion_3d(&mut self, key: &str, target: &Target, origin: [f64; 3]) -> Option<Command> {
        let wobble = self.wobbles.get_mut(key)?;
        wobble.advance_3d(origin, Instant::now());
        self.origins.insert(key.into(), origin);
        Some(mesh(target, wobble.mesh([origin[0], origin[1]])))
    }

    /// Whether a lamp runs for a window.
    #[must_use]
    pub fn has_lamp(&self, key: &str) -> bool {
        self.lamps.contains_key(key)
    }

    /// Ends the wobble of a window without a mesh update; true when it
    /// wobbled. The manager removes the mesh itself.
    pub fn cancel_wobble(&mut self, key: &str) -> bool {
        self.origins.remove(key);
        self.wobbles.remove(key).is_some()
    }

    /// Ends the wobble and the lamp of a window without a mesh update.
    pub fn cancel(&mut self, key: &str) {
        self.wobbles.remove(key);
        self.origins.remove(key);
        self.lamps.remove(key);
    }

    /// Ends every wobble and answers their keys.
    pub fn take_wobbles(&mut self) -> Vec<String> {
        self.origins.clear();
        std::mem::take(&mut self.wobbles).into_keys().collect()
    }

    /// Ends every lamp and answers their keys.
    pub fn take_lamps(&mut self) -> Vec<String> {
        std::mem::take(&mut self.lamps).into_keys().collect()
    }

    /// Ends the wobble of a window that leaves the manager, for example
    /// to another window manager: the update that removes the mesh of its
    /// node, if it wobbled.
    pub fn unbend(&mut self, key: &str, node: &str) -> Option<Command> {
        self.cancel_wobble(key).then(|| unmesh(node))
    }

    /// Whether a window wobbles.
    #[must_use]
    pub fn wobbles(&self, key: &str) -> bool {
        self.wobbles.contains_key(key)
    }

    /// The window moved: its mesh at the new place, with the grid
    /// advanced to now. `None` when it does not wobble.
    pub fn motion(&mut self, key: &str, target: &Target) -> Option<Command> {
        let wobble = self.wobbles.get_mut(key)?;
        wobble.advance(target.origin, Instant::now());
        Some(mesh(target, wobble.mesh(target.origin)))
    }

    /// The pointer lets go: the window shakes until it settles.
    pub fn release(&mut self, key: &str) {
        if let Some(wobble) = self.wobbles.get_mut(key) {
            wobble.release();
        }
    }

    /// Starts the lamp that pours a window into its task button, or turns
    /// a running lamp around. The first mesh; `None` when the lamp is off
    /// or the core dropped the effects, and the manager hides the window
    /// at once.
    pub fn lamp_out(&mut self, key: &str, target: &Target, pour: Pour) -> Option<Command> {
        self.lamp(key, target, pour, false)
    }

    /// Starts the lamp that pours a window out of its task button, or turns
    /// a running lamp around. The first mesh; `None` when the lamp is off
    /// or the core dropped the effects, and the manager shows the window
    /// at once.
    pub fn lamp_in(&mut self, key: &str, target: &Target, pour: Pour) -> Option<Command> {
        self.lamp(key, target, pour, true)
    }

    fn lamp(&mut self, key: &str, target: &Target, pour: Pour, restore: bool) -> Option<Command> {
        if !(self.lamp && self.calm()) {
            return None;
        }
        let now = Instant::now();
        let lamp = match self.lamps.remove(key) {
            Some(mut lamp) if lamp.restore != restore => {
                lamp.flip(now);
                lamp
            }
            Some(lamp) => lamp,
            None => Lamp::new(pour.bounds, pour.button, pour.duration, restore),
        };
        let first = mesh(target, lamp.mesh(lamp.time(now)));
        self.lamps.insert(key.into(), lamp);
        Some(first)
    }

    /// Moves every effect one frame forward. `find` gives the target of a
    /// key; an effect whose window is gone ends.
    pub fn tick(&mut self, now: Instant, find: impl Fn(&str) -> Option<Target>) -> Frame {
        let mut frame = Frame::default();
        let mut settled = Vec::new();
        for (key, wobble) in &mut self.wobbles {
            let Some(target) = find(key) else {
                settled.push(key.clone());
                continue;
            };
            let origin = if let Some(&origin) = self.origins.get(key) {
                wobble.advance_3d(origin, now);
                [origin[0], origin[1]]
            } else {
                wobble.advance(target.origin, now);
                target.origin
            };
            if wobble.settled() {
                settled.push(key.clone());
                frame.commands.push(unmesh(&target.node));
            } else {
                frame.commands.push(mesh(&target, wobble.mesh(origin)));
            }
        }
        for key in settled {
            self.wobbles.remove(&key);
            self.origins.remove(&key);
        }
        for (key, lamp) in &self.lamps {
            let Some(target) = find(key) else {
                frame.lamps_done.push(key.clone());
                continue;
            };
            let time = lamp.time(now);
            frame.commands.push(mesh(&target, lamp.mesh(time)));
            if time >= 1.0 {
                frame.lamps_done.push(key.clone());
            }
        }
        for key in &frame.lamps_done {
            self.lamps.remove(key);
        }
        frame
    }

    /// The watchdog of the core removed all meshes because frames were
    /// slow: every wobble and lamp ends and the driver goes calm for a
    /// while. The keys of the lamps come back, so that the manager shows
    /// or hides their windows.
    pub fn dropped(&mut self) -> Vec<String> {
        self.calm_until = Some(Instant::now() + CALM);
        self.wobbles.clear();
        self.origins.clear();
        self.take_lamps()
    }
}

/// The mesh of a target.
fn mesh(target: &Target, mesh: Mesh) -> Command {
    Command::Update {
        id: target.node.clone(),
        props: PropsUpdate {
            size: target.set_size.then_some(target.size),
            mesh: Some(mesh),
            ..PropsUpdate::default()
        },
    }
}

/// Removes the mesh of a node, at the end of an effect.
fn unmesh(node: &str) -> Command {
    Command::Update {
        id: node.into(),
        props: PropsUpdate {
            mesh: Some(Mesh {
                cols: 1,
                rows: 1,
                points: Vec::new(),
                depth: Vec::new(),
            }),
            ..PropsUpdate::default()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target() -> Target {
        Target {
            node: "w".into(),
            origin: [100.0, 100.0],
            size: [200.0, 100.0],
            set_size: false,
        }
    }

    fn at(x: f64) -> Target {
        Target {
            origin: [x, 100.0],
            ..target()
        }
    }

    fn has_mesh(command: &Command) -> bool {
        matches!(command, Command::Update { id, props }
            if id == "w" && props.mesh.as_ref().is_some_and(|m| !m.points.is_empty()))
    }

    /// A window grabbed in 3D and moved away from the eye wobbles with a
    /// depth, also in the frames after the move, and settles flat.
    #[test]
    fn a_window_grabbed_in_3d_wobbles_with_a_depth() {
        let mut driver = Driver::new(&[Kind::Wobbly]);
        let target = target();
        driver.grab_3d("w", &target, [0.0; 3], [100.0, 50.0, 0.0], [0.0; 2]);
        let mut deep = false;
        for step in 1..=10 {
            let origin = [0.0, 0.0, f64::from(step) * 40.0];
            std::thread::sleep(std::time::Duration::from_millis(12));
            let Some(Command::Update { props, .. }) = driver.motion_3d("w", &target, origin) else {
                panic!("no mesh");
            };
            let mesh = props.mesh.expect("a mesh");
            deep |= mesh.depth.iter().any(|d| d.abs() > 0.5);
        }
        assert!(deep, "the grid lags in depth");
        driver.release("w");
        let start = Instant::now();
        let mut now = start;
        while driver.wobbles("w") {
            now += std::time::Duration::from_millis(16);
            let frame = driver.tick(now, |_| Some(target.clone()));
            assert!(
                now - start < std::time::Duration::from_secs(10),
                "it settles"
            );
            drop(frame);
        }
    }

    #[test]
    fn a_grab_move_and_release_wobble_and_settle() {
        let mut driver = Driver::new(&[Kind::Wobbly]);
        driver.grab("w", &target(), [200.0, 110.0]);
        assert!(driver.animating());
        let moved = driver.motion("w", &at(160.0)).expect("a wobbling window");
        assert!(has_mesh(&moved), "{moved:?}");
        driver.release("w");
        let mut now = Instant::now();
        let mut last = Vec::new();
        for _ in 0..600 {
            now += Duration::from_millis(16);
            last = driver.tick(now, |_| Some(at(160.0))).commands;
            if !driver.animating() {
                break;
            }
            assert!(last.iter().all(has_mesh), "{last:?}");
        }
        assert!(!driver.animating(), "the window settles");
        assert!(
            matches!(&last[..], [Command::Update { props, .. }]
                if props.mesh.as_ref().is_some_and(|m| m.points.is_empty())),
            "an unmesh at the end: {last:?}"
        );
    }

    #[test]
    fn without_the_opt_in_nothing_wobbles() {
        let mut driver = Driver::new(&[Kind::Lamp]);
        driver.grab("w", &target(), [200.0, 110.0]);
        assert!(!driver.animating());
        assert!(driver.motion("w", &at(160.0)).is_none());
    }

    #[test]
    fn a_surface_gets_no_size_and_a_group_does() {
        let mut driver = Driver::new(&[Kind::Wobbly]);
        driver.grab("w", &target(), [200.0, 110.0]);
        let flat = driver.motion("w", &at(150.0));
        assert!(matches!(flat, Some(Command::Update { props, .. }) if props.size.is_none()));
        let group = Target {
            set_size: true,
            ..at(150.0)
        };
        let sized = driver.motion("w", &group);
        assert!(
            matches!(sized, Some(Command::Update { props, .. }) if props.size == Some([200.0, 100.0]))
        );
    }

    fn pour() -> Pour {
        Pour {
            bounds: [100.0, 100.0, 200.0, 100.0],
            button: [0.0, 500.0, 80.0, 20.0],
            duration: Duration::from_millis(100),
        }
    }

    #[test]
    fn the_lamp_runs_to_its_end_and_reports_it() {
        let mut driver = Driver::new(&[Kind::Lamp]);
        let first = driver.lamp_out("w", &target(), pour());
        assert!(first.as_ref().is_some_and(has_mesh));
        assert!(driver.has_lamp("w"));
        let frame = driver.tick(Instant::now() + Duration::from_secs(1), |_| Some(target()));
        assert_eq!(frame.lamps_done, ["w"]);
        assert!(!driver.animating(), "the driver ends the lamp itself");
        let off = Driver::new(&[]).lamp_out("w", &target(), pour());
        assert!(off.is_none(), "no lamp without the opt-in");
    }

    #[test]
    fn a_lamp_turned_around_at_its_start_ends_at_once() {
        let mut driver = Driver::new(&[Kind::Lamp]);
        driver.lamp_out("w", &target(), pour());
        driver.lamp_in("w", &target(), pour());
        let later = Instant::now() + Duration::from_millis(50);
        let frame = driver.tick(later, |_| Some(target()));
        assert_eq!(frame.lamps_done, ["w"], "flipped, not a second lamp");
    }

    #[test]
    fn a_drop_ends_the_lamps_and_names_them() {
        let mut driver = Driver::new(&[Kind::Lamp]);
        driver.lamp_out("w", &target(), pour());
        assert_eq!(driver.dropped(), ["w"]);
        assert!(!driver.animating(), "nothing animates after a drop");
    }

    #[test]
    fn a_wobble_that_leaves_loses_its_mesh() {
        let mut driver = Driver::new(&[Kind::Wobbly]);
        driver.grab("w", &target(), [200.0, 110.0]);
        assert!(matches!(
            driver.unbend("w", "w"),
            Some(Command::Update { .. })
        ));
        assert!(driver.unbend("w", "w").is_none());
        assert!(!driver.animating());
    }

    #[test]
    fn a_dropped_driver_stays_calm() {
        let mut driver = Driver::new(&[Kind::Wobbly, Kind::Lamp]);
        driver.grab("w", &target(), [200.0, 110.0]);
        assert_eq!(driver.dropped(), Vec::<String>::new());
        assert!(!driver.animating() && !driver.calm());
        driver.grab("w", &target(), [200.0, 110.0]);
        assert!(!driver.animating());
    }
}
