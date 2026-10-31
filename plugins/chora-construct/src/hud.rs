//! The elements over the space (F28, docs/construct-style.md): the voice
//! HUD, the questions of the voice agent, the pane label and the glass of
//! the window list. Pure builders of commands, without a connection;
//! `main.rs` sends them.
#![allow(
    clippy::cognitive_complexity,
    clippy::many_single_char_names,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::unnecessary_wraps,
    reason = "builders of nodes, one prop after the other"
)]

use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

use chora_scene::{
    Border, Clip, Command, Easing, InputMode, NodeKind, PropsUpdate, Transform, Uniform,
};
use serde::Deserialize;
use serde_json::Value;

use crate::config::{Colors, Style};

pub const LABEL: &str = "construct-label";
const LABEL_GLASS: &str = "construct-label-glass";
const LABEL_TEXT: &str = "construct-label-text";
pub const HUD: &str = "construct-hud";
const HUD_GLASS: &str = "construct-hud-glass";
pub const HUD_GLOW: &str = "construct-hud-glow";
const HUD_STATE: &str = "construct-hud-state";
pub const HUD_SAY: &str = "construct-hud-say";
pub const LIST: &str = "construct-list";
pub const LIST_GLASS: &str = "construct-list-glass";
pub const LIST_GLOW: &str = "construct-list-glow";
pub const LIST_LABEL: &str = "construct-list-label";
/// The rows of the list, made anew on each change.
pub const LIST_ROWS_GROUP: &str = "construct-list-rows";
pub const ASK: &str = "construct-ask";
/// The glass of a question takes the wheel that scrolls it.
pub const ASK_GLASS: &str = "construct-ask-glass";
pub const ASK_GLOW: &str = "construct-ask-glow";
const ASK_VIEW: &str = "construct-ask-view";
pub const ASK_BODY: &str = "construct-ask-body";

/// The three roles of type (rule 2): the size and the line.
#[derive(Clone, Copy)]
pub enum Role {
    Label,
    Data,
    Text,
}

impl Role {
    const fn size(self) -> (f64, f64) {
        match self {
            Self::Label => (11.0, 16.0),
            Self::Data => (12.0, 16.0),
            Self::Text => (16.0, 24.0),
        }
    }
}

/// The diameter of the state ring.
const RING: f64 = 40.0;
/// The widths at which a spoken sentence and a question wrap.
const SAY_WIDTH: f64 = 480.0;
const ASK_WIDTH: f64 = 640.0;
/// The window list: the rows that it shows at most, its width and its
/// line.
pub const LIST_ROWS: usize = 12;
pub const LIST_WIDTH: f64 = 560.0;
pub const LIST_LINE: f64 = 24.0;
/// The fades of rule 12, in milliseconds.
pub const FADE_IN: f64 = 160.0;
const GLOW_IN: f64 = 240.0;
pub const FADE_OUT: f64 = 120.0;
/// How long a spoken sentence stays after its end.
pub const LINGER: Duration = Duration::from_secs(2);
/// How long a notice shows.
const NOTICE: Duration = Duration::from_secs(4);
/// The most bytes of a text node (api.md).
const CHUNK: usize = 4096;
/// The most characters of a spoken sentence: two lines of 480.
const SAY_CHARS: usize = 100;

/// The ring mode of `glass.frag` and the label of a state of the voice
/// agent; an unknown state shows as `listening`, with an empty label that
/// the caller fills with the state in capitals.
#[must_use]
pub fn mode(state: &str) -> (f64, &'static str) {
    match state {
        "asleep" => (0.0, "ASLEEP"),
        "listening" => (1.0, "LISTENING"),
        "thinking" => (2.0, "THINKING"),
        "working" => (3.0, "WORKING"),
        "question" => (4.0, "QUESTION"),
        "speaking" => (5.0, "SPEAKING"),
        _ => (1.0, ""),
    }
}

/// The estimated width of a text, as the core gives no metrics before a
/// query. Labels are capitals with their spacing.
// ponytail: an average advance per character; measure with scene.query
// if a long label overflows its pill.
#[allow(clippy::cast_precision_loss, reason = "short texts")]
fn width(role: Role, text: &str) -> f64 {
    let k = match role {
        Role::Label => 0.72,
        Role::Data => 0.602,
        Role::Text => 0.5,
    };
    (text.chars().count() as f64 * role.size().0 * k).ceil()
}

/// The lines of a text wrapped at `wrap`, estimated as [`width`] does.
#[allow(clippy::cast_precision_loss, reason = "short texts")]
fn lines(role: Role, text: &str, wrap: f64) -> f64 {
    text.split('\n')
        .map(|line| (width(role, line) / wrap).ceil().max(1.0))
        .sum()
}

/// The end of a long sentence, from a word on, after an ellipsis
/// (rule 11).
#[must_use]
pub fn tail(text: &str) -> String {
    let count = text.chars().count();
    if count <= SAY_CHARS {
        return text.into();
    }
    let (cut, _) = text
        .char_indices()
        .nth(count - SAY_CHARS)
        .unwrap_or_default();
    let rest = &text[cut..];
    let words = rest.split_once(' ').map_or(rest, |(_, words)| words);
    format!("\u{2026}{words}")
}

/// A detail in pieces of at most 4 KiB, cut at line ends, which the
/// pieces leave out; a line longer than a piece is cut at a character.
#[must_use]
pub fn chunks(detail: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = detail;
    while rest.len() > CHUNK {
        let mut end = CHUNK;
        while !rest.is_char_boundary(end) {
            end -= 1;
        }
        let (piece, skip) = rest[..end].rfind('\n').map_or((end, 0), |nl| (nl, 1));
        out.push(&rest[..piece]);
        rest = &rest[piece + skip..];
    }
    out.push(rest);
    out
}

/// What the builders need of the settings.
pub struct Look<'a> {
    pub style: &'a Style,
    pub colors: &'a Colors,
    /// After the watchdog: no shader nodes.
    pub plain: bool,
    /// The path of `glass.frag`.
    pub shader: &'a str,
}

fn rgba(c: [f64; 3], a: f64) -> Option<[f64; 4]> {
    Some([c[0], c[1], c[2], a])
}

fn at(x: f64, y: f64) -> Option<Transform> {
    Some(Transform::translate(x, y, 0.0))
}

fn create(id: &str, kind: NodeKind, parent: &str, props: PropsUpdate) -> Command {
    Command::Create {
        id: id.into(),
        kind,
        parent: parent.into(),
        props,
    }
}

fn update(id: &str, props: PropsUpdate) -> Command {
    Command::Update {
        id: id.into(),
        props,
    }
}

/// Moves the opacity of a node to `to` with an eased curve; at 0 the node
/// hides at the end.
#[must_use]
pub fn fade(id: &str, to: f64, ms: f64) -> Command {
    Command::Animate {
        id: id.into(),
        props: PropsUpdate {
            opacity: Some(to),
            visible: (to <= 0.0).then_some(false),
            ..PropsUpdate::default()
        },
        duration: ms,
        easing: Easing::EaseOut,
    }
}

impl Look<'_> {
    fn pad(&self) -> f64 {
        self.style.unit * 2.0
    }

    /// A text node in a role.
    fn text(&self, id: &str, parent: &str, role: Role, text: &str, color: [f64; 3]) -> Command {
        create(
            id,
            NodeKind::Text,
            parent,
            self.text_props(role, text, color),
        )
    }

    fn text_props(&self, role: Role, text: &str, color: [f64; 3]) -> PropsUpdate {
        let (size, line) = role.size();
        let label = matches!(role, Role::Label);
        PropsUpdate {
            text: Some(if text.is_empty() { " " } else { text }.into()),
            family: Some(match role {
                Role::Data => self.style.mono.clone(),
                _ => self.style.font.clone(),
            }),
            font_size: Some(size),
            line_height: Some(line / size),
            weight: Some(if label { 500 } else { 400 }),
            letter_spacing: label.then_some(0.16),
            color: rgba(color, 1.0),
            ..PropsUpdate::default()
        }
    }

    /// The glass of a panel: the fill, the radius and the hairline.
    fn glass(&self, id: &str, parent: &str, input: Option<InputMode>) -> Command {
        create(
            id,
            NodeKind::Rect,
            parent,
            PropsUpdate {
                size: Some([1.0, 1.0]),
                color: Some(self.colors.glass),
                radius: Some(self.style.radius),
                border: Some(Some(Border {
                    width: 1.0,
                    color: self.colors.edge,
                })),
                input,
                ..PropsUpdate::default()
            },
        )
    }

    /// The glow node of a panel, faded out until its panel shows; none
    /// after the watchdog.
    fn glow(&self, id: &str, parent: &str) -> Option<Command> {
        (!self.plain).then(|| {
            create(
                id,
                NodeKind::Shader,
                parent,
                PropsUpdate {
                    shader: Some(self.shader.into()),
                    opacity: Some(0.0),
                    ..self.glow_props([1.0, 1.0], [0.0; 4], 0.0)
                },
            )
        })
    }

    /// The size, the place and the uniforms of the glow node of a panel
    /// of `size`, with the ring of the HUD.
    fn glow_props(&self, size: [f64; 2], ring: [f64; 4], level: f64) -> PropsUpdate {
        let reach = self.style.glow;
        let uniforms = self.glow_uniforms(size, self.style.radius, reach, ring, level);
        PropsUpdate {
            size: Some([size[0] + 2.0 * reach, size[1] + 2.0 * reach]),
            transform: at(-reach, -reach),
            uniforms: Some(uniforms),
            ..PropsUpdate::default()
        }
    }

    /// The uniforms of `glass.frag` for a box of `size` with corners of
    /// `radius` and a glow that reaches `reach` out of it.
    #[must_use]
    pub fn glow_uniforms(
        &self,
        size: [f64; 2],
        radius: f64,
        reach: f64,
        ring: [f64; 4],
        level: f64,
    ) -> BTreeMap<String, Uniform> {
        let accent = self.style.accent;
        BTreeMap::from([
            (
                "accent".into(),
                Uniform::Vector(vec![accent[0], accent[1], accent[2], self.colors.glow]),
            ),
            ("muted".into(), Uniform::Vector(self.colors.muted.to_vec())),
            (
                "box".into(),
                Uniform::Vector(vec![size[0], size[1], radius, reach]),
            ),
            ("ring".into(), Uniform::Vector(ring.to_vec())),
            ("level".into(), Uniform::Float(level)),
        ])
    }

    /// The glow of a panel of `size`: the update, then the rise to full
    /// (rule 12), since an update ends a running animation.
    fn glow_update(
        &self,
        id: &str,
        size: [f64; 2],
        ring: [f64; 4],
        level: f64,
        shown: bool,
    ) -> Vec<Command> {
        if self.plain {
            return Vec::new();
        }
        let props = PropsUpdate {
            visible: Some(shown),
            ..self.glow_props(size, ring, level)
        };
        vec![update(id, props), fade(id, 1.0, GLOW_IN)]
    }

    /// The groups of the elements over the space, hidden, in the stack of
    /// rule 9 from the back: the pane label, the HUD and the window list.
    /// A question comes on top when it opens.
    #[must_use]
    pub fn groups(&self, parent: &str) -> Vec<Command> {
        let group = |id: &str, input| {
            create(
                id,
                NodeKind::Group,
                parent,
                PropsUpdate {
                    input,
                    visible: Some(false),
                    opacity: Some(0.0),
                    ..PropsUpdate::default()
                },
            )
        };
        let mut commands = vec![
            group(LABEL, Some(InputMode::Pass)),
            self.glass(LABEL_GLASS, LABEL, None),
            self.text(LABEL_TEXT, LABEL, Role::Label, " ", self.colors.text),
            update(
                LABEL,
                PropsUpdate {
                    visible: Some(true),
                    opacity: Some(1.0),
                    ..PropsUpdate::default()
                },
            ),
            update(
                LABEL_GLASS,
                PropsUpdate {
                    visible: Some(false),
                    ..PropsUpdate::default()
                },
            ),
            update(
                LABEL_TEXT,
                PropsUpdate {
                    visible: Some(false),
                    ..PropsUpdate::default()
                },
            ),
            group(HUD, Some(InputMode::Pass)),
            self.glass(HUD_GLASS, HUD, None),
        ];
        commands.extend(self.glow(HUD_GLOW, HUD));
        commands.extend([
            self.text(HUD_STATE, HUD, Role::Label, " ", self.colors.muted),
            self.text(HUD_SAY, HUD, Role::Text, " ", self.colors.text),
            group(LIST, None),
            self.glass(LIST_GLASS, LIST, None),
        ]);
        commands.extend(self.glow(LIST_GLOW, LIST));
        commands.push(self.text(LIST_LABEL, LIST, Role::Label, " ", self.colors.muted));
        commands
    }

    /// The pane label of `name`: its text and its pill, which fade in, as
    /// the transform of the group moves with each frame. `fresh` starts
    /// the fade from 0. Answers the width of the pill.
    #[must_use]
    pub fn label(&self, name: &str, fresh: bool) -> (Vec<Command>, f64) {
        let text = name.to_uppercase();
        let w = width(Role::Label, &text) + self.pad() * 2.0;
        let start = PropsUpdate {
            visible: Some(true),
            opacity: fresh.then_some(0.0),
            ..PropsUpdate::default()
        };
        let commands = vec![
            update(
                LABEL_GLASS,
                PropsUpdate {
                    size: Some([w, self.style.unit * 4.0]),
                    ..start.clone()
                },
            ),
            update(
                LABEL_TEXT,
                PropsUpdate {
                    transform: at(self.pad(), self.style.unit),
                    ..PropsUpdate {
                        ..self.text_props(Role::Label, &text, self.colors.text)
                    }
                },
            ),
            update(LABEL_TEXT, start),
            fade(LABEL_GLASS, 1.0, FADE_IN),
            fade(LABEL_TEXT, 1.0, FADE_IN),
        ];
        (commands, w)
    }

    /// Fades the pane label out.
    #[must_use]
    pub fn label_hide() -> Vec<Command> {
        vec![
            fade(LABEL_GLASS, 0.0, FADE_OUT),
            fade(LABEL_TEXT, 0.0, FADE_OUT),
        ]
    }

    /// The panel of the window list for `rows` rows, at `place`, with its
    /// label; `open` fades it in.
    #[must_use]
    pub fn list(&self, size: [f64; 2], place: [f64; 2], label: &str, open: bool) -> Vec<Command> {
        let mut commands = vec![
            update(
                LIST,
                PropsUpdate {
                    transform: at(place[0], place[1]),
                    visible: Some(true),
                    meta: Some(list_meta("open")),
                    ..PropsUpdate::default()
                },
            ),
            update(
                LIST_GLASS,
                PropsUpdate {
                    size: Some(size),
                    ..PropsUpdate::default()
                },
            ),
            update(
                LIST_LABEL,
                PropsUpdate {
                    transform: at(self.pad(), self.pad()),
                    ..self.text_props(Role::Label, label, self.colors.muted)
                },
            ),
        ];
        if open {
            commands.push(fade(LIST, 1.0, FADE_IN));
            commands.extend(self.glow_update(LIST_GLOW, size, [0.0; 4], 0.0, true));
        }
        commands
    }

    /// The nodes of one row of the list in the group of the rows: a rect
    /// that takes the click, the name and the note, and the mark of the
    /// selection.
    #[must_use]
    pub fn row(&self, id: &str, top: f64, name: &str, note: &str, selected: bool) -> Vec<Command> {
        let pad = self.pad();
        let color = if selected {
            self.colors.text
        } else {
            self.colors.muted
        };
        let mut commands = vec![
            create(
                id,
                NodeKind::Rect,
                LIST_ROWS_GROUP,
                PropsUpdate {
                    transform: at(0.0, top),
                    size: Some([LIST_WIDTH, LIST_LINE]),
                    color: Some([0.0; 4]),
                    ..PropsUpdate::default()
                },
            ),
            create(
                &format!("{id}-name"),
                NodeKind::Text,
                LIST_ROWS_GROUP,
                PropsUpdate {
                    transform: at(pad, top),
                    max_width: Some(LIST_WIDTH * 0.6 - pad),
                    ..self.text_props(Role::Text, name, color)
                },
            ),
        ];
        if !note.is_empty() {
            commands.push(create(
                &format!("{id}-note"),
                NodeKind::Text,
                LIST_ROWS_GROUP,
                PropsUpdate {
                    transform: at(LIST_WIDTH * 0.62, top + 4.0),
                    ..self.text_props(Role::Data, note, self.colors.muted)
                },
            ));
        }
        if selected {
            commands.push(create(
                "construct-list-mark",
                NodeKind::Rect,
                LIST_ROWS_GROUP,
                PropsUpdate {
                    transform: at(pad / 2.0 - 1.0, top + 4.0),
                    size: Some([2.0, LIST_LINE - 8.0]),
                    color: rgba(self.style.accent, 1.0),
                    radius: Some(1.0),
                    ..PropsUpdate::default()
                },
            ));
        }
        commands
    }
}

/// The meta `state` of the list: `open`, or empty, which removes it. The
/// group stays for its fade, so tests find an open list by it.
#[must_use]
pub fn list_meta(state: &str) -> BTreeMap<String, String> {
    BTreeMap::from([("state".into(), state.into())])
}

/// Closes the window list: the meta goes at once, the panel fades out.
#[must_use]
pub fn list_close() -> Vec<Command> {
    vec![
        update(
            LIST,
            PropsUpdate {
                meta: Some(list_meta("")),
                ..PropsUpdate::default()
            },
        ),
        fade(LIST, 0.0, FADE_OUT),
    ]
}

/// The size of the window list panel with `rows` rows shown.
#[must_use]
#[allow(clippy::cast_precision_loss, reason = "a dozen rows")]
pub fn list_size(unit: f64, rows: usize) -> [f64; 2] {
    let pad = unit * 2.0;
    [
        LIST_WIDTH,
        pad * 2.0 + 16.0 + unit + LIST_LINE * rows.max(1) as f64,
    ]
}

/// The top of the first row of the list.
#[must_use]
pub fn list_top(unit: f64) -> f64 {
    unit * 2.0 + 16.0 + unit
}

/// `voice.say`: a sentence when its sound starts.
#[derive(Deserialize)]
struct Say {
    text: String,
    #[serde(default)]
    duration_ms: f64,
}

/// `voice.state`; `name` is a sub-agent, which the HUD leaves out in v1.
#[derive(Deserialize)]
struct Status {
    #[serde(default)]
    name: Option<String>,
    state: String,
    #[serde(default)]
    task: String,
}

/// `voice.confirm`: a question, or with `done` its end. The agent sends
/// a number as the id.
#[derive(Deserialize)]
struct Confirm {
    id: Value,
    #[serde(default)]
    prompt: String,
    #[serde(default)]
    detail: String,
    #[serde(default)]
    done: bool,
}

/// An open question.
pub struct Ask {
    pub id: Value,
    pub prompt: String,
    pub detail: String,
    /// How far the wheel scrolled it, and how far it can.
    pub scroll: f64,
    pub scroll_max: f64,
}

/// What the voice agent shows: its state, its sentence and its question.
#[derive(Default)]
pub struct Voice {
    /// None until the first `voice.state`: the HUD does not show.
    pub state: Option<String>,
    pub task: String,
    pub say: Option<String>,
    /// The end of the sound of the sentence, the start of its fade-out
    /// and the end of the fade.
    pub say_until: Option<Instant>,
    pub fade_at: Option<Instant>,
    pub drop_at: Option<Instant>,
    pub ask: Option<Ask>,
}

/// What an event changed.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Changed {
    pub hud: bool,
    pub ask: bool,
}

impl Voice {
    /// Follows an event `voice.KIND` of the agent `voice`.
    ///
    /// # Errors
    /// The data do not fit the event.
    pub fn follow(&mut self, kind: &str, data: Value, now: Instant) -> Result<Changed, String> {
        let err = |e: serde_json::Error| e.to_string();
        match kind {
            "state" => {
                let state: Status = serde_json::from_value(data).map_err(err)?;
                if state.name.as_deref().is_some_and(|n| n != "voice") {
                    return Ok(Changed::default());
                }
                self.state = Some(state.state);
                self.task = state.task;
                Ok(Changed {
                    hud: true,
                    ask: false,
                })
            }
            "say" => {
                let say: Say = serde_json::from_value(data).map_err(err)?;
                let len = Duration::from_secs_f64(say.duration_ms.clamp(0.0, 600_000.0) / 1000.0);
                self.say = Some(tail(&say.text));
                self.say_until = Some(now + len);
                self.fade_at = Some(now + len + LINGER);
                self.drop_at = None;
                Ok(Changed {
                    hud: true,
                    ask: false,
                })
            }
            "confirm" => {
                let c: Confirm = serde_json::from_value(data).map_err(err)?;
                if c.done {
                    let open = self.ask.as_ref().is_some_and(|a| a.id == c.id);
                    if open {
                        self.ask = None;
                    }
                    return Ok(Changed {
                        hud: false,
                        ask: open,
                    });
                }
                self.ask = Some(Ask {
                    id: c.id,
                    prompt: c.prompt,
                    detail: c.detail,
                    scroll: 0.0,
                    scroll_max: 0.0,
                });
                Ok(Changed {
                    hud: false,
                    ask: true,
                })
            }
            _ => Ok(Changed::default()),
        }
    }

    /// Shows a short notice in the text line for a few seconds, also
    /// without the voice agent.
    pub fn notice(&mut self, text: &str, now: Instant) {
        self.say = Some(text.into());
        self.say_until = None;
        self.fade_at = Some(now + NOTICE);
        self.drop_at = None;
    }

    /// The next deadline of the sentence.
    #[must_use]
    pub fn deadline(&self) -> Option<Instant> {
        [self.say_until, self.fade_at, self.drop_at]
            .into_iter()
            .flatten()
            .min()
    }

    /// Whether the waveform moves.
    #[must_use]
    pub fn level(&self, now: Instant) -> f64 {
        if self.say_until.is_some_and(|t| now < t) {
            1.0
        } else {
            0.0
        }
    }

    /// The text line of the HUD and whether it is muted: the sentence,
    /// else the task while thinking or working; compact, the sentence only.
    fn line(&self, compact: bool) -> Option<(&str, bool)> {
        if let Some(say) = &self.say {
            return Some((say, false));
        }
        let busy = matches!(self.state.as_deref(), Some("thinking" | "working"));
        (!compact && busy && !self.task.is_empty()).then_some((self.task.as_str(), true))
    }

    /// The HUD for an output of `out`: its place, the panel, the ring, the
    /// label and the text line. Compact, flown into a pane, it sits at
    /// the bottom right. Answers nothing before the first state.
    #[must_use]
    pub fn hud(
        &self,
        look: &Look,
        out: [f64; 2],
        compact: bool,
        level: f64,
        glow: bool,
    ) -> Vec<Command> {
        // Without the agent, only a notice shows, without a ring.
        let (ring_mode, label) = match &self.state {
            Some(state) => match mode(state) {
                (ring, "") => (ring, state.to_uppercase()),
                (ring, label) => (ring, label.into()),
            },
            None if self.say.is_some() => (-1.0, "NOTICE".into()),
            None => return vec![fade(HUD, 0.0, FADE_OUT)],
        };
        let ringless = look.plain || ring_mode < 0.0;
        let (u, pad) = (look.style.unit, look.pad());
        let line = self.line(compact);
        let slot = if ringless { 0.0 } else { RING + 2.0 * u };
        let (line_w, line_h) = line.map_or((0.0, 0.0), |(t, _)| {
            let w = width(Role::Text, t).min(SAY_WIDTH);
            (w, lines(Role::Text, t, SAY_WIDTH).min(2.0) * 24.0)
        });
        let content = 16.0 + line_h;
        let w = (pad * 2.0 + slot + width(Role::Label, &label).max(line_w)).ceil();
        let h = pad * 2.0 + content.max(if ringless { 0.0 } else { RING });
        let x = if compact {
            out[0] - 4.0 * u - w
        } else {
            ((out[0] - w) / 2.0).round()
        };
        let y = out[1] - 4.0 * u - h;
        let top = ((h - content) / 2.0).round();
        let opacity = if ring_mode == 0.0 { 0.6 } else { 1.0 };
        let mut commands = vec![
            update(
                HUD,
                PropsUpdate {
                    transform: at(x.round(), y),
                    visible: Some(true),
                    ..PropsUpdate::default()
                },
            ),
            fade(HUD, opacity, FADE_IN),
            update(
                HUD_GLASS,
                PropsUpdate {
                    size: Some([w, h]),
                    ..PropsUpdate::default()
                },
            ),
            update(
                HUD_STATE,
                PropsUpdate {
                    transform: at(pad + slot, top),
                    ..look.text_props(Role::Label, &label, look.colors.muted)
                },
            ),
        ];
        let say = match line {
            Some((text, muted)) => {
                let color = if muted {
                    look.colors.muted
                } else {
                    look.colors.text
                };
                PropsUpdate {
                    transform: at(pad + slot, top + 16.0),
                    max_width: Some(SAY_WIDTH),
                    visible: Some(true),
                    opacity: Some(1.0),
                    ..look.text_props(Role::Text, text, color)
                }
            }
            None => PropsUpdate {
                visible: Some(false),
                ..PropsUpdate::default()
            },
        };
        commands.push(update(HUD_SAY, say));
        let diameter = if ringless { 0.0 } else { RING };
        let ring = [pad + RING / 2.0, h / 2.0, diameter, ring_mode];
        commands.extend(look.glow_update(HUD_GLOW, [w, h], ring, level, glow));
        commands
    }

    /// The question panel at the top centre of an output of `out`, made
    /// anew: the label, the prompt, the whole detail in pieces and the
    /// hint. When it is taller than the output minus 64, a clip shows a
    /// part, and the wheel scrolls it. Sets how far it can scroll.
    #[must_use]
    pub fn ask(&mut self, look: &Look, out: [f64; 2], glow: bool) -> Vec<Command> {
        let mut commands = vec![Command::Remove {
            id: ASK.into(),
            to: None,
        }];
        let Some(ask) = &mut self.ask else {
            return commands;
        };
        let (u, pad) = (look.style.unit, look.pad());
        let w = ASK_WIDTH + pad * 2.0;
        let text = |id: &str, role, text: &str, color, y: f64| {
            create(
                id,
                NodeKind::Text,
                ASK_BODY,
                PropsUpdate {
                    transform: at(pad, y),
                    max_width: Some(ASK_WIDTH),
                    ..look.text_props(role, text, color)
                },
            )
        };
        let label = "construct-ask-label";
        let mut body = vec![text(label, Role::Label, "QUESTION", look.colors.muted, pad)];
        let mut y = pad + 16.0 + u;
        body.push(text(
            "construct-ask-prompt",
            Role::Text,
            &ask.prompt,
            look.colors.text,
            y,
        ));
        y += lines(Role::Text, &ask.prompt, ASK_WIDTH) * 24.0 + u;
        if !ask.detail.is_empty() {
            for (k, piece) in chunks(&ask.detail).into_iter().enumerate() {
                let id = format!("construct-ask-detail-{k}");
                body.push(text(&id, Role::Data, piece, look.colors.text, y));
                y += lines(Role::Data, piece, ASK_WIDTH) * 16.0;
            }
            y += u;
        }
        body.push(text(
            "construct-ask-hint",
            Role::Label,
            "ANSWER YES OR NO",
            look.colors.muted,
            y,
        ));
        let full = (y + 16.0 + pad).ceil();
        // At most the output minus 32 above and below, and minus the room
        // of the HUD with two lines, which stays in sight under it.
        let h = full.min(out[1] - 8.0 * u - (2.0 * pad + 16.0 + 48.0 + 2.0 * u));
        ask.scroll_max = full - h;
        ask.scroll = ask.scroll.clamp(0.0, ask.scroll_max);
        let x = ((out[0] - w) / 2.0).round();
        commands.extend([
            create(
                ASK,
                NodeKind::Group,
                crate::NAME,
                PropsUpdate {
                    transform: at(x, 4.0 * u),
                    opacity: Some(0.0),
                    ..PropsUpdate::default()
                },
            ),
            look.glass(ASK_GLASS, ASK, None),
            update(
                ASK_GLASS,
                PropsUpdate {
                    size: Some([w, h]),
                    ..PropsUpdate::default()
                },
            ),
        ]);
        commands.extend(look.glow(ASK_GLOW, ASK));
        let clip = (ask.scroll_max > 0.0).then_some(Clip {
            radius: look.style.radius,
        });
        commands.extend([
            create(
                ASK_VIEW,
                NodeKind::Group,
                ASK,
                PropsUpdate {
                    size: clip.map(|_| [w, h]),
                    clip: clip.map(Some),
                    input: Some(InputMode::Pass),
                    ..PropsUpdate::default()
                },
            ),
            create(
                ASK_BODY,
                NodeKind::Group,
                ASK_VIEW,
                PropsUpdate {
                    transform: at(0.0, -ask.scroll),
                    ..PropsUpdate::default()
                },
            ),
        ]);
        commands.extend(body);
        commands.push(fade(ASK, 1.0, FADE_IN));
        commands.extend(look.glow_update(ASK_GLOW, [w, h], [0.0; 4], 0.0, glow));
        commands
    }

    /// Scrolls the open question by `dy`; answers the move of its body.
    pub fn scroll(&mut self, dy: f64) -> Option<Command> {
        let ask = self.ask.as_mut().filter(|a| a.scroll_max > 0.0)?;
        ask.scroll = (ask.scroll + dy).clamp(0.0, ask.scroll_max);
        Some(update(
            ASK_BODY,
            PropsUpdate {
                transform: at(0.0, -ask.scroll.round()),
                ..PropsUpdate::default()
            },
        ))
    }
}

/// The claim of the indicator of the voice agent: due at the start and
/// after each connection of the agent.
pub struct Claim {
    pub due: bool,
}

impl Claim {
    /// Calls `call` while a claim is due. A failed call, as without an
    /// agent, leaves it due; after an event of the agent (`last`) it is no
    /// longer due.
    pub fn claim(&mut self, last: bool, call: impl FnOnce() -> bool) {
        if !self.due {
            return;
        }
        let done = call();
        self.due = !done && !last;
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::config::{Config, DEFAULT};

    fn config() -> Config {
        serde_saphyr::from_str(DEFAULT).unwrap()
    }

    fn look(config: &Config) -> Look<'_> {
        Look {
            style: &config.style,
            colors: &config.style.themes.light,
            plain: false,
            shader: "/glass.frag",
        }
    }

    #[test]
    fn each_state_has_its_ring_and_label() {
        let rows = [
            ("asleep", 0.0, "ASLEEP"),
            ("listening", 1.0, "LISTENING"),
            ("thinking", 2.0, "THINKING"),
            ("working", 3.0, "WORKING"),
            ("question", 4.0, "QUESTION"),
            ("speaking", 5.0, "SPEAKING"),
            ("dreaming", 1.0, ""),
        ];
        for (state, ring, label) in rows {
            assert_eq!(mode(state), (ring, label), "{state}");
        }
    }

    #[test]
    fn a_state_of_a_sub_agent_changes_nothing() {
        let mut voice = Voice::default();
        let now = Instant::now();
        let coder = json!({"name": "coder", "state": "working", "task": "x"});
        assert_eq!(
            voice.follow("state", coder, now).unwrap(),
            Changed::default()
        );
        assert!(voice.state.is_none());
        let own = json!({"state": "thinking", "task": "the weather"});
        assert!(voice.follow("state", own, now).unwrap().hud);
        assert_eq!(voice.state.as_deref(), Some("thinking"));
    }

    /// The port of `a_confirmation_shows_until_its_answer` of mc: a done
    /// of another id keeps the question, of its id closes it; the detail
    /// shows whole.
    #[test]
    #[allow(clippy::cognitive_complexity, reason = "one question after the other")]
    fn a_confirmation_shows_until_its_answer() {
        let config = config();
        let mut voice = Voice::default();
        let now = Instant::now();
        let detail = format!("{}\n", "x".repeat(51)).repeat(200);
        assert!(detail.len() > 10 * 1024 - 1000);
        let ask = json!({"id": 7, "prompt": "Run it?", "detail": detail});
        assert!(voice.follow("confirm", ask, now).unwrap().ask);
        let commands = voice.ask(&look(&config), [1920.0, 1080.0], true);
        let pieces: Vec<&str> = (commands.iter())
            .filter_map(|c| match c {
                Command::Create { id, props, .. } if id.starts_with("construct-ask-detail-") => {
                    props.text.as_deref()
                }
                _ => None,
            })
            .collect();
        assert_eq!(pieces.len(), 3);
        assert!(pieces.iter().all(|p| p.len() <= CHUNK));
        assert_eq!(pieces.join("\n"), detail, "no byte lost");
        assert!(
            voice.ask.as_ref().unwrap().scroll_max > 0.0,
            "taller than the output"
        );
        let done = |id| json!({"id": id, "done": true});
        assert!(!voice.follow("confirm", done(6), now).unwrap().ask);
        assert!(voice.ask.is_some(), "another id");
        assert!(voice.follow("confirm", done(7), now).unwrap().ask);
        assert!(voice.ask.is_none());
    }

    #[test]
    fn a_long_line_splits_at_a_character() {
        let line = "é".repeat(3000);
        let pieces = chunks(&line);
        assert_eq!(pieces.concat(), line);
        assert!(pieces.iter().all(|p| p.len() <= CHUNK));
    }

    #[test]
    fn a_long_sentence_keeps_its_end() {
        let long = format!("{}end", "word ".repeat(60));
        let short = tail(&long);
        assert!(short.starts_with('\u{2026}') && short.ends_with("word end"));
        assert!(short.chars().count() <= SAY_CHARS + 1);
        assert_eq!(tail("short"), "short");
    }

    /// The port of `mc_claims_the_indicator_of_voice_once_per_connection`:
    /// once at the start, again after `client.connected`, and no more
    /// after the first event.
    #[test]
    fn the_construct_claims_the_indicator_of_voice_once_per_connection() {
        let mut calls = 0;
        let mut claim = Claim { due: true };
        claim.claim(false, || {
            calls += 1;
            true
        });
        assert_eq!(calls, 1);
        // The first event of the agent: claimed already.
        claim.claim(true, || {
            calls += 1;
            true
        });
        assert_eq!(calls, 1);
        // client.connected of voice.
        claim.due = true;
        claim.claim(false, || {
            calls += 1;
            true
        });
        assert_eq!(calls, 2);
        assert!(!claim.due);
        // Without an agent the claim stays due until its first event.
        claim.due = true;
        claim.claim(false, || false);
        assert!(claim.due);
        claim.claim(true, || false);
        assert!(!claim.due);
    }

    /// At most 16 own uniforms per shader (scene.rs).
    #[test]
    fn each_shader_keeps_to_sixteen_uniforms() {
        let core = ["alpha", "size", "time", "tex"];
        for (name, text) in [
            ("fog", include_str!("../fog.frag")),
            ("pane", include_str!("../pane.frag")),
            ("mirror", include_str!("../mirror.frag")),
            ("glass", include_str!("../glass.frag")),
        ] {
            let own = (text.lines())
                .filter_map(|l| l.trim().strip_prefix("uniform "))
                .filter_map(|l| l.split_whitespace().nth(1))
                .filter(|n| !core.contains(&n.trim_end_matches(';')))
                .count();
            assert!(own <= 16, "{name}.frag has {own} uniforms");
        }
    }

    /// The layers of the elements over the space at 1920x1080, each at its
    /// largest (plan section 6).
    #[test]
    fn the_elements_keep_to_the_layer_budget() {
        let config = config();
        let look = look(&config);
        let out = [1920.0, 1080.0];
        let mut voice = Voice::default();
        let now = Instant::now();
        voice
            .follow("state", json!({"state": "speaking"}), now)
            .unwrap();
        voice
            .follow(
                "say",
                json!({"text": "word ".repeat(80), "duration_ms": 1000}),
                now,
            )
            .unwrap();
        let detail = "x\n".repeat(4000);
        voice
            .follow(
                "confirm",
                json!({"id": 1, "prompt": "?", "detail": detail}),
                now,
            )
            .unwrap();
        let mut commands = look.groups(crate::NAME);
        commands.extend(voice.hud(&look, out, false, 1.0, true));
        commands.extend(voice.ask(&look, out, true));
        commands.extend(look.label(&"w".repeat(32), true).0);
        let size = list_size(config.style.unit, LIST_ROWS);
        commands.extend(look.list(size, [0.0; 2], "WINDOWS", true));
        // The last size of each node that is a layer: a shader, a rect
        // with a radius, a group with a size.
        let mut layers: BTreeMap<String, (bool, [f64; 2])> = BTreeMap::new();
        for c in &commands {
            let (id, props, kind) = match c {
                Command::Create {
                    id, props, kind, ..
                } => (id, props, Some(kind)),
                Command::Update { id, props } => (id, props, None),
                _ => continue,
            };
            let entry = layers.entry(id.clone()).or_insert((false, [0.0; 2]));
            if let Some(kind) = kind {
                entry.0 = matches!(kind, NodeKind::Shader)
                    || (matches!(kind, NodeKind::Rect) && props.radius.unwrap_or(0.0) > 0.0)
                    || (matches!(kind, NodeKind::Group) && props.size.is_some());
            }
            if let Some(size) = props.size {
                entry.1 = size;
            }
        }
        let sum: f64 = (layers.values())
            .filter(|(layer, _)| *layer)
            .map(|(_, s)| s[0] * s[1])
            .sum();
        // The glow nodes alone, as plan section 6 counts them, and all the
        // layers: each glass rect and the clip of a question that scrolls
        // are layers too. At scale 2 that is under 11 Mi beside 8 panes of
        // 16.6 Mi, within the 64 Mi of the core.
        let glows: f64 = (layers.iter())
            .filter(|(id, _)| id.ends_with("-glow"))
            .map(|(_, (_, s))| s[0] * s[1])
            .sum();
        assert!(glows < 1.5 * 1024.0 * 1024.0, "{glows}");
        assert!(sum < 2.75 * 1024.0 * 1024.0, "{sum}");
        assert!(sum > 500_000.0, "the question counts: {sum}");
    }
}
