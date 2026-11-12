//! The starting and failed states of a face of the cube (construct look
//! v2, section 4): a face whose window manager is starting shows its
//! plain fill with a flat glass card, the working ring and `Starting
//! Media center`. A manager that is not up within 5 s, or that exits,
//! turns the card into what failed, the reason in a data well and the
//! next step. There is no red: the word and a hollow dot carry the
//! failure. The words live here, as those of the start card do.

use std::time::Duration;

use chora_kit::{
    glass::{PANEL_RADIUS, Type, WELL_RADIUS},
    words::Language,
};
use chora_scene::{Command, InputMode};

use crate::{
    glass::{Surface, face, rect, surface, text},
    hud::{Look, Role},
};

/// The group of the card.
pub const CARD: &str = "construct-face-card";
/// How long a window manager may take to connect.
pub const LIMIT: Duration = Duration::from_secs(5);
/// The width of the card, its padding and the padding of its well.
const WIDTH: f64 = 400.0;
const PAD: f64 = 24.0;
const WELL_PAD: f64 = 12.0;
/// The most lines of the reason.
const REASON_LINES: f64 = 3.0;

/// Whether the window manager of a face is starting, up, or failed with
/// its reason.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Health {
    Starting,
    Up,
    Failed(String),
}

/// The words of the card of a face.
#[derive(Debug, PartialEq, Eq)]
pub struct Words {
    pub head: String,
    pub reason: Option<String>,
    pub foot: Option<String>,
}

/// The words of the card of a face of `manager` in a health; `others_up`
/// says whether another face works.
#[must_use]
pub fn words(language: Language, manager: &str, health: &Health, others_up: bool) -> Words {
    let Health::Failed(reason) = health else {
        let head = match language {
            Language::En => format!("Starting {manager}"),
            Language::De => format!("{manager} wird gestartet"),
        };
        return Words {
            head,
            reason: None,
            foot: None,
        };
    };
    let (head, foot) = match (language, others_up) {
        (Language::En, true) => (
            format!("{manager} did not start."),
            "The other faces still work.",
        ),
        (Language::En, false) => (
            format!("{manager} did not start."),
            "Click or press Return to try again.",
        ),
        (Language::De, true) => (
            format!("{manager} wurde nicht gestartet."),
            "Die anderen Flächen funktionieren weiter.",
        ),
        (Language::De, false) => (
            format!("{manager} wurde nicht gestartet."),
            "Klicken oder Return drücken, um es erneut zu versuchen.",
        ),
    };
    Words {
        head,
        reason: Some(reason.clone()),
        foot: Some(foot.to_owned()),
    }
}

/// The place of the card of a face, fetched only when `inputs` differ
/// from those of the cached place.
#[allow(dead_code)]
pub fn cached<E>(
    _cache: &mut Option<(String, Option<[f64; 4]>)>,
    _inputs: &str,
    fetch: impl FnOnce() -> Result<Option<[f64; 4]>, E>,
) -> Result<Option<[f64; 4]>, E> {
    fetch()
}

/// What the face label says of a failed face.
#[must_use]
pub fn label_words(language: Language) -> &'static str {
    match language {
        Language::En => "did not start",
        Language::De => "nicht gestartet",
    }
}

/// The word of the middle hint while the face ahead failed.
#[must_use]
pub fn retry_word(language: Language) -> &'static str {
    match language {
        Language::En => "Try again",
        Language::De => "Erneut versuchen",
    }
}

/// The reason of a window manager that stopped after `secs` seconds,
/// with its last line of output.
#[must_use]
pub fn stopped(language: Language, program: &str, secs: u64, last: &str) -> String {
    let last = last.trim();
    match (language, last.is_empty()) {
        (Language::En, true) => format!("{program} stopped after {secs} s."),
        (Language::En, false) => format!("{program} stopped after {secs} s: {last}"),
        (Language::De, true) => format!("{program} endete nach {secs} s."),
        (Language::De, false) => format!("{program} endete nach {secs} s: {last}"),
    }
}

/// The reason of a window manager that did not connect within `secs`.
#[must_use]
pub fn late(language: Language, program: &str, secs: u64) -> String {
    match language {
        Language::En => format!("{program} did not connect within {secs} s."),
        Language::De => format!("{program} hat sich nicht innerhalb von {secs} s gemeldet."),
    }
}

/// Why a window manager that was started `waited` ago failed, if it
/// did: it exited, or it did not connect within the limit.
#[must_use]
pub fn failure(
    language: Language,
    program: &str,
    waited: Duration,
    exited: bool,
    last: &str,
) -> Option<String> {
    if exited {
        let secs = waited.as_secs_f64().round();
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a few seconds"
        )]
        return Some(stopped(language, program, secs as u64, last));
    }
    (waited >= LIMIT).then(|| late(language, program, LIMIT.as_secs()))
}

/// A text node of the card that wraps at `wrap`.
fn wrapped(
    look: &Look,
    (id, role, line): (&str, Role, &str),
    at: [f64; 2],
    wrap: f64,
    color: [f64; 4],
) -> Command {
    let mut node = text(look, id, CARD, (role, line), at, color);
    if let Command::Create { props, .. } = &mut node {
        props.max_width = Some(wrap);
    }
    node
}

/// The ring and the head of a card that starts, centred; the head of a
/// card that failed, at the left. Answers the nodes and the next top.
fn head(look: &Look, words: &Words, ring: bool) -> (Vec<Command>, f64) {
    let m = &look.glass;
    let role = face(Type::Text, 500);
    let line = look.one_line(role, &words.head, WIDTH - 2.0 * PAD);
    let id = format!("{CARD}-head");
    if !ring {
        let node = text(look, &id, CARD, (role, &line), [PAD, PAD], m.text);
        return (vec![node], PAD + 24.0 + 16.0);
    }
    let spot = crate::placeholder::SPOT;
    let middle = [WIDTH / 2.0, PAD + spot / 2.0];
    let mut nodes: Vec<Command> =
        crate::placeholder::ring(look, &format!("{CARD}-ring"), CARD, middle)
            .into_iter()
            .collect();
    let top = PAD + spot + 8.0;
    let x = ((WIDTH - look.size(role, &line, None)[0]) / 2.0).round();
    nodes.push(text(look, &id, CARD, (role, &line), [x, top], m.text));
    (nodes, top + 24.0)
}

/// The data well with the reason at `top`. Answers the nodes and the
/// next top.
fn well(look: &Look, reason: &str, top: f64) -> (Vec<Command>, f64) {
    let m = &look.glass;
    let data = face(Type::Data, 400);
    let wrap = WIDTH - 2.0 * PAD - 2.0 * WELL_PAD;
    let shown = look.last_lines(data, reason, wrap, REASON_LINES);
    let height = look.size(data, &shown, Some(wrap))[1] + 2.0 * WELL_PAD;
    let nodes = vec![
        rect(
            &format!("{CARD}-well"),
            CARD,
            [PAD, top, WIDTH - 2.0 * PAD, height],
            WELL_RADIUS,
            m.well,
        ),
        wrapped(
            look,
            (&format!("{CARD}-reason"), data, &shown),
            [PAD + WELL_PAD, top + WELL_PAD],
            wrap,
            m.text,
        ),
    ];
    (nodes, top + height + 16.0)
}

/// The card of a face for `words`, centred on `centre` in an output of
/// `out`, with the working ring while the manager starts. A card with a
/// reason takes the click that tries again.
#[must_use]
pub fn draw(
    look: &Look,
    words: &Words,
    ring: bool,
    centre: [f64; 2],
    out: [f64; 2],
) -> Vec<Command> {
    let (mut body, mut top) = head(look, words, ring);
    if let Some(reason) = &words.reason {
        let (nodes, next) = well(look, reason, top);
        body.extend(nodes);
        top = next;
    }
    if let Some(foot) = &words.foot {
        let small = face(Type::Small, 400);
        let inner = WIDTH - 2.0 * PAD;
        let lines = look.size(small, foot, Some(inner))[1];
        let id = format!("{CARD}-foot");
        body.push(wrapped(
            look,
            (&id, small, foot),
            [PAD, top],
            inner,
            look.glass.muted,
        ));
        top += lines;
    }
    let height = top + PAD;
    let place =
        |c: f64, size: f64, room: f64| (c - size / 2.0).round().clamp(0.0, (room - size).max(0.0));
    let input = if words.reason.is_some() {
        InputMode::Accept
    } else {
        InputMode::Pass
    };
    let mut commands = surface(
        look,
        &look.glass,
        &Surface {
            id: CARD,
            parent: crate::NAME,
            rect: [
                place(centre[0], WIDTH, out[0]),
                place(centre[1], height, out[1]),
                WIDTH,
                height,
            ],
            radius: PANEL_RADIUS,
            strong: true,
            name: "face-card",
            input,
        },
    );
    commands.extend(body);
    commands
}

/// The commands that fade the card out.
#[must_use]
pub fn hide() -> Vec<Command> {
    vec![crate::hud::fade(CARD, 0.0, crate::hud::FADE_OUT)]
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
            w.foot.as_deref(),
            Some("Click or press Return to try again.")
        );
        assert_eq!(
            w.reason.as_deref(),
            Some("chora-mc stopped after 2 s: it needs foot.")
        );
        let alone = words(EN, "Media center", &failed, false);
        assert_eq!(
            alone.foot.as_deref(),
            Some("Click or press Return to try again.")
        );
        let de = words(DE, "XP", &failed, true);
        assert_eq!(de.head, "XP wurde nicht gestartet.");
        assert_eq!(
            de.foot.as_deref(),
            Some("Klicken oder Return drücken, um es erneut zu versuchen.")
        );
    }

    #[test]
    fn the_place_of_the_card_is_fetched_only_when_its_inputs_change() {
        let mut cache = None;
        let mut fetched = 0;
        let mut ask = |inputs: &str, cache: &mut _| {
            cached::<()>(cache, inputs, || {
                fetched += 1;
                Ok(Some([1.0, 2.0, 3.0, 4.0]))
            })
        };
        assert_eq!(ask("a", &mut cache), Ok(Some([1.0, 2.0, 3.0, 4.0])));
        assert_eq!(ask("a", &mut cache), Ok(Some([1.0, 2.0, 3.0, 4.0])));
        assert_eq!(ask("b", &mut cache), Ok(Some([1.0, 2.0, 3.0, 4.0])));
        assert_eq!(fetched, 2);
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
        assert_eq!(
            LIMIT,
            Duration::from_secs(10),
            "one limit for every manager"
        );
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
