//! The start menu: XP's header with the user name, programs on the left,
//! places on the right, and Log Off and Turn Off Computer in the footer.
//! The geometry follows the measurements of the reference start menu
//! (`XpTheme.qml`): 382 pixels wide, a 64 pixel header, a 40 pixel footer.

use std::path::PathBuf;

use chora_scene::{Command, NodeKind, PropsUpdate, Transform};

use crate::{
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
const MIN_BODY: i32 = 330;
const GROUP: &str = "xp-menu";

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
    /// The command of All Programs and Run.
    pub launcher: String,
}

/// The start menu, one for all outputs.
pub struct Menu {
    user: String,
    entries: Vec<Entry>,
    /// The left column ends with a separator and All Programs.
    separators: Vec<Rect>,
    height: i32,
    /// The output whose start button opened the menu.
    open: Option<String>,
    /// The entry that is down.
    pressed: Option<usize>,
    /// The top-left corner in scene coordinates.
    origin: [i32; 2],
}

impl Menu {
    /// Lays out the menu.
    pub fn new(config: &Config, theme: &Theme, icons: &mut Icons) -> Self {
        let mut entries = programs(config, icons);
        let places = places(config, icons);
        let body = body_height(&entries, &places);
        let height = HEADER + 1 + body + 1 + FOOTER;
        let top = HEADER + 1;
        let mut separators = vec![[
            FRAME + 2,
            top + body - 6 - PLACE_ROW - SEPARATOR,
            LEFT - 8,
            SEPARATOR,
        ]];
        separators.extend(place_separators(&places, top));
        entries.push(Entry {
            title: "All Programs".into(),
            subtitle: None,
            icon: None,
            bold: true,
            kind: Kind::Program,
            action: Action::Run(config.launcher.clone()),
            rect: [FRAME + 2, top + body - 6 - PLACE_ROW, LEFT - 4, PLACE_ROW],
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
            origin: [0, 0],
        }
    }

    /// Creates the hidden menu: the frame, the header and one image per
    /// entry.
    pub fn create(&self, theme: &mut Theme, parent: &str) -> Vec<Command> {
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
                id: "xp-menu-frame".into(),
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
                    image: Some(theme.render(&picture(entry, false))),
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

    /// Shows the menu above a start button.
    pub fn open(&mut self, output: &str, [x, bottom]: [i32; 2]) -> Vec<Command> {
        self.open = Some(output.into());
        self.origin = [x, bottom - self.height];
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

    /// Hides the menu.
    pub fn close(&mut self, theme: &mut Theme) -> Vec<Command> {
        if self.open.take().is_none() {
            return Vec::new();
        }
        let mut commands = self.release_look(theme);
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

    /// Whether a node belongs to the menu.
    pub fn owns(node: &str) -> bool {
        node.starts_with("xp-menu") || node.starts_with("xp-item-")
    }

    /// Highlights an entry that went down.
    pub fn press(&mut self, theme: &mut Theme, node: &str) -> Vec<Command> {
        let Some(index) = item_index(node).filter(|i| *i < self.entries.len()) else {
            return Vec::new();
        };
        self.pressed = Some(index);
        vec![self.look(theme, index, true)]
    }

    /// The action of the entry that went down, if the pointer is still on
    /// it, and the commands that end the highlight.
    pub fn release(
        &mut self,
        theme: &mut Theme,
        [px, py]: [f64; 2],
    ) -> (Vec<Command>, Option<Action>) {
        let Some(index) = self.pressed else {
            return (Vec::new(), None);
        };
        let [x, y, w, h] = self.entries[index].rect;
        let (left, top) = (f64::from(self.origin[0] + x), f64::from(self.origin[1] + y));
        let inside =
            (left..left + f64::from(w)).contains(&px) && (top..top + f64::from(h)).contains(&py);
        let action = inside.then(|| self.entries[index].action.clone());
        (self.release_look(theme), action)
    }

    fn release_look(&mut self, theme: &mut Theme) -> Vec<Command> {
        self.pressed
            .take()
            .map(|i| self.look(theme, i, false))
            .into_iter()
            .collect()
    }

    fn look(&self, theme: &mut Theme, index: usize, down: bool) -> Command {
        Command::Update {
            id: item_id(index),
            props: PropsUpdate {
                image: Some(theme.render(&picture(&self.entries[index], down))),
                ..PropsUpdate::default()
            },
        }
    }
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
    let left = bottom(programs) + SEPARATOR + PLACE_ROW + 6;
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
fn picture(entry: &Entry, down: bool) -> Picture {
    let [_, _, w, h] = entry.rect;
    let mut picture = Picture::new(w, h);
    if down {
        picture = match entry.kind {
            Kind::Footer => picture.frame(Sheet::Winxp, "startmenu-footer-pressed", [0, 0, w, h]),
            _ => picture.element(Sheet::Winxp, "menu-highlight-center", [0, 0, w, h]),
        };
    }
    let icon_size = match entry.kind {
        Kind::Program => 32,
        Kind::Place | Kind::Footer => 24,
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
        None if entry.title == "All Programs" => picture
            .text(Text::new(&entry.title, font, color, [x, 0, width, h]).centered())
            .element(
                Sheet::Winxp,
                "startmenu-allprograms-arrow",
                [w - 40, (h - 22) / 2, 17, 22],
            ),
        None => picture.text(Text::new(&entry.title, font, color, [x, 0, width, h])),
    }
}

fn text_color(kind: Kind, down: bool) -> [u8; 4] {
    match (kind, down) {
        (Kind::Footer, _) | (_, true) => [255, 255, 255, 255],
        (Kind::Program, false) => [0, 0, 0, 255],
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
