//! The start menu: XP's header with the user name, programs on the left,
//! places on the right, and Log Off and Turn Off Computer in the footer.
//! The geometry follows the measurements of the reference start menu
//! (`XpTheme.qml`) at XP Normal size: 380 pixels wide, a 66 pixel
//! header, a 38 pixel footer, rows of 34 and 42 pixels.

use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use chora_kit::icons::Icons;
use chora_scene::{Command, NodeKind, PropsUpdate, Transform};

use crate::{
    apps::{self, Node},
    cascade::{self, Cascade, Margins, Row},
    config::Config,
    context::{Key, wrap},
    control::{self, Setting},
    desktop_icons, sheets,
    text::{Font, Text},
    theme::{Picture, Rect, Theme},
};

const WIDTH: i32 = 380;
const HEADER: i32 = 66;
const FOOTER: i32 = 38;
/// The frame around the columns.
const FRAME: i32 = 2;
const LEFT: i32 = 190;
const TILE: i32 = 48;
const TILE_X: i32 = 6;
const TILE_Y: i32 = 3;
const PROGRAM_ROW: i32 = 34;
const PINNED_ROW: i32 = 42;
const PLACE_ROW: i32 = 26;
const SEPARATOR: i32 = 6;
const ALL_PROGRAMS: i32 = 26;
/// XP opens and closes cascades after the pointer rests this long: 400
/// milliseconds, on the label of a row as well as on its arrow.
const HOVER_DELAY: Duration = Duration::from_millis(400);
const GROUP: &str = "xp-menu";
/// The node that takes the keyboard while the menu is open.
pub const FOCUS: &str = GROUP;
const FRAME_ID: &str = "xp-menu-frame";
const CONTROL_PANEL: &str = "Control Panel";

/// What an entry does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Runs a shell command.
    Run(String),
    /// Opens the Log Off Windows dialog.
    LogOff,
    /// Opens the Turn off computer dialog.
    TurnOff,
    /// Changes a setting of the Control Panel.
    Set(Setting),
    /// Opens Help and Support on its first topic.
    Help,
    /// Locks the screen.
    Lock,
    /// Takes a screenshot.
    Screenshot,
}

/// The column or the footer of an entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Program,
    Place,
    AllPrograms,
    Footer,
}

/// One entry of the menu.
struct Entry {
    title: String,
    subtitle: Option<String>,
    icon: Option<PathBuf>,
    bold: bool,
    kind: Kind,
    action: Action,
    /// All Programs and the Control Panel open a cascade.
    cascade: bool,
    /// The rectangle in the menu.
    rect: Rect,
}

/// What went down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Press {
    /// An entry of the menu.
    Entry(usize),
    /// A row of a cascade: its level and index.
    Row(usize, usize),
}

/// An entry of the menu or a program of a cascade, for its context menu
/// and its Properties.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    /// The name.
    pub title: String,
    /// What Open does.
    pub action: Action,
    /// The icon.
    pub icon: Option<PathBuf>,
    /// The index of a program of the left column, which can leave it.
    pub program: Option<usize>,
}

/// The start menu, one for all outputs, with the cascades of All
/// Programs.
pub struct Menu {
    user: String,
    entries: Vec<Entry>,
    /// The left column ends with a separator and All Programs.
    separators: Vec<Rect>,
    height: i32,
    /// The output whose start button opened the menu.
    open: Option<String>,
    /// What is down.
    pressed: Option<Press>,
    /// The entry under the pointer.
    hovered: Option<usize>,
    /// The top-left corner in scene coordinates.
    origin: [i32; 2],
    /// The work area of the output, for the cascades.
    area: Rect,
    /// The node of the menu and of the cascades.
    parent: String,
    /// The contents of All Programs.
    programs: Vec<Node>,
    /// The contents of the Control Panel, made when the menu opens.
    control: Vec<Node>,
    /// The entry whose cascade is open, or was open last.
    root: usize,
    /// The open cascades, the one of the entry first.
    cascades: Vec<Cascade>,
    margins: Margins,
    /// An entry or a row that the pointer rests on, and when its cascade
    /// opens or closes.
    pending: Option<(Instant, Press)>,
    /// The entry or the row whose context menu is open: it stays lit.
    held: Option<Press>,
}

impl Menu {
    /// Lays out the menu.
    pub fn new(config: &Config, apps: Vec<Node>, theme: &Theme, icons: &mut Icons) -> Self {
        let places = places(config, icons);
        let pool = fill_pool(config, &apps);
        let right = right_height(&places);
        let mut extra = 0;
        let mut entries = programs(config, icons, &pool[..extra]);
        while left_height(&entries) < right && extra < pool.len() {
            extra += 1;
            entries = programs(config, icons, &pool[..extra]);
        }
        let body = body_height(&entries, &places);
        let height = HEADER + 1 + body + 1 + FOOTER;
        let top = HEADER + 1;
        // XP: 4 pixels from the column, 6 from its right and bottom.
        let all = [
            FRAME + 4,
            top + body - 6 - ALL_PROGRAMS,
            LEFT - 10,
            ALL_PROGRAMS,
        ];
        let mut separators = vec![[FRAME + 2, all[1] - 2 - SEPARATOR, LEFT - 8, SEPARATOR]];
        if entries.len() > 2 {
            separators.push([FRAME + 2, top + 2 * PINNED_ROW, LEFT - 8, SEPARATOR]);
        }
        separators.extend(place_separators(&places, top));
        entries.push(Entry {
            title: "All Programs".into(),
            subtitle: None,
            icon: None,
            bold: true,
            kind: Kind::AllPrograms,
            // Without desktop files, All Programs runs the launcher.
            action: Action::Run(config.launcher.clone()),
            cascade: !apps.is_empty(),
            rect: all,
        });
        entries.extend(places);
        entries.extend(footer(theme, height));
        let root = entries
            .iter()
            .position(|e| e.kind == Kind::AllPrograms)
            .unwrap_or(0);
        Self {
            user: account_name(),
            entries,
            separators,
            height,
            open: None,
            pressed: None,
            hovered: None,
            origin: [0, 0],
            area: [0, 0, 1280, 770],
            parent: String::new(),
            programs: apps,
            control: Vec::new(),
            root,
            cascades: Vec::new(),
            margins: Margins::of(theme),
            pending: None,
            held: None,
        }
    }

    /// Creates the hidden menu under a node: the frame, the header and
    /// one image per entry.
    pub fn create(&mut self, theme: &mut Theme, parent: &str) -> Vec<Command> {
        self.parent = parent.into();
        let mut commands = vec![
            Command::Create {
                id: GROUP.into(),
                kind: NodeKind::Group,
                parent: parent.into(),
                props: PropsUpdate {
                    visible: Some(false),
                    ..PropsUpdate::default()
                },
            },
            Command::Create {
                id: FRAME_ID.into(),
                kind: NodeKind::Image,
                parent: GROUP.into(),
                props: PropsUpdate {
                    image: Some(theme.render(&self.chrome(theme))),
                    ..PropsUpdate::default()
                },
            },
        ];
        for (i, entry) in self.entries.iter().enumerate() {
            commands.push(Command::Create {
                id: item_id(i),
                kind: NodeKind::Image,
                parent: GROUP.into(),
                props: PropsUpdate {
                    image: Some(theme.render(&picture(theme, entry, false))),
                    meta: Some([("text".to_owned(), entry.title.clone())].into()),
                    ..at(entry.rect[0], entry.rect[1])
                },
            });
        }
        commands
    }

    /// The header, the columns and the footer, with the user name.
    fn chrome(&self, theme: &Theme) -> Picture {
        let body = self.height - HEADER - FOOTER - 2;
        let header = theme
            .size(sheets::WINXP, "startmenu-header")
            .unwrap_or([WIDTH, 67]);
        let footer = theme
            .size(sheets::WINXP, "startmenu-footer")
            .unwrap_or([WIDTH, 41]);
        let face = std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".face"));
        let name = Text::new(
            &self.user,
            Font::Name,
            [255, 255, 255, 255],
            [67, 7, WIDTH - 77, 52],
        )
        .shadow([0, 0, 0, 100]);
        let mut picture = Picture::new(WIDTH, self.height)
            .element(
                sheets::WINXP,
                "startmenu-body",
                [0, header[1], WIDTH, body + HEADER + 1 - header[1] + 1],
            )
            .layers_of(
                header_art(theme, face.as_deref().filter(|f| f.is_file()), header[1]),
                [0, 0],
            )
            .element(
                sheets::WINXP,
                "startmenu-footer",
                [0, self.height - footer[1], WIDTH, footer[1]],
            )
            .text(name);
        for rect in &self.separators {
            let id = if rect[0] < LEFT {
                "startmenu-separator"
            } else {
                "startmenu-places-separator"
            };
            let [x, y, w, h] = *rect;
            picture = picture.element(sheets::WINXP, id, [x, y + h / 2, w, 1]);
        }
        picture
    }

    /// Whether the menu shows.
    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    /// Shows the menu above a start button, with the Control Panel of the
    /// output; its cascades stay in the work area.
    pub fn open(
        &mut self,
        output: &str,
        [x, bottom]: [i32; 2],
        area: Rect,
        control: Vec<Node>,
    ) -> Vec<Command> {
        self.open = Some(output.into());
        self.control = control;
        self.origin = [x, bottom - self.height];
        self.area = area;
        vec![
            Command::Update {
                id: GROUP.into(),
                props: PropsUpdate {
                    visible: Some(true),
                    ..at(self.origin[0], self.origin[1])
                },
            },
            Command::Move {
                id: GROUP.into(),
                parent: None,
                index: None,
            },
        ]
    }

    /// Hides the menu and closes its cascades.
    pub fn close(&mut self, theme: &mut Theme) -> Vec<Command> {
        if self.open.take().is_none() {
            return Vec::new();
        }
        self.pending = None;
        self.pressed = None;
        let mut commands = self.close_from(theme, 0);
        if let Some(i) = self.hovered.take() {
            commands.push(self.look(theme, i));
        }
        commands.push(Command::Update {
            id: GROUP.into(),
            props: PropsUpdate {
                visible: Some(false),
                ..PropsUpdate::default()
            },
        });
        commands
    }

    /// The output of the open menu.
    pub fn output(&self) -> Option<&str> {
        self.open.as_deref()
    }

    /// Whether a node belongs to the menu or its cascades.
    pub fn owns(node: &str) -> bool {
        node.starts_with("xp-menu") || node.starts_with("xp-item-")
    }

    /// What a node of the menu is.
    fn part(&self, node: &str) -> Option<Press> {
        if let Some(i) = item_index(node).filter(|i| *i < self.entries.len()) {
            return Some(Press::Entry(i));
        }
        let (level, i) = cascade::row_of(node)?;
        (self.cascades.get(level)?.count > i).then_some(Press::Row(level, i))
    }

    /// The program or place that a node shows: an entry of the columns or
    /// a program of a cascade. `None` for All Programs, the Control Panel,
    /// the footer and the groups.
    pub fn item(&self, node: &str, icons: &mut Icons) -> Option<Item> {
        match self.part(node)? {
            Press::Entry(i) => {
                let e = &self.entries[i];
                let program = match e.kind {
                    Kind::Program => i.checked_sub(2),
                    Kind::Place if !e.cascade => None,
                    _ => return None,
                };
                Some(Item {
                    title: e.title.clone(),
                    action: e.action.clone(),
                    icon: e.icon.clone(),
                    program,
                })
            }
            Press::Row(level, i) => {
                let node = self.nodes(level).get(i)?;
                let apps::Kind::Run(command) = &node.kind else {
                    return None;
                };
                Some(Item {
                    title: node.title.clone(),
                    action: Action::Run(command.clone()),
                    icon: program_art(icons, &node.icon),
                    program: None,
                })
            }
        }
    }

    /// The menu of other programs and places, made again under its node.
    pub fn rebuild(
        &mut self,
        config: &Config,
        apps: Vec<Node>,
        theme: &mut Theme,
        icons: &mut Icons,
    ) -> Vec<Command> {
        let parent = self.parent.clone();
        *self = Self::new(config, apps, theme, icons);
        let mut commands = vec![Command::Remove {
            id: GROUP.into(),
            to: None,
        }];
        commands.extend(self.create(theme, &parent));
        commands
    }

    /// Highlights the entry or the row under the pointer, as XP does, or
    /// ends the highlight when the pointer leaves it. Resting on it opens
    /// or closes cascades after a delay.
    pub fn hover(&mut self, theme: &mut Theme, node: &str, on: bool) -> Vec<Command> {
        let Some(part) = self.part(node) else {
            return Vec::new();
        };
        // The entry of an open context menu stays lit.
        if !on && self.held == Some(part) {
            return Vec::new();
        }
        if on {
            self.pending = Some((Instant::now() + HOVER_DELAY, part));
        }
        match part {
            Press::Entry(index) => {
                if on {
                    self.hovered = Some(index);
                } else if self.hovered == Some(index) {
                    self.hovered = None;
                }
                vec![self.look(theme, index), self.look(theme, self.root)]
            }
            Press::Row(level, index) => self.relight(theme, level, |c| {
                if on {
                    c.hovered = Some(index);
                } else if c.hovered == Some(index) {
                    c.hovered = None;
                }
            }),
        }
    }

    /// The context menu of an entry or a row opens, and it stays lit, as
    /// on XP; with no node the menu closed, and the light goes.
    pub fn hold(&mut self, theme: &mut Theme, node: Option<&str>) -> Vec<Command> {
        if let Some(part) = node.and_then(|n| self.part(n)) {
            self.held = Some(part);
            return Vec::new();
        }
        match self.held.take() {
            Some(Press::Entry(i)) if self.hovered == Some(i) => {
                self.hovered = None;
                vec![self.look(theme, i)]
            }
            Some(Press::Row(level, i)) => self.relight(theme, level, |c| {
                if c.hovered == Some(i) {
                    c.hovered = None;
                }
            }),
            _ => Vec::new(),
        }
    }

    /// When the pointer has rested long enough for a cascade to open or
    /// close.
    pub fn due(&self) -> Option<Instant> {
        self.pending.map(|(at, _)| at)
    }

    /// Opens the cascade of the entry or row that the pointer rests on, or
    /// closes the cascades that it left.
    pub fn rest(&mut self, theme: &mut Theme, icons: &mut Icons, now: Instant) -> Vec<Command> {
        let Some((_, part)) = self.pending.filter(|(at, _)| now >= *at) else {
            return Vec::new();
        };
        self.pending = None;
        match part {
            // Only for what is still under the pointer, not for what it
            // crossed on its way into a cascade.
            Press::Entry(i) if self.hovered != Some(i) => Vec::new(),
            Press::Entry(i) if self.entries[i].cascade => self.open_root(theme, icons, i),
            Press::Entry(_) => self.close_from(theme, 0),
            Press::Row(level, i) if self.cascades[level].hovered != Some(i) => Vec::new(),
            Press::Row(level, i) => self.open_row(theme, icons, level, i),
        }
    }

    /// Highlights an entry that went down; a click on empty space of the
    /// menu closes the cascades.
    pub fn press(&mut self, theme: &mut Theme, node: &str) -> Vec<Command> {
        if node == FRAME_ID {
            return self.close_from(theme, 0);
        }
        let Some(part) = self.part(node) else {
            return Vec::new();
        };
        self.pressed = Some(part);
        match part {
            Press::Entry(index) => vec![self.look(theme, index)],
            Press::Row(..) => Vec::new(),
        }
    }

    /// The action of the entry or row that went down, if the pointer is
    /// still on it, and the commands that end the highlight or open a
    /// cascade.
    pub fn release(
        &mut self,
        theme: &mut Theme,
        icons: &mut Icons,
        position: [f64; 2],
    ) -> (Vec<Command>, Option<Action>) {
        match self.pressed.take() {
            Some(Press::Entry(index)) => {
                let [x, y, w, h] = self.entries[index].rect;
                let rect = [self.origin[0] + x, self.origin[1] + y, w, h];
                let mut commands = vec![self.look(theme, index)];
                if !inside(rect, position) {
                    return (commands, None);
                }
                if self.entries[index].cascade {
                    self.pending = None;
                    commands.extend(self.open_root(theme, icons, index));
                    return (commands, None);
                }
                (commands, Some(self.entries[index].action.clone()))
            }
            Some(Press::Row(level, index)) => {
                let on_row = self
                    .cascades
                    .get(level)
                    .is_some_and(|c| inside(c.row(index), position));
                if !on_row {
                    return (Vec::new(), None);
                }
                self.pending = None;
                match &self.nodes(level)[index].kind {
                    apps::Kind::Run(command) => (Vec::new(), Some(Action::Run(command.clone()))),
                    apps::Kind::Pick(setting, _) => {
                        (Vec::new(), Some(Action::Set(setting.clone())))
                    }
                    apps::Kind::Group(_) => (self.open_row(theme, icons, level, index), None),
                }
            }
            None => (Vec::new(), None),
        }
    }

    /// A key while the menu is open, as on XP: Up and Down move the
    /// highlight in a column or a cascade, Left and Right go between the
    /// columns and into and out of the cascades, Return acts, and Escape
    /// closes the innermost cascade, then the menu.
    pub fn key(&mut self, theme: &mut Theme, icons: &mut Icons, key: &str) -> Key<Action> {
        let Some(level) = self.cascades.iter().rposition(|c| c.hovered.is_some()) else {
            return self.entry_key(theme, icons, key);
        };
        let count = self.cascades[level].count;
        let current = self.cascades[level].hovered.unwrap_or(0);
        match key {
            "Home" | "End" => {
                let row = if key == "Home" { 0 } else { count - 1 };
                Key::Commands(self.relight(theme, level, |c| c.hovered = Some(row)))
            }
            "Down" | "Up" => {
                let next = wrap(current, key == "Down", count);
                Key::Commands(self.relight(theme, level, |c| c.hovered = Some(next)))
            }
            "Right" | "Return" | "KP_Enter" | "space" => match &self.nodes(level)[current].kind {
                apps::Kind::Group(_) => {
                    let mut commands = self.open_row(theme, icons, level, current);
                    commands.extend(self.relight(theme, level + 1, |c| c.hovered = Some(0)));
                    Key::Commands(commands)
                }
                _ if key == "Right" => Key::Commands(Vec::new()),
                apps::Kind::Run(command) => Key::Act(Action::Run(command.clone())),
                apps::Kind::Pick(setting, _) => Key::Act(Action::Set(setting.clone())),
            },
            "Left" | "Escape" => {
                let mut commands = self.close_from(theme, level);
                if level == 0 {
                    // Back to the entry of the cascade.
                    let old = self.hovered.replace(self.root);
                    commands.extend(old.map(|i| self.look(theme, i)));
                    commands.push(self.look(theme, self.root));
                } else {
                    commands.extend(self.relight(theme, level - 1, |_| {}));
                }
                Key::Commands(commands)
            }
            _ => Key::Commands(Vec::new()),
        }
    }

    /// A key on the entries of the two columns.
    fn entry_key(&mut self, theme: &mut Theme, icons: &mut Icons, key: &str) -> Key<Action> {
        let target = match (key, self.hovered) {
            // A cascade that the pointer opened closes first.
            ("Escape", _) => {
                return match self.cascades.len().checked_sub(1) {
                    Some(innermost) => Key::Commands(self.close_from(theme, innermost)),
                    None => Key::Close,
                };
            }
            // &Log Off and T&urn Off Computer, as on XP.
            ("l" | "L", _) => return Key::Act(Action::LogOff),
            ("u" | "U", _) => return Key::Act(Action::TurnOff),
            ("Down" | "Up", None) => self.column(true).first().copied(),
            ("Home" | "End", Some(i)) => {
                let column = self.column(self.entries[i].rect[0] < LEFT);
                if key == "Home" {
                    column.first()
                } else {
                    column.last()
                }
                .copied()
            }
            ("Down" | "Up", Some(i)) => {
                let column = self.column(self.entries[i].rect[0] < LEFT);
                let at = column.iter().position(|&j| j == i).unwrap_or(0);
                Some(column[wrap(at, key == "Down", column.len())])
            }
            ("Right" | "Return" | "KP_Enter" | "space", Some(i)) if self.entries[i].cascade => {
                let mut commands = self.open_root(theme, icons, i);
                commands.extend(self.relight(theme, 0, |c| c.hovered = Some(0)));
                return Key::Commands(commands);
            }
            ("Return" | "KP_Enter" | "space", Some(i)) => {
                return Key::Act(self.entries[i].action.clone());
            }
            ("Left" | "Right", Some(i)) => {
                // The entry of the other column nearest in height.
                let [x, y, ..] = self.entries[i].rect;
                self.column(x >= LEFT)
                    .into_iter()
                    .min_by_key(|&j| (self.entries[j].rect[1] - y).abs())
            }
            _ => None,
        };
        let Some(target) = target else {
            return Key::Commands(Vec::new());
        };
        let old = self.hovered.replace(target);
        let mut commands: Vec<Command> = old.map(|i| self.look(theme, i)).into_iter().collect();
        commands.push(self.look(theme, target));
        Key::Commands(commands)
    }

    /// The entries of the left or the right column, from the top, that the
    /// arrow keys walk: Log Off and Turn Off take no part, as on XP.
    fn column(&self, left: bool) -> Vec<usize> {
        let mut column: Vec<usize> = (0..self.entries.len())
            .filter(|&j| {
                let e = &self.entries[j];
                e.kind != Kind::Footer && (e.rect[0] < LEFT) == left
            })
            .collect();
        column.sort_by_key(|&j| self.entries[j].rect[1]);
        column
    }

    /// The entries of the cascade of a level.
    fn nodes(&self, level: usize) -> &[Node] {
        let mut nodes = if self.entries[self.root].kind == Kind::AllPrograms {
            &self.programs[..]
        } else {
            &self.control[..]
        };
        for cascade in &self.cascades[..level] {
            match cascade.open.map(|i| &nodes[i].kind) {
                Some(apps::Kind::Group(children)) => nodes = children,
                _ => return &[],
            }
        }
        nodes
    }

    /// Opens the cascade of All Programs or of the Control Panel, unless
    /// it is open, and closes the other one. All Programs grows up from its
    /// button; the Control Panel opens beside the menu, level with its
    /// row.
    fn open_root(&mut self, theme: &mut Theme, icons: &mut Icons, index: usize) -> Vec<Command> {
        if self.root == index && !self.cascades.is_empty() {
            return Vec::new();
        }
        let mut commands = self.close_from(theme, 0);
        self.root = index;
        let [x, y, w, h] = self.entries[index].rect;
        let row = [self.origin[0] + x, self.origin[1] + y, w, h];
        let parent = (self.entries[index].kind != Kind::AllPrograms).then(|| {
            // As if the menu had the shadow of a cascade.
            let shadow = self.margins.shadow();
            [self.origin[0], self.origin[1], WIDTH + shadow, self.height]
        });
        commands.extend(self.open_cascade(theme, icons, row, parent));
        commands.push(self.look(theme, index));
        commands
    }

    /// The pointer rests on a row or clicked it: a group opens its
    /// cascade, a program closes the cascades of other groups.
    fn open_row(
        &mut self,
        theme: &mut Theme,
        icons: &mut Icons,
        level: usize,
        index: usize,
    ) -> Vec<Command> {
        let group = matches!(self.nodes(level)[index].kind, apps::Kind::Group(_));
        if !group {
            return self.close_from(theme, level + 1);
        }
        if self.cascades[level].open == Some(index) {
            return Vec::new();
        }
        let mut commands = self.close_from(theme, level + 1);
        commands.extend(self.relight(theme, level, |c| c.open = Some(index)));
        let parent = &self.cascades[level];
        let (row, rect) = (parent.row(index), parent.rect);
        commands.extend(self.open_cascade(theme, icons, row, Some(rect)));
        commands
    }

    /// Opens the next cascade beside a row of the menu or of a cascade.
    fn open_cascade(
        &mut self,
        theme: &mut Theme,
        icons: &mut Icons,
        row: Rect,
        parent: Option<Rect>,
    ) -> Vec<Command> {
        let level = self.cascades.len();
        let nodes = self.nodes(level);
        let widths: Vec<f64> = nodes
            .iter()
            .map(|n| theme.text_width(&n.title, Font::Ui))
            .collect();
        let mut cascade = Cascade::new(level, &widths, self.margins, [row, self.area], parent);
        cascade.icons = nodes
            .iter()
            .map(|n| match n.kind {
                apps::Kind::Run(_) => program_art(icons, &n.icon),
                _ => icons.find(&n.icon),
            })
            .collect();
        let rows: Vec<Row<'_>> = nodes
            .iter()
            .zip(&cascade.icons)
            .map(|(n, icon)| row_of(n, icon.as_deref()))
            .collect();
        let commands = cascade.create(theme, &self.parent, &rows);
        self.cascades.push(cascade);
        commands
    }

    /// Closes the cascades from a level on.
    fn close_from(&mut self, theme: &mut Theme, level: usize) -> Vec<Command> {
        if self.cascades.len() <= level {
            return Vec::new();
        }
        let commands: Vec<Command> = self.cascades.drain(level..).map(|c| c.remove()).collect();
        if matches!(self.pressed, Some(Press::Row(l, _)) if l >= level) {
            self.pressed = None;
        }
        let mut commands = commands;
        match level.checked_sub(1) {
            None => commands.push(self.look(theme, self.root)),
            Some(parent) => commands.extend(self.relight(theme, parent, |c| c.open = None)),
        }
        commands
    }

    /// Changes the pointer or the open row of a cascade, and updates the
    /// rows whose highlight changes.
    fn relight(
        &mut self,
        theme: &mut Theme,
        level: usize,
        change: impl FnOnce(&mut Cascade),
    ) -> Vec<Command> {
        let Some(cascade) = self.cascades.get_mut(level) else {
            return Vec::new();
        };
        let old = (cascade.hovered, cascade.open);
        change(cascade);
        let new = (cascade.hovered, cascade.open);
        let mut rows: Vec<usize> = [old.0, old.1, new.0, new.1].into_iter().flatten().collect();
        rows.sort_unstable();
        rows.dedup();
        let nodes = self.nodes(level);
        let cascade = &self.cascades[level];
        rows.into_iter()
            .filter(|i| cascade::lit(old.0, old.1, *i) != cascade.lit(*i))
            .map(|i| {
                let row = row_of(&nodes[i], cascade.icons[i].as_deref());
                cascade.relook(theme, i, &row)
            })
            .collect()
    }

    /// The look of an entry: highlighted under the pointer, while down,
    /// and while its cascade is open and the pointer is on no other entry.
    fn look(&self, theme: &mut Theme, index: usize) -> Command {
        let entry = &self.entries[index];
        let lit = self.hovered == Some(index)
            || self.pressed == Some(Press::Entry(index))
            || (self.root == index && !self.cascades.is_empty() && self.hovered.is_none());
        Command::Update {
            id: item_id(index),
            props: PropsUpdate {
                image: Some(theme.render(&picture(theme, entry, lit))),
                ..PropsUpdate::default()
            },
        }
    }
}

/// What a row of a cascade shows for an entry of All Programs.
fn row_of<'a>(node: &'a Node, icon: Option<&'a Path>) -> Row<'a> {
    Row {
        title: &node.title,
        icon,
        group: matches!(node.kind, apps::Kind::Group(_)),
        checked: matches!(node.kind, apps::Kind::Pick(_, true)),
    }
}

fn inside([x, y, w, h]: Rect, [px, py]: [f64; 2]) -> bool {
    (f64::from(x)..f64::from(x + w)).contains(&px) && (f64::from(y)..f64::from(y + h)).contains(&py)
}

/// The icon of a program, but only art that fills its square: a PNG file
/// whose ink is under 60 % of its side, or whose marks cover under 12 % of
/// it, is a fragment, and the program gets the generic icon of the theme
/// instead.
pub fn program_art(icons: &mut Icons, name: &str) -> Option<PathBuf> {
    let found = icons.program(name);
    if !found.as_deref().is_some_and(is_fragment) {
        return found;
    }
    icons
        .find("applications-other")
        .or_else(|| icons.find("application-x-executable"))
        .or(found)
}

/// Whether a file is a PNG that is a fragment of an icon.
pub fn is_fragment(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == "png")
        && ink_of(path).is_some_and(|(side, marks)| side < 0.6 || marks < 0.12)
}

/// The larger side of the ink of a PNG file as a share of its larger side,
/// and the share of its pixels that are marks: not clear and not white (in
/// a premultiplied pixel white has all channels at its alpha).
fn ink_of(path: &Path) -> Option<(f64, f64)> {
    let image = resvg::tiny_skia::Pixmap::load_png(path).ok()?;
    let mut found: Option<[u32; 4]> = None;
    let mut marks = 0u32;
    for y in 0..image.height() {
        for x in 0..image.width() {
            let Some(p) = image.pixel(x, y).filter(|p| p.alpha() > 40) else {
                continue;
            };
            found = Some(found.map_or([x, y, x, y], |f| {
                [f[0].min(x), f[1].min(y), f[2].max(x), f[3].max(y)]
            }));
            if p.red().min(p.green()).min(p.blue()) < p.alpha().saturating_sub(20) {
                marks += 1;
            }
        }
    }
    let [x0, y0, x1, y1] = found?;
    let side = f64::from(image.width().max(image.height()));
    Some((
        f64::from((x1 - x0 + 1).max(y1 - y0 + 1)) / side,
        f64::from(marks) / (f64::from(image.width()) * f64::from(image.height())),
    ))
}

/// The programs of the left column, below each other: the terminal of
/// the settings first.
fn programs(config: &Config, icons: &mut Icons, extra: &[&Node]) -> Vec<Entry> {
    let top = HEADER + 1 + 5;
    let pinned = [
        (
            "Terminal",
            "Type commands",
            config.terminal.clone(),
            icons.program("utilities-terminal"),
        ),
        (
            "Chat with a helper",
            "Ask in your words",
            control::CHAT.to_string(),
            icons
                .first_sized(&["chora-chat", "user-available", "dialog-information"], 32)
                .or_else(|| icons.program("")),
        ),
    ];
    let mut entries: Vec<Entry> = pinned
        .into_iter()
        .zip(0..)
        .map(|((title, subtitle, command, icon), i)| Entry {
            title: title.into(),
            subtitle: Some(subtitle.into()),
            icon,
            bold: true,
            kind: Kind::Program,
            action: Action::Run(command),
            cascade: false,
            rect: [FRAME + 2, top + i * PINNED_ROW, LEFT - 4, PINNED_ROW],
        })
        .collect();
    let recent = top + 2 * PINNED_ROW + SEPARATOR;
    entries.extend(config.programs.iter().zip(0..).map(|(program, i)| Entry {
        title: program.title.clone(),
        subtitle: None,
        icon: program_art(icons, &program.icon),
        bold: false,
        kind: Kind::Program,
        action: Action::Run(program.command.clone()),
        cascade: false,
        rect: [FRAME + 2, recent + i * PROGRAM_ROW, LEFT - 4, PROGRAM_ROW],
    }));
    let first = i32::try_from(config.programs.len()).unwrap_or(0);
    entries.extend(extra.iter().zip(first..).map(|(node, i)| Entry {
        title: node.title.clone(),
        subtitle: None,
        icon: program_art(icons, &node.icon),
        bold: false,
        kind: Kind::Program,
        action: Action::Run(match &node.kind {
            apps::Kind::Run(command) => command.clone(),
            _ => String::new(),
        }),
        cascade: false,
        rect: [FRAME + 2, recent + i * PROGRAM_ROW, LEFT - 4, PROGRAM_ROW],
    }));
    entries
}

/// The programs of the desktop files that fill the left column when the
/// settings list too few for it to reach the height of the right one:
/// Notepad, the plain text editor, then the first programs of All
/// Programs, none that the settings list.
fn fill_pool<'a>(config: &Config, apps: &'a [Node]) -> Vec<&'a Node> {
    fn flatten<'a>(nodes: &'a [Node], out: &mut Vec<&'a Node>) {
        for node in nodes {
            match &node.kind {
                apps::Kind::Group(inner) => flatten(inner, out),
                apps::Kind::Run(_) => out.push(node),
                apps::Kind::Pick(..) => {}
            }
        }
    }
    let mut pool = Vec::new();
    flatten(apps, &mut pool);
    pool.retain(|n| !config.programs.iter().any(|p| p.title == n.title));
    pool.sort_by_key(|n| n.title != apps::NOTEPAD);
    pool
}

/// The name that the header shows: the full name of the account, else its
/// login name.
fn account_name() -> String {
    let login = std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "User".into());
    let passwd = std::fs::read_to_string("/etc/passwd").unwrap_or_default();
    full_name(&passwd, &login).unwrap_or(login)
}

/// The first field of the comment of an account in the text of
/// `/etc/passwd`, when it is not empty.
fn full_name(passwd: &str, login: &str) -> Option<String> {
    let line = passwd
        .lines()
        .find(|l| l.split(':').next() == Some(login))?;
    let gecos = line.split(':').nth(4)?;
    let name = gecos.split(',').next()?.trim();
    (!name.is_empty()).then(|| name.to_string())
}

/// The places of the right column; an empty title is a separator.
fn places(config: &Config, icons: &mut Icons) -> Vec<Entry> {
    // The user directories, else the folders of XP's names in the home
    // directory.
    let open = |dir: &Path| Action::Run(desktop_icons::open_with(&config.file_manager, dir));
    let folder = |var: &str, name: &str| {
        let home = std::env::var_os("HOME").map_or_else(|| "/".into(), PathBuf::from);
        open(&desktop_icons::user_dir(var).unwrap_or_else(|| home.join(name)))
    };
    let run = |command: &str| Action::Run(command.into());
    let list = [
        (
            "My Documents",
            folder("DOCUMENTS", ""),
            "folder-documents",
            true,
        ),
        (
            "My Pictures",
            folder("PICTURES", "Pictures"),
            "folder-pictures",
            true,
        ),
        ("My Music", folder("MUSIC", "Music"), "folder-music", true),
        ("My Computer", open(Path::new("/")), "computer", true),
        ("", run(""), "", false),
        // A cascade, as with XP's "Display as a menu".
        (CONTROL_PANEL, run(""), "preferences-system", false),
        ("", run(""), "", false),
        ("Help and Support", Action::Help, "help-browser", false),
        ("Run...", run(&config.launcher), "system-run", false),
        ("Screenshot", Action::Screenshot, "camera-photo", false),
    ];
    let mut y = HEADER + 1 + 5;
    let mut entries = Vec::new();
    for (title, action, icon, bold) in list {
        if title.is_empty() {
            y += SEPARATOR;
            continue;
        }
        entries.push(Entry {
            title: title.into(),
            subtitle: None,
            icon: icons.find(icon),
            bold,
            kind: Kind::Place,
            action,
            cascade: title == CONTROL_PANEL,
            rect: [
                FRAME + LEFT + 1 + 2,
                y,
                WIDTH - LEFT - 2 * FRAME - 5,
                PLACE_ROW,
            ],
        });
        y += PLACE_ROW;
    }
    entries
}

/// The separators of the right column: where a row of places skips a
/// separator's height.
fn place_separators(places: &[Entry], top: i32) -> Vec<Rect> {
    let mut y = top + 5;
    let mut out = Vec::new();
    for entry in places {
        if entry.rect[1] > y {
            out.push([
                FRAME + LEFT + 1 + 2,
                y,
                WIDTH - LEFT - 2 * FRAME - 5,
                SEPARATOR,
            ]);
        }
        y = entry.rect[1] + entry.rect[3];
    }
    out
}

/// The height of the columns: the higher of both.
fn body_height(programs: &[Entry], places: &[Entry]) -> i32 {
    left_height(programs).max(right_height(places))
}

fn bottom(entries: &[Entry]) -> i32 {
    entries
        .iter()
        .map(|e| e.rect[1] + e.rect[3])
        .max()
        .unwrap_or(0)
        - HEADER
        - 1
}

/// The left column ends with a separator and All Programs.
fn left_height(programs: &[Entry]) -> i32 {
    bottom(programs) + SEPARATOR + 2 + ALL_PROGRAMS + 6
}

fn right_height(places: &[Entry]) -> i32 {
    bottom(places) + 6
}

/// Lock, Log Off and Turn Off Computer at the right of the footer.
fn footer(theme: &Theme, height: i32) -> Vec<Entry> {
    let y = height - FOOTER + 7;
    let button = |title: &str, icon: &str, action, x, w| Entry {
        title: title.into(),
        subtitle: None,
        icon: Some(theme.path(icon)),
        bold: false,
        kind: Kind::Footer,
        action,
        cascade: false,
        rect: [x, y, w, 30],
    };
    vec![
        button(
            "Lock",
            "winxp/lock.svg",
            Action::Lock,
            WIDTH - 5 - 176 - 8 - 96 - 8 - 72,
            72,
        ),
        button(
            "Log Off",
            "winxp/log-off.svg",
            Action::LogOff,
            WIDTH - 5 - 176 - 8 - 96,
            96,
        ),
        button(
            "Turn Off Computer",
            "winxp/turn-off.svg",
            Action::TurnOff,
            WIDTH - 5 - 176,
            176,
        ),
    ]
}

/// The header of the menu: XP's blue band with the picture of the user in
/// a square tile of 48 pixels, a ring of 2 pixels white all round.
fn header_art(theme: &Theme, face: Option<&Path>, height: i32) -> Picture {
    let picture = Picture::new(WIDTH, height).element(
        sheets::WINXP,
        "startmenu-header",
        [0, 0, WIDTH, height],
    );
    match face {
        None => picture.file(&theme.path("winxp/user.svg"), [TILE_X, TILE_Y, TILE, TILE]),
        Some(face) => picture
            .file(face, [TILE_X + 2, TILE_Y + 2, TILE - 4, TILE - 4])
            .svg(format!(
                r##"<rect x="{}" y="{}" width="{}" height="{}" rx="4" fill="none" stroke="#ffffff" stroke-width="2"/>"##,
                TILE_X + 1,
                TILE_Y + 1,
                TILE - 2,
                TILE - 2
            )),
    }
}

/// The picture of an entry, normal or highlighted.
fn picture(theme: &Theme, entry: &Entry, down: bool) -> Picture {
    let [_, _, w, h] = entry.rect;
    let mut picture = background(entry, down);
    if entry.kind == Kind::AllPrograms {
        return all_programs(theme, picture, entry, down);
    }
    let icon_size = match entry.kind {
        Kind::Program => 32,
        Kind::Place | Kind::AllPrograms | Kind::Footer => 24,
    };
    let pad = if entry.kind == Kind::Program { 3 } else { 5 };
    if let Some(icon) = &entry.icon {
        picture = picture.file(icon, [pad, (h - icon_size) / 2, icon_size, icon_size]);
    }
    let x = pad + icon_size + if entry.kind == Kind::Footer { 6 } else { 3 };
    let mut width = w - x - 4;
    if entry.cascade {
        picture = cascade::arrow(theme, picture, down);
        width -= 12;
    }
    let color = text_color(entry.kind, down);
    let font = if entry.bold { Font::UiBold } else { Font::Ui };
    match &entry.subtitle {
        Some(sub) => picture
            .text(Text::new(&entry.title, font, color, [x, 2, width, h / 2]))
            .text(Text::new(
                sub,
                Font::Ui,
                subtitle_color(down),
                [x, h / 2 - 2, width, h / 2],
            )),
        None => picture.text(Text::new(&entry.title, font, color, [x, 0, width, h])),
    }
}

/// An empty entry, or its highlight.
fn background(entry: &Entry, down: bool) -> Picture {
    let [_, _, w, h] = entry.rect;
    let picture = Picture::new(w, h);
    match (down, entry.kind) {
        (false, _) => picture,
        (true, Kind::Footer) => {
            picture.frame(sheets::WINXP, "startmenu-footer-pressed", [0, 0, w, h])
        }
        (true, _) => picture.frame(sheets::WINXP, "menu-highlight", [0, 0, w, h]),
    }
}

/// "All Programs" and XP's green arrow, centered together, 18 pixels
/// apart.
fn all_programs(theme: &Theme, picture: Picture, entry: &Entry, down: bool) -> Picture {
    let [_, _, w, h] = entry.rect;
    let arrow = theme
        .size(sheets::WINXP, "startmenu-allprograms-arrow")
        .unwrap_or([17, 22]);
    #[allow(clippy::cast_possible_truncation)]
    let label = theme.text_width(&entry.title, Font::UiBold).ceil() as i32;
    let x = (w - label - 18 - arrow[0]) / 2;
    let color = text_color(entry.kind, down);
    picture
        .text(Text::new(
            &entry.title,
            Font::UiBold,
            color,
            [x, 0, label + 2, h],
        ))
        .element(
            sheets::WINXP,
            "startmenu-allprograms-arrow",
            [x + label + 18, (h - arrow[1]) / 2, arrow[0], arrow[1]],
        )
}

fn text_color(kind: Kind, down: bool) -> [u8; 4] {
    match (kind, down) {
        (Kind::Footer, _) | (_, true) => [255, 255, 255, 255],
        (Kind::Program | Kind::AllPrograms, false) => [0, 0, 0, 255],
        (Kind::Place, false) => [0, 19, 107, 255],
    }
}

fn subtitle_color(down: bool) -> [u8; 4] {
    if down {
        [255, 255, 255, 255]
    } else {
        [112, 112, 112, 255]
    }
}

fn item_id(index: usize) -> String {
    format!("xp-item-{index}")
}

fn item_index(node: &str) -> Option<usize> {
    node.strip_prefix("xp-item-")?.parse().ok()
}

fn at(x: i32, y: i32) -> PropsUpdate {
    PropsUpdate {
        transform: Some(Transform::translate(f64::from(x), f64::from(y), 0.0)),
        ..PropsUpdate::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Program;

    /// The default settings with kitty and no programs.
    fn config() -> Config {
        let mut config = crate::config::defaults();
        config.terminal = "kitty".into();
        config.programs.clear();
        config
    }

    #[test]
    fn the_columns_have_the_height_of_xp() {
        let mut icons = Icons::new(crate::icons::THEMES);
        let programs = programs(&config(), &mut icons, &[]);
        assert_eq!(programs[0].action, Action::Run("kitty".into()));
        let places = places(&config(), &mut icons);
        assert!(body_height(&programs, &places) < 330);
        assert_eq!(place_separators(&places, HEADER + 1).len(), 2);
        let cascades: Vec<&str> = places
            .iter()
            .filter(|e| e.cascade)
            .map(|e| e.title.as_str())
            .collect();
        assert_eq!(cascades, [CONTROL_PANEL]);
    }

    const WHITE: [u8; 3] = [255, 255, 255];
    const FIGURE: [u8; 3] = [0x3b, 0x6f, 0xd0];

    fn near(a: [u8; 3], b: [u8; 3]) -> bool {
        a.iter().zip(b).all(|(x, y)| x.abs_diff(y) < 24)
    }

    #[test]
    fn the_user_tile_is_one_square_with_an_equal_ring_and_a_centred_figure() {
        let (mut theme, cache) = crate::sheets::tests::theme("tile");
        let art = header_art(&theme, None, 57);
        let image = crate::sheets::tests::draw(&mut theme, &art);
        let at = |x, y| crate::sheets::tests::rgb(&image, x, y);
        for y in 4..50 {
            for x in TILE_X + TILE + 2..TILE_X + TILE + 14 {
                assert!(
                    near(at(x, y), at(120, y)),
                    "{x},{y} beside the tile is {:?}, the band is {:?}",
                    at(x, y),
                    at(120, y)
                );
            }
        }
        let mid = TILE_Y + TILE / 2;
        let left = (TILE_X..TILE_X + 6)
            .filter(|x| near(at(*x, mid), WHITE))
            .count();
        let right = (TILE_X + TILE - 6..TILE_X + TILE)
            .filter(|x| near(at(*x, mid), WHITE))
            .count();
        let col = TILE_X + TILE / 2;
        let top = (TILE_Y..TILE_Y + 6)
            .filter(|y| near(at(col, *y), WHITE))
            .count();
        let bottom = (TILE_Y + TILE - 6..TILE_Y + TILE)
            .filter(|y| near(at(col, *y), WHITE))
            .count();
        assert_eq!([left, right, top, bottom], [2, 2, 2, 2]);
        let ink: Vec<[i32; 2]> = (TILE_Y + 2..TILE_Y + TILE - 2)
            .flat_map(|y| (TILE_X + 2..TILE_X + TILE - 2).map(move |x| [x, y]))
            .filter(|[x, y]| near(at(*x, *y), FIGURE))
            .collect();
        let min_x = ink.iter().map(|p| p[0]).min().expect("a figure");
        let max_x = ink.iter().map(|p| p[0]).max().expect("a figure");
        let min_y = ink.iter().map(|p| p[1]).min().expect("a figure");
        assert!(
            ((min_x - TILE_X - 2) - (TILE_X + TILE - 3 - max_x)).abs() <= 1,
            "the figure is off centre: {min_x}..{max_x}"
        );
        assert!(
            min_y - TILE_Y - 2 >= 5,
            "the head is {} px from the ring",
            min_y - TILE_Y - 2
        );
        let _ = std::fs::remove_dir_all(cache);
    }

    fn with_programs(n: usize) -> Config {
        let mut config = config();
        config.programs = (0..n)
            .map(|i| Program {
                title: format!("Notes {i}"),
                command: format!("chora-notes-{i}"),
                icon: String::new(),
            })
            .collect();
        config
    }

    #[test]
    fn the_pinned_rows_have_plain_subtitles_and_the_recent_rows_none() {
        let mut icons = Icons::new(crate::icons::THEMES);
        let config = with_programs(3);
        let programs = programs(&config, &mut icons, &[]);
        assert_eq!(programs[0].title, "Terminal");
        assert_eq!(programs[1].title, "Chat with a helper");
        assert_eq!(programs[1].action, Action::Run("chora-chat".into()));
        let commands: Vec<String> = programs
            .iter()
            .filter_map(|e| match &e.action {
                Action::Run(c) => Some(c.clone()),
                _ => None,
            })
            .collect();
        for entry in &programs {
            if let Some(sub) = &entry.subtitle {
                assert!(!commands.contains(sub), "{sub} is a command");
                assert_eq!(entry.rect[3], PINNED_ROW);
            }
        }
        assert!(
            programs[2..]
                .iter()
                .all(|e| e.subtitle.is_none() && !e.bold)
        );
        assert!(programs[2..].iter().all(|e| e.rect[3] == PROGRAM_ROW));
    }

    #[test]
    fn the_menu_has_no_filler_when_the_left_column_is_the_taller() {
        let mut icons = Icons::new(crate::icons::THEMES);
        let config = with_programs(4);
        let programs = programs(&config, &mut icons, &[]);
        let places = places(&config, &mut icons);
        let top = HEADER + 1;
        let body = body_height(&programs, &places);
        let last = programs
            .iter()
            .map(|e| e.rect[1] + e.rect[3])
            .max()
            .unwrap();
        let all = top + body - 6 - ALL_PROGRAMS;
        assert!(all - last <= SEPARATOR + 2 + 12, "{} px", all - last);
    }

    fn app(title: &str) -> Node {
        Node {
            title: title.into(),
            icon: String::new(),
            kind: apps::Kind::Run(format!("run-{title}")),
        }
    }

    #[test]
    fn notepad_is_the_first_program_below_the_pinned_rows() {
        let (theme, cache) = crate::dialog::tests::theme("startmenu-notepad");
        let mut icons = Icons::new(crate::icons::THEMES);
        let apps: Vec<Node> = vec![
            Node {
                title: "Accessories".into(),
                icon: String::new(),
                kind: apps::Kind::Group(vec![app("Calc"), app(apps::NOTEPAD), app("Zip")]),
            },
            app("Loose"),
        ];
        let menu = Menu::new(&config(), apps, &theme, &mut icons);
        assert_eq!(menu.entries[2].title, "Notepad");
        assert_eq!(menu.entries[2].kind, Kind::Program);
        assert_eq!(menu.entries[2].action, Action::Run("run-Notepad".into()));
        assert_eq!(menu.entries[3].title, "Calc");
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn few_programs_are_filled_from_the_desktop_files_so_the_left_column_has_no_gap() {
        let (theme, cache) = crate::dialog::tests::theme("startmenu-fill");
        let mut icons = Icons::new(crate::icons::THEMES);
        let apps: Vec<Node> = vec![
            Node {
                title: "Accessories".into(),
                icon: String::new(),
                kind: apps::Kind::Group((0..12).map(|i| app(&format!("App {i}"))).collect()),
            },
            app("Loose"),
        ];
        let menu = Menu::new(&config(), apps, &theme, &mut icons);
        let programs: Vec<&Entry> = menu
            .entries
            .iter()
            .filter(|e| e.kind == Kind::Program)
            .collect();
        assert!(programs.len() > 2, "{} rows", programs.len());
        assert!(programs[2..].iter().all(|e| e.subtitle.is_none()));
        let last = programs
            .iter()
            .map(|e| e.rect[1] + e.rect[3])
            .max()
            .unwrap();
        let all = menu
            .entries
            .iter()
            .find(|e| e.kind == Kind::AllPrograms)
            .unwrap();
        assert!(
            all.rect[1] - last <= SEPARATOR + 2 + 12,
            "{} px",
            all.rect[1] - last
        );
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn the_first_rows_of_both_columns_stand_a_gap_under_the_header() {
        let (theme, cache) = crate::dialog::tests::theme("startmenu-gap");
        let mut icons = Icons::new(crate::icons::THEMES);
        let menu = Menu::new(&with_programs(3), Vec::new(), &theme, &mut icons);
        let header = theme.size(sheets::WINXP, "startmenu-header").unwrap()[1];
        for kind in [Kind::Program, Kind::Place] {
            let first = menu
                .entries
                .iter()
                .filter(|e| e.kind == kind)
                .map(|e| e.rect[1])
                .min()
                .unwrap();
            assert!(
                first >= header + 4,
                "{kind:?} starts at {first}, header {header}"
            );
        }
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn no_entry_of_the_menu_has_a_clipped_text_or_an_icon_of_another_size() {
        let (theme, cache) = crate::dialog::tests::theme("startmenu-audit");
        let mut icons = Icons::new(crate::icons::THEMES);
        let menu = Menu::new(&with_programs(6), Vec::new(), &theme, &mut icons);
        let mut found = Vec::new();
        for entry in &menu.entries {
            for down in [false, true] {
                let picture = picture(&theme, entry, down);
                for finding in sheets::tests::icon_size_findings(&picture)
                    .into_iter()
                    .chain(sheets::tests::clipped_text_findings(&theme, &picture))
                {
                    found.push(format!("{}: {finding}", entry.title));
                }
            }
        }
        found.sort();
        found.dedup();
        assert_eq!(found, Vec::<String>::new());
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn a_long_title_ends_inside_its_row_with_an_ellipsis() {
        let (mut theme, cache) = crate::dialog::tests::theme("startmenu-long");
        let mut icons = Icons::new(crate::icons::THEMES);
        let mut config = with_programs(3);
        config.programs[0].title = "ImageMagick (color depth and other settings)".into();
        let menu = Menu::new(&config, Vec::new(), &theme, &mut icons);
        let entry = menu
            .entries
            .iter()
            .find(|e| e.title.starts_with("ImageMagick"))
            .expect("the long program");
        let drawn = picture(&theme, entry, false);
        let image = sheets::tests::draw(&mut theme, &drawn);
        let width = image.width();
        let right = (0..image.height())
            .flat_map(|y| (width - 4..width).map(move |x| (x, y)))
            .filter(|(x, y)| image.pixel(*x, *y).is_some_and(|p| p.alpha() > 0))
            .count();
        assert_eq!(right, 0, "ink in the last 4 columns of the row");
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn the_right_column_has_help_and_support_above_run() {
        let mut icons = Icons::new(crate::icons::THEMES);
        let places = places(&config(), &mut icons);
        let titles: Vec<&str> = places.iter().map(|e| e.title.as_str()).collect();
        let help = titles
            .iter()
            .position(|t| *t == "Help and Support")
            .unwrap();
        assert_eq!(titles[help + 1], "Run...");
        assert_eq!(places[help].action, Action::Help);
    }

    #[test]
    fn the_header_shows_the_full_name_of_the_account() {
        let passwd = "root:x:0:0:root:/root:/bin/bash\nerika:x:1000:1000:Erika Muster,,,:/home/erika:/bin/sh\nplain:x:1001:1001:::/home/plain:/bin/sh\n";
        assert_eq!(full_name(passwd, "erika").as_deref(), Some("Erika Muster"));
        assert_eq!(full_name(passwd, "plain"), None);
        assert_eq!(full_name(passwd, "nobody"), None);
    }

    #[test]
    fn resting_on_the_label_of_the_control_panel_opens_its_cascade_after_400_ms() {
        let (mut theme, cache) = crate::dialog::tests::theme("rest");
        let mut icons = Icons::new(crate::icons::THEMES);
        let mut menu = Menu::new(&config(), Vec::new(), &theme, &mut icons);
        menu.control = vec![Node {
            title: "Display".into(),
            icon: String::new(),
            kind: apps::Kind::Run("true".into()),
        }];
        let index = menu
            .entries
            .iter()
            .position(|e| e.title == CONTROL_PANEL)
            .unwrap();
        let start = Instant::now();
        menu.hover(&mut theme, &item_id(index), true);
        let due = menu.due().expect("a cascade waits");
        assert!(
            due >= start + Duration::from_millis(400),
            "{:?}",
            due - start
        );
        assert_eq!(menu.rest(&mut theme, &mut icons, start).len(), 0);
        assert!(menu.cascades.is_empty());
        menu.rest(&mut theme, &mut icons, due);
        assert_eq!(menu.cascades.len(), 1);
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn items_parse_their_index() {
        assert_eq!(item_index(&item_id(7)), Some(7));
        assert_eq!(item_index("xp-menu-frame"), None);
        assert!(Menu::owns("xp-item-0") && Menu::owns("xp-menu-frame"));
    }

    fn down_to_all_programs(menu: &mut Menu, theme: &mut Theme, icons: &mut Icons) {
        while menu.entries[menu.hovered.unwrap()].kind != Kind::AllPrograms {
            menu.key(theme, icons, "Down");
        }
    }

    #[test]
    fn the_keys_walk_the_columns_and_the_cascades() {
        let (mut theme, cache) = crate::dialog::tests::theme("startkeys");
        let mut icons = Icons::new(crate::icons::THEMES);
        let run = |title: &str| Node {
            title: title.into(),
            icon: String::new(),
            kind: apps::Kind::Run(title.into()),
        };
        let apps = vec![Node {
            title: "Accessories".into(),
            icon: String::new(),
            kind: apps::Kind::Group(vec![run("calc")]),
        }];
        let mut menu = Menu::new(&config(), apps, &theme, &mut icons);
        let mut key = |menu: &mut Menu, k: &str| menu.key(&mut theme, &mut icons, k);
        key(&mut menu, "Down");
        assert_eq!(menu.entries[menu.hovered.unwrap()].title, "Terminal");
        key(&mut menu, "Down");
        assert_eq!(
            menu.entries[menu.hovered.unwrap()].title,
            "Chat with a helper"
        );
        down_to_all_programs(&mut menu, &mut theme, &mut icons);
        let mut key = |menu: &mut Menu, k: &str| menu.key(&mut theme, &mut icons, k);
        key(&mut menu, "Right");
        key(&mut menu, "Right");
        assert_eq!(menu.cascades.len(), 2);
        assert!(matches!(key(&mut menu, "Return"), Key::Act(Action::Run(c)) if c == "calc"));
        // Escape closes the cascades one by one, then the menu.
        key(&mut menu, "Escape");
        key(&mut menu, "Escape");
        assert!(menu.cascades.is_empty());
        assert!(matches!(key(&mut menu, "Escape"), Key::Close));
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn the_keys_leave_out_log_off_and_turn_off() {
        let (mut theme, cache) = crate::dialog::tests::theme("startfooter");
        let mut icons = Icons::new(crate::icons::THEMES);
        let mut menu = Menu::new(&config(), Vec::new(), &theme, &mut icons);
        for key in [
            "Down", "Down", "Down", "Right", "Down", "Down", "Left", "Up",
        ] {
            menu.key(&mut theme, &mut icons, key);
            let hovered = menu.hovered.map(|i| menu.entries[i].kind);
            assert!(hovered != Some(Kind::Footer), "{key} lit a footer button");
        }
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn art_that_is_a_fragment_is_told_from_art_that_fills_its_square() {
        let dir = std::env::temp_dir().join(format!("chora-xp-art-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("a folder");
        let draw = |name: &str, side: u32, ink: u32| {
            let mut image = resvg::tiny_skia::Pixmap::new(side, side).expect("a pixmap");
            for y in 0..ink {
                for x in 0..ink {
                    image.pixels_mut()[(y * side + x) as usize] =
                        resvg::tiny_skia::PremultipliedColorU8::from_rgba(200, 0, 0, 255)
                            .expect("a colour");
                }
            }
            image.save_png(dir.join(name)).expect("a file");
            dir.join(name)
        };
        let fragment = draw("fragment.png", 48, 12);
        let whole = draw("whole.png", 48, 44);
        assert!(is_fragment(&fragment));
        assert!(!is_fragment(&whole));
        assert!(!is_fragment(&dir.join("missing.png")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn art_of_a_white_page_with_a_few_marks_is_a_fragment() {
        let dir = std::env::temp_dir().join(format!("chora-xp-page-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("a folder");
        let draw = |name: &str, every: u32| {
            let mut image = resvg::tiny_skia::Pixmap::new(64, 64).expect("a pixmap");
            for (i, pixel) in (0u32..).zip(image.pixels_mut()) {
                let marked = i % 64 % every == 0 && i / 64 % every == 0;
                let green = if marked { 0 } else { 255 };
                *pixel = resvg::tiny_skia::PremultipliedColorU8::from_rgba(255, green, green, 255)
                    .expect("a colour");
            }
            image.save_png(dir.join(name)).expect("a file");
            dir.join(name)
        };
        assert!(is_fragment(&draw("dots.png", 8)), "dots on white");
        assert!(!is_fragment(&draw("solid.png", 1)), "a filled square");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
