//! XP's dialog windows and their Luna controls, drawn by the plugin: the
//! window frame of the theme with the title and the close button, and on
//! the face tabs, group boxes, check boxes, radio buttons, push buttons,
//! text and pictures. The colors and sizes follow XP's Date and Time and
//! Display Properties: Tahoma 8 pt, the face `#ECE9D8`, the white tab
//! page, group boxes in `#D0D0BF` and buttons of 73 by 21 pixels.
//!
//! A dialog shows a [`Page`]: one picture of everything that does not
//! react, and the controls, each an image node of its own, so that the
//! one under the pointer changes alone. Its owner builds the page from
//! its state and shows it again after each click.

use std::fmt::Write as _;

use chora_scene::{Command, InputMode, NodeKind, PropsUpdate, Transform};

use crate::{
    decoration::{self, BORDER, Button, SHADOW, TITLE},
    text::{Font, Text},
    theme::{Picture, Rect, Sheet, Theme},
};

/// The face of dialogs.
pub const FACE: &str = "#ECE9D8";
/// Black text.
pub const BLACK: [u8; 4] = [0, 0, 0, 255];
/// The text of a disabled control.
pub const GRAY: [u8; 4] = [161, 161, 146, 255];
/// The caption of a group box.
const CAPTION: [u8; 4] = [0, 70, 213, 255];
/// The size of OK, Cancel and Apply.
pub const BUTTON: [i32; 2] = [73, 21];
/// The height of a tab; the selected one is 2 pixels higher.
const TAB_HEIGHT: i32 = 18;
/// The box of a check box or a radio button.
const BOX: i32 = 13;

/// What a control is.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// A push button; the default one has a blue rim.
    Push {
        /// Enter would press it.
        default: bool,
    },
    /// A check box, checked or not.
    Check(bool),
    /// A radio button, chosen or not.
    Radio(bool),
    /// A tab, selected or not.
    Tab(bool),
    /// A picture of the owner that takes clicks, such as a calendar.
    Area(Picture),
}

/// A control of a page.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Control {
    /// The name that a click reports.
    pub name: String,
    /// What it is.
    pub kind: Kind,
    /// The label.
    pub label: String,
    /// The rectangle in the client area.
    pub rect: Rect,
    /// A disabled control is gray and takes no click.
    pub enabled: bool,
}

impl Control {
    fn new(name: &str, kind: Kind, label: &str, rect: Rect) -> Self {
        Self {
            name: name.into(),
            kind,
            label: label.into(),
            rect,
            enabled: true,
        }
    }

    /// A push button of XP's size at a place.
    pub fn push(name: &str, label: &str, [x, y]: [i32; 2]) -> Self {
        Self::new(
            name,
            Kind::Push { default: false },
            label,
            [x, y, BUTTON[0], BUTTON[1]],
        )
    }

    /// A check box with its label in a rectangle.
    pub fn check(name: &str, label: &str, rect: Rect, on: bool) -> Self {
        Self::new(name, Kind::Check(on), label, rect)
    }

    /// A radio button with its label in a rectangle.
    pub fn radio(name: &str, label: &str, rect: Rect, on: bool) -> Self {
        Self::new(name, Kind::Radio(on), label, rect)
    }

    /// A tab; [`tabs`] lays out a row of them.
    pub fn tab(name: &str, label: &str, rect: Rect, selected: bool) -> Self {
        Self::new(name, Kind::Tab(selected), label, rect)
    }

    /// A picture that takes clicks.
    pub fn area(name: &str, rect: Rect, picture: Picture) -> Self {
        Self::new(name, Kind::Area(picture), "", rect)
    }

    /// The default button.
    pub fn default(mut self) -> Self {
        if let Kind::Push { default } = &mut self.kind {
            *default = true;
        }
        self
    }

    /// Enabled or disabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Whether a point of the client area is on the control.
    fn contains(&self, [px, py]: [i32; 2]) -> bool {
        let [x, y, w, h] = self.rect;
        (x..x + w).contains(&px) && (y..y + h).contains(&py)
    }

    /// The control under the pointer, down or not.
    pub fn picture(&self, hovered: bool, down: bool) -> Picture {
        let [_, _, w, h] = self.rect;
        let hot = hovered && self.enabled;
        let color = if self.enabled { BLACK } else { GRAY };
        let picture = Picture::new(w, h);
        match &self.kind {
            Kind::Push { default } => {
                let look = match (self.enabled, hot && down, hot, *default) {
                    (false, ..) => Look::Disabled,
                    (true, true, ..) => Look::Down,
                    (true, false, true, _) => Look::Hot,
                    (true, false, false, true) => Look::Default,
                    (true, false, false, false) => Look::Normal,
                };
                picture
                    .svg(button_svg(w, h, look))
                    .text(Text::new(&self.label, Font::Ui, color, [0, 0, w, h]).centered())
            }
            Kind::Check(on) | Kind::Radio(on) => {
                let radio = matches!(self.kind, Kind::Radio(_));
                let look = match (self.enabled, hot && down, hot) {
                    (false, ..) => Look::Disabled,
                    (true, true, _) => Look::Down,
                    (true, false, true) => Look::Hot,
                    (true, false, false) => Look::Normal,
                };
                let y = (h - BOX) / 2;
                let svg = if radio {
                    radio_svg(y, *on, look)
                } else {
                    check_svg(y, *on, look)
                };
                picture.svg(svg).text(Text::new(
                    &self.label,
                    Font::Ui,
                    color,
                    [BOX + 5, 0, w - BOX - 5, h],
                ))
            }
            Kind::Tab(selected) => {
                // The label of the selected tab sits 1 pixel higher.
                let top = i32::from(*selected);
                picture.svg(tab_svg(w, h, *selected, hot)).text(
                    Text::new(&self.label, Font::Ui, BLACK, [0, top, w, TAB_HEIGHT]).centered(),
                )
            }
            Kind::Area(picture) => picture.clone(),
        }
    }
}

/// The look of a push button, check box or radio button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Look {
    Normal,
    /// The default button.
    Default,
    /// Under the pointer.
    Hot,
    Down,
    Disabled,
}

/// What a page shows: one picture of the client area, and the controls.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Page {
    /// The face, the tab page, group boxes and text.
    pub body: Option<Picture>,
    /// The controls, bottom first.
    pub controls: Vec<Control>,
}

/// A row of tabs from the left edge of a page at `x`, `y` the top of the
/// page: each as wide as its label and 10 pixels, 1 pixel apart, the
/// selected one 2 pixels larger on each side and on top.
pub fn tabs(theme: &Theme, [x, y]: [i32; 2], labels: &[&str], selected: usize) -> Vec<Control> {
    let mut left = x + 2;
    let mut out = Vec::new();
    for (i, label) in labels.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let w = theme.text_width(label, Font::Ui).ceil() as i32 + 10;
        let rect = if i == selected {
            [left - 2, y - TAB_HEIGHT - 2, w + 4, TAB_HEIGHT + 3]
        } else {
            [left, y - TAB_HEIGHT, w, TAB_HEIGHT]
        };
        out.push(Control::tab(
            &format!("tab-{i}"),
            label,
            rect,
            i == selected,
        ));
        left += w + 1;
    }
    // The selected tab lies above its neighbors.
    out.sort_by_key(|c| matches!(c.kind, Kind::Tab(true)));
    out
}

/// The face of a client area of a size.
pub fn face([w, h]: [i32; 2]) -> Picture {
    Picture::new(w, h).svg(format!(r#"<rect width="{w}" height="{h}" fill="{FACE}"/>"#))
}

/// The white page of the tabs: a gray border, a gradient to the bottom
/// and a shadow of 2 pixels at the right and the bottom. `rect` is the
/// border.
pub fn tab_page([x, y, w, h]: Rect) -> String {
    format!(
        r##"<defs><linearGradient id="page" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#FCFCFE"/><stop offset="1" stop-color="#F4F3EE"/></linearGradient></defs><rect x="{x}" y="{y}" width="{w}" height="{h}" fill="url(#page)"/><rect x="{a}" y="{b}" width="{c}" height="{d}" fill="none" stroke="#919B9C"/><path d="M{e} {f}v{h}M{x} {g}h{w}" stroke="#D0CEBF"/><path d="M{i} {j}v{h}M{k} {l}h{w}" stroke="#E3E0D0"/>"##,
        a = f64::from(x) + 0.5,
        b = f64::from(y) + 0.5,
        c = w - 1,
        d = h - 1,
        e = f64::from(x + w) + 0.5,
        f = y + 1,
        g = f64::from(y + h) + 0.5,
        i = f64::from(x + w) + 1.5,
        j = y + 2,
        k = x + 2,
        l = f64::from(y + h) + 1.5,
    )
}

/// A group box: a rounded line around a rectangle with the caption in
/// blue at its top left, which breaks the line.
#[allow(clippy::many_single_char_names)] // coordinates
pub fn group_box(theme: &Theme, picture: Picture, [x, y, w, h]: Rect, caption: &str) -> Picture {
    #[allow(clippy::cast_possible_truncation)]
    let width = theme.text_width(caption, Font::Ui).ceil() as i32;
    // The line runs through the middle of the caption.
    let top = f64::from(y + 6) + 0.5;
    let [l, r, b] = [
        f64::from(x) + 0.5,
        f64::from(x + w) - 0.5,
        f64::from(y + h) - 0.5,
    ];
    let gap_start = x + 6;
    let gap_end = gap_start + width + 4;
    let svg = format!(
        r##"<path d="M{gs} {top}H{l3}Q{l} {top} {l} {t3}V{b3}Q{l} {b} {l3} {b}H{r3}Q{r} {b} {r} {b3}V{t3}Q{r} {top} {r3} {top}H{ge}" fill="none" stroke="#D0D0BF"/>"##,
        gs = gap_start,
        ge = gap_end,
        l3 = l + 3.0,
        t3 = top + 3.0,
        b3 = b - 3.0,
        r3 = r - 3.0,
    );
    picture.svg(svg).text(Text::new(
        caption,
        Font::Ui,
        CAPTION,
        [gap_start + 2, y, width + 2, 13],
    ))
}

/// A line of black text, gray when disabled.
pub fn label(picture: Picture, text: &str, rect: Rect, enabled: bool) -> Picture {
    picture.text(Text::new(
        text,
        Font::Ui,
        if enabled { BLACK } else { GRAY },
        rect,
    ))
}

/// A field with XP's blue-gray border, white inside.
pub fn field([x, y, w, h]: Rect) -> String {
    format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#fff" stroke="#7F9DB9"/>"##,
        f64::from(x) + 0.5,
        f64::from(y) + 0.5,
        w - 1,
        h - 1
    )
}

/// The blue button of a drop-down list or of a spin box, in a
/// rectangle: a down arrow, or arrows up and down.
#[allow(clippy::many_single_char_names)] // coordinates
pub fn arrow_button([x, y, w, h]: Rect, spin: bool) -> String {
    let mut svg = format!(
        r##"<defs><linearGradient id="ab{x}{y}" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#D8E4FD"/><stop offset="1" stop-color="#B2C8FA"/></linearGradient></defs><rect x="{}" y="{}" width="{}" height="{}" rx="1.5" fill="url(#ab{x}{y})" stroke="#BACCF4"/>"##,
        f64::from(x) + 0.5,
        f64::from(y) + 0.5,
        w - 1,
        h - 1
    );
    let cx = f64::from(x) + f64::from(w) / 2.0;
    let chevron = |svg: &mut String, cy: f64, up: bool| {
        let d = if up { -2.0 } else { 2.0 };
        let _ = write!(
            svg,
            r##"<path d="M{} {}L{cx} {}L{} {}" fill="none" stroke="#4D6185" stroke-width="2"/>"##,
            cx - 3.5,
            cy - d,
            cy + d,
            cx + 3.5,
            cy - d,
        );
    };
    let mid = f64::from(y) + f64::from(h) / 2.0;
    if spin {
        chevron(&mut svg, mid - 3.5, true);
        chevron(&mut svg, mid + 3.5, false);
    } else {
        chevron(&mut svg, mid, false);
    }
    svg
}

/// A Luna push button: a dark blue rounded border and a white face that
/// grows gray to the bottom; a blue rim for the default button, an
/// orange one under the pointer, a darker face when down and pale colors
/// when disabled.
fn button_svg(w: i32, h: i32, look: Look) -> String {
    let (border, stops) = match look {
        Look::Disabled => ("#C9C7BA", "#F5F4EA:0,#F5F4EA:1"),
        Look::Down => ("#003C74", "#E5E4DD:0,#E3E2DA:0.8,#F1F0EB:1"),
        _ => (
            "#003C74",
            "#FFFFFF:0,#F6F6F3:0.35,#F0F0EA:0.8,#ECEBE6:0.88,#E2DFD6:0.94,#D6D0C5:1",
        ),
    };
    let mut svg = gradient("face", stops, false);
    let [fw, fh] = [w - 1, h - 1];
    let _ = write!(
        svg,
        r#"<rect x="0.5" y="0.5" width="{fw}" height="{fh}" rx="3" fill="url(#face)" stroke="{border}"/>"#
    );
    let rim = match look {
        Look::Default => Some(["#CEE7FF", "#BCD4F6", "#89ADE4", "#6982EE", "#A0BEEC"]),
        Look::Hot => Some(["#FFF0CF", "#FDD889", "#F8B330", "#E59700", "#FBC761"]),
        _ => None,
    };
    if let Some([top1, top2, bottom2, bottom1, side]) = rim {
        let [iw, ih] = [w - 4, h - 4];
        let _ = write!(
            svg,
            r#"<rect x="2" y="2" width="{iw}" height="{ih}" fill="none" stroke="{side}" stroke-width="2"/><rect x="2" y="1" width="{iw}" height="1" fill="{top1}"/><rect x="2" y="2" width="{iw}" height="1" fill="{top2}"/><rect x="2" y="{b2}" width="{iw}" height="1" fill="{bottom2}"/><rect x="2" y="{b1}" width="{iw}" height="1" fill="{bottom1}"/>"#,
            b2 = h - 3,
            b1 = h - 2,
        );
    }
    svg
}

/// A check box at the left, `y` from the top: XP's green check in a box
/// with a dark blue border and a face that grows white to the bottom
/// right.
fn check_svg(y: i32, on: bool, look: Look) -> String {
    let mut svg = box_face(y, look, false);
    if on {
        let color = if look == Look::Disabled {
            "#CAC8BB"
        } else {
            "#21A121"
        };
        // Seven columns of three pixels: the hook of XP's check.
        for (i, top) in [2, 3, 4, 3, 2, 1, 0].into_iter().enumerate() {
            let _ = write!(
                svg,
                r#"<rect x="{}" y="{}" width="1" height="3" fill="{color}"/>"#,
                3 + i,
                y + 3 + top
            );
        }
    }
    svg
}

/// A radio button at the left, `y` from the top, with a green dot when
/// chosen.
fn radio_svg(y: i32, on: bool, look: Look) -> String {
    let mut svg = box_face(y, look, true);
    if on {
        let (light, dark) = if look == Look::Disabled {
            ("#E0DFD4", "#CAC8BB")
        } else {
            ("#55D551", "#1A891A")
        };
        let _ = write!(
            svg,
            r#"<defs><radialGradient id="dot" cx="0.4" cy="0.35" r="0.7"><stop offset="0" stop-color="{light}"/><stop offset="1" stop-color="{dark}"/></radialGradient></defs><circle cx="6.5" cy="{}" r="2.6" fill="url(#dot)"/>"#,
            f64::from(y) + 6.5
        );
    }
    svg
}

/// The box of a check box or the circle of a radio button.
fn box_face(y: i32, look: Look, round: bool) -> String {
    let (border, stops) = match look {
        Look::Disabled => ("#CAC8BB", "#FFFFFF:0,#FFFFFF:1"),
        Look::Down => ("#1C5180", "#B0B0A7:0,#E3E1D4:1"),
        _ => ("#1C5180", "#DCDCD7:0,#FFFFFF:1"),
    };
    let mut svg = gradient("box", stops, true);
    let shape = |stroke: &str, fill: &str, inset: f64, width: f64| {
        let side = f64::from(BOX) - 1.0 - 2.0 * inset;
        let at = 0.5 + inset;
        if round {
            format!(
                r#"<circle cx="6.5" cy="{}" r="{}" fill="{fill}" stroke="{stroke}" stroke-width="{width}"/>"#,
                f64::from(y) + 6.5,
                side / 2.0
            )
        } else {
            format!(
                r#"<rect x="{at}" y="{}" width="{side}" height="{side}" fill="{fill}" stroke="{stroke}" stroke-width="{width}"/>"#,
                f64::from(y) + at
            )
        }
    };
    svg += &shape(border, "url(#box)", 0.0, 1.0);
    if look == Look::Hot {
        svg += &gradient("hot", "#FFF0CF:0,#F8B330:1", true);
        svg += &shape("url(#hot)", "none", 1.5, 2.0);
    }
    svg
}

/// A tab: rounded at the top, white growing gray to the bottom; the
/// selected one is white with XP's orange line on top, as is a tab under
/// the pointer.
#[allow(clippy::many_single_char_names)] // coordinates
fn tab_svg(w: i32, h: i32, selected: bool, hot: bool) -> String {
    let stops = if selected {
        "#FCFCFE:0,#FCFCFE:1"
    } else {
        "#FFFFFF:0,#F3F3EF:0.6,#ECEBE6:1"
    };
    let mut svg = gradient("tab", stops, false);
    // Without a bottom edge: the selected tab joins its page.
    let bottom = if selected { h } else { h - 1 };
    let [l, r] = [0.5, f64::from(w) - 0.5];
    let t = 0.5;
    let _ = write!(
        svg,
        r##"<path d="M{l} {bottom}V{t2}Q{l} {t} {l2} {t}H{r2}Q{r} {t} {r} {t2}V{bottom}" fill="url(#tab)" stroke="#919B9C"/>"##,
        t2 = t + 2.0,
        l2 = l + 2.0,
        r2 = r - 2.0,
    );
    if selected || hot {
        let inner = w - 2;
        let _ = write!(
            svg,
            r##"<rect x="1" y="0" width="{inner}" height="1" fill="#E68B2C"/><rect x="0" y="1" width="{w}" height="2" fill="#FFC73C"/>"##,
        );
    }
    svg
}

/// A linear gradient from `"#color:offset,..."`, down or to the bottom
/// right.
fn gradient(id: &str, stops: &str, diagonal: bool) -> String {
    let (x2, y2) = if diagonal { (1, 1) } else { (0, 1) };
    let mut svg = format!(r#"<defs><linearGradient id="{id}" x1="0" y1="0" x2="{x2}" y2="{y2}">"#);
    for stop in stops.split(',') {
        let (color, offset) = stop.split_once(':').unwrap_or((stop, "0"));
        let _ = write!(svg, r#"<stop offset="{offset}" stop-color="{color}"/>"#);
    }
    svg + "</linearGradient></defs>"
}

/// What a click in a dialog did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Hit {
    /// The close button of the title bar.
    Close,
    /// A control, by name, and the point in it.
    Control(String, [i32; 2]),
}

/// What is down.
#[derive(Clone, Debug, PartialEq)]
enum Down {
    Close,
    Control(String),
    /// The title bar: the pointer and the origin at the press.
    Title([f64; 2], [i32; 2]),
}

/// A dialog window in the scene, with the page it shows.
pub struct Dialog {
    /// The first part of the ids of its nodes.
    prefix: String,
    title: String,
    /// The client area.
    size: [i32; 2],
    /// The top-left corner of the frame in scene coordinates.
    origin: [i32; 2],
    page: Page,
    hovered: Option<String>,
    down: Option<Down>,
}

impl Dialog {
    /// A dialog with a client area of a size in the middle of a work area.
    /// `name` is unique among the dialogs.
    pub fn new(name: &str, title: &str, size: [i32; 2], [x, y, w, h]: Rect) -> Self {
        let [ow, oh] = outer(size);
        Self {
            prefix: format!("xp-dlg-{name}"),
            title: title.into(),
            size,
            origin: [x + (w - ow) / 2, (y + (h - oh) / 2).max(y)],
            page: Page::default(),
            hovered: None,
            down: None,
        }
    }

    /// Whether a node belongs to the dialog.
    pub fn owns(&self, node: &str) -> bool {
        node.strip_prefix(&self.prefix)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('-'))
    }

    /// Whether any dialog owns a node.
    pub fn any_owns(node: &str) -> bool {
        node.starts_with("xp-dlg-")
    }

    fn id(&self, part: &str) -> String {
        format!("{}-{part}", self.prefix)
    }

    /// The top-left corner of the client area in the scene.
    pub fn client(&self) -> [i32; 2] {
        [self.origin[0] + BORDER, self.origin[1] + TITLE]
    }

    /// The group of the dialog: raise it with a move to the end.
    pub fn group(&self) -> String {
        self.prefix.clone()
    }

    /// Creates the frame, the title and the close button, and shows a
    /// page.
    pub fn create(&mut self, theme: &mut Theme, parent: &str, page: Page) -> Vec<Command> {
        let [ow, oh] = outer(self.size);
        let frame = theme.frame(Sheet::Decoration, "decoration");
        let shadow = f64::from(SHADOW);
        let image = |id: String, props: PropsUpdate| Command::Create {
            id,
            kind: NodeKind::Image,
            parent: self.prefix.clone(),
            props,
        };
        let title = Picture::new(ow - 2 * EDGE - 30, TITLE).text(
            Text::new(
                &self.title,
                Font::Title,
                [255, 255, 255, 255],
                [2, 1, ow, TITLE],
            )
            .shadow([10, 24, 131, 255]),
        );
        let mut commands = vec![
            Command::Create {
                id: self.prefix.clone(),
                kind: NodeKind::Group,
                parent: parent.into(),
                props: at(self.origin),
            },
            image(
                self.id("frame"),
                PropsUpdate {
                    image: Some(frame.image),
                    size: Some([f64::from(ow) + 2.0 * shadow, f64::from(oh) + 2.0 * shadow]),
                    slice: Some(frame.slice),
                    transform: Some(Transform::translate(-shadow, -shadow, 0.0)),
                    ..PropsUpdate::default()
                },
            ),
            image(
                self.id("title"),
                PropsUpdate {
                    image: Some(theme.render(&title)),
                    input: Some(InputMode::Accept),
                    ..at([EDGE, 0])
                },
            ),
            image(self.id("close"), PropsUpdate::default()),
            Command::Create {
                id: self.id("content"),
                kind: NodeKind::Group,
                parent: self.prefix.clone(),
                props: at([BORDER, TITLE]),
            },
        ];
        commands.push(self.close_look(theme));
        commands.extend(self.content(theme, page));
        commands
    }

    /// Shows a page: only the parts that changed when the controls stay
    /// the same, else the whole content again.
    pub fn show(&mut self, theme: &mut Theme, page: Page) -> Vec<Command> {
        let same = page.controls.len() == self.page.controls.len()
            && page
                .controls
                .iter()
                .zip(&self.page.controls)
                .all(|(a, b)| a.name == b.name && a.rect == b.rect);
        if !same {
            let mut commands = vec![Command::Remove {
                id: self.id("content"),
            }];
            commands.push(Command::Create {
                id: self.id("content"),
                kind: NodeKind::Group,
                parent: self.prefix.clone(),
                props: at([BORDER, TITLE]),
            });
            commands.extend(self.content(theme, page));
            return commands;
        }
        let old = std::mem::replace(&mut self.page, page);
        let mut commands = Vec::new();
        if old.body != self.page.body
            && let Some(body) = &self.page.body
        {
            commands.push(update(self.id("body"), theme.render(body)));
        }
        for i in 0..self.page.controls.len() {
            if old.controls[i] != self.page.controls[i] {
                commands.push(self.control_look(theme, i));
            }
        }
        commands
    }

    /// The nodes of a page in the content group.
    fn content(&mut self, theme: &mut Theme, page: Page) -> Vec<Command> {
        self.page = page;
        let mut commands = Vec::new();
        if let Some(body) = &self.page.body {
            commands.push(Command::Create {
                id: self.id("body"),
                kind: NodeKind::Image,
                parent: self.id("content"),
                props: PropsUpdate {
                    image: Some(theme.render(body)),
                    ..PropsUpdate::default()
                },
            });
        }
        for (i, control) in self.page.controls.iter().enumerate() {
            commands.push(Command::Create {
                id: self.id(&format!("c{i}")),
                kind: NodeKind::Image,
                parent: self.id("content"),
                props: PropsUpdate {
                    input: Some(InputMode::Accept),
                    ..at([control.rect[0], control.rect[1]])
                },
            });
        }
        for i in 0..self.page.controls.len() {
            commands.push(self.control_look(theme, i));
        }
        commands
    }

    fn index(&self, node: &str) -> Option<usize> {
        let i: usize = node
            .strip_prefix(&self.prefix)?
            .strip_prefix("-c")?
            .parse()
            .ok()?;
        (i < self.page.controls.len()).then_some(i)
    }

    fn control_look(&self, theme: &mut Theme, i: usize) -> Command {
        let control = &self.page.controls[i];
        let hovered = self.hovered.as_deref() == Some(control.name.as_str());
        let down = self.down == Some(Down::Control(control.name.clone()));
        let picture = control.picture(hovered, down);
        update(self.id(&format!("c{i}")), theme.render(&picture))
    }

    fn close_look(&self, theme: &mut Theme) -> Command {
        let state = match (&self.down, self.hovered.as_deref()) {
            (Some(Down::Close), Some("close")) => "pressed",
            (_, Some("close")) => "hover",
            _ => "active",
        };
        let [x, y, w, h] = decoration::button_rect(outer(self.size)[0], Button::Close);
        let picture = Picture::new(w, h).frame(Sheet::Close, state, [0, 0, w, h]);
        Command::Update {
            id: self.id("close"),
            props: PropsUpdate {
                image: Some(theme.render(&picture)),
                input: Some(InputMode::Accept),
                ..at([x, y])
            },
        }
    }

    /// The name of the control or button that a node shows.
    fn name_of(&self, node: &str) -> Option<String> {
        if node == self.id("close") {
            return Some("close".into());
        }
        let control = &self.page.controls[self.index(node)?];
        control.enabled.then(|| control.name.clone())
    }

    /// The pointer came onto a node of the dialog or left it.
    pub fn hover(&mut self, theme: &mut Theme, node: &str, on: bool) -> Vec<Command> {
        let Some(name) = self.name_of(node) else {
            return Vec::new();
        };
        if on {
            self.hovered = Some(name.clone());
        } else if self.hovered.as_ref() == Some(&name) {
            self.hovered = None;
        }
        self.relook(theme, &name)
    }

    fn relook(&self, theme: &mut Theme, name: &str) -> Vec<Command> {
        if name == "close" {
            return vec![self.close_look(theme)];
        }
        self.page
            .controls
            .iter()
            .position(|c| c.name == name)
            .map(|i| self.control_look(theme, i))
            .into_iter()
            .collect()
    }

    /// A press on a node of the dialog, at a point of the scene: the title
    /// bar starts a drag, a button goes down.
    pub fn press(&mut self, theme: &mut Theme, node: &str, position: [f64; 2]) -> Vec<Command> {
        if node == self.id("title") {
            self.down = Some(Down::Title(position, self.origin));
            return Vec::new();
        }
        let Some(name) = self.name_of(node) else {
            return Vec::new();
        };
        self.down = Some(if name == "close" {
            Down::Close
        } else {
            Down::Control(name.clone())
        });
        self.relook(theme, &name)
    }

    /// Whether the title bar is down.
    pub fn dragging(&self) -> bool {
        matches!(self.down, Some(Down::Title(..)))
    }

    /// The pointer moved while the title bar is down: the dialog follows.
    pub fn drag(&mut self, [px, py]: [f64; 2], top: i32) -> Vec<Command> {
        let Some(Down::Title([sx, sy], [ox, oy])) = self.down else {
            return Vec::new();
        };
        #[allow(clippy::cast_possible_truncation)]
        let origin = [
            ox + (px - sx).round() as i32,
            (oy + (py - sy).round() as i32).max(top),
        ];
        if origin == self.origin {
            return Vec::new();
        }
        self.origin = origin;
        vec![Command::Update {
            id: self.prefix.clone(),
            props: at(origin),
        }]
    }

    /// The release of the button: what was clicked, if the pointer is
    /// still on what went down.
    pub fn release(
        &mut self,
        theme: &mut Theme,
        [px, py]: [f64; 2],
    ) -> (Vec<Command>, Option<Hit>) {
        #[allow(clippy::cast_possible_truncation)]
        let [x, y] = [
            px.floor() as i32 - self.origin[0],
            py.floor() as i32 - self.origin[1],
        ];
        match self.down.take() {
            Some(Down::Close) => {
                let [bx, by, bw, bh] = decoration::button_rect(outer(self.size)[0], Button::Close);
                let inside = (bx..bx + bw).contains(&x) && (by..by + bh).contains(&y);
                (vec![self.close_look(theme)], inside.then_some(Hit::Close))
            }
            Some(Down::Control(name)) => {
                let commands = self.relook(theme, &name);
                let point = [x - BORDER, y - TITLE];
                let hit = self
                    .page
                    .controls
                    .iter()
                    .find(|c| c.name == name && c.contains(point))
                    .map(|c| {
                        Hit::Control(name.clone(), [point[0] - c.rect[0], point[1] - c.rect[1]])
                    });
                (commands, hit)
            }
            Some(Down::Title(..)) | None => (Vec::new(), None),
        }
    }

    /// Removes the dialog.
    pub fn remove(&self) -> Command {
        Command::Remove {
            id: self.prefix.clone(),
        }
    }
}

/// The space between the left edge of the frame and the title.
const EDGE: i32 = 6;

/// The frame around a client area of a size.
pub fn outer([w, h]: [i32; 2]) -> [i32; 2] {
    decoration::outer([w, h], false)
}

fn update(id: String, image: String) -> Command {
    Command::Update {
        id,
        props: PropsUpdate {
            image: Some(image),
            ..PropsUpdate::default()
        },
    }
}

fn at([x, y]: [i32; 2]) -> PropsUpdate {
    PropsUpdate {
        transform: Some(Transform::translate(f64::from(x), f64::from(y), 0.0)),
        ..PropsUpdate::default()
    }
}

#[cfg(test)]
pub mod tests {
    use std::path::{Path, PathBuf};

    use super::*;

    /// The theme of the source tree, with a cache of its own for a test.
    pub fn theme(test: &str) -> (Theme, PathBuf) {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flavors/xp/theme");
        let cache = std::env::temp_dir().join(format!("chora-xp-{test}-{}", std::process::id()));
        (Theme::load(&dir, &cache).expect("the theme loads"), cache)
    }

    /// The whole dialog as one picture, as the scene shows it, for a look
    /// at the result: `CHORA_XP_PREVIEW=dir cargo test` writes it there.
    pub fn preview(theme: &mut Theme, dialog: &Dialog, name: &str) {
        let Some(dir) = std::env::var_os("CHORA_XP_PREVIEW") else {
            return;
        };
        let [ow, oh] = outer(dialog.size);
        let mut picture = Picture::new(ow, oh)
            .svg(format!(
                r##"<rect width="{ow}" height="{oh}" fill="#3A6EA5"/>"##
            ))
            .frame(
                Sheet::Decoration,
                "decoration",
                [-SHADOW, -SHADOW, ow + 2 * SHADOW, oh + 2 * SHADOW],
            );
        picture = picture.text(
            Text::new(
                &dialog.title,
                Font::Title,
                [255, 255, 255, 255],
                [EDGE + 2, 1, ow, TITLE],
            )
            .shadow([10, 24, 131, 255]),
        );
        let close = decoration::button_rect(ow, Button::Close);
        picture = picture.frame(Sheet::Close, "active", close);
        let mut layers = Vec::new();
        if let Some(body) = &dialog.page.body {
            layers.push((theme.render(body), [0, 0], body.size));
        }
        for c in &dialog.page.controls {
            let hovered = dialog.hovered.as_deref() == Some(c.name.as_str());
            let p = c.picture(hovered, false);
            layers.push((theme.render(&p), [c.rect[0], c.rect[1]], p.size));
        }
        for (file, [x, y], [w, h]) in layers {
            picture = picture.file(Path::new(&file), [x + BORDER, y + TITLE, w, h]);
        }
        let out = theme.render(&picture);
        let target = PathBuf::from(dir).join(format!("{name}.png"));
        std::fs::copy(out, &target).expect("the preview is written");
    }

    #[test]
    fn tabs_line_up_with_the_selected_one_on_top() {
        let (theme, cache) = theme("tabs");
        let tabs = tabs(&theme, [6, 27], &["Taskbar", "Start Menu"], 0);
        // The selected tab comes last, so that it lies on top.
        assert_eq!(tabs[1].kind, Kind::Tab(true));
        let [x, y, w, h] = tabs[1].rect;
        assert_eq!([x, y, h], [6, 7, 21]);
        let [nx, ny, ..] = tabs[0].rect;
        assert_eq!([nx, ny], [x + 2 + w - 4 + 1, 9]);
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn a_click_reports_the_control_and_the_point_in_it() {
        let (mut theme, cache) = theme("click");
        let mut dialog = Dialog::new("t", "Test", [398, 316], [0, 0, 1280, 800]);
        let page = Page {
            body: Some(face([398, 316])),
            controls: vec![
                Control::push("ok", "OK", [156, 287]).default(),
                Control::push("apply", "Apply", [318, 287]).enabled(false),
            ],
        };
        dialog.create(&mut theme, "root", page);
        let [ox, oy] = dialog.origin;
        let click = |d: &mut Dialog, theme: &mut Theme, node: &str, [x, y]: [i32; 2]| {
            let at = [f64::from(ox + BORDER + x), f64::from(oy + TITLE + y)];
            d.press(theme, node, at);
            d.release(theme, at).1
        };
        assert_eq!(
            click(&mut dialog, &mut theme, "xp-dlg-t-c0", [160, 290]),
            Some(Hit::Control("ok".into(), [4, 3]))
        );
        // A disabled button takes no click, and a release elsewhere
        // clicks nothing.
        assert_eq!(
            click(&mut dialog, &mut theme, "xp-dlg-t-c1", [320, 290]),
            None
        );
        assert_eq!(
            click(&mut dialog, &mut theme, "xp-dlg-t-c0", [10, 10]),
            None
        );
        assert!(dialog.owns("xp-dlg-t-c0") && !dialog.owns("xp-dlg-tt-c0"));
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn the_title_bar_drags_the_dialog() {
        let (mut theme, cache) = theme("drag");
        let mut dialog = Dialog::new("t", "Test", [200, 100], [0, 0, 800, 600]);
        let origin = dialog.origin;
        dialog.press(&mut theme, "xp-dlg-t-title", [300.0, 260.0]);
        assert!(dialog.dragging());
        dialog.drag([350.0, 100.0], 0);
        assert_eq!(dialog.origin, [origin[0] + 50, origin[1] - 160]);
        // Not above the top of the work area.
        dialog.drag([350.0, -900.0], 0);
        assert_eq!(dialog.origin[1], 0);
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn the_controls_look_like_xp() {
        let (mut theme, cache) = theme("kit");
        let mut dialog = Dialog::new("kit", "Display Properties", [398, 316], [0, 0, 800, 600]);
        let size = [398, 316];
        let mut body = face(size).svg(tab_page([6, 27, 384, 251]));
        body = group_box(&theme, body, [16, 38, 183, 196], "Date");
        body = label(
            body,
            "Current time zone:  Central European Time",
            [16, 250, 360, 13],
            true,
        );
        let mut controls = tabs(
            &theme,
            [6, 27],
            &["Date & Time", "Time Zone", "Internet Time"],
            0,
        );
        controls.extend([
            Control::check("a", "Lock the taskbar", [24, 60, 160, 17], true),
            Control::check("b", "Auto-hide the taskbar", [24, 80, 160, 17], false),
            Control::check("c", "Disabled", [24, 100, 160, 17], true).enabled(false),
            Control::radio("d", "Start menu", [24, 120, 160, 17], true),
            Control::radio("e", "Classic Start menu", [24, 140, 160, 17], false),
            Control::push("ok", "OK", [156, 287]).default(),
            Control::push("cancel", "Cancel", [237, 287]),
            Control::push("apply", "Apply", [318, 287]).enabled(false),
            Control::push("hot", "Hover", [237, 200]),
        ]);
        dialog.create(
            &mut theme,
            "root",
            Page {
                body: Some(body),
                controls,
            },
        );
        dialog.hovered = Some("hot".into());
        preview(&mut theme, &dialog, "kit");
        let _ = std::fs::remove_dir_all(cache);
    }
}
