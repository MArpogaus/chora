//! The starting and failed states of a face of the cube (construct look
//! v2, section 4): a face whose window manager is starting shows its
//! plain fill with a flat glass card, the working ring and `Starting
//! Media center`. A manager that is not up within 5 s, or that exits,
//! turns the card into what failed, the reason in a data well and the
//! next step. There is no red: the word and a hollow dot carry the
//! failure. The words live here, as those of the start card do.

use std::time::Duration;

use chora_kit::words::Language;
use chora_scene::Command;

use crate::hud::Look;

/// The group of the card.
#[allow(dead_code, reason = "a stub until the next commit")]
pub const CARD: &str = "construct-face-card";
/// How long a window manager may take to connect.
#[allow(dead_code, reason = "a stub until the next commit")]
pub const LIMIT: Duration = Duration::from_secs(10);

/// Whether the window manager of a face is starting, up, or failed with
/// its reason.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code, reason = "a stub until the next commit")]
pub enum Health {
    Starting,
    Up,
    Failed(String),
}

/// The words of the card of a face.
#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code, reason = "a stub until the next commit")]
pub struct Words {
    pub head: String,
    pub reason: Option<String>,
    pub foot: Option<String>,
}

/// The words of the card of a face of `manager` in a health; `others_up`
/// says whether another face works.
#[must_use]
#[allow(dead_code, reason = "a stub until the next commit")]
pub fn words(_language: Language, manager: &str, _health: &Health, _others_up: bool) -> Words {
    Words {
        head: manager.to_owned(),
        reason: None,
        foot: None,
    }
}

/// What the face label says of a failed face.
#[must_use]
#[allow(dead_code, reason = "a stub until the next commit")]
pub fn label_words(_language: Language) -> &'static str {
    ""
}

/// The word of the middle hint while the face ahead failed.
#[must_use]
#[allow(dead_code, reason = "a stub until the next commit")]
pub fn retry_word(_language: Language) -> &'static str {
    ""
}

/// The reason of a window manager that stopped after `secs` seconds,
/// with its last line of output.
#[must_use]
#[allow(dead_code, reason = "a stub until the next commit")]
pub fn stopped(_language: Language, program: &str, _secs: u64, _last: &str) -> String {
    program.to_owned()
}

/// The reason of a window manager that did not connect within `secs`.
#[must_use]
#[allow(dead_code, reason = "a stub until the next commit")]
pub fn late(_language: Language, program: &str, _secs: u64) -> String {
    program.to_owned()
}

/// Why a window manager that was started `waited` ago failed, if it
/// did: it exited, or it did not connect within the limit.
#[must_use]
#[allow(dead_code, reason = "a stub until the next commit")]
pub fn failure(
    _language: Language,
    _program: &str,
    _waited: Duration,
    _exited: bool,
    _last: &str,
) -> Option<String> {
    None
}

/// The card of a face for `words`, centred on `centre` in an output of
/// `out`, with the working ring while the manager starts.
#[must_use]
#[allow(dead_code, reason = "a stub until the next commit")]
pub fn draw(
    _look: &Look,
    _words: &Words,
    _ring: bool,
    _centre: [f64; 2],
    _out: [f64; 2],
) -> Vec<Command> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, hud::Metrics};
    use chora_kit::glass::Material;

    const EN: Language = Language::En;
    const DE: Language = Language::De;

    #[test]
    fn a_starting_face_says_which_manager_starts() {
        let w = words(EN, "Media center", &Health::Starting, false);
        assert_eq!(w.head, "Starting Media center");
        assert_eq!((w.reason, w.foot), (None, None));
        let de = words(DE, "Media center", &Health::Starting, false);
        assert_eq!(de.head, "Media center wird gestartet");
    }

    #[test]
    fn a_failed_face_says_what_failed_why_and_what_to_do() {
        let failed = Health::Failed("chora-mc stopped after 2 s: it needs foot.".into());
        let w = words(EN, "Media center", &failed, true);
        assert_eq!(w.head, "Media center did not start.");
        assert_eq!(
            w.reason.as_deref(),
            Some("chora-mc stopped after 2 s: it needs foot.")
        );
        assert_eq!(w.foot.as_deref(), Some("The other faces still work."));
        let alone = words(EN, "Media center", &failed, false);
        assert_eq!(
            alone.foot.as_deref(),
            Some("Click or press Return to try again.")
        );
        let de = words(DE, "XP", &failed, true);
        assert_eq!(de.head, "XP wurde nicht gestartet.");
        assert_eq!(
            de.foot.as_deref(),
            Some("Die anderen Flächen funktionieren weiter.")
        );
    }

    #[test]
    fn the_label_and_the_hint_of_a_failed_face_have_words() {
        assert_eq!(label_words(EN), "did not start");
        assert_eq!(label_words(DE), "nicht gestartet");
        assert_eq!(retry_word(EN), "Try again");
        assert_eq!(retry_word(DE), "Erneut versuchen");
    }

    #[test]
    fn a_manager_that_stopped_names_its_program_its_time_and_its_last_line() {
        assert_eq!(
            stopped(EN, "chora-mc", 2, "it needs foot 1.26 or later."),
            "chora-mc stopped after 2 s: it needs foot 1.26 or later."
        );
        assert_eq!(
            stopped(EN, "chora-mc", 2, "  "),
            "chora-mc stopped after 2 s."
        );
        assert_eq!(
            stopped(DE, "chora-mc", 2, "foot fehlt"),
            "chora-mc endete nach 2 s: foot fehlt"
        );
        assert_eq!(
            late(EN, "chora-mc", 5),
            "chora-mc did not connect within 5 s."
        );
    }

    #[test]
    fn a_manager_fails_when_it_exits_or_is_late_and_not_before() {
        let check = |secs: u64, exited: bool| {
            failure(
                EN,
                "chora-mc",
                Duration::from_secs(secs),
                exited,
                "it is gone",
            )
        };
        assert_eq!(check(1, false), None);
        assert_eq!(
            check(2, true).as_deref(),
            Some("chora-mc stopped after 2 s: it is gone")
        );
        assert_eq!(
            check(LIMIT.as_secs(), false),
            Some(late(EN, "chora-mc", LIMIT.as_secs()))
        );
        assert_eq!(LIMIT, Duration::from_secs(5), "the spec gives it 5 s");
    }

    fn look_drawn(words: &Words, ring: bool) -> Vec<Command> {
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
        draw(&look, words, ring, [640.0, 360.0], [1280.0, 720.0])
    }

    fn created(commands: &[Command]) -> Vec<(&str, &chora_scene::PropsUpdate)> {
        commands
            .iter()
            .filter_map(|c| match c {
                Command::Create { id, props, .. } => Some((id.as_str(), props)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_card_is_a_glass_card_centred_on_the_face_with_its_words_in_it() {
        let failed = Health::Failed("chora-mc stopped after 2 s: it needs foot.".into());
        let w = words(EN, "Media center", &failed, true);
        let commands = look_drawn(&w, false);
        let nodes = created(&commands);
        let group = nodes.iter().find(|(id, _)| *id == CARD).expect("the card");
        let glass = nodes
            .iter()
            .find(|(id, _)| *id == format!("{CARD}-glass"))
            .expect("its glass")
            .1;
        let [width, height] = glass.size.unwrap();
        let (x, y) = group.1.transform.as_ref().unwrap().translation_2d();
        assert!(
            (x + width / 2.0 - 640.0).abs() <= 0.5,
            "centred across: {x}"
        );
        assert!((y + height / 2.0 - 360.0).abs() <= 0.5, "centred down: {y}");
        let texts: Vec<&str> = nodes
            .iter()
            .filter_map(|(_, p)| p.text.as_deref())
            .collect();
        for line in [
            w.head.as_str(),
            w.reason.as_deref().unwrap(),
            w.foot.as_deref().unwrap(),
        ] {
            assert!(texts.iter().any(|t| t.contains(line)), "{line} is drawn");
        }
    }

    #[test]
    fn the_working_ring_shows_while_the_manager_starts_and_no_red_anywhere() {
        let starting = words(EN, "XP", &Health::Starting, false);
        let with = look_drawn(&starting, true);
        assert!(created(&with).iter().any(|(id, _)| id.ends_with("-ring")));
        let failed = words(EN, "XP", &Health::Failed("x".into()), true);
        let without = look_drawn(&failed, false);
        assert!(
            !created(&without)
                .iter()
                .any(|(id, _)| id.ends_with("-ring"))
        );
        for commands in [with, without] {
            for (id, props) in created(&commands) {
                let red = |c: [f64; 4]| c[0] > c[1] + 0.2 && c[0] > c[2] + 0.2;
                assert!(!props.color.is_some_and(red), "{id} is red");
            }
        }
    }
}
