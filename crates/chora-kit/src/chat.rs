//! The chat model: the entries of a helper's transcript as `agent.read`
//! answers them, the sanitizer for text that an agent wrote, the wrap
//! into whole lines, and the field that holds typed text while a
//! question waits. No drawing and no IPC: the host draws and asks.

use std::collections::VecDeque;

use serde_json::Value;

use crate::{
    ask::{Keys, show_key},
    words::{Language, Word, say},
};

/// The entries that the chat keeps, as many as voice keeps.
const KEPT: usize = 500;

/// Whether a character may never reach a text node: C0 except the line
/// break and the tab, DEL, C1, the bidi controls and the marks and
/// joiners that hide or reorder text.
fn hidden(c: char) -> bool {
    match c {
        '\n' | '\t' => false,
        '\u{0}'..='\u{1f}'
        | '\u{7f}'..='\u{9f}'
        | '\u{61c}'
        | '\u{200b}'..='\u{200f}'
        | '\u{2028}'..='\u{202e}'
        | '\u{2060}'..='\u{206f}'
        | '\u{feff}'
        | '\u{e0000}'..='\u{e01ef}' => true,
        _ => false,
    }
}

/// The text without the characters that an agent could use to run a
/// terminal sequence, to hide text or to reorder it. Agent text is
/// untrusted.
#[must_use]
pub fn sanitize(text: &str) -> String {
    text.chars().filter(|c| !hidden(*c)).collect()
}

/// Who an entry is from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// The person.
    User,
    /// The helper.
    Agent,
    /// A tool call of the helper.
    Tool,
    /// A question of the helper.
    Question,
    /// The plan of the helper.
    Plan,
    /// Text of the system, never of the helper. A role that is not
    /// known is one too.
    Notice,
}

impl Role {
    /// The role of the name that `agent.read` gives.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "user" => Self::User,
            "agent" => Self::Agent,
            "tool" => Self::Tool,
            "question" => Self::Question,
            "plan" => Self::Plan,
            _ => Self::Notice,
        }
    }
}

/// An entry of the transcript; the text is sanitized.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The number that voice gave it.
    pub seq: u64,
    /// Who it is from.
    pub role: Role,
    /// The text, without control characters.
    pub text: String,
}

/// A line to draw.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    /// The role of its entry.
    pub role: Role,
    /// The name above the first line of an entry: the person's word or
    /// the helper's name. Other roles have none.
    pub label: Option<String>,
    /// The text of one line, no wider than asked.
    pub text: String,
}

/// The transcript of one helper.
#[derive(Debug, Default)]
pub struct Chat {
    entries: VecDeque<Entry>,
    seq: u64,
}

impl Chat {
    /// The entries, oldest first.
    #[must_use]
    pub fn entries(&self) -> &VecDeque<Entry> {
        &self.entries
    }

    /// The `after` of the next `agent.read`: the last `seq` held.
    #[must_use]
    pub fn after(&self) -> u64 {
        self.seq
    }

    /// Takes the answer of `agent.read`. An entry that is held is not
    /// added again; entries after a gap are added as they are. An answer
    /// whose `seq` is below the one held is of an agent that started
    /// again, so the chat starts again with it.
    pub fn apply(&mut self, answer: &Value) {
        let last = answer["seq"].as_u64().unwrap_or(self.seq);
        if last < self.seq {
            self.entries.clear();
            self.seq = 0;
        }
        for entry in answer["entries"].as_array().into_iter().flatten() {
            let (Some(seq), Some(text)) = (entry["seq"].as_u64(), entry["text"].as_str()) else {
                continue;
            };
            if seq <= self.seq {
                continue;
            }
            self.seq = seq;
            self.entries.push_back(Entry {
                seq,
                role: Role::from_name(entry["role"].as_str().unwrap_or_default()),
                text: sanitize(text),
            });
        }
        self.seq = self.seq.max(last);
        while self.entries.len() > KEPT {
            self.entries.pop_front();
        }
    }

    /// The whole transcript as lines no wider than `max`, measured with
    /// `width` per character. The first line of a message of the person
    /// or of the helper carries a label.
    #[must_use]
    pub fn lines(
        &self,
        language: Language,
        helper: &str,
        max: f32,
        width: impl Fn(char) -> f32,
    ) -> Vec<Line> {
        let mut lines = Vec::new();
        for entry in &self.entries {
            let label = match entry.role {
                Role::User => Some(say(language, Word::You, &[])),
                Role::Agent => Some(sanitize(helper)),
                _ => None,
            };
            for (i, text) in wrap(&entry.text, max, &width).into_iter().enumerate() {
                lines.push(Line {
                    role: entry.role,
                    label: label.clone().filter(|_| i == 0),
                    text,
                });
            }
        }
        lines
    }
}

/// Wraps text into lines no wider than `max`, at spaces; a word wider
/// than a line is cut between characters, and a line has at least one
/// character. A line break of the text stays; a tab is four spaces;
/// spaces at the start or end of a line are dropped.
#[must_use]
pub fn wrap(text: &str, max: f32, width: impl Fn(char) -> f32) -> Vec<String> {
    let measure = |s: &str| s.chars().map(&width).sum::<f32>();
    let mut out = Vec::new();
    for paragraph in text.replace('\t', "    ").split('\n') {
        let first = out.len();
        let mut line = String::new();
        let mut used = 0.0;
        for word in paragraph.split(' ').filter(|w| !w.is_empty()) {
            let word_width = measure(word);
            let gap = if line.is_empty() { 0.0 } else { width(' ') };
            if !line.is_empty() && used + gap + word_width <= max {
                line.push(' ');
                line.push_str(word);
                used += gap + word_width;
                continue;
            }
            if !line.is_empty() {
                out.push(std::mem::take(&mut line));
            }
            used = 0.0;
            for c in word.chars() {
                let c_width = width(c);
                if used + c_width > max && !line.is_empty() {
                    out.push(std::mem::take(&mut line));
                    used = 0.0;
                }
                line.push(c);
                used += c_width;
            }
        }
        if !line.is_empty() || out.len() == first {
            out.push(line);
        }
    }
    out
}

/// What Enter does in the field.
#[derive(Debug, PartialEq, Eq)]
pub enum Submit {
    /// Send this text; the field is empty.
    Send(String),
    /// A question waits: the text stays and goes after it ends.
    Held,
    /// Nothing to send.
    Empty,
}

/// The text field. It answers no question: typed text cannot be told
/// from pasted or injected text, so while a question waits Enter holds
/// the text and the field shows the hint.
#[derive(Debug, Default)]
pub struct Field {
    text: String,
    waiting: bool,
    held: bool,
}

impl Field {
    /// The text in the field.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Replaces the text; a text that was held is no longer held.
    pub fn set_text(&mut self, text: &str) {
        if text != self.text {
            self.held = false;
        }
        text.clone_into(&mut self.text);
    }

    /// Enter.
    pub fn submit(&mut self) -> Submit {
        if self.text.trim().is_empty() {
            Submit::Empty
        } else if self.waiting {
            self.held = true;
            Submit::Held
        } else {
            Submit::Send(std::mem::take(&mut self.text))
        }
    }

    /// A question of the helper starts or ends. At its end the text
    /// that Enter held is answered as the text to send, and the field
    /// is empty; text without an Enter stays as a draft.
    pub fn set_waiting(&mut self, waiting: bool) -> Option<String> {
        self.waiting = waiting;
        if waiting || !std::mem::take(&mut self.held) {
            return None;
        }
        Some(std::mem::take(&mut self.text))
    }

    /// The line to show while Enter holds text, naming the keys of the
    /// question.
    #[must_use]
    pub fn hint(&self, language: Language, keys: &Keys) -> Option<String> {
        (self.waiting && self.held).then(|| {
            say(
                language,
                Word::TypedYes,
                &[("yes", &show_key(&keys.yes)), ("no", &show_key(&keys.no))],
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::words::Language;
    use serde_json::json;

    fn read(entries: &[(u64, &str, &str)], seq: u64) -> serde_json::Value {
        let entries: Vec<_> = entries
            .iter()
            .map(|(seq, role, text)| json!({"seq": seq, "role": role, "text": text}))
            .collect();
        json!({"entries": entries, "seq": seq})
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
        for hidden in [
            '\u{202e}',
            '\u{2066}',
            '\u{2069}',
            '\u{200d}',
            '\u{200b}',
            '\u{feff}',
            '\u{e0041}',
        ] {
            assert!(!clean.contains(hidden), "{hidden:?} in {clean:?}");
        }
        assert_eq!(sanitize("one\ntwo\tthree"), "one\ntwo\tthree");
        assert_eq!(sanitize("Grüße 日本語"), "Grüße 日本語");
        let mut chat = Chat::default();
        chat.apply(&read(&[(1, "agent", "x\u{1b}]52;c;AAAA\u{7}y")], 1));
        assert!(!chat.entries()[0].text.contains(['\u{1b}', '\u{7}']));
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
        assert_eq!(chat.entries().len(), 4);
        chat.apply(&read(&[(1, "agent", "new agent")], 1));
        assert_eq!(chat.after(), 1, "a restarted agent starts the chat again");
        assert_eq!(chat.entries().len(), 1);
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
        let hint = field
            .hint(Language::En, &crate::ask::Keys::default())
            .unwrap();
        assert_eq!(
            hint,
            "Typing does not answer. Press Alt+Y or Alt+N, or click Yes or No."
        );
        assert_eq!(field.submit(), Submit::Held);
    }

    #[test]
    fn held_text_is_sent_after_the_question_ends() {
        let mut field = Field::default();
        field.set_waiting(true);
        field.set_text("yes please");
        assert_eq!(field.submit(), Submit::Held);
        assert_eq!(field.set_waiting(false), Some("yes please".into()));
        assert_eq!(field.text(), "");
        assert_eq!(field.hint(Language::En, &crate::ask::Keys::default()), None);

        field.set_waiting(true);
        field.set_text("draft");
        assert_eq!(field.set_waiting(false), None, "no Enter, no send");
        assert_eq!(field.text(), "draft");

        field.set_waiting(true);
        field.set_text("old");
        field.submit();
        field.set_text("old, edited");
        assert_eq!(field.set_waiting(false), None, "an edit cancels the hold");
    }

    fn assert_fits(text: &str, max: u8) {
        for line in wrap(text, f32::from(max), |_| 1.0) {
            assert!(line.chars().count() <= usize::from(max), "{line:?}");
            assert!(!line.starts_with(' ') && !line.ends_with(' '), "{line:?}");
        }
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
        assert_fits("the quick brown fox jumps over the lazy dog", 9);
        let mut chat = Chat::default();
        chat.apply(&read(&[(1, "agent", "hello brave world")], 1));
        let lines = chat.lines(Language::En, "Atlas", 8.0, one);
        let texts: Vec<_> = lines.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(texts, ["hello", "brave", "world"]);
        assert_eq!(lines[0].label.as_deref(), Some("Atlas"));
        assert_eq!(lines[1].label, None);
    }

    #[test]
    fn the_notice_role_is_system_text_never_the_helpers_name() {
        let mut chat = Chat::default();
        chat.apply(&read(
            &[
                (1, "user", "go"),
                (2, "agent", "done"),
                (3, "notice", "Atlas: the agent stopped"),
                (4, "weird", "unknown roles are system text too"),
            ],
            4,
        ));
        let lines = chat.lines(Language::De, "Atlas", 80.0, |_| 1.0);
        assert_eq!(lines[0].label.as_deref(), Some("Sie"));
        assert_eq!(lines[1].label.as_deref(), Some("Atlas"));
        for line in &lines[2..] {
            assert_eq!(line.role, Role::Notice);
            assert_eq!(line.label, None);
        }
    }
}
