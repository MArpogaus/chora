//! XP's Windows Explorer, drawn by the plugin as the client of a window
//! of its own: the menu bar, the Standard Buttons, the address bar, the
//! blue task pane with its boxes, and the view. The sizes and colors are
//! measured on XP's My Computer at 785 by 559 pixels.
//!
//! The window shows a [`Page`] of the dialog kit: one picture of the
//! chrome, and controls for the menus, the buttons, the links and the
//! items. A click goes to [`Explorer::release`], which says what the
//! window manager does: nothing more, a menu, a program, the Control
//! Panel.

use std::{fmt::Write as _, path::PathBuf};

use chora_scene::Command;

use crate::{
    context::{Act, Entry},
    dialog::{BLACK, Content, Control, GRAY, Hit, Page},
    folder::{History, Place},
    icons::{self, Icons},
    text::{Font, Text},
    theme::{Picture, Rect, Theme},
};

/// The client size of a new window: XP's My Computer of the reference.
pub const SIZE: [i32; 2] = [777, 525];
/// The smallest client size that keeps the toolbar and the pane whole.
pub const MIN_SIZE: [i32; 2] = [420, 260];

/// The menu bar from the top.
const MENU_H: i32 = 22;
/// The Standard Buttons.
const TOOLBAR_Y: i32 = 24;
const TOOLBAR_H: i32 = 38;
/// The address bar.
const ADDRESS_Y: i32 = 64;
const ADDRESS_H: i32 = 22;
/// The top of the task pane and of the view.
pub const VIEW_Y: i32 = 86;
/// The width of the task pane.
const PANE_W: i32 = 210;
/// The boxes of the task pane.
const BOX_X: i32 = 13;
const BOX_W: i32 = 185;
const BOX_HEAD: i32 = 25;
const BOX_GAP: i32 = 15;
const LINK_ROW: i32 = 20;
const DETAIL_ROW: i32 = 13;

/// The blue of the titles of the boxes and of the links.
const LINK: [u8; 4] = [33, 93, 198, 255];
/// A link under the pointer.
const LINK_HOT: [u8; 4] = [66, 142, 255, 255];
const WHITE: [u8; 4] = [255, 255, 255, 255];
/// The gray of the word "Address".
const ADDRESS_GRAY: [u8; 4] = [113, 111, 100, 255];
/// The background of the menu bar and of the toolbar, from left to
/// right.
const REBAR: (&str, &str) = ("#F3F4F7", "#EEEBDA");

/// The menus of the menu bar and the left edges of their titles.
const MENUS: [(&str, i32); 6] = [
    ("File", 10),
    ("Edit", 42),
    ("View", 76),
    ("Favorites", 114),
    ("Tools", 175),
    ("Help", 216),
];

/// What a row of an Explorer menu, a link or a button does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmd {
    Back,
    Forward,
    Up,
    Refresh,
    Close,
    /// My Documents.
    Documents,
    /// My Computer.
    Computer,
    /// The Recycle Bin.
    Trash,
}

/// What the window manager does after a click.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing, or only the new look that came with the release.
    None,
    /// The page changed: the title and the task button may too.
    Redraw,
    /// A menu below a point of the client area.
    Menu(Vec<Entry>, [i32; 2]),
    /// Opens the Control Panel of the start menu.
    ControlPanel,
    /// Closes the window.
    Close,
}

/// Where a link of the task pane goes.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Target {
    Place(Place),
    ControlPanel,
    /// A gray link: XP has it, chora not.
    None,
}

/// A link of a box.
struct Link {
    title: String,
    icon: &'static str,
    target: Target,
}

/// What a box of the task pane holds.
enum Body {
    Links(Vec<Link>),
    /// Lines of the Details box, the bold ones first.
    Details(Vec<(String, bool)>),
}

/// A box of the task pane.
struct TaskBox {
    title: &'static str,
    body: Body,
}

/// An Explorer window.
pub struct Explorer {
    /// The node id that stands in for the surface of the window.
    pub id: String,
    /// Back, Forward and the current place.
    pub history: History,
    content: Content,
    /// The client size of the page that shows.
    size: [i32; 2],
    /// The boxes of the task pane that are collapsed, by title.
    collapsed: Vec<&'static str>,
    /// Where each link of the page goes, by its name.
    links: Vec<(String, Target)>,
}

impl Explorer {
    /// A window `id` at a place, with a client size.
    pub fn new(id: &str, place: Place, size: [i32; 2]) -> Self {
        Self {
            id: id.into(),
            history: History::new(place),
            content: Content::new(&format!("{id}-view"), id, [0, 0]),
            size,
            collapsed: Vec::new(),
            links: Vec::new(),
        }
    }

    /// Whether a node belongs to the page of the window.
    pub fn owns(&self, node: &str) -> bool {
        self.content.owns(node)
    }

    /// The title of the window.
    pub fn title(&self) -> String {
        self.history.current.title()
    }

    /// The icon of the place, for the title bar and the task button.
    pub fn icon(&self, icons: &mut Icons) -> Option<PathBuf> {
        icons.first_sized(place_icons(&self.history.current), 16)
    }

    /// The client size of the page.
    pub fn size(&self) -> [i32; 2] {
        self.size
    }

    /// The nodes of the page.
    pub fn create(&mut self, theme: &mut Theme, icons: &mut Icons) -> Vec<Command> {
        let page = self.page(theme, icons);
        self.content.create(theme, page)
    }

    /// Shows the page again, for a new state or size.
    pub fn redraw(&mut self, theme: &mut Theme, icons: &mut Icons) -> Vec<Command> {
        let page = self.page(theme, icons);
        self.content.show(theme, page)
    }

    /// Takes a new client size.
    pub fn resize(&mut self, size: [i32; 2]) {
        self.size = size;
    }

    /// The pointer came onto a node of the page or left it.
    pub fn hover(&mut self, theme: &mut Theme, node: &str, on: bool) -> Vec<Command> {
        self.content.hover(theme, node, on)
    }

    /// A press on a node of the page.
    pub fn press(&mut self, theme: &mut Theme, node: &str) -> Vec<Command> {
        self.content.press(theme, node).unwrap_or_default()
    }

    /// The release of the button at a point of the client area.
    pub fn release(&mut self, theme: &mut Theme, point: [i32; 2]) -> (Vec<Command>, Outcome) {
        let (commands, hit) = self.content.release(theme, point);
        let outcome = match hit {
            Some(Hit::Control(name, _)) => self.click(&name),
            _ => Outcome::None,
        };
        (commands, outcome)
    }

    /// What a click on a control does.
    fn click(&mut self, name: &str) -> Outcome {
        if let Some(menu) = name.strip_prefix("menu-") {
            let Some(&(title, x)) = MENUS.iter().find(|(t, _)| *t == menu) else {
                return Outcome::None;
            };
            return Outcome::Menu(self.menu(title), [x - 6, MENU_H]);
        }
        if let Some(title) = name.strip_prefix("box-") {
            if let Some(i) = self.collapsed.iter().position(|t| *t == title) {
                self.collapsed.remove(i);
            } else if let Some(title) = BOX_TITLES.iter().find(|t| **t == title) {
                self.collapsed.push(title);
            }
            return Outcome::Redraw;
        }
        if let Some((_, target)) = self.links.iter().find(|(n, _)| n == name) {
            return match target.clone() {
                Target::Place(place) => self.go(place),
                Target::ControlPanel => Outcome::ControlPanel,
                Target::None => Outcome::None,
            };
        }
        match name {
            "back" => self.command(Cmd::Back),
            "forward" => self.command(Cmd::Forward),
            "up" => self.command(Cmd::Up),
            "go" => self.command(Cmd::Refresh),
            "views" => Outcome::Menu(views_menu(), [320, TOOLBAR_Y + TOOLBAR_H - 1]),
            "places" => {
                let x = self.size[0] - 57 - 160;
                Outcome::Menu(places_menu(), [x.max(49), ADDRESS_Y + ADDRESS_H])
            }
            _ => Outcome::None,
        }
    }

    /// Does what a row of a menu, a button or a link says.
    pub fn command(&mut self, cmd: Cmd) -> Outcome {
        let moved = match cmd {
            Cmd::Back => self.history.back(),
            Cmd::Forward => self.history.forward(),
            Cmd::Up => self.history.up(),
            Cmd::Refresh => true,
            Cmd::Close => return Outcome::Close,
            Cmd::Documents => return self.go(Place::Folder(documents())),
            Cmd::Computer => return self.go(Place::Computer),
            Cmd::Trash => return self.go(Place::Trash),
        };
        if moved {
            Outcome::Redraw
        } else {
            Outcome::None
        }
    }

    /// Goes to a place in the same window, as XP does.
    pub fn go(&mut self, place: Place) -> Outcome {
        self.history.go(place);
        Outcome::Redraw
    }

    /// The rows of a menu of the menu bar.
    fn menu(&self, title: &str) -> Vec<Entry> {
        match title {
            "File" => vec![
                Entry::sub("New", Vec::new()).enabled(false),
                Entry::separator(),
                Entry::gray("Create Shortcut"),
                Entry::gray("Delete"),
                Entry::gray("Rename"),
                Entry::gray("Properties"),
                Entry::separator(),
                Entry::item("Close", Act::Explore(Cmd::Close)),
            ],
            "Edit" => vec![
                Entry::gray("Undo"),
                Entry::separator(),
                Entry::gray("Cut"),
                Entry::gray("Copy"),
                Entry::gray("Paste"),
                Entry::separator(),
                Entry::gray("Select All"),
                Entry::gray("Invert Selection"),
            ],
            "View" => view_menu(&self.history),
            "Favorites" => vec![
                Entry::gray("Add to Favorites..."),
                Entry::gray("Organize Favorites..."),
            ],
            "Tools" => vec![
                Entry::gray("Map Network Drive..."),
                Entry::gray("Disconnect Network Drive..."),
                Entry::gray("Synchronize..."),
                Entry::separator(),
                Entry::gray("Folder Options..."),
            ],
            _ => vec![
                Entry::gray("Help and Support Center"),
                Entry::separator(),
                Entry::gray("About Windows"),
            ],
        }
    }

    // The page

    /// The page of the current state at the current size.
    fn page(&mut self, theme: &Theme, icons: &mut Icons) -> Page {
        let [w, h] = self.size;
        let mut page = Page {
            body: None,
            controls: MENUS
                .iter()
                .map(|(title, x)| menu_title(theme, title, *x))
                .collect(),
        };
        let mut body = Picture::new(w, h).svg(chrome([w, h]));
        // XP's flag at the right of the menu bar.
        if let Some(flag) = icons::data_file("winxp-kde/start_flag.png") {
            body = body
                .svg(format!(
                    r##"<rect x="{}" y="0" width="38" height="{MENU_H}" fill="#FFFFFF"/>"##,
                    w - 38
                ))
                .file(&flag, [w - 33, 1, 26, 20]);
        }
        page.controls.extend(self.toolbar(icons));
        page.body = Some(self.address_bar(body, icons));
        page.controls.extend(address_buttons(w, icons));
        self.task_pane(&mut page, theme, icons);
        page
    }

    /// The Standard Buttons: Back and Forward as the history allows, Up
    /// below the top, Search and Folders gray.
    fn toolbar(&self, icons: &mut Icons) -> Vec<Control> {
        let history = &self.history;
        let tools = [
            Tool::new("back", "go-previous", [4, 72], history.can_back())
                .label("Back")
                .arrow(),
            Tool::new("forward", "go-next", [80, 48], history.can_forward()).arrow(),
            Tool::new(
                "up",
                "go-parent-folder",
                [132, 28],
                history.current.up().is_some(),
            ),
            Tool::new("search", "system-search", [167, 69], false).label("Search"),
            Tool::new("folders", "folder", [240, 70], false).label("Folders"),
            Tool::new("views", "view-list-icons", [320, 40], true).arrow(),
        ];
        tools
            .iter()
            .map(|t| t.control(icons.find_sized(t.icon, 24).as_deref()))
            .collect()
    }

    /// The address bar: the place with its icon in the field.
    fn address_bar(&self, body: Picture, icons: &mut Icons) -> Picture {
        let w = self.size[0];
        let mut body = body.text(Text::new(
            "Address",
            Font::Ui,
            ADDRESS_GRAY,
            [6, ADDRESS_Y, 42, ADDRESS_H],
        ));
        if let Some(icon) = self.icon(icons) {
            body = body.file(&icon, [55, ADDRESS_Y + 3, 16, 16]);
        }
        body.text(Text::new(
            &self.history.current.address(),
            Font::Ui,
            BLACK,
            [72, ADDRESS_Y, w - 57 - 72 - 20, ADDRESS_H],
        ))
    }

    /// The boxes of the task pane, from the top, as far as they fit.
    fn task_pane(&mut self, page: &mut Page, theme: &Theme, icons: &mut Icons) {
        self.links.clear();
        let mut y = VIEW_Y + 12;
        for task in self.boxes() {
            if y >= self.size[1] {
                break;
            }
            let collapsed = self.collapsed.contains(&task.title);
            page.controls
                .push(box_header(task.title, [BOX_X, y], collapsed));
            y += BOX_HEAD;
            if collapsed {
                y += BOX_GAP;
                continue;
            }
            let height = match &task.body {
                Body::Links(links) => 17 + LINK_ROW * count(links.len()),
                Body::Details(lines) => 21 + DETAIL_ROW * count(lines.len()),
            };
            let body = page.body.take().unwrap_or_else(|| Picture::new(1, 1));
            let mut body = body.svg(box_body([BOX_X, y, BOX_W, height]));
            match &task.body {
                Body::Links(links) => {
                    page.controls
                        .extend(self.links(theme, icons, (task.title, links), y));
                }
                Body::Details(lines) => body = details(body, lines, y),
            }
            page.body = Some(body);
            y += height + BOX_GAP;
        }
    }

    /// The links of a box whose body starts at a height, as far as they
    /// fit.
    fn links(
        &mut self,
        theme: &Theme,
        icons: &mut Icons,
        (title, links): (&str, &[Link]),
        y: i32,
    ) -> Vec<Control> {
        let mut controls = Vec::new();
        for (i, link) in links.iter().enumerate() {
            let rect = [BOX_X + 11, y + 7 + LINK_ROW * count(i), BOX_W - 22, 20];
            if rect[1] + rect[3] > self.size[1] {
                break;
            }
            let name = format!("link-{title}-{i}");
            let icon = icons.find_sized(link.icon, 16);
            controls.push(link_control(theme, &name, rect, link, icon.as_deref()));
            self.links.push((name, link.target.clone()));
        }
        controls
    }

    /// The boxes of the task pane for the current place.
    fn boxes(&self) -> Vec<TaskBox> {
        let (tasks, places) = match &self.history.current {
            Place::Folder(path) => folder_boxes(path),
            Place::Computer => computer_boxes(),
            Place::Trash => trash_boxes(),
        };
        let mut boxes: Vec<TaskBox> = tasks.into_iter().collect();
        boxes.push(TaskBox {
            title: "Other Places",
            body: Body::Links(places),
        });
        boxes.push(TaskBox {
            title: "Details",
            body: Body::Details(self.details()),
        });
        boxes
    }

    /// The lines of the Details box: the place, as nothing is selected.
    fn details(&self) -> Vec<(String, bool)> {
        let kind = match &self.history.current {
            Place::Folder(_) => "File Folder",
            Place::Computer | Place::Trash => "System Folder",
        };
        vec![(self.title(), true), (kind.into(), false)]
    }
}

impl Link {
    /// A gray link: XP has it, chora not.
    fn gray(title: &str, icon: &'static str) -> Self {
        Self {
            title: title.into(),
            icon,
            target: Target::None,
        }
    }

    /// A link to a place.
    fn to(title: &str, icon: &'static str, place: Place) -> Self {
        Self {
            title: title.into(),
            icon,
            target: Target::Place(place),
        }
    }

    fn documents() -> Self {
        Self::to(
            "My Documents",
            "folder-documents",
            Place::Folder(documents()),
        )
    }

    fn computer() -> Self {
        Self::to("My Computer", "computer", Place::Computer)
    }

    fn network() -> Self {
        Self::gray("My Network Places", "network-workgroup")
    }
}

/// The lines of the Details box whose body starts at a height: the
/// bold ones first.
fn details(mut body: Picture, lines: &[(String, bool)], y: i32) -> Picture {
    for (i, (line, bold)) in lines.iter().enumerate() {
        let font = if *bold { Font::UiBold } else { Font::Ui };
        let rect = [BOX_X + 12, y + 11 + DETAIL_ROW * count(i), BOX_W - 24, 13];
        body = body.text(Text::new(line, font, BLACK, rect));
    }
    body
}

/// The tasks and places of a folder: the tasks are gray, as the window
/// changes no file; the places start with the parent folder.
fn folder_boxes(path: &std::path::Path) -> (Option<TaskBox>, Vec<Link>) {
    let mut places = Vec::new();
    if let Some(parent) = path.parent() {
        let parent = Place::Folder(parent.into());
        places.push(Link::to(&parent.title(), "folder", parent));
    }
    places.extend([Link::documents(), Link::computer(), Link::network()]);
    let tasks = TaskBox {
        title: "File and Folder Tasks",
        body: Body::Links(vec![
            Link::gray("Make a new folder", "folder-new"),
            Link::gray("Publish this folder to the Web", "network-server"),
            Link::gray("Share this folder", "folder-remote"),
        ]),
    };
    (Some(tasks), places)
}

/// The tasks and places of My Computer.
fn computer_boxes() -> (Option<TaskBox>, Vec<Link>) {
    let tasks = TaskBox {
        title: "System Tasks",
        body: Body::Links(vec![
            Link::gray("View system information", "dialog-information"),
            Link::gray("Add or remove programs", "system-software-install"),
            Link {
                title: "Change a setting".into(),
                icon: "preferences-desktop",
                target: Target::ControlPanel,
            },
        ]),
    };
    let places = vec![
        Link::network(),
        Link::documents(),
        Link::gray("Shared Documents", "folder-publicshare"),
        Link {
            title: "Control Panel".into(),
            icon: "preferences-system",
            target: Target::ControlPanel,
        },
    ];
    (Some(tasks), places)
}

/// The places of the Recycle Bin.
fn trash_boxes() -> (Option<TaskBox>, Vec<Link>) {
    let places = vec![
        Link::to("Desktop", "user-desktop", Place::Folder(desktop())),
        Link::documents(),
        Link::computer(),
        Link::network(),
    ];
    (None, places)
}

/// A button of the Standard Buttons.
struct Tool {
    name: &'static str,
    icon: &'static str,
    label: &'static str,
    /// The left edge and the width.
    span: [i32; 2],
    /// The arrow of a drop-down.
    arrow: bool,
    enabled: bool,
}

impl Tool {
    fn new(name: &'static str, icon: &'static str, span: [i32; 2], enabled: bool) -> Self {
        Self {
            name,
            icon,
            label: "",
            span,
            arrow: false,
            enabled,
        }
    }

    fn label(mut self, label: &'static str) -> Self {
        self.label = label;
        self
    }

    fn arrow(mut self) -> Self {
        self.arrow = true;
        self
    }

    /// The button: the icon, the label right of it and the arrow; under
    /// the pointer a light frame.
    fn control(&self, icon: Option<&std::path::Path>) -> Control {
        let [x, w] = self.span;
        let h = TOOLBAR_H - 2;
        let draw = |hot: bool| {
            let mut picture = Picture::new(w, h);
            if hot {
                picture = picture.svg(hot_frame(w, h));
            }
            let icon_rect = [4, (h - 24) / 2, 24, 24];
            if let Some(icon) = icon {
                picture = if self.enabled {
                    picture.file(icon, icon_rect)
                } else {
                    picture.gray(icon, icon_rect)
                };
            }
            let color = if self.enabled { BLACK } else { GRAY };
            if !self.label.is_empty() {
                picture = picture.text(Text::new(self.label, Font::Ui, color, [33, 0, w - 33, h]));
            }
            if self.arrow {
                let fill = if self.enabled { "#000000" } else { "#A1A192" };
                picture = picture.svg(format!(
                    r#"<path d="M{} {}h5l-2.5 3z" fill="{fill}"/>"#,
                    w - 10,
                    h / 2 - 1
                ));
            }
            picture
        };
        Control::hover(self.name, [x, TOOLBAR_Y + 2, w, h], draw(false), draw(true))
            .enabled(self.enabled)
    }
}

/// The list button of the address field and Go.
fn address_buttons(w: i32, icons: &mut Icons) -> [Control; 2] {
    [
        arrow_control("places", [w - 74, ADDRESS_Y + 2, 15, 18]),
        go_button(
            icons.first_sized(&["go-jump", "go-next"], 16).as_deref(),
            [w - 54, ADDRESS_Y, 50, ADDRESS_H],
        ),
    ]
}

/// The rows of the View menu: only Tiles, Go To and Refresh work.
fn view_menu(history: &History) -> Vec<Entry> {
    let cmd = |title: &str, cmd: Cmd| Entry::item(title, Act::Explore(cmd));
    vec![
        Entry::sub("Toolbars", Vec::new()).enabled(false),
        Entry::gray("Status Bar"),
        Entry::sub("Explorer Bar", Vec::new()).enabled(false),
        Entry::separator(),
        Entry::gray("Thumbnails"),
        Entry::item("Tiles", Act::Explore(Cmd::Refresh)).checked(true),
        Entry::gray("Icons"),
        Entry::gray("List"),
        Entry::gray("Details"),
        Entry::separator(),
        Entry::sub(
            "Go To",
            vec![
                cmd("Back", Cmd::Back).enabled(history.can_back()),
                cmd("Forward", Cmd::Forward).enabled(history.can_forward()),
                cmd("Up One Level", Cmd::Up).enabled(history.current.up().is_some()),
                Entry::separator(),
                cmd("My Documents", Cmd::Documents),
                cmd("My Computer", Cmd::Computer),
                cmd("Recycle Bin", Cmd::Trash),
            ],
        ),
        cmd("Refresh", Cmd::Refresh),
    ]
}

/// The titles of all boxes, for the list of the collapsed ones.
const BOX_TITLES: [&str; 5] = [
    "File and Folder Tasks",
    "System Tasks",
    "Recycle Bin Tasks",
    "Other Places",
    "Details",
];

/// The icons of a place, the most fitting first.
fn place_icons(place: &Place) -> &'static [&'static str] {
    match place {
        Place::Folder(path) if *path == documents() => &["folder-documents", "folder"],
        Place::Folder(_) => &["folder"],
        Place::Computer => &["computer"],
        Place::Trash => &["user-trash-full", "user-trash"],
    }
}

/// My Documents: `XDG_DOCUMENTS_DIR`, else the home directory.
pub fn documents() -> PathBuf {
    crate::desktop_icons::user_dir("DOCUMENTS").unwrap_or_else(home)
}

/// The Desktop folder: `XDG_DESKTOP_DIR`, else `~/Desktop`.
fn desktop() -> PathBuf {
    crate::desktop_icons::user_dir("DESKTOP").unwrap_or_else(|| home().join("Desktop"))
}

fn home() -> PathBuf {
    std::env::var_os("HOME").map_or_else(|| PathBuf::from("/"), PathBuf::from)
}

fn count(n: usize) -> i32 {
    i32::try_from(n).unwrap_or(i32::MAX)
}

/// The rows of the Views button: only Tiles works.
fn views_menu() -> Vec<Entry> {
    vec![
        Entry::gray("Thumbnails"),
        Entry::item("Tiles", Act::Explore(Cmd::Refresh)).checked(true),
        Entry::gray("Icons"),
        Entry::gray("List"),
        Entry::gray("Details"),
    ]
}

/// The list of the address bar: the places of XP's list.
fn places_menu() -> Vec<Entry> {
    vec![
        Entry::item("My Documents", Act::Explore(Cmd::Documents)),
        Entry::item("My Computer", Act::Explore(Cmd::Computer)),
        Entry::item("Recycle Bin", Act::Explore(Cmd::Trash)),
    ]
}

// The pictures

/// The chrome of a client area of a size: the menu bar, the toolbar and
/// the address bar on XP's rebar, the lines between them and their
/// separators, the task pane and the white view.
fn chrome([w, h]: [i32; 2]) -> String {
    let (left, right) = REBAR;
    let mut svg = format!(
        r##"<defs><linearGradient id="rebar" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="{left}"/><stop offset="1" stop-color="{right}"/></linearGradient><linearGradient id="address" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#EFEFE9"/><stop offset="0.8" stop-color="#E9E8DD"/><stop offset="1" stop-color="#CDCABB"/></linearGradient><linearGradient id="pane" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#7BA2E7"/><stop offset="1" stop-color="#6375D6"/></linearGradient></defs><rect width="{w}" height="{ADDRESS_Y}" fill="url(#rebar)"/><rect y="{ADDRESS_Y}" width="{w}" height="{ADDRESS_H}" fill="url(#address)"/><rect y="{VIEW_Y}" width="{PANE_W}" height="{ph}" fill="url(#pane)"/><rect x="{PANE_W}" y="{VIEW_Y}" width="{vw}" height="{ph}" fill="#FFFFFF"/>"##,
        ph = h - VIEW_Y,
        vw = w - PANE_W,
    );
    // The lines below the menu bar and the toolbar: dark, then white.
    for y in [MENU_H, TOOLBAR_Y + TOOLBAR_H] {
        let _ = write!(
            svg,
            r##"<rect y="{y}" width="{w}" height="1" fill="#D8D2BD"/><rect y="{}" width="{w}" height="1" fill="#FFFFFF"/>"##,
            y + 1
        );
    }
    // The separators of the toolbar.
    for x in [164, 316] {
        let _ = write!(
            svg,
            r##"<rect x="{x}" y="{}" width="1" height="35" fill="#CAC6AF"/>"##,
            TOOLBAR_Y + 2
        );
    }
    // The address field.
    svg += &crate::dialog::field([49, ADDRESS_Y, w - 49 - 57, ADDRESS_H]);
    svg
}

/// The title of a menu: white on XP's blue under the pointer.
fn menu_title(theme: &Theme, title: &str, x: i32) -> Control {
    #[allow(clippy::cast_possible_truncation)]
    let width = theme.text_width(title, Font::Ui).ceil() as i32 + 12;
    let rect = [x - 6, 1, width, MENU_H - 2];
    let draw = |color, hot: bool| {
        let mut picture = Picture::new(width, MENU_H - 2);
        if hot {
            picture = picture.svg(format!(
                r##"<rect width="{width}" height="{}" fill="#316AC5"/>"##,
                MENU_H - 2
            ));
        }
        picture.text(Text::new(
            title,
            Font::Ui,
            color,
            [6, 0, width - 6, MENU_H - 2],
        ))
    };
    Control::hover(
        &format!("menu-{title}"),
        rect,
        draw(BLACK, false),
        draw(WHITE, true),
    )
}

/// The light frame of a toolbar button under the pointer.
fn hot_frame(w: i32, h: i32) -> String {
    format!(
        r##"<defs><linearGradient id="hot" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#FFFFFF"/><stop offset="0.6" stop-color="#F7F6F1"/><stop offset="1" stop-color="#E6E3D3"/></linearGradient></defs><rect x="0.5" y="0.5" width="{}" height="{}" rx="3" fill="url(#hot)" stroke="#CECEC3"/>"##,
        w - 1,
        h - 1
    )
}

/// The blue arrow button of the address field, which opens its list.
fn arrow_control(name: &str, [x, y, w, h]: Rect) -> Control {
    let normal = Picture::new(w, h).svg(crate::dialog::arrow_button([0, 0, w, h], false));
    let hot = normal
        .clone()
        .svg(format!(
            r##"<rect x="0.5" y="0.5" width="{}" height="{}" rx="1.5" fill="#FFFFFF" fill-opacity="0.3"/>"##,
            w - 1,
            h - 1
        ));
    Control::hover(name, [x, y, w, h], normal, hot)
}

/// Go, right of the address field: XP's green arrow and the word.
fn go_button(icon: Option<&std::path::Path>, rect: Rect) -> Control {
    let [_, _, w, h] = rect;
    let draw = |hot: bool| {
        let mut picture = Picture::new(w, h);
        if hot {
            picture = picture.svg(hot_frame(w, h));
        }
        if let Some(icon) = icon {
            picture = picture.file(icon, [4, (h - 16) / 2, 16, 16]);
        }
        picture.text(Text::new("Go", Font::Ui, BLACK, [26, 0, w - 26, h]))
    };
    Control::hover("go", rect, draw(false), draw(true))
}

/// The header of a box of the task pane: rounded at the top, white
/// growing blue to the right, the title in bold blue and the round
/// button with XP's double chevron, up for an open box.
#[allow(clippy::many_single_char_names)] // coordinates
fn box_header(title: &str, [x, y]: [i32; 2], collapsed: bool) -> Control {
    let (w, h) = (BOX_W, BOX_HEAD);
    let draw = |hot: bool| {
        let color = if hot { LINK_HOT } else { LINK };
        let chevron = if hot { "#428EFF" } else { "#215DC6" };
        let mut svg = format!(
            r##"<defs><linearGradient id="head" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#FFFFFF"/><stop offset="0.45" stop-color="#FFFFFF"/><stop offset="1" stop-color="#C6D3F7"/></linearGradient><linearGradient id="knob" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#FFFFFF"/><stop offset="1" stop-color="#D8E1F7"/></linearGradient></defs><path d="M0 {h}V3Q0 0 3 0H{r}Q{w} 0 {w} 3V{h}Z" fill="url(#head)"/><circle cx="{cx}" cy="{cy}" r="8.5" fill="url(#knob)" stroke="#C3CDE5"/>"##,
            r = w - 3,
            cx = f64::from(w) - 14.5,
            cy = 11.5,
        );
        // Two chevrons, up when the box is open.
        let x = f64::from(w) - 17.5;
        for top in [7.5, 11.5] {
            let d = if collapsed {
                format!("M{x} {top}l3 3l3 -3")
            } else {
                format!("M{x} {}l3 -3l3 3", top + 3.0)
            };
            let _ = write!(
                svg,
                r#"<path d="{d}" fill="none" stroke="{chevron}" stroke-width="1.6"/>"#
            );
        }
        Picture::new(w, h)
            .svg(svg)
            .text(Text::new(title, Font::UiBold, color, [13, 1, w - 40, h]))
    };
    Control::hover(
        &format!("box-{title}"),
        [x, y, w, h],
        draw(false),
        draw(true),
    )
}

/// The body of a box: XP's pale blue with a white border.
fn box_body([x, y, w, h]: Rect) -> String {
    format!(
        r##"<rect x="{}" y="{y}" width="{}" height="{}" fill="#D6DFF7" stroke="#FFFFFF"/>"##,
        f64::from(x) + 0.5,
        w - 1,
        f64::from(h) - 0.5
    )
}

/// A link of a box: the icon and the text in blue, lighter and
/// underlined under the pointer; gray without a target.
fn link_control(
    theme: &Theme,
    name: &str,
    rect: Rect,
    link: &Link,
    icon: Option<&std::path::Path>,
) -> Control {
    let [_, _, w, h] = rect;
    let enabled = link.target != Target::None;
    let draw = |hot: bool| {
        let mut picture = Picture::new(w, h);
        if let Some(icon) = icon {
            picture = if enabled {
                picture.file(icon, [2, 2, 16, 16])
            } else {
                picture.gray(icon, [2, 2, 16, 16])
            };
        }
        let color = match (enabled, hot) {
            (false, _) => GRAY,
            (true, false) => LINK,
            (true, true) => LINK_HOT,
        };
        picture = picture.text(Text::new(&link.title, Font::Ui, color, [24, 0, w - 24, h]));
        if hot {
            #[allow(clippy::cast_possible_truncation)]
            let width = (theme.text_width(&link.title, Font::Ui).ceil() as i32).min(w - 24);
            picture = picture.svg(format!(
                r#"<rect x="24" y="{}" width="{width}" height="1" fill="rgb({},{},{})"/>"#,
                h / 2 + 5,
                color[0],
                color[1],
                color[2]
            ));
        }
        picture
    };
    Control::hover(name, rect, draw(false), draw(true)).enabled(enabled)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialog::tests::theme;

    /// The whole client area as one picture: `CHORA_XP_PREVIEW=dir cargo
    /// test` writes it there, in a frame, as the scene shows it.
    pub fn preview(theme: &mut Theme, explorer: &Explorer, name: &str) {
        let Some(dir) = std::env::var_os("CHORA_XP_PREVIEW") else {
            return;
        };
        let page = explorer.content.page();
        let [w, h] = explorer.size;
        let mut picture = Picture::new(w, h);
        let mut layers = Vec::new();
        if let Some(body) = &page.body {
            layers.push((theme.render(body), [0, 0], body.size));
        }
        for c in &page.controls {
            let p = c.picture(false, false);
            layers.push((theme.render(&p), [c.rect[0], c.rect[1]], p.size));
        }
        for (file, [x, y], [w, h]) in layers {
            picture = picture.file(std::path::Path::new(&file), [x, y, w, h]);
        }
        let out = theme.render(&picture);
        let target = PathBuf::from(dir).join(format!("{name}.png"));
        std::fs::copy(out, &target).expect("the preview is written");
    }

    #[test]
    fn the_chrome_looks_like_xp() {
        let (mut theme, cache) = theme("explorer");
        let mut icons = Icons::default();
        let mut explorer = Explorer::new("xpx1", Place::Computer, SIZE);
        explorer.create(&mut theme, &mut icons);
        preview(&mut theme, &explorer, "explorer-computer");
        // A click on a box header collapses the box.
        assert_eq!(explorer.click("box-System Tasks"), Outcome::Redraw);
        assert_eq!(explorer.collapsed, ["System Tasks"]);
        assert_eq!(explorer.click("box-System Tasks"), Outcome::Redraw);
        assert_eq!(explorer.collapsed, Vec::<&str>::new());
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn the_buttons_walk_the_history() {
        let mut explorer = Explorer::new("xpx1", Place::Folder("/usr/share".into()), SIZE);
        assert_eq!(explorer.click("back"), Outcome::None);
        assert_eq!(explorer.click("up"), Outcome::Redraw);
        assert_eq!(explorer.history.current, Place::Folder("/usr".into()));
        assert_eq!(explorer.click("back"), Outcome::Redraw);
        assert_eq!(explorer.title(), "share");
        assert_eq!(explorer.click("forward"), Outcome::Redraw);
        assert_eq!(explorer.title(), "usr");
    }

    #[test]
    fn the_menus_open_below_their_titles() {
        let mut explorer = Explorer::new("xpx1", Place::Computer, SIZE);
        assert!(matches!(
            explorer.click("menu-File"),
            Outcome::Menu(_, [4, 22])
        ));
        assert_eq!(explorer.command(Cmd::Close), Outcome::Close);
    }
}
