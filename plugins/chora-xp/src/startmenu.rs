//! The start menu: XP's header with the user name, programs on the left,
//! places on the right, and Log Off and Turn Off Computer in the footer.
//! The geometry follows the measurements of the reference start menu
//! (`XpTheme.qml`): 382 pixels wide, a 64 pixel header, a 40 pixel footer.

use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use chora_scene::{Command, NodeKind, PropsUpdate, Transform};

use crate::{
    apps::{self, Node},
    cascade::{self, Cascade, Margins, Row},
    icons::Icons,
    text::{Font, Text},
    theme::{Picture, Rect, Sheet, Theme},
};

const WIDTH: i32 = 382;
const HEADER: i32 = 64;
const FOOTER: i32 = 40;
/// The frame around the columns.
const FRAME: i32 = 2;
const LEFT: i32 = 189;
const PROGRAM_ROW: i32 = 36;
const PLACE_ROW: i32 = 30;
const SEPARATOR: i32 = 8;
const ALL_PROGRAMS: i32 = 24;
/// XP opens and closes cascades after the pointer rests this long.
const HOVER_DELAY: Duration = Duration::from_millis(350);
const MIN_BODY: i32 = 330;
const GROUP: &str = "xp-menu";
const FRAME_ID: &str = "xp-menu-frame";

/// What an entry does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Runs a shell command.
    Run(String),
    /// Asks every window to close.
    LogOff,
    /// Ends the plugin.
    TurnOff,
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
    /// The rectangle in the menu.
    rect: Rect,
}

/// The programs and the places of the menu.
pub struct Config {
    /// The user name for the header.
    pub user: String,
    /// The programs of the left column: title, command and icon name.
    pub programs: Vec<(String, String, String)>,
    /// The command of Run, and of All Programs without programs.
    pub launcher: String,
    /// The contents of All Programs.
    pub apps: Vec<Node>,
}

/// What went down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Press {
    /// An entry of the menu.
    Entry(usize),
    /// A row of a cascade: its level and index.
    Row(usize, usize),
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
    /// The open cascades, All Programs first.
    cascades: Vec<Cascade>,
    margins: Margins,
    /// An entry or a row that the pointer rests on, and when its cascade
    /// opens or closes.
    pending: Option<(Instant, Press)>,
}

impl Menu {
    /// Lays out the menu.
    pub fn new(config: &Config, theme: &Theme, icons: &mut Icons) -> Self {
        let mut entries = programs(config, icons);
        let places = places(config, icons);
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
        separators.extend(place_separators(&places, top));
        entries.push(Entry {
            title: "All Programs".into(),
            subtitle: None,
            icon: None,
            bold: true,
            kind: Kind::AllPrograms,
            // Without desktop files, All Programs runs the launcher.
            action: Action::Run(config.launcher.clone()),
            rect: all,
        });
        entries.extend(places);
        entries.extend(footer(theme, height));
        Self {
            user: config.user.clone(),
            entries,
            separators,
            height,
            open: None,
            pressed: None,
            hovered: None,
            origin: [0, 0],
            area: [0, 0, 1280, 770],
            parent: String::new(),
            programs: config.apps.clone(),
            cascades: Vec::new(),
            margins: Margins::of(theme),
            pending: None,
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
            .size(Sheet::Winxp, "startmenu-header")
            .unwrap_or([WIDTH, 67]);
        let footer = theme
            .size(Sheet::Winxp, "startmenu-footer")
            .unwrap_or([WIDTH, 41]);
        let face = std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".face"));
        let avatar = face
            .filter(|f| f.is_file())
            .unwrap_or_else(|| theme.path("winxp/start-flag.svg"));
        let name = Text::new(
            &self.user,
            Font::Name,
            [255, 255, 255, 255],
            [67, 7, WIDTH - 77, 52],
        )
        .shadow([0, 0, 0, 100]);
        let mut picture = Picture::new(WIDTH, self.height)
            .element(
                Sheet::Winxp,
                "startmenu-body",
                [0, header[1], WIDTH, body + HEADER + 1 - header[1] + 1],
            )
            .element(Sheet::Winxp, "startmenu-header", [0, 0, WIDTH, header[1]])
            .element(
                Sheet::Winxp,
                "startmenu-footer",
                [0, self.height - footer[1], WIDTH, footer[1]],
            )
            .file(&avatar, [9, 9, 48, 48])
            .text(name);
        for rect in &self.separators {
            let id = if rect[0] < LEFT {
                "startmenu-separator"
            } else {
                "startmenu-places-separator"
            };
            let [x, y, w, h] = *rect;
            picture = picture.element(Sheet::Winxp, id, [x, y + h / 2, w, 1]);
        }
        picture
    }

    /// Whether the menu shows.
    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    /// Shows the menu above a start button; its cascades stay in the work
    /// area.
    pub fn open(&mut self, output: &str, [x, bottom]: [i32; 2], area: Rect) -> Vec<Command> {
        self.open = Some(output.into());
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

    /// Highlights the entry or the row under the pointer, as XP does, or
    /// ends the highlight when the pointer leaves it. Resting on it opens
    /// or closes cascades after a delay.
    pub fn hover(&mut self, theme: &mut Theme, node: &str, on: bool) -> Vec<Command> {
        let Some(part) = self.part(node) else {
            return Vec::new();
        };
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
                vec![
                    self.look(theme, index),
                    self.look(theme, self.all_programs()),
                ]
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
            Press::Entry(i) if i == self.all_programs() => self.open_programs(theme, icons),
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
                if index == self.all_programs() && !self.programs.is_empty() {
                    self.pending = None;
                    commands.extend(self.open_programs(theme, icons));
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
                    apps::Kind::Group(_) => (self.open_row(theme, icons, level, index), None),
                }
            }
            None => (Vec::new(), None),
        }
    }

    /// Closes the innermost cascade, as Escape does on XP. `None` when
    /// none is open: then Escape closes the menu.
    pub fn escape(&mut self, theme: &mut Theme) -> Option<Vec<Command>> {
        let innermost = self.cascades.len().checked_sub(1)?;
        Some(self.close_from(theme, innermost))
    }

    fn all_programs(&self) -> usize {
        self.entries
            .iter()
            .position(|e| e.kind == Kind::AllPrograms)
            .unwrap_or(0)
    }

    /// The entries of the cascade of a level.
    fn nodes(&self, level: usize) -> &[Node] {
        let mut nodes = &self.programs[..];
        for cascade in &self.cascades[..level] {
            match cascade.open.map(|i| &nodes[i].kind) {
                Some(apps::Kind::Group(children)) => nodes = children,
                _ => return &[],
            }
        }
        nodes
    }

    /// Opens All Programs, unless it is open.
    fn open_programs(&mut self, theme: &mut Theme, icons: &mut Icons) -> Vec<Command> {
        if !self.cascades.is_empty() || self.programs.is_empty() {
            return Vec::new();
        }
        let [x, y, w, h] = self.entries[self.all_programs()].rect;
        let button = [self.origin[0] + x, self.origin[1] + y, w, h];
        let mut commands = self.open_cascade(theme, icons, button, None);
        commands.push(self.look(theme, self.all_programs()));
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
        cascade.icons = nodes.iter().map(|n| icons.find(&n.icon)).collect();
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
            None => commands.push(self.look(theme, self.all_programs())),
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
    /// and for All Programs while its cascade is open and the pointer is
    /// on no other entry.
    fn look(&self, theme: &mut Theme, index: usize) -> Command {
        let entry = &self.entries[index];
        let lit = self.hovered == Some(index)
            || self.pressed == Some(Press::Entry(index))
            || (entry.kind == Kind::AllPrograms
                && !self.cascades.is_empty()
                && self.hovered.is_none());
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
    }
}

fn inside([x, y, w, h]: Rect, [px, py]: [f64; 2]) -> bool {
    (f64::from(x)..f64::from(x + w)).contains(&px) && (f64::from(y)..f64::from(y + h)).contains(&py)
}

/// The programs of the left column, below each other.
fn programs(config: &Config, icons: &mut Icons) -> Vec<Entry> {
    let top = HEADER + 1 + 5;
    config
        .programs
        .iter()
        .zip(0..)
        .map(|((title, command, icon), i)| Entry {
            title: title.clone(),
            subtitle: Some(command.clone()),
            icon: icons.find(icon),
            bold: true,
            kind: Kind::Program,
            action: Action::Run(command.clone()),
            rect: [FRAME + 2, top + i * PROGRAM_ROW, LEFT - 4, PROGRAM_ROW],
        })
        .collect()
}

/// The places of the right column; an empty title is a separator.
fn places(config: &Config, icons: &mut Icons) -> Vec<Entry> {
    let open = |dir: &str| format!("xdg-open \"$HOME/{dir}\"");
    let list = [
        ("My Documents", open("Documents"), "folder-documents", true),
        ("My Pictures", open("Pictures"), "folder-pictures", true),
        ("My Music", open("Music"), "folder-music", true),
        ("My Computer", "xdg-open /".into(), "computer", true),
        ("", String::new(), "", false),
        ("Run...", config.launcher.clone(), "system-run", false),
    ];
    let mut y = HEADER + 1 + 5;
    let mut entries = Vec::new();
    for (title, command, icon, bold) in list {
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
            action: Action::Run(command),
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

/// The height of the columns: the higher of both, at least 330 pixels.
fn body_height(programs: &[Entry], places: &[Entry]) -> i32 {
    let bottom = |entries: &[Entry]| {
        entries
            .iter()
            .map(|e| e.rect[1] + e.rect[3])
            .max()
            .unwrap_or(0)
            - HEADER
            - 1
    };
    // The left column ends with a separator and All Programs.
    let left = bottom(programs) + SEPARATOR + 2 + ALL_PROGRAMS + 6;
    let right = bottom(places) + 6;
    left.max(right).max(MIN_BODY)
}

/// Log Off and Turn Off Computer at the right of the footer.
fn footer(theme: &Theme, height: i32) -> Vec<Entry> {
    let y = height - FOOTER + 5;
    let button = |title: &str, icon: &str, action, x, w| Entry {
        title: title.into(),
        subtitle: None,
        icon: Some(theme.path(icon)),
        bold: false,
        kind: Kind::Footer,
        action,
        rect: [x, y, w, 30],
    };
    vec![
        button(
            "Log Off",
            "winxp/log-off.svg",
            Action::LogOff,
            WIDTH - 5 - 140 - 8 - 80,
            80,
        ),
        button(
            "Turn Off Computer",
            "winxp/turn-off.svg",
            Action::TurnOff,
            WIDTH - 5 - 140,
            140,
        ),
    ]
}

/// The picture of an entry, normal or highlighted.
fn picture(theme: &Theme, entry: &Entry, down: bool) -> Picture {
    let [_, _, w, h] = entry.rect;
    let mut picture = Picture::new(w, h);
    if down {
        picture = match entry.kind {
            Kind::Footer => picture.frame(Sheet::Winxp, "startmenu-footer-pressed", [0, 0, w, h]),
            _ => picture.frame(Sheet::Winxp, "menu-highlight", [0, 0, w, h]),
        };
    }
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
    let width = w - x - 4;
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

/// "All Programs" and XP's green arrow, centered together, 18 pixels
/// apart.
fn all_programs(theme: &Theme, picture: Picture, entry: &Entry, down: bool) -> Picture {
    let [_, _, w, h] = entry.rect;
    let arrow = theme
        .size(Sheet::Winxp, "startmenu-allprograms-arrow")
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
            Sheet::Winxp,
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

    fn config() -> Config {
        Config {
            user: "tester".into(),
            programs: vec![(
                "Terminal".into(),
                "foot".into(),
                "utilities-terminal".into(),
            )],
            launcher: "fuzzel".into(),
            apps: Vec::new(),
        }
    }

    #[test]
    fn the_columns_have_the_height_of_xp() {
        let mut icons = Icons::default();
        let programs = programs(&config(), &mut icons);
        let places = places(&config(), &mut icons);
        assert_eq!(body_height(&programs, &places), MIN_BODY);
        assert_eq!(place_separators(&places, HEADER + 1).len(), 1);
    }

    #[test]
    fn items_parse_their_index() {
        assert_eq!(item_index(&item_id(7)), Some(7));
        assert_eq!(item_index("xp-menu-frame"), None);
        assert!(Menu::owns("xp-item-0") && Menu::owns("xp-menu-frame"));
    }
}
