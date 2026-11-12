//! The context hints of the construct HUD (spec, sections 5 and 6): one
//! glass capsule at the bottom centre with the hints that apply, at most
//! five, each the keycaps of its key and a word, apart by thin
//! separators. They show only where they apply and fade: walking for 6 s after entering and while
//! the pointer rests at the bottom, flown into a pane `Alt Esc Step back`
//! for 3 s, and stood back from the cube while it stands. A hint takes a
//! click and does its act (P3); the pointer over it shows the hover fill.
#![allow(clippy::many_single_char_names, reason = "x, y, w of shapes")]

use std::time::{Duration, Instant};

use chora_kit::glass::{CAPSULE, Material, Type, margin};
use chora_scene::{Command, InputMode, NodeKind, PropsUpdate, Transform};

use crate::{
    glass::{Surface, face, group, keycaps, keycaps_width, rect, surface, text},
    hud::Look,
};

/// The face label of a cube stood back.
pub const FACE_LABEL: &str = "construct-face-label";
/// The group of the hints.
pub const HINTS: &str = "construct-hints";
/// The prefix of the nodes that take the click of a hint.
pub const HINT: &str = "construct-hint-";
/// The group of those nodes: it takes input, and it sits at the origin
/// of the output, because the glass of the hints lets input through, which
/// every node under it would inherit.
pub const HITS: &str = "construct-hints-hit";
/// How long the hints of the walk and of a pane stay.
pub const WALK: Duration = Duration::from_secs(6);
pub const FLOWN: Duration = Duration::from_secs(3);
/// The height at the bottom of the output where a resting pointer shows
/// the hints of the walk.
pub const BOTTOM: f64 = 64.0;

/// Which hints show.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Set {
    Walk,
    Flown,
    Cube,
}

/// What the hints show and until when.
#[derive(Debug, Default)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent states of the hints"
)]
pub struct State {
    /// The set on the screen, if any, the hint under the pointer, and
    /// what the set shows: the front face of a cube, or the panes and
    /// the pane ahead of the walk.
    pub shown: Option<(Set, Option<String>, usize)>,
    /// The set of the walk or a pane shows until then.
    pub until: Option<Instant>,
    /// The pointer rests in the bottom of the output.
    pub low: bool,
    /// Whether the camera was in a pane at the last look.
    pub flown: bool,
    /// The construct drew its first frame.
    pub started: bool,
    /// The camera went into a pane or out of one: the host hears it.
    pub announce: bool,
    /// The hints are drawn anew at the next look, also when their set
    /// stays: the chips of the cube moved under them.
    pub stale: bool,
    /// The face label as last drawn, and the shares of its dots last sent.
    pub label: Option<Label>,
    pub weights: Vec<f64>,
}

/// The face label as drawn: its width, the corner of the hints that its
/// place is relative to, and the place from there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Label {
    pub width: f64,
    pub corner: [f64; 2],
    pub at: [f64; 2],
}

/// The hints of the walk, in the order they show.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Walking {
    Walk,
    FlyIn,
    Windows,
    Terminal,
    Keys,
}

/// The hints of the walk that apply: the keys to walk and to start a
/// terminal always, `Return Fly in` when a pane is ahead, `Tab Windows`
/// from two windows, and the help; at most five.
#[must_use]
pub fn walking(panes: usize, ahead: bool) -> Vec<Walking> {
    let mut hints = vec![Walking::Walk];
    if ahead {
        hints.push(Walking::FlyIn);
    }
    if panes >= 2 {
        hints.push(Walking::Windows);
    }
    hints.extend([Walking::Terminal, Walking::Keys]);
    hints
}

/// The word of the hint `Terminal`.
#[must_use]
pub fn terminal_word(_language: chora_kit::words::Language) -> &'static str {
    "Terminal"
}

/// The word of the middle hint of the cube: `Step in`, or `Try again`
/// while the face ahead failed to start.
#[must_use]
pub fn middle_word(language: chora_kit::words::Language, failed: bool) -> String {
    if failed {
        return crate::faces::retry_word(language).to_owned();
    }
    chora_kit::words::Word::StepIn.text(language).to_owned()
}

/// The updates that fill the dots of the face label with the turn:
/// `weights` says how much each dot has of the accent.
#[must_use]
pub fn dot_updates(m: &Material, weights: &[f64]) -> Vec<Command> {
    let [r, g, b, _] = m.muted;
    let base = [r, g, b, 0.35];
    (weights.iter().enumerate())
        .map(|(k, w)| {
            let color = [0, 1, 2, 3].map(|i| base[i] + (m.accent[i] - base[i]) * w);
            Command::Update {
                id: format!("{FACE_LABEL}-dot-{k}"),
                props: PropsUpdate {
                    color: Some(color),
                    ..PropsUpdate::default()
                },
            }
        })
        .collect()
}

/// How dim the hint `Step in` is while the cube turns.
pub const DIM: f64 = 0.45;

/// The hint that has the hover fill: the one under the pointer, else,
/// while the pointer lights the edge of a face, the hint `step_in` that
/// steps into that face.
#[must_use]
pub fn hovered(pointer: Option<&str>, face_lit: bool, step_in: &str) -> Option<String> {
    (pointer.map(str::to_owned)).or_else(|| face_lit.then(|| step_in.to_owned()))
}

/// The updates that dim a hint to `DIM` while the cube turns: its keycaps
/// and its word.
#[must_use]
pub fn dim(item: &Item) -> Vec<Command> {
    let id = format!("{HINTS}-{}", item.hit.trim_start_matches(HINT));
    let word = if item.chip {
        format!("{}-text", item.hit)
    } else {
        format!("{id}-word")
    };
    let mut nodes = vec![word];
    for k in 0..item.keys.len() {
        let cap = format!("{id}-keys-{k}");
        nodes.extend([format!("{cap}-low"), format!("{cap}-text"), cap]);
    }
    nodes
        .into_iter()
        .map(|id| Command::Update {
            id,
            props: PropsUpdate {
                opacity: Some(DIM),
                ..PropsUpdate::default()
            },
        })
        .collect()
}

/// One hint: the caps of its key, its word, the keys after the word, the
/// node that takes its click, and that node is a chip that exists.
pub struct Item {
    pub keys: Vec<String>,
    pub word: String,
    pub keys_last: bool,
    pub hit: String,
    pub chip: bool,
}

/// The gaps of a hint: its padding, the gap between keys and word, and the
/// room of a separator.
const PAD: f64 = 8.0;
const GAP: f64 = 8.0;
const SEPARATOR: f64 = 9.0;
const ITEM: f64 = 32.0;

fn width_of(look: &Look, item: &Item) -> f64 {
    let word = look.size(face(Type::Small, 400), &item.word, None)[0];
    PAD + keycaps_width(look, &item.keys) + GAP + word.ceil() + PAD
}

/// The width of the capsule of `items`.
#[must_use]
pub fn capsule_width(look: &Look, items: &[Item]) -> f64 {
    let inner: f64 = items.iter().map(|i| width_of(look, i)).sum();
    #[allow(clippy::cast_precision_loss, reason = "a few hints")]
    let gaps = items.len().saturating_sub(1) as f64 * SEPARATOR;
    (2.0 * PAD + inner + gaps).ceil()
}

/// The hints at the bottom centre of an output of `out`, made anew; the
/// hint under the pointer, `hovered`, has the hover fill. A hint that is a
/// chip moves its node over the capsule.
#[must_use]
pub fn draw(look: &Look, items: &[Item], out: [f64; 2], hovered: Option<&str>) -> Vec<Command> {
    let m = &look.glass;
    let w = capsule_width(look, items);
    let x = ((out[0] - w) / 2.0).round();
    let y = out[1] - margin(out[0]) - CAPSULE;
    let mut commands = surface(
        look,
        m,
        &Surface {
            id: HINTS,
            parent: crate::NAME,
            rect: [x, y, w, CAPSULE],
            radius: CAPSULE / 2.0,
            strong: false,
            name: "hints",
            input: InputMode::Pass,
        },
    );
    commands.push(group(HITS, crate::NAME, [0.0, 0.0], InputMode::Accept));
    let mut left = PAD;
    for (k, item) in items.iter().enumerate() {
        let iw = width_of(look, item).ceil();
        let fill = if hovered == Some(item.hit.as_str()) {
            m.hover
        } else {
            [0.0; 4]
        };
        let top = (CAPSULE - ITEM) / 2.0;
        commands.extend(hit(item, [x + left, y + top, iw], fill));
        commands.extend(content(look, m, item, [left, top]));
        left += iw;
        if k + 1 < items.len() {
            commands.push(rect(
                &format!("{HINTS}-sep-{k}"),
                HINTS,
                [left + 4.0, (CAPSULE - 16.0) / 2.0, 1.0, 16.0],
                0.0,
                m.separator,
            ));
            left += SEPARATOR;
        }
    }
    commands
}

/// The keycaps and the word of a hint in the capsule.
fn content(look: &Look, m: &Material, item: &Item, [left, top]: [f64; 2]) -> Vec<Command> {
    let word_w = look.size(face(Type::Small, 400), &item.word, None)[0];
    let caps_w = keycaps_width(look, &item.keys);
    let (caps_x, word_x) = if item.keys_last {
        (left + PAD + word_w.ceil() + GAP, left + PAD)
    } else {
        (left + PAD, left + PAD + caps_w + GAP)
    };
    let id = format!("{HINTS}-{}", item.hit.trim_start_matches(HINT));
    let (mut commands, _) = keycaps(
        look,
        m,
        (&format!("{id}-keys"), HINTS),
        &item.keys,
        [caps_x, top + 4.0],
    );
    if !item.chip {
        commands.push(text(
            look,
            &format!("{id}-word"),
            HINTS,
            (face(Type::Small, 400), &item.word),
            [word_x, top + (ITEM - 20.0) / 2.0],
            m.text,
        ));
    }
    commands
}

/// The node that takes the click of a hint, over it on the screen: a new
/// rect, or the chip of the cube with its word.
fn hit(item: &Item, [x, y, w]: [f64; 3], fill: [f64; 4]) -> Vec<Command> {
    let props = PropsUpdate {
        transform: Some(Transform::translate(x, y, 0.0)),
        size: Some([w, ITEM]),
        radius: Some(ITEM / 2.0),
        color: Some(fill),
        border: Some(None),
        visible: Some(true),
        input: Some(InputMode::Accept),
        ..PropsUpdate::default()
    };
    if !item.chip {
        return vec![Command::Create {
            id: item.hit.clone(),
            kind: NodeKind::Rect,
            parent: HITS.into(),
            props,
        }];
    }
    vec![Command::Update {
        id: item.hit.clone(),
        props,
    }]
}

/// The words of a chip that a hint takes: its text node, at the place of
/// the word of the hint.
#[must_use]
pub fn chip_word(look: &Look, item: &Item, [x, y]: [f64; 2]) -> Command {
    let caps_w = keycaps_width(look, &item.keys);
    let left = if item.keys_last {
        x + PAD
    } else {
        x + PAD + caps_w + GAP
    };
    Command::Update {
        id: format!("{}-text", item.hit),
        props: PropsUpdate {
            transform: Some(Transform::translate(left, y + (ITEM - 20.0) / 2.0, 0.0)),
            visible: Some(true),
            opacity: Some(1.0),
            input: Some(InputMode::Pass),
            ..look.text_props(face(Type::Small, 400), &item.word, rgb(look.glass.text))
        },
    }
}

fn rgb(c: [f64; 4]) -> [f64; 3] {
    [c[0], c[1], c[2]]
}

/// The width of the face label of `faces` dots.
#[must_use]
pub fn face_label_width(look: &Look, (manager, words): (&str, &str), faces: usize) -> f64 {
    let name_w = look.size(face(Type::Small, 500), manager, None)[0].ceil();
    let words_w = look.size(face(Type::Small, 400), words, None)[0].ceil();
    #[allow(clippy::cast_precision_loss, reason = "a few faces")]
    let dots_w = faces as f64 * 6.0 + faces.saturating_sub(1) as f64 * 4.0;
    16.0 + name_w + GAP + words_w + GAP + 4.0 + dots_w + 16.0
}

/// The place of a face label of `width` under the point `[centre, bottom]`
/// of the hints, 8 below it: the left is rounded once, so the middle of
/// the label is within half a pixel of the point.
#[must_use]
pub fn face_label_place(width: f64, [centre, bottom]: [f64; 2]) -> [f64; 2] {
    [(centre - width / 2.0).round(), bottom.round() + 8.0]
}

/// The update that moves the face label to `[x, y]` of the hints.
#[must_use]
pub fn face_label_move([x, y]: [f64; 2]) -> Command {
    Command::Update {
        id: FACE_LABEL.into(),
        props: PropsUpdate {
            transform: Some(Transform::translate(x, y, 0.0)),
            ..PropsUpdate::default()
        },
    }
}

/// The face label of a cube stood back (spec, section 5): a capsule 32
/// high, 8 under the cube and centred on `centre`, with the name of its
/// window manager, the face in words, muted, and a dot per face, the
/// front one in the accent.
#[must_use]
pub fn face_label(
    look: &Look,
    (manager, words): (&str, &str),
    (front, faces): (usize, usize),
    [centre, bottom]: [f64; 2],
    hollow: &[bool],
) -> Vec<Command> {
    let m = &look.glass;
    let (name_role, words_role) = (face(Type::Small, 500), face(Type::Small, 400));
    let name_w = look.size(name_role, manager, None)[0].ceil();
    let words_w = look.size(words_role, words, None)[0].ceil();
    let w = face_label_width(look, (manager, words), faces);
    let h = chora_kit::glass::LABEL_CAPSULE;
    let [x, y] = face_label_place(w, [centre, bottom]);
    let mut commands = surface(
        look,
        m,
        &Surface {
            id: FACE_LABEL,
            parent: HINTS,
            rect: [x, y, w, h],
            radius: h / 2.0,
            strong: false,
            name: "face-label",
            input: InputMode::Pass,
        },
    );
    let top = (h - 20.0) / 2.0;
    commands.push(text(
        look,
        &format!("{FACE_LABEL}-name"),
        FACE_LABEL,
        (name_role, manager),
        [16.0, top],
        m.text,
    ));
    let words_x = 16.0 + name_w + GAP;
    commands.push(text(
        look,
        &format!("{FACE_LABEL}-words"),
        FACE_LABEL,
        (words_role, words),
        [words_x, top],
        m.muted,
    ));
    let dots_x = words_x + words_w + GAP + 4.0;
    commands.extend(dots(m, (front, faces), hollow, [dots_x, (h - 6.0) / 2.0]));
    commands
}

/// The dots of the face label at `[x, y]`: one per face, the front one
/// in the accent; a face that failed to start has a ring and no fill.
fn dots(
    m: &Material,
    (front, faces): (usize, usize),
    hollow: &[bool],
    [x, y]: [f64; 2],
) -> Vec<Command> {
    let [r, g, b, _] = m.muted;
    let mut dots = Vec::new();
    for k in 0..faces {
        let color = if k == front {
            m.accent
        } else {
            [r, g, b, 0.35]
        };
        #[allow(clippy::cast_precision_loss, reason = "a few faces")]
        let left = x + k as f64 * 10.0;
        let mut dot = rect(
            &format!("{FACE_LABEL}-dot-{k}"),
            FACE_LABEL,
            [left, y, 6.0, 6.0],
            3.0,
            color,
        );
        if let (true, Command::Create { props, .. }) = (hollow.get(k) == Some(&true), &mut dot) {
            props.color = Some([0.0; 4]);
            props.border = Some(Some(chora_scene::Border { width: 1.0, color }));
        }
        dots.push(dot);
    }
    dots
}

/// The top-left corner of the capsule of `items` on the screen.
#[must_use]
pub fn origin(look: &Look, items: &[Item], out: [f64; 2]) -> [f64; 2] {
    let w = capsule_width(look, items);
    [
        ((out[0] - w) / 2.0).round(),
        out[1] - margin(out[0]) - CAPSULE,
    ]
}

/// The place of each hint on the screen: its left edge and top.
#[must_use]
pub fn places(look: &Look, items: &[Item], out: [f64; 2]) -> Vec<[f64; 2]> {
    let w = capsule_width(look, items);
    let x = ((out[0] - w) / 2.0).round();
    let y = out[1] - margin(out[0]) - CAPSULE + (CAPSULE - ITEM) / 2.0;
    let mut left = x + PAD;
    items
        .iter()
        .map(|item| {
            let at = [left, y];
            left += width_of(look, item).ceil() + SEPARATOR;
            at
        })
        .collect()
}

/// The commands that take the hints away; the chips of a cube go back to
/// hiding with their mode.
#[must_use]
pub fn remove() -> Vec<Command> {
    [HINTS, HITS]
        .map(|id| Command::Remove {
            id: id.into(),
            to: None,
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, hud::Metrics};

    #[test]
    fn the_walk_shows_the_hints_that_apply_and_at_most_five() {
        use Walking::{FlyIn, Keys, Terminal, Walk, Windows};
        assert_eq!(walking(0, false), [Walk, Terminal, Keys]);
        assert_eq!(walking(1, true), [Walk, FlyIn, Terminal, Keys]);
        assert_eq!(walking(1, false), [Walk, Terminal, Keys]);
        assert_eq!(walking(2, false), [Walk, Windows, Terminal, Keys]);
        assert_eq!(walking(3, true), [Walk, FlyIn, Windows, Terminal, Keys]);
        for panes in 0..8 {
            assert!(walking(panes, true).len() <= 5);
        }
    }

    #[test]
    fn the_word_of_the_terminal_hint_is_terminal_in_both_languages() {
        use chora_kit::words::Language;
        assert_eq!(terminal_word(Language::En), "Terminal");
        assert_eq!(terminal_word(Language::De), "Terminal");
    }

    #[test]
    fn the_face_label_names_the_manager_and_the_face_with_a_dot_each() {
        let config: Config = serde_saphyr::from_str(crate::config::DEFAULT).unwrap();
        let metrics = Metrics::default();
        let look = Look {
            style: &config.style,
            colors: &config.style.themes.dark,
            plain: false,
            shader: "/glass.frag",
            shadow: "/shadow.frag",
            glass: Material::of(true),
            metrics: &metrics,
        };
        let commands = face_label(
            &look,
            ("xp", "face 1 of 4"),
            (0, 4),
            [640.0, 592.0],
            &[false; 4],
        );
        let created = |id: &str| {
            commands.iter().find_map(|c| match c {
                Command::Create { id: i, props, .. } if i == id => Some(props.clone()),
                _ => None,
            })
        };
        let glass = created(&format!("{FACE_LABEL}-glass")).expect("a glass capsule");
        assert_eq!(glass.size.unwrap()[1], 32.0);
        let group = created(FACE_LABEL).unwrap();
        let (x, y) = group.transform.unwrap().translation_2d();
        assert_eq!(y, 600.0, "8 under the cube");
        assert!(
            (x + glass.size.unwrap()[0] / 2.0 - 640.0).abs() <= 0.5,
            "centred"
        );
        assert_eq!(
            created(&format!("{FACE_LABEL}-name"))
                .unwrap()
                .text
                .as_deref(),
            Some("xp")
        );
        let dots: Vec<[f64; 4]> = (0..4)
            .map(|k| {
                created(&format!("{FACE_LABEL}-dot-{k}"))
                    .unwrap()
                    .color
                    .unwrap()
            })
            .collect();
        assert_eq!(dots[0], look.glass.accent);
        assert_ne!(dots[1], look.glass.accent);
    }

    #[test]
    fn a_failed_face_has_a_hollow_dot_and_its_own_words() {
        use chora_kit::words::Language;
        assert_eq!(middle_word(Language::En, false), "Step in");
        assert_eq!(middle_word(Language::En, true), "Try again");
        assert_eq!(middle_word(Language::De, true), "Erneut versuchen");
        let config: Config = serde_saphyr::from_str(crate::config::DEFAULT).unwrap();
        let metrics = Metrics::default();
        let look = Look {
            style: &config.style,
            colors: &config.style.themes.light,
            plain: false,
            shader: "/glass.frag",
            shadow: "/shadow.frag",
            glass: Material::of(false),
            metrics: &metrics,
        };
        let hollow = [false, false, true, false];
        let commands = face_label(
            &look,
            ("Media center", "\u{b7} did not start"),
            (2, 4),
            [640.0, 592.0],
            &hollow,
        );
        let dot = |k: usize| {
            commands.iter().find_map(|c| match c {
                Command::Create { id, props, .. } if *id == format!("{FACE_LABEL}-dot-{k}") => {
                    Some(props.clone())
                }
                _ => None,
            })
        };
        let ring = dot(2).expect("the dot of the failed face");
        assert_eq!(ring.color, Some([0.0; 4]), "no fill");
        assert!(ring.border.flatten().is_some(), "a ring instead");
        assert!(
            dot(0).unwrap().border.is_none(),
            "a face that works is filled"
        );
    }

    #[test]
    fn the_dots_fill_with_the_turn_and_step_in_dims() {
        let m = Material::of(false);
        let updates = dot_updates(&m, &[0.5, 0.5, 0.0, 0.0]);
        let colour = |k: usize| {
            updates.iter().find_map(|c| match c {
                Command::Update { id, props } if *id == format!("{FACE_LABEL}-dot-{k}") => {
                    props.color
                }
                _ => None,
            })
        };
        let [ar, ag, ab, _] = m.accent;
        let [mr, mg, mb, _] = m.muted;
        let half = colour(0).expect("dot 0");
        assert_eq!(colour(0), colour(1), "half and half");
        let want = [
            mr.midpoint(ar),
            mg.midpoint(ag),
            mb.midpoint(ab),
            0.35_f64.midpoint(1.0),
        ];
        assert!(
            half.iter().zip(want).all(|(x, y)| (x - y).abs() < 1e-9),
            "{half:?}"
        );
        let muted = colour(2).expect("dot 2");
        assert!(
            (muted[3] - 0.35).abs() < 1e-9,
            "a dot without a share stays muted"
        );
        let item = Item {
            keys: vec!["Return".into()],
            word: "Step in".into(),
            keys_last: false,
            hit: "construct-mode-step-in".into(),
            chip: true,
        };
        assert!((DIM - 0.45).abs() < 1e-9);
        let dimmed = dim(&item);
        let opacity = |id: &str| {
            dimmed.iter().find_map(|c| match c {
                Command::Update { id: i, props } if i == id => props.opacity,
                _ => None,
            })
        };
        assert_eq!(opacity("construct-mode-step-in-text"), Some(DIM));
        assert_eq!(
            opacity("construct-hints-construct-mode-step-in-keys-0"),
            Some(DIM)
        );
        assert_eq!(
            opacity("construct-hints-construct-mode-step-in-keys-0-text"),
            Some(DIM)
        );
    }

    #[test]
    fn a_face_under_the_pointer_gives_its_hint_the_hover_fill() {
        let step_in = "construct-mode-step-in";
        assert_eq!(hovered(None, true, step_in).as_deref(), Some(step_in));
        assert_eq!(hovered(None, false, step_in), None);
        assert_eq!(
            hovered(Some("construct-mode-turn-left"), true, step_in).as_deref(),
            Some("construct-mode-turn-left"),
            "a hint under the pointer keeps its own fill"
        );
    }

    #[test]
    fn the_face_label_is_centred_within_half_a_pixel_and_rounds_its_left_once() {
        for centre in [400.0, 400.3, 400.5, 400.7, 655.25] {
            let [x, y] = face_label_place(205.0, [centre, 300.4]);
            assert!((x + 205.0 / 2.0 - centre).abs() <= 0.5, "{centre}: {x}");
            assert!((x - x.round()).abs() < 1e-12, "a whole pixel");
            assert!((y - 308.0).abs() < 1e-12, "8 below the edge");
        }
    }
}
