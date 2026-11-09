//! The assistant of the voice agent: a figure above the corner of the
//! first output that follows the state of the agent (D65) and speaks its
//! sentences in a bubble. The figure is a dot of the asset folder, or the
//! paperclip of the XP flavor. The core moves and scales it; the renderer
//! draws no rotation yet, so the picture itself leans. A few frames swap
//! the face.

use std::time::{Duration, Instant};

use chora_draw::{
    bubble::{Corner, Style},
    text::{Font, Text, wrap},
    theme::{Picture, Rect, Sheet, Theme},
};
use chora_scene::{Command, Easing, InputMode, NodeKind, OutputInfo, PropsUpdate, Transform};

use crate::{
    config::{Figure, Values},
    paperclip,
};

/// The group of the assistant under `screen:OUTPUT`.
const GROUP: &str = "assistant";
/// The group that moves: leans, wobbles and bounces.
const BODY: &str = "assistant-body";
/// The image of the figure, which takes the clicks.
const CLIP: &str = "assistant-clip";
/// The balloon of the spoken sentence.
const SPEECH: &str = "assistant-speech";
/// The buttons of a question in the balloon: a click answers it with
/// `voice.confirm.answer`.
const YES: &str = "assistant-yes";
const NO: &str = "assistant-no";
/// The gap between the buttons and above them.
const BUTTON_GAP: i32 = 6;
/// The size of a button of a question.
const BUTTON: [i32; 2] = [73, 21];
const PAD: i32 = 10;
const LINE: i32 = 13;
const MIN_WIDTH: i32 = 120;
/// The widest line of the assistant's speech.
const MAX_SPEECH: f64 = 220.0;

/// The sheet of the dot: one element per pose, named by [`Pose::id`].
pub const DOT: Sheet = Sheet("dot.svg");
/// The size of the dot.
const DOT_SIZE: [i32; 2] = [24, 24];

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

impl Pose {
    /// The id of the element of the dot for the pose.
    fn id(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Listening => "listening",
            Self::Thinking => "thinking",
            Self::Speaking => "speaking",
            Self::Asking => "asking",
            Self::Sleeping => "sleeping",
        }
    }

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

/// The figure in a pose, with its eyes open or shut.
fn draw(look: Figure, pose: Pose, shut: bool) -> Picture {
    match look {
        Figure::Dot => {
            let [w, h] = DOT_SIZE;
            Picture::new(w, h).element(DOT, pose.id(), [0, 0, w, h])
        }
        Figure::Paperclip => paperclip::draw(pose, shut),
    }
}

impl Figure {
    /// The size of the picture.
    fn size(self) -> [i32; 2] {
        match self {
            Self::Dot => DOT_SIZE,
            Self::Paperclip => paperclip::SIZE,
        }
    }

    /// The point that the body turns around: the bottom of the figure.
    fn pivot(self) -> [f64; 2] {
        match self {
            Self::Dot => [12.0, 12.0],
            Self::Paperclip => paperclip::PIVOT,
        }
    }

    /// Where the tail of the balloon points: above the head.
    fn head(self) -> [i32; 2] {
        match self {
            Self::Dot => [12, 0],
            Self::Paperclip => paperclip::HEAD,
        }
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
    /// The id of the spoken confirmation that the balloon shows until
    /// its answer.
    asking: Option<String>,
    /// The confirmation has the buttons Ja and Nein.
    buttons: bool,
}

/// What a question in the balloon offers besides the voice: the buttons
/// Ja and Nein, and the keys of a question about a security key.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Offer {
    pub buttons: bool,
    /// The key of yes and of no, such as `Alt+y`.
    pub keys: Option<[String; 2]>,
}

/// A key as XP writes it: `Alt+y` as `Alt+Y`.
fn key_name(key: &str) -> String {
    match key.rsplit_once('+') {
        Some((m, k)) if k.chars().count() == 1 => format!("{m}+{}", k.to_uppercase()),
        _ => key.to_owned(),
    }
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
/// `room` is a width that the inside keeps at least and a height under
/// the text, for the buttons of a question.
fn speech(
    theme: &mut Theme,
    style: &Style,
    text: &str,
    (tip, anchor): ([i32; 2], (i32, i32)),
    [room_w, room_h]: [i32; 2],
) -> (Picture, [i32; 2], i32) {
    let width = |s: &str| theme.text_width(s, Font::Ui);
    let lines = wrap(text, MAX_SPEECH, width);
    let text_width = lines.iter().map(|l| width(l)).fold(0.0, f64::max);
    #[allow(clippy::cast_possible_truncation)]
    let w = (PAD + (text_width.ceil() as i32).max(room_w) + PAD).max(MIN_WIDTH);
    let h = PAD + i32::try_from(lines.len()).unwrap_or(0) * LINE + room_h + PAD;
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

/// A push button of a question: a rounded rectangle with its label.
fn button(style: &Style, label: &str) -> Picture {
    let [w, h] = BUTTON;
    let flat = Style {
        tail: 0,
        ..style.clone()
    };
    let rect: Rect = [0, 0, w, h];
    Picture::new(w, h)
        .svg(flat.shape([w, h], 0))
        .text(Text::new(label, Font::Ui, flat.text_rgba(), rect).centered())
}

impl Assistant {
    /// Creates the figure in a corner of an output.
    pub fn new(
        theme: &mut Theme,
        values: &Values,
        (output, corner): (&OutputInfo, Corner),
        (pose, now): (Pose, Instant),
    ) -> (Self, Vec<Command>) {
        let screen = output.size[0];
        let origin = corner.origin(output.size, values.figure.size(), values.margin);
        let assistant = Self {
            values: values.clone(),
            origin,
            screen,
            pose,
            shut: false,
            step: 0,
            blinks: 0,
            moves_at: Some(now),
            blinks_at: now + blink_gap(0),
            speech: false,
            fades_at: None,
            asking: None,
            buttons: false,
        };
        let group = |id: &str, parent: String, transform| Command::Create {
            id: id.into(),
            kind: NodeKind::Group,
            parent,
            props: PropsUpdate {
                transform: Some(transform),
                ..PropsUpdate::default()
            },
        };
        let [x, y] = origin.map(f64::from);
        let commands = vec![
            group(
                GROUP,
                format!("screen:{}", output.name),
                Transform::translate(x, y, 0.0),
            ),
            group(
                BODY,
                GROUP.into(),
                body(values.figure.pivot(), [1.0, 1.0], [0.0, 0.0]),
            ),
            Command::Create {
                id: CLIP.into(),
                kind: NodeKind::Image,
                parent: BODY.into(),
                props: PropsUpdate {
                    image: Some(theme.render(&draw(values.figure, pose, false))),
                    opacity: Some(pose.opacity()),
                    input: Some(InputMode::Accept),
                    ..PropsUpdate::default()
                },
            },
        ];
        (assistant, commands)
    }

    /// Whether a node is the figure.
    pub fn owns(node: &str) -> bool {
        node == CLIP
    }

    /// Removes the figure and its balloon.
    pub fn remove() -> Command {
        Command::Remove {
            id: GROUP.into(),
            to: None,
        }
    }

    /// Takes another pose; its moves start at once.
    pub fn set_pose(&mut self, theme: &mut Theme, pose: Pose, now: Instant) -> Vec<Command> {
        if pose == self.pose {
            return Vec::new();
        }
        self.pose = pose;
        self.step = 0;
        self.moves_at = Some(now);
        vec![self.face(theme)]
    }

    fn face(&self, theme: &mut Theme) -> Command {
        Command::Update {
            id: CLIP.into(),
            props: PropsUpdate {
                image: Some(theme.render(&draw(self.values.figure, self.pose, self.shut))),
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
        self.balloon(theme, text, false)
    }

    /// The balloon of a sentence or of a question, with room for the
    /// buttons of a question.
    fn balloon(&mut self, theme: &mut Theme, text: &str, buttons: bool) -> Vec<Command> {
        let [x, y] = self.origin;
        let [head_x, head_y] = self.values.figure.head();
        let tip = [x + head_x, y + head_y];
        let room = if buttons {
            [2 * BUTTON[0] + BUTTON_GAP, BUTTON[1] + BUTTON_GAP]
        } else {
            [0, 0]
        };
        let anchor = (self.screen, y + self.values.figure.size()[1]);
        let (picture, [bx, by], tail) =
            speech(theme, &self.values.speech, text, (tip, anchor), room);
        let mut commands = Vec::new();
        if std::mem::replace(&mut self.buttons, buttons) {
            commands.extend([YES, NO].map(|id| Command::Remove {
                id: id.into(),
                to: None,
            }));
        }
        if buttons {
            let top = by - y + picture.size[1] - tail - PAD - BUTTON[1];
            let right = bx - x + picture.size[0] - PAD;
            let lefts = [right - 2 * BUTTON[0] - BUTTON_GAP, right - BUTTON[0]];
            for ((id, label), left) in [(YES, "Ja"), (NO, "Nein")].into_iter().zip(lefts) {
                let look = button(&self.values.buttons, label);
                commands.push(Command::Create {
                    id: id.into(),
                    kind: NodeKind::Image,
                    parent: GROUP.into(),
                    props: PropsUpdate {
                        image: Some(theme.render(&look)),
                        transform: Some(Transform::translate(f64::from(left), f64::from(top), 0.0)),
                        input: Some(InputMode::Accept),
                        ..PropsUpdate::default()
                    },
                });
            }
        }
        let mut speech = self.speech_node(theme, &picture, [bx - x, by - y]);
        speech.extend(commands);
        speech
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

    /// Shows the question of a spoken confirmation in the balloon until
    /// [`Self::answered`] with the same id: with the buttons Ja and Nein
    /// when the agent takes clicks, and with one line of the keys of a
    /// question about a security key.
    pub fn ask(&mut self, theme: &mut Theme, id: &str, text: &str, offer: &Offer) -> Vec<Command> {
        let text = match &offer.keys {
            Some([yes, no]) => format!("{text}\n{} = Ja, {} = Nein", key_name(yes), key_name(no)),
            None => text.to_owned(),
        };
        let commands = self.balloon(theme, &text, offer.buttons);
        self.fades_at = None;
        self.asking = Some(id.into());
        commands
    }

    /// The confirmation `id` is answered or timed out: its balloon fades,
    /// and its buttons go.
    pub fn answered(&mut self, id: &str, now: Instant) -> Vec<Command> {
        if self.asking.as_deref() != Some(id) {
            return Vec::new();
        }
        self.asking = None;
        self.fades_at = Some(now);
        if !std::mem::replace(&mut self.buttons, false) {
            return Vec::new();
        }
        [YES, NO]
            .map(|id| Command::Remove {
                id: id.into(),
                to: None,
            })
            .into()
    }

    /// The parameters of `voice.confirm.answer` for a press on `node`: a
    /// button of the open question. Only a pointer press on the button
    /// calls it; no method of xp does.
    pub fn answer(&self, node: &str) -> Option<serde_json::Value> {
        let yes = match node {
            YES => true,
            NO => false,
            _ => return None,
        };
        let id = self.asking.as_ref().filter(|_| self.buttons)?;
        Some(serde_json::json!({"id": id, "yes": yes}))
    }

    /// Whether the balloon shows a confirmation.
    pub fn asking(&self) -> bool {
        self.asking.is_some()
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

    fn assistant(values: &Values, pose: Pose, now: Instant) -> Assistant {
        Assistant {
            values: values.clone(),
            origin: [800, 500],
            screen: 1024,
            pose,
            shut: false,
            step: 0,
            blinks: 0,
            moves_at: None,
            blinks_at: now,
            speech: false,
            fades_at: None,
            asking: None,
            buttons: false,
        }
    }

    /// While the agent sleeps, the figure stays, dimmed, so that the mic
    /// that waits for the wake words shows.
    #[test]
    fn a_sleeping_agent_shows_a_dim_figure() {
        let (mut theme, values, _) = crate::tests::loaded("sleeping");
        let now = Instant::now();
        let mut a = assistant(&values, Pose::Idle, now);
        let commands = a.set_pose(&mut theme, Pose::Sleeping, now);
        let dim = commands
            .iter()
            .any(|c| matches!(c, Command::Update { props, .. } if props.opacity == Some(DIM)));
        assert!(dim, "{commands:?}");
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

    /// A confirmation stays until its own answer, then fades.
    #[test]
    #[allow(clippy::cognitive_complexity, reason = "one answer after the other")]
    fn a_confirmation_stays_until_its_answer() {
        let (mut theme, values, _) = crate::tests::loaded("confirm");
        let now = Instant::now();
        let mut a = assistant(&values, Pose::Asking, now);
        let none = Offer::default();
        assert_eq!(
            a.ask(&mut theme, "c1", "Run it?\nrm -r build", &none).len(),
            1
        );
        assert!(a.asking() && a.fades_at.is_none());
        assert_eq!(a.answer(YES), None, "no buttons");
        a.answered("c2", now);
        assert!(a.asking(), "another id");
        a.answered("c1", now);
        assert!(!a.asking() && a.fades_at == Some(now));
        let offer = Offer {
            buttons: true,
            keys: Some(["Alt+y".into(), "Alt+n".into()]),
        };
        let commands = a.ask(&mut theme, "c3", "Set allow?", &offer);
        let made: Vec<&str> = (commands.iter())
            .filter_map(|c| match c {
                Command::Create { id, .. } => Some(id.as_str()),
                _ => None,
            })
            .collect();
        assert!(made.contains(&YES) && made.contains(&NO), "{made:?}");
        let json = |yes| serde_json::json!({"id": "c3", "yes": yes});
        assert_eq!(a.answer(YES), Some(json(true)));
        assert_eq!(a.answer(NO), Some(json(false)));
        assert_eq!(a.answer(CLIP), None);
        assert_eq!(a.answered("c3", now).len(), 2, "the buttons go");
        assert_eq!(a.answer(YES), None, "answered");
    }

    #[test]
    fn the_body_grows_from_its_foot() {
        let pivot = Figure::Paperclip.pivot();
        let (x, y) = body(pivot, [1.1, 1.2], [0.0, 0.0]).apply(pivot[0], pivot[1]);
        assert!((x - pivot[0]).abs() < 1e-9 && (y - pivot[1]).abs() < 1e-9);
    }
}
