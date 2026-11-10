//! The assistant of the voice agent: a figure above the corner of the
//! first output that follows the state of the agent (D65) and speaks its
//! sentences in a bubble. The figure is a dot of the asset folder, or the
//! paperclip of the XP flavor. The core moves and scales it; the renderer
//! draws no rotation yet, so the picture itself leans. A few frames swap
//! the face.

use std::time::{Duration, Instant};

use chora_draw::{
    bubble::Style,
    text::{Font, Text, wrap},
    theme::{Picture, Theme},
};
use chora_kit::hud::{Anchor, render};
use chora_scene::{Command, Easing, InputMode, NodeKind, OutputInfo, PropsUpdate, Transform};

use crate::{
    config::{Figure, Values},
    paperclip,
};

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

/// What the paperclip does.
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

/// The opacity of the paperclip while the agent sleeps.
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
            let id = render(template, &serde_json::json!({ "state": state }));
            let id = if theme.size(*sheet, &id).is_some() {
                id
            } else {
                "idle".to_owned()
            };
            Picture::new(*w, *h).element(*sheet, &id, [0, 0, *w, *h])
        }
        Figure::Paperclip => paperclip::draw(pose(state), shut),
    }
}

impl Figure {
    /// The size of the picture.
    pub fn size(&self) -> [i32; 2] {
        match self {
            Self::Dot => DOT_SIZE,
            Self::Sheet { size, .. } => *size,
            Self::Paperclip => paperclip::SIZE,
        }
    }

    /// The point that the body turns around: the middle of a dot, the
    /// bottom of the paperclip.
    fn pivot(&self) -> [f64; 2] {
        match self {
            Self::Paperclip => paperclip::PIVOT,
            _ => self.size().map(|side| f64::from(side) / 2.0),
        }
    }

    /// Where the tail of the balloon points: above the head.
    fn head(&self) -> [i32; 2] {
        match self {
            Self::Paperclip => paperclip::HEAD,
            _ => [self.size()[0] / 2, 0],
        }
    }
}

/// The top-left corner of a picture of a size at an anchor of an output of
/// a size, with the margin to the edges.
pub fn origin(anchor: Anchor, screen: [i32; 2], size: [i32; 2], margin: [i32; 2]) -> [i32; 2] {
    let left = margin[0];
    let right = screen[0] - margin[0] - size[0];
    let centre_x = (screen[0] - size[0]) / 2;
    let top = margin[1];
    let bottom = screen[1] - margin[1] - size[1];
    let centre_y = (screen[1] - size[1]) / 2;
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
    screen: i32,
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
}

/// Where a balloon goes: its top-left corner, the x of its tail tip in it
/// and whether it hangs below the figure without a tail. A balloon sits
/// above the tip, which is a point of a screen of a width, with its tail
/// pointing down at it; where it would leave the top of the screen, it
/// sits below the bottom of the figure instead.
fn place(
    style: &Style,
    [w, h]: [i32; 2],
    [tip_x, tip_y]: [i32; 2],
    (screen, figure_bottom): (i32, i32),
) -> ([i32; 2], i32, bool) {
    let right = (tip_x + style.tail_inset).min(screen - 2);
    let left = (right - w).max(0);
    let tail = (tip_x - left).clamp(style.radius + style.tail_base, w - style.radius);
    let above = tip_y - style.tail - h;
    if above >= 0 {
        ([left, above], tail, false)
    } else {
        ([left, figure_bottom], tail, true)
    }
}

/// A balloon with text only, as the assistant speaks: its picture and its
/// top-left corner, for a tail tip at a point of a screen of a width.
fn speech(
    theme: &mut Theme,
    style: &Style,
    text: &str,
    (tip, anchor): ([i32; 2], (i32, i32)),
) -> (Picture, [i32; 2], i32) {
    let width = |s: &str| theme.text_width(s, Font::Ui);
    let lines = wrap(text, MAX_SPEECH, width);
    let text_width = lines.iter().map(|l| width(l)).fold(0.0, f64::max);
    #[allow(clippy::cast_possible_truncation)]
    let w = (PAD + (text_width.ceil() as i32) + PAD).max(MIN_WIDTH);
    let h = PAD + i32::try_from(lines.len()).unwrap_or(0) * LINE + PAD;
    let (origin, tail, below) = place(style, [w, h], tip, anchor);
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
        let mut origin = origin(anchor, output.size, values.figure.size(), values.margin);
        if !inside(origin, values.figure.size(), output.size) {
            values = Values::neutral(&values.font);
            origin = origin_of(&values, output.size);
        }
        let screen = output.size[0];
        let assistant = Self {
            values,
            origin,
            screen,
            state,
            pose: pose(state),
            shut: false,
            step: 0,
            blinks: 0,
            moves_at: Some(now),
            blinks_at: now + blink_gap(0),
            speech: false,
            fades_at: None,
        };
        let commands = assistant.created(theme, &output.name, label);
        (assistant, commands)
    }

    /// The commands that create the nodes of the figure on an output.
    fn created(&self, theme: &mut Theme, output: &str, label: &str) -> Vec<Command> {
        let [x, y] = self.origin.map(f64::from);
        let figure = &self.values.figure;
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

    /// The balloon of a sentence.
    fn balloon(&mut self, theme: &mut Theme, text: &str) -> Vec<Command> {
        let [x, y] = self.origin;
        let [head_x, head_y] = self.values.figure.head();
        let tip = [x + head_x, y + head_y];
        let anchor = (self.screen, y + self.values.figure.size()[1]);
        let (picture, [bx, by], _) = speech(theme, &self.values.speech, text, (tip, anchor));
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
fn origin_of(values: &Values, screen: [i32; 2]) -> [i32; 2] {
    origin(values.anchor, screen, values.figure.size(), values.margin)
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
        let (origin, _, below) = place(&style, [120, 40], [100, 300], (1000, 330));
        assert_eq!((origin[1], below), (300 - 18 - 40, false));
        let (origin, _, below) = place(&style, [120, 40], [100, 10], (1000, 40));
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
            screen: 1024,
            state,
            pose: pose(state),
            shut: false,
            step: 0,
            blinks: 0,
            moves_at: None,
            blinks_at: now,
            speech: false,
            fades_at: None,
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
        let at = |anchor| origin(anchor, [100, 80], [20, 10], [4, 6]);
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
        let pivot = Figure::Paperclip.pivot();
        let (x, y) = body(pivot, [1.1, 1.2], [0.0, 0.0]).apply(pivot[0], pivot[1]);
        assert!((x - pivot[0]).abs() < 1e-9 && (y - pivot[1]).abs() < 1e-9);
    }
}
