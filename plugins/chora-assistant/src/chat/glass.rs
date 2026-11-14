//! The glass chat of the construct HUD (spec, section 5): the column at
//! the right with the header of the helper, the transcript, the room of
//! the field and the key line. The person's messages stand at the right in
//! accent bubbles, the helper's under its name, its tool steps in `data`
//! with a check or an arc. Stop and Close are no buttons: the key line
//! names their keys, and a click on its words does the same.
#![allow(
    clippy::many_single_char_names,
    reason = "x, y, w, h of shapes, as SVG names them"
)]

use chora_draw::{
    text::wrap,
    theme::{Picture, Theme},
};
use chora_kit::{
    chat::{Line as Said, Measure, Role},
    glass::{CAPSULE, Material, PANEL_RADIUS, Type, caps},
    words::{self, Language, Word, say},
};
use chora_scene::{Command, InputMode, NodeKind, PropsUpdate, Transform};

use super::{
    Frame, Mode, Session,
    view::{CHAT, CLOSE, HINT, PICK, STOP, VIEW, create, image},
};
use crate::glass::{Line, keycaps, paint, rect, shadow, surface};

/// A node of the key line that takes a click: its id, rect and words.
type Hit = (&'static str, [i32; 4], String);

/// The rect of the tint of the column over the blur.
const CHAT_GLASS: &str = "hud-chat-glass";

/// The shadow and the hairline of the column.
pub const SHADOW: &str = "hud-chat-shadow";
/// The height of the header, of the field with its margin and of the key
/// line; the padding at the sides.
const HEADER: f64 = 56.0;
const FIELD: f64 = 48.0;
const KEY_LINE: f64 = CAPSULE;
const PAD: f64 = 16.0;
/// The widest bubble of the person, and its padding.
const BUBBLE: f64 = 300.0;
const BUBBLE_PAD: [f64; 2] = [16.0, 8.0];
/// The gap between messages, and between a message and its steps.
const GAP: f64 = 16.0;
const STEP_GAP: f64 = 4.0;

/// A message of the transcript: its role, its name and its text.
struct Message {
    role: Role,
    label: Option<String>,
    text: String,
}

/// The messages of the lines of the transcript, each line of a step on its
/// own.
fn messages(lines: Vec<Said>) -> Vec<Message> {
    let mut out: Vec<Message> = Vec::new();
    for line in lines {
        let continues = line.label.is_none()
            && matches!(line.role, Role::User | Role::Agent)
            && out.last().is_some_and(|m| m.role == line.role);
        match out.last_mut() {
            Some(last) if continues => {
                last.text.push('\n');
                last.text += &line.text;
            }
            _ => out.push(Message {
                role: line.role,
                label: line.label,
                text: line.text,
            }),
        }
    }
    out
}

/// A message laid out: its height, the gap above it and its SVG at a top.
struct Laid {
    height: f64,
    gap: f64,
    draw: Box<dyn Fn(f64) -> String>,
}

/// The parts of the view that a frame and the session give.
struct Column<'a> {
    theme: &'a Theme,
    m: Material,
    w: f64,
    h: f64,
    /// The bottom of a card in the view, in its coordinates.
    under: Option<f64>,
}

impl Column<'_> {
    fn wrapped(&self, text: &str, role: Type, max: f64) -> Vec<String> {
        let measure = |line: &str| Line::new(line, role, self.m.text).width(self.theme);
        wrap(text, max, measure)
    }

    fn bubble(&self, text: &str) -> Laid {
        let lines = self.wrapped(text, Type::Text, BUBBLE - 2.0 * BUBBLE_PAD[0]);
        let ink = (lines.iter())
            .map(|l| Line::new(l, Type::Text, self.m.text).width(self.theme))
            .fold(0.0, f64::max);
        let bw = (ink + 2.0 * BUBBLE_PAD[0]).ceil();
        #[allow(clippy::cast_precision_loss, reason = "lines of a message")]
        let bh = 24.0 * lines.len() as f64 + 2.0 * BUBBLE_PAD[1];
        let (m, w) = (self.m, self.w);
        Laid {
            height: bh,
            gap: GAP,
            draw: Box::new(move |top| {
                let x = w - PAD - bw;
                let mut svg = corners([x, top, bw, bh], [20.0, 20.0, 6.0, 20.0], m.bubble);
                for (k, line) in lines.iter().enumerate() {
                    #[allow(clippy::cast_precision_loss, reason = "lines of a message")]
                    let y = top + BUBBLE_PAD[1] + 24.0 * k as f64;
                    svg += &Line::new(line, Type::Text, m.text).at([x + BUBBLE_PAD[0], y], false);
                }
                svg
            }),
        }
    }

    fn helper(&self, label: Option<&str>, text: &str, role: Role) -> Laid {
        let lines = self.wrapped(text, Type::Text, self.w - 2.0 * PAD);
        let head = if label.is_some() { 20.0 } else { 0.0 };
        #[allow(clippy::cast_precision_loss, reason = "lines of a message")]
        let height = head + 24.0 * lines.len() as f64;
        let (m, label) = (self.m, label.map(str::to_owned));
        let weight = if role == Role::Question { 500 } else { 400 };
        Laid {
            height,
            gap: GAP,
            draw: Box::new(move |top| {
                let mut svg = label.as_deref().map_or_else(String::new, |name| {
                    Line::new(name, Type::Label, m.muted).at([PAD, top], false)
                });
                for (k, line) in lines.iter().enumerate() {
                    #[allow(clippy::cast_precision_loss, reason = "lines of a message")]
                    let y = top + head + 24.0 * k as f64;
                    let text = Line::new(line, Type::Text, m.text).weight(weight);
                    svg += &text.at([PAD, y], false);
                }
                svg
            }),
        }
    }

    fn step(&self, text: &str, running: bool) -> Laid {
        let line = self
            .wrapped(text, Type::Data, self.w - 2.0 * PAD - 20.0)
            .into_iter()
            .next()
            .unwrap_or_default();
        let m = self.m;
        Laid {
            height: 20.0,
            gap: STEP_GAP,
            draw: Box::new(move |top| {
                let (x, y) = (PAD, top + 4.0);
                let icon = if running {
                    format!(
                        r#"<circle cx="{}" cy="{}" r="4.2" fill="none" stroke-width="1.6" stroke-dasharray="7 20" stroke-linecap="round" {}/>"#,
                        x + 6.0,
                        y + 6.0,
                        paint("stroke", m.accent)
                    )
                } else {
                    format!(
                        r#"<path d="M{} {}l2.3 2.3 4.7-5" fill="none" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" {}/>"#,
                        x + 2.5,
                        y + 6.2,
                        paint("stroke", m.muted)
                    )
                };
                icon + &Line::new(&line, Type::Data, m.muted).at([PAD + 20.0, top], false)
            }),
        }
    }

    fn note(&self, text: &str) -> Laid {
        let lines = self.wrapped(text, Type::Small, self.w - 2.0 * PAD);
        #[allow(clippy::cast_precision_loss, reason = "lines of a note")]
        let height = 20.0 * lines.len() as f64;
        let m = self.m;
        Laid {
            height,
            gap: GAP,
            draw: Box::new(move |top| {
                let mut svg = String::new();
                for (k, line) in lines.iter().enumerate() {
                    #[allow(clippy::cast_precision_loss, reason = "lines of a note")]
                    let y = top + 20.0 * k as f64;
                    svg += &Line::new(line, Type::Small, m.muted).at([PAD, y], false);
                }
                svg
            }),
        }
    }

    /// The top and the bottom of the transcript.
    fn room(&self) -> [f64; 2] {
        let top = self.under.map_or(HEADER + 1.0 + 8.0, |bottom| bottom + GAP);
        [top, self.h - KEY_LINE - FIELD - 8.0 - 8.0]
    }

    /// The newest messages that fit the room, top down.
    fn transcript(&self, laid: &[Laid], hint: bool) -> String {
        let [top, mut bottom] = self.room();
        if hint {
            bottom -= 28.0;
        }
        let mut used = 0.0;
        let mut first = laid.len();
        for (k, item) in laid.iter().enumerate().rev() {
            let below = laid.get(first).map_or(0.0, |next| next.gap);
            let need = item.height + below;
            if used + need > bottom - top {
                break;
            }
            used += need;
            first = k;
        }
        let mut svg = String::new();
        let mut y = top;
        for (k, item) in laid.iter().enumerate().skip(first) {
            if k > first {
                y += item.gap;
            }
            svg += &(item.draw)(y);
            y += item.height;
        }
        svg
    }
}

/// A rounded rect with its own corners: top left, top right, bottom right,
/// bottom left.
fn corners([x, y, w, h]: [f64; 4], [a, b, c, d]: [f64; 4], fill: [f64; 4]) -> String {
    format!(
        r#"<path d="M{} {y}H{}A{b} {b} 0 0 1 {} {}V{}A{c} {c} 0 0 1 {} {}H{}A{d} {d} 0 0 1 {x} {}V{}A{a} {a} 0 0 1 {} {y}Z" {}/>"#,
        x + a,
        x + w - b,
        x + w,
        y + b,
        y + h - c,
        x + w - c,
        y + h,
        x + d,
        y + h - d,
        y + a,
        x + a,
        paint("fill", fill)
    )
}

/// The keys and the words of the key line, left and right, and the node
/// that a click on each takes.
fn key_line(language: Language) -> [(Vec<String>, String, Option<&'static str>); 3] {
    [
        (caps("Return"), Word::Send.text(language).to_owned(), None),
        (
            caps("Ctrl+period"),
            Word::Stop.text(language).to_owned(),
            Some(STOP),
        ),
        (
            caps("Escape"),
            Word::Close.text(language).to_owned(),
            Some(CLOSE),
        ),
    ]
}

impl Session {
    /// The state word and the line of the helper of the header:
    /// `WORKING` and `Atlas · Tidy notes · 1 of 2`.
    fn glass_head(&self, language: Language) -> (String, String) {
        if !matches!(self.mode, Mode::Chat(_)) || self.offline {
            return (String::new(), self.header(language));
        }
        let word = words::state(&self.status.state).map_or_else(
            || self.status.state.clone(),
            |w| w.text(language).to_owned(),
        );
        let mut parts = vec![self.label()];
        if !self.status.task.is_empty() {
            parts.push(self.status.task.clone());
        }
        if let Some((step, steps)) = self.status.steps {
            let (step, steps) = (step.to_string(), steps.to_string());
            parts.push(say(
                language,
                Word::Step,
                &[("step", &step), ("steps", &steps)],
            ));
        }
        (word, parts.join(" · "))
    }

    /// The commands of a glass view, from nothing.
    pub(super) fn glass_commands(
        &mut self,
        theme: &mut Theme,
        frame: &Frame,
        language: Language,
    ) -> Vec<Command> {
        let [x, y, w, h] = frame.rect();
        let m = Material::of(frame.dark);
        let size = [f64::from(w), f64::from(h)];
        let hint = self.hint(language);
        let entries = self.entries(language);
        let (picture, hits, shown) = self.glass_picture(theme, frame, (language, hint.is_some()));
        let mut commands = vec![
            Command::Create {
                id: VIEW.into(),
                kind: NodeKind::Group,
                parent: format!("screen:{}", frame.output),
                props: PropsUpdate {
                    transform: Some(Transform::translate(f64::from(x), f64::from(y), 0.0)),
                    ..PropsUpdate::default()
                },
            },
            create(
                SHADOW,
                VIEW,
                image(
                    shadow(theme, size, PANEL_RADIUS, &m, false),
                    [-crate::glass::REACH, -crate::glass::REACH],
                    "",
                    InputMode::Pass,
                ),
            ),
        ];
        commands.extend(crate::glass::backdrop(
            (CHAT_GLASS, VIEW),
            [0.0, 0.0, size[0], size[1]],
            (PANEL_RADIUS, true),
            (&m, true),
        ));
        let path = theme.render(&picture);
        commands.push(create(
            CHAT,
            VIEW,
            image(path, [0, 0], &shown, InputMode::Pass),
        ));
        commands.extend(hint.map(|hint| hint_node(theme, &hint, size, &m)));
        commands.extend(click_nodes(theme, hits, &entries, w));
        commands
    }

    /// The picture of the column, the nodes of the key line that take a
    /// click, and the words that it shows.
    fn glass_picture(
        &mut self,
        theme: &Theme,
        frame: &Frame,
        (language, hint): (Language, bool),
    ) -> (Picture, Vec<Hit>, String) {
        let [_, y, w, h] = frame.rect();
        let column = Column {
            theme,
            m: Material::of(frame.dark),
            w: f64::from(w),
            h: f64::from(h),
            under: frame.card.map(|[_, cy, _, ch]| f64::from(cy + ch - y)),
        };
        let wide = |_: char| 0.0_f32;
        let measure = Measure {
            max: 1.0e9,
            font: 1,
            width: &wide,
        };
        let said = messages(self.lines(language, &measure));
        let running = self.status.state == "working";
        let last_step = said.iter().rposition(|m| m.role == Role::Tool);
        let laid: Vec<Laid> = (said.iter().enumerate())
            .map(|(k, m)| match m.role {
                Role::User => column.bubble(&m.text),
                Role::Tool | Role::Plan => column.step(&m.text, running && Some(k) == last_step),
                Role::Notice => column.note(&m.text),
                Role::Agent | Role::Question => column.helper(m.label.as_deref(), &m.text, m.role),
            })
            .collect();
        let mut body = surface([column.w, column.h], PANEL_RADIUS, &column.m, true);
        let (header, ring) = self.glass_header(&column, frame, language);
        body += &header;
        body += &column.transcript(&laid, hint);
        let [room_top, _] = column.room();
        for (n, entry) in (0..).zip(&self.entries(language)) {
            let top = room_top + 40.0 * f64::from(n);
            body += &Line::new(entry, Type::Text, column.m.text).at([PAD, top + 8.0], false);
        }
        let (keys, hits) = glass_keys(&column, language);
        body += &keys;
        let mut picture = Picture::new(w, h).svg(body);
        if let Some((sheet, id)) = ring {
            #[allow(clippy::cast_possible_truncation, reason = "a few pixels")]
            let top = ((HEADER - 24.0) / 2.0) as i32;
            picture = picture.element(sheet, id, [16, top, 24, 24]);
        }
        let (word, who) = self.glass_head(language);
        let shown = [word.as_str(), who.as_str()]
            .into_iter()
            .filter(|part| !part.is_empty())
            .chain(said.iter().map(|m| m.text.as_str()))
            .collect::<Vec<_>>()
            .join("\n");
        (picture, hits, shown)
    }

    fn glass_header(
        &self,
        column: &Column<'_>,
        frame: &Frame,
        language: Language,
    ) -> (String, Option<(chora_draw::theme::Sheet, &'static str)>) {
        let m = &column.m;
        let (word, who) = self.glass_head(language);
        let mut svg = String::new();
        let mut text_x = PAD;
        let ring = frame.ring.filter(|_| !word.is_empty()).map(|sheet| {
            let state = crate::assistant::context_state(&self.status.state);
            let id = if column.theme.size(sheet, state).is_some() {
                state
            } else {
                "idle"
            };
            (sheet, id)
        });
        if ring.is_some() {
            text_x += 24.0 + 8.0;
        }
        if !word.is_empty() {
            svg +=
                &Line::new(&word, Type::Label, m.text).at([text_x, (HEADER - 16.0) / 2.0], false);
        }
        let who_line = Line::new(&who, Type::Small, m.muted);
        let anchor_end = !word.is_empty();
        let at = if anchor_end { column.w - PAD } else { text_x };
        svg += &who_line.at([at, (HEADER - 20.0) / 2.0], anchor_end);
        svg += &rect([8.0, HEADER, column.w - 16.0, 1.0], 0.0, m.separator);
        (svg, ring)
    }
}

/// The key line at the bottom of the column, and the nodes that take a
/// click on its words: id, rect and words.
fn glass_keys(column: &Column<'_>, language: Language) -> (String, Vec<Hit>) {
    let m = &column.m;
    let top = column.h - KEY_LINE + 4.0;
    let mut svg = String::new();
    let mut hits = Vec::new();
    let line = key_line(language);
    let mut x = PAD;
    let place = |keys: &[String], word: &str, at: f64, svg: &mut String| -> f64 {
        let (caps_svg, caps_w) = keycaps(column.theme, keys, [at, top], m);
        *svg += &caps_svg;
        let label = Line::new(word, Type::Small, m.muted);
        *svg += &label.at([at + caps_w + 6.0, top + 2.0], false);
        caps_w + 6.0 + label.width(column.theme)
    };
    for (keys, word, id) in &line[..2] {
        let width = place(keys, word, x, &mut svg);
        if let Some(id) = id {
            hits.push((*id, hit([x, top, width]), word.clone()));
        }
        x += width + 16.0;
    }
    let (keys, word, id) = &line[2];
    let right_w = {
        let caps_w = crate::glass::keycaps_width(column.theme, keys, m);
        caps_w + 6.0 + Line::new(word, Type::Small, m.muted).width(column.theme)
    };
    let right = column.w - PAD - right_w.ceil();
    place(keys, word, right, &mut svg);
    if let Some(id) = id {
        hits.push((*id, hit([right, top, right_w]), word.clone()));
    }
    (svg, hits)
}

/// The node of the line that typing does not answer, above the field.
fn hint_node(theme: &mut Theme, hint: &str, size: [f64; 2], m: &Material) -> Command {
    let line = Line::new(hint, Type::Small, m.muted);
    #[allow(clippy::cast_possible_truncation, reason = "a line of a chat")]
    let width = (size[0] - 2.0 * PAD) as i32;
    let picture = Picture::new(width, 20).svg(line.at([0.0, 0.0], false));
    #[allow(clippy::cast_possible_truncation, reason = "a line of a chat")]
    let top = (size[1] - KEY_LINE - FIELD - 8.0 - 28.0) as i32;
    let path = theme.render(&picture);
    create(HINT, VIEW, image(path, [16, top], hint, InputMode::Pass))
}

/// The clear nodes that take the clicks of the key line and of the entries
/// of the list.
fn click_nodes(theme: &mut Theme, hits: Vec<Hit>, entries: &[String], w: i32) -> Vec<Command> {
    let mut commands = Vec::new();
    for (id, [hx, hy, hw, hh], label) in hits {
        let blank = theme.render(&Picture::new(hw, hh));
        commands.push(create(
            id,
            VIEW,
            image(blank, [hx, hy], &label, InputMode::Accept),
        ));
    }
    #[allow(clippy::cast_possible_truncation, reason = "a row of a chat")]
    let room_top = (HEADER + 9.0) as i32;
    for (n, entry) in (0..).zip(entries) {
        let blank = theme.render(&Picture::new(w - 32, 40));
        let id = format!("{PICK}{}", n + 1);
        let at = [16, room_top + 40 * n];
        commands.push(create(
            &id,
            VIEW,
            image(blank, at, entry, InputMode::Accept),
        ));
    }
    commands
}

/// The rect of a hit area of the key line around content at `x`, `top`,
/// `width`.
#[allow(clippy::cast_possible_truncation, reason = "a few hundred pixels")]
fn hit([x, top, width]: [f64; 3]) -> [i32; 4] {
    [
        (x - 4.0).floor() as i32,
        (top - 4.0).floor() as i32,
        (width + 8.0).ceil() as i32,
        32,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lines_of_a_message_join_and_each_step_stands_alone() {
        let line = |role, label: Option<&str>, text: &str| Said {
            role,
            label: label.map(str::to_owned),
            text: text.into(),
        };
        let joined = messages(vec![
            line(Role::User, Some("You"), "Please tidy"),
            line(Role::User, None, "the notes"),
            line(Role::Agent, Some("Atlas"), "I read notes.txt."),
            line(Role::Tool, None, "Read notes.txt"),
            line(Role::Tool, None, "Sort 14 items"),
        ]);
        let texts: Vec<&str> = joined.iter().map(|m| m.text.as_str()).collect();
        assert_eq!(
            texts,
            [
                "Please tidy\nthe notes",
                "I read notes.txt.",
                "Read notes.txt",
                "Sort 14 items"
            ]
        );
    }

    #[test]
    fn the_key_line_names_return_ctrl_period_and_esc() {
        let line = key_line(Language::En);
        assert_eq!(line[0].0, ["Return"]);
        assert_eq!(line[1].0, ["Ctrl", "."]);
        assert_eq!(line[2].0, ["Esc"]);
        assert_eq!((line[1].2, line[2].2), (Some(STOP), Some(CLOSE)));
    }
}
