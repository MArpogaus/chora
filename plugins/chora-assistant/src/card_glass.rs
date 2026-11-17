//! The question card of a folder with a glass chat (construct HUD spec,
//! section 5): the title as a label, the prompt in `text`, the whole
//! detail in a well in `data`, the note of a physical question and the
//! answers as key segments, `Alt Y  Yes` and `Alt N  No`, which hold the
//! key line of the event. The parts and their order are the card's.

use chora_draw::{
    text::wrap,
    theme::{Picture, Theme},
};
use chora_kit::{
    ask::Part,
    glass::{INNER_RADIUS, Material, PANEL_RADIUS, Type, WELL_RADIUS, caps},
    words::{Language, Word},
};

use crate::{
    card::{Layout, MAX_TEXT_META, SIDES, Shown, floor_boundary},
    glass::{Line, keycaps, keycaps_width, rect, surface},
};

/// The width of the card, its padding and the height of an answer.
const WIDTH: i32 = 400;
const PAD: f64 = 16.0;
const SEGMENT: i32 = 40;
const SEGMENT_GAP: i32 = 8;

/// The card as it grows from the top: its SVG, the height so far and the
/// text of its parts.
struct Sheet<'a> {
    theme: &'a Theme,
    m: Material,
    inner: f64,
    body: String,
    y: f64,
    texts: Vec<String>,
}

impl Sheet<'_> {
    fn lines(&mut self, text: &str, role: Type, colour: [f64; 4]) {
        let face = role.face();
        let measure = |line: &str| Line::new(line, role, colour).width(self.theme);
        for line in wrap(text, self.inner, measure) {
            self.body += &Line::new(&line, role, colour).at([PAD, self.y], false);
            self.y += face.line;
        }
    }

    fn detail(&mut self, text: &str) {
        let measure = |line: &str| Line::new(line, Type::Data, self.m.text).width(self.theme);
        let lines = wrap(text, self.inner - 24.0, measure);
        #[allow(clippy::cast_precision_loss, reason = "lines of a card")]
        let h = 16.0 + 20.0 * lines.len() as f64;
        self.body += &rect([PAD, self.y, self.inner, h], WELL_RADIUS, self.m.well);
        for (k, line) in lines.iter().enumerate() {
            let colour = if k == 0 { self.m.text } else { self.m.muted };
            #[allow(clippy::cast_precision_loss, reason = "lines of a card")]
            let top = self.y + 8.0 + 20.0 * k as f64;
            self.body += &Line::new(line, Type::Data, colour).at([PAD + 12.0, top], false);
        }
        self.y += h;
    }

    fn part(&mut self, part: Part, text: &str) {
        match part {
            Part::Title => {
                self.body += &Line::new(text, Type::Label, self.m.text).at([PAD, self.y], false);
                self.y += 16.0;
            }
            Part::Prompt => {
                self.y += 8.0;
                self.lines(text, Type::Text, self.m.text);
            }
            Part::Detail => {
                self.y += 8.0;
                self.detail(text);
            }
            Part::NoVoiceYes => {
                self.y += 8.0;
                self.lines(text, Type::Small, self.m.muted);
            }
            Part::KeyLine => {}
        }
        self.texts.push(text.to_owned());
    }
}

/// The card laid out on a screen, in the light or the dark theme.
pub fn layout(
    theme: &Theme,
    shown: &Shown,
    language: Language,
    (screen, dark): ([i32; 2], bool),
) -> Layout {
    let width = WIDTH
        .min(screen[0] - SIDES)
        .min(shown.max_width.unwrap_or(i32::MAX));
    let mut sheet = Sheet {
        theme,
        m: Material::of(dark),
        inner: f64::from(width) - 2.0 * PAD,
        body: String::new(),
        y: PAD,
        texts: Vec::new(),
    };
    let parts = shown.card.parts(language);
    for (part, text) in &parts {
        sheet.part(*part, text);
    }
    let keyed = parts.iter().any(|(part, _)| *part == Part::KeyLine);
    let most = f64::from(screen[1]) * 0.8;
    let too_long = shown.too_long || sheet.y + 16.0 + f64::from(SEGMENT) + PAD > most;
    let mut buttons = None;
    if too_long {
        refused(&mut sheet, &parts[0].1, language);
    } else if shown.buttons || keyed {
        sheet.y += 16.0;
        #[allow(clippy::cast_possible_truncation, reason = "a card fits a screen")]
        let top = sheet.y.round() as i32;
        #[allow(clippy::cast_possible_truncation, reason = "a few pixels")]
        let pad = PAD as i32;
        let half = (width - 2 * pad - SEGMENT_GAP) / 2;
        buttons = Some([[pad, top], [pad + half + SEGMENT_GAP, top]]);
        sheet.y += f64::from(SEGMENT);
        if shown.notice {
            sheet.y += 8.0;
            let injected = Word::Injected.text(language);
            sheet.lines(injected, Type::Small, sheet.m.text);
            sheet.texts.push(injected.to_owned());
        }
    }
    sheet.y += PAD;
    #[allow(clippy::cast_possible_truncation, reason = "a card fits a screen")]
    let height = sheet.y.ceil() as i32;
    let glass = surface(
        [f64::from(width), f64::from(height)],
        PANEL_RADIUS,
        &sheet.m,
        true,
    );
    let mut text = sheet.texts.join("\n");
    text.truncate(floor_boundary(&text, MAX_TEXT_META));
    Layout {
        picture: Picture::new(width, height).svg(glass + &sheet.body),
        buttons,
        text,
        too_long,
    }
}

/// The card of a question that does not fit: the title and the words that
/// it is refused, no answers.
fn refused(sheet: &mut Sheet<'_>, title: &str, language: Language) {
    sheet.body.clear();
    sheet.texts.clear();
    sheet.y = PAD;
    sheet.part(Part::Title, title);
    sheet.y += 8.0;
    let words = Word::TooLong.text(language);
    sheet.lines(words, Type::Text, sheet.m.text);
    sheet.texts.push(words.to_owned());
}

/// The picture of an answer on a card `card_width` wide: a well with the
/// caps of its key and its word, centred, half the inner width.
pub fn segment(
    theme: &mut Theme,
    key: &str,
    word: &str,
    (card_width, dark): (i32, bool),
) -> String {
    let m = Material::of(dark);
    #[allow(clippy::cast_possible_truncation, reason = "a few pixels")]
    let width = (card_width - 2 * PAD as i32 - SEGMENT_GAP) / 2;
    let keys = caps(key);
    let label = Line::new(word, Type::Small, m.text).weight(500);
    let content = keycaps_width(theme, &keys, &m) + 8.0 + label.width(theme);
    let (w, h) = (f64::from(width), f64::from(SEGMENT));
    let x = ((w - content) / 2.0).round();
    let (caps_svg, caps_w) = keycaps(theme, &keys, [x, (h - 24.0) / 2.0], &m);
    let body = rect([0.0, 0.0, w, h], INNER_RADIUS, m.well)
        + &caps_svg
        + &label.at([x + caps_w + 8.0, (h - 20.0) / 2.0], false);
    theme.render(&Picture::new(width, SEGMENT).svg(body))
}

/// The shadow of the card, with the hairline and the glow of a waiting
/// question.
pub fn shadow(theme: &mut Theme, size: [i32; 2], dark: bool) -> String {
    let m = Material::of(dark);
    crate::glass::shadow(theme, size.map(f64::from), PANEL_RADIUS, &m, true)
}
