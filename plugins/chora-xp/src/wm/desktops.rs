//! Four desktops on the faces of a cube. Each desktop is a group with its
//! windows; one shows at a time. A turn shows the faces as layers in
//! perspective, and the windows fly off their faces as layers of their
//! own. After the turn the new desktop is flat again.

use std::time::{Duration, Instant};

use chora_scene::{Command, NodeKind, PropsUpdate, Transform};

use super::{WALLPAPERS, Wm};
use crate::{
    cube::{self, DESKTOPS, Face, Turn, View},
    decoration,
    taskbar::Bar,
    wallpaper,
};

/// How long a quarter turn takes, before slow motion.
const TURN_TIME: Duration = Duration::from_millis(800);
/// The color behind the cube: the background of the XP desktop without a
/// wallpaper.
const BACKGROUND: [f64; 4] = [0.227, 0.431, 0.647, 1.0];
const CUBE_BACKGROUND: &str = "xp-cube-bg";

/// The group of a desktop.
pub fn desk_id(desktop: usize) -> String {
    format!("xp-desk-{desktop}")
}

fn face_id(desktop: usize) -> String {
    format!("xp-face-{desktop}")
}

impl Wm {
    /// Turns the cube to a desktop the short way round.
    pub(super) fn turn_to(&mut self, desktop: usize) -> Vec<Command> {
        match (desktop + DESKTOPS - self.desktop) % DESKTOPS {
            0 => Vec::new(),
            3 => self.turn(-1),
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)] // below 4
            steps => self.turn(steps as i32),
        }
    }

    /// Moves the focused window to the next desktop and turns the cube
    /// with it.
    pub(super) fn carry(&mut self, steps: i32) -> Vec<Command> {
        if self.effects.turn.is_some() {
            return Vec::new();
        }
        let target = cube::offset(self.desktop, steps);
        let Some(i) = self.focused.as_deref().and_then(|s| self.index(s)) else {
            return self.turn(steps);
        };
        self.windows[i].desktop = target;
        let mut commands = vec![Command::Move {
            id: decoration::group_id(&self.windows[i].surface),
            parent: Some(desk_id(target)),
            index: None,
        }];
        commands.extend(self.turn(steps));
        commands
    }

    /// Starts a turn of the cube by some quarter turns: the faces with the
    /// wallpaper go below the windows of each desktop.
    pub(super) fn turn(&mut self, steps: i32) -> Vec<Command> {
        if self.effects.turn.is_some() || steps == 0 {
            return Vec::new();
        }
        let Some(area) = self.bars.first().map(Bar::area) else {
            return Vec::new();
        };
        if !self.effects_on() {
            return self.show_desktop(cube::offset(self.desktop, steps));
        }
        let mut commands = self.settle_for_turn();
        let duration = self
            .slow(TURN_TIME)
            .mul_f64(f64::from(steps.unsigned_abs()));
        self.effects.turn = Some(Turn::new(self.desktop, steps, View::of(area), duration));
        let [x, y, w, h] = area;
        let size = [f64::from(w), f64::from(h)];
        let image = wallpaper::image(&mut self.theme, &self.config.wallpaper, [w, h]);
        commands.push(Command::Create {
            id: CUBE_BACKGROUND.into(),
            kind: NodeKind::Rect,
            parent: WALLPAPERS.into(),
            props: PropsUpdate {
                size: Some(size),
                color: Some(BACKGROUND),
                transform: Some(Transform::translate(f64::from(x), f64::from(y), 0.0)),
                ..PropsUpdate::default()
            },
        });
        for desktop in 0..DESKTOPS {
            commands.push(Command::Create {
                id: face_id(desktop),
                kind: NodeKind::Image,
                parent: desk_id(desktop),
                props: PropsUpdate {
                    image: Some(image.clone()),
                    size: Some(size),
                    visible: Some(false),
                    ..PropsUpdate::default()
                },
            });
            commands.push(Command::Move {
                id: face_id(desktop),
                parent: None,
                index: Some(0),
            });
        }
        commands.extend(self.cube_frame(Instant::now()));
        commands
    }

    /// Ends the drags, wobbles and lamps before a turn: the windows fly flat.
    fn settle_for_turn(&mut self) -> Vec<Command> {
        self.drag = None;
        let mut commands = Vec::new();
        for surface in std::mem::take(&mut self.effects.wobbles).into_keys() {
            if let Some(i) = self.index(&surface) {
                commands.push(self.windows[i].unmesh());
            }
        }
        let lamps: Vec<String> = self.effects.lamps.keys().cloned().collect();
        for surface in lamps {
            commands.extend(self.end_lamp(&surface));
        }
        for w in &mut self.windows {
            w.held = true;
        }
        commands
    }

    /// One frame of the turn: the faces toward the viewer in their
    /// perspective, the farthest first, and their windows above them.
    pub(super) fn cube_frame(&mut self, now: Instant) -> Vec<Command> {
        let Some(turn) = &self.effects.turn else {
            return Vec::new();
        };
        let t = turn.time(now);
        let faces = turn.faces(t);
        let (view, push) = (turn.view, turn.push(t));
        let origin = self.bars.first().map_or([0, 0, 0, 0], Bar::area);
        let mut commands: Vec<Command> = (0..DESKTOPS)
            .map(|d| visible(&desk_id(d), faces.iter().any(|f| f.desktop == d)))
            .collect();
        for face in &faces {
            commands.push(Command::Move {
                id: desk_id(face.desktop),
                parent: None,
                index: None,
            });
            let at = Transform::translate(f64::from(origin[0]), f64::from(origin[1]), 0.0);
            let flat = cube::transform(&view, face.angle, push, 0.0).then(&at);
            commands.push(Command::Update {
                id: face_id(face.desktop),
                props: PropsUpdate {
                    transform: Some(flat),
                    visible: Some(true),
                    ..PropsUpdate::default()
                },
            });
            commands.extend(self.fly(face, t));
        }
        if t >= 1.0 {
            commands.extend(self.finish_turn());
        }
        commands
    }

    /// The windows of a desktop, lifted off its face toward the viewer.
    fn fly(&self, face: &Face, t: f64) -> Vec<Command> {
        let Some(turn) = &self.effects.turn else {
            return Vec::new();
        };
        let (view, push) = (turn.view, turn.push(t));
        let flying: Vec<_> = self
            .windows
            .iter()
            .filter(|w| w.desktop == face.desktop && w.shown && !w.minimized)
            .collect();
        let count = flying.len();
        flying
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let lift = Turn::lift(t, i, count);
                let transform = cube::transform(&view, face.angle, push, lift).then(&w.transform());
                Command::Update {
                    id: decoration::group_id(&w.surface),
                    props: PropsUpdate {
                        transform: Some(transform),
                        size: Some(w.area()),
                        ..PropsUpdate::default()
                    },
                }
            })
            .collect()
    }

    /// The end of a turn: the new desktop shows flat, its topmost window
    /// gets the focus, and the taskbar lists its windows.
    pub(super) fn finish_turn(&mut self) -> Vec<Command> {
        let Some(turn) = self.effects.turn.take() else {
            return Vec::new();
        };
        let mut commands = vec![Command::Remove {
            id: CUBE_BACKGROUND.into(),
        }];
        commands.extend((0..DESKTOPS).map(|d| Command::Remove { id: face_id(d) }));
        commands.extend(self.show_desktop(turn.target()));
        commands
    }

    /// Shows a desktop flat, without a turn.
    fn show_desktop(&mut self, target: usize) -> Vec<Command> {
        self.desktop = target;
        let mut commands: Vec<Command> = (0..DESKTOPS)
            .map(|d| visible(&desk_id(d), d == self.desktop))
            .collect();
        for w in &mut self.windows {
            w.held = false;
            commands.push(w.place());
        }
        let top = self.topmost_except("");
        commands.extend(self.focus(top.as_deref()));
        commands.extend(self.sync_tasks());
        commands.extend(self.pagers());
        commands
    }
}

fn visible(id: &str, visible: bool) -> Command {
    Command::Update {
        id: id.into(),
        props: PropsUpdate {
            visible: Some(visible),
            ..PropsUpdate::default()
        },
    }
}
