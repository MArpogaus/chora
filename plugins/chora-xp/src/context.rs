//! XP's context menus: a white menu with a gray border and a soft shadow,
//! rows of 17 pixels that turn white on XP's selection blue under the
//! pointer, gray rows that do nothing, check marks, separators and one
//! level of submenus. The right button opens them on the taskbar, the
//! clock and the desktop; a click outside closes them. The month list of
//! Date and Time is one, too.

use std::fmt::Write as _;

use chora_scene::{Command, InputMode, NodeKind, PropsUpdate, Transform};

use crate::{
    text::{Font, Text},
    theme::{Picture, Rect, Sheet, Theme},
};

/// The height of a row.
const ROW: i32 = 17;
/// The height of a separator.
const SEPARATOR: i32 = 9;
/// The border and the padding around the rows.
const PAD: i32 = 3;
/// The soft shadow at the right and the bottom.
const SHADOW: i32 = 4;
/// The text starts after the column of the check marks.
const TEXT_X: i32 = 17;
/// The room right of the text, for the arrow of a submenu.
const TEXT_END: i32 = 24;
const PREFIX: &str = "xp-ctx-";
/// The node that takes the keyboard while a menu is open: its root.
pub const FOCUS: &str = "xp-ctx-0";
/// The room between the text and the shortcut of a row.
const KEY_GAP: f64 = 20.0;
/// The room of the sample of a color before the text.
const SWATCH: i32 = 28;

/// What a row does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    /// Cascade Windows.
    Cascade,
    /// Tile Windows Horizontally: one above the other.
    TileRows,
    /// Tile Windows Vertically: side by side.
    TileColumns,
    /// Show the Desktop: minimizes every window.
    ShowDesktop,
    /// Lock the Taskbar.
    Lock,
    /// Properties of the taskbar.
    TaskbarProperties,
    /// Adjust Date/Time.
    DateTime,
    /// Arrange Icons By Name.
    ArrangeByName,
    /// Arrange Icons By Type.
    ArrangeByType,
    /// Refresh of the desktop.
    Refresh,
    /// Properties of the desktop.
    DesktopProperties,
    /// A month of the month list of Date and Time, from 1.
    Month(u32),
    /// A position of the wallpaper in Display Properties, by index.
    Position(usize),
    /// A color of the desktop in Display Properties.
    Color([u8; 3]),
    /// A row of a menu of the Explorer window that has the focus.
    Explore(crate::explorer::Cmd),
    /// A row of the system menu of a window.
    Window(Sys),
    /// A row of the menu of a desktop icon or a start menu entry.
    Item(Item),
}

/// What a row of the menu of a desktop icon or of an entry of the start
/// menu does to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    /// Open, as a double click does.
    Open,
    /// Delete: moves the file to the Recycle Bin.
    Delete,
    /// Empty Recycle Bin.
    EmptyTrash,
    /// Remove from This List: the program leaves the start menu.
    Remove,
    /// Properties.
    Properties,
}

/// What an icon or an entry is, for its menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// A folder of the Desktop folder.
    Folder,
    /// A file or a desktop file of the Desktop folder.
    File,
    /// My Documents.
    Documents,
    /// My Computer.
    Computer,
    /// The Recycle Bin, empty or not.
    Trash(bool),
    /// A program or a place of the start menu; a program of its left
    /// column can leave it.
    Program(bool),
}

/// What a row of XP's system menu does to its window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sys {
    /// Restore: out of the maximized or the minimized state.
    Restore,
    /// Minimize.
    Minimize,
    /// Maximize.
    Maximize,
    /// Close.
    Close,
}

impl Sys {
    /// XP's glyph of the row, from Marlett, in a 9 by 9 box.
    fn glyph(self) -> &'static str {
        match self {
            Self::Restore => "M2 0h7v6H7V5h1V2H3v1H2zM0 3h7v6H0zM1 5h5v3H1z",
            Self::Minimize => "M1 7h6v2H1z",
            Self::Maximize => "M0 0h9v9H0zM1 2h7v6H1z",
            Self::Close => "M1 1l7 7M8 1L1 8",
        }
    }
}

/// A row of a menu.
#[allow(clippy::struct_excessive_bools)] // independent states of a row
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    title: String,
    act: Option<Act>,
    enabled: bool,
    checked: bool,
    /// A row with an arrow, which opens its submenu.
    group: bool,
    /// The default row, in bold.
    bold: bool,
    /// The shortcut at the right edge, such as `Alt+F4`.
    key: &'static str,
    /// The glyph left of a row of the system menu.
    glyph: Option<Sys>,
    /// A sample of a color before the text, as in a palette.
    swatch: Option<[u8; 3]>,
    sub: Vec<Entry>,
}

impl Entry {
    /// A row that does something.
    pub fn item(title: &str, act: Act) -> Self {
        Self {
            title: title.into(),
            act: Some(act),
            enabled: true,
            checked: false,
            group: false,
            bold: false,
            key: "",
            glyph: None,
            swatch: None,
            sub: Vec::new(),
        }
    }

    /// A gray row that does nothing, as XP shows the ones chora lacks.
    pub fn gray(title: &str) -> Self {
        Self {
            enabled: false,
            act: None,
            ..Self::item(title, Act::Refresh)
        }
    }

    /// A row that opens a submenu.
    pub fn sub(title: &str, sub: Vec<Self>) -> Self {
        Self {
            act: None,
            group: true,
            sub,
            ..Self::item(title, Act::Refresh)
        }
    }

    /// A separator line.
    pub fn separator() -> Self {
        Self {
            enabled: false,
            act: None,
            ..Self::item("", Act::Refresh)
        }
    }

    /// With a check mark, or without.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Enabled or gray.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// In bold, as the default row.
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// With a sample of a color before the text.
    pub fn swatch(mut self, color: [u8; 3]) -> Self {
        self.swatch = Some(color);
        self
    }

    fn font(&self) -> Font {
        if self.bold { Font::UiBold } else { Font::Ui }
    }

    /// The width of the text and of the shortcut with its gap.
    fn width(&self, theme: &Theme) -> f64 {
        let key = if self.key.is_empty() {
            0.0
        } else {
            theme.text_width(self.key, Font::Ui) + KEY_GAP
        };
        let swatch = if self.swatch.is_some() {
            f64::from(SWATCH)
        } else {
            0.0
        };
        theme.text_width(&self.title, self.font()) + key + swatch
    }

    /// The place of the access key in the title, XP's underlined letter.
    fn access(&self) -> Option<usize> {
        ACCESS
            .iter()
            .find(|(title, _)| *title == self.title)
            .map(|(_, at)| *at)
    }

    /// Whether a typed letter is the access key of the row.
    fn has_access(&self, letter: &str) -> bool {
        self.access()
            .and_then(|at| self.title.get(at..=at))
            .is_some_and(|key| key.eq_ignore_ascii_case(letter))
    }

    fn is_separator(&self) -> bool {
        self.title.is_empty()
    }

    fn height(&self) -> i32 {
        if self.is_separator() { SEPARATOR } else { ROW }
    }
}

/// The rows of the taskbar menu.
pub fn taskbar(locked: bool) -> Vec<Entry> {
    vec![
        Entry::sub("Toolbars", Vec::new()).enabled(false),
        Entry::separator(),
        Entry::item("Cascade Windows", Act::Cascade),
        Entry::item("Tile Windows Horizontally", Act::TileRows),
        Entry::item("Tile Windows Vertically", Act::TileColumns),
        Entry::item("Show the Desktop", Act::ShowDesktop),
        Entry::separator(),
        Entry::gray("Task Manager"),
        Entry::separator(),
        Entry::item("Lock the Taskbar", Act::Lock).checked(locked),
        Entry::item("Properties", Act::TaskbarProperties),
    ]
}

/// The rows of the clock menu: Adjust Date/Time and those of the
/// taskbar.
pub fn clock(locked: bool) -> Vec<Entry> {
    let mut entries = vec![
        Entry::item("Adjust Date/Time", Act::DateTime),
        Entry::gray("Customize Notifications..."),
        Entry::separator(),
    ];
    entries.extend(taskbar(locked));
    entries
}

/// The rows of the desktop menu.
pub fn desktop() -> Vec<Entry> {
    vec![
        Entry::sub(
            "Arrange Icons By",
            vec![
                Entry::item("Name", Act::ArrangeByName),
                Entry::item("Type", Act::ArrangeByType),
            ],
        ),
        Entry::item("Refresh", Act::Refresh),
        Entry::separator(),
        Entry::gray("Paste"),
        Entry::gray("Paste Shortcut"),
        Entry::separator(),
        Entry::item("Properties", Act::DesktopProperties),
    ]
}

/// XP's menu of a desktop icon or of an entry of the start menu, by what
/// it is. Open is the default row; what `chora-xp` cannot do is gray.
pub fn item(target: Target) -> Vec<Entry> {
    let row = |title: &str, item: Item| Entry::item(title, Act::Item(item));
    let open = row("Open", Item::Open).bold();
    let send_to = Entry::sub("Send To", Vec::new()).enabled(false);
    let properties = row("Properties", Item::Properties);
    let sep = Entry::separator;
    let gray = Entry::gray;
    match target {
        Target::Folder | Target::File | Target::Documents => {
            file_rows(target, open, send_to, properties)
        }
        Target::Computer => vec![
            open,
            gray("Search..."),
            gray("Manage"),
            sep(),
            gray("Map Network Drive..."),
            gray("Disconnect Network Drive..."),
            sep(),
            gray("Create Shortcut"),
            gray("Delete"),
            gray("Rename"),
            sep(),
            gray("Properties"),
        ],
        Target::Trash(full) => vec![
            open,
            row("Empty Recycle Bin", Item::EmptyTrash).enabled(full),
            sep(),
            gray("Create Shortcut"),
            sep(),
            properties,
        ],
        Target::Program(removable) => vec![
            open,
            gray("Run as..."),
            gray("Pin to Start menu"),
            sep(),
            send_to,
            sep(),
            row("Remove from This List", Item::Remove).enabled(removable),
            gray("Rename"),
            sep(),
            properties,
        ],
    }
}

/// The rows of a folder, a file or My Documents.
fn file_rows(target: Target, open: Entry, send_to: Entry, properties: Entry) -> Vec<Entry> {
    let (sep, gray) = (Entry::separator, Entry::gray);
    let mut rows = vec![open];
    if target == Target::File {
        rows.push(Entry::sub("Open With", Vec::new()).enabled(false));
    } else {
        rows.push(gray("Search..."));
    }
    rows.extend([sep(), send_to, sep(), gray("Cut"), gray("Copy"), sep()]);
    rows.extend([
        gray("Create Shortcut"),
        Entry::item("Delete", Act::Item(Item::Delete)).enabled(target != Target::Documents),
        gray("Rename"),
        sep(),
        properties,
    ]);
    rows
}

/// XP's system menu of a window, as the title bar and the task button
/// open it. Move and Size are gray, as the keyboard does not work them
/// (D45); Close is the default row.
pub fn system(maximized: bool, minimized: bool) -> Vec<Entry> {
    let row = |title: &str, sys: Sys, enabled: bool| Entry {
        act: enabled.then_some(Act::Window(sys)),
        glyph: Some(sys),
        ..Entry::item(title, Act::Window(sys)).enabled(enabled)
    };
    vec![
        row("Restore", Sys::Restore, maximized || minimized),
        Entry::gray("Move"),
        Entry::gray("Size"),
        row("Minimize", Sys::Minimize, !minimized),
        row("Maximize", Sys::Maximize, !maximized),
        Entry::separator(),
        Entry {
            bold: true,
            key: "Alt+F4",
            ..row("Close", Sys::Close, true)
        },
    ]
}

/// One menu: the root or the submenu.
struct Level {
    entries: Vec<Entry>,
    /// In scene coordinates, with the shadow.
    rect: Rect,
    hovered: Option<usize>,
    /// The row whose submenu is open.
    open: Option<usize>,
}

impl Level {
    fn new(entries: Vec<Entry>, theme: &Theme, min_width: i32) -> Self {
        let widest = entries.iter().map(|e| e.width(theme)).fold(0.0, f64::max);
        #[allow(clippy::cast_possible_truncation)]
        let w = (widest.ceil() as i32 + TEXT_X + TEXT_END + 2 * PAD).max(min_width) + SHADOW;
        let h = entries.iter().map(Entry::height).sum::<i32>() + 2 * PAD + SHADOW;
        Self {
            entries,
            rect: [0, 0, w, h],
            hovered: None,
            open: None,
        }
    }

    /// A row in the coordinates of the menu.
    fn row(&self, index: usize) -> Rect {
        let y = PAD + self.entries[..index].iter().map(Entry::height).sum::<i32>();
        [
            PAD,
            y,
            self.rect[2] - SHADOW - 2 * PAD,
            self.entries[index].height(),
        ]
    }

    /// The next enabled row after a row, or before it with a negative
    /// step, around the ends; from none, the first or the last one.
    fn next(&self, from: Option<usize>, step: i32) -> Option<usize> {
        let n = self.entries.len();
        let start = from.unwrap_or(if step > 0 { n - 1 } else { 0 });
        std::iter::successors(Some(start), |&i| Some(wrap(i, step > 0, n)))
            .skip(1)
            .take(n)
            .find(|&i| self.entries[i].enabled)
    }

    fn lit(&self, index: usize) -> bool {
        self.entries[index].enabled
            && (self.hovered == Some(index) || (self.open == Some(index) && self.hovered.is_none()))
    }
}

/// The underlined access key of a row whose text starts at `x`.
#[allow(clippy::many_single_char_names)] // a place and a color
fn underline(
    picture: Picture,
    theme: &Theme,
    entry: &Entry,
    [x, h]: [i32; 2],
    [r, g, b]: [u8; 3],
) -> Picture {
    let Some(at) = entry.access() else {
        return picture;
    };
    let [from, to] = [at, at + 1].map(|n| {
        entry
            .title
            .get(..n)
            .map_or(0.0, |t| theme.text_width(t, entry.font()).round())
    });
    picture.svg(format!(
        r#"<rect x="{}" y="{}" width="{}" height="1" fill="rgb({r},{g},{b})"/>"#,
        f64::from(x) + from,
        h / 2 + 6,
        (to - from).max(1.0),
    ))
}

/// The access keys of XP's menus: the title of a row and the place of
/// its underlined letter.
const ACCESS: &[(&str, usize)] = &[
    ("About Windows", 0),
    ("Add to Favorites...", 0),
    ("Adjust Date/Time", 0),
    ("Arrange Icons By", 8),
    ("Back", 0),
    ("Cascade Windows", 2),
    ("Close", 0),
    ("Copy", 0),
    ("Create Shortcut", 7),
    ("Customize Notifications...", 0),
    ("Cut", 2),
    ("Delete", 0),
    ("Details", 0),
    ("Disconnect Network Drive...", 0),
    ("Empty Recycle Bin", 14),
    ("Explore", 0),
    ("Explorer Bar", 0),
    ("Folder Options...", 7),
    ("Forward", 0),
    ("Go To", 0),
    ("Help and Support Center", 0),
    ("Icons", 3),
    ("Invert Selection", 0),
    ("List", 0),
    ("Lock the Taskbar", 0),
    ("Map Network Drive...", 0),
    ("Maximize", 2),
    ("Minimize", 2),
    ("Move", 0),
    ("Name", 0),
    ("New", 2),
    ("Open", 0),
    ("Open With", 7),
    ("Organize Favorites...", 0),
    ("Paste", 0),
    ("Paste Shortcut", 6),
    ("Properties", 1),
    ("Refresh", 1),
    ("Remove from This List", 17),
    ("Rename", 4),
    ("Restore", 0),
    ("Select All", 7),
    ("Send To", 2),
    ("Show the Desktop", 0),
    ("Size", 0),
    ("Status Bar", 7),
    ("Synchronize...", 0),
    ("Task Manager", 3),
    ("Thumbnails", 1),
    ("Tile Windows Horizontally", 13),
    ("Tile Windows Vertically", 14),
    ("Tiles", 4),
    ("Toolbars", 0),
    ("Type", 0),
    ("Undo", 0),
    ("Up One Level", 0),
];

/// What a key does in a menu, a context menu or the start menu.
#[derive(Debug, PartialEq)]
pub enum Key<A> {
    /// The menu changes its look.
    Commands(Vec<Command>),
    /// A row acts; the menu closes.
    Act(A),
    /// The menu closes.
    Close,
}

/// The place one step down or, with `forward` false, up from a place in
/// a list of `n`, around the ends.
pub fn wrap(at: usize, forward: bool, n: usize) -> usize {
    if forward {
        (at + 1) % n
    } else {
        (at + n - 1) % n
    }
}

/// The open menu, with its submenu.
pub struct Menu {
    levels: Vec<Level>,
    area: Rect,
    /// The row that went down.
    down: Option<(usize, usize)>,
    /// The keyboard works the menu: its access keys show underlined, as
    /// XP shows them only then.
    cues: bool,
}

impl Menu {
    /// A menu at a point of the scene, inside an area. `min_width` makes
    /// a list as wide as its field.
    pub fn new(
        theme: &Theme,
        entries: Vec<Entry>,
        point: [i32; 2],
        area: Rect,
        min_width: i32,
    ) -> Self {
        let mut level = Level::new(entries, theme, min_width);
        let [x, y] = place([level.rect[2], level.rect[3]], point, area);
        level.rect[0] = x;
        level.rect[1] = y;
        Self {
            levels: vec![level],
            area,
            down: None,
            cues: false,
        }
    }

    /// A drop-down list as wide as its field: it opens on its current
    /// value, the checked row, as on XP.
    pub fn list(
        theme: &Theme,
        entries: Vec<Entry>,
        point: [i32; 2],
        area: Rect,
        width: i32,
    ) -> Self {
        let mut list = Self::new(theme, entries, point, area, width);
        let root = &mut list.levels[0];
        root.hovered = root.entries.iter().position(|e| e.checked);
        list
    }

    /// Whether a node belongs to a context menu.
    pub fn owns(node: &str) -> bool {
        node.starts_with(PREFIX)
    }

    /// The nodes of the menu under a parent.
    pub fn create(&self, theme: &mut Theme, parent: &str) -> Vec<Command> {
        self.create_level(theme, parent, 0)
    }

    fn create_level(&self, theme: &mut Theme, parent: &str, level: usize) -> Vec<Command> {
        let menu = &self.levels[level];
        let [x, y, w, h] = menu.rect;
        let group = group_id(level);
        let mut commands = vec![
            Command::Create {
                id: group.clone(),
                kind: NodeKind::Group,
                parent: parent.into(),
                props: at([x, y]),
            },
            Command::Create {
                id: format!("{group}-frame"),
                kind: NodeKind::Image,
                parent: group.clone(),
                props: PropsUpdate {
                    image: Some(theme.render(&frame(w, h))),
                    input: Some(InputMode::Accept),
                    ..PropsUpdate::default()
                },
            },
        ];
        for i in 0..menu.entries.len() {
            let [rx, ry, ..] = menu.row(i);
            commands.push(Command::Create {
                id: row_id(level, i),
                kind: NodeKind::Image,
                parent: group.clone(),
                props: PropsUpdate {
                    image: Some(self.render(theme, level, i)),
                    input: Some(InputMode::Accept),
                    ..at([rx, ry])
                },
            });
        }
        commands
    }

    fn render(&self, theme: &mut Theme, level: usize, index: usize) -> String {
        let menu = &self.levels[level];
        let entry = &menu.entries[index];
        let [_, _, w, h] = menu.row(index);
        let picture = row(theme, entry, [w, h], menu.lit(index), self.cues);
        theme.render(&picture)
    }

    fn relook(&self, theme: &mut Theme, level: usize, index: usize) -> Command {
        Command::Update {
            id: row_id(level, index),
            props: PropsUpdate {
                image: Some(self.render(theme, level, index)),
                ..PropsUpdate::default()
            },
        }
    }

    /// Removes the menu.
    pub fn remove(&self) -> Vec<Command> {
        (0..self.levels.len())
            .map(|level| Command::Remove {
                id: group_id(level),
                to: None,
            })
            .collect()
    }

    /// The pointer came onto a row or left it: the row lights up, and a
    /// row with a submenu opens it.
    pub fn hover(&mut self, theme: &mut Theme, parent: &str, node: &str, on: bool) -> Vec<Command> {
        let Some((level, index)) = self.part(node) else {
            return Vec::new();
        };
        let mut commands = Vec::new();
        let old = self.levels[level].hovered;
        if on {
            self.levels[level].hovered = Some(index);
        } else if old == Some(index) {
            self.levels[level].hovered = None;
        }
        if on && level == 0 && self.levels[0].open != Some(index) {
            commands.extend(self.close_sub(theme));
            let entry = &self.levels[0].entries[index];
            if entry.enabled && !entry.sub.is_empty() {
                commands.extend(self.open_sub(theme, parent, index));
            }
        }
        for i in [old, Some(index)].into_iter().flatten() {
            if i < self.levels[level].entries.len() {
                commands.push(self.relook(theme, level, i));
            }
        }
        commands
    }

    fn open_sub(&mut self, theme: &mut Theme, parent: &str, index: usize) -> Vec<Command> {
        let root = &self.levels[0];
        let [rx, ry, rw, _] = root.rect;
        let [_, row_y, ..] = root.row(index);
        let mut sub = Level::new(root.entries[index].sub.clone(), theme, 0);
        let [w, h] = [sub.rect[2], sub.rect[3]];
        let [ax, ay, aw, ah] = self.area;
        // Right of the menu, the first row level with the row; left of
        // it, or higher, where there is no room.
        let mut x = rx + rw - SHADOW - PAD;
        if x + w > ax + aw {
            x = rx - w + SHADOW + PAD;
        }
        let y = (ry + row_y - PAD).min(ay + ah - h).max(ay);
        sub.rect[0] = x.max(ax);
        sub.rect[1] = y;
        self.levels[0].open = Some(index);
        self.levels.push(sub);
        self.create_level(theme, parent, 1)
    }

    fn close_sub(&mut self, theme: &mut Theme) -> Vec<Command> {
        if self.levels.len() < 2 {
            return Vec::new();
        }
        self.levels.truncate(1);
        if matches!(self.down, Some((1, _))) {
            self.down = None;
        }
        let mut commands = vec![Command::Remove {
            id: group_id(1),
            to: None,
        }];
        if let Some(open) = self.levels[0].open.take() {
            commands.push(self.relook(theme, 0, open));
        }
        commands
    }

    /// A key while the menu is open, as on XP: Up and Down move the
    /// highlight over the enabled rows, Right and Left open and close the
    /// submenu, Return acts, and Escape closes the submenu or the menu.
    pub fn key(&mut self, theme: &mut Theme, parent: &str, key: &str) -> Key<Act> {
        let mut commands = self.show_cues(theme);
        match self.step(theme, parent, key) {
            Key::Commands(more) => {
                commands.extend(more);
                Key::Commands(commands)
            }
            other => other,
        }
    }

    /// The access keys show from the first key on: all rows again.
    pub fn show_cues(&mut self, theme: &mut Theme) -> Vec<Command> {
        if std::mem::replace(&mut self.cues, true) {
            return Vec::new();
        }
        (0..self.levels.len())
            .flat_map(|level| (0..self.levels[level].entries.len()).map(move |i| (level, i)))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|(level, i)| self.relook(theme, level, i))
            .collect()
    }

    fn step(&mut self, theme: &mut Theme, parent: &str, key: &str) -> Key<Act> {
        let level = self.levels.len() - 1;
        let current = self.levels[level].hovered;
        match key {
            "Down" | "Up" => {
                let step = if key == "Down" { 1 } else { -1 };
                let Some(next) = self.levels[level].next(current, step) else {
                    return Key::Commands(Vec::new());
                };
                Key::Commands(self.light(theme, level, Some(next)))
            }
            "Home" | "End" => {
                let last = self.levels[level].entries.len().saturating_sub(1);
                let (from, step) = if key == "Home" { (last, 1) } else { (0, -1) };
                let row = self.levels[level].next(Some(from), step);
                Key::Commands(self.light(theme, level, row))
            }
            "Right" if level == 0 => match current {
                Some(i)
                    if self.levels[0].entries[i].enabled
                        && !self.levels[0].entries[i].sub.is_empty() =>
                {
                    let mut commands = self.open_sub(theme, parent, i);
                    let first = self.levels[1].next(None, 1);
                    commands.extend(self.light(theme, 1, first));
                    Key::Commands(commands)
                }
                _ => Key::Commands(Vec::new()),
            },
            "Left" | "Escape" if level == 1 => {
                let open = self.levels[0].open;
                let mut commands = self.close_sub(theme);
                commands.extend(self.light(theme, 0, open));
                Key::Commands(commands)
            }
            "Escape" => Key::Close,
            letter if letter.chars().count() == 1 => self.access_key(theme, parent, letter),
            "Return" | "KP_Enter" | "space" => {
                let Some(i) = current else {
                    return Key::Commands(Vec::new());
                };
                let entry = &self.levels[level].entries[i];
                if level == 0 && entry.enabled && !entry.sub.is_empty() {
                    return self.step(theme, parent, "Right");
                }
                match entry.act.filter(|_| entry.enabled) {
                    Some(act) => Key::Act(act),
                    None => Key::Commands(Vec::new()),
                }
            }
            _ => Key::Commands(Vec::new()),
        }
    }

    /// Whether Left or Right would go beside the menu instead of into or
    /// out of a submenu: no submenu is open, and for Right the lit row
    /// opens none.
    pub fn goes_beside(&self, right: bool) -> bool {
        let level = &self.levels[self.levels.len() - 1];
        let opens = level
            .hovered
            .is_some_and(|i| level.entries[i].enabled && !level.entries[i].sub.is_empty());
        if right {
            !opens
        } else {
            self.levels.len() == 1
        }
    }

    /// A letter: the one enabled row with it as its access key acts, as
    /// Return would; with several, the highlight goes to the next one.
    fn access_key(&mut self, theme: &mut Theme, parent: &str, letter: &str) -> Key<Act> {
        let level = self.levels.len() - 1;
        let menu = &self.levels[level];
        let rows: Vec<usize> = (0..menu.entries.len())
            .filter(|&i| menu.entries[i].enabled && menu.entries[i].has_access(letter))
            .collect();
        match rows[..] {
            [] => Key::Commands(Vec::new()),
            [only] => {
                let mut commands = self.light(theme, level, Some(only));
                match self.step(theme, parent, "Return") {
                    Key::Commands(more) => {
                        commands.extend(more);
                        Key::Commands(commands)
                    }
                    other => other,
                }
            }
            _ => {
                let next = rows
                    .iter()
                    .copied()
                    .find(|&i| Some(i) > menu.hovered)
                    .unwrap_or(rows[0]);
                Key::Commands(self.light(theme, level, Some(next)))
            }
        }
    }

    /// Moves the highlight of a level to a row, or takes it away.
    fn light(&mut self, theme: &mut Theme, level: usize, row: Option<usize>) -> Vec<Command> {
        let old = std::mem::replace(&mut self.levels[level].hovered, row);
        [old, row]
            .into_iter()
            .flatten()
            .map(|i| self.relook(theme, level, i))
            .collect()
    }

    /// A press on a node of the menu.
    pub fn press(&mut self, node: &str) {
        self.down = self.part(node);
    }

    /// The release of the button: the act of the row that went down, if
    /// the pointer is still on it and it is enabled.
    pub fn release(&mut self, [px, py]: [f64; 2]) -> Option<Act> {
        let (level, index) = self.down.take()?;
        let menu = self.levels.get(level)?;
        let [x, y, w, h] = menu.row(index);
        let [x, y] = [x + menu.rect[0], y + menu.rect[1]];
        let inside = (f64::from(x)..f64::from(x + w)).contains(&px)
            && (f64::from(y)..f64::from(y + h)).contains(&py);
        let entry = &menu.entries[index];
        if inside && entry.enabled {
            entry.act
        } else {
            None
        }
    }

    /// The level and the index of a row node.
    fn part(&self, node: &str) -> Option<(usize, usize)> {
        let (level, index) = node.strip_prefix(PREFIX)?.split_once('-')?;
        let (level, index): (usize, usize) = (level.parse().ok()?, index.parse().ok()?);
        (index < self.levels.get(level)?.entries.len()).then_some((level, index))
    }
}

/// The top-left corner of a menu of a size that opens at a point: right
/// of and below it, or left of or above it where there is no room, and
/// always inside the area.
pub fn place([w, h]: [i32; 2], [px, py]: [i32; 2], [ax, ay, aw, ah]: Rect) -> [i32; 2] {
    let x = if px + w > ax + aw { px - w } else { px };
    let y = if py + h > ay + ah { py - h } else { py };
    [x.min(ax + aw - w).max(ax), y.min(ay + ah - h).max(ay)]
}

/// The white menu with its gray border and the soft shadow.
fn frame(w: i32, h: i32) -> Picture {
    let [mw, mh] = [w - SHADOW, h - SHADOW];
    let mut svg = String::new();
    // The shadow: four steps that fade out.
    for i in 0..SHADOW {
        let alpha = 0.18 - 0.04 * f64::from(i);
        let _ = write!(
            svg,
            r##"<rect x="{}" y="{}" width="{mw}" height="{mh}" rx="{}" fill="#000" fill-opacity="{alpha:.2}"/>"##,
            SHADOW - i,
            SHADOW - i,
            f64::from(i) / 2.0
        );
    }
    let _ = write!(
        svg,
        r##"<rect x="0.5" y="0.5" width="{}" height="{}" fill="#fff" stroke="#ACA899"/>"##,
        mw - 1,
        mh - 1
    );
    Picture::new(w, h).svg(svg)
}

/// A row: the highlight, the check mark, the text and the arrow of a
/// submenu, or a separator.
#[allow(clippy::many_single_char_names)] // colors and coordinates
fn row(theme: &Theme, entry: &Entry, [w, h]: [i32; 2], lit: bool, cues: bool) -> Picture {
    let mut picture = Picture::new(w, h);
    if entry.is_separator() {
        return picture.svg(format!(
            r##"<rect x="1" y="{}" width="{}" height="1" fill="#ACA899"/>"##,
            h / 2,
            w - 2
        ));
    }
    if lit {
        picture = picture.frame(Sheet::Winxp, "menu-highlight", [0, 0, w, h]);
    }
    if entry.checked {
        let check = if lit {
            "winxp/check-selected.svg"
        } else {
            "winxp/check.svg"
        };
        picture = picture.file(&theme.path(check), [5, (h - 8) / 2, 8, 8]);
    }
    let color = match (entry.enabled, lit) {
        (false, _) => [172, 168, 153, 255],
        (true, true) => [255, 255, 255, 255],
        (true, false) => [0, 0, 0, 255],
    };
    let [r, g, b, _] = color;
    let mut x = TEXT_X;
    if let Some([sr, sg, sb]) = entry.swatch {
        picture = picture.svg(format!(
            r##"<rect x="{}" y="3.5" width="21" height="{}" fill="rgb({sr},{sg},{sb})" stroke="#000"/>"##,
            f64::from(x) + 0.5,
            h - 7
        ));
        x += SWATCH;
    }
    if cues {
        picture = underline(picture, theme, entry, [x, h], [r, g, b]);
    }
    picture = system_parts(picture, theme, entry, [w, h], color).text(Text::new(
        &entry.title,
        entry.font(),
        color,
        [x, 0, w - x - TEXT_END + 4, h],
    ));
    if entry.group {
        // XP's arrow of a submenu, 4 by 7 pixels, in the color of the text.
        let y = f64::from(h) / 2.0;
        picture = picture.svg(format!(
            r#"<path d="M{} {}l4 3.5l-4 3.5z" fill="rgb({r},{g},{b})"/>"#,
            w - 12,
            y - 3.5
        ));
    }
    picture
}

/// The glyph and the shortcut of a row of the system menu.
#[allow(clippy::many_single_char_names)] // colors and coordinates
fn system_parts(
    mut picture: Picture,
    theme: &Theme,
    entry: &Entry,
    [w, h]: [i32; 2],
    [r, g, b, _]: [u8; 4],
) -> Picture {
    if let Some(sys) = entry.glyph {
        let color = format!("rgb({r},{g},{b})");
        // The cross of Close is a line, the others are areas.
        let (fill, stroke) = if sys == Sys::Close {
            ("none", color.as_str())
        } else {
            (color.as_str(), "none")
        };
        picture = picture.svg(format!(
            r#"<path transform="translate(4 {})" d="{}" fill="{fill}" fill-rule="evenodd" stroke="{stroke}" stroke-width="1.5"/>"#,
            (h - 9) / 2,
            sys.glyph()
        ));
    }
    if !entry.key.is_empty() {
        #[allow(clippy::cast_possible_truncation)]
        let kw = theme.text_width(entry.key, Font::Ui).ceil() as i32;
        picture = picture.text(Text::new(
            entry.key,
            Font::Ui,
            [r, g, b, 255],
            [w - TEXT_END + 4 - kw, 0, kw + 2, h],
        ));
    }
    picture
}

fn group_id(level: usize) -> String {
    format!("{PREFIX}{level}")
}

fn row_id(level: usize, index: usize) -> String {
    format!("{PREFIX}{level}-{index}")
}

fn at([x, y]: [i32; 2]) -> PropsUpdate {
    PropsUpdate {
        transform: Some(Transform::translate(f64::from(x), f64::from(y), 0.0)),
        ..PropsUpdate::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Rect = [0, 0, 1280, 800];

    #[test]
    fn a_menu_opens_beside_the_pointer_and_stays_on_the_screen() {
        assert_eq!(place([150, 200], [100, 100], SCREEN), [100, 100]);
        // Near the right edge it opens left of the pointer, near the
        // bottom above it: the taskbar menu grows up from the taskbar.
        assert_eq!(place([150, 200], [1200, 790], SCREEN), [1050, 590]);
        // Larger than the room on both sides: inside the screen.
        assert_eq!(place([150, 700], [600, 400], SCREEN), [600, 0]);
        assert_eq!(place([150, 200], [-10, 0], SCREEN), [0, 0]);
    }

    #[test]
    fn the_rows_follow_each_other() {
        let level = Level {
            entries: taskbar(true),
            rect: [0, 0, 200, 0],
            hovered: None,
            open: None,
        };
        // Toolbars, a separator, four rows, a separator...
        assert_eq!(level.row(0), [PAD, PAD, 200 - SHADOW - 2 * PAD, ROW]);
        assert_eq!(level.row(1)[1], PAD + ROW);
        assert_eq!(level.row(2)[1], PAD + ROW + SEPARATOR);
        assert!(!level.lit(7), "Task Manager is gray");
    }

    #[test]
    fn the_system_menu_follows_the_state_of_the_window() {
        let state = |entries: Vec<Entry>| -> Vec<(String, bool)> {
            entries
                .into_iter()
                .filter(|e| !e.is_separator())
                .map(|e| (e.title, e.enabled))
                .collect()
        };
        let rows = |r: [bool; 6]| -> Vec<(String, bool)> {
            ["Restore", "Move", "Size", "Minimize", "Maximize", "Close"]
                .into_iter()
                .zip(r)
                .map(|(t, e)| (t.to_string(), e))
                .collect()
        };
        assert_eq!(
            state(system(false, false)),
            rows([false, false, false, true, true, true])
        );
        assert_eq!(
            state(system(true, false)),
            rows([true, false, false, true, false, true])
        );
        assert_eq!(
            state(system(false, true)),
            rows([true, false, false, false, true, true])
        );
        let close = system(false, false).pop().expect("Close");
        assert!(close.bold);
        assert_eq!(close.key, "Alt+F4");
        assert_eq!(close.act, Some(Act::Window(Sys::Close)));
        assert_eq!(system(true, false)[4].act, None, "Maximize is gray");
    }

    #[test]
    fn icons_and_entries_have_xp_menus() {
        let rows = |t: Target| -> Vec<(String, bool)> {
            item(t)
                .into_iter()
                .filter(|e| !e.is_separator())
                .map(|e| (e.title, e.enabled))
                .collect()
        };
        let enabled = |t: Target| -> Vec<String> {
            rows(t)
                .into_iter()
                .filter(|(_, e)| *e)
                .map(|(t, _)| t)
                .collect()
        };
        assert_eq!(enabled(Target::Folder), ["Open", "Delete", "Properties"]);
        assert_eq!(enabled(Target::File), ["Open", "Delete", "Properties"]);
        assert_eq!(enabled(Target::Documents), ["Open", "Properties"]);
        assert_eq!(enabled(Target::Computer), ["Open"]);
        assert_eq!(
            enabled(Target::Trash(true)),
            ["Open", "Empty Recycle Bin", "Properties"]
        );
        assert_eq!(enabled(Target::Trash(false)), ["Open", "Properties"]);
        assert_eq!(
            enabled(Target::Program(true)),
            ["Open", "Remove from This List", "Properties"]
        );
    }

    #[test]
    fn open_is_the_default_row() {
        let rows = |t: Target| -> Vec<(String, bool)> {
            item(t)
                .into_iter()
                .filter(|e| !e.is_separator())
                .map(|e| (e.title, e.enabled))
                .collect()
        };
        // Open is the default row; the gray rows do nothing.
        let menu = item(Target::File);
        assert!(menu[0].bold && menu[0].act == Some(Act::Item(Item::Open)));
        assert!(rows(Target::File).contains(&("Rename".into(), false)));
    }

    #[test]
    fn rows_parse_their_node() {
        let menu = Menu {
            levels: vec![Level {
                entries: desktop(),
                rect: [0, 0, 100, 100],
                hovered: None,
                open: None,
            }],
            area: SCREEN,
            down: None,
            cues: false,
        };
        assert_eq!(menu.part("xp-ctx-0-6"), Some((0, 6)));
        assert_eq!(menu.part("xp-ctx-0-7"), None);
        assert_eq!(menu.part("xp-ctx-1-0"), None);
        assert_eq!(menu.part("xp-ctx-0-frame"), None);
    }

    /// A menu with a row, a gray row, a separator and a submenu.
    fn keyed(theme: &Theme) -> Menu {
        let entries = vec![
            Entry::item("Cascade Windows", Act::Cascade),
            Entry::gray("Toolbars"),
            Entry::separator(),
            Entry::sub(
                "Arrange",
                vec![Entry::item("Show the Desktop", Act::ShowDesktop)],
            ),
        ];
        Menu::new(theme, entries, [10, 10], SCREEN, 0)
    }

    #[test]
    fn the_keys_move_over_the_enabled_rows_into_the_submenu() {
        let (mut theme, cache) = crate::dialog::tests::theme("menukeys");
        let mut menu = keyed(&theme);
        menu.key(&mut theme, "root", "Down");
        assert_eq!(menu.levels[0].hovered, Some(0));
        // The gray row and the separator are skipped.
        menu.key(&mut theme, "root", "Down");
        assert_eq!(menu.levels[0].hovered, Some(3));
        // Right opens the submenu on its first row; Return acts there.
        menu.key(&mut theme, "root", "Right");
        assert_eq!(menu.levels.len(), 2);
        assert_eq!(
            menu.key(&mut theme, "root", "Return"),
            Key::Act(Act::ShowDesktop)
        );
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn escape_closes_the_submenu_then_the_menu() {
        let (mut theme, cache) = crate::dialog::tests::theme("menuescape");
        let mut menu = keyed(&theme);
        menu.key(&mut theme, "root", "Up");
        menu.key(&mut theme, "root", "Right");
        menu.key(&mut theme, "root", "Escape");
        assert_eq!(menu.levels.len(), 1);
        assert_eq!(menu.key(&mut theme, "root", "Escape"), Key::Close);
        // Up goes around to the first enabled row from the last one.
        menu.key(&mut theme, "root", "Up");
        assert_eq!(
            menu.key(&mut theme, "root", "Return"),
            Key::Act(Act::Cascade)
        );
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn home_and_end_go_to_the_ends_and_a_list_starts_on_its_value() {
        let (mut theme, cache) = crate::dialog::tests::theme("menuhome");
        let mut menu = keyed(&theme);
        menu.key(&mut theme, "root", "End");
        assert_eq!(menu.levels[0].hovered, Some(3));
        menu.key(&mut theme, "root", "Home");
        assert_eq!(menu.levels[0].hovered, Some(0));
        let list = vec![
            Entry::item("January", Act::Cascade),
            Entry::item("February", Act::Cascade).checked(true),
        ];
        let list = Menu::list(&theme, list, [10, 10], SCREEN, 120);
        assert_eq!(list.levels[0].hovered, Some(1));
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn a_letter_acts_on_the_row_of_its_access_key() {
        let (mut theme, cache) = crate::dialog::tests::theme("menuaccess");
        let mut menu = keyed(&theme);
        assert!(!menu.cues);
        // Ca&scade Windows; the gray &Toolbars takes no letter.
        assert!(matches!(
            menu.key(&mut theme, "root", "t"),
            Key::Commands(_)
        ));
        assert_eq!(menu.levels[0].hovered, None);
        assert!(menu.cues);
        assert_eq!(menu.key(&mut theme, "root", "S"), Key::Act(Act::Cascade));
        let _ = std::fs::remove_dir_all(cache);
    }
}
