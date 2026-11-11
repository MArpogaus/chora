//! Help and Support: an ordinary window of the plugin with a blue task pane
//! of topics on the left and a white page on the right. The page
//! "Keyboard shortcuts" lists the live bindings of the plugin, grouped,
//! one key cap per chord.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "rows and pixels of a small page"
)]

use std::fmt::Write as _;

use chora_kit::give;

use crate::{
    dialog::{Control, Dialog, Hit, Page},
    text::{Font, Text},
    theme::{Picture, Rect, Theme},
    wm::{KEYS, Key},
};

/// The client area: the pane and the page.
pub const SIZE: [i32; 2] = [776, 600];
const PANE: i32 = 220;
const LINK_ROW: i32 = 24;
const NAVY: [u8; 4] = [0, 60, 116, 255];
const LINK: [u8; 4] = [33, 93, 198, 255];
const INK: [u8; 4] = [0, 0, 0, 255];
const GREY: [u8; 4] = [77, 77, 77, 255];
const CAP_HEIGHT: i32 = 24;
const ROW: i32 = 32;
const PAGE_LEFT: i32 = PANE + 24;
const PAGE_WIDTH: i32 = SIZE[0] - PANE - 48;

/// The topics of the task pane, in order. The last is the page that F1
/// opens.
pub const TOPICS: [&str; 5] = [
    "Start a program",
    "Find a window",
    "Chat with a helper",
    "Change the volume",
    "Keyboard shortcuts",
];

/// The topic that F1 opens.
pub const KEYBOARD: usize = 4;

/// The groups of the table of keys, in order.
pub const GROUPS: [&str; 3] = ["Start and find", "The helper", "Windows"];

/// One line of the table: the chords that do one thing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// The group.
    pub group: &'static str,
    /// The chords, in the words of the help.
    pub keys: Vec<String>,
    /// What they do.
    pub text: &'static str,
}

fn name(binding: &str) -> String {
    if binding.ends_with("+Super_L") || binding.ends_with("+Super_R") {
        return "Super".into();
    }
    binding
        .split('+')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                _ if part == "BTN_LEFT" => "drag".to_string(),
                Some(c) if part.len() == 1 => c.to_ascii_uppercase().to_string(),
                Some(c) if part.chars().all(|c| c.is_ascii_lowercase() || c == '_') => {
                    format!("{}{}", c.to_ascii_uppercase(), chars.as_str())
                }
                _ => part.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join("+")
}

fn about(key: Key) -> Option<(&'static str, &'static str)> {
    Some(match key {
        Key::Start | Key::Tap => (GROUPS[0], "Open or close the Start menu"),
        Key::Switch(1) => (GROUPS[0], "Switch between windows"),
        Key::Help => (GROUPS[0], "Open this help"),
        Key::Desktop => (GROUPS[0], "Show the desktop; press again for the windows"),
        Key::Launcher => (GROUPS[0], "Run a program: open the launcher"),
        Key::Files => (GROUPS[0], "Open the file manager"),
        Key::Chat => (GROUPS[1], "Chat with a helper"),
        Key::Close => (GROUPS[2], "Close the window"),
        Key::Turn(-1) => (GROUPS[2], "Turn the desktop cube to the left"),
        Key::Turn(_) => (GROUPS[2], "Turn the desktop cube to the right"),
        Key::Carry(-1) => (GROUPS[2], "Carry the window to the left"),
        Key::Carry(_) => (GROUPS[2], "Carry the window to the right"),
        Key::Cube => (GROUPS[2], "Turn the desktop cube with the pointer"),
        Key::Switch(_) | Key::Volume(_) | Key::Give(_) => return None,
    })
}

/// The rows of the table, made from the bindings that the plugin
/// holds: `held` tells whether a binding is held, and `cube` whether the
/// cube is on.
pub fn rows(held: impl Fn(&str) -> bool) -> Vec<Row> {
    let mut rows: Vec<Row> = Vec::new();
    for (binding, fallback, key) in KEYS {
        let Some((group, text)) = about(key) else {
            continue;
        };
        let shown = [binding, fallback]
            .into_iter()
            .find(|b| !b.is_empty() && held(b));
        let Some(shown) = shown else {
            continue;
        };
        let chord = name(shown);
        match rows.iter_mut().find(|r| r.text == text) {
            Some(row) if row.keys.contains(&chord) => {}
            Some(row) => row.keys.push(chord),
            None => rows.push(Row {
                group,
                keys: vec![chord],
                text,
            }),
        }
    }
    let keys = give::KEYS
        .iter()
        .filter(|(binding, _)| held(binding))
        .map(|(binding, _)| name(binding))
        .collect::<Vec<_>>();
    if let (Some(first), Some(last)) = (keys.first(), keys.last()) {
        let digits = last.rsplit('+').next().unwrap_or_default();
        rows.push(Row {
            group: GROUPS[2],
            keys: vec![format!("{first} ... {digits}")],
            text: "Give the window to tiling, xp, mc or construct",
        });
    }
    rows.sort_by_key(|r| GROUPS.iter().position(|g| *g == r.group));
    rows
}

/// What a page of the help shows besides the keys.
fn lines(topic: usize) -> &'static [&'static str] {
    match topic {
        0 => &[
            "Click Start in the bottom left corner and click a program.",
            "Programs you used lately are listed below the two pinned ones.",
            "All Programs lists every program that is installed.",
        ],
        1 => &[
            "Every open window has a button on the taskbar. Click it to bring the window",
            "forward, or click it again to put the window away.",
            "Alt+Tab shows all windows and takes you to the one you pick.",
        ],
        2 => &[
            "The paperclip in the bottom right corner is your helper. Click it to open the",
            "chat, type what you want in your own words and press Enter.",
            "When the helper needs your permission, a question appears in the chat.",
            "Click Yes or No to answer.",
        ],
        3 => &[
            "Click the speaker in the notification area to change the volume or to mute it.",
            "The volume keys of your keyboard do the same.",
        ],
        _ => &[],
    }
}

/// The open help.
pub struct HelpDialog {
    /// The window.
    pub dialog: Dialog,
    topic: usize,
    rows: Vec<Row>,
}

impl HelpDialog {
    /// The help on a topic, in the middle of a work area, with the rows of
    /// the table of keys.
    pub fn new(topic: usize, rows: Vec<Row>, area: Rect) -> Self {
        Self {
            dialog: Dialog::new("help", "Help and Support", SIZE, area),
            topic: topic.min(TOPICS.len() - 1),
            rows,
        }
    }

    /// Shows a topic.
    pub fn show_topic(&mut self, topic: usize) {
        self.topic = topic.min(TOPICS.len() - 1);
    }

    /// A click: a topic of the pane opens, the close button closes.
    pub fn click(&mut self, hit: &Hit) -> Click {
        let Hit::Control(name, _) = hit else {
            return Click::Close;
        };
        if let Some(i) = name
            .strip_prefix("topic-")
            .and_then(|i| i.parse::<usize>().ok())
        {
            self.show_topic(i);
            return Click::Redraw;
        }
        match name.as_str() {
            "helper" => Click::Helper,
            _ => Click::Redraw,
        }
    }

    /// The page: the face, the pane, the topic.
    pub fn page(&self, theme: &Theme) -> Page {
        let [w, h] = SIZE;
        let mut canvas = Canvas {
            svg: format!(
                r##"<defs><linearGradient id="pane" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#7BA2E7"/><stop offset="1" stop-color="#6375D6"/></linearGradient><linearGradient id="head" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#FFFFFF"/><stop offset="1" stop-color="#C6D3F7"/></linearGradient><linearGradient id="rule" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#3C6FD6"/><stop offset="1" stop-color="#FFFFFF"/></linearGradient><linearGradient id="cap" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#FFFFFF"/><stop offset="1" stop-color="#E3E1D7"/></linearGradient></defs><rect width="{w}" height="{h}" fill="#FFFFFF"/><rect width="{PANE}" height="{h}" fill="url(#pane)"/>"##
            ),
            texts: Vec::new(),
        };
        let controls = self.pane(&mut canvas);
        self.heading(&mut canvas);
        if self.topic == KEYBOARD {
            self.table(theme, &mut canvas);
        } else {
            for (i, line) in lines(self.topic).iter().enumerate() {
                let y = 100 + i as i32 * 24;
                canvas.text(line, Font::Body, INK, [PAGE_LEFT, y, PAGE_WIDTH, 24]);
            }
        }
        let mut picture = Picture::new(w, h).svg(canvas.svg);
        for text in canvas.texts {
            picture = picture.text(text);
        }
        Page {
            body: Some(picture),
            controls,
        }
    }

    fn pane(&self, canvas: &mut Canvas) -> Vec<Control> {
        let mut controls = Vec::new();
        let link = |name: &str, label: &str, rect: Rect, selected: bool| {
            let picture = Picture::new(rect[2], rect[3]).text(Text::new(
                label,
                if selected { Font::BodyBold } else { Font::Body },
                if selected { NAVY } else { LINK },
                [12, 0, rect[2] - 12, rect[3]],
            ));
            Control::area(name, rect, picture)
        };
        let topics_h = 28 + TOPICS.len() as i32 * LINK_ROW + 12;
        group(&mut canvas.svg, [12, 12, PANE - 24, topics_h]);
        canvas.text("Help topics", Font::BodyBold, LINK, [24, 12, 170, 28]);
        for (i, topic) in TOPICS.iter().enumerate() {
            let rect = [12, 40 + i as i32 * LINK_ROW, PANE - 24, LINK_ROW];
            controls.push(link(&format!("topic-{i}"), topic, rect, i == self.topic));
        }
        let see_y = 12 + topics_h + 12;
        group(&mut canvas.svg, [12, see_y, PANE - 24, 28 + LINK_ROW + 12]);
        canvas.text("See also", Font::BodyBold, LINK, [24, see_y, 170, 28]);
        let rect = [12, see_y + 28, PANE - 24, LINK_ROW];
        controls.push(link("helper", "Ask the helper", rect, false));
        controls
    }

    fn heading(&self, canvas: &mut Canvas) {
        let _ = write!(
            canvas.svg,
            r##"<text x="{PAGE_LEFT}" y="42" font-family="Tahoma" font-size="18" font-weight="bold" fill="#003C74">{}</text><rect x="{PAGE_LEFT}" y="82" width="{PAGE_WIDTH}" height="2" fill="url(#rule)"/>"##,
            TOPICS[self.topic]
        );
        let subtitle = if self.topic == KEYBOARD {
            "Each of these also works with the mouse. Super is the key between Ctrl and Alt."
        } else {
            "Everything here also works with the mouse."
        };
        canvas.text(subtitle, Font::Body, GREY, [PAGE_LEFT, 52, PAGE_WIDTH, 24]);
    }

    fn table(&self, theme: &Theme, canvas: &mut Canvas) {
        let mut y = 100;
        let mut group_now = "";
        for row in &self.rows {
            if row.group != group_now {
                group_now = row.group;
                y += 8;
                canvas.text(
                    group_now,
                    Font::BodyBold,
                    NAVY,
                    [PAGE_LEFT, y, PAGE_WIDTH, 24],
                );
                y += 26;
                canvas.rule(y, "#7F9DB9");
            }
            let mut x = PAGE_LEFT;
            for chord in &row.keys {
                let width = theme.text_width(chord, Font::Body).ceil() as i32 + 16;
                canvas.cap(
                    chord,
                    [x, y + (ROW - CAP_HEIGHT) / 2 + 1, width, CAP_HEIGHT],
                );
                x += width + 8;
            }
            let rect = [PAGE_LEFT + 214, y, PAGE_WIDTH - 214, ROW];
            canvas.text(row.text, Font::Body, INK, rect);
            y += ROW;
            canvas.rule(y - 1, "#E6E4DB");
        }
    }
}

/// The picture of a page: its vector shapes and its lines of text.
struct Canvas {
    svg: String,
    texts: Vec<Text>,
}

impl Canvas {
    fn text(&mut self, line: &str, font: Font, color: [u8; 4], rect: Rect) {
        self.texts.push(Text::new(line, font, color, rect));
    }

    fn rule(&mut self, y: i32, color: &str) {
        let _ = write!(
            self.svg,
            r#"<rect x="{PAGE_LEFT}" y="{y}" width="{PAGE_WIDTH}" height="1" fill="{color}"/>"#
        );
    }

    fn cap(&mut self, chord: &str, [x, y, w, h]: Rect) {
        let _ = write!(
            self.svg,
            r##"<rect x="{}" y="{}" width="{}" height="{}" rx="3" fill="url(#cap)" stroke="#8C8A7F"/><rect x="{}" y="{}" width="{}" height="2" fill="#8C8A7F"/>"##,
            f64::from(x) + 0.5,
            f64::from(y) + 0.5,
            w - 1,
            h - 3,
            x + 2,
            y + h - 3,
            w - 4
        );
        self.texts
            .push(Text::new(chord, Font::Body, INK, [x, y - 1, w, h]).centered());
    }
}

fn group(svg: &mut String, [x, y, w, h]: Rect) {
    let _ = write!(
        svg,
        r##"<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="4" fill="#D6DFF7"/><path d="M{x} {}V{}a4 4 0 0 1 4 -4h{}a4 4 0 0 1 4 4V{}z" fill="url(#head)"/>"##,
        y + 28,
        y + 4,
        w - 8,
        y + 28
    );
}

/// What a click in the help did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Click {
    /// Show the page again.
    Redraw,
    /// Close the window.
    Close,
    /// Open the chat.
    Helper,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all(_: &str) -> bool {
        true
    }

    #[test]
    fn every_bound_key_is_in_the_table() {
        let rows = rows(all);
        let shown: Vec<String> = rows.iter().flat_map(|r| r.keys.clone()).collect();
        for (binding, _, key) in KEYS {
            if about(key).is_some() {
                assert!(
                    shown.contains(&name(binding)),
                    "{binding} is bound but not in the table: {shown:?}"
                );
            }
        }
        assert!(shown.iter().any(|k| k == "F1"), "{shown:?}");
        assert!(
            shown.iter().any(|k| k.starts_with("Super+Alt+1 ... 4")),
            "{shown:?}"
        );
    }

    #[test]
    fn a_key_that_is_not_held_is_not_listed() {
        let rows = rows(|b| b != "Super+a");
        assert!(
            rows.iter()
                .all(|r| !r.keys.contains(&"Super+A".to_string()))
        );
    }

    #[test]
    fn the_rows_follow_the_groups() {
        let rows = rows(all);
        let order: Vec<usize> = rows
            .iter()
            .map(|r| GROUPS.iter().position(|g| *g == r.group).unwrap())
            .collect();
        assert!(order.windows(2).all(|w| w[0] <= w[1]), "{order:?}");
    }

    #[test]
    fn chords_read_as_key_caps() {
        assert_eq!(name("Super+e"), "Super+E");
        assert_eq!(name("Ctrl+Escape"), "Ctrl+Escape");
        assert_eq!(name("Alt+F4"), "Alt+F4");
        assert_eq!(name("F1"), "F1");
    }
}
