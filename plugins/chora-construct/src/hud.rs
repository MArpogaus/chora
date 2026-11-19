//! The elements over the space (F28, docs/construct-style.md): the
//! notices, the pane label and the glass of the window list; the HUD host
//! draws the voice HUD and the questions. Pure builders
//! of commands, without a connection; `main.rs` sends them.
#![allow(
    clippy::cognitive_complexity,
    clippy::many_single_char_names,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::unnecessary_wraps,
    reason = "builders of nodes, one prop after the other"
)]

use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    time::{Duration, Instant},
};

use chora_scene::{Border, Command, Easing, InputMode, NodeKind, PropsUpdate, Transform, Uniform};

use crate::config::{Colors, Style};

pub const LABEL: &str = "construct-label";
const LABEL_GLASS: &str = "construct-label-glass";
const LABEL_TEXT: &str = "construct-label-text";
/// The glass capsule of the pane label and its text, made anew for each
/// pane.
pub const LABEL_BODY: &str = "construct-label-body";
const LABEL_TEXT_BODY: &str = "construct-label-body-text";
pub const LIST: &str = "construct-list";
pub const LIST_GLASS: &str = "construct-list-glass";
pub const LIST_GLOW: &str = "construct-list-glow";
pub const LIST_LABEL: &str = "construct-list-label";
/// The help panel with the bindings.
pub const HELP: &str = "construct-help";
/// The roles of type: the label (rule 2), and the faces of the glass HUD
/// (rule 23) in a weight.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    Label,
    Face(chora_kit::glass::Type, u16),
}

impl Role {
    const fn size(self) -> (f64, f64) {
        match self {
            Self::Label => (11.0, 16.0),
            Self::Face(face, _) => (face.face().size, face.face().line),
        }
    }
}

/// The window list: the rows that it shows at most.
pub const LIST_ROWS: usize = 12;
/// The fades of rule 12, in milliseconds.
pub const FADE_IN: f64 = 160.0;
pub const FADE_OUT: f64 = 120.0;
/// How long a notice shows.
const NOTICE: Duration = Duration::from_secs(4);
/// A chip of the cube: a capsule 32 high with its words in `small` 500.
const CHIP_HEIGHT: f64 = 32.0;
const CHIP_ROLE: Role = Role::Face(chora_kit::glass::Type::Small, 500);

fn rgb3(c: [f64; 4]) -> [f64; 3] {
    [c[0], c[1], c[2]]
}

/// The opacity of the label of a pane that is not in focus.
pub const NEIGHBOUR_OPACITY: f64 = 0.6;

/// The id of the label of pane `k` that is not in focus.
#[must_use]
pub fn neighbour_id(k: usize) -> String {
    format!("construct-label-n-{k}")
}

/// The width of the pill of a pane label at most.
const LABEL_MAX: f64 = 320.0;
/// The most characters of a single line that the fit tries: more never
/// fits a panel of the guide.
const LINE_CHARS: usize = 120;
/// The most characters of the end of a text that the fit tries: three
/// lines of [`LINE_CHARS`], more than a sentence or a reason shows.
const END_CHARS: usize = 3 * LINE_CHARS;
/// The ellipsis of a cut text (rule 11).
const ELLIPSIS: char = '\u{2026}';

/// A text that the core measures: its role, the text, and the width it
/// wraps at, as the bits of the f64.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Probe {
    pub role: Role,
    pub text: String,
    pub wrap: Option<u64>,
}

impl Probe {
    #[must_use]
    pub fn wrap(&self) -> Option<f64> {
        self.wrap.map(f64::from_bits)
    }
}

/// The sizes of texts as the core lays them out, from probe text nodes
/// (`main.rs`), and the texts that a builder asked for but are not
/// measured yet. A builder then uses the estimate of [`width`], and the
/// caller measures the misses and builds again.
#[derive(Default)]
pub struct Metrics {
    sizes: RefCell<HashMap<Probe, [f64; 2]>>,
    misses: RefCell<Vec<Probe>>,
    /// A fake core lays no text out: its tests keep the estimates and
    /// ask no measure.
    #[cfg(test)]
    estimates: std::cell::Cell<bool>,
}

impl Metrics {
    /// The size of `text` in `role`, wrapped at `wrap`: measured, or
    /// estimated and noted as a miss.
    fn size(&self, role: Role, text: &str, wrap: Option<f64>) -> [f64; 2] {
        let probe = Probe {
            role,
            text: text.into(),
            wrap: wrap.map(f64::to_bits),
        };
        if let Some(size) = self.sizes.borrow().get(&probe) {
            return *size;
        }
        let (_, line) = role.size();
        let w = width(role, text);
        let size = wrap.map_or([w, line], |wrap| {
            [w.min(wrap), lines(role, text, wrap) * line]
        });
        #[cfg(test)]
        if self.estimates.get() {
            return size;
        }
        let mut misses = self.misses.borrow_mut();
        if !misses.contains(&probe) {
            misses.push(probe);
        }
        size
    }

    /// Keeps the estimates: no text is measured (tests on a fake core).
    #[cfg(test)]
    pub(crate) fn keep_estimates(&self) {
        self.estimates.set(true);
    }

    /// The texts to measure, once.
    #[must_use]
    pub fn take_misses(&self) -> Vec<Probe> {
        std::mem::take(&mut self.misses.borrow_mut())
    }

    /// A measured size.
    pub fn insert(&self, probe: Probe, size: [f64; 2]) {
        let mut sizes = self.sizes.borrow_mut();
        // ponytail: forget all at 4096 texts; an LRU if that churns.
        if sizes.len() >= 4096 {
            sizes.clear();
        }
        sizes.insert(probe, size);
    }
}

/// The ways to show `text` on one line, longest first: whole, then cut
/// at each character with an ellipsis.
fn cuts(text: &str) -> Vec<String> {
    let text: String = text.chars().take(LINE_CHARS).collect();
    let ends: Vec<usize> = text.char_indices().map(|(i, _)| i).skip(1).collect();
    let mut out = vec![text.clone()];
    out.extend((ends.into_iter().rev()).map(|i| format!("{}{ELLIPSIS}", text[..i].trim_end())));
    out
}

/// The ways to show the end of `text`, longest first: whole, then from
/// each later word after an ellipsis, then from each later character for
/// a word longer than the panel. A text of more than [`END_CHARS`]
/// characters starts cut, at that many before its end.
fn ends(text: &str) -> Vec<String> {
    let skip = text.chars().count().saturating_sub(END_CHARS);
    let (cut, _) = text.char_indices().nth(skip).unwrap_or_default();
    let text = &text[cut..];
    let starts: Vec<usize> = text.char_indices().map(|(i, _)| i).skip(1).collect();
    let after_space = |i: &usize| text[..*i].ends_with(' ');
    let words = starts.iter().filter(|i| after_space(i));
    let chars = starts.iter().filter(|i| !after_space(i));
    let first = if cut > 0 {
        format!("{ELLIPSIS}{}", text.trim_start())
    } else {
        text.to_owned()
    };
    let mut out = vec![first];
    out.extend(
        words
            .chain(chars)
            .map(|&i| format!("{ELLIPSIS}{}", text[i..].trim_start())),
    );
    out
}

/// The estimated width of a text, the fallback until the core measured
/// it. Labels are capitals with their spacing.
#[allow(clippy::cast_precision_loss, reason = "short texts")]
fn width(role: Role, text: &str) -> f64 {
    let k = match role {
        Role::Label => 0.72,
        Role::Face(face, _) if face.face().mono => 0.602,
        Role::Face(face, _) if face.face().caps => 0.72,
        Role::Face(..) => 0.55,
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

/// What the builders need of the settings.
pub struct Look<'a> {
    pub style: &'a Style,
    pub colors: &'a Colors,
    /// After the watchdog: no shader nodes.
    pub plain: bool,
    /// The path of `glass.frag`.
    pub shader: &'a str,
    /// The path of `shadow.frag`.
    pub shadow: &'a str,
    /// The tokens of the glass of the theme.
    pub glass: chora_kit::glass::Material,
    /// The sizes of the texts.
    pub metrics: &'a Metrics,
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

    /// The size of a text on one line, or wrapped at `wrap`.
    #[must_use]
    pub fn size(&self, role: Role, text: &str, wrap: Option<f64>) -> [f64; 2] {
        self.metrics.size(role, text, wrap)
    }

    /// `text` on one line within `max`: whole, or cut with an ellipsis
    /// (rule 11). Asks for the sizes of all cuts at once.
    #[must_use]
    pub fn one_line(&self, role: Role, text: &str, max: f64) -> String {
        let cuts = cuts(text);
        let fits: Vec<bool> = (cuts.iter())
            .map(|t| self.size(role, t, None)[0] <= max)
            .collect();
        (cuts.iter().zip(fits))
            .find(|(_, fits)| *fits)
            .map_or_else(|| ELLIPSIS.to_string(), |(t, _)| t.clone())
    }

    /// The end of `text` that fits `lines` lines wrapped at `wrap`: the
    /// earlier lines scroll away behind an ellipsis.
    #[must_use]
    pub fn last_lines(&self, role: Role, text: &str, wrap: f64, lines: f64) -> String {
        let ends = ends(text);
        let max = lines * role.size().1 + 0.5;
        let fits: Vec<bool> = (ends.iter())
            .map(|t| self.size(role, t, Some(wrap))[1] <= max)
            .collect();
        (ends.iter().zip(fits))
            .find(|(_, fits)| *fits)
            .map_or_else(|| ELLIPSIS.to_string(), |(t, _)| t.clone())
    }

    /// The props of a text node of a probe; `main.rs` measures with them.
    #[must_use]
    pub fn probe_props(&self, probe: &Probe) -> PropsUpdate {
        PropsUpdate {
            max_width: probe.wrap(),
            ..self.text_props(probe.role, &probe.text, self.colors.text)
        }
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

    pub fn text_props(&self, role: Role, text: &str, color: [f64; 3]) -> PropsUpdate {
        if let Role::Face(face, weight) = role {
            let face = face.face();
            let shown = if face.caps {
                text.to_uppercase()
            } else {
                text.to_owned()
            };
            return PropsUpdate {
                text: Some(if shown.is_empty() { " ".into() } else { shown }),
                family: Some(if face.mono {
                    self.style.mono.clone()
                } else {
                    self.style.font.clone()
                }),
                font_size: Some(face.size),
                line_height: Some(face.line / face.size),
                weight: Some(weight),
                letter_spacing: (face.spacing != 0.0).then_some(face.spacing),
                color: rgba(color, 1.0),
                ..PropsUpdate::default()
            };
        }
        let (size, line) = role.size();
        PropsUpdate {
            text: Some(if text.is_empty() { " " } else { text }.into()),
            family: Some(self.style.font.clone()),
            font_size: Some(size),
            line_height: Some(line / size),
            weight: Some(500),
            letter_spacing: Some(0.16),
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

    /// The groups of the elements over the space, hidden, in the stack of
    /// rule 9 from the back: the pane label and the window list.
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
            group(LIST, None),
            self.glass(LIST_GLASS, LIST, None),
        ];
        commands.extend(self.glow(LIST_GLOW, LIST));
        commands.push(self.text(LIST_LABEL, LIST, Role::Label, " ", self.colors.muted));
        commands
    }

    /// The pane label of `name` (construct HUD spec, section 5): a glass
    /// capsule 32 high with the title in `small` 500, at most 320 wide,
    /// made anew in the group of the label, which moves with each frame.
    /// `drawn` takes the old one away first; `fresh` fades it in. Answers
    /// the width of the capsule.
    #[must_use]
    pub fn label(&self, name: &str, fresh: bool, drawn: bool) -> (Vec<Command>, f64) {
        use chora_kit::glass::{LABEL_CAPSULE, Type};
        let role = Role::Face(Type::Small, 500);
        let pad = 2.0 * self.style.unit;
        let text = self.one_line(role, name, LABEL_MAX - pad * 2.0);
        let w = (self.size(role, &text, None)[0] + pad * 2.0).ceil();
        let mut commands = Vec::new();
        if drawn {
            commands.push(Command::Remove {
                id: LABEL_BODY.into(),
                to: None,
            });
        }
        commands.extend(crate::glass::surface(
            self,
            &self.glass,
            &crate::glass::Surface {
                id: LABEL_BODY,
                parent: LABEL,
                rect: [0.0, 0.0, w, LABEL_CAPSULE],
                radius: LABEL_CAPSULE / 2.0,
                strong: false,
                name: "pane-label",
                input: InputMode::Pass,
            },
        ));
        commands.push(crate::glass::text(
            self,
            LABEL_TEXT_BODY,
            LABEL_BODY,
            (role, &text),
            [pad, (LABEL_CAPSULE - 20.0) / 2.0],
            self.glass.text,
        ));
        commands.push(update(
            LABEL_BODY,
            PropsUpdate {
                opacity: Some(if fresh { 0.0 } else { 1.0 }),
                ..PropsUpdate::default()
            },
        ));
        commands.push(fade(LABEL_BODY, 1.0, FADE_IN));
        (commands, w)
    }

    /// The label of a pane that is not the focus (the brief: every pane
    /// has a label): the capsule of [`Self::label`] at 60 % in a group of
    /// its own, `construct-label-n-{k}`, which the walk moves with each
    /// frame. Answers the commands and the width.
    #[must_use]
    pub fn neighbour_label(&self, k: usize, name: &str) -> (Vec<Command>, f64) {
        use chora_kit::glass::{LABEL_CAPSULE, Type};
        let role = Role::Face(Type::Small, 500);
        let pad = 2.0 * self.style.unit;
        let text = self.one_line(role, name, LABEL_MAX - pad * 2.0);
        let w = (self.size(role, &text, None)[0] + pad * 2.0).ceil();
        let id = neighbour_id(k);
        let mut commands = crate::glass::surface(
            self,
            &self.glass,
            &crate::glass::Surface {
                id: &id,
                parent: crate::NAME,
                rect: [0.0, 0.0, w, LABEL_CAPSULE],
                radius: LABEL_CAPSULE / 2.0,
                strong: false,
                name: "pane-label",
                input: InputMode::Pass,
            },
        );
        commands.push(crate::glass::text(
            self,
            &format!("{id}-text"),
            &id,
            (role, &text),
            [pad, (LABEL_CAPSULE - 20.0) / 2.0],
            self.glass.text,
        ));
        commands.push(update(
            &id,
            PropsUpdate {
                opacity: Some(NEIGHBOUR_OPACITY),
                ..PropsUpdate::default()
            },
        ));
        (commands, w)
    }

    /// A chip of the modes of a cube (K4): a glass capsule, which takes
    /// the click, and its words in `small` 500, both hidden until the mode
    /// shows it.
    #[must_use]
    pub fn chip(&self, id: &str, parent: &str, text: &str) -> Vec<Command> {
        let m = &self.glass;
        let glass = create(
            id,
            NodeKind::Rect,
            parent,
            PropsUpdate {
                size: Some([1.0, 1.0]),
                color: Some(m.fill(false)),
                radius: Some(CHIP_HEIGHT / 2.0),
                border: Some(Some(Border {
                    width: 1.0,
                    color: m.edge_out,
                })),
                input: Some(InputMode::Accept),
                visible: Some(false),
                meta: Some(BTreeMap::from([("surface".into(), "chip".into())])),
                ..PropsUpdate::default()
            },
        );
        let mut label = create(
            &format!("{id}-text"),
            NodeKind::Text,
            parent,
            self.text_props(CHIP_ROLE, text, rgb3(m.text)),
        );
        if let Command::Create { props, .. } = &mut label {
            props.input = Some(InputMode::Pass);
            props.visible = Some(false);
        }
        vec![glass, label]
    }

    /// The top and the bottom of a chip that [`Self::place_chip`] places
    /// above `bottom`.
    #[must_use]
    pub fn chip_row(&self, bottom: f64) -> [f64; 2] {
        let unit = self.style.unit;
        [(bottom - unit - CHIP_HEIGHT).round(), bottom - unit]
    }

    /// Places a chip with its right edge a pad left of `right` and its
    /// bottom a unit above `bottom`, and shows or hides it. Answers the
    /// width that it takes with its gap.
    #[must_use]
    pub fn place_chip(
        &self,
        id: &str,
        text: &str,
        [right, bottom]: [f64; 2],
        show: bool,
    ) -> (Vec<Command>, f64) {
        let (pad, unit) = (self.pad(), self.style.unit);
        let w = self.chip_width(text);
        let x = (right - pad - w).round();
        let y = (bottom - unit - CHIP_HEIGHT).round();
        (self.chip_at(id, text, [x, y], show), w + unit)
    }

    /// The width of a chip of `text`.
    #[must_use]
    pub fn chip_width(&self, text: &str) -> f64 {
        (self.size(CHIP_ROLE, text, None)[0] + 2.0 * self.pad()).ceil()
    }

    /// A chip at `[x, y]` on the screen, shown or hidden.
    #[must_use]
    pub fn chip_at(&self, id: &str, text: &str, [x, y]: [f64; 2], show: bool) -> Vec<Command> {
        let m = &self.glass;
        vec![
            update(
                id,
                PropsUpdate {
                    size: Some([self.chip_width(text), CHIP_HEIGHT]),
                    transform: at(x, y),
                    visible: Some(show),
                    color: Some(m.fill(false)),
                    radius: Some(CHIP_HEIGHT / 2.0),
                    border: Some(Some(Border {
                        width: 1.0,
                        color: m.edge_out,
                    })),
                    ..PropsUpdate::default()
                },
            ),
            update(
                &format!("{id}-text"),
                PropsUpdate {
                    transform: at(x + self.pad(), y + (CHIP_HEIGHT - 20.0) / 2.0),
                    visible: Some(show),
                    ..self.text_props(CHIP_ROLE, text, rgb3(m.text))
                },
            ),
        ]
    }

    /// Fades the pane label out.
    #[must_use]
    pub fn label_hide() -> Vec<Command> {
        vec![fade(LABEL_BODY, 0.0, FADE_OUT)]
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

/// The notice of the construct: a short line for a few seconds, also
/// without the voice agent.
#[derive(Default)]
pub struct Notice {
    /// The text that shows, none without a notice.
    pub say: Option<String>,
    /// The start of its fade-out and its end.
    pub fade_at: Option<Instant>,
    pub drop_at: Option<Instant>,
}

impl Notice {
    /// Shows a short notice for a few seconds.
    pub fn notice(&mut self, text: &str, now: Instant) {
        self.say = Some(text.into());
        self.fade_at = Some(now + NOTICE);
        self.drop_at = None;
    }

    /// The next deadline of the notice.
    #[must_use]
    pub fn deadline(&self) -> Option<Instant> {
        [self.fade_at, self.drop_at].into_iter().flatten().min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, DEFAULT};

    fn config() -> Config {
        serde_saphyr::from_str(DEFAULT).unwrap()
    }

    fn look<'a>(config: &'a Config, metrics: &'a Metrics) -> Look<'a> {
        Look {
            style: &config.style,
            colors: &config.style.themes.light,
            plain: false,
            shader: "/glass.frag",
            shadow: "/shadow.frag",
            glass: chora_kit::glass::Material::of(false),
            metrics,
        }
    }

    /// Measures every miss as the core would with a monospace advance
    /// of 10 per character and wraps by character, then builds again.
    fn measured<T>(metrics: &Metrics, mut build: impl FnMut() -> T) -> T {
        for _ in 0..3 {
            let out = build();
            let misses = metrics.take_misses();
            if misses.is_empty() {
                return out;
            }
            for probe in misses {
                #[allow(clippy::cast_precision_loss, reason = "short texts")]
                let w = probe.text.chars().count() as f64 * 10.0;
                let line = probe.role.size().1;
                let size = probe.wrap().map_or([w, line], |wrap| {
                    [w.min(wrap), (w / wrap).ceil().max(1.0) * line]
                });
                metrics.insert(probe, size);
            }
        }
        panic!("the misses do not end");
    }

    fn text_of<'a>(commands: &'a [Command], node: &str) -> Option<&'a str> {
        commands.iter().rev().find_map(|c| match c {
            Command::Create { id, props, .. } | Command::Update { id, props } if id == node => {
                props.text.as_deref()
            }
            _ => None,
        })
    }

    /// A pane label keeps its text inside its panel, by the sizes that
    /// the core measures: it ends with an ellipsis.
    #[test]
    fn every_text_fits_its_panel() {
        let config = config();
        let metrics = Metrics::default();
        let look = look(&config, &metrics);
        let long = "Donaudampfschifffahrtsgesellschaftskapitänsmützenabzeichen";
        let (label, w) = measured(&metrics, || look.label(long, true, false));
        assert!(w <= LABEL_MAX, "{w}");
        assert!(
            text_of(&label, LABEL_TEXT_BODY)
                .unwrap()
                .ends_with(ELLIPSIS)
        );
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

    /// A task of a few thousand characters made one candidate and one
    /// probe per character (review plugins r1, CON-A-04): the ends start
    /// at most three lines of the longest line before the end.
    #[test]
    fn the_ends_of_a_long_text_are_capped() {
        let text = format!("{}the end", "word ".repeat(2000));
        let ends = ends(&text);
        assert!(
            ends.len() <= 3 * LINE_CHARS + 1,
            "{} candidates",
            ends.len()
        );
        assert!(ends.iter().all(|e| e.chars().count() <= 3 * LINE_CHARS + 1));
        assert!(ends.iter().all(|e| e.ends_with('d')));
        assert!(ends[0].starts_with(ELLIPSIS), "a cut text shows the cut");
        assert_eq!(super::ends("short text")[0], "short text");
    }
}
