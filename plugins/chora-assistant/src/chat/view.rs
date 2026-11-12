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

use super::{
    Colours, Frame, Mode, Session, colours,
    face::{self, Face},
    rgba,
};

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
/// The height of a row of the list of helpers in the margin of a client:
/// a line of its grid.
const MARGIN_ROW: i32 = 28;
/// The room of the number of a row of the list.
const NUMBER_ROOM: i32 = 22;
const INDENT: i32 = 12;
const MAX_META: usize = 4000;

/// How far below the top of a glass chat a card stands: where it stands
/// under the island, which the chat replaces.
const GLASS_CARD_TOP: i32 = 48;

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
    if frame.look.style == ChatStyle::Glass {
        return [x + w - size[0], y + GLASS_CARD_TOP];
    }
    let [ax, ay, aw, ah] = frame.area;
    let left = x + w - card_inset(frame) - size[0];
    let top = y + h - field_room(frame) - LINE - 2 * GAP - size[1];
    [
        left.clamp(ax, (ax + aw - size[0]).max(ax)),
        top.clamp(ay, (ay + ah - size[1]).max(ay)),
    ]
}

/// The gap between a card and the edges of a view that is a balloon,
/// whose outline the card leaves whole.
fn card_inset(frame: &Frame) -> i32 {
    if frame.tail().is_some() { PAD } else { 0 }
}

/// The room at the bottom of the view that the field takes, with its
/// inset in a balloon and a gap above it.
fn field_room(frame: &Frame) -> i32 {
    frame.field + card_inset(frame) + GAP
}

/// The widest card inside the view.
#[must_use]
pub fn card_width(frame: &Frame) -> i32 {
    frame.rect()[2] - 2 * card_inset(frame)
}

fn look_id(look: &Look) -> u64 {
    let mut hasher = DefaultHasher::new();
    format!("{look:?}").hash(&mut hasher);
    hasher.finish()
}

/// The style of the panel and of a box in it.
pub fn style_of(look: &Look, colours: Colours) -> Style {
    let radius = match look.style {
        ChatStyle::Plain => look.radius.map_or(0, |r| i32::try_from(r).unwrap_or(0)),
        ChatStyle::Bubble => super::BALLOON_RADIUS,
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

/// A row of the view: its text, font, colour and indent; `read` is text
/// that a person reads, which a look with a face sets in it.
pub struct Row {
    text: String,
    font: Font,
    colour: [u8; 3],
    indent: i32,
    read: bool,
    starts: bool,
}

/// Keeps the newest `keep` rows, so that the transcript scrolls by whole
/// messages: the first row that stays is a speaker's line. A cut through a
/// message goes on to the next message, and the end of a message that
/// fills the view alone is shown under its speaker's line.
fn keep_newest(rows: &mut Vec<Row>, keep: usize) {
    let cut = rows.len().saturating_sub(keep);
    if cut == 0 || rows[cut].starts {
        rows.drain(..cut);
        return;
    }
    if let Some(next) = rows[cut..].iter().position(|row| row.starts) {
        rows.drain(..cut + next);
        return;
    }
    let Some(speaker) = rows[..cut].iter().rposition(|row| row.starts) else {
        rows.drain(..cut);
        return;
    };
    if keep < 2 {
        rows.drain(..cut);
        return;
    }
    rows.drain(speaker + 1..=cut);
    rows.drain(..speaker);
    let extra = rows.len().saturating_sub(keep);
    rows.drain(1..=extra);
}

fn rows_of(lines: &[Line], colours: Colours) -> Vec<Row> {
    let mut rows = Vec::new();
    for line in lines {
        if let Some(label) = &line.label {
            rows.push(Row {
                text: label.clone(),
                font: Font::Body,
                colour: colours.name,
                indent: 0,
                read: false,
                starts: true,
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
            read: matches!(line.role, Role::User | Role::Agent),
            starts: false,
        });
    }
    rows
}

fn button(theme: &mut Theme, label: &str, colours: Colours) -> String {
    let [w, h] = BUTTON;
    let plain = style_of(&Look::default(), colours);
    let style = Style {
        radius: colours.radius.map_or(4, |r| (r - 2).max(0)),
        fill: colours.well.unwrap_or(plain.fill),
        ..plain
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

/// The height of a row of the list of helpers.
fn list_row(margin: bool) -> i32 {
    if margin { MARGIN_ROW } else { LINE }
}

/// The width of the bar that marks the item under the pointer.
pub const MARK: [i32; 2] = [2, 20];
/// The gap between that bar and its item.
pub const MARK_GAP: i32 = 12;

/// The id of the mark of an item.
#[must_use]
pub fn mark_id(item: &str) -> String {
    format!("{item}-mark")
}

/// The id of the item that a mark belongs to, for a node of the host that
/// shows a mark while the pointer is over it.
#[must_use]
pub fn marked(node: &str) -> Option<String> {
    let known = [STOP, CLOSE, crate::card::YES, crate::card::NO];
    (known.contains(&node) || pick_of(node).is_some()).then(|| mark_id(node))
}

/// A bar, hidden, that shows left of an item at `at` while the pointer is
/// over it: the one focus mark of mc.
pub fn bar(id: &str, parent: &str, at: [i32; 2], ink: [u8; 3]) -> Command {
    let props = PropsUpdate {
        transform: Some(Transform::translate(
            f64::from(at[0] - MARK_GAP - MARK[0]),
            f64::from(at[1]),
            0.0,
        )),
        size: Some(MARK.map(f64::from)),
        color: Some(unit(ink)),
        opacity: Some(0.0),
        visible: Some(true),
        input: Some(InputMode::Pass),
        ..PropsUpdate::default()
    };
    Command::Create {
        id: mark_id(id),
        kind: NodeKind::Rect,
        parent: parent.into(),
        props,
    }
}

/// A frame of 2 pixels around a button at `at`, hidden, that shows while
/// the pointer is over the button.
pub fn frame(id: &str, parent: &str, (at, size): ([i32; 2], [i32; 2]), ink: [u8; 3]) -> Command {
    let props = PropsUpdate {
        transform: Some(Transform::translate(
            f64::from(at[0]),
            f64::from(at[1]),
            0.0,
        )),
        size: Some(size.map(f64::from)),
        color: Some([0.0; 4]),
        border: Some(Some(chora_scene::Border {
            width: 2.0,
            color: unit(ink),
        })),
        opacity: Some(0.0),
        visible: Some(true),
        input: Some(InputMode::Pass),
        ..PropsUpdate::default()
    };
    Command::Create {
        id: mark_id(id),
        kind: NodeKind::Rect,
        parent: parent.into(),
        props,
    }
}

/// An opaque colour as the scene takes it.
fn unit([r, g, b]: [u8; 3]) -> [f64; 4] {
    [
        f64::from(r) / 255.0,
        f64::from(g) / 255.0,
        f64::from(b) / 255.0,
        1.0,
    ]
}

/// Shows the mark of `node` where `commands` create it, so that a view or a
/// card drawn again under the pointer keeps its mark.
pub fn keep_mark(commands: &mut [Command], node: &str) {
    let wanted = mark_id(node);
    for command in commands {
        if let Command::Create { id, props, .. } = command
            && *id == wanted
        {
            props.opacity = Some(1.0);
        }
    }
}

/// The command that shows or hides a mark.
#[must_use]
pub fn show_mark(mark: &str, shown: bool) -> Command {
    Command::Update {
        id: mark.into(),
        props: PropsUpdate {
            opacity: Some(if shown { 1.0 } else { 0.0 }),
            ..PropsUpdate::default()
        },
    }
}

/// The buttons of the view, left to right: Close in every view, so that a
/// person who uses only the mouse can always leave; Stop only in a chat. A
/// view in the margin of a client has no Close: the client has it in its
/// head of the chat.
fn buttons(mode: &Mode, column: bool) -> &'static [(&'static str, Word)] {
    match (mode, column) {
        (Mode::Chat(_), false) => &[(STOP, Word::Stop), (CLOSE, Word::Close)],
        (Mode::Chat(_), true) => &[(STOP, Word::Stop)],
        (_, false) => &[(CLOSE, Word::Close)],
        (_, true) => &[],
    }
}

/// Whether the view stands in the margin of a client: a plain view in a
/// slot, which has no header of its own, since the client names the helper.
fn in_margin(frame: &Frame) -> bool {
    frame.slot.is_some() && frame.look.style == ChatStyle::Plain
}

/// What a view shows, laid out.
struct Parts {
    rect: [i32; 4],
    face: Option<Face>,
    row_h: i32,
    margin: bool,
    inset: i32,
    header: String,
    hint: Vec<String>,
    rows: Vec<Row>,
    list: Vec<String>,
    buttons: &'static [(&'static str, Word)],
    running: bool,
    top_row: i32,
    body_top: i32,
    field_room: i32,
}

/// The width of a character in the text of the view, a little over what
/// it is, so that a wrapped line is never wider than its room.
fn measured<'a>(theme: &'a Theme, face: Option<&'a Face>) -> impl Fn(char) -> f32 + 'a {
    let cache = std::cell::RefCell::new(std::collections::HashMap::new());
    move |c| {
        *cache.borrow_mut().entry(c).or_insert_with(|| {
            let width = face.map_or_else(|| face::sans_advance(theme, c), |f| f.advance(theme, c));
            width * 1.02
        })
    }
}

/// How many of `rows` rows from `start` on the output end above the card
/// at `card_top`, with a gap: the newest stay.
fn rows_above(card_top: Option<i32>, start: i32, (rows, row_h): (usize, i32)) -> usize {
    let Some(top) = card_top else {
        return rows;
    };
    let fit = (top - GAP - start).max(0) / row_h;
    rows.min(usize::try_from(fit).unwrap_or(0))
}

/// The top of a hint of `lines` lines above the field in a view `h` high: a
/// gap above the room that the field takes.
fn hint_top(h: i32, field_room: i32, lines: i32) -> i32 {
    h - field_room - lines * LINE - GAP
}

/// The room that a hint takes above the field, with its gap.
fn hint_room(hint: &[String]) -> i32 {
    if hint.is_empty() {
        return 0;
    }
    i32::try_from(hint.len()).unwrap_or(1) * LINE + GAP
}

/// The lines of a hint: it wraps at its words and is never cut.
fn hint_lines(theme: &Theme, hint: &str, width: i32) -> Vec<String> {
    chora_draw::text::wrap(hint, f64::from(width), |line| {
        theme.text_width(line, Font::Body)
    })
}

/// The rectangle of the view: a balloon of the figure as high as its
/// content, up to its most, its bottom at the tail; any other view as it
/// is.
fn fitted(frame: &Frame, [x, y, w, h]: [i32; 4], content: i32) -> [i32; 4] {
    if frame.tail().is_none() {
        return [x, y, w, h];
    }
    let high = content.min(h);
    [x, y + h - high, w, high]
}

impl Session {
    fn parts(&mut self, theme: &Theme, frame: &Frame, language: Language) -> Parts {
        let colours = colours(&frame.look);
        let rect = frame.rect();
        let face = Face::of(&frame.look);
        let row_h = face.as_ref().map_or(LINE, Face::line);
        let width_of = measured(theme, face.as_ref());
        let margin = in_margin(frame);
        let inset = if margin { 0 } else { PAD };
        let measure = Measure {
            max: f32::from(u16::try_from(rect[2] - 2 * inset).unwrap_or(0)),
            font: look_id(&frame.look),
            width: &width_of,
        };
        let lines = self.lines(language, &measure);
        let hint = (self.hint(language))
            .map(|hint| hint_lines(theme, &hint, rect[2] - 2 * inset))
            .unwrap_or_default();
        let list = self.entries(language);
        let buttons = buttons(&self.mode, margin);
        let top_row = if margin { PAD } else { PAD + LINE + GAP };
        let body_top = match (margin, buttons.is_empty()) {
            (true, true) => PAD,
            (true, false) => PAD + LINE + GAP,
            (false, _) => top_row + BUTTON[1] + GAP,
        };
        let below = field_room(frame) + hint_room(&hint) + self.card_room();
        let listed = list_row(margin) * i32::try_from(list.len()).unwrap_or(0);
        let room = usize::try_from((rect[3] - body_top - listed - below) / row_h).unwrap_or(0);
        let mut rows = rows_of(&lines, colours);
        keep_newest(&mut rows, room);
        let content =
            body_top + listed + row_h * i32::try_from(rows.len()).unwrap_or(0) + GAP + below;
        let rect = fitted(frame, rect, content);
        let above = rows_above(
            self.card_top(),
            rect[1] + body_top + listed,
            (rows.len(), row_h),
        );
        keep_newest(&mut rows, above);
        Parts {
            rect,
            face: face.clone(),
            row_h,
            margin,
            inset,
            header: if margin {
                String::new()
            } else {
                self.header(language)
            },
            hint,
            rows,
            list,
            buttons,
            running: self.running(),
            top_row,
            body_top,
            field_room: field_room(frame),
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
        if frame.look.style == ChatStyle::Glass {
            commands.extend(self.glass_commands(theme, frame, language));
            if top {
                commands.push(Command::Move {
                    id: crate::card::TOP.into(),
                    parent: None,
                    index: None,
                });
            }
            self.drawn = Some(frame.output.clone());
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

/// The page of a view in the margin of a client: its fill, no box.
fn page_shape([w, h]: [i32; 2], colours: Colours) -> String {
    let [red, green, blue] = colours.fill;
    format!(r#"<rect width="{w}" height="{h}" fill="rgb({red},{green},{blue})"/>"#)
}

/// A word as a link in the margin: text in the colour of the names, with no
/// box, that takes the click.
fn link(theme: &mut Theme, label: &str, colours: Colours) -> (String, [i32; 2]) {
    #[allow(clippy::cast_possible_truncation, reason = "a width of a few pixels")]
    let w = theme.text_width(label, Font::Body).ceil() as i32;
    let picture = Picture::new(w, LINE).text(Text::new(
        label,
        Font::Body,
        rgba(colours.name),
        [0, 0, w, LINE],
    ));
    (theme.render(&picture), [w, LINE])
}

/// The outline of the view: glass, or a box, with a tail at the figure
/// when the view is its balloon.
fn shape_of(frame: &Frame, size: [i32; 2], colours: Colours) -> String {
    if frame.look.style == ChatStyle::Glass {
        return glass(size, colours);
    }
    let tail = frame.tail();
    let style = Style {
        tail: tail.map_or(0, |_| crate::config::TAIL),
        ..style_of(&frame.look, colours)
    };
    style.shape(size, tail.unwrap_or(0))
}

/// The rows of the transcript on the picture of the view: the messages in
/// the face of the look where it has one, the rest in the sans.
fn rows_picture(mut picture: Picture, parts: &Parts, w: i32) -> Picture {
    let below_list =
        parts.body_top + list_row(parts.margin) * i32::try_from(parts.list.len()).unwrap_or(0);
    for (n, row) in (0..).zip(&parts.rows) {
        let y = below_list + n * parts.row_h;
        let face = parts.face.as_ref().filter(|_| row.read);
        picture = if let Some(face) = face {
            picture.svg(face.element(&row.text, [parts.inset + row.indent, y], row.colour))
        } else {
            let at = [
                parts.inset + row.indent,
                y,
                w - parts.inset - row.indent,
                parts.row_h,
            ];
            picture.text(Text::new(&row.text, row.font, rgba(row.colour), at))
        };
    }
    picture
}

/// The rows of the list of helpers on the picture of the view: the number
/// in `name`, the words in the text colour.
fn list_picture(mut picture: Picture, parts: &Parts, (w, colours): (i32, Colours)) -> Picture {
    let row = list_row(parts.margin);
    for (n, entry) in (0..).zip(&parts.list) {
        let top = parts.body_top + n * row + (row - LINE) / 2;
        let (number, words) = entry.split_once("  ").unwrap_or(("", entry));
        let at = [parts.inset, top, NUMBER_ROOM, LINE];
        picture = picture.text(Text::new(number, Font::Body, rgba(colours.name), at));
        let at = [
            parts.inset + NUMBER_ROOM,
            top,
            w - 2 * parts.inset - NUMBER_ROOM,
            LINE,
        ];
        picture = picture.text(Text::new(words, Font::Body, rgba(colours.text), at));
    }
    picture
}

fn panel(theme: &mut Theme, frame: &Frame, parts: &Parts, colours: Colours) -> Vec<Command> {
    let [x, y, w, h] = parts.rect;
    let is_glass = frame.look.style == ChatStyle::Glass;
    let tail = frame.tail();
    let shape = if parts.margin {
        page_shape([w, h], colours)
    } else {
        shape_of(frame, [w, h], colours)
    };
    let halo = frame
        .look
        .glow
        .filter(|_| is_glass)
        .map(|colour| glow(theme, [w, h], colour));
    let below = tail.map_or(0, |_| crate::config::TAIL);
    let mut picture = Picture::new(w, h + below).svg(shape);
    if !parts.header.is_empty() {
        picture = picture.text(Text::new(
            &parts.header,
            Font::BodyBold,
            rgba(colours.text),
            [PAD, PAD, w - 2 * PAD, LINE],
        ));
    }
    picture = rows_picture(picture, parts, w);
    picture = list_picture(picture, parts, (w, colours));
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
        image(rendered, [0, 0], &shown.join("\n"), InputMode::Accept),
    );
    std::iter::once(view).chain(halo).chain([chat]).collect()
}

fn extras(theme: &mut Theme, parts: &Parts, colours: Colours, language: Language) -> Vec<Command> {
    let [_, _, w, h] = parts.rect;
    let mut commands = Vec::new();
    if !parts.hint.is_empty() {
        let lines = i32::try_from(parts.hint.len()).unwrap_or(1);
        let size = [w - 2 * parts.inset, lines * LINE];
        let mut picture = Picture::new(size[0], size[1]);
        for (n, line) in (0..).zip(&parts.hint) {
            let at = [0, n * LINE, size[0], LINE];
            picture = picture.text(Text::new(line, Font::Body, rgba(colours.text), at));
        }
        let path = theme.render(&picture);
        let at = [parts.inset, hint_top(h, parts.field_room, lines)];
        let text = parts.hint.join(" ");
        commands.push(create(HINT, VIEW, image(path, at, &text, InputMode::Pass)));
    }
    for (n, (id, word)) in (0..).zip(parts.buttons) {
        if *id == STOP && !parts.running {
            continue;
        }
        let label = word.text(language);
        let (path, at) = if parts.margin {
            let (path, [width, _]) = link(theme, label, colours);
            (path, [w - width, parts.top_row])
        } else {
            let at = [PAD + n * (BUTTON[0] + GAP), parts.top_row];
            (button(theme, label, colours), at)
        };
        commands.push(create(id, VIEW, image(path, at, label, InputMode::Accept)));
        commands.push(if parts.margin {
            bar(id, VIEW, at, colours.text)
        } else {
            frame(id, VIEW, (at, BUTTON), colours.text)
        });
    }
    let row = list_row(parts.margin);
    for (n, entry) in (0..).zip(&parts.list) {
        let blank = theme.render(&Picture::new(w - 2 * parts.inset, row));
        let at = [parts.inset, parts.body_top + n * row];
        let id = format!("{PICK}{}", n + 1);
        commands.push(create(
            &id,
            VIEW,
            image(blank, at, entry, InputMode::Accept),
        ));
        let bar_at = [at[0], at[1] + (row - MARK[1]) / 2];
        commands.push(bar(&id, VIEW, bar_at, colours.text));
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
    fn close_shows_in_the_list_and_in_the_chat_and_stop_only_in_the_chat() {
        let list = Mode::List(Vec::new());
        assert_eq!(buttons(&list, false), [(CLOSE, Word::Close)]);
        let chat = Mode::Chat("atlas".into());
        assert_eq!(
            buttons(&chat, false),
            [(STOP, Word::Stop), (CLOSE, Word::Close)]
        );
        assert_eq!(
            buttons(&chat, true),
            [(STOP, Word::Stop)],
            "the margin has the Close"
        );
        assert_eq!(buttons(&list, true), []);
    }

    #[test]
    fn a_balloon_is_as_high_as_its_content_up_to_its_most() {
        let (theme, _, cache) = crate::tests::loaded("balloon-height");
        let mut frame = Frame {
            output: "HEADLESS-1".into(),
            size: [1280, 720],
            area: [0, 0, 1280, 690],
            look: Look {
                style: ChatStyle::Bubble,
                width: Some(340),
                ..Look::default()
            },
            slot: None,
            clear: Vec::new(),
            tip: Some([1144, 560]),
            field: 36,
            ring: None,
            dark: false,
            scale: 1,
            card: None,
        };
        let most = frame.rect();
        let mut session = Session::new();
        session.agent_said("Atlas is writing your notes.");
        let short = session.parts(&theme, &frame, Language::En).rect;
        assert!(short[3] < most[3] / 2, "{short:?} in {most:?}");
        assert_eq!(
            short[1] + short[3],
            most[1] + most[3],
            "the tail stays at the head"
        );
        session.agent_said(&"many words ".repeat(400));
        let full = session.parts(&theme, &frame, Language::En).rect;
        assert_eq!(full, most, "it grows up to its most");
        frame.field = 100;
        session.agent_said("Atlas is writing your notes.");
        let four = session.parts(&theme, &frame, Language::En).rect;
        assert_eq!(
            four[3] - short[3],
            100 - 36,
            "the balloon grows with its field"
        );
        frame.tip = None;
        assert_eq!(
            session.parts(&theme, &frame, Language::En).rect,
            frame.rect(),
            "a panel keeps its size"
        );
        let _ = std::fs::remove_dir_all(cache);
    }

    fn row(text: &str, starts: bool) -> Row {
        Row {
            text: text.into(),
            font: Font::Body,
            colour: [0; 3],
            indent: 0,
            read: !starts,
            starts,
        }
    }

    fn texts(rows: &[Row]) -> Vec<&str> {
        rows.iter().map(|row| row.text.as_str()).collect()
    }

    #[test]
    fn the_transcript_scrolls_by_whole_messages() {
        let all = || {
            vec![
                row("You", true),
                row("Tidy my notes.", false),
                row("Atlas", true),
                row("I read 14 notes", false),
                row("and made 5 days.", false),
                row("Moving them", false),
            ]
        };
        let mut rows = all();
        keep_newest(&mut rows, 6);
        assert_eq!(rows.len(), 6, "nothing cut");
        let mut rows = all();
        keep_newest(&mut rows, 5);
        assert_eq!(texts(&rows)[0], "Atlas", "the cut leaves no orphan line");
        let mut rows = all();
        keep_newest(&mut rows, 2);
        assert_eq!(
            texts(&rows),
            ["Atlas", "Moving them"],
            "the end of a long message stands under its speaker"
        );
        let mut rows = all();
        keep_newest(&mut rows, 3);
        assert_eq!(texts(&rows), ["Atlas", "and made 5 days.", "Moving them"]);
    }

    #[test]
    fn the_rows_end_above_the_card_where_it_stands() {
        assert_eq!(rows_above(None, 100, (10, LINE)), 10, "no card");
        assert_eq!(rows_above(Some(100 + 3 * LINE + GAP), 100, (10, LINE)), 3);
        assert_eq!(
            rows_above(Some(100 + 3 * LINE + GAP - 1), 100, (10, LINE)),
            2,
            "no half row"
        );
        assert_eq!(rows_above(Some(50), 100, (10, LINE)), 0);
    }

    #[test]
    fn the_hint_stands_above_the_field_whatever_its_height() {
        let look = Look::default();
        for field in [36, 56, 92, 200] {
            let frame = Frame {
                output: "HEADLESS-1".into(),
                size: [1280, 800],
                area: [0, 0, 1280, 800],
                look: look.clone(),
                slot: Some([888, 98, 336, 674]),
                clear: Vec::new(),
                tip: None,
                field,
                ring: None,
                dark: false,
                scale: 1,
                card: None,
            };
            let h = frame.rect()[3];
            let end = hint_top(h, field_room(&frame), 1) + LINE;
            assert!(
                end + GAP <= h - field,
                "{end} meets the field of {field} in {h}"
            );
        }
    }

    #[test]
    fn stop_and_the_entries_refuse_a_press_of_a_program() {
        assert!(refuses_injected(STOP));
        assert!(refuses_injected("hud-chat-pick-2"));
        assert!(!refuses_injected(ROW));
        assert!(!refuses_injected(CLOSE));
    }
}
