//! The draft of the field and what each key does to it. No drawing and no
//! IPC: the window loop feeds keys in and carries the effects out.
//!
//! The field answers no question. It cannot tell typed text from pasted
//! or injected text, so while a question waits [`Session`] sends nothing,
//! and text that Enter held stays a draft after the question ended.

use chora_kit::{
    ask::Keys,
    chat::{Field, Submit, sanitize},
    words::{Language, Word, say},
};

/// The most bytes of a draft.
const MAX_DRAFT: usize = 8192;

/// The most lines of a draft that Enter sends at once; a longer draft
/// needs a second Enter.
pub const MAX_LINES: usize = 4;

/// A key of the field, as the window loop reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Key {
    /// Typed or pasted text.
    Text(String),
    /// Shift+Enter.
    Newline,
    /// Enter.
    Enter,
    /// Esc.
    Escape,
    /// Backspace.
    Backspace,
    /// Delete.
    Delete,
    /// Left.
    Left,
    /// Right.
    Right,
    /// Home.
    Home,
    /// End.
    End,
}

/// What the window loop does after a key.
#[derive(Debug, PartialEq, Eq)]
pub enum Effect {
    /// Send this text to the helper with `agent_say`.
    Say(String),
    /// Close the field and keep the draft.
    Close,
}

/// The draft of the field.
#[derive(Debug, Default)]
pub struct Session {
    field: Field,
    cursor: usize,
    preedit: String,
    keys: Keys,
    language: Language,
    /// Why the field gave the last text back: voice is offline, or no
    /// helper was picked.
    given_back: Option<Word>,
    waiting: bool,
    confirm: bool,
}

impl Session {
    /// A draft that speaks `language`.
    #[must_use]
    pub fn new(language: Language) -> Self {
        Self {
            language,
            ..Self::default()
        }
    }

    /// The text of the draft.
    #[must_use]
    pub fn text(&self) -> &str {
        self.field.text()
    }

    /// The text before the cursor, with the preedit of the input method.
    #[must_use]
    pub fn before_cursor(&self) -> String {
        format!("{}{}", &self.text()[..self.cursor], self.preedit)
    }

    /// The text after the cursor.
    #[must_use]
    pub fn after_cursor(&self) -> &str {
        &self.text()[self.cursor..]
    }

    /// The byte offset of the cursor in [`Self::text`].
    #[must_use]
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// A key.
    pub fn key(&mut self, key: Key) -> Option<Effect> {
        self.given_back = None;
        if key != Key::Enter {
            self.confirm = false;
        }
        match key {
            Key::Text(text) => self.insert(&text),
            Key::Newline => self.insert("\n"),
            Key::Enter => return self.enter(),
            Key::Escape => return Some(Effect::Close),
            Key::Backspace => self.backspace(),
            Key::Delete => self.delete(),
            Key::Left => self.cursor = self.previous(),
            Key::Right => self.cursor = self.next(),
            Key::Home => self.cursor = self.line_start(),
            Key::End => self.cursor = self.line_end(),
        }
        None
    }

    /// Text that the input method commits; it ends the preedit.
    pub fn commit(&mut self, text: &str) {
        self.confirm = false;
        self.preedit.clear();
        self.insert(text);
    }

    /// The text that the input method composes.
    pub fn set_preedit(&mut self, text: &str) {
        self.preedit = sanitize(text).replace('\n', " ");
    }

    /// The input method deletes bytes around the cursor.
    pub fn delete_surrounding(&mut self, before: usize, after: usize) {
        self.confirm = false;
        let text = self.text();
        let start = floor(text, self.cursor.saturating_sub(before));
        let end = floor(text, (self.cursor + after).min(text.len()));
        let kept = format!("{}{}", &text[..start], &text[end..]);
        self.field.set_text(&kept);
        self.cursor = start;
    }

    /// The keys that answer the question, for the hint.
    pub fn set_keys(&mut self, keys: Keys) {
        self.keys = keys;
    }

    /// A question of the helper starts or ends. Text that Enter held
    /// stays a draft.
    pub fn set_waiting(&mut self, waiting: bool) {
        self.waiting = waiting;
        self.confirm = false;
        self.field.set_waiting(waiting);
    }

    /// The text did not reach the helper: the draft gets it back.
    pub fn say_failed(&mut self, text: &str) {
        if self.text().is_empty() {
            self.field.set_text(text);
            self.cursor = text.len();
        }
        self.given_back = Some(Word::Offline);
    }

    /// The chat goes to another helper: the draft and its line go, so no
    /// text meant for one helper reaches another.
    pub fn clear(&mut self) {
        self.field.set_text("");
        self.cursor = 0;
        self.preedit.clear();
        self.given_back = None;
        self.confirm = false;
    }

    /// The text was typed before a helper was picked: the draft gets it
    /// back, and the line under it says to pick first.
    pub fn pick_first(&mut self, text: &str) {
        if self.text().is_empty() {
            self.field.set_text(text);
            self.cursor = text.len();
        }
        self.given_back = Some(Word::PickFirst);
    }

    /// The line under the text: what the field holds back and why.
    #[must_use]
    pub fn status(&self) -> Option<String> {
        if let Some(hint) = self.field.hint(self.language, &self.keys) {
            return Some(hint);
        }
        if let Some(word) = self.given_back {
            return Some(say(self.language, word, &[]));
        }
        let lines = self.lines().to_string();
        if self.confirm {
            return Some(say(self.language, Word::SendLines, &[("n", &lines)]));
        }
        if self.lines() > MAX_LINES {
            return Some(say(self.language, Word::Lines, &[("n", &lines)]));
        }
        self.waiting.then(|| say(self.language, Word::Waiting, &[]))
    }

    /// The lines of the draft, by its line breaks.
    fn lines(&self) -> usize {
        self.text().matches('\n').count() + 1
    }

    /// The line in an empty field.
    #[must_use]
    pub fn placeholder(&self) -> Option<String> {
        (self.text().is_empty() && self.preedit.is_empty())
            .then(|| say(self.language, Word::FieldHint, &[]))
    }

    fn enter(&mut self) -> Option<Effect> {
        if self.lines() > MAX_LINES && !self.confirm && !self.text().trim().is_empty() {
            self.confirm = true;
            return None;
        }
        match self.field.submit() {
            Submit::Send(text) => {
                self.cursor = 0;
                Some(Effect::Say(text))
            }
            Submit::Held | Submit::Empty => None,
        }
    }

    fn insert(&mut self, text: &str) {
        let clean = sanitize(text).replace('\t', " ");
        let room = MAX_DRAFT.saturating_sub(self.text().len());
        let clean = &clean[..floor(&clean, room)];
        let mut next = self.text().to_owned();
        next.insert_str(self.cursor, clean);
        self.field.set_text(&next);
        self.cursor += clean.len();
    }

    fn backspace(&mut self) {
        let start = self.previous();
        let mut next = self.text().to_owned();
        next.replace_range(start..self.cursor, "");
        self.field.set_text(&next);
        self.cursor = start;
    }

    fn delete(&mut self) {
        let end = self.next();
        let mut next = self.text().to_owned();
        next.replace_range(self.cursor..end, "");
        self.field.set_text(&next);
    }

    fn previous(&self) -> usize {
        self.text()[..self.cursor]
            .char_indices()
            .next_back()
            .map_or(0, |(i, _)| i)
    }

    fn next(&self) -> usize {
        self.text()[self.cursor..]
            .chars()
            .next()
            .map_or(self.cursor, |c| self.cursor + c.len_utf8())
    }

    fn line_start(&self) -> usize {
        self.text()[..self.cursor].rfind('\n').map_or(0, |i| i + 1)
    }

    fn line_end(&self) -> usize {
        self.text()[self.cursor..]
            .find('\n')
            .map_or(self.text().len(), |i| self.cursor + i)
    }
}

/// The largest char boundary of `text` at or below `index`.
fn floor(text: &str, index: usize) -> usize {
    (0..=index.min(text.len()))
        .rev()
        .find(|&i| text.is_char_boundary(i))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn type_text(session: &mut Session, text: &str) -> Vec<Effect> {
        text.chars()
            .filter_map(|c| session.key(Key::Text(c.to_string())))
            .collect()
    }

    #[test]
    fn a_typed_yes_answers_nothing() {
        let mut session = Session::new(Language::En);
        session.set_waiting(true);
        let mut effects = type_text(&mut session, "yes");
        effects.extend(session.key(Key::Enter));
        effects.extend(session.key(Key::Enter));
        assert_eq!(effects, []);
        assert_eq!(session.text(), "yes");
        let hint = session.status().expect("the typing hint");
        assert!(hint.contains("Alt+Y") && hint.contains("Alt+N"), "{hint}");
    }

    #[test]
    fn held_text_stays_a_draft_after_the_question_ends() {
        let mut session = Session::new(Language::En);
        session.set_waiting(true);
        type_text(&mut session, "yes");
        session.key(Key::Enter);
        session.set_waiting(false);
        assert_eq!(session.text(), "yes");
        assert!(session.status().expect("a hint").starts_with("Not sent"));
        assert_eq!(session.key(Key::Enter), Some(Effect::Say("yes".into())));
    }

    #[test]
    fn a_long_paste_needs_a_second_enter() {
        let mut session = Session::new(Language::En);
        session.key(Key::Text("a\nb\nc\nd\ne\nf".into()));
        assert_eq!(session.status().as_deref(), Some("6 lines"));
        assert_eq!(session.key(Key::Enter), None);
        assert_eq!(
            session.status().as_deref(),
            Some("Press Enter again to send 6 lines")
        );
        assert!(matches!(session.key(Key::Enter), Some(Effect::Say(_))));
    }

    #[test]
    fn an_edit_cancels_the_second_enter() {
        let mut session = Session::new(Language::En);
        session.key(Key::Text("a\nb\nc\nd\ne".into()));
        session.key(Key::Enter);
        session.key(Key::Left);
        assert_eq!(session.key(Key::Enter), None);
    }

    #[test]
    fn enter_sends_and_empties_the_field() {
        let mut session = Session::new(Language::En);
        type_text(&mut session, "hello");
        assert_eq!(session.key(Key::Enter), Some(Effect::Say("hello".into())));
        assert_eq!(session.text(), "");
        assert_eq!(session.key(Key::Enter), None);
    }

    #[test]
    fn shift_enter_breaks_a_line() {
        let mut session = Session::new(Language::En);
        type_text(&mut session, "a");
        session.key(Key::Newline);
        type_text(&mut session, "b");
        assert_eq!(session.text(), "a\nb");
    }

    #[test]
    fn escape_closes_and_keeps_the_draft() {
        let mut session = Session::new(Language::En);
        type_text(&mut session, "draft");
        assert_eq!(session.key(Key::Escape), Some(Effect::Close));
        assert_eq!(session.text(), "draft");
    }

    #[test]
    fn the_cursor_moves_and_edits_whole_characters() {
        let mut session = Session::new(Language::De);
        type_text(&mut session, "äöü");
        session.key(Key::Left);
        session.key(Key::Backspace);
        assert_eq!(session.text(), "äü");
        session.key(Key::Home);
        session.key(Key::Delete);
        assert_eq!(session.text(), "ü");
        session.key(Key::End);
        type_text(&mut session, "x");
        assert_eq!(session.text(), "üx");
        session.key(Key::Right);
        assert_eq!(session.cursor(), 3);
    }

    #[test]
    fn a_commit_and_a_paste_arrive_without_controls() {
        let mut session = Session::new(Language::En);
        session.set_preedit("ka");
        assert_eq!(session.before_cursor(), "ka");
        session.commit("かな");
        session.key(Key::Text("\u{1b}]52;c;x\u{7}ok\ttab".into()));
        assert_eq!(session.text(), "かなok tab");
        assert_eq!(session.before_cursor(), session.text());
    }

    #[test]
    fn the_input_method_deletes_around_the_cursor() {
        let mut session = Session::new(Language::En);
        type_text(&mut session, "abcd");
        session.key(Key::Left);
        session.delete_surrounding(1, 1);
        assert_eq!(session.text(), "ab");
        assert_eq!(session.cursor(), 2);
    }

    #[test]
    fn clear_empties_the_draft_for_another_helper() {
        let mut session = Session::new(Language::En);
        type_text(&mut session, "for atlas");
        session.pick_first("x");
        session.clear();
        assert_eq!(session.text(), "");
        assert_eq!(session.cursor(), 0);
        assert_eq!(session.status(), None);
    }

    #[test]
    fn a_line_before_a_pick_asks_to_pick_a_helper_first() {
        let mut session = Session::new(Language::En);
        type_text(&mut session, "Hallo");
        session.key(Key::Enter);
        session.pick_first("Hallo");
        assert_eq!(session.text(), "Hallo");
        let status = session.status().unwrap_or_default();
        assert_eq!(status, say(Language::En, Word::PickFirst, &[]));
        assert!(!status.contains("voice control"), "{status}");
        session.key(Key::Text("!".into()));
        assert_ne!(session.status(), Some(status));
    }

    #[test]
    fn a_failed_send_gives_the_text_back() {
        let mut session = Session::new(Language::En);
        type_text(&mut session, "hello");
        session.key(Key::Enter);
        session.say_failed("hello");
        assert_eq!(session.text(), "hello");
        assert!(session.status().is_some());
    }
}
