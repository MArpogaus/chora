//! Window decorations: the frame of Exposé around a window, with its title
//! and its buttons, as image nodes in the group of the window.
//!
//! The group sits at the top-left corner of the shadow of the frame, which
//! reaches [`SHADOW`] pixels beyond the frame. So a layer of the size of
//! the group, for the effects, holds the whole window. The layout follows
//! `exposerc`: 4 pixel borders, a 30 pixel title bar, 22 pixel buttons.

use std::path::Path;

use chora_scene::{Command, InputMode, NodeKind, PropsUpdate, Transform};

use crate::{
    text::{Font, Text},
    theme::{Picture, Rect, Sheet, Theme},
    window::GRIP,
};

/// The width of the left, right and bottom borders.
pub const BORDER: i32 = 4;
/// The height of the title bar.
pub const TITLE: i32 = 30;
/// The shadow around the frame in the frame elements.
pub const SHADOW: i32 = 10;
/// The size of a button.
const BUTTON: i32 = 22;
/// The space above the buttons.
const BUTTON_TOP: i32 = 5;
/// The space between two buttons.
const BUTTON_GAP: i32 = 2;
/// The space between the buttons and the right edge of the frame, and
/// between the left edge and the title.
const EDGE: i32 = 6;

/// A button of the title bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    /// Closes the window.
    Close,
    /// Maximizes or restores the window.
    Maximize,
    /// Minimizes the window.
    Minimize,
}

impl Button {
    /// All buttons, from the right.
    pub const ALL: [Self; 3] = [Self::Close, Self::Maximize, Self::Minimize];

    fn name(self) -> &'static str {
        match self {
            Self::Close => "close",
            Self::Maximize => "max",
            Self::Minimize => "min",
        }
    }

    fn sheet(self, maximized: bool) -> Sheet {
        match self {
            Self::Close => Sheet::Close,
            Self::Maximize if maximized => Sheet::Restore,
            Self::Maximize => Sheet::Maximize,
            Self::Minimize => Sheet::Minimize,
        }
    }

    /// The left edge of the button in a frame of a width.
    fn x(self, width: i32) -> i32 {
        let index = match self {
            Self::Close => 1,
            Self::Maximize => 2,
            Self::Minimize => 3,
        };
        width - EDGE - index * BUTTON - (index - 1) * BUTTON_GAP
    }
}

/// The id of the group of a window.
pub fn group_id(surface: &str) -> String {
    format!("xp-win-{surface}")
}

fn frame_id(surface: &str) -> String {
    format!("xp-frame-{surface}")
}

fn title_id(surface: &str) -> String {
    format!("xp-title-{surface}")
}

fn button_id(surface: &str, button: Button) -> String {
    format!("xp-{}-{surface}", button.name())
}

/// The invisible strips around a window that draws its own frame, which
/// take the presses on its edges: top, bottom, left and right.
fn grip_id(surface: &str, side: usize) -> String {
    format!("xp-grip{side}-{surface}")
}

/// A node of a decoration, by its id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Part {
    /// The frame of a surface.
    Frame(String),
    /// The title bar of a surface.
    Title(String),
    /// A button of a surface.
    Button(String, Button),
    /// A strip around a surface that draws its own frame.
    Grip(String),
}

impl Part {
    /// The part that a node id names.
    pub fn of(node: &str) -> Option<Self> {
        let rest = node.strip_prefix("xp-")?;
        let (kind, surface) = rest.split_once('-')?;
        let surface = surface.to_string();
        match kind {
            "frame" => Some(Self::Frame(surface)),
            "title" => Some(Self::Title(surface)),
            grip if grip.starts_with("grip") => Some(Self::Grip(surface)),
            name => Button::ALL
                .into_iter()
                .find(|b| b.name() == name)
                .map(|b| Self::Button(surface, b)),
        }
    }
}

/// How a decoration looks.
pub struct Look<'a> {
    /// The window title.
    pub title: &'a str,
    /// The icon of the application, left of the title.
    pub icon: Option<&'a Path>,
    /// The size of the client.
    pub size: [i32; 2],
    /// The window has the focus.
    pub active: bool,
    /// The window is maximized: no borders and no shadow.
    pub maximized: bool,
    /// The button that is down.
    pub pressed: Option<Button>,
    /// The button under the pointer.
    pub hovered: Option<Button>,
    /// The plugin draws the frame; a client that draws its own gets none.
    pub framed: bool,
}

impl Look<'_> {
    /// The width and height of the frame.
    pub fn outer(&self) -> [i32; 2] {
        if self.framed {
            outer(self.size, self.maximized)
        } else {
            self.size
        }
    }
}

/// The width and height of the frame around a client.
pub fn outer([w, h]: [i32; 2], maximized: bool) -> [i32; 2] {
    let border = if maximized { 0 } else { BORDER };
    [w + 2 * border, h + TITLE + border]
}

/// The client size in a frame of a size.
pub fn inner([w, h]: [i32; 2], maximized: bool) -> [i32; 2] {
    let border = if maximized { 0 } else { BORDER };
    [(w - 2 * border).max(1), (h - TITLE - border).max(1)]
}

/// The rectangle of a button in the group of a frame of a width.
pub fn button_rect(width: i32, button: Button) -> Rect {
    [button.x(width), BUTTON_TOP, BUTTON, BUTTON]
}

/// The position of the client in the group.
fn client_offset(maximized: bool) -> [f64; 2] {
    if maximized {
        [0.0, f64::from(TITLE)]
    } else {
        [f64::from(BORDER), f64::from(TITLE)]
    }
}

/// Creates the nodes of a decoration in a hidden group, bottom to top.
/// The surface follows them, so that it lies above them; for a window of
/// the plugin, `own`, a group of that id takes its place.
pub fn create(surface: &str, parent: &str, own: bool) -> Vec<Command> {
    let group = Command::Create {
        id: group_id(surface),
        kind: NodeKind::Group,
        parent: parent.into(),
        props: PropsUpdate {
            visible: Some(false),
            meta: Some([("xp".to_string(), "window".to_string())].into()),
            ..PropsUpdate::default()
        },
    };
    let ids = [frame_id(surface), title_id(surface)]
        .into_iter()
        .chain(Button::ALL.iter().rev().map(|b| button_id(surface, *b)));
    let mut commands = vec![group];
    commands.extend(ids.map(|id| Command::Create {
        id,
        kind: NodeKind::Image,
        parent: group_id(surface),
        props: PropsUpdate::default(),
    }));
    let surface_node = if own {
        Command::Create {
            id: surface.into(),
            kind: NodeKind::Group,
            parent: group_id(surface),
            props: PropsUpdate::default(),
        }
    } else {
        Command::Attach {
            surface: surface.into(),
            parent: group_id(surface),
        }
    };
    commands.push(surface_node);
    // Above the surface, so that they win over the shadow of a client.
    commands.extend((0..4).map(|side| Command::Create {
        id: grip_id(surface, side),
        kind: NodeKind::Rect,
        parent: group_id(surface),
        props: PropsUpdate {
            visible: Some(false),
            color: Some([0.0; 4]),
            input: Some(InputMode::Accept),
            ..PropsUpdate::default()
        },
    }));
    commands
}

/// The updates that give a decoration its look.
pub fn update(theme: &mut Theme, surface: &str, look: &Look<'_>) -> Vec<Command> {
    if !look.framed {
        return unframed(surface, look);
    }
    let mut commands = vec![
        frame(theme, surface, look),
        title(theme, surface, look),
        Command::Update {
            id: surface.into(),
            props: at(client_offset(look.maximized)),
        },
    ];
    for button in Button::ALL {
        commands.push(self::button(theme, surface, look, button));
    }
    commands.extend((0..4).map(|side| Command::Update {
        id: grip_id(surface, side),
        props: PropsUpdate {
            visible: Some(false),
            ..PropsUpdate::default()
        },
    }));
    commands
}

/// A window that draws its own frame: the parts of the frame hide, and the
/// client sits at the corner of the frame.
fn unframed(surface: &str, look: &Look<'_>) -> Vec<Command> {
    let hide = |id: String| Command::Update {
        id,
        props: PropsUpdate {
            visible: Some(false),
            ..PropsUpdate::default()
        },
    };
    let mut commands = vec![
        hide(frame_id(surface)),
        hide(title_id(surface)),
        Command::Update {
            id: surface.into(),
            props: at([0.0, 0.0]),
        },
    ];
    commands.extend(Button::ALL.map(|b| hide(button_id(surface, b))));
    // The strips just outside the client, the corners on the top and
    // the bottom one.
    let ([w, h], g) = (look.size, GRIP);
    let strips = [
        [-g, -g, w + 2 * g, g],
        [-g, h, w + 2 * g, g],
        [-g, 0, g, h],
        [w, 0, g, h],
    ];
    for (side, [x, y, sw, sh]) in strips.into_iter().enumerate() {
        commands.push(Command::Update {
            id: grip_id(surface, side),
            props: PropsUpdate {
                visible: Some(!look.maximized),
                size: Some([f64::from(sw), f64::from(sh)]),
                ..at([f64::from(x), f64::from(y)])
            },
        });
    }
    commands
}

fn frame(theme: &mut Theme, surface: &str, look: &Look<'_>) -> Command {
    let [w, h] = look.outer();
    let inactive = if look.active { "" } else { "-inactive" };
    let props = if look.maximized {
        // A title bar without borders, stretched over the width.
        let id = format!("decoration-maximized{inactive}-center");
        let [nw, nh] = theme.size(Sheet::Decoration, &id).unwrap_or([1, TITLE]);
        let picture = Picture::new(nw, nh).element(Sheet::Decoration, &id, [0, 0, nw, nh]);
        PropsUpdate {
            image: Some(theme.render(&picture)),
            size: Some([f64::from(w), f64::from(TITLE)]),
            slice: Some([0.0; 4]),
            ..at([0.0, 0.0])
        }
    } else {
        let frame = theme.frame(Sheet::Decoration, &format!("decoration{inactive}"));
        let shadow = f64::from(SHADOW);
        PropsUpdate {
            image: Some(frame.image),
            size: Some([f64::from(w) + 2.0 * shadow, f64::from(h) + 2.0 * shadow]),
            slice: Some(frame.slice),
            ..at([-shadow, -shadow])
        }
    };
    Command::Update {
        id: frame_id(surface),
        props: PropsUpdate {
            visible: Some(true),
            ..props
        },
    }
}

/// The title bar left of the buttons: the text, and the handle to drag
/// the window.
fn title(theme: &mut Theme, surface: &str, look: &Look<'_>) -> Command {
    let [w, _] = look.outer();
    let width = (Button::Minimize.x(w) - 2 * EDGE).max(1);
    let color = if look.active {
        [255, 255, 255, 255]
    } else {
        [216, 228, 248, 255]
    };
    let mut picture = Picture::new(width, TITLE);
    let mut left = 2;
    if let Some(icon) = look.icon {
        picture = picture.file(icon, [0, 7, 16, 16]);
        left = 21;
    }
    let mut text = Text::new(
        look.title,
        Font::Title,
        color,
        [left, 1, width - left, TITLE],
    );
    if look.active {
        text = text.shadow([10, 24, 131, 255]);
    }
    let picture = picture.text(text);
    Command::Update {
        id: title_id(surface),
        props: PropsUpdate {
            image: Some(theme.render(&picture)),
            visible: Some(true),
            ..at([f64::from(EDGE), 0.0])
        },
    }
}

fn button(theme: &mut Theme, surface: &str, look: &Look<'_>, button: Button) -> Command {
    let [w, _] = look.outer();
    let state = if look.pressed == Some(button) {
        "pressed"
    } else if look.hovered == Some(button) {
        "hover"
    } else {
        "active"
    };
    let state = match (state, look.active) {
        ("active", false) => "inactive".to_string(),
        (state, true) => state.to_string(),
        (state, false) => format!("{state}-inactive"),
    };
    let picture = Picture::new(BUTTON, BUTTON).frame(
        button.sheet(look.maximized),
        &state,
        [0, 0, BUTTON, BUTTON],
    );
    Command::Update {
        id: button_id(surface, button),
        props: PropsUpdate {
            image: Some(theme.render(&picture)),
            input: Some(InputMode::Accept),
            visible: Some(true),
            ..at([f64::from(button.x(w)), f64::from(BUTTON_TOP)])
        },
    }
}

/// Props that put a node at a position relative to the top-left corner of
/// the frame.
fn at([x, y]: [f64; 2]) -> PropsUpdate {
    let shadow = f64::from(SHADOW);
    PropsUpdate {
        transform: Some(Transform::translate(x + shadow, y + shadow, 0.0)),
        ..PropsUpdate::default()
    }
}

/// The size of the group of a frame of a size: the frame with its shadow.
pub fn area([w, h]: [i32; 2]) -> [f64; 2] {
    [f64::from(w + 2 * SHADOW), f64::from(h + 2 * SHADOW)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_ids_name_their_part() {
        assert_eq!(Part::of("xp-title-s3"), Some(Part::Title("s3".into())));
        assert_eq!(
            Part::of(&button_id("s12", Button::Minimize)),
            Some(Part::Button("s12".into(), Button::Minimize))
        );
        assert_eq!(Part::of("xp-grip2-s3"), Some(Part::Grip("s3".into())));
        assert_eq!(Part::of("xp-win-s3"), None);
        assert_eq!(Part::of("s3"), None);
    }

    #[test]
    fn the_frame_wraps_the_client() {
        assert_eq!(outer([640, 480], false), [648, 514]);
        assert_eq!(inner(outer([640, 480], false), false), [640, 480]);
        assert_eq!(outer([1280, 740], true), [1280, 770]);
    }

    #[test]
    fn buttons_line_up_from_the_right() {
        assert_eq!(Button::Close.x(200), 172);
        assert_eq!(Button::Maximize.x(200), 148);
        assert_eq!(Button::Minimize.x(200), 124);
    }
}
