//! The effects of the window manager and the frames that drive them:
//! wobbly windows while a title bar drags, and the magic lamp of minimize
//! and restore. The cube lives in [`super::desktops`].

use std::time::{Duration, Instant};

use chora_kit::effects::{Driver, Pour, Target};
use chora_scene::Command;

use super::{Panel, Wm};
use crate::window::Window;
use crate::{control::Toggles, cube::Turn, decoration};

/// How long the magic lamp takes, before slow motion.
const LAMP_TIME: Duration = Duration::from_millis(400);

/// The running effects: the shared driver of wobbly windows and the magic
/// lamp, and the turn of the cube.
pub struct Effects {
    /// Wobbles and lamps, by surface, and dialogs, by group.
    pub driver: Driver,
    /// The turn of the cube.
    pub turn: Option<Turn>,
    /// The effects that the Control Panel turned on.
    pub toggles: Toggles,
    /// The time of the last frame.
    last: Instant,
}

impl Effects {
    /// No effect runs yet; the toggles say which may.
    pub fn new(toggles: Toggles) -> Self {
        Self {
            driver: Driver::new(&toggles.kinds()),
            turn: None,
            toggles,
            last: Instant::now(),
        }
    }

    /// Takes the toggles of the settings.
    pub fn set_toggles(&mut self, toggles: Toggles) {
        self.driver.set(&toggles.kinds());
        self.toggles = toggles;
    }
}

impl Wm {
    /// Whether effects may run: not for a while after the watchdog of the
    /// core dropped them.
    pub(super) fn effects_on(&self) -> bool {
        self.effects.driver.calm()
    }

    /// Whether an effect needs frames.
    pub fn animating(&self) -> bool {
        self.effects.driver.animating()
            || self.effects.turn.is_some()
            || self.fading()
            || self.sliding()
            || self.resize_due()
    }

    /// Moves every effect one frame forward, in one transaction.
    pub fn frame(&mut self, now: Instant) -> Result<(), chora_scene::ipc::ClientError> {
        let dt = now
            .saturating_duration_since(self.effects.last)
            .as_secs_f64();
        self.effects.last = now;
        let mut commands = self.resize_frame();
        let (windows, panels) = (&self.windows, &self.panels);
        let frame = (self.effects.driver).tick(now, |key| target_of(windows, panels, key));
        commands.extend(frame.commands);
        for surface in frame.lamps_done {
            commands.extend(self.end_lamp(&surface));
        }
        commands.extend(self.cube_frame(now));
        commands.extend(self.fade_frame(now));
        commands.extend(self.slide_frame(now, dt));
        self.apply(commands)
    }

    /// The group that wobbles for a key, a surface or the group of a
    /// dialog, with its top-left corner in the scene and its size.
    fn wobble_target(&self, key: &str) -> Option<Target> {
        target_of(&self.windows, &self.panels, key)
    }

    /// A window or a dialog starts to wobble, grabbed at a point of the
    /// scene.
    pub(super) fn start_wobble(&mut self, key: &str, grab: [f64; 2]) {
        if let Some(target) = self.wobble_target(key) {
            self.effects.driver.grab(key, &target, grab);
        }
    }

    /// The springs of a window that wobbles, for the window manager that
    /// gets it with its drag.
    pub(super) fn wobble_springs(&self, key: &str) -> Option<serde_json::Value> {
        let target = self.wobble_target(key)?;
        self.effects.driver.springs(key, &target)
    }

    /// The pointer lets go: the window shakes until it settles.
    pub(super) fn release_wobble(&mut self, surface: &str) {
        self.effects.driver.release(surface);
    }

    /// The mesh of a wobbling window or dialog at its current place, with
    /// the grid advanced to now.
    pub(super) fn wobble_mesh(&mut self, key: &str) -> Option<Command> {
        let target = self.wobble_target(key)?;
        self.effects.driver.motion(key, &target)
    }

    /// Minimizes a window into its task button through the magic lamp.
    pub(super) fn lamp_out(&mut self, surface: &str, button: [f64; 4]) -> Vec<Command> {
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        self.effects.driver.cancel_wobble(surface);
        let duration = self.slow(LAMP_TIME);
        let w = &mut self.windows[i];
        w.minimized = true;
        let pour = Pour {
            bounds: w.bounds(),
            button,
            duration,
        };
        let target = self.wobble_target(surface);
        let first = target.and_then(|t| self.effects.driver.lamp_out(surface, &t, pour));
        let w = &mut self.windows[i];
        let Some(first) = first else {
            return vec![w.unmesh(), w.place()];
        };
        w.held = true;
        vec![first]
    }

    /// Restores a minimized window out of its task button.
    pub(super) fn lamp_in(&mut self, surface: &str, button: [f64; 4]) -> Vec<Command> {
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        let duration = self.slow(LAMP_TIME);
        let w = &mut self.windows[i];
        w.minimized = false;
        let pour = Pour {
            bounds: w.bounds(),
            button,
            duration,
        };
        let target = self.wobble_target(surface);
        let first = target.and_then(|t| self.effects.driver.lamp_in(surface, &t, pour));
        let w = &mut self.windows[i];
        let Some(mut update) = first else {
            return vec![w.unmesh(), w.place()];
        };
        w.held = true;
        if let Command::Update { props, .. } = &mut update {
            props.visible = Some(true);
            props.transform = Some(w.transform());
        }
        vec![update]
    }

    /// The end of a lamp: a minimized window hides, a restored one is
    /// flat again.
    pub(super) fn end_lamp(&mut self, surface: &str) -> Vec<Command> {
        self.effects.driver.cancel(surface);
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        let w = &mut self.windows[i];
        w.held = false;
        vec![w.unmesh(), w.place()]
    }

    /// The watchdog of the core removed all meshes because frames were
    /// slow: every effect ends at once.
    pub(super) fn effects_dropped(&mut self) -> Vec<Command> {
        eprintln!("chora-xp: frames were slow, the core dropped the effects");
        let mut commands = Vec::new();
        for surface in self.effects.driver.dropped() {
            commands.extend(self.end_lamp(&surface));
        }
        if self.effects.turn.is_some() {
            commands.extend(self.finish_turn());
        }
        commands
    }

    /// A duration in the slow motion of the configuration.
    pub(super) fn slow(&self, duration: Duration) -> Duration {
        duration.mul_f64(self.slow.max(0.01))
    }
}

/// The node that wobbles for a key, a surface or the group of a dialog,
/// with its top-left corner in the scene and its size.
fn target_of(windows: &[Window], panels: &[Panel], key: &str) -> Option<Target> {
    if let Some(w) = windows.iter().find(|w| w.surface == key) {
        return Some(Target {
            node: decoration::group_id(key),
            origin: w.origin(),
            size: w.area(),
            set_size: true,
        });
    }
    let dialog = panels
        .iter()
        .map(Panel::window_ref)
        .find(|d| d.group() == key)?;
    let (origin, size) = dialog.layer();
    Some(Target {
        node: key.into(),
        origin,
        size,
        set_size: true,
    })
}
