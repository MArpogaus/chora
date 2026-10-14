//! The taskbar at the bottom of the work area of each output, drawn as
//! image nodes: XP's blue bar, the green start button, one button per
//! window and the notification area with the clock.

use std::{collections::HashMap, path::PathBuf};

use chora_scene::{Command, NodeKind, OutputInfo, PropsUpdate, Transform};

use crate::{
    cube::DESKTOPS,
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
/// The buttons of the desktops, left of the tray, as in the virtual
/// desktop manager of the power toys for XP.
const PAGER_BUTTON: i32 = 22;
const PAGER_GAP: i32 = 2;
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)] // four
const PAGER_WIDTH: i32 = DESKTOPS as i32 * PAGER_BUTTON + (DESKTOPS as i32 - 1) * PAGER_GAP + 6;
const WHITE: [u8; 4] = [255, 255, 255, 255];

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
    /// The surfaces with a button, from the left.
    tasks: Vec<String>,
    /// The last update of each task button, to skip repeats.
    shown: HashMap<String, Command>,
}

impl Bar {
    /// The taskbar of an output.
    pub fn new(output: &OutputInfo) -> Self {
        Self {
            output: output.name.clone(),
            area: output.layout_area(),
            tasks: Vec::new(),
            shown: HashMap::new(),
        }
    }

    fn group(&self) -> String {
        format!("xp-bar-{}", self.output)
    }

    fn task_id(&self, surface: &str) -> String {
        format!("xp-task-{surface}-{}", self.output)
    }

    /// The work area of the output, with the taskbar.
    pub fn area(&self) -> Rect {
        self.area
    }

    /// The part of the work area above the taskbar.
    pub fn usable(&self) -> Rect {
        let [x, y, w, h] = self.area;
        [x, y, w, (h - HEIGHT).max(1)]
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

    /// Moves the taskbar to a new work area.
    pub fn resize(&mut self, theme: &mut Theme, area: Rect, clock: &str) -> Vec<Command> {
        self.area = area;
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
            update(&self.group(), at(x, y)),
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
        let shift = i32::from(look == "pressed");
        let picture = Picture::new(START_WIDTH, HEIGHT)
            .element(
                Sheet::Winxp,
                &format!("startbutton-{look}"),
                [0, 0, START_WIDTH, HEIGHT],
            )
            .file(&theme.path("winxp/start-flag.svg"), [10, 5 + shift, 20, 20])
            .text(
                Text::new("start", Font::Start, WHITE, [35, 1 + shift, 62, HEIGHT])
                    .shadow([0, 20, 0, 180]),
            );
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
        let left = self.area[2] - TRAY_WIDTH - PAGER_WIDTH;
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

    /// The notification area with the clock, at the right edge.
    pub fn tray(&self, theme: &mut Theme, clock: &str) -> Command {
        let picture = Picture::new(TRAY_WIDTH, HEIGHT)
            .element(Sheet::Winxp, "tray-left", [0, 0, 2, HEIGHT])
            .element(Sheet::Winxp, "tray-center", [2, 0, TRAY_WIDTH - 2, HEIGHT])
            .text(Text::new(clock, Font::Ui, WHITE, [2, 0, TRAY_WIDTH - 4, HEIGHT]).centered());
        update(
            &format!("xp-tray-{}", self.output),
            PropsUpdate {
                image: Some(theme.render(&picture)),
                ..at(f64::from(self.area[2] - TRAY_WIDTH), 0.0)
            },
        )
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
        let room = self.area[2] - TRAY_WIDTH - PAGER_WIDTH - 6 - TASKS_LEFT;
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
            tasks: (0..tasks).map(|i| format!("s{i}")).collect(),
            shown: HashMap::new(),
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
}
