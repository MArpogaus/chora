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
    ask::{Button, Card, Press},
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
/// The shadow, the hairline and the glow of a glass card.
pub const SHADOW: &str = "hud-card-shadow";

/// The widest card, and the room that it leaves at the sides of the output.
const WIDTH: i32 = 560;
pub const SIDES: i32 = 64;
const PAD: i32 = 14;
const LINE: i32 = 20;
const GAP: i32 = 6;
const BUTTON: [i32; 2] = [88, 32];
const BUTTON_GAP: i32 = 8;
const EDGE: i32 = 8;
pub const MAX_TEXT_META: usize = 4000;

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
    /// The detail does not fit: the card has no buttons and the question
    /// is refused.
    pub too_long: bool,
    /// The widest the card may be: the width of the chat view that shows it.
    pub max_width: Option<i32>,
    /// The card is glass, in the dark theme or not: the folder has a glass
    /// chat.
    pub glass: Option<bool>,
}

/// The lines and the buttons of a card, with the size of the card.
pub struct Layout {
    pub picture: Picture,
    pub buttons: Option<[[i32; 2]; 2]>,
    pub text: String,
    pub too_long: bool,
}

/// The width of the card on an output.
fn width(screen: [i32; 2], most: Option<i32>) -> i32 {
    WIDTH
        .min(screen[0] - SIDES)
        .min(most.unwrap_or(i32::MAX))
        .max(2 * PAD + 40)
}

fn layout(theme: &Theme, shown: &Shown, language: Language, screen: [i32; 2]) -> Layout {
    if let Some(dark) = shown.glass {
        return crate::card_glass::layout(theme, shown, language, (screen, dark));
    }
    let width = width(screen, shown.max_width);
    let inner = f64::from(width - 2 * PAD);
    let injected = Word::Injected.text(language);
    let notice = wrap(injected, inner, |line| {
        theme.text_width(line, Font::BodyBold)
    });
    let notice_lines = i32::try_from(notice.len()).unwrap_or(0);
    let below = if shown.buttons && !shown.too_long {
        2 * GAP + BUTTON[1] + notice_lines * LINE
    } else {
        0
    };
    let most = usize::try_from((screen[1] * 8 / 10 - 2 * PAD - below) / LINE)
        .unwrap_or(1)
        .max(2);
    let mut rows = content_of(theme, &shown.card, language, inner, usize::MAX);
    let too_long = shown.too_long || rows.len() > most;
    if too_long {
        rows = refusal_rows(theme, &shown.card, language, inner, most);
    }
    let buttons_shown = shown.buttons && !too_long;
    let lines = i32::try_from(rows.len()).unwrap_or(0);
    let height = 2 * PAD + lines * LINE + below;
    let ink = [TEXT[0], TEXT[1], TEXT[2], 255];
    let mut picture = Picture::new(width, height).svg(style().shape([width, height], 0));
    let line_at = |n: i32| [PAD, PAD + n * LINE, width - 2 * PAD, LINE];
    for (n, (font, line)) in (0..).zip(&rows) {
        picture = picture.text(Text::new(line, *font, ink, line_at(n)));
    }
    let top = PAD + lines * LINE + GAP;
    let buttons = buttons_shown.then_some([[PAD, top], [PAD + BUTTON[0] + BUTTON_GAP, top]]);
    if buttons_shown && shown.notice {
        for (n, line) in (0..).zip(&notice) {
            let at = line_at(lines + n);
            let rect = [at[0], at[1] + GAP + BUTTON[1] + GAP, at[2], at[3]];
            picture = picture.text(Text::new(line, Font::BodyBold, ink, rect));
        }
    }
    let mut text = rows
        .iter()
        .map(|(_, line)| line.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    if buttons_shown && shown.notice {
        text = format!("{text}\n{injected}");
    }
    text.truncate(floor_boundary(&text, MAX_TEXT_META));
    Layout {
        picture,
        buttons,
        text,
        too_long,
    }
}

/// The title and the prompt, and the words that the question is refused.
fn refusal_rows(
    theme: &Theme,
    card: &Card,
    language: Language,
    width: f64,
    most: usize,
) -> Vec<(Font, String)> {
    let mut rows = content_of(theme, card, language, width, 2);
    rows.truncate(most - 1);
    let long = wrap(Word::TooLong.text(language), width, |line| {
        theme.text_width(line, Font::BodyBold)
    });
    rows.extend(long.into_iter().map(|line| (Font::BodyBold, line)));
    rows
}

/// The first `parts` parts of a card, wrapped: the title and the prompt.
fn content_of(
    theme: &Theme,
    card: &Card,
    language: Language,
    width: f64,
    parts: usize,
) -> Vec<(Font, String)> {
    let mut rows = Vec::new();
    for (part, text) in card.parts(language).into_iter().take(parts) {
        let font = if part == chora_kit::ask::Part::Title {
            Font::BodyBold
        } else {
            Font::Body
        };
        let lines = wrap(&text, width, |line| theme.text_width(line, font));
        rows.extend(lines.into_iter().map(|line| (font, line)));
    }
    rows
}

/// Whether the whole question, with its detail, key line and note, fits on
/// a card on the screen.
#[must_use]
pub fn fits(theme: &Theme, shown: &Shown, language: Language, screen: [i32; 2]) -> bool {
    let whole = Shown {
        too_long: false,
        ..shown.clone()
    };
    !layout(theme, &whole, language, screen).too_long
}

pub fn floor_boundary(text: &str, at: usize) -> usize {
    (0..=at.min(text.len()))
        .rev()
        .find(|n| text.is_char_boundary(*n))
        .unwrap_or(0)
}

/// The top-left corner of a new card: above the figure with its right edge
/// at the figure's, below it where there is no room above, and inside the
/// work area of the output.
#[must_use]
pub fn place(figure: [i32; 4], size: [i32; 2], area: [i32; 4]) -> [i32; 2] {
    let [fx, fy, fw, fh] = figure;
    let [ax, ay, aw, ah] = area;
    let above = fy - EDGE - size[1];
    let y = if above >= ay + EDGE {
        above
    } else {
        fy + fh + EDGE
    };
    let x = fx + fw - size[0];
    [
        x.clamp(ax + EDGE, (ax + aw - size[0] - EDGE).max(ax + EDGE)),
        y.clamp(ay + EDGE, (ay + ah - size[1] - EDGE).max(ay + EDGE)),
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

/// The command that creates a node of the card, or updates it.
fn put(id: &str, props: PropsUpdate, fresh: bool) -> Command {
    if fresh {
        Command::Create {
            id: id.into(),
            kind: NodeKind::Image,
            parent: TOP.into(),
            props,
        }
    } else {
        Command::Update {
            id: id.into(),
            props,
        }
    }
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
        ..
    } = layout(theme, shown, language, screen);
    let [x, y] = shown.origin;
    let size = picture.size;
    let card = image(theme.render(&picture), shown.origin, &text, InputMode::Pass);
    let mut commands: Vec<Command> = shown
        .glass
        .map(|dark| {
            let reach = crate::glass::REACH;
            let path = crate::card_glass::shadow(theme, size, dark);
            let props = image(path, [x - reach, y - reach], "", InputMode::Pass);
            put(SHADOW, props, fresh)
        })
        .into_iter()
        .collect();
    commands.push(put(CARD, card, fresh));
    if let (true, Some(at)) = (fresh, buttons) {
        let [yes, no] = Card::buttons(language);
        let keys = &shown.card.question.keys;
        for (((id, label), key), [bx, by]) in [(YES, yes), (NO, no)]
            .into_iter()
            .zip([&keys.yes, &keys.no])
            .zip(at)
        {
            let picture = match shown.glass {
                Some(dark) => crate::card_glass::segment(theme, key, label, (size[0], dark)),
                None => button(theme, label),
            };
            let props = image(picture, [x + bx, y + by], label, InputMode::Accept);
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

/// The commands that take the card of `shown`, its shadow and its
/// buttons away.
#[must_use]
pub fn hide(shown: &Shown) -> Vec<Command> {
    let shadow = shown.glass.map(|_| SHADOW);
    let buttons = (shown.buttons && !shown.too_long).then_some([YES, NO]);
    shadow
        .into_iter()
        .chain([CARD])
        .chain(buttons.into_iter().flatten())
        .map(|id| Command::Remove {
            id: id.into(),
            to: None,
        })
        .collect()
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

/// Whether two rectangles, x, y, width and height, share a pixel.
fn overlap([ax, ay, aw, ah]: [i32; 4], [bx, by, bw, bh]: [i32; 4]) -> bool {
    ax < bx + bw && bx < ax + aw && ay < by + bh && by < ay + ah
}

/// The size of the badge in the work area of an output.
fn badge_size(theme: &Theme, language: Language, area: [i32; 4]) -> [i32; 2] {
    let width = theme
        .text_width(Word::Muted.text(language), Font::Body)
        .ceil();
    #[allow(clippy::cast_possible_truncation)]
    let w = (width as i32 + 2 * PAD).min(area[2] - 2 * EDGE);
    [w, LINE + 12]
}

/// The place of the badge: beside the figure, on the side of the middle of
/// the output; above or below the card where the card is in the way.
fn badge_place(
    [w, h]: [i32; 2],
    figure: [i32; 4],
    area: [i32; 4],
    avoid: Option<[i32; 4]>,
) -> [i32; 2] {
    let [fx, fy, fw, fh] = figure;
    let [ax, ay, aw, ah] = area;
    let beside = if fx + fw / 2 > ax + aw / 2 {
        fx - EDGE - w
    } else {
        fx + fw + EDGE
    };
    let clamp = |x: i32, y: i32| {
        [
            x.clamp(ax + EDGE, (ax + aw - w - EDGE).max(ax + EDGE)),
            y.clamp(ay + EDGE, (ay + ah - h - EDGE).max(ay + EDGE)),
        ]
    };
    let at = clamp(beside, fy + (fh - h) / 2);
    let Some(card) = avoid.filter(|card| overlap([at[0], at[1], w, h], *card)) else {
        return at;
    };
    let above = card[1] - EDGE - h;
    let y = if above >= ay + EDGE {
        above
    } else {
        card[1] + card[3] + EDGE
    };
    clamp(at[0], y)
}

/// The badge of a muted mic: a click on it turns the mic on. It keeps clear
/// of the card, if there is one.
pub fn badge(
    theme: &mut Theme,
    language: Language,
    (figure, area): ([i32; 4], [i32; 4]),
    avoid: Option<[i32; 4]>,
) -> Vec<Command> {
    let text = Word::Muted.text(language);
    let [w, h] = badge_size(theme, language, area);
    let picture = Picture::new(w, h)
        .svg(style().shape([w, h], 0))
        .text(Text::new(
            text,
            Font::Body,
            [TEXT[0], TEXT[1], TEXT[2], 255],
            [PAD, 6, w - 2 * PAD, LINE],
        ));
    let at = badge_place([w, h], figure, area, avoid);
    vec![Command::Create {
        id: BADGE.into(),
        kind: NodeKind::Image,
        parent: TOP.into(),
        props: image(theme.render(&picture), at, text, InputMode::Accept),
    }]
}

/// The command that moves the badge clear of the card.
#[must_use]
pub fn badge_move(
    theme: &Theme,
    language: Language,
    (figure, area): ([i32; 4], [i32; 4]),
    avoid: [i32; 4],
) -> Command {
    let size = badge_size(theme, language, area);
    let [x, y] = badge_place(size, figure, area, Some(avoid));
    Command::Update {
        id: BADGE.into(),
        props: PropsUpdate {
            transform: Some(Transform::translate(f64::from(x), f64::from(y), 0.0)),
            ..PropsUpdate::default()
        },
    }
}

/// The rectangle of the card on the screen: x, y, width and height.
#[must_use]
pub fn rect(theme: &Theme, shown: &Shown, language: Language, screen: [i32; 2]) -> [i32; 4] {
    let [w, h] = size(theme, shown, language, screen);
    [shown.origin[0], shown.origin[1], w, h]
}

/// Whether a rectangle lies inside the output.
#[must_use]
pub fn inside([x, y, w, h]: [i32; 4], screen: [i32; 2]) -> bool {
    x >= 0 && y >= 0 && x + w <= screen[0] && y + h <= screen[1]
}

/// Keeps a card where it is, unless its output is another one or it no
/// longer fits inside the output: then it takes the place of a new card and
/// its buttons take no click for [`chora_kit::ask::GUARD`]. True when it
/// moved.
pub fn settle(
    shown: &mut Shown,
    (output, screen, area): (&str, [i32; 2], [i32; 4]),
    (theme, language, figure): (&Theme, Language, [i32; 4]),
    now: Instant,
) -> bool {
    let other = shown.output != output;
    output.clone_into(&mut shown.output);
    let [w, h] = size(theme, shown, language, screen);
    if !other && inside([shown.origin[0], shown.origin[1], w, h], screen) {
        return false;
    }
    shown.origin = place(figure, [w, h], area);
    shown.card.moved(now);
    true
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chora_draw::theme::Layer;
    use chora_kit::{ask::Question, hud};

    use super::*;

    fn cache(test: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("chora-assistant-{test}-{}", std::process::id()))
    }

    fn shown(data: &Value, now: Instant) -> Shown {
        let question = Question::from_event(data, Some("Atlas")).unwrap();
        Shown {
            id: question.answer_id.clone(),
            card: Card::new(question, now),
            buttons: data["buttons"] == true,
            output: "HEADLESS-1".into(),
            origin: [900, 500],
            notice: false,
            too_long: false,
            max_width: None,
            glass: None,
        }
    }

    fn notes() -> Value {
        json!({"id": "51", "prompt": "Atlas wants to write the file notes.txt.",
            "detail": "~/projects/notes/notes.txt (412 bytes)", "physical": true,
            "keys": {"yes": "Alt+y", "no": "Alt+n"}, "buttons": true})
    }

    #[test]
    fn a_glass_card_is_400_wide_with_the_keys_in_its_answers() {
        let cache = cache("glass");
        let theme = theme(&cache, "sans-serif").unwrap();
        let mut card = shown(&notes(), Instant::now());
        card.glass = Some(false);
        let laid = layout(&theme, &card, Language::En, [1280, 720]);
        assert_eq!(laid.picture.size[0], 400);
        let [yes, no] = laid.buttons.expect("a question with buttons");
        assert_eq!(yes[0], 16);
        assert_eq!(no[0], 16 + 180 + 8);
        assert_eq!(yes[1], no[1]);
        assert_eq!(laid.picture.size[1], yes[1] + 40 + 16);
        assert!(laid.text.contains("Question from Atlas"), "{}", laid.text);
        assert!(laid.text.contains("Alt+Y"), "{}", laid.text);
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn a_card_in_the_chat_is_no_wider_than_the_view() {
        let cache = cache("width");
        let theme = theme(&cache, "sans-serif").unwrap();
        let mut card = shown(&notes(), Instant::now());
        assert_eq!(size(&theme, &card, Language::En, [1280, 720])[0], 560);
        card.max_width = Some(400);
        assert!(size(&theme, &card, Language::En, [1280, 720])[0] <= 400);
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn the_card_draws_with_host_fonts_only() {
        let cache = cache("fonts");
        let theme = theme(&cache, "sans-serif").unwrap();
        assert!(theme.fonts().is_host());
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
    fn the_card_sits_above_the_figure_inside_the_output() {
        assert_eq!(
            place([1240, 760, 24, 24], [380, 150], [0, 0, 1280, 800]),
            [884, 602]
        );
        assert_eq!(
            place([10, 10, 24, 24], [380, 150], [0, 0, 1280, 800]),
            [8, 42]
        );
        assert_eq!(
            place([1240, 760, 24, 24], [380, 900], [0, 0, 1280, 800]),
            [884, 8]
        );
    }

    #[test]
    fn the_colours_read_at_7_to_1() {
        assert!(hud::contrast(TEXT, FILL) >= hud::MIN_KEY_CONTRAST);
        assert!(hud::contrast(TEXT, BUTTON_FILL) >= hud::MIN_KEY_CONTRAST);
        assert!(hud::contrast(BORDER, FILL) >= 3.0);
    }

    fn with_detail(lines: usize) -> Value {
        let detail = (0..lines)
            .map(|n| format!("line {n}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut data = notes();
        data["detail"] = json!(detail);
        data
    }

    fn texts(theme: &Theme, shown: &Shown) -> String {
        layout(theme, shown, Language::En, [1280, 800]).text
    }

    #[test]
    fn a_detail_that_does_not_fit_is_refused_without_buttons() {
        let cache = cache("toolong");
        let mut theme = theme(&cache, "sans-serif").unwrap();
        let hidden = "padding\n".repeat(100) + "; rm -rf ~";
        let mut data = notes();
        data["detail"] = json!(hidden);
        let mut waiting = shown(&data, Instant::now());
        assert!(!fits(&theme, &waiting, Language::En, [1280, 800]));
        waiting.too_long = true;
        waiting.buttons = false;
        let laid = layout(&theme, &waiting, Language::En, [1280, 800]);
        assert!(laid.too_long && laid.buttons.is_none());
        let said = "Too long to show. Refused.";
        assert!(laid.text.contains(said), "{}", laid.text);
        assert!(laid.picture.size[1] <= 800 * 8 / 10 + 2 * PAD);
        let commands = show(&mut theme, &waiting, Language::En, [1280, 800], true);
        assert_eq!(commands.len(), 1, "no buttons");
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn the_key_line_and_the_note_are_never_cut() {
        let cache = cache("keys");
        let theme = theme(&cache, "sans-serif").unwrap();
        let mut refused = 0;
        for lines in [0, 1, 5, 12, 20, 25, 30, 60, 100, 400] {
            let waiting = shown(&with_detail(lines), Instant::now());
            if fits(&theme, &waiting, Language::En, [1280, 800]) {
                let text = texts(&theme, &waiting);
                assert!(text.contains("Press Alt+Y to allow"), "{lines}: {text}");
                assert!(text.contains("spoken"), "{lines}: {text}");
                if lines > 0 {
                    assert!(text.contains(&format!("line {}", lines - 1)));
                }
            } else {
                refused += 1;
            }
        }
        assert!(refused >= 3, "{refused}");
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn a_card_that_no_longer_fits_moves_and_waits_for_the_guard() {
        let cache = cache("settle");
        let theme = theme(&cache, "sans-serif").unwrap();
        let now = Instant::now();
        let mut waiting = shown(&notes(), now);
        let later = now + Duration::from_secs(1);
        let big = [1280, 800];
        let figure = [1240, 760, 24, 24];
        let en = Language::En;
        waiting.origin = place(
            figure,
            size(&theme, &waiting, en, big),
            [0, 0, big[0], big[1]],
        );
        assert!(!settle(
            &mut waiting,
            ("HEADLESS-1", big, [0, 0, big[0], big[1]]),
            (&theme, en, figure),
            later
        ));
        let yes = Outcome::Answer(json!({"id": "51", "yes": true}));
        assert_eq!(press(&waiting, YES, false, later), Some(yes.clone()));
        let small = [800, 600];
        let near = (&theme, en, [760, 560, 24, 24]);
        assert!(settle(
            &mut waiting,
            ("HEADLESS-1", small, [0, 0, 800, 600]),
            near,
            later
        ));
        assert!(inside(rect(&theme, &waiting, en, small), small));
        assert_eq!(press(&waiting, YES, false, later), Some(Outcome::Ignored));
        let after = later + Duration::from_millis(500);
        assert_eq!(press(&waiting, YES, false, after), Some(yes));
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn the_badge_keeps_clear_of_the_card() {
        let figure = [1240, 760, 24, 24];
        let screen = [1280, 800];
        let card = [880, 700, 380, 90];
        let area = [0, 0, 1280, 800];
        let free = badge_place([250, 32], figure, area, None);
        assert!(overlap([free[0], free[1], 250, 32], card));
        let moved = badge_place([250, 32], figure, area, Some(card));
        assert!(!overlap([moved[0], moved[1], 250, 32], card), "{moved:?}");
        assert!(inside([moved[0], moved[1], 250, 32], screen));
    }

    #[test]
    fn the_card_and_the_badge_keep_clear_of_a_bar() {
        let area = [0, 40, 1280, 720];
        let figure = [1240, 80, 24, 24];
        let at = place(figure, [380, 90], area);
        assert!(at[1] >= 40 + EDGE, "{at:?}: below the bar at the top");
        let low = place([1240, 720, 24, 24], [380, 90], area);
        assert!(
            low[1] + 90 <= 760 - EDGE,
            "{low:?}: above the bar at the bottom"
        );
        let badge = badge_place([250, 32], [1240, 720, 24, 24], area, None);
        assert!(badge[1] + 32 <= 760 - EDGE, "{badge:?}");
    }

    #[test]
    fn the_card_is_at_most_560_wide_and_leaves_64_at_the_sides() {
        assert_eq!(width([1920, 1080], None), 560);
        assert_eq!(width([500, 400], None), 436);
    }

    #[test]
    fn the_badge_names_the_way_back_and_takes_clicks() {
        let cache = cache("badge");
        let mut theme = theme(&cache, "sans-serif").unwrap();
        let place = ([1240, 760, 24, 24], [0, 0, 1280, 800]);
        let commands = badge(&mut theme, Language::En, place, None);
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
