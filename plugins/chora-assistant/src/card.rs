//! The question card and the muted badge: what the host draws for
//! `voice.confirm` and for a muted mic, on top of every panel of a HUD
//! folder. No folder styles, moves or hides them. They are drawn with the
//! fonts of the host (`Theme::host`), never with those of a folder, and
//! their text is at least 14 pixels.

use std::{path::Path, time::Instant};

use chora_draw::{
    bubble::Style,
    text::{Font, Text, wrap},
    theme::{Picture, Theme},
};
use chora_kit::{
    ask::{Button, Card, Press, Question},
    words::{Language, Word},
};
use chora_scene::{Command, InputMode, NodeKind, PropsUpdate, Transform};
use serde_json::{Value, json};

/// The group above the group of the HUD folder; it holds the card and the
/// badge and is created after it.
pub const TOP: &str = "hud-top";
/// The card.
pub const CARD: &str = "hud-card";
/// The buttons of the card.
pub const YES: &str = "hud-card-yes";
/// The button that refuses.
pub const NO: &str = "hud-card-no";
/// The badge of a muted mic.
pub const BADGE: &str = "hud-badge";

const WIDTH: i32 = 380;
const PAD: i32 = 14;
const LINE: i32 = 20;
const GAP: i32 = 6;
const BUTTON: [i32; 2] = [88, 32];
const BUTTON_GAP: i32 = 8;
const EDGE: i32 = 8;
const MAX_TEXT_META: usize = 4000;

/// The colours of the card and the badge. Dark text on a light fill, far
/// above the 7:1 that the key line and the buttons need.
const FILL: [u8; 3] = [250, 250, 250];
const BORDER: [u8; 3] = [60, 60, 60];
const TEXT: [u8; 3] = [20, 20, 20];
const BUTTON_FILL: [u8; 3] = [232, 232, 232];

fn style() -> Style {
    Style {
        fill: FILL,
        border: BORDER,
        text: TEXT,
        radius: 8,
        tail: 0,
        ..Style::default()
    }
}

/// The theme of the card and the badge: the fonts of the host only.
///
/// # Errors
///
/// The cache cannot be written.
pub fn theme(cache: &Path, family: &str) -> Result<Theme, String> {
    Theme::host(&cache.join("card"), family)
}

/// A card on the screen.
#[derive(Clone, Debug)]
pub struct Shown {
    /// The model: the question, its place and the guard of a click.
    pub card: Card,
    /// The id as `voice.confirm` sent it, for `confirm.answer`.
    pub id: String,
    /// The event offers the buttons.
    pub buttons: bool,
    /// The output that shows it.
    pub output: String,
    /// The top-left corner on that output, fixed when the card appears.
    pub origin: [i32; 2],
    /// The card says that an injected press does not count.
    pub notice: bool,
}

/// The question of a `voice.confirm` event, with its id as sent. The id of
/// the agent is a text such as `"3"`; a number is taken too. `None` for the
/// end of a question or an event without a usable id or prompt.
#[must_use]
pub fn question(data: &Value, agent: Option<&str>) -> Option<(Question, String)> {
    let id = match &data["id"] {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        _ => return None,
    };
    let mut numbered = data.clone();
    numbered["id"] = json!(id.parse::<u64>().ok()?);
    Question::from_event(&numbered, agent).map(|question| (question, id))
}

/// The lines and the buttons of a card, with the size of the card.
struct Layout {
    picture: Picture,
    buttons: Option<[[i32; 2]; 2]>,
    text: String,
}

/// The lines of the parts of a card, wrapped to a width.
fn content(theme: &Theme, card: &Card, language: Language, width: f64) -> Vec<(Font, String)> {
    let mut rows = Vec::new();
    for (part, text) in card.parts(language) {
        let font = match part {
            chora_kit::ask::Part::Title => Font::BodyBold,
            _ => Font::Body,
        };
        let lines = wrap(&text, width, |line| theme.text_width(line, font));
        rows.extend(lines.into_iter().map(|line| (font, line)));
    }
    rows
}

fn layout(theme: &Theme, shown: &Shown, language: Language, screen: [i32; 2]) -> Layout {
    let width = WIDTH.min(screen[0] - 2 * EDGE);
    let inner = f64::from(width - 2 * PAD);
    let measure = |font| move |s: &str| theme.text_width(s, font);
    let parts = shown.card.parts(language);
    let mut rows = content(theme, &shown.card, language, inner);
    let injected = Word::Injected.text(language);
    let notice = wrap(injected, inner, measure(Font::BodyBold));
    let notice_lines = i32::try_from(notice.len()).unwrap_or(0);
    let below = if shown.buttons {
        2 * GAP + BUTTON[1] + notice_lines * LINE
    } else {
        0
    };
    let most = usize::try_from((screen[1] * 8 / 10 - 2 * PAD - below) / LINE)
        .unwrap_or(1)
        .max(1);
    let all = rows.len();
    if all > most {
        rows.truncate(most - 1);
        rows.push((Font::Body, format!("… +{}", all - most + 1)));
    }
    let lines = i32::try_from(rows.len()).unwrap_or(0);
    let height = 2 * PAD + lines * LINE + below;
    let ink = [TEXT[0], TEXT[1], TEXT[2], 255];
    let mut picture = Picture::new(width, height).svg(style().shape([width, height], 0));
    let line_at = |n: i32| [PAD, PAD + n * LINE, width - 2 * PAD, LINE];
    for (n, (font, line)) in (0..).zip(&rows) {
        picture = picture.text(Text::new(line, *font, ink, line_at(n)));
    }
    let top = PAD + lines * LINE + GAP;
    let buttons = shown
        .buttons
        .then_some([[PAD, top], [PAD + BUTTON[0] + BUTTON_GAP, top]]);
    if shown.buttons && shown.notice {
        for (n, line) in (0..).zip(&notice) {
            let at = line_at(lines + n);
            let rect = [at[0], at[1] + GAP + BUTTON[1] + GAP, at[2], at[3]];
            picture = picture.text(Text::new(line, Font::BodyBold, ink, rect));
        }
    }
    let mut text: String = parts
        .iter()
        .map(|(_, text)| text.as_str())
        .chain((shown.buttons && shown.notice).then_some(injected))
        .collect::<Vec<_>>()
        .join("\n");
    text.truncate(floor_boundary(&text, MAX_TEXT_META));
    Layout {
        picture,
        buttons,
        text,
    }
}

fn floor_boundary(text: &str, at: usize) -> usize {
    (0..=at.min(text.len()))
        .rev()
        .find(|n| text.is_char_boundary(*n))
        .unwrap_or(0)
}

/// The top-left corner of a new card: above the figure with its right edge
/// at the figure's, below it where there is no room above, and inside the
/// output.
#[must_use]
pub fn place(figure: [i32; 4], size: [i32; 2], screen: [i32; 2]) -> [i32; 2] {
    let [fx, fy, fw, fh] = figure;
    let above = fy - EDGE - size[1];
    let y = if above >= EDGE { above } else { fy + fh + EDGE };
    let x = fx + fw - size[0];
    [
        x.clamp(EDGE, (screen[0] - size[0] - EDGE).max(EDGE)),
        y.clamp(EDGE, (screen[1] - size[1] - EDGE).max(EDGE)),
    ]
}

/// The size that a card of `shown` has on a screen.
#[must_use]
pub fn size(theme: &Theme, shown: &Shown, language: Language, screen: [i32; 2]) -> [i32; 2] {
    layout(theme, shown, language, screen).picture.size
}

fn image(path: String, at: [i32; 2], text: &str, input: InputMode) -> PropsUpdate {
    PropsUpdate {
        image: Some(path),
        transform: Some(Transform::translate(
            f64::from(at[0]),
            f64::from(at[1]),
            0.0,
        )),
        opacity: Some(1.0),
        visible: Some(true),
        input: Some(input),
        meta: Some([("text".to_owned(), text.to_owned())].into()),
        ..PropsUpdate::default()
    }
}

fn button(theme: &mut Theme, label: &str) -> String {
    let flat = Style {
        fill: BUTTON_FILL,
        radius: 4,
        ..style()
    };
    let [w, h] = BUTTON;
    let picture = Picture::new(w, h).svg(flat.shape([w, h], 0)).text(
        Text::new(
            label,
            Font::BodyBold,
            [TEXT[0], TEXT[1], TEXT[2], 255],
            [0, 0, w, h],
        )
        .centered(),
    );
    theme.render(&picture)
}

/// The commands that draw a card, or draw it again when `fresh` is false.
/// The card is the last node of its group, so it covers what the folder
/// drew; the group `TOP` is made by the caller before the first one.
pub fn show(
    theme: &mut Theme,
    shown: &Shown,
    language: Language,
    screen: [i32; 2],
    fresh: bool,
) -> Vec<Command> {
    let Layout {
        picture,
        buttons,
        text,
    } = layout(theme, shown, language, screen);
    let [x, y] = shown.origin;
    let card = image(theme.render(&picture), shown.origin, &text, InputMode::Pass);
    let mut commands = vec![if fresh {
        Command::Create {
            id: CARD.into(),
            kind: NodeKind::Image,
            parent: TOP.into(),
            props: card,
        }
    } else {
        Command::Update {
            id: CARD.into(),
            props: card,
        }
    }];
    if let (true, Some(at)) = (fresh, buttons) {
        let [yes, no] = Card::buttons(language);
        for ((id, label), [bx, by]) in [(YES, yes), (NO, no)].into_iter().zip(at) {
            let props = image(
                button(theme, label),
                [x + bx, y + by],
                label,
                InputMode::Accept,
            );
            commands.push(Command::Create {
                id: id.into(),
                kind: NodeKind::Image,
                parent: TOP.into(),
                props,
            });
        }
    }
    commands
}

/// The commands that take the card and its buttons away.
#[must_use]
pub fn hide() -> Vec<Command> {
    [CARD, YES, NO]
        .map(|id| Command::Remove {
            id: id.into(),
            to: None,
        })
        .into()
}

/// What a press on a node of the card does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The parameters of `confirm.answer`.
    Answer(Value),
    /// The card says that an injected press does not count, and stays.
    Notice,
    /// Nothing: a click too soon after the card appeared.
    Ignored,
}

/// The outcome of a press on `node`; `None` when it is no button of the
/// card.
#[must_use]
pub fn press(shown: &Shown, node: &str, injected: bool, now: Instant) -> Option<Outcome> {
    let button = match node {
        YES => Button::Yes,
        NO => Button::No,
        _ => return None,
    };
    Some(match shown.card.press(button, injected, now) {
        Press::Answer(yes) => Outcome::Answer(json!({"id": shown.id, "yes": yes})),
        Press::Injected => Outcome::Notice,
        Press::TooSoon => Outcome::Ignored,
    })
}

/// The badge of a muted mic: a click on it turns the mic on.
pub fn badge(
    theme: &mut Theme,
    language: Language,
    figure: [i32; 4],
    screen: [i32; 2],
) -> Vec<Command> {
    let text = Word::Muted.text(language);
    let width = theme.text_width(text, Font::Body).ceil();
    #[allow(clippy::cast_possible_truncation)]
    let w = (width as i32 + 2 * PAD).min(screen[0] - 2 * EDGE);
    let h = LINE + 12;
    let picture = Picture::new(w, h)
        .svg(style().shape([w, h], 0))
        .text(Text::new(
            text,
            Font::Body,
            [TEXT[0], TEXT[1], TEXT[2], 255],
            [PAD, 6, w - 2 * PAD, LINE],
        ));
    let [fx, fy, fw, fh] = figure;
    let beside = if fx + fw / 2 > screen[0] / 2 {
        fx - EDGE - w
    } else {
        fx + fw + EDGE
    };
    let at = [
        beside.clamp(EDGE, (screen[0] - w - EDGE).max(EDGE)),
        (fy + (fh - h) / 2).clamp(EDGE, (screen[1] - h - EDGE).max(EDGE)),
    ];
    vec![Command::Create {
        id: BADGE.into(),
        kind: NodeKind::Image,
        parent: TOP.into(),
        props: image(theme.render(&picture), at, text, InputMode::Accept),
    }]
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chora_draw::theme::Layer;
    use chora_kit::hud;

    use super::*;

    fn cache(test: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("chora-assistant-{test}-{}", std::process::id()))
    }

    fn shown(data: &Value, now: Instant) -> Shown {
        let (question, id) = question(data, Some("Atlas")).unwrap();
        Shown {
            card: Card::new(question, now),
            id,
            buttons: data["buttons"] == true,
            output: "HEADLESS-1".into(),
            origin: [900, 500],
            notice: false,
        }
    }

    fn notes() -> Value {
        json!({"id": "51", "prompt": "Atlas wants to write the file notes.txt.",
            "detail": "~/projects/notes/notes.txt (412 bytes)", "physical": true,
            "keys": {"yes": "Alt+y", "no": "Alt+n"}, "buttons": true})
    }

    #[test]
    fn the_card_draws_with_host_fonts_only() {
        let cache = cache("fonts");
        let mut theme = theme(&cache, "sans-serif").unwrap();
        assert!(theme.fonts().is_host());
        assert!(!theme.confined() || theme.fonts().is_host());
        let s = shown(&notes(), Instant::now());
        let laid = layout(&theme, &s, Language::En, [1280, 800]);
        let fonts: Vec<Font> = laid
            .picture
            .layers
            .iter()
            .filter_map(|layer| match layer {
                Layer::Text(text) => Some(text.font),
                _ => None,
            })
            .collect();
        assert_ne!(fonts.len(), 0);
        for font in fonts {
            assert!(matches!(font, Font::Body | Font::BodyBold), "{font:?}");
            assert!(theme.fonts().size(font) >= 14);
        }
        theme.set_scale(1.0);
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn the_card_names_the_file_the_keys_and_the_buttons() {
        let cache = cache("names");
        let mut theme = theme(&cache, "sans-serif").unwrap();
        let s = shown(&notes(), Instant::now());
        let commands = show(&mut theme, &s, Language::En, [1280, 800], true);
        let meta = |id: &str| -> String {
            commands
                .iter()
                .find_map(|c| match c {
                    Command::Create { id: i, props, .. } if i == id => {
                        props.meta.as_ref().and_then(|m| m.get("text")).cloned()
                    }
                    _ => None,
                })
                .unwrap_or_default()
        };
        let text = meta(CARD);
        for want in ["notes.txt", "Atlas", "Alt+Y", "Alt+N"] {
            assert!(text.contains(want), "{want} in {text}");
        }
        assert_eq!((meta(YES).as_str(), meta(NO).as_str()), ("Yes", "No"));
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn a_folder_cannot_hide_the_question() {
        let cache = cache("hide");
        let mut theme = theme(&cache, "sans-serif").unwrap();
        let lines: Vec<String> = (0..32)
            .map(|n| format!("- {{id: p{n}, source: say, anchor: center, text: x}}\n"))
            .collect();
        let panels = lines.concat();
        let hud = hud::parse(&format!("hud: 1\npanels:\n{panels}")).unwrap();
        let folder = crate::config::Values::from_hud(&hud, |_| None);
        let waiting = shown(&notes(), Instant::now());
        let screen = [1280, 800];
        let commands = show(&mut theme, &waiting, Language::En, screen, true);
        let ids: Vec<&str> = commands
            .iter()
            .filter_map(|c| match c {
                Command::Create { id, .. } => Some(id.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(ids, [CARD, YES, NO]);
        for command in &commands {
            let Command::Create { parent, props, .. } = command else {
                panic!("{command:?}")
            };
            assert_eq!(parent, TOP);
            assert_eq!((props.visible, props.opacity), (Some(true), Some(1.0)));
        }
        let [left, top] = waiting.origin;
        let [width, height] = size(&theme, &waiting, Language::En, screen);
        assert!(left >= 0 && top >= 0 && left + width <= screen[0] && top + height <= screen[1]);
        assert_eq!(
            folder.anchor,
            chora_kit::hud::Anchor::BottomRight,
            "the folder has no say"
        );
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn an_injected_press_shows_the_line_and_a_early_one_does_nothing() {
        let now = Instant::now();
        let s = shown(&notes(), now);
        let later = now + Duration::from_secs(1);
        assert_eq!(press(&s, YES, true, later), Some(Outcome::Notice));
        assert_eq!(press(&s, NO, true, later), Some(Outcome::Notice));
        assert_eq!(press(&s, YES, false, now), Some(Outcome::Ignored));
        assert_eq!(
            press(&s, YES, false, later),
            Some(Outcome::Answer(json!({"id": "51", "yes": true})))
        );
        assert_eq!(
            press(&s, NO, false, later),
            Some(Outcome::Answer(json!({"id": "51", "yes": false})))
        );
        assert_eq!(press(&s, CARD, false, later), None);
    }

    #[test]
    fn the_id_goes_back_as_it_came() {
        let (q, id) = question(&json!({"id": "7", "prompt": "p"}), None).unwrap();
        assert_eq!((q.id, id.as_str()), (7, "7"));
        let (_, id) = question(&json!({"id": 8, "prompt": "p"}), None).unwrap();
        assert_eq!(id, "8");
        assert!(question(&json!({"id": "x", "prompt": "p"}), None).is_none());
        assert!(question(&json!({"id": "7", "done": true}), None).is_none());
    }

    #[test]
    fn the_card_sits_above_the_figure_inside_the_output() {
        assert_eq!(
            place([1240, 760, 24, 24], [380, 150], [1280, 800]),
            [884, 602]
        );
        assert_eq!(place([10, 10, 24, 24], [380, 150], [1280, 800]), [8, 42]);
        assert_eq!(
            place([1240, 760, 24, 24], [380, 900], [1280, 800]),
            [884, 8]
        );
    }

    #[test]
    fn the_colours_read_at_7_to_1() {
        assert!(hud::contrast(TEXT, FILL) >= hud::MIN_KEY_CONTRAST);
        assert!(hud::contrast(TEXT, BUTTON_FILL) >= hud::MIN_KEY_CONTRAST);
        assert!(hud::contrast(BORDER, FILL) >= 3.0);
    }

    #[test]
    fn a_long_detail_ends_in_a_count_and_the_text_node_has_all_it_can() {
        let cache = cache("long");
        let theme = theme(&cache, "sans-serif").unwrap();
        let detail = (0..300)
            .map(|n| format!("line {n}"))
            .collect::<Vec<_>>()
            .join("\n");
        let data = json!({"id": "1", "prompt": "p", "detail": detail});
        let s = shown(&data, Instant::now());
        let [_, h] = size(&theme, &s, Language::En, [1280, 800]);
        assert!(h <= 800 * 8 / 10 + 2 * PAD, "{h}");
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn the_badge_names_the_way_back_and_takes_clicks() {
        let cache = cache("badge");
        let mut theme = theme(&cache, "sans-serif").unwrap();
        let commands = badge(&mut theme, Language::En, [1240, 760, 24, 24], [1280, 800]);
        let [Command::Create { id, props, .. }] = commands.as_slice() else {
            panic!("{commands:?}")
        };
        assert_eq!(id, BADGE);
        assert_eq!(props.input, Some(InputMode::Accept));
        let text = props.meta.as_ref().unwrap()["text"].clone();
        assert!(
            text.contains("Microphone off") && text.contains("turn it on"),
            "{text}"
        );
        let _ = std::fs::remove_dir_all(cache);
    }
}
