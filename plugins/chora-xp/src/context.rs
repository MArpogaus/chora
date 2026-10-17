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
    /// A row of a menu of the Explorer window that has the focus.
    Explore(crate::explorer::Cmd),
}

/// A row of a menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    title: String,
    act: Option<Act>,
    enabled: bool,
    checked: bool,
    /// A row with an arrow, which opens its submenu.
    group: bool,
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
        let widest = entries
            .iter()
            .map(|e| theme.text_width(&e.title, Font::Ui))
            .fold(0.0, f64::max);
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

    fn lit(&self, index: usize) -> bool {
        self.entries[index].enabled
            && (self.hovered == Some(index) || (self.open == Some(index) && self.hovered.is_none()))
    }
}

/// The open menu, with its submenu.
pub struct Menu {
    levels: Vec<Level>,
    area: Rect,
    /// The row that went down.
    down: Option<(usize, usize)>,
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
        }
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
        let picture = row(theme, entry, [w, h], menu.lit(index));
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
        let mut commands = vec![Command::Remove { id: group_id(1) }];
        if let Some(open) = self.levels[0].open.take() {
            commands.push(self.relook(theme, 0, open));
        }
        commands
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
fn row(theme: &Theme, entry: &Entry, [w, h]: [i32; 2], lit: bool) -> Picture {
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
    picture = picture.text(Text::new(
        &entry.title,
        Font::Ui,
        color,
        [TEXT_X, 0, w - TEXT_X - TEXT_END + 4, h],
    ));
    if entry.group {
        // XP's arrow of a submenu, 4 by 7 pixels, in the color of the text.
        let [r, g, b, _] = color;
        let y = f64::from(h) / 2.0;
        picture = picture.svg(format!(
            r#"<path d="M{} {}l4 3.5l-4 3.5z" fill="rgb({r},{g},{b})"/>"#,
            w - 12,
            y - 3.5
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
        };
        assert_eq!(menu.part("xp-ctx-0-6"), Some((0, 6)));
        assert_eq!(menu.part("xp-ctx-0-7"), None);
        assert_eq!(menu.part("xp-ctx-1-0"), None);
        assert_eq!(menu.part("xp-ctx-0-frame"), None);
    }
}
