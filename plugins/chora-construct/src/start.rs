//! The start card of the empty floor (construct look v2, section 2):
//! while no pane exists, one strong glass card stands where the first
//! pane will appear, with the three ways to start something.

#![allow(dead_code, reason = "the stub, until the card is drawn")]

use chora_kit::words::Language;
use chora_scene::Command;

use crate::hud::Look;

/// The group of the card.
pub const START: &str = "construct-start";
/// The prefix of a row of the card; a click starts it.
pub const ROW: &str = "construct-start-row-";
/// The size of the card.
pub const WIDTH: f64 = 448.0;
pub const HEIGHT: f64 = 296.0;

/// What a row starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    Terminal,
    Programs,
    Cube,
}

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
pub fn words(_language: Language) -> Words {
    Words {
        head: "",
        lead: "",
        rows: ["", "", ""],
        foot: "",
        all_keys: "",
    }
}

/// What the card shows: its words, the keys of its rows and the row
/// under the pointer.
pub struct Model<'a> {
    pub words: &'a Words,
    pub keys: [Vec<String>; 3],
    pub hovered: Option<usize>,
}

/// The act of a row.
#[must_use]
pub fn act(_row: usize) -> Option<Act> {
    None
}

/// The row that a node is, or is inside.
#[must_use]
pub fn row_of(_node: &str) -> Option<usize> {
    None
}

/// The card, made anew, centred in an output of `out`.
#[must_use]
pub fn draw(_look: &Look, _model: &Model<'_>, _out: [f64; 2]) -> Vec<Command> {
    Vec::new()
}

/// The fills of the rows when the pointer moves over another row.
#[must_use]
pub fn hover(_look: &Look, _hovered: Option<usize>) -> Vec<Command> {
    Vec::new()
}

/// The commands that fade the card out.
#[must_use]
pub fn hide() -> Vec<Command> {
    Vec::new()
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
