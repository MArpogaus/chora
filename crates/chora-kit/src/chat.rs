//! The chat model: the entries of a helper's transcript as `agent.read`
//! answers them, the sanitizer for text that an agent wrote, the wrap
//! into whole lines, and the field that holds typed text while a
//! question waits. No drawing and no IPC: the host draws and asks.
//!
//! A host styles a [`Line`] by its [`Role`] and never by its text or
//! its label: a helper may be named like the person, and its text may
//! look like a label or a system line.

use std::collections::VecDeque;

use serde_json::Value;

use crate::{
    ask::{Keys, show_key},
    words::{Language, Word, say},
};

/// The entries that the chat keeps, as many as voice keeps.
const KEPT: usize = 500;
/// The most lines of one entry, the last one an ellipsis if it is cut.
const MAX_ENTRY_LINES: usize = 40;
/// The most combining marks after one base character.
const MAX_MARKS: usize = 2;
/// The label of a helper whose name is empty once sanitized.
const NO_NAME: &str = "?";

/// Whether a character may never reach a text node: C0 except the line
/// break and the tab, DEL, C1, the bidi controls, and the marks, fillers
/// and joiners that hide or reorder text.
fn hidden(c: char) -> bool {
    !matches!(c, '\n' | '\t')
        && matches!(c,
            '\u{0}'..='\u{1f}'
            | '\u{7f}'..='\u{9f}'
            | '\u{ad}'
            | '\u{34f}'
            | '\u{61c}'
            | '\u{115f}'
            | '\u{1160}'
            | '\u{17b4}'
            | '\u{17b5}'
            | '\u{180b}'..='\u{180f}'
            | '\u{200b}'..='\u{200f}'
            | '\u{2028}'..='\u{202e}'
            | '\u{2060}'..='\u{206f}'
            | '\u{2800}'
            | '\u{3164}'
            | '\u{fe00}'..='\u{fe0f}'
            | '\u{feff}'
            | '\u{ffa0}'
            | '\u{fff9}'..='\u{fffc}'
            | '\u{1d173}'..='\u{1d17a}'
            | '\u{e0000}'..='\u{e01ef}'
        )
}

/// A space separator other than the plain space.
fn odd_space(c: char) -> bool {
    matches!(
        c,
        '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{202f}' | '\u{205f}' | '\u{3000}'
    )
}

/// A character of the main combining blocks: it sits on the character
/// before it.
fn is_mark(c: char) -> bool {
    matches!(c,
        '\u{300}'..='\u{36f}'
        | '\u{483}'..='\u{489}'
        | '\u{591}'..='\u{5bd}'
        | '\u{5bf}'
        | '\u{5c1}'..='\u{5c2}'
        | '\u{5c4}'..='\u{5c5}'
        | '\u{5c7}'
        | '\u{610}'..='\u{61a}'
        | '\u{64b}'..='\u{65f}'
        | '\u{670}'
        | '\u{6d6}'..='\u{6dc}'
        | '\u{6df}'..='\u{6e4}'
        | '\u{6e7}'..='\u{6e8}'
        | '\u{6ea}'..='\u{6ed}'
        | '\u{900}'..='\u{903}'
        | '\u{93a}'..='\u{94f}'
        | '\u{1ab0}'..='\u{1aff}'
        | '\u{1dc0}'..='\u{1dff}'
        | '\u{20d0}'..='\u{20ff}'
        | '\u{fe20}'..='\u{fe2f}'
    )
}

/// The text without the characters that an agent could use to run a
/// terminal sequence, to hide text or to reorder it. A space separator
/// becomes a plain space, and a character keeps at most two combining
/// marks. Agent text is untrusted.
#[must_use]
pub fn sanitize(text: &str) -> String {
    let mut clean = String::with_capacity(text.len());
    let mut marks = 0;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            skip_sequence(&mut chars);
            continue;
        }
        if matches!(
            c,
            '\u{9b}' | '\u{9d}' | '\u{90}' | '\u{98}' | '\u{9e}' | '\u{9f}'
        ) {
            skip_body(&mut chars, c);
            continue;
        }
        if hidden(c) {
            continue;
        }
        if is_mark(c) {
            marks += 1;
            if marks <= MAX_MARKS {
                clean.push(c);
            }
            continue;
        }
        marks = 0;
        clean.push(if odd_space(c) { ' ' } else { c });
    }
    clean
}

/// Skips what follows an ESC: a sequence of a terminal, which a text must
/// not leave behind in part.
fn skip_sequence(chars: &mut std::iter::Peekable<std::str::Chars>) {
    let Some(kind) = chars.next() else {
        return;
    };
    match kind {
        '[' | ']' | 'P' | 'X' | '^' | '_' => skip_body(chars, kind),
        '\u{20}'..='\u{2f}' => {
            while chars
                .next_if(|c| ('\u{20}'..='\u{2f}').contains(c))
                .is_some()
            {}
            chars.next();
        }
        _ => {}
    }
}

/// Skips the body of a control sequence (`[`, 0x9b: up to its final byte)
/// or of a string sequence (up to BEL or ST).
fn skip_body(chars: &mut std::iter::Peekable<std::str::Chars>, kind: char) {
    if matches!(kind, '[' | '\u{9b}') {
        for c in chars.by_ref() {
            if ('\u{40}'..='\u{7e}').contains(&c) {
                return;
            }
        }
        return;
    }
    while let Some(c) = chars.next() {
        match c {
            '\u{7}' | '\u{9c}' => return,
            '\u{1b}' if chars.peek() == Some(&'\\') => {
                chars.next();
                return;
            }
            _ => {}
        }
    }
}

/// Who an entry is from, and so how a host styles its lines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// The person.
    User,
    /// The helper.
    Agent,
    /// A tool call of the helper; also a role that is not known, in the
    /// same dim style and without a label.
    Tool,
    /// A question of the helper.
    Question,
    /// The plan of the helper.
    Plan,
    /// Text of the system, never of the helper.
    Notice,
}

impl Role {
    /// The role of the name that `agent.read` gives.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "user" => Self::User,
            "agent" => Self::Agent,
            "question" => Self::Question,
            "plan" => Self::Plan,
            "notice" => Self::Notice,
            _ => Self::Tool,
        }
    }
}

/// An entry of the transcript; the text is sanitized.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The number that voice gave it; none for the line that says
    /// that messages are missing.
    pub seq: Option<u64>,
    /// Who it is from.
    pub role: Role,
    /// The text, without control characters.
    pub text: String,
    shaped: Option<Vec<String>>,
}

/// A line to draw.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    /// The role of its entry; the style follows it.
    pub role: Role,
    /// The name above the first line of an entry: the person's word or
    /// the helper's name. Other roles have none.
    pub label: Option<String>,
    /// The text of one line, no wider than asked.
    pub text: String,
}

/// How the host measures text: the width of a line and of a character.
/// `font` names the font and size; lines are wrapped again when it, the
/// width or the language changes.
pub struct Measure<'a> {
    /// The widest line.
    pub max: f32,
    /// Any number that is the same for the same font and size.
    pub font: u64,
    /// The width of one character.
    pub width: &'a dyn Fn(char) -> f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Shape {
    language: Language,
    max: u32,
    font: u64,
}

/// The transcript of one helper.
#[derive(Debug, Default)]
pub struct Chat {
    entries: VecDeque<Entry>,
    seq: u64,
    epoch: Option<u64>,
    shape: Option<Shape>,
}

impl Chat {
    /// The entries, oldest first.
    #[must_use]
    pub fn entries(&self) -> &VecDeque<Entry> {
        &self.entries
    }

    /// The `after` of the next `agent.read`: the `seq` of the last entry
    /// held.
    #[must_use]
    pub fn after(&self) -> u64 {
        self.seq
    }

    /// Takes the answer of `agent.read`. An entry that is held is not
    /// added again. Entries after a gap come with a line that says
    /// messages are missing. A new `epoch` is another start of the
    /// agent, so the chat starts again; an answer with a `seq` below the
    /// one held is late and changes nothing. The next pull starts after
    /// the last entry of the answer, so an answer that voice cut is
    /// followed by the rest.
    pub fn apply(&mut self, answer: &Value) {
        let epoch = answer["epoch"].as_u64();
        if epoch.is_some() && self.epoch.is_some() && epoch != self.epoch {
            self.entries.clear();
            self.seq = 0;
        }
        if epoch.is_some() {
            self.epoch = epoch;
        }
        let head = answer["seq"].as_u64().unwrap_or(self.seq);
        if head < self.seq {
            return;
        }
        let mut gap_checked = false;
        for entry in answer["entries"].as_array().into_iter().flatten() {
            let (Some(seq), Some(text)) = (entry["seq"].as_u64(), entry["text"].as_str()) else {
                continue;
            };
            if seq <= self.seq {
                continue;
            }
            if !gap_checked && seq > self.seq + 1 {
                self.push(None, Role::Notice, String::new());
            }
            gap_checked = true;
            self.seq = seq;
            let role = Role::from_name(entry["role"].as_str().unwrap_or_default());
            self.push(Some(seq), role, sanitize(text));
        }
        if !gap_checked {
            self.seq = head;
        }
    }

    fn push(&mut self, seq: Option<u64>, role: Role, text: String) {
        self.entries.push_back(Entry {
            seq,
            role,
            text,
            shaped: None,
        });
        while self.entries.len() > KEPT {
            self.entries.pop_front();
        }
    }

    /// The whole transcript as lines no wider than `measure.max`. The
    /// first line of a message of the person or of the helper carries a
    /// label. Wrapped lines are kept per entry until the width, the font
    /// or the language changes.
    pub fn lines(&mut self, language: Language, helper: &str, measure: &Measure) -> Vec<Line> {
        let shape = Shape {
            language,
            max: measure.max.to_bits(),
            font: measure.font,
        };
        if self.shape != Some(shape) {
            self.entries.iter_mut().for_each(|e| e.shaped = None);
            self.shape = Some(shape);
        }
        let helper = Some(sanitize(helper))
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| NO_NAME.to_owned());
        let mut lines = Vec::new();
        for entry in &mut self.entries {
            let label = match entry.role {
                Role::User => Some(say(language, Word::You, &[])),
                Role::Agent => Some(helper.clone()),
                _ => None,
            };
            let role = entry.role;
            let (seq, text) = (entry.seq, &entry.text);
            let shaped = entry.shaped.get_or_insert_with(|| match seq {
                Some(_) => shorten(wrap(text, measure.max, measure.width)),
                None => wrap(
                    &say(language, Word::Dropped, &[]),
                    measure.max,
                    measure.width,
                ),
            });
            for (i, text) in shaped.iter().enumerate() {
                lines.push(Line {
                    role,
                    label: label.clone().filter(|_| i == 0),
                    text: text.clone(),
                });
            }
        }
        lines
    }
}

/// The lines of one entry without blank lines at the ends, with a blank
/// run as one blank line, and at most `MAX_ENTRY_LINES` lines, the last
/// an ellipsis if lines were cut.
fn shorten(lines: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in lines {
        let blank = line.is_empty();
        if blank && out.last().is_none_or(String::is_empty) {
            continue;
        }
        out.push(line);
    }
    if out.last().is_some_and(String::is_empty) {
        out.pop();
    }
    if out.len() > MAX_ENTRY_LINES {
        out.truncate(MAX_ENTRY_LINES - 1);
        out.push("…".to_owned());
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

/// A line that is being filled.
struct Filler<'a> {
    out: Vec<String>,
    line: String,
    used: f32,
    has_word: bool,
    max: f32,
    width: &'a dyn Fn(char) -> f32,
}

impl Filler<'_> {
    fn flush(&mut self) {
        let done = std::mem::take(&mut self.line);
        if self.has_word {
            self.out.push(done.trim_end().to_owned());
        }
        self.used = 0.0;
        self.has_word = false;
    }

    /// A base character with its marks; its width is the base's.
    fn put(&mut self, cluster: &str) {
        let width = cluster.chars().next().map_or(0.0, self.width);
        if self.used + width > self.max && self.has_word {
            self.flush();
        } else if self.used + width > self.max {
            self.line.clear();
            self.used = 0.0;
        }
        self.line.push_str(cluster);
        self.used += width;
        self.has_word = self.has_word || !cluster.starts_with(' ');
    }

    fn word(&mut self, word: &[String], gap: &str) {
        let width = |s: &str| s.chars().next().map_or(0.0, self.width);
        let word_width: f32 = word.iter().map(|c| width(c)).sum();
        let gap_width: f32 = gap.chars().map(self.width).sum();
        if self.has_word && self.used + gap_width + word_width <= self.max {
            self.line.push_str(gap);
            self.used += gap_width;
        } else if self.has_word {
            self.flush();
        }
        for cluster in word {
            self.put(cluster);
        }
    }
}

/// A character with the marks after it.
fn clusters(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for c in text.chars() {
        match out.last_mut() {
            Some(last) if is_mark(c) => last.push(c),
            _ => out.push(c.to_string()),
        }
    }
    out
}

/// Wraps text into lines no wider than `max`, at spaces; a word wider
/// than a line is cut between characters, never between a character and
/// its marks, and a line has at least one character. A line break of
/// the text stays; a tab is four spaces. The indentation of the first
/// line of a paragraph and the spaces inside a line stay; the spaces at
/// a wrap and at the end of a line are dropped.
#[must_use]
pub fn wrap(text: &str, max: f32, width: impl Fn(char) -> f32) -> Vec<String> {
    let mut filler = Filler {
        out: Vec::new(),
        line: String::new(),
        used: 0.0,
        has_word: false,
        max,
        width: &width,
    };
    for paragraph in text.replace('\t', "    ").split('\n') {
        let first = filler.out.len();
        let indent = paragraph.len() - paragraph.trim_start_matches(' ').len();
        filler.line = " ".repeat(indent);
        filler.used = filler.line.chars().map(&width).sum::<f32>().min(max);
        let rest = &paragraph[indent..];
        let mut gap = String::new();
        for token in rest.split_inclusive(' ') {
            let word = token.trim_end_matches(' ');
            let spaces = &token[word.len()..];
            if !word.is_empty() {
                filler.word(&clusters(word), &gap);
            }
            gap = if word.is_empty() {
                gap + spaces
            } else {
                spaces.to_owned()
            };
        }
        filler.flush();
        if filler.out.len() == first {
            filler.out.push(String::new());
        }
    }
    filler.out
}

/// What Enter does in the field.
#[derive(Debug, PartialEq, Eq)]
pub enum Submit {
    /// Send this text; the field is empty.
    Send(String),
    /// A question waits: the text stays as a draft.
    Held,
    /// Nothing to send.
    Empty,
}

/// What Enter did to the text of the field.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Hold {
    #[default]
    Free,
    /// Enter held the text while the question waits.
    Held,
    /// The question ended and the text was not sent.
    Unsent,
}

/// The text field. It answers no question: typed text cannot be told
/// from pasted or injected text, so while a question waits Enter holds
/// the text and the field shows the hint.
#[derive(Debug, Default)]
pub struct Field {
    text: String,
    waiting: bool,
    hold: Hold,
}

impl Field {
    /// The text in the field.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Replaces the text. A change ends the hint that the text was not
    /// sent.
    pub fn set_text(&mut self, text: &str) {
        if text != self.text && self.hold == Hold::Unsent {
            self.hold = Hold::Free;
        }
        text.clone_into(&mut self.text);
    }

    /// Enter.
    pub fn submit(&mut self) -> Submit {
        self.hold = Hold::Free;
        if self.text.trim().is_empty() {
            Submit::Empty
        } else if self.waiting {
            self.hold = Hold::Held;
            Submit::Held
        } else {
            Submit::Send(std::mem::take(&mut self.text))
        }
    }

    /// A question of the helper starts or ends. A new question drops
    /// every earlier Enter. At its end the text that Enter held is not
    /// sent: it stays as a draft with the hint that it was not sent, as
    /// a person who typed during the question may not mean it any more.
    pub fn set_waiting(&mut self, waiting: bool) {
        self.waiting = waiting;
        let before = std::mem::take(&mut self.hold);
        if !waiting && before == Hold::Held {
            self.hold = Hold::Unsent;
        }
    }

    /// The line to show: while Enter holds text, the hint that typing
    /// does not answer, naming the keys of the question; after its
    /// question ended, that the text is not sent.
    #[must_use]
    pub fn hint(&self, language: Language, keys: &Keys) -> Option<String> {
        if self.waiting && self.hold == Hold::Held {
            return Some(say(
                language,
                Word::TypedYes,
                &[("yes", &show_key(&keys.yes)), ("no", &show_key(&keys.no))],
            ));
        }
        (self.hold == Hold::Unsent && !self.text.is_empty())
            .then(|| say(language, Word::NotSent, &[]))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;
    use crate::ask::Keys;
    use crate::words::Language;
    use serde_json::json;

    fn read(entries: &[(u64, &str, &str)], seq: u64) -> serde_json::Value {
        read_in(1, entries, seq)
    }

    fn read_in(epoch: u64, entries: &[(u64, &str, &str)], seq: u64) -> serde_json::Value {
        let entries: Vec<_> = entries
            .iter()
            .map(|(seq, role, text)| json!({"seq": seq, "role": role, "text": text}))
            .collect();
        json!({"entries": entries, "seq": seq, "epoch": epoch})
    }

    fn measure(max: f32) -> Measure<'static> {
        Measure {
            max,
            font: 0,
            width: &|_| 1.0,
        }
    }

    fn chat_with(text: &str) -> Chat {
        let mut chat = Chat::default();
        chat.apply(&read(&[(1, "agent", text)], 1));
        chat
    }

    fn texts(lines: &[Line]) -> Vec<&str> {
        lines.iter().map(|l| l.text.as_str()).collect()
    }

    #[test]
    fn control_characters_and_bidi_overrides_are_dropped() {
        let hostile = "a\u{1b}[31mb\u{1b}]52;c;Zm9v\u{7}c\u{9b}31md\u{202e}e\u{2066}f\u{2069}g\u{200d}h\u{7f}i\u{200b}j\u{feff}k\u{e0041}l";
        let clean = sanitize(hostile);
        assert!(
            clean
                .chars()
                .all(|c| c == '\n' || c == '\t' || !c.is_control()),
            "{clean:?}"
        );
        for gone in [
            '\u{202e}',
            '\u{2066}',
            '\u{2069}',
            '\u{200d}',
            '\u{200b}',
            '\u{feff}',
            '\u{e0041}',
        ] {
            assert!(!clean.contains(gone), "{gone:?} in {clean:?}");
        }
        assert_eq!(sanitize("one\ntwo\tthree"), "one\ntwo\tthree");
        assert_eq!(sanitize("Grüße 日本語"), "Grüße 日本語");
        let chat = chat_with("x\u{1b}]52;c;AAAA\u{7}y");
        assert!(!chat.entries()[0].text.contains(['\u{1b}', '\u{7}']));
    }

    #[test]
    fn escape_sequences_are_dropped_whole() {
        let hostile = "a\u{1b}[31mb\u{1b}]52;c;Zm9v\u{7}c\u{1b}]0;title\u{1b}\\d\u{9b}31me\u{1b}Pq data\u{1b}\\f\u{1b}Mg";
        assert_eq!(sanitize(hostile), "abcdefg");
        assert_eq!(sanitize("x\u{1b}]52;c;AAA"), "x");
    }

    #[test]
    fn invisible_characters_are_dropped_and_spaces_become_spaces() {
        for gone in [
            '\u{ad}',
            '\u{34f}',
            '\u{115f}',
            '\u{1160}',
            '\u{3164}',
            '\u{ffa0}',
            '\u{17b4}',
            '\u{17b5}',
            '\u{180b}',
            '\u{180f}',
            '\u{fe00}',
            '\u{fe0f}',
            '\u{fff9}',
            '\u{fffc}',
            '\u{2800}',
            '\u{1d173}',
            '\u{1d17a}',
        ] {
            assert_eq!(sanitize(&format!("a{gone}b")), "ab", "{gone:?}");
        }
        assert_eq!(sanitize("a\u{a0}b\u{3000}c\u{2003}d\u{202f}e"), "a b c d e");
    }

    #[test]
    fn stacked_combining_marks_are_capped() {
        let zalgo = format!("a{}b{}", "\u{300}".repeat(40), "\u{301}\u{302}\u{303}");
        assert_eq!(sanitize(&zalgo), "a\u{300}\u{300}b\u{301}\u{302}");
        assert_eq!(sanitize("e\u{301}e\u{301}"), "e\u{301}e\u{301}");
    }

    #[test]
    fn pull_after_seq_resumes_after_a_gap() {
        let mut chat = Chat::default();
        assert_eq!(chat.after(), 0);
        chat.apply(&read(&[(1, "user", "hi"), (2, "agent", "hello")], 2));
        assert_eq!(chat.after(), 2);
        chat.apply(&read(&[(2, "agent", "hello"), (3, "tool", "Read a")], 3));
        assert_eq!(chat.entries().len(), 3, "an entry seen twice stays one");
        chat.apply(&read(&[(40, "agent", "late")], 40));
        assert_eq!(chat.after(), 40);
        assert_eq!(chat.entries().back().unwrap().text, "late");
        chat.apply(&read(&[], 40));
        assert_eq!(chat.after(), 40);
        assert_eq!(
            chat.entries().len(),
            5,
            "three, the marker and the late one"
        );
    }

    #[test]
    fn a_cut_answer_resumes_at_its_last_entry() {
        let mut chat = Chat::default();
        chat.apply(&read(&[(1, "agent", "a")], 1));
        chat.apply(&read(&[(2, "agent", "b"), (3, "agent", "c")], 9));
        assert_eq!(chat.after(), 3, "entries 4 to 9 are still to pull");
        chat.apply(&read(&[(4, "agent", "d"), (5, "agent", "e")], 9));
        assert_eq!(chat.after(), 5);
        assert!(chat.entries().iter().all(|e| e.seq.is_some()), "no gap");
    }

    #[test]
    fn a_gap_shows_a_system_line_without_a_seq() {
        let mut chat = Chat::default();
        chat.apply(&read(&[(1, "user", "hi")], 1));
        chat.apply(&read(&[(2, "agent", "next")], 2));
        assert_eq!(chat.entries().len(), 2, "seq 2 follows 1: no gap");
        chat.apply(&read(&[(9, "agent", "later")], 9));
        let marker = &chat.entries()[2];
        assert_eq!((marker.seq, marker.role), (None, Role::Notice));
        let en = chat.lines(Language::En, "Atlas", &measure(80.0));
        let de = chat.lines(Language::De, "Atlas", &measure(80.0));
        assert_eq!(en[2].text, "Earlier messages were dropped.");
        assert_eq!(de[2].text, "Frühere Nachrichten wurden verworfen.");
        assert_eq!((en[2].role, en[2].label.as_deref()), (Role::Notice, None));
        assert_eq!(en[3].text, "later");
    }

    #[test]
    fn a_stale_answer_changes_nothing_and_a_new_epoch_starts_again() {
        let mut chat = Chat::default();
        chat.apply(&read(
            &[(1, "user", "a"), (2, "agent", "b"), (3, "agent", "c")],
            3,
        ));
        chat.apply(&read(&[(1, "user", "a")], 1));
        assert_eq!((chat.after(), chat.entries().len()), (3, 3));
        chat.apply(&read(&[], 2));
        assert_eq!((chat.after(), chat.entries().len()), (3, 3));
        chat.apply(&read_in(2, &[(1, "agent", "fresh")], 1));
        assert_eq!(chat.after(), 1);
        assert_eq!(chat.entries().len(), 1);
        assert_eq!(chat.entries()[0].text, "fresh");
    }

    #[test]
    fn the_chat_keeps_500_entries() {
        let mut chat = Chat::default();
        for seq in 1..=520 {
            chat.apply(&read(&[(seq, "agent", "x")], seq));
        }
        assert_eq!(chat.entries().len(), 500);
        assert_eq!(chat.entries().front().unwrap().seq, Some(21));
        assert_eq!(chat.after(), 520);
    }

    #[test]
    fn a_typed_yes_is_held_while_a_question_waits() {
        let mut field = Field::default();
        field.set_text("yes");
        assert_eq!(field.submit(), Submit::Send("yes".into()));
        field.set_text("yes");
        field.set_waiting(true);
        assert_eq!(field.submit(), Submit::Held);
        assert_eq!(field.text(), "yes", "the field keeps the text");
        let hint = field.hint(Language::En, &Keys::default()).unwrap();
        assert_eq!(
            hint,
            "Typing does not answer. Press Alt+Y or Alt+N, or click Yes or No."
        );
        assert_eq!(field.submit(), Submit::Held);
    }

    #[test]
    fn held_text_stays_a_draft_after_the_question_ends() {
        let mut field = Field::default();
        field.set_waiting(true);
        field.set_text("yes please");
        assert_eq!(field.submit(), Submit::Held);
        field.set_waiting(false);
        assert_eq!(field.text(), "yes please");
        assert_eq!(
            field.hint(Language::En, &Keys::default()).unwrap(),
            "Not sent. Press Enter to send."
        );
        assert_eq!(field.submit(), Submit::Send("yes please".into()));

        field.set_waiting(true);
        field.set_text("draft");
        field.set_waiting(false);
        assert_eq!(field.text(), "draft");
        assert_eq!(field.hint(Language::En, &Keys::default()), None);
    }

    #[test]
    fn an_edit_ends_the_not_sent_hint() {
        let mut field = Field::default();
        field.set_waiting(true);
        field.set_text("old");
        field.submit();
        field.set_waiting(false);
        assert!(field.hint(Language::En, &Keys::default()).is_some());
        field.set_text("old, edited");
        assert_eq!(field.hint(Language::En, &Keys::default()), None);
    }

    #[test]
    fn a_new_question_drops_the_old_hold() {
        let mut field = Field::default();
        field.set_waiting(true);
        field.set_text("yes");
        field.submit();
        field.set_waiting(true);
        assert_eq!(field.hint(Language::En, &Keys::default()), None);
        field.set_waiting(false);
        assert_eq!(field.hint(Language::En, &Keys::default()), None);
        assert_eq!(field.text(), "yes");
    }

    #[test]
    fn entries_wrap_to_whole_lines() {
        let one = |_: char| 1.0;
        assert_eq!(
            wrap("hello brave new world", 11.0, one),
            ["hello brave", "new world"]
        );
        assert_eq!(wrap("a\n\nb", 5.0, one), ["a", "", "b"]);
        assert_eq!(wrap("abcdefghij", 4.0, one), ["abcd", "efgh", "ij"]);
        assert_eq!(wrap("", 4.0, one), [""]);
        assert_eq!(wrap("ab", 0.0, one), ["a", "b"]);
        assert_eq!(wrap("    let x = 1;", 20.0, one), ["    let x = 1;"]);
        assert_eq!(wrap("a  b", 20.0, one), ["a  b"]);
        for text in [
            "the quick brown fox jumps over the lazy dog",
            "abcdefghijklmnopqrstuvwxyz and then some",
            "  indented\n\n  more text here, long enough to wrap",
        ] {
            assert_fits_and_keeps(text, 9);
        }
    }

    #[test]
    fn an_entry_wraps_with_its_label_on_the_first_line() {
        let lines = chat_with("hello brave world").lines(Language::En, "Atlas", &measure(8.0));
        assert_eq!(texts(&lines), ["hello", "brave", "world"]);
        assert_eq!(lines[0].label.as_deref(), Some("Atlas"));
        assert_eq!(lines[1].label, None);
    }

    fn assert_fits_and_keeps(text: &str, max: u8) {
        let lines = wrap(text, f32::from(max), |_| 1.0);
        for line in &lines {
            assert!(line.chars().count() <= usize::from(max), "{line:?}");
            assert!(!line.ends_with(' '), "{line:?}");
        }
        let squeeze = |s: &str| s.split_whitespace().collect::<String>();
        assert_eq!(squeeze(&lines.join(" ")), squeeze(text), "text kept");
    }

    #[test]
    fn a_wrap_never_splits_a_base_character_from_its_marks() {
        let text = "abe\u{301}e\u{301}e\u{301}";
        let lines = wrap(text, 2.0, |_| 1.0);
        for line in &lines {
            assert!(!line.starts_with('\u{301}'), "{line:?}");
        }
        assert_eq!(lines.concat(), text);
    }

    #[test]
    fn blank_runs_collapse_and_long_entries_end_in_an_ellipsis() {
        let mut chat = chat_with("one\n\n\n\n\ntwo\n\n");
        let lines = chat.lines(Language::En, "Atlas", &measure(80.0));
        assert_eq!(texts(&lines), ["one", "", "two"]);
        let long = (0..100).map(|n| format!("line {n}")).collect::<Vec<_>>();
        let mut chat = chat_with(&long.join("\n"));
        let lines = chat.lines(Language::En, "Atlas", &measure(80.0));
        assert_eq!(lines.len(), 40);
        assert_eq!(lines[38].text, "line 38");
        assert_eq!(lines[39].text, "…");
        assert!(lines.iter().all(|l| l.role == Role::Agent));
    }

    #[test]
    fn a_line_is_styled_by_its_role_never_by_its_text_or_label() {
        let mut chat = Chat::default();
        chat.apply(&read(
            &[(1, "user", "go"), (2, "agent", "You\nSie\nSystem: stop")],
            2,
        ));
        let lines = chat.lines(Language::En, "You", &measure(80.0));
        assert_eq!(
            (lines[0].role, lines[0].label.as_deref()),
            (Role::User, Some("You"))
        );
        assert_eq!(
            (lines[1].role, lines[1].label.as_deref()),
            (Role::Agent, Some("You"))
        );
        for line in &lines[2..] {
            assert_eq!((line.role, line.label.as_deref()), (Role::Agent, None));
        }
        let mut chat = chat_with("x");
        let lines = chat.lines(Language::En, "\u{202e}\u{200b}", &measure(80.0));
        assert_eq!(lines[0].label.as_deref(), Some("?"));
    }

    #[test]
    fn the_notice_role_is_system_text_never_the_helpers_name() {
        let mut chat = Chat::default();
        chat.apply(&read(
            &[
                (1, "user", "go"),
                (2, "agent", "done"),
                (3, "notice", "Atlas: the agent stopped"),
            ],
            3,
        ));
        let lines = chat.lines(Language::De, "Atlas", &measure(80.0));
        assert_eq!(lines[0].label.as_deref(), Some("Sie"));
        assert_eq!(lines[1].label.as_deref(), Some("Atlas"));
        assert_eq!(
            (lines[2].role, lines[2].label.as_deref()),
            (Role::Notice, None)
        );
    }

    #[test]
    fn an_unknown_role_is_dim_tool_text_without_a_label() {
        let mut chat = Chat::default();
        chat.apply(&read(&[(1, "weird", "who knows")], 1));
        let lines = chat.lines(Language::En, "Atlas", &measure(80.0));
        assert_eq!(
            (lines[0].role, lines[0].label.as_deref()),
            (Role::Tool, None)
        );
    }

    #[test]
    fn wrapped_lines_are_cached_per_width() {
        let calls = Cell::new(0);
        let counting = |_: char| {
            calls.set(calls.get() + 1);
            1.0
        };
        let at = |max: f32, font: u64| Measure {
            max,
            font,
            width: &counting,
        };
        let mut chat = chat_with("some words to wrap");
        let first = chat.lines(Language::En, "Atlas", &at(8.0, 1));
        let made = calls.get();
        assert!(made > 0);
        assert_eq!(chat.lines(Language::En, "Atlas", &at(8.0, 1)), first);
        assert_eq!(calls.get(), made, "the second call measures nothing");
        chat.lines(Language::En, "Atlas", &at(9.0, 1));
        assert!(calls.get() > made, "another width wraps again");
        let made = calls.get();
        chat.lines(Language::En, "Atlas", &at(9.0, 2));
        assert!(calls.get() > made, "another font wraps again");
        let made = calls.get();
        chat.apply(&read(&[(2, "agent", "new")], 2));
        chat.lines(Language::En, "Atlas", &at(9.0, 2));
        assert!(
            calls.get() > made && calls.get() < made + 20,
            "only the new entry"
        );
    }
}
