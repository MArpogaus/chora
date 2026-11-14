//! The card of a notification in a corner of an output: a rounded
//! rectangle with a border, an icon, a bold title, the wrapped text and a
//! close button. The values of the asset folder give its colors, its
//! corners and an optional tail. One card shows at a time; the others
//! wait in a queue.

use std::{
    collections::VecDeque,
    path::Path,
    time::{Duration, Instant},
};

use chora_draw::{
    bubble::{Corner, Style},
    text::{Font, Text, wrap},
    theme::{Picture, Rect, Theme},
};
use chora_scene::{Command, InputMode, NodeKind, OutputInfo, PropsUpdate, Transform};

use crate::{
    bus::{Note, Reason, Signal},
    config::Config,
};

/// The widest line of the text.
const MAX_TEXT: f64 = 300.0;
/// The most lines of the body that a card shows.
const MAX_LINES: usize = 8;
/// The most bytes of the body that are laid out.
const MAX_BYTES: usize = 1024;
const PAD: i32 = 10;
const TOP: i32 = 8;
const ICON: i32 = 16;
const TITLE_HEIGHT: i32 = 16;
/// The space between the title and the text.
const GAP: i32 = 5;
const LINE: i32 = 13;
const CLOSE: i32 = 14;
const MIN_WIDTH: i32 = 120;
pub const GROUP: &str = "notify-card";
const BODY: &str = "notify-card-body";
const CLOSE_ID: &str = "notify-card-close";

/// The pictures that the card reads from the asset folder.
pub const PICTURES: [&str; 2] = ["close.svg", "close-hover.svg"];

/// The values of `values.yaml` of the asset folder.
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Values {
    /// The base font family of the text; `sans-serif` is the system's.
    pub font: String,
    /// The distance of the card from the corner of the output.
    pub margin: [i32; 2],
    /// The look of the card.
    pub card: Style,
}

impl Default for Values {
    fn default() -> Self {
        Self {
            font: chora_draw::text::SANS_SERIF.into(),
            margin: [16, 16],
            card: Style::default(),
        }
    }
}

/// The notification that shows and the ones that wait.
pub struct Queue {
    times: Config,
    shown: Option<(Note, Option<Instant>)>,
    waiting: VecDeque<Note>,
}

impl Queue {
    /// An empty queue; the settings give the times of the notifications.
    pub fn new(times: Config) -> Self {
        Self {
            times,
            shown: None,
            waiting: VecDeque::new(),
        }
    }

    /// Takes the times of changed settings for the notifications that show
    /// from now on.
    pub fn set_times(&mut self, times: Config) {
        self.times = times;
    }

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
        let until = self.times.timeout(note.expire_ms).map(|t| now + t);
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

    /// A click on the card invokes its default action and closes it.
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

/// Where the parts of a card sit, without the tail below it.
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

/// The wrapped lines of a body that fit the card.
fn body_lines(body: &str, width: impl Fn(&str) -> f64) -> Vec<String> {
    let mut end = body.len().min(MAX_BYTES);
    while !body.is_char_boundary(end) {
        end -= 1;
    }
    let mut lines = wrap(&body[..end], MAX_TEXT, width);
    if end < body.len() || lines.len() > MAX_LINES {
        lines.truncate(MAX_LINES);
        if let Some(last) = lines.last_mut() {
            last.push('…');
        }
    }
    lines
}

/// A card on the overlay root of an output.
pub struct Card {
    output: String,
    layout: Layout,
    origin: [i32; 2],
    picture: Picture,
    hovered: bool,
}

impl Card {
    /// Lays out a notification in a corner of an output.
    pub fn new(
        theme: &mut Theme,
        note: &Note,
        icon: Option<&Path>,
        values: &Values,
        (output, corner): (&OutputInfo, Corner),
    ) -> Self {
        let style = &values.card;
        let color = style.text_rgba();
        let width = |s: &str| theme.text_width(s, Font::Ui);
        let lines = body_lines(&note.body, width);
        let widths: Vec<f64> = lines.iter().map(|l| width(l)).collect();
        let layout = layout(
            icon.is_some(),
            theme.text_width(&note.summary, Font::UiBold),
            &widths,
        );
        let [w, h] = layout.size;
        let height = h + style.tail;
        let origin = corner.origin(output.size, [w, height], values.margin);
        let mut picture =
            Picture::new(w, height).svg(style.shape(layout.size, style.tail_x(w, corner)));
        if let (Some(path), Some(rect)) = (icon, layout.icon) {
            picture = picture.file(path, rect);
        }
        picture = picture.text(Text::new(&note.summary, Font::UiBold, color, layout.title));
        for (line, y) in lines.iter().zip((layout.text_top..).step_by(LINE as usize)) {
            picture = picture.text(Text::new(
                line,
                Font::Ui,
                color,
                [PAD, y, w - 2 * PAD, LINE],
            ));
        }
        Self {
            output: output.name.clone(),
            layout,
            origin,
            picture,
            hovered: false,
        }
    }

    /// Whether a node belongs to a card.
    pub fn owns(node: &str) -> bool {
        node.starts_with(GROUP)
    }

    /// Whether a node is the close button.
    pub fn is_close(node: &str) -> bool {
        node == CLOSE_ID
    }

    /// Creates the card and its close button.
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
            "close-hover.svg"
        } else {
            "close.svg"
        };
        let picture = Picture::new(CLOSE, CLOSE).file(&theme.path(file), [0, 0, CLOSE, CLOSE]);
        let [x, y, ..] = self.layout.close;
        PropsUpdate {
            image: Some(theme.render(&picture)),
            transform: Some(Transform::translate(f64::from(x), f64::from(y), 0.0)),
            ..PropsUpdate::default()
        }
    }

    /// Removes the card.
    pub fn remove() -> Command {
        Command::Remove {
            id: GROUP.into(),
            to: None,
        }
    }
}

/// How long the main loop may sleep before it looks for new
/// notifications: the bus thread cannot wake the read of the core socket,
/// so the loop polls at 10 Hz until the client of chora-scene can wait on
/// a wake file descriptor.
pub const POLL: Duration = Duration::from_millis(100);

#[cfg(test)]
mod tests {
    use super::*;

    fn note(id: u32, expire_ms: i32) -> Note {
        Note {
            id,
            icon: String::new(),
            summary: format!("n{id}"),
            body: String::new(),
            actions: Vec::new(),
            expire_ms,
        }
    }

    fn queue() -> Queue {
        Queue::new(chora_kit::settings::parse(crate::config::DEFAULT).unwrap())
    }

    #[test]
    fn the_card_fits_its_text() {
        let short = layout(true, 40.0, &[]);
        assert_eq!(short.size, [MIN_WIDTH, 32]);
        assert_eq!(short.icon, Some([10, 8, 16, 16]));
        assert_eq!(short.title[0], 32);
        let long = layout(false, 40.0, &[300.0, 120.0]);
        assert_eq!(long.size, [320, 29 + 26 + 10]);
        assert_eq!(long.close, [320 - 20, 6, 14, 14]);
        assert!(long.icon.is_none());
    }

    fn chars_wide(s: &str) -> f64 {
        f64::from(u32::try_from(s.chars().count()).unwrap_or(u32::MAX)) * 6.0
    }

    #[test]
    fn an_unbounded_body_is_cut_to_a_few_lines_with_an_ellipsis() {
        let body = "word ".repeat(200_000);
        let lines = body_lines(&body, chars_wide);
        assert!(lines.len() <= MAX_LINES, "{} lines", lines.len());
        assert!(lines.last().is_some_and(|l| l.ends_with('…')));
        let one_line = "x".repeat(1_000_000);
        let lines = body_lines(&one_line, chars_wide);
        assert!(lines.iter().map(String::len).sum::<usize>() <= MAX_BYTES + MAX_LINES * 4);
    }

    #[test]
    fn notifications_wait_their_turn() {
        let now = Instant::now();
        let mut queue = queue();
        queue.push(note(1, 10_000), now);
        queue.push(note(2, 0), now);
        queue.push(note(3, 10_000), now);
        assert_eq!(queue.current().map(|n| n.id), Some(1));
        let mut two = note(2, 0);
        two.summary = "new".into();
        queue.push(two, now);
        assert_eq!(queue.waiting.len(), 2);
        assert_eq!(queue.expire(now + Duration::from_secs(9)).len(), 0);
        let later = now + Duration::from_secs(10);
        assert_eq!(queue.expire(later), [Signal::Closed(1, Reason::Expired)]);
        assert_eq!(queue.current().map(|n| n.summary.as_str()), Some("new"));
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
        let mut queue = queue();
        queue.push(note(1, 10_000), now);
        let later = now + Duration::from_secs(5);
        queue.push(note(1, 10_000), later);
        assert_eq!(queue.due(), Some(later + Duration::from_secs(10)));
    }

    #[test]
    fn a_click_invokes_the_default_action() {
        let now = Instant::now();
        let mut queue = queue();
        let mut one = note(1, 0);
        one.actions = vec!["default".into()];
        queue.push(one, now);
        queue.push(note(2, 0), now);
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
