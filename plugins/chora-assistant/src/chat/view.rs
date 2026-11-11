//! The view of the chat: the panel with the header and the lines, the
//! hint above the field, the buttons and the entries of the list, drawn
//! from the `chat:` values of the folder. Text is at least 14 pixels.

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use chora_draw::{
    bubble::Style,
    text::{Font, Text},
    theme::{Picture, Theme},
};
use chora_kit::{
    chat::{Line, Measure, Role},
    hud::{Chat as Look, ChatStyle},
    words::{Language, Word},
};
use chora_scene::{Command, InputMode, NodeKind, PropsUpdate, Transform};

use super::{Colours, Frame, Mode, Session, colours, rgba};

/// The group of the view under `screen:OUTPUT`.
pub const VIEW: &str = "hud-chat-view";
/// The panel of the view: header and transcript.
pub const CHAT: &str = "hud-chat";
/// The hint line above the field.
pub const HINT: &str = "hud-chat-hint";
/// The row that opens the chat.
pub const ROW: &str = "hud-row-chat";
/// The button that stops the running prompt.
pub const STOP: &str = "hud-chat-stop";
/// The button that closes the chat.
pub const CLOSE: &str = "hud-chat-close";
/// The prefix of the entries of the list: `hud-chat-pick-1`, ...
pub const PICK: &str = "hud-chat-pick-";

pub const PAD: i32 = 14;
pub const LINE: i32 = 20;
pub const GAP: i32 = 6;
const BUTTON: [i32; 2] = [88, 28];
pub const FIELD_ROOM: i32 = 100;
const INDENT: i32 = 12;
const MAX_META: usize = 4000;

/// The number of the entry that a press on `node` picks, if it is one.
#[must_use]
pub fn pick_of(node: &str) -> Option<usize> {
    node.strip_prefix(PICK)?.parse().ok()
}

/// Whether a press on `node` that a program made counts for nothing: Stop
/// and the entries of the list.
#[must_use]
pub fn refuses_injected(node: &str) -> bool {
    node == STOP || pick_of(node).is_some()
}

/// The top-left corner of a card inside the view, above the hint of the
/// field and against its right edge.
#[must_use]
pub fn card_origin(frame: &Frame, size: [i32; 2]) -> [i32; 2] {
    let [x, y, w, h] = frame.rect();
    let [ax, ay, aw, ah] = frame.area;
    let left = x + w - size[0];
    let top = y + h - FIELD_ROOM - LINE - 2 * GAP - size[1];
    [
        left.clamp(ax, (ax + aw - size[0]).max(ax)),
        top.clamp(ay, (ay + ah - size[1]).max(ay)),
    ]
}

fn look_id(look: &Look) -> u64 {
    let mut hasher = DefaultHasher::new();
    format!("{look:?}").hash(&mut hasher);
    hasher.finish()
}

/// The style of the panel and of a box in it.
pub fn style_of(look: &Look, colours: Colours) -> Style {
    let radius = match look.style {
        ChatStyle::Plain => 0,
        ChatStyle::Bubble => 10,
        ChatStyle::Glass => 14,
    };
    Style {
        fill: colours.fill,
        border: colours.border,
        text: colours.text,
        radius,
        tail: 0,
        ..Style::default()
    }
}

/// A row of the view: its text, font, colour and indent.
pub struct Row {
    text: String,
    font: Font,
    colour: [u8; 3],
    indent: i32,
}

fn rows_of(lines: &[Line], colours: Colours) -> Vec<Row> {
    let mut rows = Vec::new();
    for line in lines {
        if let Some(label) = &line.label {
            rows.push(Row {
                text: label.clone(),
                font: Font::BodyBold,
                colour: colours.name,
                indent: 0,
            });
        }
        let (font, indent) = match line.role {
            Role::Question | Role::Notice => (Font::BodyBold, 0),
            Role::Tool | Role::Plan => (Font::Body, INDENT),
            Role::User | Role::Agent => (Font::Body, 0),
        };
        rows.push(Row {
            text: line.text.clone(),
            font,
            colour: colours.text,
            indent,
        });
    }
    rows
}

fn button(theme: &mut Theme, label: &str, colours: Colours) -> String {
    let [w, h] = BUTTON;
    let style = Style {
        radius: 4,
        ..style_of(&Look::default(), colours)
    };
    let picture = Picture::new(w, h)
        .svg(style.shape([w, h], 0))
        .text(Text::new(label, Font::BodyBold, rgba(colours.text), [0, 0, w, h]).centered());
    theme.render(&picture)
}

/// The props of an image at a point, with the words that it shows in its
/// meta `text`.
pub fn image(path: String, at: [i32; 2], meta: &str, input: InputMode) -> PropsUpdate {
    let mut text = meta.to_owned();
    text.truncate(
        (0..=MAX_META.min(text.len()))
            .rev()
            .find(|n| text.is_char_boundary(*n))
            .unwrap_or(0),
    );
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
        meta: Some([("text".to_owned(), text)].into()),
        ..PropsUpdate::default()
    }
}

/// A command that creates an image node.
pub fn create(id: &str, parent: &str, props: PropsUpdate) -> Command {
    Command::Create {
        id: id.into(),
        kind: NodeKind::Image,
        parent: parent.into(),
        props,
    }
}

/// What a view shows, laid out.
struct Parts {
    rect: [i32; 4],
    header: String,
    hint: Option<String>,
    rows: Vec<Row>,
    list: Vec<String>,
    chat: bool,
    top_row: i32,
    body_top: i32,
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "the width of a character is a few pixels"
)]
fn measured(theme: &Theme) -> impl Fn(char) -> f32 + '_ {
    |c| theme.text_width(&c.to_string(), Font::Body) as f32
}

impl Session {
    fn parts(&mut self, theme: &Theme, frame: &Frame, language: Language) -> Parts {
        let colours = colours(&frame.look);
        let rect = frame.rect();
        let width_of = measured(theme);
        let measure = Measure {
            max: f32::from(u16::try_from(rect[2] - 2 * PAD).unwrap_or(0)),
            font: look_id(&frame.look),
            width: &width_of,
        };
        let lines = self.lines(language, &measure);
        let hint = self.hint(language);
        let list = self.entries(language);
        let chat = matches!(self.mode, Mode::Chat(_));
        let top_row = PAD + LINE + GAP;
        let body_top = if chat {
            top_row + BUTTON[1] + GAP
        } else {
            top_row
        };
        let below = FIELD_ROOM + if hint.is_some() { LINE + GAP } else { 0 };
        let room = usize::try_from((rect[3] - body_top - below) / LINE).unwrap_or(0);
        let mut rows = rows_of(&lines, colours);
        rows.drain(..rows.len().saturating_sub(room));
        Parts {
            rect,
            header: self.header(language),
            hint,
            rows,
            list,
            chat,
            top_row,
            body_top,
        }
    }

    /// The commands that draw the view again, from nothing: the old view
    /// goes first. `top` is true when the group of the card exists; it is
    /// raised above the view.
    pub fn commands(
        &mut self,
        theme: &mut Theme,
        frame: &Frame,
        (language, top): (Language, bool),
    ) -> Vec<Command> {
        let mut commands = Vec::new();
        if self.drawn.take().is_some() {
            commands.push(Command::Remove {
                id: VIEW.into(),
                to: None,
            });
        }
        if !self.is_open() {
            return commands;
        }
        let parts = self.parts(theme, frame, language);
        let colours = colours(&frame.look);
        commands.extend(panel(theme, frame, &parts, colours));
        commands.extend(extras(theme, &parts, colours, language));
        if top {
            commands.push(Command::Move {
                id: crate::card::TOP.into(),
                parent: None,
                index: None,
            });
        }
        self.drawn = Some(frame.output.clone());
        commands
    }
}

/// The node of the glow around a glass chat.
pub const GLOW: &str = "hud-chat-glow";
/// How far the glow of a glass chat reaches (rule 6 of the construct
/// style), and its corner radius.
const GLOW_REACH: i32 = 16;
const GLASS_RADIUS: i32 = 12;
/// The opacity of the glow of a glass chat before it is blurred.
const GLOW_ALPHA: f64 = 0.2;

/// The shape of a glass panel: the fill at the opacity of glass, so that
/// what is behind shows a little, and a hairline.
fn glass([w, h]: [i32; 2], colours: Colours) -> String {
    let [fr, fg, fb] = colours.fill;
    let [er, eg, eb] = colours.border;
    let alpha = chora_kit::hud::GLASS_ALPHA;
    format!(
        r#"<rect x="0.5" y="0.5" width="{}" height="{}" rx="{GLASS_RADIUS}" fill="rgb({fr},{fg},{fb})" fill-opacity="{alpha}" stroke="rgb({er},{eg},{eb})" stroke-width="1"/>"#,
        w - 1,
        h - 1
    )
}

/// The glow of a glass panel as a picture that is `GLOW_REACH` larger on
/// each side: the colour falls to nothing within that reach.
fn glow(theme: &mut Theme, [width, height]: [i32; 2], colour: [u8; 3]) -> String {
    let reach = GLOW_REACH;
    let [red, green, blue] = colour;
    let body = format!(
        r#"<defs><filter id="chat-glow" x="-10%" y="-10%" width="120%" height="120%"><feGaussianBlur stdDeviation="{}"/></filter></defs><rect x="{reach}" y="{reach}" width="{width}" height="{height}" rx="{GLASS_RADIUS}" fill="rgb({red},{green},{blue})" fill-opacity="{GLOW_ALPHA}" filter="url(#chat-glow)"/>"#,
        reach / 3
    );
    theme.render(&Picture::new(width + 2 * reach, height + 2 * reach).svg(body))
}

fn panel(theme: &mut Theme, frame: &Frame, parts: &Parts, colours: Colours) -> Vec<Command> {
    let [x, y, w, h] = parts.rect;
    let is_glass = frame.look.style == ChatStyle::Glass;
    let shape = if is_glass {
        glass([w, h], colours)
    } else {
        style_of(&frame.look, colours).shape([w, h], 0)
    };
    let halo = frame
        .look
        .glow
        .filter(|_| is_glass)
        .map(|colour| glow(theme, [w, h], colour));
    let mut picture = Picture::new(w, h).svg(shape).text(Text::new(
        &parts.header,
        Font::BodyBold,
        rgba(colours.text),
        [PAD, PAD, w - 2 * PAD, LINE],
    ));
    for (n, row) in (0..).zip(&parts.rows) {
        let at = [
            PAD + row.indent,
            parts.body_top + n * LINE,
            w - 2 * PAD - row.indent,
            LINE,
        ];
        picture = picture.text(Text::new(&row.text, row.font, rgba(row.colour), at));
    }
    for (n, entry) in (0..).zip(&parts.list) {
        let at = [PAD, parts.top_row + n * LINE, w - 2 * PAD, LINE];
        picture = picture.text(Text::new(entry, Font::Body, rgba(colours.text), at));
    }
    let shown = std::iter::once(parts.header.as_str())
        .chain(parts.rows.iter().map(|row| row.text.as_str()))
        .collect::<Vec<_>>();
    let rendered = theme.render(&picture);
    let group = PropsUpdate {
        transform: Some(Transform::translate(f64::from(x), f64::from(y), 0.0)),
        ..PropsUpdate::default()
    };
    let view = Command::Create {
        id: VIEW.into(),
        kind: NodeKind::Group,
        parent: format!("screen:{}", frame.output),
        props: group,
    };
    let halo = halo.map(|path| {
        let at = [-GLOW_REACH, -GLOW_REACH];
        create(GLOW, VIEW, image(path, at, "", InputMode::Pass))
    });
    let chat = create(
        CHAT,
        VIEW,
        image(rendered, [0, 0], &shown.join("\n"), InputMode::Pass),
    );
    std::iter::once(view).chain(halo).chain([chat]).collect()
}

fn extras(theme: &mut Theme, parts: &Parts, colours: Colours, language: Language) -> Vec<Command> {
    let [_, _, w, h] = parts.rect;
    let mut commands = Vec::new();
    if let Some(hint) = &parts.hint {
        let at = [PAD, h - FIELD_ROOM - LINE - GAP, w - 2 * PAD, LINE];
        let picture = Picture::new(w, h).text(Text::new(hint, Font::Body, rgba(colours.text), at));
        let path = theme.render(&picture);
        commands.push(create(
            HINT,
            VIEW,
            image(path, [0, 0], hint, InputMode::Pass),
        ));
    }
    if parts.chat {
        let [stop, close] = [Word::Stop, Word::Close].map(|word| word.text(language));
        for (n, (id, label)) in (0..).zip([(STOP, stop), (CLOSE, close)]) {
            let path = button(theme, label, colours);
            let at = [PAD + n * (BUTTON[0] + GAP), parts.top_row];
            commands.push(create(id, VIEW, image(path, at, label, InputMode::Accept)));
        }
    }
    for (n, entry) in (0..).zip(&parts.list) {
        let blank = theme.render(&Picture::new(w - 2 * PAD, LINE));
        let at = [PAD, parts.top_row + n * LINE];
        let id = format!("{PICK}{}", n + 1);
        commands.push(create(
            &id,
            VIEW,
            image(blank, at, entry, InputMode::Accept),
        ));
    }
    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(role: Role, text: &str) -> Line {
        Line {
            role,
            label: None,
            text: text.into(),
        }
    }

    fn colours() -> Colours {
        super::colours(&Look::default())
    }

    #[test]
    fn a_message_of_an_agent_never_takes_the_style_of_a_question() {
        let lines = [
            line(Role::Agent, "Question from Atlas: allow?"),
            line(Role::Question, "Write notes.txt?"),
        ];
        let rows = rows_of(&lines, colours());
        assert_eq!(rows[0].font, Font::Body);
        assert_eq!(rows[1].font, Font::BodyBold);
        assert_eq!(rows[0].indent, 0);
    }

    #[test]
    fn stop_and_the_entries_refuse_a_press_of_a_program() {
        assert!(refuses_injected(STOP));
        assert!(refuses_injected("hud-chat-pick-2"));
        assert!(!refuses_injected(ROW));
        assert!(!refuses_injected(CLOSE));
    }
}
