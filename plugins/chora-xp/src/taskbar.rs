//! The taskbar at the bottom of the work area of each output, drawn as
//! image nodes: XP's blue bar, the green start button, one button per
//! window and the notification area with the clock.

use std::{collections::HashMap, fmt::Write as _, path::PathBuf};

use chora_scene::{Command, NodeKind, OutputInfo, PropsUpdate, Transform};

use crate::{
    cube::DESKTOPS,
    sheets,
    text::{Font, Text},
    theme::{Picture, Rect, Theme},
};

/// The height of the taskbar.
pub const HEIGHT: i32 = 30;
const START_WIDTH: i32 = 99;
/// The left edge of the first task button.
const TASKS_LEFT: i32 = 105;
const TASK_TOP: i32 = 3;
/// The rows of XP's taskbar at its own height of 30 pixels, top first, as
/// Luna draws it: a light edge, then the blue that the task buttons stand
/// out from. A taller taskbar repeats rows of the blue between the light
/// edge and the dark one.
const LUNA_ROWS: [&str; 30] = [
    "3168d5", "3888e9", "4993e6", "307be5", "286add", "245dd7", "2359d6", "2257d5", "2257d5",
    "2257d6", "2157d7", "2259d7", "225ad9", "235bd9", "235cdb", "245ddb", "245edc", "245ddb",
    "245edc", "255fdc", "2660dd", "2661dd", "2662df", "2662df", "2663e0", "2663e0", "2561de",
    "235bd9", "1e50c4", "1941a5",
];
const TASK_HEIGHT: i32 = 24;
const TASK_GAP: i32 = 2;
const TASK_WIDTH: i32 = 160;
const TRAY_WIDTH: i32 = 72;
/// The notification area without the clock.
const TRAY_EMPTY: i32 = 8;
/// The helper of the notification area, at its left: 12 pixels of room
/// before the 16 pixel icon and 4 after. A click opens the chat.
pub const HELPER: i32 = 32;
/// The helper, a small paperclip, drawn on a grid of 16 pixels.
const HELPER_SHAPE: &str = r##"<path d="M6.5 5.5V10.5a1.75 1.75 0 0 0 3.5 0V4.25a3.5 3.5 0 0 0-7 0V10a5 5 0 0 0 10 0V5" stroke="#ffffff" fill="none" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>"##;
/// The speaker of the notification area, left of the clock: 4 pixels
/// of room before the 16 pixel icon and 4 after, so that the icons stand
/// 8 pixels apart. A click mutes or unmutes. It keeps its width: the level
/// and the word "Muted" are in its tooltip.
pub const SPEAKER: i32 = 24;
/// The speaker on a grid of 16 pixels.
const SPEAKER_SHAPE: &str = r##"<path d="M1.5 5.75H5L9 2.25v11.5L5 10.25H1.5z" fill="#ffffff"/>"##;
/// The waves of a speaker that plays.
const SPEAKER_WAVE: &str = r##"<path d="M11.5 5.5a3.5 3.5 0 0 1 0 5M13.25 3.5a6.25 6.25 0 0 1 0 9" stroke="#ffffff" fill="none" stroke-width="1.5"/>"##;
/// The red no-sign over a speaker that is muted or has no output, round
/// the middle of its grid of 16 pixels.
const NO_SIGN: &str = r##"<circle cx="8" cy="8" r="6.5" fill="none" stroke="#D8321F" stroke-width="2"/><path d="M3.4 3.4L12.6 12.6" stroke="#D8321F" stroke-width="2"/>"##;
/// The red no-sign as a badge at the lower right corner of a speaker, so
/// that its cone stays whole.
const NO_BADGE: &str = r##"<circle cx="12" cy="10.8" r="3.2" fill="none" stroke="#D8321F" stroke-width="1.6"/><path d="M9.8 8.6L14.2 13" stroke="#D8321F" stroke-width="1.6"/>"##;
/// The room before the first icon of the notification area and after the
/// clock.
const TRAY_PAD: i32 = 12;
/// How far the measured width of the clock runs beyond its last column of
/// ink.
const CLOCK_BEARING: i32 = 2;
/// The side of an icon of the notification area.
const TRAY_ICON: i32 = 16;
/// What shows of a taskbar that hides itself, as on XP.
pub const HIDDEN_STRIP: i32 = 2;
/// How long the taskbar takes to slide away or back.
const SLIDE_TIME: f64 = 0.15;
/// The buttons of the desktops, left of the tray, as in the virtual
/// desktop manager of the power toys for XP.
const PAGER_BUTTON: i32 = 22;
const PAGER_TOP: i32 = (HEIGHT - PAGER_BUTTON) / 2;
const PAGER_GAP: i32 = 2;
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)] // four
const PAGER_WIDTH: i32 = DESKTOPS as i32 * PAGER_BUTTON + (DESKTOPS as i32 - 1) * PAGER_GAP + 6;
const WHITE: [u8; 4] = [255, 255, 255, 255];

/// The settings of Taskbar and Start Menu Properties.
#[allow(clippy::struct_excessive_bools)] // independent check boxes
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
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

/// What shows an output: a screen, or the client that added a virtual
/// output.
enum Kind {
    Screen,
    Virtual { owner: Option<String> },
}

impl Kind {
    fn of(output: &OutputInfo) -> Self {
        if output.is_virtual == Some(true) {
            Self::Virtual {
                owner: output.owner.clone(),
            }
        } else {
            Self::Screen
        }
    }
}

/// A window as its task button shows it.
#[derive(Clone)]
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
    /// The speaker of the notification area of an output.
    Speaker(String),
    /// The helper of the notification area of an output.
    Helper(String),
}

/// What the icons of the notification area show: the speaker and the
/// helper, which shows the red no-sign while the microphone is off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Icons {
    /// The speaker.
    pub speaker: Speaker,
    /// The microphone of the helper is off.
    pub mic_off: bool,
}

/// What the speaker of the notification area shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Speaker {
    /// Sound plays.
    On,
    /// Muted, or no output device: the red no-sign.
    Off,
}

impl Part {
    /// The part that a node id names.
    pub fn of(node: &str) -> Option<Self> {
        if let Some(output) = node.strip_prefix("xp-start-") {
            return Some(Self::Start(output.into()));
        }
        if let Some(output) = node.strip_prefix("xp-speaker-") {
            return Some(Self::Speaker(output.into()));
        }
        if let Some(output) = node.strip_prefix("xp-helper-") {
            return Some(Self::Helper(output.into()));
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
    /// The taskbar hides itself when the pointer leaves it.
    auto_hide: bool,
    /// The clock shows.
    clock: bool,
    /// The buttons of the desktops show: not on a virtual output, where
    /// the plugin has no cube of its own.
    pager: bool,
    /// What shows the output.
    kind: Kind,
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
            tasks: Vec::new(),
            shown: HashMap::new(),
            auto_hide: false,
            clock: true,
            pager: output.is_virtual != Some(true),
            kind: Kind::of(output),
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
            "xp-speaker-",
            "xp-helper-",
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

    /// Whether the output has another area than the taskbar knows, for
    /// example after a change that came before the events were wanted.
    pub fn is_stale(&self, output: &OutputInfo) -> bool {
        self.output == output.name && self.area != output.layout_area()
    }

    /// The part of the work area above the taskbar, or above its strip
    /// when it hides itself.
    pub fn usable(&self) -> Rect {
        let [x, y, w, h] = self.area;
        let bar = if self.auto_hide { HIDDEN_STRIP } else { HEIGHT };
        [x, y, w, (h - bar).max(1)]
    }

    /// The height that the taskbar reserves at the bottom of its output:
    /// its own, its strip when it hides itself, none on a virtual output.
    pub fn reserve(&self) -> i32 {
        match (self.is_virtual(), self.auto_hide) {
            (true, _) => 0,
            (false, true) => HIDDEN_STRIP,
            (false, false) => HEIGHT,
        }
    }

    /// Whether the taskbar shows the buttons of the desktops.
    pub fn has_pager(&self) -> bool {
        self.pager
    }

    /// Whether the output is a virtual one, which a client shows.
    pub fn is_virtual(&self) -> bool {
        matches!(self.kind, Kind::Virtual { .. })
    }

    /// The client that added the virtual output.
    pub fn owner(&self) -> Option<&str> {
        match &self.kind {
            Kind::Virtual { owner } => owner.as_deref(),
            Kind::Screen => None,
        }
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
        (clock, icons): (&str, Icons),
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
            image(self.speaker_id()),
            image(self.helper_id()),
        ];
        commands.extend((0..DESKTOPS).map(|d| image(self.pager_id(d))));
        commands.extend(self.layout(theme));
        commands.push(self.start(theme, "normal"));
        commands.extend(self.tray(theme, clock, icons));
        commands.extend(self.pager(theme, desktop, None));
        commands
    }

    /// Moves the taskbar to the new work area of its output.
    pub fn resize(
        &mut self,
        theme: &mut Theme,
        output: &OutputInfo,
        (clock, icons): (&str, Icons),
    ) -> Vec<Command> {
        self.area = output.layout_area();
        let mut commands = self.layout(theme);
        commands.extend(self.tray(theme, clock, icons));
        commands
    }

    /// The group at the bottom of the work area and the background over
    /// its width, XP's rows of blue.
    fn layout(&self, theme: &mut Theme) -> Vec<Command> {
        let [x, y] = self.origin().map(f64::from);
        let picture = Picture::new(32, HEIGHT).svg(rows(32, 0));
        vec![
            update(&self.group(), at(x, y + self.offset)),
            update(
                &format!("xp-panel-{}", self.output),
                PropsUpdate {
                    image: Some(theme.render_flat(&picture)),
                    size: Some([f64::from(self.area[2]), f64::from(HEIGHT)]),
                    slice: Some([0.0; 4]),
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

    fn speaker_id(&self) -> String {
        format!("xp-speaker-{}", self.output)
    }

    fn helper_id(&self) -> String {
        format!("xp-helper-{}", self.output)
    }

    /// The button of a desktop in scene coordinates.
    pub fn pager_rect(&self, desktop: usize) -> Rect {
        let [left, top, width, height] = self.area;
        let first = left + width - self.tray_width() - PAGER_WIDTH;
        let nth = i32::try_from(desktop).unwrap_or(0);
        [
            first + nth * (PAGER_BUTTON + PAGER_GAP),
            top + height - HEIGHT + PAGER_TOP,
            PAGER_BUTTON,
            PAGER_BUTTON,
        ]
    }

    fn pager_id(&self, desktop: usize) -> String {
        format!("xp-pager-{desktop}-{}", self.output)
    }

    /// The buttons of the desktops, the current one down, and one maybe
    /// under the pointer.
    pub fn pager(&self, theme: &mut Theme, current: usize, hovered: Option<usize>) -> Vec<Command> {
        let left = self.area[2] - self.tray_width() - PAGER_WIDTH;
        let visible = Some(self.pager);
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
                    [0, 0, PAGER_BUTTON, PAGER_BUTTON],
                )
                .centered();
                let picture = Picture::new(PAGER_BUTTON, PAGER_BUTTON)
                    .frame(sheets::TASKS, prefix, [0, 0, PAGER_BUTTON, PAGER_BUTTON])
                    .text(label);
                update(
                    &self.pager_id(desktop),
                    PropsUpdate {
                        image: Some(theme.render(&picture)),
                        visible,
                        ..at(
                            f64::from(left + i * (PAGER_BUTTON + PAGER_GAP)),
                            f64::from(PAGER_TOP),
                        )
                    },
                )
            })
            .collect()
    }

    /// The width of the notification area.
    fn tray_width(&self) -> i32 {
        tray_width(self.clock)
    }

    /// The clock in scene coordinates, while it shows: a double click
    /// there opens Date and Time.
    pub fn clock_rect(&self) -> Option<Rect> {
        let [x, y, w, h] = self.area;
        self.clock
            .then(|| [x + w - TRAY_WIDTH, y + h - HEIGHT, TRAY_WIDTH, HEIGHT])
    }

    /// The speaker in scene coordinates: a click there mutes or unmutes.
    pub fn speaker_rect(&self) -> Rect {
        let [x, y, w, h] = self.area;
        [
            x + w - self.tray_width() + HELPER,
            y + h - HEIGHT,
            SPEAKER,
            HEIGHT,
        ]
    }

    /// The helper in scene coordinates: a click there opens the chat.
    pub fn helper_rect(&self) -> Rect {
        let [x, y, w, h] = self.area;
        [x + w - self.tray_width(), y + h - HEIGHT, HELPER, HEIGHT]
    }

    /// The notification area with the speaker and the clock, at the right
    /// edge.
    pub fn tray(&self, theme: &mut Theme, clock: &str, icons: Icons) -> Vec<Command> {
        let width = self.tray_width();
        let clock = self.clock.then_some(clock);
        let picture = tray(theme, width, clock, icons);
        let blank = Picture::new(SPEAKER, HEIGHT);
        let helper = Picture::new(HELPER, HEIGHT);
        let left = f64::from(self.area[2] - width);
        vec![
            update(
                &format!("xp-tray-{}", self.output),
                PropsUpdate {
                    image: Some(theme.render(&picture)),
                    meta: Some([("text".to_owned(), clock.unwrap_or_default().to_owned())].into()),
                    ..at(left, 0.0)
                },
            ),
            update(
                &self.speaker_id(),
                PropsUpdate {
                    image: Some(theme.render(&blank)),
                    ..at(left + f64::from(HELPER), 0.0)
                },
            ),
            update(
                &self.helper_id(),
                PropsUpdate {
                    image: Some(theme.render(&helper)),
                    ..at(left, 0.0)
                },
            ),
        ]
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
                to: None,
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
        let picture = task_picture(w, task);
        update(
            &self.task_id(task.surface),
            PropsUpdate {
                image: Some(theme.render(&picture)),
                ..at(f64::from(x), f64::from(TASK_TOP))
            },
        )
    }

    /// The rectangle of the n-th task button in the taskbar.
    fn task_rect(&self, index: usize) -> Rect {
        let n = i32::try_from(self.tasks.len().max(1)).unwrap_or(i32::MAX);
        let i = i32::try_from(index).unwrap_or(0);
        let pager = if self.pager { PAGER_WIDTH } else { 0 };
        let room = self.area[2] - self.tray_width() - pager - 6 - TASKS_LEFT;
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
        Command::Remove {
            id: self.group(),
            to: None,
        }
    }
}

/// The width of a notification area with or without the clock.
fn tray_width(clock: bool) -> i32 {
    HELPER + SPEAKER + if clock { TRAY_WIDTH } else { TRAY_EMPTY }
}

/// The notification area of a width, with the helper and the speaker at the
/// left and the clock at the right: 12 pixels at both ends, 8 between the
/// icons and the clock.
fn tray(theme: &Theme, width: i32, clock: Option<&str>, icons: Icons) -> Picture {
    let mut picture = Picture::new(width, HEIGHT)
        .element(sheets::WINXP, "tray-left", [0, 0, 2, HEIGHT])
        .element(sheets::WINXP, "tray-center", [2, 0, width - 2, HEIGHT]);
    let speaker = match icons.speaker {
        Speaker::On => format!("{SPEAKER_SHAPE}{SPEAKER_WAVE}"),
        Speaker::Off => format!("{SPEAKER_SHAPE}{NO_BADGE}"),
    };
    let helper = if icons.mic_off {
        format!("{HELPER_SHAPE}{NO_SIGN}")
    } else {
        HELPER_SHAPE.to_owned()
    };
    let top = (HEIGHT - TRAY_ICON) / 2;
    let speaker_left = HELPER + (SPEAKER - TRAY_ICON) / 2;
    picture = picture.svg(format!(
        r#"<g transform="translate({TRAY_PAD} {top})">{helper}</g><g transform="translate({speaker_left} {top})">{speaker}</g>"#
    ));
    if let Some(clock) = clock {
        #[allow(clippy::cast_possible_truncation)]
        let ink = theme.text_width(clock, Font::Ui).ceil() as i32;
        let left = width - TRAY_PAD - ink + CLOCK_BEARING;
        picture = picture.text(Text::new(
            clock,
            Font::Ui,
            WHITE,
            [left, 0, ink + 2, HEIGHT],
        ));
    }
    picture
}

/// The picture of a task button of a width: the frame of its look, the
/// icon and the title on one centre line.
fn task_picture(w: i32, task: &Task<'_>) -> Picture {
    let prefix = match (task.active && !task.minimized, task.minimized, task.hovered) {
        (true, _, true) => "focus-hover",
        (true, _, false) => "focus",
        (false, _, true) => "hover",
        (false, true, false) => "minimized",
        (false, false, false) => "normal",
    };
    let shift = i32::from(task.active && !task.minimized);
    let mut picture =
        Picture::new(w, TASK_HEIGHT).frame(sheets::TASKS, prefix, [0, 0, w, TASK_HEIGHT]);
    let mut left = 8;
    if let Some(icon) = &task.icon {
        picture = picture.file(icon, [5 + shift, (TASK_HEIGHT - 16) / 2 + shift, 16, 16]);
        left = 25;
    }
    picture.text(Text::new(
        task.title,
        Font::Ui,
        WHITE,
        [left + shift, shift, w - left - 6, TASK_HEIGHT],
    ))
}

/// The start button in a look: `normal`, `hover` or `pressed`.
fn start_button(theme: &Theme, look: &str) -> Picture {
    let shift = i32::from(look == "pressed");
    Picture::new(START_WIDTH, HEIGHT)
        .element(
            sheets::WINXP,
            &format!("startbutton-{look}"),
            [0, 0, START_WIDTH, HEIGHT],
        )
        .file(&start_mark(theme), [9, 5 + shift, 20, 20])
        .text(
            Text::new("start", Font::Start, WHITE, [33, shift, 64, HEIGHT]).shadow([0, 20, 0, 180]),
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
    let bar_y = if hidden { h - HIDDEN_STRIP } else { h - HEIGHT };
    let mut picture = Picture::new(w, h)
        .svg(format!(
            r##"<rect width="{w}" height="{h}" fill="#5A7EDC"/>"##
        ))
        .svg(rows(w, bar_y));
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
                        sheets::TASKS,
                        look,
                        [x, bar_y + TASK_TOP, TASK_WIDTH, TASK_HEIGHT],
                    )
                    .text(label);
                x += TASK_WIDTH + TASK_GAP;
            }
        }
        Some(clock) => {
            let width = tray_width(!clock.is_empty());
            let clock = (!clock.is_empty()).then_some(clock);
            let tray = tray(
                theme,
                width,
                clock,
                Icons {
                    speaker: Speaker::On,
                    mic_off: false,
                },
            );
            picture = picture.layers_of(tray, [w - width, bar_y]);
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

/// The mark of the start button: the imported mark when the assets folder
/// has one, else the own mark of chora, one color.
fn start_mark(theme: &Theme) -> PathBuf {
    chora_kit::xdg::xp_assets()
        .map(|d| d.join("start-mark.png"))
        .filter(|p| p.is_file())
        .unwrap_or_else(|| theme.path("winxp/start-mark.svg"))
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

/// The rows of the taskbar at its height: the five light rows at the top
/// and the two dark ones at the bottom of Luna's 30, and the blue between
/// them stretched over the rest.
fn luna_rows() -> Vec<&'static str> {
    let (top, bottom) = (5, 2);
    let middle = LUNA_ROWS.len() - top - bottom;
    let room = usize::try_from(HEIGHT).unwrap_or(0) - top - bottom;
    (0..usize::try_from(HEIGHT).unwrap_or(0))
        .map(|y| match y {
            y if y < top => LUNA_ROWS[y],
            y if y >= top + room => LUNA_ROWS[LUNA_ROWS.len() - (top + room + bottom - y)],
            y => LUNA_ROWS[top + (y - top) * middle / room],
        })
        .collect()
}

/// XP's rows of the taskbar as SVG, `width` wide from `top` down.
fn rows(width: i32, top: i32) -> String {
    luna_rows()
        .into_iter()
        .zip(top..)
        .fold(String::new(), |mut out, (color, y)| {
            let _ = write!(
                out,
                r##"<rect y="{y}" width="{width}" height="1" fill="#{color}"/>"##
            );
            out
        })
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
            auto_hide: false,
            clock: true,
            pager: true,
            kind: Kind::Screen,
            offset: 0.0,
            target: 0.0,
        }
    }

    #[test]
    fn a_bar_is_stale_when_its_output_has_another_area() {
        let mut output = OutputInfo {
            name: "DP-1".into(),
            position: [0, 0],
            size: [2560, 1600],
            scale: 1.0,
            camera: Transform::default(),
            work_area: [0, 0, 2560, 1600],
            default_work_area: [0, 0, 2560, 1600],
            is_virtual: None,
            owner: None,
        };
        let bar = bar(0);
        assert!(bar.is_stale(&output));
        output.work_area = [0, 0, 1280, 800];
        output.default_work_area = [0, 0, 1280, 800];
        assert!(!bar.is_stale(&output));
        output.name = "DP-2".into();
        output.work_area = [0, 0, 1, 1];
        output.default_work_area = [0, 0, 1, 1];
        assert!(!bar.is_stale(&output));
    }

    #[test]
    fn the_icon_and_the_title_of_a_task_button_share_a_centre_line() {
        for (active, minimized) in [(false, false), (true, false), (false, true)] {
            let task = Task {
                surface: "s1",
                title: "foot",
                icon: Some("/i/terminal.png".into()),
                active,
                minimized,
                hovered: false,
            };
            let picture = task_picture(180, &task);
            let mut centres = Vec::new();
            for layer in &picture.layers {
                match layer {
                    chora_draw::theme::Layer::File { rect, .. } => {
                        centres.push(rect[1] * 2 + rect[3]);
                        assert!(
                            sheets::tests::ICON_SIZES.contains(&rect[2]),
                            "icon slot {rect:?}"
                        );
                    }
                    chora_draw::theme::Layer::Text(text) => {
                        centres.push(text.rect[1] * 2 + text.rect[3]);
                    }
                    _ => {}
                }
            }
            assert_eq!(centres.len(), 2);
            assert_eq!(
                centres[0], centres[1],
                "active {active} minimized {minimized}"
            );
        }
    }

    #[test]
    fn the_start_button_draws_no_flag() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flavors/xp/theme");
        let cache = std::env::temp_dir().join(format!("chora-xp-start-{}", std::process::id()));
        let theme =
            Theme::load(&dir, &cache, &sheets::ALL, sheets::FAMILY).expect("the theme loads");
        for look in ["normal", "hover", "pressed"] {
            let picture = start_button(&theme, look);
            for layer in &picture.layers {
                if let chora_draw::theme::Layer::File { path, .. } = layer {
                    let path = path.to_string_lossy().to_lowercase();
                    assert!(!path.contains("flag"), "{look}: {path}");
                }
            }
        }
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn no_flag_asset_ships_with_the_xp_flavor() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flavors/xp");
        let mut dirs = vec![root];
        while let Some(dir) = dirs.pop() {
            for entry in std::fs::read_dir(&dir).unwrap().flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_lowercase();
                assert!(!name.contains("flag"), "{}", path.display());
                if path.is_dir() {
                    dirs.push(path);
                }
            }
        }
    }

    #[test]
    fn the_sources_of_the_plugin_name_no_flag_asset() {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        for entry in std::fs::read_dir(src).unwrap().flatten() {
            let Ok(text) = std::fs::read_to_string(entry.path()) else {
                continue;
            };
            for pattern in ["start-", "start_"] {
                let pattern = format!("{pattern}{}", "flag");
                let hits = text
                    .lines()
                    .filter(|l| !l.trim_start().starts_with("//") && l.contains(&pattern));
                assert_eq!(
                    hits.count(),
                    0,
                    "{} names {pattern}",
                    entry.path().display()
                );
            }
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
        assert_eq!(bar(2).task_rect(1), [105 + 162, 3, 160, 24]);
        let crowded = bar(40);
        let [x, _, w, _] = crowded.task_rect(39);
        assert!(
            x + w <= 1280 - TRAY_WIDTH - PAGER_WIDTH || w == 24,
            "{x} {w}"
        );
        assert_eq!(bar(1).task_target("s0"), Some([105.0, 773.0, 160.0, 24.0]));
    }

    #[test]
    fn a_taskbar_that_hides_itself_leaves_the_work_area_to_the_windows() {
        let mut bar = bar(0);
        assert_eq!(bar.usable(), [0, 0, 1280, 770]);
        bar.configure(Settings {
            auto_hide: true,
            ..crate::config::defaults().taskbar
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

    fn icons(speaker: Speaker, mic_off: bool) -> Icons {
        Icons { speaker, mic_off }
    }

    fn texts(picture: &Picture) -> Vec<String> {
        picture
            .layers
            .iter()
            .filter_map(|l| match l {
                chora_draw::theme::Layer::Text(t) => Some(t.line.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_tray_has_an_icon_for_the_speaker_and_a_clock() {
        let (theme, cache) = crate::sheets::tests::theme("tray-texts");
        let width = tray_width(true);
        assert_eq!(
            texts(&tray(
                &theme,
                width,
                Some("6:46 AM"),
                icons(Speaker::On, false)
            )),
            ["6:46 AM"]
        );
        assert_eq!(
            texts(&tray(
                &theme,
                tray_width(false),
                None,
                icons(Speaker::Off, false)
            ))
            .len(),
            0
        );
        let _ = std::fs::remove_dir_all(cache);
    }

    fn light(c: [u8; 3]) -> bool {
        c.iter().all(|v| *v > 120)
    }

    fn red(c: [u8; 3]) -> bool {
        c[0] > 140 && c[1] < 120 && c[2] < 120
    }

    fn tray_runs(clock: &str, icons: Icons, step: i32) -> (Vec<crate::sheets::tests::Ink>, i32) {
        let (mut theme, cache) = crate::sheets::tests::theme(&format!("tray-ink-{step}"));
        theme.set_scale(f64::from(step));
        let width = tray_width(true);
        let picture = tray(&theme, width, Some(clock), icons);
        let image = crate::sheets::tests::draw(&mut theme, &picture);
        let found = crate::sheets::tests::runs(&image, 7 * step, |c| light(c) || red(c));
        let _ = std::fs::remove_dir_all(cache);
        (
            found
                .into_iter()
                .map(|r| crate::sheets::tests::Ink {
                    left: r.left / step,
                    top: r.top / step,
                    right: (r.right + step - 1) / step,
                    bottom: (r.bottom + step - 1) / step,
                    count: r.count / (step * step),
                })
                .collect(),
            width,
        )
    }

    fn check_tray_grid(clock: &str, step: i32) {
        let (runs, width) = tray_runs(clock, icons(Speaker::On, false), step);
        assert_eq!(runs.len(), 3, "{runs:?} at scale {step}");
        let [helper, speaker, text] = [runs[0], runs[1], runs[2]];
        for icon in [helper, speaker] {
            assert!(icon.width() <= 16 && icon.height() <= 16, "{icon:?}");
            assert!(
                (icon.top + icon.bottom - HEIGHT).abs() <= 1,
                "{icon:?} is not on the middle of the bar"
            );
        }
        assert!(
            (helper.left + helper.right - 2 * TRAY_PAD - 16).abs() <= 1,
            "{helper:?}"
        );
        assert!(
            ((speaker.left + speaker.right) - (helper.left + helper.right) - 48).abs() <= 2,
            "{helper:?} {speaker:?}"
        );
        assert!(text.left - speaker.right >= 8, "{speaker:?} {text:?}");
        assert!(
            (width - TRAY_PAD - text.right).abs() <= 1,
            "the clock ends {} px from the edge at scale {step}",
            width - text.right
        );
    }

    #[test]
    fn the_tray_icons_sit_on_a_grid_of_16_with_12_pixels_at_the_ends_and_8_between() {
        for step in [1, 2] {
            for clock in ["3:26 AM", "12:59 PM"] {
                check_tray_grid(clock, step);
            }
        }
    }

    #[test]
    fn the_speaker_has_ink_and_the_no_sign_is_the_palette_red() {
        let (runs, _) = tray_runs("3:26 AM", icons(Speaker::On, false), 1);
        let speaker = runs[1];
        assert!(
            speaker.count * 10 >= speaker.width() * speaker.height() * 3,
            "{speaker:?} is a stub"
        );
        assert!(speaker.width() >= 12, "{speaker:?}");
        let (runs, _) = tray_runs("3:26 AM", icons(Speaker::Off, true), 1);
        for (icon, middle, slack) in [(runs[0], TRAY_PAD + 8, 1), (runs[1], HELPER + 12, 3)] {
            assert!((14..=16).contains(&icon.width()), "{icon:?}");
            assert!((12..=16).contains(&icon.height()), "{icon:?}");
            assert!(
                (icon.left + icon.right - 2 * middle).abs() <= slack,
                "{icon:?}"
            );
            assert!((icon.top + icon.bottom - HEIGHT).abs() <= slack, "{icon:?}");
        }
        let (mut theme, cache) = crate::sheets::tests::theme("tray-red");
        let picture = tray(&theme, tray_width(true), None, icons(Speaker::Off, false));
        let image = crate::sheets::tests::draw(&mut theme, &picture);
        let reds: Vec<[u8; 3]> = (0..i32::try_from(image.width()).expect("w"))
            .flat_map(|x| (0..HEIGHT).map(move |y| [x, y]))
            .map(|[x, y]| crate::sheets::tests::rgb(&image, x, y))
            .filter(|c| red(*c))
            .collect();
        let purest = reds.iter().min_by_key(|c| c[1]).expect("a red");
        let palette = [0xD8, 0x32, 0x1F];
        assert!(
            purest.iter().zip(palette).all(|(a, b)| a.abs_diff(b) <= 12),
            "the red of the no-sign is {purest:?}, not #D8321F"
        );
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn a_muted_speaker_keeps_its_cone_and_wears_the_no_sign_as_a_badge() {
        let (mut theme, cache) = crate::sheets::tests::theme("tray-cone");
        let mut cone = |speaker| {
            let picture = tray(&theme, tray_width(true), None, icons(speaker, false));
            let image = crate::sheets::tests::draw(&mut theme, &picture);
            let left = HELPER + (SPEAKER - TRAY_ICON) / 2;
            let top = (HEIGHT - TRAY_ICON) / 2;
            (left..left + 9)
                .flat_map(|x| (top..top + TRAY_ICON).map(move |y| [x, y]))
                .filter(|[x, y]| {
                    let [r, g, b] = crate::sheets::tests::rgb(&image, *x, *y);
                    r > 235 && g > 235 && b > 235
                })
                .count()
        };
        let (on, off) = (cone(Speaker::On), cone(Speaker::Off));
        let _ = std::fs::remove_dir_all(cache);
        assert!(on >= 40, "the cone has {on} white pixels");
        assert!(
            off * 10 >= on * 8,
            "the no-sign cuts the cone: {off} of {on}"
        );
    }

    #[test]
    fn the_tray_has_the_same_width_muted_or_not() {
        let (theme, cache) = crate::sheets::tests::theme("tray-width");
        let width = tray_width(true);
        let on = tray(&theme, width, Some("6:46 AM"), icons(Speaker::On, false));
        let off = tray(&theme, width, Some("6:46 AM"), icons(Speaker::Off, true));
        let _ = std::fs::remove_dir_all(cache);
        assert_eq!(on.size, off.size);
        let bar = bar(0);
        let [x, y, w, h] = bar.speaker_rect();
        assert_eq!([y, h, w], [800 - HEIGHT, HEIGHT, SPEAKER]);
        assert_eq!(x + w, bar.clock_rect().expect("a clock")[0]);
        assert!(w >= 24, "a click target of 24 px");
    }

    #[test]
    fn the_helper_stands_left_of_the_speaker_in_the_tray_with_12_px_of_room() {
        let bar = bar(0);
        let [hx, hy, hw, hh] = bar.helper_rect();
        let [sx, sy, sw, sh] = bar.speaker_rect();
        assert_eq!([hy, hh, hw], [800 - HEIGHT, HEIGHT, HELPER]);
        assert_eq!(hx + hw, sx);
        assert_eq!([sy, sh, sw], [hy, hh, SPEAKER]);
        assert!(hw >= 24, "a click target of 24 px");
        assert_eq!(hx, 1280 - bar.tray_width());
        assert_eq!(
            Part::of("xp-helper-DP-1"),
            Some(Part::Helper("DP-1".into()))
        );
    }

    #[test]
    fn the_helper_shows_the_red_no_sign_while_the_microphone_is_off_at_the_same_width() {
        let (theme, cache) = crate::sheets::tests::theme("tray-mic");
        let width = tray_width(true);
        let on = tray(&theme, width, Some("6:46 AM"), icons(Speaker::On, false));
        let off = tray(&theme, width, Some("6:46 AM"), icons(Speaker::On, true));
        let _ = std::fs::remove_dir_all(cache);
        assert_eq!(on.size, off.size);
        assert_ne!(on, off);
    }

    #[test]
    fn the_desktop_buttons_are_22_by_22_two_apart() {
        let bar = bar(0);
        let rects: Vec<Rect> = (0..DESKTOPS).map(|d| bar.pager_rect(d)).collect();
        for [_, y, w, h] in &rects {
            assert_eq!([*w, *h], [22, 22]);
            assert_eq!(*y, 800 - HEIGHT + 4);
        }
        for pair in rects.windows(2) {
            assert_eq!(pair[0][0] + 22 + 2, pair[1][0]);
        }
        assert!(rects[DESKTOPS - 1][0] + 22 <= 1280 - bar.tray_width());
    }

    #[test]
    fn a_minimized_window_never_has_the_pressed_look_of_the_active_one() {
        let (mut theme, cache) = crate::dialog::tests::theme("taskbar-minimized");
        let bar = bar(1);
        let task = |active, minimized| Task {
            surface: "s0",
            title: "Notes",
            icon: None,
            active,
            minimized,
            hovered: false,
        };
        let mut image = |t: Task<'_>| match bar.task(&mut theme, 0, &t) {
            Command::Update { props, .. } => props.image,
            other => panic!("{other:?}"),
        };
        assert_eq!(image(task(true, true)), image(task(false, true)));
        assert_ne!(image(task(true, false)), image(task(false, false)));
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn the_taskbar_is_30_high_with_24_high_task_buttons_160_wide_and_2_apart() {
        assert_eq!(HEIGHT, 30);
        let bar = bar(2);
        let [x0, y0, w0, h0] = bar.task_rect(0);
        let [x1, ..] = bar.task_rect(1);
        assert_eq!([y0, w0, h0], [3, 160, 24]);
        assert_eq!(x1, x0 + w0 + 2);
        assert_eq!(luna_rows().len(), 30);
        assert_eq!(luna_rows()[0], LUNA_ROWS[0]);
        assert_eq!(luna_rows()[29], LUNA_ROWS[29]);
        assert_eq!(luna_rows()[28], LUNA_ROWS[28]);
    }

    #[test]
    fn without_the_clock_the_tray_shrinks() {
        let mut bar = bar(0);
        assert!(bar.clock_rect().is_some());
        bar.configure(Settings {
            clock: false,
            ..crate::config::defaults().taskbar
        });
        assert_eq!(bar.tray_width(), TRAY_EMPTY + SPEAKER + HELPER);
        assert_eq!(bar.clock_rect(), None);
    }
}
