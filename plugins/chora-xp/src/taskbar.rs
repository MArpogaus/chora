//! The taskbar at the bottom of the work area of each output, drawn as
//! image nodes: XP's blue bar, the green start button, one button per
//! window and the notification area with the clock.

use std::{collections::HashMap, path::PathBuf};

use chora_scene::{Command, NodeKind, OutputInfo, PropsUpdate, Transform};

use crate::{
    cube::DESKTOPS,
    icons,
    text::{Font, Text},
    theme::{Picture, Rect, Sheet, Theme},
};

/// The height of the taskbar.
pub const HEIGHT: i32 = 30;
const START_WIDTH: i32 = 100;
/// The left edge of the first task button.
const TASKS_LEFT: i32 = 106;
const TASK_TOP: i32 = 3;
const TASK_HEIGHT: i32 = 25;
const TASK_GAP: i32 = 3;
const TASK_WIDTH: i32 = 160;
const TRAY_WIDTH: i32 = 82;
/// The notification area without the clock.
const TRAY_EMPTY: i32 = 14;
/// What shows of a taskbar that hides itself, as on XP.
pub const HIDDEN_STRIP: i32 = 2;
/// How long the taskbar takes to slide away or back.
const SLIDE_TIME: f64 = 0.15;
/// The room of an icon in the notification area, and its left edge.
const TRAY_ICON: i32 = 22;
const TRAY_ICON_LEFT: i32 = 10;
/// The buttons of the desktops, left of the tray, as in the virtual
/// desktop manager of the power toys for XP.
const PAGER_BUTTON: i32 = 22;
const PAGER_GAP: i32 = 2;
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)] // four
const PAGER_WIDTH: i32 = DESKTOPS as i32 * PAGER_BUTTON + (DESKTOPS as i32 - 1) * PAGER_GAP + 6;
const WHITE: [u8; 4] = [255, 255, 255, 255];

/// The settings of Taskbar and Start Menu Properties, for the session.
#[allow(clippy::struct_excessive_bools)] // independent check boxes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    /// Lock the taskbar: only a check mark, since chora-xp neither moves
    /// nor sizes the taskbar.
    pub locked: bool,
    /// The taskbar slides away until the pointer touches the bottom edge.
    pub auto_hide: bool,
    /// Windows cannot cover the taskbar.
    pub on_top: bool,
    /// The clock in the notification area.
    pub clock: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            locked: true,
            auto_hide: false,
            on_top: true,
            clock: true,
        }
    }
}

/// A window as its task button shows it.
pub struct Task<'a> {
    /// The surface node.
    pub surface: &'a str,
    /// The window title.
    pub title: &'a str,
    /// The icon of the application.
    pub icon: Option<PathBuf>,
    /// The window has the focus.
    pub active: bool,
    /// The window is minimized.
    pub minimized: bool,
    /// The pointer is on the button.
    pub hovered: bool,
}

/// A node of a taskbar, by its id.
#[derive(Debug, PartialEq, Eq)]
pub enum Part {
    /// The start button of an output.
    Start(String),
    /// The task button of a surface.
    Task(String),
    /// The button of a desktop.
    Pager(usize),
}

impl Part {
    /// The part that a node id names.
    pub fn of(node: &str) -> Option<Self> {
        if let Some(output) = node.strip_prefix("xp-start-") {
            return Some(Self::Start(output.into()));
        }
        if let Some(rest) = node.strip_prefix("xp-pager-") {
            let (desktop, _output) = rest.split_once('-')?;
            return desktop.parse().ok().map(Self::Pager);
        }
        let rest = node.strip_prefix("xp-task-")?;
        let (surface, _output) = rest.split_once('-')?;
        Some(Self::Task(surface.into()))
    }
}

/// The taskbar of one output.
pub struct Bar {
    /// The output name.
    pub output: String,
    /// The work area of the output.
    area: Rect,
    /// The position of the output in the layout.
    position: [i32; 2],
    /// The icon in the notification area, left of the clock.
    pub icon: Option<PathBuf>,
    /// The surfaces with a button, from the left.
    tasks: Vec<String>,
    /// The last update of each task button, to skip repeats.
    shown: HashMap<String, Command>,
    /// The taskbar hides itself when the pointer leaves it.
    auto_hide: bool,
    /// The clock shows.
    clock: bool,
    /// How far the taskbar slid down, and how far it goes.
    offset: f64,
    target: f64,
}

impl Bar {
    /// The taskbar of an output.
    pub fn new(output: &OutputInfo) -> Self {
        Self {
            output: output.name.clone(),
            area: output.layout_area(),
            position: output.position,
            icon: None,
            tasks: Vec::new(),
            shown: HashMap::new(),
            auto_hide: false,
            clock: true,
            offset: 0.0,
            target: 0.0,
        }
    }

    /// Takes the settings: a taskbar that hides itself leaves the work
    /// area to the windows but for its strip, and slides away.
    pub fn configure(&mut self, settings: Settings) {
        self.clock = settings.clock;
        self.auto_hide = settings.auto_hide;
        self.hide(settings.auto_hide);
    }

    /// Slides the taskbar away or back; only one that hides itself goes.
    pub fn hide(&mut self, hidden: bool) {
        self.target = if hidden && self.auto_hide {
            f64::from(HEIGHT - HIDDEN_STRIP)
        } else {
            0.0
        };
    }

    /// Whether the taskbar slides.
    pub fn sliding(&self) -> bool {
        (self.offset - self.target).abs() > f64::EPSILON
    }

    /// The next step of the slide after `dt` seconds.
    pub fn slide(&mut self, dt: f64) -> Option<Command> {
        if !self.sliding() {
            return None;
        }
        let step = f64::from(HEIGHT) * dt / SLIDE_TIME;
        self.offset = if self.offset < self.target {
            (self.offset + step).min(self.target)
        } else {
            (self.offset - step).max(self.target)
        };
        let [x, y] = self.origin().map(f64::from);
        Some(update(&self.group(), at(x, y + self.offset)))
    }

    fn group(&self) -> String {
        format!("xp-bar-{}", self.output)
    }

    /// Whether a node belongs to the taskbar.
    pub fn owns(&self, node: &str) -> bool {
        [
            "xp-panel-",
            "xp-start-",
            "xp-tray-",
            "xp-task-",
            "xp-pager-",
        ]
        .iter()
        .any(|p| node.starts_with(p))
            && node.ends_with(&format!("-{}", self.output))
    }

    fn task_id(&self, surface: &str) -> String {
        format!("xp-task-{surface}-{}", self.output)
    }

    /// The work area of the output, with the taskbar.
    pub fn area(&self) -> Rect {
        self.area
    }

    /// The part of the work area above the taskbar, or above its strip
    /// when it hides itself.
    pub fn usable(&self) -> Rect {
        let [x, y, w, h] = self.area;
        let bar = if self.auto_hide { HIDDEN_STRIP } else { HEIGHT };
        [x, y, w, (h - bar).max(1)]
    }

    /// Whether a point lies in the work area of the output.
    pub fn contains(&self, [px, py]: [i32; 2]) -> bool {
        let [x, y, w, h] = self.area;
        (x..x + w).contains(&px) && (y..y + h).contains(&py)
    }

    /// The top-left corner of the taskbar in scene coordinates.
    pub fn origin(&self) -> [i32; 2] {
        let [x, y, _, h] = self.area;
        [x, y + h - HEIGHT]
    }

    /// Creates the taskbar with its background, start button, desktop
    /// buttons and tray.
    pub fn create(
        &self,
        theme: &mut Theme,
        parent: &str,
        clock: &str,
        desktop: usize,
    ) -> Vec<Command> {
        let image = |id: String| Command::Create {
            id,
            kind: NodeKind::Image,
            parent: self.group(),
            props: PropsUpdate::default(),
        };
        let mut commands = vec![
            Command::Create {
                id: self.group(),
                kind: NodeKind::Group,
                parent: parent.into(),
                props: PropsUpdate::default(),
            },
            image(format!("xp-panel-{}", self.output)),
            image(format!("xp-start-{}", self.output)),
            image(format!("xp-tray-{}", self.output)),
        ];
        commands.extend((0..DESKTOPS).map(|d| image(self.pager_id(d))));
        commands.extend(self.layout(theme));
        commands.push(self.start(theme, "normal"));
        commands.push(self.tray(theme, clock));
        commands.extend(self.pager(theme, desktop, None));
        commands
    }

    /// Moves the taskbar to the new work area of its output.
    pub fn resize(&mut self, theme: &mut Theme, output: &OutputInfo, clock: &str) -> Vec<Command> {
        self.area = output.layout_area();
        self.position = output.position;
        let mut commands = self.layout(theme);
        commands.push(self.tray(theme, clock));
        commands
    }

    /// The group at the bottom of the work area and the background over
    /// its width: the top edge and the center of the panel of Exposé.
    fn layout(&self, theme: &mut Theme) -> Vec<Command> {
        let [x, y] = self.origin().map(f64::from);
        let top = theme.size(Sheet::Panel, "top").map_or(6, |s| s[1]);
        let picture = Picture::new(32, top + 32)
            .element(Sheet::Panel, "top", [0, 0, 32, top])
            .element(Sheet::Panel, "center", [0, top, 32, 32]);
        vec![
            update(&self.group(), at(x, y + self.offset)),
            update(
                &format!("xp-panel-{}", self.output),
                PropsUpdate {
                    image: Some(theme.render(&picture)),
                    size: Some([f64::from(self.area[2]), f64::from(HEIGHT)]),
                    slice: Some([0.0, f64::from(top), 0.0, 0.0]),
                    ..PropsUpdate::default()
                },
            ),
        ]
    }

    /// The start button in a look: `normal`, `hover` or `pressed`.
    pub fn start(&self, theme: &mut Theme, look: &str) -> Command {
        let picture = start_button(theme, look);
        update(
            &format!("xp-start-{}", self.output),
            PropsUpdate {
                image: Some(theme.render(&picture)),
                ..PropsUpdate::default()
            },
        )
    }

    fn pager_id(&self, desktop: usize) -> String {
        format!("xp-pager-{desktop}-{}", self.output)
    }

    /// The buttons of the desktops, the current one down, and one maybe
    /// under the pointer.
    pub fn pager(&self, theme: &mut Theme, current: usize, hovered: Option<usize>) -> Vec<Command> {
        let left = self.area[2] - self.tray_width() - PAGER_WIDTH;
        (0..DESKTOPS)
            .zip(0..)
            .map(|(desktop, i)| {
                let prefix = match (desktop == current, hovered == Some(desktop)) {
                    (true, true) => "focus-hover",
                    (true, false) => "focus",
                    (false, true) => "hover",
                    (false, false) => "normal",
                };
                let label = Text::new(
                    &(i + 1).to_string(),
                    Font::UiBold,
                    WHITE,
                    [0, 0, PAGER_BUTTON, TASK_HEIGHT],
                )
                .centered();
                let picture = Picture::new(PAGER_BUTTON, TASK_HEIGHT)
                    .frame(Sheet::Tasks, prefix, [0, 0, PAGER_BUTTON, TASK_HEIGHT])
                    .text(label);
                update(
                    &self.pager_id(desktop),
                    PropsUpdate {
                        image: Some(theme.render(&picture)),
                        ..at(
                            f64::from(left + i * (PAGER_BUTTON + PAGER_GAP)),
                            f64::from(TASK_TOP),
                        )
                    },
                )
            })
            .collect()
    }

    /// The width of the notification area: it grows for an icon.
    fn tray_width(&self) -> i32 {
        tray_width(self.clock, self.icon.is_some())
    }

    /// The clock in scene coordinates, while it shows: a double click
    /// there opens Date and Time.
    pub fn clock_rect(&self) -> Option<Rect> {
        let [x, y, w, h] = self.area;
        self.clock
            .then(|| [x + w - TRAY_WIDTH, y + h - HEIGHT, TRAY_WIDTH, HEIGHT])
    }

    /// The notification area with the icon and the clock, at the right
    /// edge.
    pub fn tray(&self, theme: &mut Theme, clock: &str) -> Command {
        let width = self.tray_width();
        let clock = self.clock.then_some(clock);
        let picture = tray(width, clock, self.icon.as_deref());
        update(
            &format!("xp-tray-{}", self.output),
            PropsUpdate {
                image: Some(theme.render(&picture)),
                ..at(f64::from(self.area[2] - width), 0.0)
            },
        )
    }

    /// Where the tail of a balloon points, in the coordinates of the
    /// output: the top of the icon of the notification area; and the
    /// right edge of the work area.
    pub fn tray_tip(&self) -> ([i32; 2], i32) {
        let [x, y, w, h] = self.area;
        let [x, y] = [x - self.position[0], y - self.position[1]];
        let icon = x + w - self.tray_width() + TRAY_ICON_LEFT + 8;
        ([icon, y + h - HEIGHT + 8], x + w)
    }

    /// The task buttons: new ones for new windows, none for closed ones,
    /// and every button in its look and place.
    pub fn tasks(&mut self, theme: &mut Theme, tasks: &[Task<'_>]) -> Vec<Command> {
        let mut commands: Vec<Command> = self
            .tasks
            .iter()
            .filter(|s| !tasks.iter().any(|t| t.surface == s.as_str()))
            .map(|s| Command::Remove {
                id: self.task_id(s),
            })
            .collect();
        for task in tasks {
            if !self.tasks.iter().any(|s| s == task.surface) {
                commands.push(Command::Create {
                    id: self.task_id(task.surface),
                    kind: NodeKind::Image,
                    parent: self.group(),
                    props: PropsUpdate::default(),
                });
            }
        }
        self.tasks = tasks.iter().map(|t| t.surface.to_string()).collect();
        self.shown
            .retain(|s, _| tasks.iter().any(|t| t.surface == s));
        for (i, task) in tasks.iter().enumerate() {
            let update = self.task(theme, i, task);
            if self.shown.get(task.surface) != Some(&update) {
                self.shown.insert(task.surface.into(), update.clone());
                commands.push(update);
            }
        }
        commands
    }

    fn task(&self, theme: &mut Theme, index: usize, task: &Task<'_>) -> Command {
        let [x, _, w, _] = self.task_rect(index);
        let prefix = match (task.active && !task.minimized, task.minimized, task.hovered) {
            (true, _, true) => "focus-hover",
            (true, _, false) => "focus",
            (false, _, true) => "hover",
            (false, true, false) => "minimized",
            (false, false, false) => "normal",
        };
        let shift = i32::from(task.active);
        let mut picture =
            Picture::new(w, TASK_HEIGHT).frame(Sheet::Tasks, prefix, [0, 0, w, TASK_HEIGHT]);
        let mut left = 8;
        if let Some(icon) = &task.icon {
            picture = picture.file(icon, [7 + shift, 4 + shift, 16, 16]);
            left = 27;
        }
        let label = Text::new(
            task.title,
            Font::Ui,
            WHITE,
            [left + shift, shift, w - left - 6, TASK_HEIGHT],
        );
        update(
            &self.task_id(task.surface),
            PropsUpdate {
                image: Some(theme.render(&picture.text(label))),
                ..at(f64::from(x), f64::from(TASK_TOP))
            },
        )
    }

    /// The rectangle of the n-th task button in the taskbar.
    fn task_rect(&self, index: usize) -> Rect {
        let n = i32::try_from(self.tasks.len().max(1)).unwrap_or(i32::MAX);
        let i = i32::try_from(index).unwrap_or(0);
        let room = self.area[2] - self.tray_width() - PAGER_WIDTH - 6 - TASKS_LEFT;
        let width = ((room - (n - 1) * TASK_GAP) / n).clamp(24, TASK_WIDTH);
        [
            TASKS_LEFT + i * (width + TASK_GAP),
            TASK_TOP,
            width,
            TASK_HEIGHT,
        ]
    }

    /// The task button of a surface in scene coordinates.
    pub fn task_target(&self, surface: &str) -> Option<[f64; 4]> {
        let index = self.tasks.iter().position(|s| s == surface)?;
        let [x, y, w, h] = self.task_rect(index);
        let [ox, oy] = self.origin();
        Some([ox + x, oy + y, w, h].map(f64::from))
    }

    /// Removes the taskbar.
    pub fn remove(&self) -> Command {
        Command::Remove { id: self.group() }
    }
}

/// The width of a notification area with or without the clock and an
/// icon.
fn tray_width(clock: bool, icon: bool) -> i32 {
    let base = if clock { TRAY_WIDTH } else { TRAY_EMPTY };
    base + if icon { TRAY_ICON } else { 0 }
}

/// The notification area of a width: an icon at the left, the clock at
/// the right.
fn tray(width: i32, clock: Option<&str>, icon: Option<&std::path::Path>) -> Picture {
    let mut picture = Picture::new(width, HEIGHT)
        .element(Sheet::Winxp, "tray-left", [0, 0, 2, HEIGHT])
        .element(Sheet::Winxp, "tray-center", [2, 0, width - 2, HEIGHT]);
    if let Some(icon) = icon {
        picture = picture.file(icon, [TRAY_ICON_LEFT, 7, 16, 16]);
    }
    if let Some(clock) = clock {
        let left = width - TRAY_WIDTH;
        picture = picture.text(
            Text::new(
                clock,
                Font::Ui,
                WHITE,
                [left + 2, 0, TRAY_WIDTH - 4, HEIGHT],
            )
            .centered(),
        );
    }
    picture
}

/// The start button in a look: `normal`, `hover` or `pressed`.
fn start_button(theme: &Theme, look: &str) -> Picture {
    let shift = i32::from(look == "pressed");
    Picture::new(START_WIDTH, HEIGHT)
        .element(
            Sheet::Winxp,
            &format!("startbutton-{look}"),
            [0, 0, START_WIDTH, HEIGHT],
        )
        .file(&start_flag(theme), [10, 5 + shift, 20, 20])
        .text(
            Text::new("start", Font::Start, WHITE, [35, 1 + shift, 62, HEIGHT])
                .shadow([0, 20, 0, 180]),
        )
}

/// The previews of Taskbar and Start Menu Properties, as on XP: a piece
/// of the desktop with the taskbar below it, drawn from the same parts.
/// `windows` lists the titles of task buttons, the first one active; the
/// taskbar shows only its strip when `hidden`, and a window covers part
/// of it when it is not `on_top`.
pub fn preview(
    theme: &Theme,
    [w, h]: [i32; 2],
    windows: &[&str],
    [hidden, on_top]: [bool; 2],
    clock: Option<&str>,
) -> Picture {
    let top = theme.size(Sheet::Panel, "top").map_or(6, |s| s[1]);
    let bar_y = if hidden { h - HIDDEN_STRIP } else { h - HEIGHT };
    let mut picture = Picture::new(w, h)
        .svg(format!(
            r##"<rect width="{w}" height="{h}" fill="#5A7EDC"/>"##
        ))
        .element(Sheet::Panel, "top", [0, bar_y, w, top])
        .element(Sheet::Panel, "center", [0, bar_y + top, w, HEIGHT - top]);
    if hidden {
        return picture;
    }
    match clock {
        None => {
            picture = picture.layers_of(start_button(theme, "normal"), [0, bar_y]);
            let mut x = TASKS_LEFT;
            for (i, title) in windows.iter().enumerate() {
                let look = if i == 0 { "focus" } else { "normal" };
                let label = Text::new(
                    title,
                    Font::Ui,
                    WHITE,
                    [x + 8, bar_y + TASK_TOP, TASK_WIDTH - 14, TASK_HEIGHT],
                );
                picture = picture
                    .frame(
                        Sheet::Tasks,
                        look,
                        [x, bar_y + TASK_TOP, TASK_WIDTH, TASK_HEIGHT],
                    )
                    .text(label);
                x += TASK_WIDTH + TASK_GAP;
            }
        }
        Some(clock) => {
            let width = tray_width(!clock.is_empty(), false);
            let clock = (!clock.is_empty()).then_some(clock);
            picture = picture.layers_of(tray(width, clock, None), [w - width, bar_y]);
        }
    }
    if !on_top {
        // A window with XP's blue frame over the taskbar.
        let x = w * 2 / 5;
        picture = picture.svg(format!(
            r##"<rect x="{x}.5" y="2.5" width="{}" height="{}" fill="#fff" stroke="#0831D9" stroke-width="5"/>"##,
            w / 2,
            h - 4
        ));
    }
    picture
}

/// The flag of the start button: XP's own `start_flag.png` of the
/// Windows XP pack, 20 pixels as on XP, when it is installed, else the
/// flag of the theme.
fn start_flag(theme: &Theme) -> PathBuf {
    icons::data_file("winxp-kde/start_flag.png")
        .unwrap_or_else(|| theme.path("winxp/start-flag.svg"))
}

fn update(id: &str, props: PropsUpdate) -> Command {
    Command::Update {
        id: id.into(),
        props,
    }
}

fn at(x: f64, y: f64) -> PropsUpdate {
    PropsUpdate {
        transform: Some(Transform::translate(x, y, 0.0)),
        ..PropsUpdate::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(tasks: usize) -> Bar {
        Bar {
            output: "DP-1".into(),
            area: [0, 0, 1280, 800],
            position: [0, 0],
            icon: None,
            tasks: (0..tasks).map(|i| format!("s{i}")).collect(),
            shown: HashMap::new(),
            auto_hide: false,
            clock: true,
            offset: 0.0,
            target: 0.0,
        }
    }

    #[test]
    fn node_ids_name_their_part() {
        assert_eq!(Part::of("xp-start-DP-1"), Some(Part::Start("DP-1".into())));
        assert_eq!(Part::of("xp-task-s4-DP-1"), Some(Part::Task("s4".into())));
        assert_eq!(Part::of("xp-tray-DP-1"), None);
        assert_eq!(Part::of("xp-pager-2-DP-1"), Some(Part::Pager(2)));
    }

    #[test]
    fn task_buttons_share_the_room() {
        assert_eq!(bar(2).task_rect(1), [106 + 163, 3, 160, 25]);
        let crowded = bar(40);
        let [x, _, w, _] = crowded.task_rect(39);
        assert!(
            x + w <= 1280 - TRAY_WIDTH - PAGER_WIDTH || w == 24,
            "{x} {w}"
        );
        assert_eq!(bar(1).task_target("s0"), Some([106.0, 773.0, 160.0, 25.0]));
    }

    #[test]
    fn a_taskbar_that_hides_itself_leaves_the_work_area_to_the_windows() {
        let mut bar = bar(0);
        assert_eq!(bar.usable(), [0, 0, 1280, 770]);
        bar.configure(Settings {
            auto_hide: true,
            ..Settings::default()
        });
        assert_eq!(bar.usable(), [0, 0, 1280, 798]);
        // It slides down until its strip of 2 pixels shows, in 0.15 s.
        assert!(bar.sliding());
        bar.slide(0.1);
        assert!(bar.sliding());
        bar.slide(0.1);
        assert!(!bar.sliding());
        assert!((bar.offset - 28.0).abs() < 1e-9);
        // The pointer at the bottom edge brings it back.
        bar.hide(false);
        bar.slide(0.2);
        assert!(bar.offset.abs() < 1e-9);
    }

    #[test]
    fn without_the_clock_the_tray_shrinks() {
        let mut bar = bar(0);
        assert!(bar.clock_rect().is_some());
        bar.configure(Settings {
            clock: false,
            ..Settings::default()
        });
        assert_eq!(bar.tray_width(), TRAY_EMPTY);
        assert_eq!(bar.clock_rect(), None);
    }

    #[test]
    fn the_tray_grows_for_an_icon() {
        let mut bar = bar(0);
        assert_eq!(bar.tray_tip(), ([1280 - 82 + 18, 778], 1280));
        bar.icon = Some("info.svg".into());
        assert_eq!(bar.tray_width(), 104);
        assert_eq!(bar.tray_tip().0[0], 1280 - 104 + 18);
    }
}
