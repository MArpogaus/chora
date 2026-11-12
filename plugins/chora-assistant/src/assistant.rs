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
use chora_scene::{
    Command, Easing, InputMode, NodeKind, OutputInfo, PropsUpdate, Transform, Uniform,
};

use crate::{
    config::{Figure, Values},
    island::{self, Content},
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
/// The state word beside the figure, if the folder shows one.
const WORD: &str = "hud-word";
/// The shadow and the hairline of the island.
const SHADOW: &str = "hud-island-shadow";
/// The opacity of the island while the camera of the construct is in a
/// pane.
const IN_PANE: f64 = 0.6;
/// The turning or pulsing ring of the island in a busy state.
const RING: &str = "hud-island-ring";
/// The shader of that ring.
const RING_SHADER: &str = include_str!("../ring.frag");
/// The rect of the tint of the island over the blur.
const ISLAND_GLASS: &str = "hud-island-glass";
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
/// How long the island keeps a sentence after it ends.
const ISLAND_LINGER: Duration = Duration::from_secs(2);
/// How long the balloon takes to fade out, in milliseconds: a capture 200 ms
/// after it went finds it gone, never half transparent.
const FADE: f64 = 160.0;

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
            (Self::Speaking, _) if even => Some(([1.0, 1.0], [0.0, -4.0], 180.0)),
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
    /// What the island shows, for a folder with `style: island`.
    island: Option<Content>,
    /// The size of the island.
    island_size: [i32; 2],
    /// The file of `ring.frag`, none where it cannot be written: the ring
    /// then stays a still picture.
    ring_shader: Option<String>,
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
        let island = values.island.then(|| Content {
            word: label.to_owned(),
            ..Content::default()
        });
        let island_size = island.as_ref().map_or([0, 0], |content| {
            island::picture(theme, content, values.dark).size
        });
        let size = if values.island {
            island_size
        } else {
            values.figure.size()
        };
        let mut origin = origin(anchor, area, size, values.margin);
        if !inside(origin, size, output.size) {
            values = Values::neutral(&values.font);
            origin = origin_of(&values, area);
        }
        let island = island.filter(|_| values.island);
        let ring_shader = (values.island)
            .then(|| theme.cache_file("ring.frag", RING_SHADER).ok())
            .flatten()
            .map(|path| path.to_string_lossy().into_owned());
        let assistant = Self {
            values,
            origin,
            area,
            state,
            pose: pose(state),
            shut: false,
            step: 0,
            blinks: 0,
            speech: false,
            fades_at: None,
            word: [0, 0],
            island_size,
            ring_shader,
            moves_at: None,
            blinks_at: now + Duration::from_secs(86_400),
            island,
        };
        let assistant = if assistant.island.is_some() {
            assistant
        } else {
            Self {
                moves_at: Some(now),
                blinks_at: now + blink_gap(0),
                ..assistant
            }
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
        let under: Vec<Command> = self
            .shadow(theme, true)
            .into_iter()
            .chain(self.island_backdrop(true))
            .collect();
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
        ]
        .into_iter()
        .chain(under)
        .chain([
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
                    image: Some(theme.render(&self.picture(theme, figure))),
                    opacity: Some(self.opacity()),
                    meta: self.island_words(),
                    input: Some(InputMode::Accept),
                    ..PropsUpdate::default()
                },
            },
        ])
        .chain(self.ring(true))
        .chain(word)
        .collect()
    }

    /// The rect that holds the tint of the island over the blur of the
    /// core; none for a figure or without the blur.
    fn island_backdrop(&self, fresh: bool) -> Option<Command> {
        self.island.as_ref()?;
        let [w, h] = self.island_size.map(f64::from);
        let m = chora_kit::glass::Material::of(self.values.dark);
        let radius = h.min(chora_kit::glass::CAPSULE) / 2.0;
        crate::glass::backdrop(
            (ISLAND_GLASS, GROUP),
            [0.0, 0.0, w, h],
            (radius, false),
            (&m, fresh),
        )
    }

    /// The picture of the figure: the island with its ring, or the figure
    /// in its state.
    fn picture(&self, theme: &Theme, figure: &Figure) -> Picture {
        let Some(content) = self.shown_island() else {
            return draw(theme, figure, self.state, self.shut);
        };
        let picture = island::picture(theme, &content, self.values.dark);
        if self.ring_moves() {
            return picture;
        }
        let ring = island::ring_rect(picture.size[1]);
        match figure {
            Figure::Sheet {
                sheet, template, ..
            } => {
                let has = |id: &str| theme.size(*sheet, id).is_some();
                let id = element_of(has, template, self.state, false);
                picture.element(*sheet, &id, ring)
            }
            Figure::Dot => picture.svg(format!(
                r#"<g transform="translate({} {})">{}</g>"#,
                ring[0],
                ring[1],
                dot_group(self.state)
            )),
        }
    }

    /// Whether the ring of the island moves: a busy state, with its shader.
    fn ring_moves(&self) -> bool {
        self.island.is_some()
            && self.ring_shader.is_some()
            && island::ring_mode(self.state).is_some()
    }

    /// The shader node of the ring of the island, made or changed; none for
    /// a figure or without the shader.
    fn ring(&self, fresh: bool) -> Option<Command> {
        self.island.as_ref()?;
        let shader = self.ring_shader.clone()?;
        let [x, y, w, h] = island::ring_rect(self.island_size[1]).map(f64::from);
        let accent = chora_kit::glass::Material::of(self.values.dark).accent;
        let mode = island::ring_mode(self.state);
        let uniforms = [
            ("accent".to_owned(), Uniform::Vector(accent[..3].to_vec())),
            ("mode".to_owned(), Uniform::Float(mode.unwrap_or(0.0))),
        ];
        let props = PropsUpdate {
            shader: fresh.then_some(shader),
            transform: Some(Transform::translate(x, y, 0.0)),
            size: Some([w, h]),
            uniforms: Some(uniforms.into_iter().collect()),
            visible: Some(mode.is_some()),
            input: Some(InputMode::Pass),
            ..PropsUpdate::default()
        };
        Some(if fresh {
            Command::Create {
                id: RING.into(),
                kind: NodeKind::Shader,
                parent: GROUP.into(),
                props,
            }
        } else {
            Command::Update {
                id: RING.into(),
                props,
            }
        })
    }

    /// What the island shows now: the task only while the agent thinks or
    /// works.
    fn shown_island(&self) -> Option<Content> {
        let mut content = self.island.clone()?;
        if !matches!(self.state, "thinking" | "working") {
            content.task.clear();
        }
        Some(content)
    }

    /// The words that the island shows, for readers of the scene.
    fn island_words(&self) -> Option<std::collections::BTreeMap<String, String>> {
        let content = self.shown_island()?;
        let words = [Some(content.word), Some(content.task), content.sentence]
            .into_iter()
            .flatten()
            .filter(|words| !words.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        Some([("text".to_owned(), words)].into())
    }

    /// The opacity of the figure: the island stays clear while it sleeps.
    fn opacity(&self) -> f64 {
        if self.island.is_some() {
            1.0
        } else {
            self.pose.opacity()
        }
    }

    /// The shadow and the hairline of the island, made or moved; none for
    /// a figure.
    fn shadow(&self, theme: &mut Theme, fresh: bool) -> Option<Command> {
        self.island.as_ref()?;
        let [w, h] = self.island_size.map(f64::from);
        let m = chora_kit::glass::Material::of(self.values.dark);
        let path = crate::glass::shadow(
            theme,
            [w, h],
            h.min(chora_kit::glass::CAPSULE) / 2.0,
            &m,
            false,
        );
        let reach = f64::from(crate::glass::REACH);
        let props = PropsUpdate {
            image: Some(path),
            transform: Some(Transform::translate(-reach, -reach, 0.0)),
            input: Some(InputMode::Pass),
            ..PropsUpdate::default()
        };
        Some(if fresh {
            Command::Create {
                id: SHADOW.into(),
                kind: NodeKind::Image,
                parent: GROUP.into(),
                props,
            }
        } else {
            Command::Update {
                id: SHADOW.into(),
                props,
            }
        })
    }

    /// The island anew after its content changed: its size, its place at
    /// the anchor of the folder, its picture and its shadow.
    fn relayout(&mut self, theme: &mut Theme) -> Vec<Command> {
        let Some(content) = self.shown_island() else {
            return Vec::new();
        };
        let size = island::picture(theme, &content, self.values.dark).size;
        self.island_size = size;
        self.origin = origin(self.values.anchor, self.area, size, self.values.margin);
        let [x, y] = self.origin.map(f64::from);
        let mut commands = vec![
            Command::Update {
                id: GROUP.into(),
                props: PropsUpdate {
                    transform: Some(Transform::translate(x, y, 0.0)),
                    ..PropsUpdate::default()
                },
            },
            self.face(theme),
        ];
        commands.extend(self.shadow(theme, false));
        commands.extend(self.island_backdrop(false));
        commands.extend(self.ring(false));
        commands
    }

    /// Shows or hides the island: an open glass chat is its expanded form
    /// and takes its place (one surface, U3). None for a figure.
    pub fn island_shown(&self, shown: bool) -> Option<Command> {
        self.island.as_ref()?;
        Some(Command::Update {
            id: GROUP.into(),
            props: PropsUpdate {
                visible: Some(shown),
                ..PropsUpdate::default()
            },
        })
    }

    /// The island dimmed inside a pane of the construct.
    pub fn island_dimmed(&self, in_pane: bool) -> Option<Command> {
        self.island.as_ref()?;
        Some(Command::Animate {
            id: GROUP.into(),
            props: PropsUpdate {
                opacity: Some(if in_pane { IN_PANE } else { 1.0 }),
                ..PropsUpdate::default()
            },
            duration: chora_kit::glass::APPEAR_MS,
            easing: Easing::EaseOut,
        })
    }

    /// The task of the helper that the island shows after the state word.
    pub fn set_task(&mut self, theme: &mut Theme, task: &str) -> Vec<Command> {
        match &mut self.island {
            Some(content) if content.task != task => task.clone_into(&mut content.task),
            _ => return Vec::new(),
        }
        self.relayout(theme)
    }

    /// The picture of the state word on the backing of a plain sentence,
    /// beside the figure, toward the middle of the area; none for a folder
    /// without a word.
    fn word(&mut self, theme: &mut Theme, label: &str) -> Option<PropsUpdate> {
        if self.island.is_some() {
            return None;
        }
        let template = self.values.word.as_deref()?;
        let text = chora_kit::hud::render(template, &serde_json::json!({ "state": label }));
        let font = self.word_font();
        #[allow(
            clippy::cast_possible_truncation,
            reason = "a word is narrower than a screen"
        )]
        let size = [
            theme.text_width(&text, font).ceil() as i32 + 2 * BACKING,
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
        let colour = self.values.word_ink.map_or_else(
            || self.values.speech.text_rgba(),
            |[red, green, blue]| [red, green, blue, 255],
        );
        let picture = Picture::new(w, h)
            .svg(backing)
            .text(Text::new(&text, font, colour, place));
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

    /// The font of the state word: text that a person reads, 14 pixels,
    /// where the folder names a slot; the small one of the status line
    /// beside a figure otherwise.
    fn word_font(&self) -> Font {
        if self.values.slot {
            Font::Body
        } else {
            Font::Ui
        }
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

    /// The head of the figure on the output, where a tail points.
    pub fn tip(&self) -> [i32; 2] {
        let [hx, hy] = self.values.figure.head();
        [self.origin[0] + hx, self.origin[1] + hy]
    }

    /// The rectangle of the figure on the output: x, y, width, height.
    pub fn rect(&self) -> [i32; 4] {
        let [w, h] = if self.island.is_some() {
            self.island_size
        } else {
            self.values.figure.size()
        };
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
        if let Some(content) = &mut self.island {
            label.clone_into(&mut content.word);
            let mut commands = self.relayout(theme);
            commands.push(Command::Update {
                id: STATE.into(),
                props: PropsUpdate {
                    meta: Some([("text".to_owned(), label.to_owned())].into()),
                    ..PropsUpdate::default()
                },
            });
            return commands;
        }
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
        let picture = self.picture(theme, &self.values.figure);
        Command::Update {
            id: CLIP.into(),
            props: PropsUpdate {
                image: Some(theme.render(&picture)),
                opacity: Some(self.opacity()),
                meta: self.island_words(),
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
        if let Some(content) = &mut self.island {
            content.sentence = Some(text.to_owned());
            self.fades_at = Some(now + time + ISLAND_LINGER);
            return self.relayout(theme);
        }
        self.balloon(theme, text)
    }

    /// Hides the balloon of the sentence at once, while the chat is the
    /// open balloon of the figure.
    pub fn hush(&mut self) -> Vec<Command> {
        self.fades_at = None;
        if !self.speech {
            return Vec::new();
        }
        vec![Command::Update {
            id: SPEECH.into(),
            props: PropsUpdate {
                visible: Some(false),
                ..PropsUpdate::default()
            },
        }]
    }

    /// Whether a node is the balloon of the sentence.
    pub fn speaks(node: &str) -> bool {
        node == SPEECH
    }

    /// The part of the output that the figure and its balloon may take:
    /// x, y, width, height. A balloon is at most `MAX_SPEECH` wide and
    /// stands above or below the figure.
    pub fn reach(&self) -> [i32; 4] {
        if self.island.is_some() {
            let [x, y, _, _] = self.rect();
            #[allow(clippy::cast_possible_truncation, reason = "a few hundred pixels")]
            let widest = island::SENTENCE_WIDTH as i32 + 2 * 48;
            let right = x + self.island_size[0];
            return [right - widest, y, widest, 64];
        }
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
            input: Some(if self.values.chats {
                InputMode::Accept
            } else {
                InputMode::Pass
            }),
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
        if self.fades_at.is_some_and(|at| at <= now) && self.island.is_some() {
            self.fades_at = None;
            if let Some(content) = &mut self.island {
                content.sentence = None;
            }
            commands.extend(self.relayout(theme));
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
            island: None,
            island_size: [0, 0],
            ring_shader: None,
        }
    }

    /// Inside a pane of the construct the island dims to 0.6 (HUD spec,
    /// section 6), and back to 1 outside it; a figure stays as it is.
    #[test]
    fn the_island_dims_inside_a_pane() {
        let values = Values::neutral("sans-serif");
        let now = Instant::now();
        let mut island = assistant(&values, "listening", now);
        island.island = Some(Content::default());
        let opacity = |command: Option<Command>| match command {
            Some(Command::Animate { id, props, .. }) if id == GROUP => props.opacity,
            _ => None,
        };
        assert_eq!(opacity(island.island_dimmed(true)), Some(0.6));
        assert_eq!(opacity(island.island_dimmed(false)), Some(1.0));
        let figure = assistant(&values, "listening", now);
        assert!(figure.island_dimmed(true).is_none());
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
    fn the_balloon_fades_in_160_ms_so_that_it_never_rests_translucent() {
        let (mut theme, values, _) = crate::tests::loaded("fade");
        let now = Instant::now();
        let mut a = assistant(&values, "idle", now);
        a.fades_at = Some(now);
        let commands = a.step(&mut theme, now);
        let fade = commands.iter().find_map(|c| match c {
            Command::Animate {
                id,
                props,
                duration,
                ..
            } if id == SPEECH && props.opacity == Some(0.0) => Some(*duration),
            _ => None,
        });
        assert!(fade.is_some_and(|ms| ms <= 160.0), "{commands:?}");
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

    #[test]
    fn the_body_never_scales_so_its_picture_is_drawn_at_the_size_of_its_file() {
        for pose in [
            Pose::Idle,
            Pose::Listening,
            Pose::Asking,
            Pose::Thinking,
            Pose::Speaking,
            Pose::Sleeping,
        ] {
            for step in 0..4 {
                if let Some((scale, offset, _)) = pose.motion(step) {
                    assert_eq!(scale, [1.0, 1.0], "{pose:?} step {step}");
                    assert!(offset.iter().all(|o| o.fract() == 0.0), "{pose:?}");
                }
            }
        }
    }
}
