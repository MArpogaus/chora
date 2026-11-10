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
    pub layer: bool,
}

/// What a frame did.
#[derive(Debug, Default, PartialEq)]
pub struct Frame {
    /// The updates of the meshes.
    pub commands: Vec<Command>,
    /// The nodes whose lamp ended; the manager hides or shows them.
    pub lamps_done: Vec<String>,
}

/// The running effects of one window manager.
#[derive(Default)]
pub struct Driver {
    /// Windows that wobble, by key.
    pub wobbles: HashMap<String, Wobble>,
    /// Windows that minimize or restore, by key.
    pub lamps: HashMap<String, Lamp>,
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

    /// Whether wobbly windows are on.
    #[must_use]
    pub fn wobbly(&self) -> bool {
        self.wobbly
    }

    /// Whether the magic lamp is on.
    #[must_use]
    pub fn lamp(&self) -> bool {
        self.lamp
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

    /// Starts or turns around the lamp of a window: it goes into its task
    /// button, or with `restore` comes out of it. Returns the first mesh;
    /// `None` when the lamp is off or the core dropped the effects, and
    /// the manager hides or shows the window at once. `fresh` is the lamp
    /// to start when none runs for the window.
    pub fn lamp_toward(&mut self, key: &str, target: &Target, fresh: Lamp) -> Option<Command> {
        if !(self.lamp && self.calm()) {
            return None;
        }
        let now = Instant::now();
        let lamp = match self.lamps.remove(key) {
            Some(mut lamp) if lamp.restore != fresh.restore => {
                lamp.flip(now);
                lamp
            }
            Some(lamp) => lamp,
            None => fresh,
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
            wobble.advance(target.origin, now);
            if wobble.settled() {
                settled.push(key.clone());
                frame.commands.push(unmesh(&target.node));
            } else {
                frame
                    .commands
                    .push(mesh(&target, wobble.mesh(target.origin)));
            }
        }
        for key in settled {
            self.wobbles.remove(&key);
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
        frame
    }

    /// The watchdog of the core removed all meshes because frames were
    /// slow: every wobble ends and the driver goes calm for a while. The
    /// keys of the running lamps come back, so that the manager ends them.
    pub fn dropped(&mut self) -> Vec<String> {
        self.calm_until = Some(Instant::now() + CALM);
        self.wobbles.clear();
        self.lamps.keys().cloned().collect()
    }
}

/// The mesh of a target.
fn mesh(target: &Target, mesh: Mesh) -> Command {
    Command::Update {
        id: target.node.clone(),
        props: PropsUpdate {
            size: target.layer.then_some(target.size),
            mesh: Some(mesh),
            ..PropsUpdate::default()
        },
    }
}

/// Removes the mesh of a node, at the end of an effect.
#[must_use]
pub fn unmesh(node: &str) -> Command {
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
            layer: false,
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
        let off = Driver::new(&[]);
        assert!(!off.wobbly() && !off.lamp());
    }

    #[test]
    fn a_surface_gets_no_size_and_a_group_does() {
        let mut driver = Driver::new(&[Kind::Wobbly]);
        driver.grab("w", &target(), [200.0, 110.0]);
        let flat = driver.motion("w", &at(150.0));
        assert!(matches!(flat, Some(Command::Update { props, .. }) if props.size.is_none()));
        let group = Target {
            layer: true,
            ..at(150.0)
        };
        let sized = driver.motion("w", &group);
        assert!(
            matches!(sized, Some(Command::Update { props, .. }) if props.size == Some([200.0, 100.0]))
        );
    }

    #[test]
    fn the_lamp_runs_to_its_end_and_reports_it() {
        let mut driver = Driver::new(&[Kind::Lamp]);
        let bounds = [100.0, 100.0, 200.0, 100.0];
        let lamp = Lamp::new(
            bounds,
            [0.0, 500.0, 80.0, 20.0],
            Duration::from_millis(100),
            false,
        );
        let first = driver.lamp_toward("w", &target(), lamp);
        assert!(first.as_ref().is_some_and(has_mesh));
        let frame = driver.tick(Instant::now() + Duration::from_secs(1), |_| Some(target()));
        assert_eq!(frame.lamps_done, ["w"]);
        let lamp = Lamp::new(bounds, bounds, Duration::from_millis(100), false);
        let calm = Driver::new(&[]).lamp_toward("w", &target(), lamp);
        assert!(calm.is_none(), "no lamp without the opt-in");
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
