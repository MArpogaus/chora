//! The assistant of the voice agent: a paperclip with eyes and eyebrows
//! above the right end of the taskbar, which follows the state of the
//! agent (D65) and speaks its sentences in a balloon. It is drawn for
//! chora. The core moves and scales it; the renderer draws no rotation
//! yet, so the picture itself leans. A few frames swap the face.

use std::time::{Duration, Instant};

use chora_scene::{Command, Easing, InputMode, NodeKind, PropsUpdate, Transform};

use crate::{
    balloon,
    taskbar::Bar,
    theme::{Picture, Theme},
};

/// The group of the assistant under `screen:OUTPUT`.
const GROUP: &str = "xp-assistant";
/// The group that moves: leans, wobbles and bounces.
const BODY: &str = "xp-assistant-body";
/// The image of the paperclip, which takes the clicks.
const CLIP: &str = "xp-assistant-clip";
/// The balloon of the spoken sentence.
const SPEECH: &str = "xp-assistant-speech";
/// The size of the picture.
const SIZE: [i32; 2] = [80, 116];
/// The point that the body turns around: the bottom of the wire.
const PIVOT: [f64; 2] = [41.0, 110.0];
/// The distance from the right edge of the screen, left of the column of
/// desktop icons, and from the taskbar.
const MARGIN: [i32; 2] = [96, 2];
/// Where the tail of the balloon points: above the eyes.
const HEAD: [i32; 2] = [38, 10];
/// How long the eyes stay shut.
const BLINK: Duration = Duration::from_millis(140);
/// How long the balloon stays after its sentence ends.
const LINGER: Duration = Duration::from_secs(3);
/// How long the balloon takes to fade out, in milliseconds.
const FADE: f64 = 600.0;

/// The settings of the assistant.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    /// The paperclip shows while the voice agent is awake.
    pub enabled: bool,
}

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
}

/// The pose for a state of the voice agent: the state of `voice.state`
/// or of the method `state`; none while it sleeps.
pub fn pose(state: &str) -> Option<Pose> {
    Some(match state {
        "sleeping" | "asleep" => return None,
        "listening" | "recording" => Pose::Listening,
        "transcribing" | "thinking" | "tool" | "working" => Pose::Thinking,
        "speaking" => Pose::Speaking,
        "question" | "confirming" => Pose::Asking,
        _ => Pose::Idle,
    })
}

/// The face of a pose.
struct Face {
    /// The height of the eyes, 1 for the normal one.
    eyes: f64,
    /// Where the pupils look, from the middle of the eyes.
    look: [f64; 2],
    /// How far each eyebrow goes up, left and right.
    brows: [f64; 2],
}

impl Pose {
    fn face(self) -> Face {
        let (eyes, look, brows) = match self {
            Self::Idle => (1.0, [0.5, 1.0], [0.0, 0.0]),
            Self::Listening => (1.18, [-1.5, 0.0], [2.0, 2.0]),
            Self::Thinking => (1.0, [2.0, -3.6], [-1.0, 3.5]),
            Self::Speaking => (1.0, [-2.0, 0.5], [1.0, 1.0]),
            Self::Asking => (1.1, [0.0, -0.5], [6.0, 6.0]),
        };
        Face { eyes, look, brows }
    }

    /// How far the picture leans, in degrees, to the right.
    fn tilt(self) -> f64 {
        match self {
            Self::Listening => -6.0,
            Self::Asking => 4.0,
            _ => 0.0,
        }
    }

    /// The move of the body at a step of the pose: the scale, the offset
    /// and the time in milliseconds. None: it holds still.
    fn motion(self, step: u32) -> Option<([f64; 2], [f64; 2], f64)> {
        let even = step.is_multiple_of(2);
        match (self, step) {
            (Self::Idle, 0) => Some(([1.0, 1.0], [0.0, 0.0], 300.0)),
            // Towards the middle of the screen.
            (Self::Listening, 0) => Some(([1.0, 1.0], [-2.0, 0.0], 250.0)),
            (Self::Asking, 0) => Some(([1.0, 1.0], [0.0, -2.0], 250.0)),
            (Self::Thinking, _) => Some(([1.0, 1.0], [if even { 2.0 } else { -2.0 }, 0.0], 700.0)),
            (Self::Speaking, _) if even => Some(([0.98, 1.03], [0.0, -4.0], 180.0)),
            (Self::Speaking, _) => Some(([1.0, 1.0], [0.0, 0.0], 180.0)),
            _ => None,
        }
    }
}

/// The wire of the paperclip: three bends, the small one inside the
/// large one.
const WIRE: &str =
    "M33,48 V84 A8.5,8.5 0 0 0 50,84 V34 A14,14 0 0 0 22,34 V90 A19,19 0 0 0 60,90 V44";

/// The paperclip in a pose, with its eyes open or shut.
fn draw(pose: Pose, shut: bool) -> Picture {
    use std::fmt::Write as _;
    let face = pose.face();
    let (tilt, [px, py]) = (pose.tilt(), PIVOT);
    let mut svg = format!(
        r##"<defs><linearGradient id="xp-assistant-metal" x1="0" x2="1" y1="0" y2="0.3"><stop offset="0" stop-color="#f4f6f9"/><stop offset="0.45" stop-color="#b9c0ca"/><stop offset="1" stop-color="#8a929e"/></linearGradient></defs><ellipse cx="41" cy="111" rx="20" ry="3.2" fill="#000" opacity="0.22"/><g transform="rotate({tilt} {px} {py})"><path d="{WIRE}" fill="none" stroke="#3d434c" stroke-width="7" stroke-linecap="round" stroke-linejoin="round"/><path d="{WIRE}" fill="none" stroke="url(#xp-assistant-metal)" stroke-width="4.4" stroke-linecap="round"/><path d="{WIRE}" fill="none" stroke="#fff" stroke-opacity="0.75" stroke-width="1.1" stroke-linecap="round" transform="translate(-0.9,-0.6)"/>"##
    );
    let [lx, ly] = face.look;
    for ((cx, side), lift) in [(30.5, -1.0), (50.5, 1.0)].into_iter().zip(face.brows) {
        let (cy, rx, ry) = (37.0, 8.0, 9.0 * face.eyes);
        let _ = if shut {
            write!(
                svg,
                r##"<path d="M{},{} Q{cx},{} {},{}" fill="none" stroke="#1d1f24" stroke-width="1.8" stroke-linecap="round"/>"##,
                cx - rx + 1.0,
                cy + 1.0,
                cy + 6.0,
                cx + rx - 1.0,
                cy + 1.0
            )
        } else {
            write!(
                svg,
                r##"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="#fff" stroke="#1d1f24" stroke-width="1.5"/><circle cx="{}" cy="{}" r="3.4" fill="#16181c"/><circle cx="{}" cy="{}" r="1.1" fill="#fff"/>"##,
                cx + lx,
                cy + ly + 1.0,
                cx + lx + 1.2,
                cy + ly - 0.2
            )
        };
        // The eyebrow: its inner end towards the middle.
        let y = cy - ry - 5.0 - lift;
        let _ = write!(
            svg,
            r##"<path d="M{},{y} Q{cx},{} {},{}" fill="none" stroke="#2b2118" stroke-width="2.6" stroke-linecap="round"/>"##,
            cx - 5.0 * side,
            y - 3.0,
            cx + 6.5 * side,
            y + 1.0
        );
    }
    svg.push_str("</g>");
    Picture::new(SIZE[0], SIZE[1]).svg(svg)
}

/// The transform of the body: scaled from its foot and moved.
fn body([sx, sy]: [f64; 2], [dx, dy]: [f64; 2]) -> Transform {
    let [px, py] = PIVOT;
    Transform::translate(px + dx, py + dy, 0.0)
        .then(&Transform::scale(sx, sy, 1.0))
        .then(&Transform::translate(-px, -py, 0.0))
}

/// The time between two blinks, a little different each time.
fn blink_gap(blinks: u32) -> Duration {
    Duration::from_millis(2500 + u64::from(blinks.wrapping_mul(1777) % 3500))
}

/// The paperclip on the overlay root of an output.
pub struct Assistant {
    /// The top-left corner of the picture, in output coordinates.
    origin: [i32; 2],
    /// The right edge of the screen.
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
}

impl Assistant {
    /// Creates the paperclip above the right end of a taskbar.
    pub fn new(theme: &mut Theme, bar: &Bar, pose: Pose, now: Instant) -> (Self, Vec<Command>) {
        let ([_, tip_y], screen) = bar.tray_tip();
        // The tip lies 8 pixels below the top of the taskbar.
        let origin = [
            screen - SIZE[0] - MARGIN[0],
            tip_y - 8 - SIZE[1] - MARGIN[1],
        ];
        let assistant = Self {
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
                format!("screen:{}", bar.output),
                Transform::translate(x, y, 0.0),
            ),
            group(BODY, GROUP.into(), body([1.0, 1.0], [0.0, 0.0])),
            Command::Create {
                id: CLIP.into(),
                kind: NodeKind::Image,
                parent: BODY.into(),
                props: PropsUpdate {
                    image: Some(theme.render(&draw(pose, false))),
                    input: Some(InputMode::Accept),
                    ..PropsUpdate::default()
                },
            },
        ];
        (assistant, commands)
    }

    /// Whether a node is the paperclip.
    pub fn owns(node: &str) -> bool {
        node == CLIP
    }

    /// Removes the paperclip and its balloon.
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
                image: Some(theme.render(&draw(self.pose, self.shut))),
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
        let [x, y] = self.origin;
        let tip = [x + HEAD[0], y + HEAD[1]];
        let (picture, [bx, by]) = balloon::speech(theme, text, tip, self.screen);
        self.fades_at = Some(now + time + LINGER);
        let props = PropsUpdate {
            image: Some(theme.render(&picture)),
            transform: Some(Transform::translate(
                f64::from(bx - x),
                f64::from(by - y),
                0.0,
            )),
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
                        transform: Some(body(scale, offset)),
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
    fn each_state_has_its_pose() {
        let cases = [
            ("idle", Some(Pose::Idle)),
            ("listening", Some(Pose::Listening)),
            ("recording", Some(Pose::Listening)),
            ("thinking", Some(Pose::Thinking)),
            ("working", Some(Pose::Thinking)),
            ("speaking", Some(Pose::Speaking)),
            ("question", Some(Pose::Asking)),
            ("confirming", Some(Pose::Asking)),
            ("sleeping", None),
            ("asleep", None),
            ("", Some(Pose::Idle)),
        ];
        for (state, want) in cases {
            assert_eq!(pose(state), want, "{state}");
        }
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
        let (x, y) = body([1.1, 1.2], [0.0, 0.0]).apply(PIVOT[0], PIVOT[1]);
        assert!((x - PIVOT[0]).abs() < 1e-9 && (y - PIVOT[1]).abs() < 1e-9);
    }
}
