//! The start card of the empty floor (construct look v2, section 2):
//! while no pane exists, one strong glass card stands where the first
//! pane will appear, ahead and centred, drawn flat, with the three ways
//! to start something. A row under the pointer has the hover fill at
//! once; a click on a row does what its keys do.

use chora_kit::{
    glass::{INNER_RADIUS, PANEL_RADIUS, Type, UNIT},
    words::Language,
};
use chora_scene::{Command, InputMode, PropsUpdate};

use crate::{
    glass::{Surface, face, keycaps, keycaps_width, rect, surface, text},
    hud::Look,
};

/// The group of the card.
pub const START: &str = "construct-start";
/// The prefix of a row of the card; a click starts it.
pub const ROW: &str = "construct-start-row-";
/// The size of the card.
pub const WIDTH: f64 = 448.0;
pub const HEIGHT: f64 = 296.0;
const PAD: f64 = 24.0;
const ROW_HEIGHT: f64 = 48.0;
const ROW_GAP: f64 = 4.0;
const ROWS_TOP: f64 = 88.0;
const FOOTER_TOP: f64 = 248.0;

/// What a row starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    Terminal,
    Programs,
    Cube,
}

const ACTS: [Act; 3] = [Act::Terminal, Act::Programs, Act::Cube];

/// The words of the card.
pub struct Words {
    pub head: &'static str,
    pub lead: &'static str,
    pub rows: [&'static str; 3],
    pub foot: &'static str,
    pub all_keys: &'static str,
}

/// The words in a language.
#[must_use]
pub fn words(language: Language) -> Words {
    match language {
        Language::En => Words {
            head: "Nothing is open yet",
            lead: "Start a program. It opens here, in front of you.",
            rows: ["Terminal", "All programs", "A cube of desktops"],
            foot: "Click a row or press its keys.",
            all_keys: "All keys",
        },
        Language::De => Words {
            head: "Noch nichts geöffnet",
            lead: "Starte ein Programm. Es öffnet sich hier vor dir.",
            rows: ["Terminal", "Alle Programme", "Ein Würfel aus Desktops"],
            foot: "Zeile anklicken oder Tasten drücken.",
            all_keys: "Alle Tasten",
        },
    }
}

/// What the card shows: its words, the keys of its rows, the key of the
/// help and the row under the pointer.
pub struct Model<'a> {
    pub words: &'a Words,
    pub keys: [Vec<String>; 3],
    pub help: Vec<String>,
    pub hovered: Option<usize>,
}

/// What the card shows now and where.
#[derive(Debug, Default)]
pub struct State {
    /// The card is drawn and not fading out.
    pub up: bool,
    /// The output it was drawn for.
    pub out: [f64; 2],
    /// The row under the pointer.
    pub hover: Option<usize>,
}

/// The act of a row.
#[must_use]
pub fn act(row: usize) -> Option<Act> {
    ACTS.get(row).copied()
}

/// The row that a node is, or is inside.
#[must_use]
pub fn row_of(node: &str) -> Option<usize> {
    let row = node.strip_prefix(ROW)?.split('-').next()?.parse().ok()?;
    (row < ACTS.len()).then_some(row)
}

fn fill(look: &Look, hovered: Option<usize>, row: usize) -> [f64; 4] {
    if hovered == Some(row) {
        look.glass.hover
    } else {
        [0.0; 4]
    }
}

/// The card, made anew, centred in an output of `out`.
#[must_use]
pub fn draw(look: &Look, model: &Model<'_>, out: [f64; 2]) -> Vec<Command> {
    let m = &look.glass;
    let place = [
        ((out[0] - WIDTH) / 2.0).round(),
        ((out[1] - HEIGHT) / 2.0).round(),
    ];
    let mut commands = surface(
        look,
        m,
        &Surface {
            id: START,
            parent: crate::NAME,
            rect: [place[0], place[1], WIDTH, HEIGHT],
            radius: PANEL_RADIUS,
            strong: true,
            name: "start-card",
            input: InputMode::Accept,
        },
    );
    commands.push(text(
        look,
        &format!("{START}-head"),
        START,
        (face(Type::Text, 500), model.words.head),
        [PAD, PAD],
        m.text,
    ));
    commands.push(text(
        look,
        &format!("{START}-lead"),
        START,
        (face(Type::Small, 400), model.words.lead),
        [PAD, PAD + 28.0],
        m.muted,
    ));
    for (k, label) in model.words.rows.iter().enumerate() {
        #[allow(clippy::cast_precision_loss, reason = "three rows")]
        let top = ROWS_TOP + k as f64 * (ROW_HEIGHT + ROW_GAP);
        commands.extend(row(look, model, (k, label), top));
    }
    commands.push(rect(
        &format!("{START}-divider"),
        START,
        [PAD, FOOTER_TOP - UNIT, WIDTH - 2.0 * PAD, 1.0],
        0.0,
        m.separator,
    ));
    commands.extend(footer(look, model));
    commands
}

/// One row at `top`: its hit rect, its label and the caps of its keys.
fn row(look: &Look, model: &Model<'_>, (k, label): (usize, &str), top: f64) -> Vec<Command> {
    let m = &look.glass;
    let id = format!("{ROW}{k}");
    let inset = PAD - UNIT;
    let mut hit = rect(
        &id,
        START,
        [inset, top, WIDTH - 2.0 * inset, ROW_HEIGHT],
        INNER_RADIUS,
        fill(look, model.hovered, k),
    );
    if let Command::Create { props, .. } = &mut hit {
        props.input = Some(InputMode::Accept);
    }
    let mut commands = vec![
        hit,
        text(
            look,
            &format!("{id}-label"),
            START,
            (face(Type::Text, 400), label),
            [PAD + UNIT, top + 12.0],
            m.text,
        ),
    ];
    let width = keycaps_width(look, &model.keys[k]);
    let (caps, _) = keycaps(
        look,
        m,
        (&format!("{id}-keys"), START),
        &model.keys[k],
        [WIDTH - PAD - UNIT - width, top + 12.0],
    );
    commands.extend(caps);
    commands
}

/// The footer: the line at the left, the help key and its word at the
/// right.
fn footer(look: &Look, model: &Model<'_>) -> Vec<Command> {
    let m = &look.glass;
    let small = face(Type::Small, 400);
    let word_w = look.size(small, model.words.all_keys, None)[0].ceil();
    let caps_w = keycaps_width(look, &model.help);
    let word_x = (WIDTH - PAD - word_w).round();
    let (mut commands, _) = keycaps(
        look,
        m,
        (&format!("{START}-help"), START),
        &model.help,
        [word_x - UNIT - caps_w, FOOTER_TOP + 12.0],
    );
    commands.push(text(
        look,
        &format!("{START}-foot"),
        START,
        (small, model.words.foot),
        [PAD, FOOTER_TOP + 14.0],
        m.muted,
    ));
    commands.push(text(
        look,
        &format!("{START}-help-word"),
        START,
        (small, model.words.all_keys),
        [word_x, FOOTER_TOP + 14.0],
        m.muted,
    ));
    commands
}

/// The fills of the rows when the pointer moves over another row.
#[must_use]
pub fn hover(look: &Look, hovered: Option<usize>) -> Vec<Command> {
    (0..ACTS.len())
        .map(|k| Command::Update {
            id: format!("{ROW}{k}"),
            props: PropsUpdate {
                color: Some(fill(look, hovered, k)),
                ..PropsUpdate::default()
            },
        })
        .collect()
}

/// The commands that fade the card out.
#[must_use]
pub fn hide() -> Vec<Command> {
    vec![crate::hud::fade(START, 0.0, crate::hud::FADE_OUT)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, hud::Metrics};
    use chora_kit::glass::Material;
    use chora_scene::PropsUpdate;

    fn config() -> Config {
        serde_saphyr::from_str(crate::config::DEFAULT).unwrap()
    }

    fn model(words: &Words, hovered: Option<usize>) -> Model<'_> {
        Model {
            words,
            keys: [
                vec!["Super".into(), "T".into()],
                vec!["Super".into(), "Space".into()],
                vec!["Super".into(), "C".into()],
            ],
            help: vec!["F1".into()],
            hovered,
        }
    }

    fn created(commands: &[Command], id: &str) -> Option<PropsUpdate> {
        commands.iter().find_map(|c| match c {
            Command::Create { id: i, props, .. } if i == id => Some(props.clone()),
            _ => None,
        })
    }

    fn drawn(hovered: Option<usize>, out: [f64; 2]) -> (Vec<Command>, Material) {
        let config = config();
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
        let words = words(Language::En);
        (
            draw(&look, &model(&words, hovered), out),
            Material::of(true),
        )
    }

    #[test]
    fn the_card_is_448_wide_strong_with_radius_24_and_centred_to_the_pixel() {
        for out in [[1280.0, 720.0], [1920.0, 1080.0], [1366.0, 768.0]] {
            let (commands, material) = drawn(None, out);
            let group = created(&commands, START).expect("the group of the card");
            let (left, top) = group.transform.unwrap().translation_2d();
            let glass = created(&commands, &format!("{START}-glass")).expect("its glass");
            let [width, height] = glass.size.unwrap();
            assert_eq!([width, height], [WIDTH, HEIGHT]);
            assert_eq!(glass.radius, Some(24.0));
            assert_eq!(glass.color, Some(material.fill(true)), "the strong tint");
            assert_eq!(left, ((out[0] - width) / 2.0).round(), "centred across");
            assert_eq!(top, ((out[1] - height) / 2.0).round(), "centred down");
        }
    }

    #[test]
    fn three_rows_of_48_with_radius_16_start_a_terminal_the_programs_a_cube() {
        let (commands, _) = drawn(None, [1280.0, 720.0]);
        for k in 0..3 {
            let row = created(&commands, &format!("{ROW}{k}")).expect("a row");
            assert_eq!(row.size.unwrap()[1], 48.0);
            assert_eq!(row.radius, Some(16.0));
            assert_eq!(row.input, Some(chora_scene::InputMode::Accept));
        }
        assert!(created(&commands, &format!("{ROW}3")).is_none());
        let acts: Vec<Option<Act>> = (0..4).map(act).collect();
        assert_eq!(
            acts,
            [
                Some(Act::Terminal),
                Some(Act::Programs),
                Some(Act::Cube),
                None
            ]
        );
    }

    #[test]
    fn the_english_words_are_those_of_the_spec_and_all_are_drawn() {
        let (commands, _) = drawn(None, [1280.0, 720.0]);
        let words = words(Language::En);
        assert_eq!(words.head, "Nothing is open yet");
        assert_eq!(
            words.lead,
            "Start a program. It opens here, in front of you."
        );
        assert_eq!(
            words.rows,
            ["Terminal", "All programs", "A cube of desktops"]
        );
        assert_eq!(words.foot, "Click a row or press its keys.");
        assert_eq!(words.all_keys, "All keys");
        let texts: Vec<String> = commands
            .iter()
            .filter_map(|c| match c {
                Command::Create { props, .. } => props.text.clone(),
                _ => None,
            })
            .collect();
        let lines = [words.head, words.lead, words.foot, words.all_keys, "F1"];
        for line in lines.into_iter().chain(words.rows) {
            assert!(texts.iter().any(|t| t == line), "{line} is drawn");
        }
    }

    #[test]
    fn german_words_differ_from_the_english_ones() {
        let (en, de) = (words(Language::En), words(Language::De));
        assert_ne!(en.head, de.head);
        assert_ne!(en.foot, de.foot);
        assert!(de.rows.iter().all(|r| !r.is_empty()));
    }

    #[test]
    fn the_row_under_the_pointer_has_the_hover_fill_and_none_else() {
        let (commands, material) = drawn(Some(1), [1280.0, 720.0]);
        let fill = |k: usize| created(&commands, &format!("{ROW}{k}")).unwrap().color;
        assert_eq!(fill(1), Some(material.hover));
        assert_eq!(fill(0), Some([0.0; 4]));
        assert_eq!(fill(2), Some([0.0; 4]));
    }

    #[test]
    fn a_node_inside_a_row_is_that_row() {
        assert_eq!(row_of("construct-start-row-2"), Some(2));
        assert_eq!(row_of("construct-start-row-1-label"), Some(1));
        assert_eq!(row_of("construct-start"), None);
        assert_eq!(row_of("construct-list-row-1"), None);
    }
}
