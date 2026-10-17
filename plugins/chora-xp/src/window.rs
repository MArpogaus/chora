//! One window: a surface in a group with its decoration, and the commands
//! that place, style, minimize and restore it.

use std::path::PathBuf;

use chora_scene::{Command, Mesh, PropsUpdate, ToplevelState, Transform};

use crate::{
    decoration::{self, Button, Look, SHADOW},
    theme::{Rect, Theme},
};

/// A window that the plugin manages.
#[allow(clippy::struct_excessive_bools)] // independent states of a window
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
    /// The desktop of the window, a face of the cube.
    pub desktop: usize,
    /// An effect moves the group: the magic lamp or the cube.
    pub held: bool,
    /// The plugin draws the frame; false for a client that draws its own.
    pub framed: bool,
    /// The window shows: it has a size and a place.
    pub shown: bool,
    /// The button of the title bar that is down.
    pub pressed: Option<Button>,
    /// The button of the title bar under the pointer.
    pub hovered: Option<Button>,
    /// The plugin draws the client, too, as for an Explorer window: no
    /// client gets a configure or the focus.
    pub own: bool,
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
            desktop: 0,
            held: false,
            framed: true,
            shown: false,
            pressed: None,
            hovered: None,
            own: false,
        }
    }

    /// Whether the window is maximized.
    pub fn maximized(&self) -> bool {
        self.restore.is_some()
    }

    /// The width and height of the frame.
    pub fn outer(&self) -> [i32; 2] {
        let size = self.size.unwrap_or([1, 1]);
        if self.framed {
            decoration::outer(size, self.maximized())
        } else {
            size
        }
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
            hovered: self.hovered,
            framed: self.framed,
        }
    }

    /// The configure command: the focus and the maximized state, and the
    /// size of a maximized window. Other windows choose their size. None
    /// for a window of the plugin.
    pub fn configure(&self, active: bool) -> Option<Command> {
        if self.own {
            return None;
        }
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
        Some(Command::Configure {
            surface: self.surface.clone(),
            size,
            states,
        })
    }

    /// The decoration in its look and the group at its place. Nothing for
    /// a window that does not show yet.
    pub fn style(&self, theme: &mut Theme, active: bool) -> Vec<Command> {
        if !self.shown {
            return Vec::new();
        }
        let mut commands = decoration::update(theme, &self.surface, &self.look(active));
        if !self.minimized && !self.held {
            commands.push(self.place());
        }
        commands
    }

    /// Moves the group flat to the position of the window.
    pub fn place(&self) -> Command {
        Command::Update {
            id: decoration::group_id(&self.surface),
            props: PropsUpdate {
                transform: Some(self.transform()),
                visible: Some(self.shown && !self.minimized),
                size: Some(self.area()),
                ..PropsUpdate::default()
            },
        }
    }

    /// Removes the mesh of the group, at the end of an effect.
    pub fn unmesh(&self) -> Command {
        Command::Update {
            id: decoration::group_id(&self.surface),
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

    /// The top-left corner of the group: the corner of the shadow.
    pub fn origin(&self) -> [f64; 2] {
        [
            f64::from(self.pos[0] - SHADOW),
            f64::from(self.pos[1] - SHADOW),
        ]
    }

    /// The size of the group, the frame with its shadow.
    pub fn area(&self) -> [f64; 2] {
        decoration::area(self.outer())
    }

    /// The group in scene coordinates: x, y, width and height.
    pub fn bounds(&self) -> [f64; 4] {
        let ([x, y], [w, h]) = (self.origin(), self.area());
        [x, y, w, h]
    }

    /// The flat transform of the group.
    pub fn transform(&self) -> Transform {
        let [x, y] = self.origin();
        Transform::translate(x, y, 0.0)
    }

    /// Fills an area, keeping the old frame to restore it.
    pub fn maximize(&mut self, area: Rect) {
        if self.restore.is_none() {
            // Before its first commit a window has no size yet; it restores
            // to the size that it picks (0 in a configure).
            let [w, h] = self.size.unwrap_or([0, 0]);
            self.restore = Some([self.pos[0], self.pos[1], w, h]);
        }
        let [x, y, w, h] = area;
        self.pos = [x, y];
        self.size = Some(if self.framed {
            decoration::inner([w, h], true)
        } else {
            [w, h]
        });
    }

    /// Ends the maximized state; returns the configure with the old size.
    pub fn unmaximize(&mut self, active: bool) -> Option<Command> {
        let [x, y, w, h] = self.restore.take()?;
        self.pos = [x, y];
        let size = (w > 0 && h > 0).then_some([w, h]);
        self.size = size;
        if self.own {
            return None;
        }
        let states = if active {
            vec![ToplevelState::Activated]
        } else {
            Vec::new()
        };
        Some(Command::Configure {
            surface: self.surface.clone(),
            size,
            states,
        })
    }

    /// Puts the frame into a rectangle, unmaximized: the configure with
    /// the new client size.
    pub fn fit(&mut self, [x, y, w, h]: Rect, active: bool) -> Option<Command> {
        self.restore = None;
        self.pos = [x, y];
        let size = if self.framed {
            decoration::inner([w, h], false)
        } else {
            [w, h]
        };
        self.size = Some(size);
        if self.own {
            return None;
        }
        let states = if active {
            vec![ToplevelState::Activated]
        } else {
            Vec::new()
        };
        Some(Command::Configure {
            surface: self.surface.clone(),
            size: Some(size),
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

/// The frames of `count` windows tiled over an area: in rows one above
/// the other, or in columns side by side. The last one takes the pixels
/// that the division leaves.
// ponytail: strips only; XP tiles four and more windows in a grid.
#[allow(clippy::many_single_char_names)] // coordinates
pub fn tile(count: usize, [x, y, w, h]: Rect, columns: bool) -> Vec<Rect> {
    let n = i32::try_from(count.max(1)).unwrap_or(i32::MAX);
    (0..n)
        .map(|i| {
            let (along, start) = if columns { (w, x) } else { (h, y) };
            let step = along / n;
            let from = start + i * step;
            let size = if i == n - 1 {
                start + along - from
            } else {
                step
            };
            if columns {
                [from, y, size, h]
            } else {
                [x, from, w, size]
            }
        })
        .take(count)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiles_fill_the_area() {
        let area = [0, 0, 1280, 770];
        assert_eq!(
            tile(3, area, false),
            [[0, 0, 1280, 256], [0, 256, 1280, 256], [0, 512, 1280, 258]]
        );
        assert_eq!(tile(2, area, true), [[0, 0, 640, 770], [640, 0, 640, 770]]);
        assert_eq!(tile(0, area, true), Vec::<Rect>::new());
    }

    #[test]
    fn a_window_maximized_before_its_first_commit_stays_maximized() {
        let mut w = Window::new("s1", None, None);
        w.maximize([0, 0, 1280, 770]);
        assert!(w.maximized());
        // It restores to a size of its own choice.
        let Some(Command::Configure { size, .. }) = w.unmaximize(false) else {
            panic!("a configure");
        };
        assert_eq!(size, None);
        assert!(!w.maximized());
    }

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
