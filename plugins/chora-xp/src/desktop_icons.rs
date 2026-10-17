//! The icons on the desktop, as on XP: My Documents, My Computer and the
//! files of the Desktop folder in columns from the top left, and the
//! Recycle Bin at the bottom right. Each icon is an image node with the
//! 32-pixel icon and its label in white with a black shadow below it. A
//! click selects an icon, a double click opens it, a drag moves it to
//! another cell of the grid.

use std::{
    collections::HashMap,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use chora_scene::{Command, InputMode, NodeKind, PropsUpdate, Transform};

use crate::{
    apps, balloon,
    icons::{self, Icons},
    text::{Font, Text},
    theme::{Picture, Rect, Theme},
};

/// The group of the icons, between the wallpapers and the windows.
pub const GROUP: &str = "xp-icons";
const PREFIX: &str = "xp-icon-";
/// A cell of XP's grid: the icon spacing of 75 pixels in both directions.
const CELL: i32 = 75;
const ICON: i32 = 32;
/// The icon in its cell: centered, 2 pixels below the top.
const ICON_X: i32 = (CELL - ICON) / 2;
const ICON_TOP: i32 = 2;
/// The top of the label box in the cell, measured on XP.
const LABEL_TOP: i32 = 40;
const LINE: i32 = 13;
/// The widest line of a label.
const LABEL_PX: i32 = 72;
const LABEL_WIDTH: f64 = LABEL_PX as f64;
/// The Recycle Bin from the bottom right of the work area: XP's
/// default place, measured at 800x600.
const BIN_FROM_RIGHT: i32 = 78;
const BIN_FROM_BOTTOM: i32 = 86;
/// A press moves this far before it drags the icon.
const DRAG_START: f64 = 4.0;
const WHITE: [u8; 4] = [255, 255, 255, 255];
const BLACK: [u8; 4] = [0, 0, 0, 255];

/// What a double click on an icon does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Open {
    /// A folder: My Documents or a folder of the Desktop folder.
    Folder(PathBuf),
    /// My Computer.
    Computer,
    /// The Recycle Bin.
    Trash,
    /// A desktop file: its command.
    Run(String),
    /// Any other file.
    File(PathBuf),
}

/// An icon on the desktop.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    /// What it stands for: a path, or a name for My Computer and the
    /// Recycle Bin. Moved icons keep their place by it.
    pub key: String,
    /// The label.
    pub title: String,
    /// The icon name, or the path of an icon file.
    pub icon: String,
    /// What a double click does.
    pub open: Open,
}

/// The icon under the pointer while the left button is down.
struct Press {
    key: String,
    /// The pointer at the press.
    start: [f64; 2],
    /// The cell of the icon at the press.
    origin: [i32; 2],
    /// The pointer went far enough to drag the icon.
    moving: bool,
}

/// The icons on the desktop of the first output.
pub struct DesktopIcons {
    terminal: String,
    items: Vec<Item>,
    /// The work area above the taskbar.
    area: Rect,
    /// The places of the icons that the user moved, for the session.
    moved: HashMap<String, [i32; 2]>,
    /// The current place of each icon.
    places: Vec<[i32; 2]>,
    selected: Option<String>,
    /// The desktop has the focus: no window has it.
    focused: bool,
    last_press: Option<(String, Instant)>,
    press: Option<Press>,
    /// Arrange Icons By Type, else by name.
    by_type: bool,
}

impl DesktopIcons {
    /// The icons of the desktop; `terminal` runs desktop files that need
    /// one.
    pub fn new(terminal: &str) -> Self {
        Self {
            terminal: terminal.into(),
            items: Vec::new(),
            area: [0, 0, 0, 0],
            moved: HashMap::new(),
            places: Vec::new(),
            selected: None,
            focused: true,
            last_press: None,
            press: None,
            by_type: false,
        }
    }

    /// Whether a node is a desktop icon.
    pub fn owns(node: &str) -> bool {
        node.starts_with(PREFIX)
    }

    fn id(i: usize) -> String {
        format!("{PREFIX}{i}")
    }

    fn index(node: &str) -> Option<usize> {
        node.strip_prefix(PREFIX)?.parse().ok()
    }

    /// Looks at the Desktop folder and the trash again, and lays the
    /// icons out on a work area. Returns the commands that show them if
    /// anything changed.
    pub fn refresh(&mut self, theme: &mut Theme, icons: &mut Icons, area: Rect) -> Vec<Command> {
        let mut items = items(&self.terminal);
        if self.by_type {
            by_type(&mut items);
        }
        if items == self.items && area == self.area {
            return Vec::new();
        }
        let mut commands: Vec<Command> = (0..self.items.len())
            .map(|i| Command::Remove { id: Self::id(i) })
            .collect();
        self.items = items;
        self.area = area;
        self.places = self.place();
        if self
            .press
            .as_ref()
            .is_some_and(|p| self.find(&p.key).is_none())
        {
            self.press = None;
        }
        for i in 0..self.items.len() {
            commands.push(Command::Create {
                id: Self::id(i),
                kind: NodeKind::Image,
                parent: GROUP.into(),
                props: PropsUpdate {
                    input: Some(InputMode::Accept),
                    ..PropsUpdate::default()
                },
            });
            commands.push(self.look(theme, icons, i));
        }
        commands
    }

    /// Arrange Icons By Name or Type: the moved icons go back into the
    /// columns, in the new order.
    pub fn arrange(&mut self, theme: &mut Theme, icons: &mut Icons, by_type: bool) -> Vec<Command> {
        self.by_type = by_type;
        self.moved.clear();
        self.reload(theme, icons)
    }

    /// Refresh: shows the icons again, also when nothing changed.
    pub fn reload(&mut self, theme: &mut Theme, icons: &mut Icons) -> Vec<Command> {
        let area = std::mem::take(&mut self.area);
        self.refresh(theme, icons, area)
    }

    fn find(&self, key: &str) -> Option<usize> {
        self.items.iter().position(|item| item.key == key)
    }

    /// Where each icon goes: its moved place, else the next free cell of
    /// the columns, and the Recycle Bin at the bottom right.
    fn place(&self) -> Vec<[i32; 2]> {
        let (grid, bin) = layout(self.items.len(), self.area);
        let mut free = grid.into_iter();
        self.items
            .iter()
            .map(|item| {
                if let Some(place) = self.moved.get(&item.key) {
                    *place
                } else if item.open == Open::Trash {
                    bin
                } else {
                    free.next().unwrap_or(bin)
                }
            })
            .collect()
    }

    /// The picture of an icon and its place.
    fn look(&self, theme: &mut Theme, icons: &mut Icons, i: usize) -> Command {
        let selected = self.selected.as_deref() == Some(self.items[i].key.as_str());
        let picture = picture(theme, icons, &self.items[i], selected, self.focused);
        let [x, y] = self.places[i].map(f64::from);
        Command::Update {
            id: Self::id(i),
            props: PropsUpdate {
                image: Some(theme.render(&picture)),
                transform: Some(Transform::translate(x, y, 0.0)),
                ..PropsUpdate::default()
            },
        }
    }

    /// The icons as layers of a picture of the work area, for the faces
    /// of the cube: XP has one desktop, so every face shows its icons.
    pub fn paint(&self, theme: &mut Theme, icons: &mut Icons, mut face: Picture) -> Picture {
        for (item, [x, y]) in self.items.iter().zip(&self.places) {
            let picture = picture(theme, icons, item, false, false);
            let [w, h] = picture.size;
            let file = PathBuf::from(theme.render(&picture));
            face = face.file(&file, [x - self.area[0], y - self.area[1], w, h]);
        }
        face
    }

    /// The desktop gains or loses the focus: the selected label shows the
    /// dotted focus rectangle only while it has it.
    pub fn focus(&mut self, theme: &mut Theme, icons: &mut Icons, focused: bool) -> Vec<Command> {
        if self.focused == focused {
            return Vec::new();
        }
        self.focused = focused;
        self.selected
            .clone()
            .and_then(|key| self.find(&key))
            .map(|i| self.look(theme, icons, i))
            .into_iter()
            .collect()
    }

    /// Selects an icon, or none.
    pub fn select(
        &mut self,
        theme: &mut Theme,
        icons: &mut Icons,
        key: Option<String>,
    ) -> Vec<Command> {
        let old = std::mem::replace(&mut self.selected, key);
        if old == self.selected {
            return Vec::new();
        }
        [old, self.selected.clone()]
            .into_iter()
            .flatten()
            .filter_map(|key| self.find(&key))
            .map(|i| self.look(theme, icons, i))
            .collect()
    }

    /// A press of the left button on an icon selects it; the second press
    /// of a double click returns what to open.
    pub fn press(
        &mut self,
        theme: &mut Theme,
        icons: &mut Icons,
        node: &str,
        position: [f64; 2],
    ) -> (Vec<Command>, Option<Open>) {
        let Some(i) = Self::index(node).filter(|i| *i < self.items.len()) else {
            return (Vec::new(), None);
        };
        let key = self.items[i].key.clone();
        let now = Instant::now();
        let commands = self.select(theme, icons, Some(key.clone()));
        if is_double(self.last_press.take(), &key, now) {
            return (commands, Some(self.items[i].open.clone()));
        }
        self.last_press = Some((key.clone(), now));
        self.press = Some(Press {
            key,
            start: position,
            origin: self.places[i],
            moving: false,
        });
        (commands, None)
    }

    /// Whether an icon is down under the pointer.
    pub fn pressed(&self) -> bool {
        self.press.is_some()
    }

    /// The pointer moved while an icon is down: the icon follows it once
    /// it went a few pixels.
    pub fn drag(&mut self, [px, py]: [f64; 2]) -> Vec<Command> {
        let Some(press) = &mut self.press else {
            return Vec::new();
        };
        let [dx, dy] = [px - press.start[0], py - press.start[1]];
        press.moving |= dx.hypot(dy) >= DRAG_START;
        if !press.moving {
            return Vec::new();
        }
        let [x, y] = press.origin.map(f64::from);
        let key = press.key.clone();
        let Some(i) = self.find(&key) else {
            return Vec::new();
        };
        vec![Command::Update {
            id: Self::id(i),
            props: PropsUpdate {
                transform: Some(Transform::translate(x + dx, y + dy, 0.0)),
                ..PropsUpdate::default()
            },
        }]
    }

    /// The release of a dragged icon: it snaps to the nearest cell, or
    /// goes back if another icon is there.
    pub fn release(&mut self, [px, py]: [f64; 2]) -> Vec<Command> {
        let Some(press) = self.press.take().filter(|p| p.moving) else {
            return Vec::new();
        };
        let Some(i) = self.find(&press.key) else {
            return Vec::new();
        };
        #[allow(clippy::cast_possible_truncation)]
        let dropped = [
            press.origin[0] + (px - press.start[0]).round() as i32,
            press.origin[1] + (py - press.start[1]).round() as i32,
        ];
        let cell = snap(dropped, self.area);
        // ponytail: a taken cell sends the icon back; XP finds the
        // nearest free one.
        let taken =
            self.places.iter().enumerate().any(|(j, p)| {
                j != i && (p[0] - cell[0]).abs() < CELL && (p[1] - cell[1]).abs() < CELL
            });
        if !taken {
            self.places[i] = cell;
            self.moved.insert(press.key, cell);
        }
        let [x, y] = self.places[i].map(f64::from);
        vec![Command::Update {
            id: Self::id(i),
            props: PropsUpdate {
                transform: Some(Transform::translate(x, y, 0.0)),
                ..PropsUpdate::default()
            },
        }]
    }
}

/// The picture of an icon: the icon centered at the top of a cell and
/// the label below it, in at most two lines unless it is selected. A
/// selected label is white on XP's dark blue, with a dotted focus
/// rectangle while the desktop has the focus.
fn picture(
    theme: &mut Theme,
    icons: &mut Icons,
    item: &Item,
    selected: bool,
    focused: bool,
) -> Picture {
    let width = |s: &str| theme.text_width(s, Font::Ui);
    let lines = label(&item.title, if selected { usize::MAX } else { 2 }, width);
    let widths: Vec<f64> = lines.iter().map(|l| width(l).min(LABEL_WIDTH)).collect();
    #[allow(clippy::cast_possible_truncation)]
    let text = widths.iter().copied().fold(0.0, f64::max).ceil() as i32;
    let rows = i32::try_from(lines.len()).unwrap_or(1);
    // The box as on XP: 1 pixel of room left of the ink, 2 right of it.
    let boxed = [(CELL - text - 2) / 2, LABEL_TOP, text + 3, rows * LINE + 2];
    let mut picture = Picture::new(CELL, LABEL_TOP + boxed[3]);
    if let Some(file) = icons.find(&item.icon) {
        picture = picture.file(&file, [ICON_X, ICON_TOP, ICON, ICON]);
    }
    if selected {
        let [_, _, w, h] = boxed;
        let svg = label_box([w, h], focused);
        let name = format!("label-{w}x{h}-{focused}.svg");
        if let Ok(file) = theme.cache_file(&name, &svg) {
            picture = picture.file(&file, boxed);
        }
    }
    for ((line, w), row) in lines.iter().zip(&widths).zip(0..) {
        #[allow(clippy::cast_possible_truncation)]
        let x = ((f64::from(CELL) - w) / 2.0).round() as i32;
        let rect = [x, LABEL_TOP + 1 + row * LINE, LABEL_PX, LINE];
        let mut text = Text::new(line, Font::Ui, WHITE, rect);
        if !selected {
            text = text.shadow(BLACK);
        }
        picture = picture.text(text);
    }
    picture
}

/// XP's selected label: dark blue, and the focus rectangle as every
/// other pixel of the edge in the inverse color.
fn label_box([w, h]: [i32; 2], focused: bool) -> String {
    let mut dots = String::new();
    if focused {
        for y in 0..h {
            for x in 0..w {
                let edge = x == 0 || y == 0 || x == w - 1 || y == h - 1;
                if edge && (x + y) % 2 == 0 {
                    let _ = write!(dots, "M{x} {y}h1v1h-1z");
                }
            }
        }
    }
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" shape-rendering="crispEdges"><rect width="{w}" height="{h}" fill="#004E98"/><path d="{dots}" fill="#FFB167"/></svg>"##
    )
}

/// The lines of a label, wrapped at XP's width: at most `max`, and the
/// last one takes the rest of the text, to end with an ellipsis.
fn label(title: &str, max: usize, width: impl Fn(&str) -> f64) -> Vec<String> {
    let mut lines = balloon::wrap(title, LABEL_WIDTH, width);
    if lines.len() > max {
        let rest = lines.split_off(max - 1).join(" ");
        lines.push(rest);
    }
    lines
}

/// The files of the Desktop folder by type: folders first, then by the
/// extension, and by name within a type. My Documents and My Computer
/// stay first, the Recycle Bin last.
fn by_type(items: &mut [Item]) {
    let end = items.len().saturating_sub(1);
    if let Some(files) = items.get_mut(2..end) {
        files.sort_by_cached_key(|item| {
            let folder = matches!(item.open, Open::Folder(_));
            let extension = Path::new(&item.key)
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase());
            (!folder, extension)
        });
    }
}

/// The cells of `count` icons in columns from the top left of a work
/// area, and the cell of the Recycle Bin at its bottom right; the columns
/// skip the cell of the bin.
fn layout(count: usize, [x, y, w, h]: Rect) -> (Vec<[i32; 2]>, [i32; 2]) {
    let bin = [x + w - BIN_FROM_RIGHT, y + h - BIN_FROM_BOTTOM];
    let rows = ((h - LABEL_TOP - 2 * LINE - 2) / CELL + 1).max(1);
    let cells = (0..)
        .map(|n| [x + n / rows * CELL, y + n % rows * CELL])
        .filter(|c| (c[0] - bin[0]).abs() >= CELL || (c[1] - bin[1]).abs() >= CELL)
        .take(count)
        .collect();
    (cells, bin)
}

/// The cell of the grid nearest to a place, inside the work area.
fn snap([px, py]: [i32; 2], [x, y, w, h]: Rect) -> [i32; 2] {
    let cell = |p: i32, origin: i32, room: i32| {
        let last = ((room - CELL) / CELL).max(0);
        origin + ((p - origin + CELL / 2).div_euclid(CELL)).clamp(0, last) * CELL
    };
    [cell(px, x, w), cell(py, y, h)]
}

/// Two presses on the same icon within XP's double-click time.
fn is_double(last: Option<(String, Instant)>, key: &str, now: Instant) -> bool {
    const DOUBLE_CLICK: Duration = Duration::from_millis(500);
    last.is_some_and(|(k, at)| k == key && now.duration_since(at) < DOUBLE_CLICK)
}

/// The icons of the desktop: My Documents, My Computer, the files of the
/// Desktop folder by name, folders first, and the Recycle Bin. My Network
/// Places is left out: without a network browser it opens nothing.
fn items(terminal: &str) -> Vec<Item> {
    let home = std::env::var_os("HOME").map_or_else(|| PathBuf::from("/"), PathBuf::from);
    let documents = user_dir("DOCUMENTS").unwrap_or_else(|| home.clone());
    let mut items = vec![
        Item {
            key: documents.to_string_lossy().into_owned(),
            title: "My Documents".into(),
            icon: "folder-documents".into(),
            open: Open::Folder(documents),
        },
        Item {
            key: "computer".into(),
            title: "My Computer".into(),
            icon: "computer".into(),
            open: Open::Computer,
        },
    ];
    let desktop = user_dir("DESKTOP").unwrap_or_else(|| home.join("Desktop"));
    if desktop != home {
        items.extend(scan(&desktop, terminal));
    }
    let full = trash_full(&trash_dir());
    items.push(Item {
        key: "trash".into(),
        title: "Recycle Bin".into(),
        icon: if full {
            "user-trash-full"
        } else {
            "user-trash"
        }
        .into(),
        open: Open::Trash,
    });
    items
}

/// The files and folders of a directory as icons, without the hidden
/// ones: folders first, then by name. A desktop file shows its name and
/// icon and runs its program.
fn scan(dir: &Path, terminal: &str) -> Vec<Item> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| !n.to_string_lossy().starts_with('.'))
        })
        .collect();
    paths.sort_by_cached_key(|p| (!p.is_dir(), p.to_string_lossy().to_lowercase()));
    paths
        .into_iter()
        .map(|path| {
            let key = path.to_string_lossy().into_owned();
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if path.is_dir() {
                return Item {
                    key,
                    title: name,
                    icon: "folder".into(),
                    open: Open::Folder(path),
                };
            }
            if path.extension().is_some_and(|e| e == "desktop")
                && let Some((title, icon, command)) = apps::launcher(&path, terminal)
            {
                return Item {
                    key,
                    title,
                    icon,
                    open: Open::Run(command),
                };
            }
            // ponytail: one icon for all files; a MIME lookup would give
            // each type its own.
            Item {
                key,
                title: name,
                icon: "text-x-generic".into(),
                open: Open::File(path),
            }
        })
        .collect()
}

/// A user directory of `user-dirs.dirs`, such as `DESKTOP`: from the
/// environment variable `XDG_DESKTOP_DIR` first, then from the file.
pub fn user_dir(name: &str) -> Option<PathBuf> {
    let var = format!("XDG_{name}_DIR");
    if let Some(dir) = std::env::var_os(&var) {
        return Some(dir.into());
    }
    let home = std::env::var("HOME").ok()?;
    let config = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{home}/.config"));
    let text = fs::read_to_string(Path::new(&config).join("user-dirs.dirs")).ok()?;
    parse_user_dir(&text, &var, &home)
}

/// The value of a line `XDG_DESKTOP_DIR="$HOME/Desktop"`.
fn parse_user_dir(text: &str, var: &str, home: &str) -> Option<PathBuf> {
    text.lines()
        .filter_map(|l| l.trim().strip_prefix(var)?.strip_prefix('='))
        .map(|v| v.trim().trim_matches('"').replace("$HOME", home))
        .next_back()
        .map(PathBuf::from)
}

/// The files of the trash: `Trash/files` of `$XDG_DATA_HOME`.
pub fn trash_dir() -> PathBuf {
    icons::data_dirs()
        .into_iter()
        .next()
        .unwrap_or_default()
        .join("Trash/files")
}

fn trash_full(dir: &Path) -> bool {
    fs::read_dir(dir).is_ok_and(|mut d| d.next().is_some())
}

/// The shell command that opens an icon. Folders, My Computer and the
/// Recycle Bin go through [`explore`].
pub fn command(open: &Open) -> String {
    match open {
        Open::Folder(path) => explore(path),
        Open::Computer => explore(Path::new("/")),
        // The trash of GVFS, else its folder.
        Open::Trash => format!(
            "gio open trash:/// 2>/dev/null || {}",
            explore(&trash_dir())
        ),
        Open::Run(command) => command.clone(),
        Open::File(path) => format!("xdg-open {}", quote(&path.to_string_lossy())),
    }
}

/// Opens a folder. The hook for XP's Explorer: until it comes, the
/// file manager of `xdg-open` shows the folder.
fn explore(path: &Path) -> String {
    format!("xdg-open {}", quote(&path.to_string_lossy()))
}

/// Quotes a word for the shell.
fn quote(word: &str) -> String {
    format!("'{}'", word.replace('\'', r"'\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrange_by_type_keeps_the_special_icons_in_place() {
        let item = |key: &str, open: Open| Item {
            key: key.into(),
            title: key.into(),
            icon: String::new(),
            open,
        };
        let file = |key: &str| item(key, Open::File(key.into()));
        let mut items = vec![
            item("docs", Open::Computer),
            item("computer", Open::Computer),
            file("/d/a.txt"),
            file("/d/b.png"),
            item("/d/c", Open::Folder("/d/c".into())),
            file("/d/d.png"),
            item("trash", Open::Trash),
        ];
        by_type(&mut items);
        let keys: Vec<&str> = items.iter().map(|i| i.key.as_str()).collect();
        assert_eq!(
            keys,
            [
                "docs", "computer", "/d/c", "/d/b.png", "/d/d.png", "/d/a.txt", "trash"
            ]
        );
    }

    fn mono(s: &str) -> f64 {
        5.0 * f64::from(u16::try_from(s.chars().count()).unwrap_or(u16::MAX))
    }

    #[test]
    fn icons_fill_columns_and_the_bin_sits_bottom_right() {
        let (cells, bin) = layout(9, [0, 0, 800, 570]);
        assert_eq!(bin, [722, 484]);
        // Seven rows of 75 pixels fit above the taskbar.
        assert_eq!(cells[..2], [[0, 0], [0, 75]]);
        assert_eq!(cells[7], [75, 0]);
        // The columns skip the cell of the bin.
        let (cells, bin) = layout(5, [0, 0, 300, 160]);
        assert_eq!(bin, [222, 74]);
        assert_eq!(cells, [[0, 0], [0, 75], [75, 0], [75, 75], [300, 0]]);
    }

    #[test]
    fn a_drop_snaps_to_the_nearest_cell_inside_the_area() {
        let area = [100, 0, 800, 570];
        assert_eq!(snap([140, 30], area), [175, 0]);
        assert_eq!(snap([120, 120], area), [100, 150]);
        assert_eq!(snap([-500, 9000], area), [100, 450]);
    }

    #[test]
    fn labels_wrap_to_two_lines_unless_selected() {
        assert_eq!(label("My Documents", 2, mono), ["My Documents"]);
        assert_eq!(
            label("My Network Places", 2, mono),
            ["My Network", "Places"]
        );
        let long = "a very long name of a text file";
        assert_eq!(label(long, 2, mono), ["a very long", "name of a text file"]);
        assert_eq!(label(long, usize::MAX, mono).len(), 3);
    }

    #[test]
    fn double_clicks_are_quick_and_on_one_icon() {
        let now = Instant::now();
        let at = |ms| {
            Some((
                "a".to_string(),
                now.checked_sub(Duration::from_millis(ms)).unwrap(),
            ))
        };
        assert!(is_double(at(200), "a", now));
        assert!(!is_double(at(600), "a", now));
        assert!(!is_double(at(200), "b", now));
        assert!(!is_double(None, "a", now));
    }

    #[test]
    fn the_desktop_folder_lists_folders_first_and_runs_desktop_files() {
        let dir = std::env::temp_dir().join(format!("chora-xp-desk-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("zeta")).unwrap();
        fs::write(dir.join("Alpha.txt"), "").unwrap();
        fs::write(dir.join(".hidden"), "").unwrap();
        fs::write(
            dir.join("app.desktop"),
            "[Desktop Entry]\nType=Application\nName=Editor\nExec=edit %F\nIcon=ed\n",
        )
        .unwrap();
        let items = scan(&dir, "foot");
        let titles: Vec<&str> = items.iter().map(|i| i.title.as_str()).collect();
        assert_eq!(titles, ["zeta", "Alpha.txt", "Editor"]);
        assert_eq!(items[0].open, Open::Folder(dir.join("zeta")));
        assert_eq!(items[2].open, Open::Run("edit".into()));
        assert_eq!(items[2].icon, "ed");
        assert!(!trash_full(&dir.join("zeta")));
        assert!(trash_full(&dir));
        assert!(!trash_full(&dir.join("missing")));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn user_dirs_come_from_their_file() {
        let text =
            "# comment\nXDG_DESKTOP_DIR=\"$HOME/Schreibtisch\"\nXDG_DOCUMENTS_DIR=\"/data/docs\"\n";
        let dir = |var| parse_user_dir(text, var, "/home/u");
        assert_eq!(dir("XDG_DESKTOP_DIR"), Some("/home/u/Schreibtisch".into()));
        assert_eq!(dir("XDG_DOCUMENTS_DIR"), Some("/data/docs".into()));
        assert_eq!(dir("XDG_MUSIC_DIR"), None);
    }

    #[test]
    fn paths_are_quoted_for_the_shell() {
        let open = Open::File("/tmp/it's here".into());
        assert_eq!(command(&open), r"xdg-open '/tmp/it'\''s here'");
    }
}
