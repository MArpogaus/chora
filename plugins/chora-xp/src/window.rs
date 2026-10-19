//! One window: a surface in a group with its decoration, and the commands
//! that place, style, minimize and restore it.

use std::path::PathBuf;

use chora_scene::{Command, Mesh, PropsUpdate, ToplevelState, Transform};

use crate::{
    decoration::{self, BORDER, Button, Look, SHADOW},
    theme::{Rect, Theme},
};

/// How far beyond its frame a window still takes a press on an edge, the
/// only grip of a window that draws its own frame.
pub const GRIP: i32 = 6;
/// How far a corner reaches along its two edges, as on XP.
const CORNER: i32 = 16;
/// The smallest frame: XP keeps the buttons and a bit of the title.
const MIN_FRAME: [i32; 2] = [112, decoration::TITLE + BORDER + 1];

/// The edges that a drag of the border of a frame moves.
#[allow(clippy::struct_excessive_bools)] // four edges
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Edges {
    /// The left edge.
    pub left: bool,
    /// The right edge.
    pub right: bool,
    /// The top edge.
    pub top: bool,
    /// The bottom edge.
    pub bottom: bool,
}

/// The edges under a point of the scene: within `inside` pixels inside a
/// frame or [`GRIP`] pixels outside it. Near a corner both of its edges
/// move. `None` when the point is not on the border.
pub fn edges_at([x, y, w, h]: Rect, inside: i32, [px, py]: [f64; 2]) -> Option<Edges> {
    #[allow(clippy::cast_possible_truncation)]
    let [px, py] = [px.floor() as i32, py.floor() as i32];
    let near = |p: i32, from: i32, to: i32| (from - GRIP..to + GRIP).contains(&p);
    if !near(px, x, x + w) || !near(py, y, y + h) {
        return None;
    }
    let mut found = Edges {
        left: px < x + inside,
        right: px >= x + w - inside,
        top: py < y + inside,
        bottom: py >= y + h - inside,
    };
    if found.left || found.right {
        found.top |= py < y + CORNER;
        found.bottom |= py >= y + h - CORNER;
    }
    if found.top || found.bottom {
        found.left |= px < x + CORNER;
        found.right |= px >= x + w - CORNER;
    }
    (found != Edges::default()).then_some(found)
}

/// A frame after a drag of its edges by a distance: the opposite edges
/// stay, and the frame keeps a smallest size.
pub fn resized([x, y, w, h]: Rect, edges: Edges, [dx, dy]: [i32; 2], [mw, mh]: [i32; 2]) -> Rect {
    let (x, w) = edge([x, w], dx, [edges.left, edges.right], mw);
    let (y, h) = edge([y, h], dy, [edges.top, edges.bottom], mh);
    [x, y, w, h]
}

/// One axis of [`resized`]: the low edge moves with the far edge fixed,
/// or the high edge moves.
fn edge([pos, size]: [i32; 2], d: i32, [low, high]: [bool; 2], min: i32) -> (i32, i32) {
    if low {
        let new = (size - d).max(min);
        (pos + size - new, new)
    } else if high {
        (pos, (size + d).max(min))
    } else {
        (pos, size)
    }
}

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

    /// Whether a point in scene coordinates is on the icon of the title
    /// bar.
    pub fn on_icon(&self, [px, py]: [f64; 2]) -> bool {
        let [ix, iy, iw, ih] = decoration::ICON;
        let (x, y) = (f64::from(self.pos[0] + ix), f64::from(self.pos[1] + iy));
        self.framed
            && self.icon.is_some()
            && (x..x + f64::from(iw)).contains(&px)
            && (y..y + f64::from(ih)).contains(&py)
    }

    /// Whether a point in scene coordinates is on a button.
    pub fn on_button(&self, button: Button, [px, py]: [f64; 2]) -> bool {
        let [bx, by, bw, bh] = decoration::button_rect(self.outer()[0], button);
        let (x, y) = (f64::from(self.pos[0] + bx), f64::from(self.pos[1] + by));
        (x..x + f64::from(bw)).contains(&px) && (y..y + f64::from(bh)).contains(&py)
    }
}

impl Window {
    /// The edges of the border under a point, for a resize; never for a
    /// maximized window. A window that draws its own frame has only the
    /// grip outside it.
    pub fn edges(&self, point: [f64; 2]) -> Option<Edges> {
        if self.maximized() || !self.shown || self.minimized {
            return None;
        }
        let inside = if self.framed { BORDER } else { 0 };
        edges_at(self.rect(), inside, point)
    }

    /// The smallest frame, for a resize.
    pub fn min_frame(&self) -> [i32; 2] {
        if !self.framed {
            return [1, 1];
        }
        let [w, h] = if self.own {
            decoration::outer(crate::explorer::MIN_SIZE, false)
        } else {
            MIN_FRAME
        };
        [w.max(MIN_FRAME[0]), h.max(MIN_FRAME[1])]
    }

    /// Asks for a new frame during a resize: a window of the plugin takes
    /// it at once; a client gets a configure and its frame follows its
    /// commit.
    pub fn resize_to(&mut self, [x, y, w, h]: Rect, active: bool) -> Option<Command> {
        let size = if self.framed {
            decoration::inner([w, h], false)
        } else {
            [w, h]
        };
        if self.own {
            self.pos = [x, y];
            self.size = Some(size);
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
}

/// A frame moved into an area, and shrunk to it where it does not fit.
pub fn inside([x, y, w, h]: Rect, [ax, ay, aw, ah]: Rect) -> Rect {
    let [w, h] = [w.min(aw), h.min(ah)];
    [x.clamp(ax, ax + aw - w), y.clamp(ay, ay + ah - h), w, h]
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
    fn a_frame_moves_into_the_area_and_shrinks_to_it() {
        let area = [0, 0, 853, 503];
        assert_eq!(inside([504, 244, 744, 567], area), [109, 0, 744, 503]);
        assert_eq!(inside([10, 10, 100, 100], area), [10, 10, 100, 100]);
        assert_eq!(inside([-50, 600, 900, 50], area), [0, 453, 853, 50]);
    }

    /// The edges as letters: `l`, `r`, `t` and `b`.
    fn named(edges: Option<Edges>) -> String {
        let Some(e) = edges else {
            return "-".into();
        };
        [(e.left, 'l'), (e.right, 'r'), (e.top, 't'), (e.bottom, 'b')]
            .into_iter()
            .filter_map(|(on, c)| on.then_some(c))
            .collect()
    }

    #[test]
    fn the_border_and_the_grip_outside_take_the_edges() {
        let frame = [100, 100, 400, 300];
        // The 4 pixel border inside, and 6 pixels of the shadow outside;
        // the corners reach 16 pixels along both edges.
        let cases = [
            (100.0, 250.0, "l"),
            (94.0, 250.0, "l"),
            (93.9, 250.0, "-"),
            (104.0, 250.0, "-"),
            (499.0, 250.0, "r"),
            (505.5, 250.0, "r"),
            (506.0, 250.0, "-"),
            (300.0, 103.0, "t"),
            (300.0, 104.0, "-"),
            (300.0, 397.0, "b"),
            (101.0, 101.0, "lt"),
            (115.0, 101.0, "lt"),
            (116.0, 101.0, "t"),
            (101.0, 115.0, "lt"),
            (498.0, 102.0, "rt"),
            (95.0, 398.0, "lb"),
            (503.0, 403.0, "rb"),
        ];
        for (x, y, want) in cases {
            assert_eq!(named(edges_at(frame, BORDER, [x, y])), want, "{x} {y}");
        }
        // A window that draws its own frame has only the grip outside.
        assert_eq!(named(edges_at(frame, 0, [100.0, 250.0])), "-");
        assert_eq!(named(edges_at(frame, 0, [97.0, 250.0])), "l");
    }

    #[test]
    fn a_resize_keeps_the_opposite_edges_and_a_smallest_size() {
        let frame = [100, 100, 400, 300];
        let min = [112, 35];
        let left_top = Edges {
            left: true,
            top: true,
            ..Edges::default()
        };
        // Left and top move; right (500) and bottom (400) stay.
        assert_eq!(
            resized(frame, left_top, [-20, 30], min),
            [80, 130, 420, 270]
        );
        assert_eq!(
            resized(frame, left_top, [1000, 1000], min),
            [388, 365, 112, 35]
        );
        let right = Edges {
            right: true,
            ..Edges::default()
        };
        assert_eq!(resized(frame, right, [50, 99], min), [100, 100, 450, 300]);
        assert_eq!(resized(frame, right, [-999, 0], min), [100, 100, 112, 300]);
    }

    #[test]
    fn a_maximized_window_does_not_resize() {
        let mut w = Window::new("s1", None, None);
        w.size = Some([400, 300]);
        w.shown = true;
        w.pos = [10, 10];
        assert!(w.edges([12.0, 100.0]).is_some());
        w.maximize([0, 0, 1280, 770]);
        assert_eq!(w.edges([1.0, 100.0]), None);
        assert_eq!(w.min_frame(), [112, 35]);
    }

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
