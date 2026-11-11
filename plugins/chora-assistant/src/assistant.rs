//! The assistant of the voice agent: a figure above the corner of the
//! first output that follows the state of the agent (D65) and speaks its
//! sentences in a bubble. The figure is a picture of the folder with one
//! element for each state, or the dot of the host. The core moves and
//! scales it. A picture with elements named `STATE-shut` blinks.

use std::time::{Duration, Instant};

use chora_draw::{
    bubble::Style,
    text::{Font, Text, wrap},
    theme::{Picture, Theme},
};
use chora_kit::hud::{Anchor, render};
use chora_scene::{Command, Easing, InputMode, NodeKind, OutputInfo, PropsUpdate, Transform};

use crate::config::{Figure, Values};

/// The group of the HUD folder under `screen:OUTPUT`.
const GROUP: &str = "hud";
/// The group that moves: leans, wobbles and bounces.
const BODY: &str = "hud-body";
/// The image of the figure, which takes the clicks.
const CLIP: &str = "hud-figure";
/// The balloon of the spoken sentence.
const SPEECH: &str = "hud-line";
/// The node that names the state in words, for readers of the scene.
const STATE: &str = "hud-state";
/// The state word beside the figure, if the folder shows one.
const WORD: &str = "hud-word";
const PAD: i32 = 10;
const LINE: i32 = 13;
const MIN_WIDTH: i32 = 120;
/// The widest line of the assistant's speech.
const MAX_SPEECH: f64 = 220.0;

/// The size of the dot of the host.
const DOT_SIZE: [i32; 2] = [24, 24];
/// The dot of the host: one group for each state, one per line.
const DOT: &str = include_str!("../huds/neutral/dot.svg");

/// How long the eyes stay shut.
const BLINK: Duration = Duration::from_millis(140);
/// How long the balloon stays after its sentence ends.
const LINGER: Duration = Duration::from_secs(3);
/// How long the balloon takes to fade out, in milliseconds.
const FADE: f64 = 600.0;

/// The widest line of a sentence without a box.
const MAX_PLAIN: f64 = 420.0;
/// The gap between the figure and a sentence without a box.
const BESIDE: i32 = 8;
/// The padding of the backing of a sentence without a box.
const BACKING: i32 = 4;
/// How far above and below the figure a balloon may reach.
const BALLOON_REACH: i32 = 144;

/// What the figure does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pose {
    /// Blinks now and then.
    Idle,
    /// Eyes wide, leaning in.
    Listening,
    /// Eyes up, a slight wobble.
    Thinking,
    /// A gentle bounce while the balloon shows the text.
    Speaking,
    /// Raised eyebrows.
    Asking,
    /// Eyes shut, dimmed: the agent sleeps, and its mic still listens
    /// for the wake words.
    Sleeping,
}

/// The opacity of the figure while the agent sleeps.
const DIM: f64 = 0.55;

/// The pose for a state of the voice agent: the state of `voice.state`
/// or of the method `state`.
pub fn pose(state: &str) -> Pose {
    match state {
        "sleeping" | "asleep" => Pose::Sleeping,
        "listening" | "recording" => Pose::Listening,
        "transcribing" | "thinking" | "tool" | "working" => Pose::Thinking,
        "speaking" => Pose::Speaking,
        "question" | "confirming" => Pose::Asking,
        _ => Pose::Idle,
    }
}

/// The name of a state of the voice agent in the context of a HUD folder:
/// `asleep`, `listening`, `thinking`, `working`, `question`, `speaking`,
/// else `idle`.
pub fn context_state(state: &str) -> &'static str {
    match state {
        "sleeping" | "asleep" => "asleep",
        "listening" | "recording" => "listening",
        "transcribing" | "thinking" => "thinking",
        "tool" | "working" => "working",
        "speaking" => "speaking",
        "question" | "confirming" => "question",
        _ => "idle",
    }
}

impl Pose {
    /// The opacity of the picture: dimmed while the agent sleeps.
    fn opacity(self) -> f64 {
        if self == Self::Sleeping { DIM } else { 1.0 }
    }

    /// The move of the body at a step of the pose: the scale, the offset
    /// and the time in milliseconds. None: it holds still.
    fn motion(self, step: u32) -> Option<([f64; 2], [f64; 2], f64)> {
        let even = step.is_multiple_of(2);
        match (self, step) {
            (Self::Idle, 0) => Some(([1.0, 1.0], [0.0, 0.0], 300.0)),
            (Self::Listening, 0) => Some(([1.0, 1.0], [-2.0, 0.0], 250.0)),
            (Self::Asking, 0) => Some(([1.0, 1.0], [0.0, -2.0], 250.0)),
            (Self::Thinking, _) => Some(([1.0, 1.0], [if even { 2.0 } else { -2.0 }, 0.0], 700.0)),
            (Self::Speaking, _) if even => Some(([0.98, 1.03], [0.0, -4.0], 180.0)),
            (Self::Speaking, _) => Some(([1.0, 1.0], [0.0, 0.0], 180.0)),
            _ => None,
        }
    }
}

/// The group of the dot of the host for a state, as one line of its SVG;
/// the group `idle` for a state that has none.
fn dot_group(state: &str) -> &'static str {
    let line = |id: &str| DOT.lines().find(|l| l.contains(&format!("id=\"{id}\"")));
    line(state).or_else(|| line("idle")).unwrap_or_default()
}

/// The figure in a state, with its eyes open or shut.
fn draw(theme: &Theme, look: &Figure, state: &str, shut: bool) -> Picture {
    match look {
        Figure::Dot => {
            let [w, h] = DOT_SIZE;
            Picture::new(w, h).svg(dot_group(state))
        }
        Figure::Sheet {
            sheet,
            template,
            size: [w, h],
        } => {
            let has = |id: &str| theme.size(*sheet, id).is_some();
            let id = element_of(has, template, state, shut);
            Picture::new(*w, *h).in_place(*sheet, &id, [0, 0, *w, *h])
        }
    }
}

/// The id of the element of a sheet for a state: the template with the
/// state, `idle` where the sheet has no such element, and with shut eyes
/// the element `ID-shut` where the sheet has one.
fn element_of(has: impl Fn(&str) -> bool, template: &str, state: &str, shut: bool) -> String {
    let named = render(template, &serde_json::json!({ "state": state }));
    let id = if has(&named) {
        named
    } else {
        "idle".to_owned()
    };
    let closed = format!("{id}-shut");
    if shut && has(&closed) { closed } else { id }
}

/// The top-left corner of a sentence without a box: beside the figure
/// (x, y, width, height), on the side of the middle of the area, with its
/// top at the top of the figure, inside the area.
fn beside([w, h]: [i32; 2], [fx, fy, fw, _]: [i32; 4], [ax, ay, aw, ah]: [i32; 4]) -> [i32; 2] {
    let at_right = fx + fw / 2 >= ax + aw / 2;
    let x = if at_right {
        fx - BESIDE - w
    } else {
        fx + fw + BESIDE
    };
    [
        x.clamp(ax, (ax + aw - w).max(ax)),
        fy.clamp(ay, (ay + ah - h).max(ay)),
    ]
}

impl Figure {
    /// The size of the picture.
    pub fn size(&self) -> [i32; 2] {
        match self {
            Self::Dot => DOT_SIZE,
            Self::Sheet { size, .. } => *size,
        }
    }

    /// The point that the body turns around: the middle of a figure, the
    /// foot of one that is taller than wide.
    fn pivot(&self) -> [f64; 2] {
        let [w, h] = self.size().map(f64::from);
        if h > w {
            [w / 2.0, h]
        } else {
            [w / 2.0, h / 2.0]
        }
    }

    /// Where the tail of the balloon points: above the head.
    fn head(&self) -> [i32; 2] {
        [self.size()[0] / 2, 0]
    }
}

/// The top-left corner of a picture of a size at an anchor of an area of
/// the output (x, y, width, height), with the margin to the edges.
pub fn origin(anchor: Anchor, area: [i32; 4], size: [i32; 2], margin: [i32; 2]) -> [i32; 2] {
    let [x, y, w, h] = area;
    let left = x + margin[0];
    let right = x + w - margin[0] - size[0];
    let centre_x = x + (w - size[0]) / 2;
    let top = y + margin[1];
    let bottom = y + h - margin[1] - size[1];
    let centre_y = y + (h - size[1]) / 2;
    match anchor {
        Anchor::TopLeft => [left, top],
        Anchor::Top => [centre_x, top],
        Anchor::TopRight => [right, top],
        Anchor::Left => [left, centre_y],
        Anchor::Center => [centre_x, centre_y],
        Anchor::Right => [right, centre_y],
        Anchor::BottomLeft => [left, bottom],
        Anchor::Bottom => [centre_x, bottom],
        Anchor::BottomRight => [right, bottom],
    }
}

/// The area of an output where the HUD anchors, in the coordinates of the
/// output: its work area, which leaves out the panels and bars of the
/// window managers.
#[must_use]
pub fn work_area(output: &OutputInfo) -> [i32; 4] {
    let [x, y, w, h] = output.layout_area();
    if w <= 0 || h <= 0 {
        return [0, 0, output.size[0], output.size[1]];
    }
    [x - output.position[0], y - output.position[1], w, h]
}

/// The transform of the body: scaled from its foot and moved.
fn body([px, py]: [f64; 2], [sx, sy]: [f64; 2], [dx, dy]: [f64; 2]) -> Transform {
    Transform::translate(px + dx, py + dy, 0.0)
        .then(&Transform::scale(sx, sy, 1.0))
        .then(&Transform::translate(-px, -py, 0.0))
}

/// The time between two blinks, a little different each time.
fn blink_gap(blinks: u32) -> Duration {
    Duration::from_millis(2500 + u64::from(blinks.wrapping_mul(1777) % 3500))
}

/// The figure on the overlay root of an output.
pub struct Assistant {
    values: Values,
    /// The top-left corner of the picture, in output coordinates.
    origin: [i32; 2],
    /// The width of the output.
    area: [i32; 4],
    /// The state of the voice agent, as [`context_state`] names it.
    state: &'static str,
    pose: Pose,
    shut: bool,
    /// The next move of the pose.
    step: u32,
    blinks: u32,
    /// When the body moves next; none while it holds still.
    moves_at: Option<Instant>,
    /// When the eyes open or shut next.
    blinks_at: Instant,
    /// The balloon exists.
    speech: bool,
    /// When the balloon fades out.
    fades_at: Option<Instant>,
    /// The size of the state word, none without one.
    word: [i32; 2],
}

/// Where a balloon goes: its top-left corner, the x of its tail tip in it
/// and whether it hangs below the figure without a tail. A balloon sits
/// above the tip, with its tail pointing down at it, inside an area of
/// the output; where it would leave the top of the area, it sits below
/// the bottom of the figure instead.
fn place(
    (style, tail_left): (&Style, bool),
    [w, h]: [i32; 2],
    [tip_x, tip_y]: [i32; 2],
    (area, figure_bottom): ([i32; 4], i32),
) -> ([i32; 2], i32, bool) {
    let [ax, ay, aw, _] = area;
    let left = if tail_left {
        (tip_x - style.tail_inset).max(ax).min(ax + aw - w)
    } else {
        let right = (tip_x + style.tail_inset).min(ax + aw - 2);
        (right - w).max(ax)
    };
    let tail = (tip_x - left).clamp(style.radius + style.tail_base, w - style.radius);
    let above = tip_y - style.tail - h;
    if above >= ay {
        ([left, above], tail, false)
    } else {
        ([left, figure_bottom], tail, true)
    }
}

/// A balloon with text only, as the assistant speaks: its picture and its
/// top-left corner, for a tail tip at a point of an area of the output.
fn speech(
    theme: &mut Theme,
    (style, tail_left): (&Style, bool),
    text: &str,
    (tip, anchor): ([i32; 2], ([i32; 4], i32)),
) -> (Picture, [i32; 2], i32) {
    let width = |s: &str| theme.text_width(s, Font::Ui);
    let lines = wrap(text, MAX_SPEECH, width);
    let text_width = lines.iter().map(|l| width(l)).fold(0.0, f64::max);
    #[allow(clippy::cast_possible_truncation)]
    let w = (PAD + (text_width.ceil() as i32) + PAD).max(MIN_WIDTH);
    let h = PAD + i32::try_from(lines.len()).unwrap_or(0) * LINE + PAD;
    let (origin, tail, below) = place((style, tail_left), [w, h], tip, anchor);
    let style = Style {
        tail: if below { 0 } else { style.tail },
        ..style.clone()
    };
    let mut picture = Picture::new(w, h + style.tail).svg(style.shape([w, h], tail));
    for (line, y) in lines.iter().zip((PAD..).step_by(LINE as usize)) {
        picture = picture.text(Text::new(
            line,
            Font::Ui,
            style.text_rgba(),
            [PAD, y, w - 2 * PAD, LINE],
        ));
    }
    (picture, origin, style.tail)
}

impl Assistant {
    /// Creates the figure in an output at an anchor, `label` being the
    /// state in words. Where the folder's place would put the figure partly
    /// outside the output, the neutral dot stands in the neutral place.
    pub fn new(
        theme: &mut Theme,
        values: &Values,
        (output, anchor): (&OutputInfo, Anchor),
        (state, label, now): (&'static str, &str, Instant),
    ) -> (Self, Vec<Command>) {
        let mut values = values.clone();
        let area = work_area(output);
        let mut origin = origin(anchor, area, values.figure.size(), values.margin);
        if !inside(origin, values.figure.size(), output.size) {
            values = Values::neutral(&values.font);
            origin = origin_of(&values, area);
        }
        let assistant = Self {
            values,
            origin,
            area,
            state,
            pose: pose(state),
            shut: false,
            step: 0,
            blinks: 0,
            moves_at: Some(now),
            blinks_at: now + blink_gap(0),
            speech: false,
            fades_at: None,
            word: [0, 0],
        };
        let mut assistant = assistant;
        let commands = assistant.created(theme, &output.name, label);
        (assistant, commands)
    }

    /// The commands that create the nodes of the figure on an output.
    fn created(&mut self, theme: &mut Theme, output: &str, label: &str) -> Vec<Command> {
        let [x, y] = self.origin.map(f64::from);
        let figure = self.values.figure.clone();
        let figure = &figure;
        let word = self.word(theme, label).map(|props| Command::Create {
            id: WORD.into(),
            kind: NodeKind::Image,
            parent: GROUP.into(),
            props,
        });
        vec![
            group(
                GROUP,
                format!("screen:{output}"),
                Transform::translate(x, y, 0.0),
            ),
            Command::Create {
                id: STATE.into(),
                kind: NodeKind::Group,
                parent: GROUP.into(),
                props: PropsUpdate {
                    meta: Some([("text".to_owned(), label.to_owned())].into()),
                    ..PropsUpdate::default()
                },
            },
            group(
                BODY,
                GROUP.into(),
                body(figure.pivot(), [1.0, 1.0], [0.0, 0.0]),
            ),
            Command::Create {
                id: CLIP.into(),
                kind: NodeKind::Image,
                parent: BODY.into(),
                props: PropsUpdate {
                    image: Some(theme.render(&draw(theme, figure, self.state, false))),
                    opacity: Some(self.pose.opacity()),
                    input: Some(InputMode::Accept),
                    ..PropsUpdate::default()
                },
            },
        ]
        .into_iter()
        .chain(word)
        .collect()
    }

    /// The picture of the state word on the backing of a plain sentence,
    /// beside the figure, toward the middle of the area; none for a folder
    /// without a word.
    fn word(&mut self, theme: &mut Theme, label: &str) -> Option<PropsUpdate> {
        let template = self.values.word.as_deref()?;
        let text = chora_kit::hud::render(template, &serde_json::json!({ "state": label }));
        #[allow(
            clippy::cast_possible_truncation,
            reason = "a word is narrower than a screen"
        )]
        let size = [
            theme.text_width(&text, Font::Ui).ceil() as i32 + 2 * BACKING,
            LINE + 2 * BACKING,
        ];
        self.word = size;
        let [x, y, ..] = self.word_at(size);
        let [red, green, blue] = self.values.speech.fill;
        let [w, h] = size;
        let backing = format!(
            r#"<rect width="{w}" height="{h}" rx="{BACKING}" fill="rgb({red},{green},{blue})"/>"#
        );
        let place = [BACKING, BACKING, w - 2 * BACKING, LINE];
        let colour = self.values.speech.text_rgba();
        let picture =
            Picture::new(w, h)
                .svg(backing)
                .text(Text::new(&text, Font::Ui, colour, place));
        Some(PropsUpdate {
            image: Some(theme.render(&picture)),
            transform: Some(Transform::translate(
                f64::from(x - self.origin[0]),
                f64::from(y - self.origin[1]),
                0.0,
            )),
            input: Some(InputMode::Pass),
            meta: Some([("text".to_owned(), text)].into()),
            ..PropsUpdate::default()
        })
    }

    /// The rectangle of a word of `size` beside the figure, in the middle
    /// of its height.
    fn word_at(&self, [w, h]: [i32; 2]) -> [i32; 4] {
        let [fx, fy, fw, fh] = self.rect();
        let [x, _] = beside([w, h], [fx, fy, fw, fh], self.area);
        [x, fy + (fh - h) / 2, w, h]
    }

    /// The rectangle of the figure and its state word.
    fn with_word(&self) -> [i32; 4] {
        let [fx, fy, fw, fh] = self.rect();
        let [x, y, w, h] = self.word_rect();
        if w == 0 {
            return [fx, fy, fw, fh];
        }
        let (left, top) = (fx.min(x), fy.min(y));
        let (right, bottom) = ((fx + fw).max(x + w), (fy + fh).max(y + h));
        [left, top, right - left, bottom - top]
    }

    /// The rectangle of the state word on the output, empty without one.
    pub fn word_rect(&self) -> [i32; 4] {
        if self.word == [0, 0] {
            return [0; 4];
        }
        self.word_at(self.word)
    }

    /// Whether a node is the figure.
    pub fn owns(node: &str) -> bool {
        node == CLIP
    }

    /// A click on the figure mutes the mic.
    pub fn mutes(&self) -> bool {
        self.values.mutes
    }

    /// The rectangle of the figure on the output: x, y, width, height.
    pub fn rect(&self) -> [i32; 4] {
        let [w, h] = self.values.figure.size();
        [self.origin[0], self.origin[1], w, h]
    }

    /// The template and the length of the text of a sentence.
    pub fn line(&self) -> &crate::config::Line {
        &self.values.line
    }

    /// Removes the figure and its balloon.
    pub fn remove() -> Command {
        Command::Remove {
            id: GROUP.into(),
            to: None,
        }
    }

    /// Takes another state; its moves start at once. `label` is the state
    /// in words.
    pub fn set_state(
        &mut self,
        theme: &mut Theme,
        (state, label): (&'static str, &str),
        now: Instant,
    ) -> Vec<Command> {
        if state == self.state {
            return Vec::new();
        }
        self.state = state;
        self.pose = pose(state);
        self.step = 0;
        self.moves_at = Some(now);
        let word = self.word(theme, label).map(|props| Command::Update {
            id: WORD.into(),
            props,
        });
        vec![
            self.face(theme),
            Command::Update {
                id: STATE.into(),
                props: PropsUpdate {
                    meta: Some([("text".to_owned(), label.to_owned())].into()),
                    ..PropsUpdate::default()
                },
            },
        ]
        .into_iter()
        .chain(word)
        .collect()
    }

    fn face(&self, theme: &mut Theme) -> Command {
        let picture = draw(theme, &self.values.figure, self.state, self.shut);
        Command::Update {
            id: CLIP.into(),
            props: PropsUpdate {
                image: Some(theme.render(&picture)),
                opacity: Some(self.pose.opacity()),
                ..PropsUpdate::default()
            },
        }
    }

    /// Shows a sentence in the balloon for a time, and a while after it.
    pub fn say(
        &mut self,
        theme: &mut Theme,
        text: &str,
        time: Duration,
        now: Instant,
    ) -> Vec<Command> {
        self.fades_at = Some(now + time + LINGER);
        self.balloon(theme, text)
    }

    /// The part of the output that the figure and its balloon may take:
    /// x, y, width, height. A balloon is at most `MAX_SPEECH` wide and
    /// stands above or below the figure.
    pub fn reach(&self) -> [i32; 4] {
        let [x, y, w, h] = self.with_word();
        if self.values.line.plain {
            #[allow(clippy::cast_possible_truncation)]
            let most = [MAX_PLAIN as i32 + 2 * BACKING, BALLOON_REACH];
            let [left, top] = beside(most, [x, y, w, h], self.area);
            let from = left.min(x);
            let to = (left + most[0]).max(x + w);
            return [from, top, to - from, most[1].max(h)];
        }
        #[allow(clippy::cast_possible_truncation)]
        let widest = PAD + MAX_SPEECH as i32 + PAD;
        let style = (&self.values.speech, self.values.line.tail_left);
        let tip = [x + self.values.figure.head()[0], y];
        let ([left, _], ..) = place(style, [widest, 0], tip, (self.area, y + h));
        let from = left.min(x);
        let to = (left + widest).max(x + w);
        [from, y - BALLOON_REACH, to - from, h + 2 * BALLOON_REACH]
    }

    /// A sentence without a box, beside the figure, on a backing in the
    /// fill of the chat, so that it reads on any window behind it.
    fn plain(&mut self, theme: &mut Theme, text: &str) -> Vec<Command> {
        let width = |s: &str| theme.text_width(s, Font::Ui);
        let lines = wrap(text, MAX_PLAIN, width);
        let widest = lines.iter().map(|l| width(l)).fold(0.0, f64::max);
        #[allow(clippy::cast_possible_truncation)]
        let w = (widest.ceil() as i32).max(1) + 2 * BACKING;
        let h = i32::try_from(lines.len()).unwrap_or(0).max(1) * LINE + 2 * BACKING;
        let [red, green, blue] = self.values.speech.fill;
        let backing = format!(
            r#"<rect width="{w}" height="{h}" rx="{BACKING}" fill="rgb({red},{green},{blue})"/>"#
        );
        let mut picture = Picture::new(w, h).svg(backing);
        let colour = self.values.speech.text_rgba();
        for (line, y) in lines.iter().zip((BACKING..).step_by(LINE as usize)) {
            let place = [BACKING, y, w - 2 * BACKING, LINE];
            picture = picture.text(Text::new(line, Font::Ui, colour, place));
        }
        let [bx, by] = beside([w, h], self.with_word(), self.area);
        self.speech_node(theme, &picture, [bx - self.origin[0], by - self.origin[1]])
    }

    /// The balloon of a sentence.
    fn balloon(&mut self, theme: &mut Theme, text: &str) -> Vec<Command> {
        if self.values.line.plain {
            return self.plain(theme, text);
        }
        let [x, y] = self.origin;
        let [head_x, head_y] = self.values.figure.head();
        let tip = [x + head_x, y + head_y];
        let anchor = (self.area, y + self.values.figure.size()[1]);
        let style = (&self.values.speech, self.values.line.tail_left);
        let (picture, [bx, by], _) = speech(theme, style, text, (tip, anchor));
        self.speech_node(theme, &picture, [bx - x, by - y])
    }

    /// The node of the balloon picture at `at` in the group.
    fn speech_node(
        &mut self,
        theme: &mut Theme,
        picture: &Picture,
        [bx, by]: [i32; 2],
    ) -> Vec<Command> {
        let props = PropsUpdate {
            image: Some(theme.render(picture)),
            transform: Some(Transform::translate(f64::from(bx), f64::from(by), 0.0)),
            opacity: Some(1.0),
            visible: Some(true),
            input: Some(InputMode::Pass),
            ..PropsUpdate::default()
        };
        if std::mem::replace(&mut self.speech, true) {
            return vec![Command::Update {
                id: SPEECH.into(),
                props,
            }];
        }
        vec![Command::Create {
            id: SPEECH.into(),
            kind: NodeKind::Image,
            parent: GROUP.into(),
            props,
        }]
    }

    /// When the next move, blink or fade is due.
    pub fn due(&self) -> Instant {
        [self.moves_at, self.fades_at]
            .into_iter()
            .flatten()
            .fold(self.blinks_at, Instant::min)
    }

    /// Moves the body, blinks and fades the balloon, as far as they are
    /// due.
    pub fn step(&mut self, theme: &mut Theme, now: Instant) -> Vec<Command> {
        let mut commands = Vec::new();
        if self.moves_at.is_some_and(|at| at <= now) {
            self.moves_at = self.pose.motion(self.step).map(|(scale, offset, ms)| {
                commands.push(Command::Animate {
                    id: BODY.into(),
                    props: PropsUpdate {
                        transform: Some(body(self.values.figure.pivot(), scale, offset)),
                        ..PropsUpdate::default()
                    },
                    duration: ms,
                    easing: Easing::EaseInOut,
                });
                self.step += 1;
                now + Duration::from_secs_f64(ms / 1000.0)
            });
        }
        if self.blinks_at <= now {
            self.shut = !self.shut;
            self.blinks_at = if self.shut {
                now + BLINK
            } else {
                self.blinks += 1;
                now + blink_gap(self.blinks)
            };
            commands.push(self.face(theme));
        }
        if self.fades_at.is_some_and(|at| at <= now) {
            self.fades_at = None;
            commands.push(Command::Animate {
                id: SPEECH.into(),
                props: PropsUpdate {
                    opacity: Some(0.0),
                    visible: Some(false),
                    ..PropsUpdate::default()
                },
                duration: FADE,
                easing: Easing::EaseIn,
            });
        }
        commands
    }
}

/// The command that creates a group with a transform.
fn group(id: &str, parent: String, transform: Transform) -> Command {
    Command::Create {
        id: id.into(),
        kind: NodeKind::Group,
        parent,
        props: PropsUpdate {
            transform: Some(transform),
            ..PropsUpdate::default()
        },
    }
}

/// Whether a picture of a size at a corner lies inside an output of a size.
fn inside([x, y]: [i32; 2], [w, h]: [i32; 2], [sw, sh]: [i32; 2]) -> bool {
    x >= 0 && y >= 0 && x + w <= sw && y + h <= sh
}

/// The top-left corner of the figure of `values` on an output of a size.
fn origin_of(values: &Values, area: [i32; 4]) -> [i32; 2] {
    origin(values.anchor, area, values.figure.size(), values.margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_balloon_hangs_below_a_figure_at_the_top() {
        let style = Style {
            tail: 18,
            ..Style::default()
        };
        let (origin, _, below) = place(
            (&style, false),
            [120, 40],
            [100, 300],
            ([0, 0, 1000, 800], 330),
        );
        assert_eq!((origin[1], below), (300 - 18 - 40, false));
        let (origin, _, below) = place(
            (&style, false),
            [120, 40],
            [100, 10],
            ([0, 0, 1000, 800], 40),
        );
        assert_eq!((origin[1], below), (40, true));
    }

    #[test]
    fn each_state_has_its_pose() {
        let cases = [
            ("idle", Pose::Idle),
            ("listening", Pose::Listening),
            ("recording", Pose::Listening),
            ("thinking", Pose::Thinking),
            ("working", Pose::Thinking),
            ("speaking", Pose::Speaking),
            ("question", Pose::Asking),
            ("confirming", Pose::Asking),
            ("sleeping", Pose::Sleeping),
            ("asleep", Pose::Sleeping),
            ("", Pose::Idle),
        ];
        for (state, want) in cases {
            assert_eq!(pose(state), want, "{state}");
        }
    }

    fn assistant(values: &Values, state: &'static str, now: Instant) -> Assistant {
        Assistant {
            values: values.clone(),
            origin: [800, 500],
            area: [0, 0, 1024, 768],
            state,
            pose: pose(state),
            shut: false,
            step: 0,
            blinks: 0,
            moves_at: None,
            blinks_at: now,
            speech: false,
            fades_at: None,
            word: [0, 0],
        }
    }

    /// While the agent sleeps, the figure stays, dimmed, so that the mic
    /// that waits for the wake words shows.
    #[test]
    fn a_sleeping_agent_shows_a_dim_figure() {
        let (mut theme, values, _) = crate::tests::loaded("sleeping");
        let now = Instant::now();
        let mut a = assistant(&values, "idle", now);
        let commands = a.set_state(&mut theme, ("asleep", "Asleep"), now);
        let dim = commands
            .iter()
            .any(|c| matches!(c, Command::Update { props, .. } if props.opacity == Some(DIM)));
        assert!(dim, "{commands:?}");
    }

    #[test]
    fn the_state_in_words_is_a_node_that_follows_the_state() {
        let (mut theme, values, _) = crate::tests::loaded("words");
        let now = Instant::now();
        let mut a = assistant(&values, "idle", now);
        assert_eq!(a.set_state(&mut theme, ("idle", "Idle"), now).len(), 0);
        let commands = a.set_state(&mut theme, ("listening", "Listening"), now);
        let said = commands.iter().any(|c| {
            matches!(c, Command::Update { id, props }
                if id == STATE && props.meta.as_ref().is_some_and(|m| m["text"] == "Listening"))
        });
        assert!(said, "{commands:?}");
    }

    #[test]
    fn a_folder_with_a_voice_text_shows_the_state_word_beside_the_figure() {
        let (mut theme, values, _) = crate::tests::loaded("word");
        let output = OutputInfo {
            name: "HEADLESS-1".into(),
            position: [0, 0],
            size: [1280, 720],
            scale: 1.0,
            camera: Transform::default(),
            work_area: [0, 0, 1280, 720],
            default_work_area: [0, 0, 1280, 720],
            is_virtual: None,
            owner: None,
        };
        let now = Instant::now();
        let (mut figure, created) = Assistant::new(
            &mut theme,
            &values,
            (&output, Anchor::BottomRight),
            ("listening", "Listening", now),
        );
        let word = |commands: &[Command], text: &str| {
            commands.iter().any(|c| match c {
                Command::Create { id, props, .. } | Command::Update { id, props } => {
                    id == WORD && props.meta.as_ref().is_some_and(|m| m["text"] == text)
                }
                _ => false,
            })
        };
        assert!(word(&created, "Listening"), "{created:?}");
        let [left, top, width, height] = figure.word_rect();
        let [fx, fy, _, fh] = figure.rect();
        assert!(left + width < fx, "left of a figure at the right");
        assert!(top >= fy - 8 && top + height <= fy + fh + 8, "beside it");
        let thinking = figure.set_state(&mut theme, ("thinking", "Thinking"), now);
        assert!(word(&thinking, "Thinking"), "{thinking:?}");
    }

    #[test]
    fn each_state_of_the_agent_has_a_name_in_the_context() {
        let cases = [
            ("sleeping", "asleep"),
            ("recording", "listening"),
            ("transcribing", "thinking"),
            ("tool", "working"),
            ("confirming", "question"),
            ("speaking", "speaking"),
            ("", "idle"),
            ("something", "idle"),
        ];
        for (state, want) in cases {
            assert_eq!(context_state(state), want, "{state}");
        }
    }

    #[test]
    fn the_anchors_put_a_picture_in_the_nine_places() {
        let at = |anchor| origin(anchor, [0, 0, 100, 80], [20, 10], [4, 6]);
        assert_eq!(at(Anchor::TopLeft), [4, 6]);
        assert_eq!(at(Anchor::Top), [40, 6]);
        assert_eq!(at(Anchor::TopRight), [76, 6]);
        assert_eq!(at(Anchor::Left), [4, 35]);
        assert_eq!(at(Anchor::Center), [40, 35]);
        assert_eq!(at(Anchor::Right), [76, 35]);
        assert_eq!(at(Anchor::BottomLeft), [4, 64]);
        assert_eq!(at(Anchor::Bottom), [40, 64]);
        assert_eq!(at(Anchor::BottomRight), [76, 64]);
    }

    #[test]
    fn a_balloon_keeps_below_a_bar_at_the_top() {
        let style = Style::default();
        let area = [0, 40, 1000, 760];
        let (origin, _, below) = place((&style, false), [120, 40], [100, 70], (area, 90));
        assert_eq!((origin[1], below), (90, true), "no room above the bar");
        let (origin, _, _) = place((&style, false), [120, 40], [995, 300], (area, 330));
        assert!(origin[0] + 120 <= 998, "{origin:?}");
    }

    #[test]
    fn anchors_use_the_work_area_not_the_output_size() {
        let area = [0, 32, 1280, 736];
        let at = |anchor| origin(anchor, area, [24, 24], [16, 16]);
        assert_eq!(at(Anchor::TopLeft), [16, 48]);
        assert_eq!(at(Anchor::BottomRight), [1240, 728]);
        let beside = origin(Anchor::TopLeft, [200, 0, 1000, 800], [24, 24], [16, 16]);
        assert_eq!(
            beside,
            [216, 16],
            "a panel at the left edge pushes the figure in"
        );
    }

    #[test]
    fn the_figure_stands_in_the_work_area_of_its_output() {
        let (mut theme, values, _) = crate::tests::loaded("area");
        let mut output = OutputInfo {
            name: "HEADLESS-1".into(),
            position: [1920, 0],
            size: [1280, 800],
            scale: 1.0,
            camera: Transform::default(),
            work_area: [1920, 0, 1280, 760],
            default_work_area: [1920, 0, 1280, 760],
            is_virtual: None,
            owner: None,
        };
        let now = Instant::now();
        let place = |output: &OutputInfo, theme: &mut Theme| {
            Assistant::new(
                theme,
                &values,
                (output, Anchor::BottomRight),
                ("idle", "Idle", now),
            )
            .0
            .rect()
        };
        assert_eq!(
            place(&output, &mut theme),
            [1240, 720, 24, 24],
            "a bar of 40 px below"
        );
        output.default_work_area = [0; 4];
        output.work_area = [1920, 40, 1280, 760];
        assert_eq!(
            place(&output, &mut theme),
            [1240, 760, 24, 24],
            "a core without the default area"
        );
    }

    #[test]
    fn a_figure_that_would_leave_the_output_is_the_neutral_dot_in_the_neutral_place() {
        let (mut theme, mut values, _) = crate::tests::loaded("outside");
        values.margin = [-40, 16];
        values.anchor = Anchor::TopLeft;
        let output = OutputInfo {
            name: "HEADLESS-1".into(),
            position: [0, 0],
            size: [1280, 800],
            scale: 1.0,
            camera: Transform::default(),
            work_area: [0, 0, 1280, 800],
            default_work_area: [0, 0, 1280, 800],
            is_virtual: None,
            owner: None,
        };
        let now = Instant::now();
        let (a, _) = Assistant::new(
            &mut theme,
            &values,
            (&output, values.anchor),
            ("idle", "Idle", now),
        );
        assert_eq!(a.rect(), [1240, 760, 24, 24]);
        assert_eq!(a.values.figure, Figure::Dot);
    }

    #[test]
    fn thinking_and_speaking_move_on_the_others_settle() {
        for pose in [Pose::Thinking, Pose::Speaking] {
            assert!(pose.motion(7).is_some(), "{pose:?}");
        }
        for pose in [Pose::Idle, Pose::Listening, Pose::Asking] {
            assert!(
                pose.motion(0).is_some() && pose.motion(1).is_none(),
                "{pose:?}"
            );
        }
    }

    #[test]
    fn the_body_grows_from_its_foot() {
        let tall = Figure::Sheet {
            sheet: chora_draw::theme::Sheet("paperclip.svg"),
            template: "{state}".into(),
            size: [80, 116],
        };
        let pivot = tall.pivot();
        assert!((pivot[1] - 116.0).abs() < 1e-9, "the foot of a tall figure");
        let (x, y) = body(pivot, [1.1, 1.2], [0.0, 0.0]).apply(pivot[0], pivot[1]);
        assert!((x - pivot[0]).abs() < 1e-9 && (y - pivot[1]).abs() < 1e-9);
    }

    #[test]
    fn a_sheet_blinks_with_its_shut_elements() {
        let has = |id: &str| ["idle", "idle-shut", "speaking"].contains(&id);
        assert_eq!(element_of(has, "{state}", "idle", false), "idle");
        assert_eq!(element_of(has, "{state}", "idle", true), "idle-shut");
        assert_eq!(element_of(has, "{state}", "speaking", true), "speaking");
        assert_eq!(element_of(has, "{state}", "working", true), "idle-shut");
        assert_eq!(element_of(|_| false, "{state}", "working", true), "idle");
    }

    #[test]
    fn a_plain_sentence_stands_beside_the_figure() {
        let area = [0, 0, 1280, 720];
        let at_right = beside([300, 26], [1232, 16, 16, 16], area);
        assert_eq!(at_right, [1232 - BESIDE - 300, 16]);
        let at_left = beside([300, 26], [32, 16, 16, 16], area);
        assert_eq!(at_left, [32 + 16 + BESIDE, 16]);
        let low = beside([300, 26], [1232, 710, 16, 16], area);
        assert_eq!(low[1], 720 - 26, "inside the area");
    }

    #[test]
    fn a_tail_at_the_left_lets_the_bubble_grow_to_the_right() {
        let style = Style {
            tail: 18,
            ..Style::default()
        };
        let area = ([0, 0, 1000, 800], 330);
        let (right, ..) = place((&style, false), [120, 40], [500, 300], area);
        let (left, ..) = place((&style, true), [120, 40], [500, 300], area);
        assert_eq!(right[0], 500 + style.tail_inset - 120);
        assert_eq!(left[0], 500 - style.tail_inset);
    }
}
