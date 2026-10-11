//! One window: a surface in a group with its decoration, and the commands
//! that place, style, minimize and restore it.

use std::path::PathBuf;

use chora_scene::{Command, Easing, PropsUpdate, ToplevelState, Transform};

use crate::{
    decoration::{self, Button, Look},
    theme::{Rect, Theme},
};

/// How long minimizing and restoring take, in milliseconds.
const MINIMIZE_TIME: f64 = 250.0;

/// A window that the plugin manages.
pub struct Window {
    /// The surface node.
    pub surface: String,
    /// The title, or the application id while there is none.
    pub title: String,
    /// The application id.
    pub app_id: Option<String>,
    /// The icon of the application.
    pub icon: Option<PathBuf>,
    /// The top-left corner of the frame, in scene coordinates.
    pub pos: [i32; 2],
    /// The size of the client, once the client told it.
    pub size: Option<[i32; 2]>,
    /// The frame before maximizing: x, y, width and height of the client.
    pub restore: Option<Rect>,
    /// The window is minimized.
    pub minimized: bool,
    /// The window shows: it has a size and a place.
    pub shown: bool,
    /// The button of the title bar that is down.
    pub pressed: Option<Button>,
}

impl Window {
    /// A new window, hidden until it has a size.
    pub fn new(surface: &str, app_id: Option<String>, title: Option<&str>) -> Self {
        Self {
            surface: surface.into(),
            title: title_of(title, app_id.as_deref()),
            app_id,
            icon: None,
            pos: [0, 0],
            size: None,
            restore: None,
            minimized: false,
            shown: false,
            pressed: None,
        }
    }

    /// Whether the window is maximized.
    pub fn maximized(&self) -> bool {
        self.restore.is_some()
    }

    /// The width and height of the frame.
    pub fn outer(&self) -> [i32; 2] {
        decoration::outer(self.size.unwrap_or([1, 1]), self.maximized())
    }

    /// The frame in scene coordinates.
    pub fn rect(&self) -> Rect {
        let [w, h] = self.outer();
        [self.pos[0], self.pos[1], w, h]
    }

    /// The look of the decoration.
    pub fn look(&self, active: bool) -> Look<'_> {
        Look {
            title: &self.title,
            icon: self.icon.as_deref(),
            size: self.size.unwrap_or([1, 1]),
            active,
            maximized: self.maximized(),
            pressed: self.pressed,
        }
    }

    /// The configure command: the focus and the maximized state, and the
    /// size of a maximized window. Other windows choose their size.
    pub fn configure(&self, active: bool) -> Command {
        let mut states = Vec::new();
        if active {
            states.push(ToplevelState::Activated);
        }
        let size = if self.maximized() {
            states.push(ToplevelState::Maximized);
            self.size
        } else {
            None
        };
        Command::Configure {
            surface: self.surface.clone(),
            size,
            states,
        }
    }

    /// The decoration in its look and the group at its place. Nothing for
    /// a window that does not show yet.
    pub fn style(&self, theme: &mut Theme, active: bool) -> Vec<Command> {
        if !self.shown {
            return Vec::new();
        }
        let mut commands = decoration::update(theme, &self.surface, &self.look(active));
        if !self.minimized {
            commands.push(self.place());
        }
        commands
    }

    /// Moves the group to the position of the window.
    pub fn place(&self) -> Command {
        Command::Update {
            id: decoration::group_id(&self.surface),
            props: PropsUpdate {
                transform: Some(self.transform()),
                visible: Some(self.shown && !self.minimized),
                ..PropsUpdate::default()
            },
        }
    }

    fn transform(&self) -> Transform {
        Transform::translate(f64::from(self.pos[0]), f64::from(self.pos[1]), 0.0)
    }

    /// Shrinks the window into a rectangle, then hides it.
    pub fn minimize(&mut self, target: [f64; 4]) -> Command {
        self.minimized = true;
        let [x, y, w, h] = target;
        let [ow, oh] = self.outer().map(f64::from);
        let shrink = Transform::translate(x, y, 0.0).then(&Transform::scale(w / ow, h / oh, 1.0));
        Command::Animate {
            id: decoration::group_id(&self.surface),
            props: PropsUpdate {
                transform: Some(shrink),
                visible: Some(false),
                ..PropsUpdate::default()
            },
            duration: MINIMIZE_TIME,
            easing: Easing::EaseIn,
        }
    }

    /// Shows a minimized window again and grows it to its place.
    pub fn unminimize(&mut self) -> Vec<Command> {
        self.minimized = false;
        let id = decoration::group_id(&self.surface);
        vec![
            Command::Update {
                id: id.clone(),
                props: PropsUpdate {
                    visible: Some(self.shown),
                    ..PropsUpdate::default()
                },
            },
            Command::Animate {
                id,
                props: PropsUpdate {
                    transform: Some(self.transform()),
                    ..PropsUpdate::default()
                },
                duration: MINIMIZE_TIME,
                easing: Easing::EaseOut,
            },
        ]
    }

    /// Fills an area, keeping the old frame to restore it.
    pub fn maximize(&mut self, area: Rect) {
        if let (None, Some([w, h])) = (self.restore, self.size) {
            self.restore = Some([self.pos[0], self.pos[1], w, h]);
        }
        let [x, y, w, h] = area;
        self.pos = [x, y];
        self.size = Some(decoration::inner([w, h], true));
    }

    /// Ends the maximized state; returns the configure with the old size.
    pub fn unmaximize(&mut self, active: bool) -> Option<Command> {
        let [x, y, w, h] = self.restore.take()?;
        self.pos = [x, y];
        self.size = Some([w, h]);
        let states = if active {
            vec![ToplevelState::Activated]
        } else {
            Vec::new()
        };
        Some(Command::Configure {
            surface: self.surface.clone(),
            size: Some([w, h]),
            states,
        })
    }

    /// Whether a point in scene coordinates is on a button.
    pub fn on_button(&self, button: Button, [px, py]: [f64; 2]) -> bool {
        let [bx, by, bw, bh] = decoration::button_rect(self.outer()[0], button);
        let (x, y) = (f64::from(self.pos[0] + bx), f64::from(self.pos[1] + by));
        (x..x + f64::from(bw)).contains(&px) && (y..y + f64::from(bh)).contains(&py)
    }
}

/// The title of a window: its title, else its application id.
pub fn title_of(title: Option<&str>, app_id: Option<&str>) -> String {
    title
        .filter(|t| !t.trim().is_empty())
        .or(app_id)
        .unwrap_or("Window")
        .to_string()
}

/// The place of a new window: cascaded from the top-left corner of an
/// area, 30 pixels per window, back at the corner when it would leave the
/// area.
pub fn cascade(step: &mut i32, [w, h]: [i32; 2], [ax, ay, aw, ah]: Rect) -> [i32; 2] {
    let at = |n: i32| [ax + 24 + 30 * n, ay + 24 + 30 * n];
    let mut pos = at(*step);
    if pos[0] + w > ax + aw || pos[1] + h > ay + ah {
        *step = 0;
        pos = at(0);
    }
    *step += 1;
    // A window larger than the area starts at its corner.
    [
        pos[0].min(ax + aw - w).max(ax),
        pos[1].min(ay + ah - h).max(ay),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_cascade_and_wrap() {
        let mut step = 0;
        let area = [0, 0, 1280, 770];
        assert_eq!(cascade(&mut step, [600, 400], area), [24, 24]);
        assert_eq!(cascade(&mut step, [600, 400], area), [54, 54]);
        step = 20;
        assert_eq!(cascade(&mut step, [600, 400], area), [24, 24]);
        assert_eq!(step, 1);
    }

    #[test]
    fn a_huge_window_starts_at_the_corner() {
        let mut step = 3;
        assert_eq!(
            cascade(&mut step, [2000, 1000], [10, 20, 1280, 770]),
            [10, 20]
        );
    }

    #[test]
    fn maximizing_keeps_the_old_frame() {
        let mut w = Window::new("s1", Some("foot".into()), None);
        w.size = Some([600, 400]);
        w.pos = [50, 60];
        w.maximize([0, 0, 1280, 770]);
        assert_eq!((w.pos, w.size), ([0, 0], Some([1280, 740])));
        assert!(w.unmaximize(true).is_some());
        assert_eq!(
            (w.pos, w.size, w.title.as_str()),
            ([50, 60], Some([600, 400]), "foot")
        );
    }
}
