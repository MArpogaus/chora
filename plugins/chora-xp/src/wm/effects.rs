//! The effects of the window manager and the frames that drive them:
//! wobbly windows while a title bar drags, and the magic lamp of minimize
//! and restore. The cube lives in [`super::desktops`].

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use chora_scene::{Command, PropsUpdate};

use super::Wm;
use crate::{control::Toggles, cube::Turn, decoration, lamp::Lamp, wobbly::Wobble};

/// How long the magic lamp takes, before slow motion.
const LAMP_TIME: Duration = Duration::from_millis(400);
/// How long the plugin goes without effects after the watchdog of the core
/// dropped them. Slow frames within ten seconds after a drop reset the
/// scene.
const CALM: Duration = Duration::from_secs(15);

/// The running effects.
pub struct Effects {
    /// Windows that wobble, by surface.
    pub wobbles: HashMap<String, Wobble>,
    /// Windows that minimize or restore, by surface.
    pub lamps: HashMap<String, Lamp>,
    /// The turn of the cube.
    pub turn: Option<Turn>,
    /// The effects that the Control Panel turned on.
    pub toggles: Toggles,
    /// The time of the last frame.
    last: Instant,
    /// No effects until then, after the watchdog dropped them.
    calm_until: Option<Instant>,
}

impl Effects {
    /// No effect runs yet; the toggles say which may.
    pub fn new(toggles: Toggles) -> Self {
        Self {
            wobbles: HashMap::new(),
            lamps: HashMap::new(),
            turn: None,
            toggles,
            last: Instant::now(),
            calm_until: None,
        }
    }
}

impl Wm {
    /// Whether effects may run: not for a while after the watchdog of the
    /// core dropped them.
    pub(super) fn effects_on(&self) -> bool {
        self.effects
            .calm_until
            .is_none_or(|until| Instant::now() >= until)
    }

    /// Whether an effect needs frames.
    pub fn animating(&self) -> bool {
        let e = &self.effects;
        !e.wobbles.is_empty()
            || !e.lamps.is_empty()
            || e.turn.is_some()
            || self.fading()
            || self.sliding()
    }

    /// Moves every effect one frame forward, in one transaction.
    pub fn frame(&mut self, now: Instant) -> Result<(), chora_scene::ipc::ClientError> {
        let dt = now
            .saturating_duration_since(self.effects.last)
            .as_secs_f64();
        self.effects.last = now;
        let mut commands = self.wobble_frame(dt);
        commands.extend(self.lamp_frame(now));
        commands.extend(self.cube_frame(now));
        commands.extend(self.fade_frame(now));
        commands.extend(self.slide_frame(now, dt));
        self.conn.apply(commands)
    }

    /// The window starts to wobble, grabbed at a point of the scene.
    pub(super) fn start_wobble(&mut self, surface: &str, grab: [f64; 2]) {
        let on = self.effects_on() && self.effects.toggles.wobbly;
        let Some(i) = self.index(surface).filter(|_| on) else {
            return;
        };
        let w = &self.windows[i];
        self.effects.last = Instant::now();
        let wobble = Wobble::new(w.area(), w.origin(), grab);
        self.effects.wobbles.insert(surface.into(), wobble);
    }

    /// The pointer lets go: the window shakes until it settles.
    pub(super) fn release_wobble(&mut self, surface: &str) {
        if let Some(wobble) = self.effects.wobbles.get_mut(surface) {
            wobble.release();
        }
    }

    /// The mesh of a wobbling window at its current place.
    pub(super) fn wobble_mesh(&self, surface: &str) -> Option<Command> {
        let wobble = self.effects.wobbles.get(surface)?;
        let w = &self.windows[self.index(surface)?];
        Some(mesh_update(surface, w.area(), wobble.mesh(w.origin())))
    }

    fn wobble_frame(&mut self, dt: f64) -> Vec<Command> {
        let mut commands = Vec::new();
        let mut settled = Vec::new();
        for (surface, wobble) in &mut self.effects.wobbles {
            let Some(w) = self.windows.iter().find(|w| &w.surface == surface) else {
                settled.push(surface.clone());
                continue;
            };
            wobble.step(w.origin(), dt);
            if wobble.settled(w.origin()) {
                settled.push(surface.clone());
                commands.push(w.unmesh());
            } else {
                commands.push(mesh_update(surface, w.area(), wobble.mesh(w.origin())));
            }
        }
        for surface in settled {
            self.effects.wobbles.remove(&surface);
        }
        commands
    }

    /// Minimizes a window into its task button through the magic lamp.
    pub(super) fn lamp_out(&mut self, surface: &str, button: [f64; 4]) -> Vec<Command> {
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        self.effects.wobbles.remove(surface);
        let (now, duration) = (Instant::now(), self.slow(LAMP_TIME));
        let calm = !self.effects_on() || !self.effects.toggles.lamp;
        let w = &mut self.windows[i];
        w.minimized = true;
        if calm {
            return vec![w.unmesh(), w.place()];
        }
        w.held = true;
        let lamp = match self.effects.lamps.remove(surface) {
            Some(mut lamp) if lamp.restore => {
                lamp.flip(now);
                lamp
            }
            Some(lamp) => lamp,
            None => Lamp::new(w.bounds(), button, duration, false),
        };
        let first = mesh_update(surface, w.area(), lamp.mesh(lamp.time(now)));
        self.effects.lamps.insert(surface.into(), lamp);
        vec![first]
    }

    /// Restores a minimized window out of its task button.
    pub(super) fn lamp_in(&mut self, surface: &str, button: [f64; 4]) -> Vec<Command> {
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        let (now, duration) = (Instant::now(), self.slow(LAMP_TIME));
        let calm = !self.effects_on() || !self.effects.toggles.lamp;
        let w = &mut self.windows[i];
        w.minimized = false;
        if calm {
            return vec![w.unmesh(), w.place()];
        }
        w.held = true;
        let lamp = match self.effects.lamps.remove(surface) {
            Some(mut lamp) if !lamp.restore => {
                lamp.flip(now);
                lamp
            }
            Some(lamp) => lamp,
            None => Lamp::new(w.bounds(), button, duration, true),
        };
        // The mesh first, so that the window shows inside the button.
        let mut update = mesh_update(surface, w.area(), lamp.mesh(lamp.time(now)));
        if let Command::Update { props, .. } = &mut update {
            props.visible = Some(true);
            props.transform = Some(w.transform());
        }
        self.effects.lamps.insert(surface.into(), lamp);
        vec![update]
    }

    fn lamp_frame(&mut self, now: Instant) -> Vec<Command> {
        let mut commands = Vec::new();
        let mut done = Vec::new();
        for (surface, lamp) in &self.effects.lamps {
            let Some(w) = self.windows.iter().find(|w| &w.surface == surface) else {
                done.push(surface.clone());
                continue;
            };
            let t = lamp.time(now);
            commands.push(mesh_update(surface, w.area(), lamp.mesh(t)));
            if t >= 1.0 {
                done.push(surface.clone());
            }
        }
        for surface in done {
            commands.extend(self.end_lamp(&surface));
        }
        commands
    }

    /// The end of a lamp: a minimized window hides, a restored one is
    /// flat again.
    pub(super) fn end_lamp(&mut self, surface: &str) -> Vec<Command> {
        self.effects.lamps.remove(surface);
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
        self.effects.calm_until = Some(Instant::now() + CALM);
        self.effects.wobbles.clear();
        let lamps: Vec<String> = self.effects.lamps.keys().cloned().collect();
        let mut commands = Vec::new();
        for surface in lamps {
            commands.extend(self.end_lamp(&surface));
        }
        if self.effects.turn.is_some() {
            commands.extend(self.finish_turn());
        }
        commands
    }

    /// A duration in the slow motion of the configuration.
    pub(super) fn slow(&self, duration: Duration) -> Duration {
        duration.mul_f64(self.config.slow.max(0.01))
    }
}

/// The mesh of a window group, with its size, so that it is a layer.
fn mesh_update(surface: &str, size: [f64; 2], mesh: chora_scene::Mesh) -> Command {
    Command::Update {
        id: decoration::group_id(surface),
        props: PropsUpdate {
            size: Some(size),
            mesh: Some(mesh),
            ..PropsUpdate::default()
        },
    }
}
