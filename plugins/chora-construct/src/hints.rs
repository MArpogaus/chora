//! The context hints of the construct HUD (spec, sections 5 and 6): one
//! glass capsule at the bottom centre with two to four hints, each the
//! keycaps of its key and a word, apart by thin separators. They show only
//! where they apply and fade: walking for 6 s after entering and while
//! the pointer rests at the bottom, flown into a pane `Alt Esc Step back`
//! for 3 s, and stood back from the cube while it stands. A hint takes a
//! click and does its act (P3); the pointer over it shows the hover fill.
#![allow(clippy::many_single_char_names, reason = "x, y, w of shapes")]

use std::time::{Duration, Instant};

use chora_kit::glass::{CAPSULE, Material, Type, margin};
use chora_scene::{Command, InputMode, NodeKind, PropsUpdate, Transform};

use crate::{
    glass::{Surface, face, keycaps, keycaps_width, rect, surface, text},
    hud::Look,
};

/// The group of the hints.
pub const HINTS: &str = "construct-hints";
/// The prefix of the nodes that take the click of a hint.
pub const HINT: &str = "construct-hint-";
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
pub struct State {
    /// The set on the screen, if any.
    pub shown: Option<(Set, Option<String>)>,
    /// The set of the walk or a pane shows until then.
    pub until: Option<Instant>,
    /// The pointer rests in the bottom of the output.
    pub low: bool,
    /// Whether the camera was in a pane at the last look.
    pub flown: bool,
    /// The construct drew its first frame.
    pub started: bool,
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
            parent: HINTS.into(),
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
            input: Some(InputMode::Pass),
            ..look.text_props(face(Type::Small, 400), &item.word, rgb(look.glass.text))
        },
    }
}

fn rgb(c: [f64; 4]) -> [f64; 3] {
    [c[0], c[1], c[2]]
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
    vec![Command::Remove {
        id: HINTS.into(),
        to: None,
    }]
}
