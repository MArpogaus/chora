//! XP's balloon tips above the notification area: a pale yellow rounded
//! rectangle with a black border and a tail towards the tray, an icon, a
//! bold title, the wrapped text and a close button. One balloon shows at a
//! time; the others wait in a queue.

use std::{
    collections::VecDeque,
    fmt::Write as _,
    path::Path,
    time::{Duration, Instant},
};

use chora_scene::{Command, InputMode, NodeKind, PropsUpdate, Transform};

use crate::{
    notify::{Note, Reason, Signal},
    taskbar::Bar,
    text::{Font, Text},
    theme::{Picture, Rect, Theme},
};

/// The widest line of the text.
const MAX_TEXT: f64 = 300.0;
const PAD: i32 = 10;
const TOP: i32 = 8;
const ICON: i32 = 16;
const TITLE_HEIGHT: i32 = 16;
/// The space between the title and the text.
const GAP: i32 = 5;
const LINE: i32 = 13;
const CLOSE: i32 = 14;
const RADIUS: i32 = 7;
/// The height of the tail and the width of its base.
pub const TAIL: i32 = 18;
const TAIL_BASE: i32 = 16;
const MIN_WIDTH: i32 = 120;
const GROUP: &str = "xp-balloon";
const BODY: &str = "xp-balloon-body";
const CLOSE_ID: &str = "xp-balloon-close";
const BLACK: [u8; 4] = [0, 0, 0, 255];

/// The notification that shows and the ones that wait.
#[derive(Default)]
pub struct Queue {
    shown: Option<(Note, Option<Instant>)>,
    waiting: VecDeque<Note>,
}

impl Queue {
    /// The notification that shows.
    pub fn current(&self) -> Option<&Note> {
        self.shown.as_ref().map(|(note, _)| note)
    }

    /// When the notification that shows expires.
    pub fn due(&self) -> Option<Instant> {
        self.shown.as_ref().and_then(|(_, until)| *until)
    }

    /// A new notification shows at once or waits. One with the id of a
    /// known one takes its place, and its time starts again.
    pub fn push(&mut self, note: Note, now: Instant) {
        if self.current().is_some_and(|n| n.id == note.id) || self.shown.is_none() {
            self.show(note, now);
        } else if let Some(old) = self.waiting.iter_mut().find(|n| n.id == note.id) {
            *old = note;
        } else {
            self.waiting.push_back(note);
        }
    }

    fn show(&mut self, note: Note, now: Instant) {
        let until = note.timeout.map(|t| now + t);
        self.shown = Some((note, until));
    }

    /// Closes a notification; the next one shows.
    pub fn close(&mut self, id: u32, reason: Reason, now: Instant) -> Vec<Signal> {
        if self.current().is_some_and(|n| n.id == id) {
            self.shown = None;
            if let Some(next) = self.waiting.pop_front() {
                self.show(next, now);
            }
        } else if let Some(i) = self.waiting.iter().position(|n| n.id == id) {
            self.waiting.remove(i);
        } else {
            return Vec::new();
        }
        vec![Signal::Closed(id, reason)]
    }

    /// Closes the notification whose time ran out.
    pub fn expire(&mut self, now: Instant) -> Vec<Signal> {
        match (self.current().map(|n| n.id), self.due()) {
            (Some(id), Some(due)) if due <= now => self.close(id, Reason::Expired, now),
            _ => Vec::new(),
        }
    }

    /// A click on the balloon invokes its default action and closes it.
    pub fn click(&mut self, now: Instant) -> Vec<Signal> {
        let Some(note) = self.current() else {
            return Vec::new();
        };
        let id = note.id;
        let mut signals = Vec::new();
        if note.actions.iter().any(|a| a == "default") {
            signals.push(Signal::Action(id, "default".into()));
        }
        signals.extend(self.close(id, Reason::Dismissed, now));
        signals
    }
}

/// Breaks text into lines no wider than `max`, at spaces where it can, and
/// at its line breaks.
pub fn wrap(text: &str, max: f64, width: impl Fn(&str) -> f64) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.lines() {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if line.is_empty() {
                word.to_string()
            } else {
                format!("{line} {word}")
            };
            if width(&candidate) <= max {
                line = candidate;
                continue;
            }
            if !line.is_empty() {
                lines.push(std::mem::take(&mut line));
            }
            // A word wider than a line breaks between its characters.
            for c in word.chars() {
                line.push(c);
                if width(&line) > max && line.chars().count() > 1 {
                    line.pop();
                    lines.push(std::mem::replace(&mut line, c.to_string()));
                }
            }
        }
        lines.push(line);
    }
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    lines
}

/// Where the parts of a balloon sit, without the tail below it.
#[derive(Debug, PartialEq, Eq)]
pub struct Layout {
    /// The rounded rectangle with its border.
    pub size: [i32; 2],
    /// The icon, if any.
    pub icon: Option<Rect>,
    /// The title.
    pub title: Rect,
    /// The close button.
    pub close: Rect,
    /// The top of the first line of the text.
    pub text_top: i32,
}

/// The layout for a title and lines of text of some widths.
pub fn layout(icon: bool, title: f64, lines: &[f64]) -> Layout {
    let ceil = |w: f64| {
        #[allow(clippy::cast_possible_truncation)]
        let w = w.min(MAX_TEXT).ceil() as i32;
        w
    };
    let title_x = if icon { PAD + ICON + 6 } else { PAD };
    let text = lines.iter().copied().fold(0.0, f64::max);
    let width = (title_x + ceil(title) + 6 + CLOSE + 6)
        .max(PAD + ceil(text) + PAD)
        .max(MIN_WIDTH);
    let lines = i32::try_from(lines.len()).unwrap_or(0);
    let text_top = TOP + TITLE_HEIGHT + GAP;
    let height = if lines == 0 {
        TOP + TITLE_HEIGHT + TOP
    } else {
        text_top + lines * LINE + PAD
    };
    Layout {
        size: [width, height],
        icon: icon.then_some([PAD, TOP, ICON, ICON]),
        title: [title_x, TOP, width - title_x - CLOSE - 12, TITLE_HEIGHT],
        close: [width - CLOSE - 6, 6, CLOSE, CLOSE],
        text_top,
    }
}

/// The top-left corner of a balloon of a size, and the x of its tail tip
/// in it, for a tip at a point of a screen of a width.
pub fn place([w, h]: [i32; 2], [tip_x, tip_y]: [i32; 2], screen: i32) -> ([i32; 2], i32) {
    let right = (tip_x + 40).min(screen - 2);
    let left = (right - w).max(0);
    let tail = (tip_x - left).clamp(RADIUS + TAIL_BASE, w - RADIUS);
    ([left, tip_y - TAIL - h], tail)
}

/// The rounded rectangle with the tail as one outline, so that the black
/// border runs around both.
fn shape([w, h]: [i32; 2], tail: i32) -> String {
    let (x1, y1) = (f64::from(w) - 0.5, f64::from(h) - 0.5);
    let r = f64::from(RADIUS);
    let t = f64::from(tail) - 0.5;
    let mut path = String::new();
    let _ = write!(
        path,
        "M{a},0.5 H{b} A{r},{r} 0 0 1 {x1},{a} V{c} A{r},{r} 0 0 1 {b},{y1} \
         H{t} V{tip} L{base},{y1} H{a} A{r},{r} 0 0 1 0.5,{c} V{a} A{r},{r} 0 0 1 {a},0.5 Z",
        a = 0.5 + r,
        b = x1 - r,
        c = y1 - r,
        tip = y1 + f64::from(TAIL),
        base = t - f64::from(TAIL_BASE),
    );
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{}"><path d="{path}" fill="#FFFFE1" stroke="#000" stroke-width="1"/></svg>"##,
        h + TAIL
    )
}

/// A balloon on the overlay root of an output.
pub struct Balloon {
    output: String,
    layout: Layout,
    origin: [i32; 2],
    picture: Picture,
    hovered: bool,
}

impl Balloon {
    /// Lays out a notification above the notification area of a taskbar.
    pub fn new(theme: &mut Theme, note: &Note, icon: Option<&Path>, bar: &Bar) -> Self {
        let (tip, screen) = bar.tray_tip();
        let width = |s: &str| theme.text_width(s, Font::Ui);
        let lines = wrap(&note.body, MAX_TEXT, width);
        let widths: Vec<f64> = lines.iter().map(|l| width(l)).collect();
        let layout = layout(
            icon.is_some(),
            theme.text_width(&note.summary, Font::UiBold),
            &widths,
        );
        let (origin, tail) = place(layout.size, tip, screen);
        let [w, h] = layout.size;
        let file = theme
            .cache_file(
                &format!("balloon-{w}x{h}-{tail}.svg"),
                &shape(layout.size, tail),
            )
            .unwrap_or_default();
        let mut picture = Picture::new(w, h + TAIL).file(&file, [0, 0, w, h + TAIL]);
        if let (Some(path), Some(rect)) = (icon, layout.icon) {
            picture = picture.file(path, rect);
        }
        picture = picture.text(Text::new(&note.summary, Font::UiBold, BLACK, layout.title));
        for (line, y) in lines.iter().zip((layout.text_top..).step_by(LINE as usize)) {
            picture = picture.text(Text::new(
                line,
                Font::Ui,
                BLACK,
                [PAD, y, w - 2 * PAD, LINE],
            ));
        }
        Self {
            output: bar.output.clone(),
            layout,
            origin,
            picture,
            hovered: false,
        }
    }

    /// Whether a node belongs to a balloon.
    pub fn owns(node: &str) -> bool {
        node.starts_with(GROUP)
    }

    /// Whether a node is the close button.
    pub fn is_close(node: &str) -> bool {
        node == CLOSE_ID
    }

    /// Creates the balloon and its close button.
    pub fn create(&self, theme: &mut Theme) -> Vec<Command> {
        let [x, y] = self.origin.map(f64::from);
        vec![
            Command::Create {
                id: GROUP.into(),
                kind: NodeKind::Group,
                parent: format!("screen:{}", self.output),
                props: PropsUpdate {
                    transform: Some(Transform::translate(x, y, 0.0)),
                    ..PropsUpdate::default()
                },
            },
            Command::Create {
                id: BODY.into(),
                kind: NodeKind::Image,
                parent: GROUP.into(),
                props: PropsUpdate {
                    image: Some(theme.render(&self.picture)),
                    input: Some(InputMode::Accept),
                    ..PropsUpdate::default()
                },
            },
            Command::Create {
                id: CLOSE_ID.into(),
                kind: NodeKind::Image,
                parent: GROUP.into(),
                props: PropsUpdate {
                    input: Some(InputMode::Accept),
                    ..self.close_look(theme)
                },
            },
        ]
    }

    /// The close button lights up under the pointer.
    pub fn hover(&mut self, theme: &mut Theme, node: &str, on: bool) -> Vec<Command> {
        if !Self::is_close(node) {
            return Vec::new();
        }
        self.hovered = on;
        vec![Command::Update {
            id: CLOSE_ID.into(),
            props: self.close_look(theme),
        }]
    }

    fn close_look(&self, theme: &mut Theme) -> PropsUpdate {
        let file = if self.hovered {
            "winxp/balloon-close-hover.svg"
        } else {
            "winxp/balloon-close.svg"
        };
        let picture = Picture::new(CLOSE, CLOSE).file(&theme.path(file), [0, 0, CLOSE, CLOSE]);
        let [x, y, ..] = self.layout.close;
        PropsUpdate {
            image: Some(theme.render(&picture)),
            transform: Some(Transform::translate(f64::from(x), f64::from(y), 0.0)),
            ..PropsUpdate::default()
        }
    }

    /// Removes the balloon.
    pub fn remove() -> Command {
        Command::Remove {
            id: GROUP.into(),
            to: None,
        }
    }
}

/// How long the main loop may sleep before it looks for new
/// notifications: the bus thread cannot wake the read of the core socket.
// ponytail: polls at 10 Hz; a wake fd in chora-scene's client would end it.
pub const POLL: Duration = Duration::from_millis(100);

#[cfg(test)]
mod tests {
    use super::*;

    fn note(id: u32, timeout: Option<u64>) -> Note {
        Note {
            id,
            icon: String::new(),
            summary: format!("n{id}"),
            body: String::new(),
            actions: Vec::new(),
            timeout: timeout.map(Duration::from_secs),
        }
    }

    /// Six pixels per character.
    #[allow(clippy::cast_precision_loss)]
    fn mono(s: &str) -> f64 {
        s.chars().count() as f64 * 6.0
    }

    #[test]
    fn text_wraps_at_spaces_and_breaks() {
        assert_eq!(wrap("aaa bbb ccc", 42.0, mono), ["aaa bbb", "ccc"]);
        assert_eq!(wrap("one\n\ntwo", 300.0, mono), ["one", "", "two"]);
        assert_eq!(wrap("abcdefghij", 24.0, mono), ["abcd", "efgh", "ij"]);
        assert_eq!(wrap("", 300.0, mono).len(), 0);
    }

    #[test]
    fn the_balloon_fits_its_text() {
        let short = layout(true, 40.0, &[]);
        assert_eq!(short.size, [MIN_WIDTH, 32]);
        assert_eq!(short.icon, Some([10, 8, 16, 16]));
        assert_eq!(short.title[0], 32);
        let long = layout(false, 40.0, &[300.0, 120.0]);
        assert_eq!(long.size, [320, 29 + 26 + 10]);
        assert_eq!(long.close, [320 - 20, 6, 14, 14]);
        assert!(long.icon.is_none());
    }

    #[test]
    fn the_tail_points_at_the_tray() {
        let (origin, tail) = place([200, 60], [1200, 785], 1280);
        assert_eq!(origin, [1040, 785 - TAIL - 60]);
        assert_eq!(origin[0] + tail, 1200);
        // At the right edge, the balloon stays on the screen.
        let (origin, tail) = place([200, 60], [1270, 785], 1280);
        assert_eq!(origin[0] + 200, 1278);
        assert!(tail <= 200 - RADIUS);
    }

    #[test]
    fn notifications_wait_their_turn() {
        let now = Instant::now();
        let mut queue = Queue::default();
        queue.push(note(1, Some(10)), now);
        queue.push(note(2, None), now);
        queue.push(note(3, Some(10)), now);
        assert_eq!(queue.current().map(|n| n.id), Some(1));
        // A replacement of a waiting one keeps its place.
        let mut two = note(2, None);
        two.summary = "new".into();
        queue.push(two, now);
        assert_eq!(queue.waiting.len(), 2);
        assert_eq!(queue.expire(now + Duration::from_secs(9)).len(), 0);
        let later = now + Duration::from_secs(10);
        assert_eq!(queue.expire(later), [Signal::Closed(1, Reason::Expired)]);
        assert_eq!(queue.current().map(|n| n.summary.as_str()), Some("new"));
        // No timeout: it waits for a click.
        assert_eq!(queue.due(), None);
        assert_eq!(
            queue.close(3, Reason::Closed, later),
            [Signal::Closed(3, Reason::Closed)]
        );
        assert_eq!(queue.close(3, Reason::Closed, later).len(), 0);
    }

    #[test]
    fn a_replacement_starts_the_time_again() {
        let now = Instant::now();
        let mut queue = Queue::default();
        queue.push(note(1, Some(10)), now);
        let later = now + Duration::from_secs(5);
        queue.push(note(1, Some(10)), later);
        assert_eq!(queue.due(), Some(later + Duration::from_secs(10)));
    }

    #[test]
    fn a_click_invokes_the_default_action() {
        let now = Instant::now();
        let mut queue = Queue::default();
        let mut one = note(1, None);
        one.actions = vec!["default".into()];
        queue.push(one, now);
        queue.push(note(2, None), now);
        assert_eq!(
            queue.click(now),
            [
                Signal::Action(1, "default".into()),
                Signal::Closed(1, Reason::Dismissed)
            ]
        );
        assert_eq!(queue.click(now), [Signal::Closed(2, Reason::Dismissed)]);
        assert_eq!(queue.click(now).len(), 0);
    }
}
