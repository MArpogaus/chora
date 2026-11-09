//! The question card: the model of what the host draws for
//! `voice.confirm`. The parts and their order, the key line, the place
//! that is fixed when the card appears, and the guards of a press: an
//! injected press and a click 400 ms after the card appeared or moved
//! answer nothing.

use std::time::{Duration, Instant};

use serde_json::Value;

use crate::words::{Language, Word, say};

/// How long after it appeared or moved a button takes no click.
pub const GUARD: Duration = Duration::from_millis(400);

/// The keys that answer a physical question.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keys {
    /// The key that allows.
    pub yes: String,
    /// The key that refuses.
    pub no: String,
}

impl Default for Keys {
    fn default() -> Self {
        Self {
            yes: "Alt+y".into(),
            no: "Alt+n".into(),
        }
    }
}

/// A question of `voice.confirm`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Question {
    /// The id that `confirm.answer` takes.
    pub id: u64,
    /// The spoken prompt.
    pub prompt: String,
    /// The full command or value, empty if there is none.
    pub detail: String,
    /// Only a key or a click answers it, never the voice.
    pub physical: bool,
    /// The keys named by the event.
    pub keys: Keys,
    /// The helper that asks, if the question is its.
    pub agent: Option<String>,
}

impl Question {
    /// The question of a `voice.confirm` event, `None` when it is the
    /// end of a question (`done`) or has no id or prompt.
    #[must_use]
    pub fn from_event(data: &Value, agent: Option<&str>) -> Option<Self> {
        if data["done"] == true {
            return None;
        }
        let text = |key: &str| data[key].as_str().map(str::to_owned);
        let keys = Keys {
            yes: text_of(&data["keys"]["yes"]).unwrap_or_else(|| Keys::default().yes),
            no: text_of(&data["keys"]["no"]).unwrap_or_else(|| Keys::default().no),
        };
        Some(Self {
            id: data["id"].as_u64()?,
            prompt: text("prompt")?,
            detail: text("detail").unwrap_or_default(),
            physical: data["physical"] == true,
            keys,
            agent: agent.map(str::to_owned),
        })
    }
}

fn text_of(value: &Value) -> Option<String> {
    value.as_str().map(str::to_owned)
}

/// A key as a person reads it: `Alt+y` is `Alt+Y`.
#[must_use]
pub fn show_key(key: &str) -> String {
    match key.rsplit_once('+') {
        Some((modifiers, last)) if last.chars().count() == 1 => {
            format!("{modifiers}+{}", last.to_uppercase())
        }
        None if key.chars().count() == 1 => key.to_uppercase(),
        _ => key.to_owned(),
    }
}

/// The line that names the keys of the event, never fixed keys.
#[must_use]
pub fn key_line(language: Language, keys: &Keys) -> String {
    say(
        language,
        Word::KeyLine,
        &[("yes", &show_key(&keys.yes)), ("no", &show_key(&keys.no))],
    )
}

/// Where the card appeared; it never changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    /// Inside the open chat view.
    Chat,
    /// At the figure.
    Figure,
}

/// A part of the card, top to bottom; the buttons come after the last.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// The helper's name in a title, or the generic title.
    Title,
    /// What is asked.
    Prompt,
    /// The full command or value.
    Detail,
    /// The keys that answer.
    KeyLine,
    /// The note that a spoken yes does not count.
    NoVoiceYes,
}

/// A button, Yes left and No right.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    /// The left button.
    Yes,
    /// The right button.
    No,
}

/// What a press on a button does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Press {
    /// The answer for `confirm.answer`.
    Answer(bool),
    /// Too soon after the card appeared or moved; nothing happens.
    TooSoon,
    /// An injected press; the card shows the line `Injected` and stays.
    Injected,
}

/// A card on the screen.
#[derive(Clone, Debug)]
pub struct Card {
    /// The question that the card shows.
    pub question: Question,
    place: Place,
    since: Instant,
}

impl Card {
    /// A card that appears now, in the chat view when the view is open.
    #[must_use]
    pub fn new(question: Question, chat_open: bool, now: Instant) -> Self {
        Self {
            question,
            place: if chat_open {
                Place::Chat
            } else {
                Place::Figure
            },
            since: now,
        }
    }

    /// Where the card appeared.
    #[must_use]
    pub fn place(&self) -> Place {
        self.place
    }

    /// The card was drawn at another place: its buttons take no click
    /// for [`GUARD`].
    pub fn moved(&mut self, now: Instant) {
        self.since = now;
    }

    /// What a press does at `now`; `injected` is the flag of the event.
    #[must_use]
    pub fn press(&self, button: Button, injected: bool, now: Instant) -> Press {
        if injected {
            Press::Injected
        } else if now.saturating_duration_since(self.since) < GUARD {
            Press::TooSoon
        } else {
            Press::Answer(button == Button::Yes)
        }
    }

    /// The texts top to bottom. A question that the voice may answer has
    /// neither the key line nor the line about the spoken yes.
    #[must_use]
    pub fn parts(&self, language: Language) -> Vec<(Part, String)> {
        let question = &self.question;
        let title = match &question.agent {
            Some(name) => say(language, Word::CardTitle, &[("name", name)]),
            None => Word::Question.text(language).to_owned(),
        };
        let mut parts = vec![
            (Part::Title, title),
            (Part::Prompt, question.prompt.clone()),
        ];
        if !question.detail.is_empty() {
            parts.push((Part::Detail, question.detail.clone()));
        }
        if question.physical {
            parts.push((Part::KeyLine, key_line(language, &question.keys)));
            parts.push((Part::NoVoiceYes, Word::NoVoiceYes.text(language).to_owned()));
        }
        parts
    }

    /// The labels of the buttons, Yes first.
    #[must_use]
    pub fn buttons(language: Language) -> [&'static str; 2] {
        [Word::Yes.text(language), Word::No.text(language)]
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn question(data: &Value) -> Question {
        Question::from_event(data, Some("Atlas")).unwrap()
    }

    fn physical(yes: &str, no: &str) -> Question {
        question(&json!({
            "id": 7, "prompt": "Write notes.txt?", "detail": "~/notes.txt (412 bytes)",
            "physical": true, "keys": {"yes": yes, "no": no},
        }))
    }

    #[test]
    fn the_key_line_names_the_keys_of_the_event() {
        let now = Instant::now();
        let card = Card::new(physical("Super+1", "Super+2"), false, now);
        let line = |language| {
            let parts = card.parts(language);
            parts
                .iter()
                .find(|(p, _)| *p == Part::KeyLine)
                .unwrap()
                .1
                .clone()
        };
        let en = line(Language::En);
        assert!(en.contains("Super+1") && en.contains("Super+2"), "{en}");
        assert!(!en.contains("Alt"), "{en}");
        let de = Card::new(physical("Alt+j", "Alt+n"), false, now)
            .parts(Language::De)
            .into_iter()
            .find(|(p, _)| *p == Part::KeyLine)
            .unwrap()
            .1;
        assert_eq!(de, "Alt+J zum Erlauben, Alt+N zum Ablehnen drücken.");
        assert_eq!(
            key_line(Language::En, &Keys::default()),
            "Press Alt+Y to allow or Alt+N to refuse."
        );
        let normal = question(&json!({"id": 8, "prompt": "Read it?"}));
        let parts = Card::new(normal, false, now).parts(Language::En);
        assert!(
            parts
                .iter()
                .all(|(p, _)| !matches!(p, Part::KeyLine | Part::NoVoiceYes))
        );
    }

    #[test]
    fn the_parts_come_in_one_order() {
        let card = Card::new(physical("Alt+y", "Alt+n"), false, Instant::now());
        let order: Vec<Part> = card.parts(Language::En).iter().map(|(p, _)| *p).collect();
        assert_eq!(
            order,
            [
                Part::Title,
                Part::Prompt,
                Part::Detail,
                Part::KeyLine,
                Part::NoVoiceYes
            ]
        );
        assert_eq!(card.parts(Language::En)[0].1, "Question from Atlas");
        assert_eq!(Card::buttons(Language::De), ["Ja", "Nein"]);
        let bare = question(&json!({"id": 1, "prompt": "Sure?"}));
        let parts = Card::new(bare, false, Instant::now()).parts(Language::En);
        assert_eq!(parts.len(), 2);
    }

    #[test]
    fn a_button_counts_only_400_ms_after_it_moved() {
        let start = Instant::now();
        let ms = Duration::from_millis;
        let mut card = Card::new(physical("Alt+y", "Alt+n"), false, start);
        assert_eq!(
            card.press(Button::Yes, false, start + ms(399)),
            Press::TooSoon
        );
        assert_eq!(
            card.press(Button::Yes, false, start + ms(400)),
            Press::Answer(true)
        );
        assert_eq!(
            card.press(Button::No, false, start + ms(900)),
            Press::Answer(false)
        );
        card.moved(start + ms(1000));
        assert_eq!(
            card.press(Button::Yes, false, start + ms(1399)),
            Press::TooSoon
        );
        assert_eq!(
            card.press(Button::Yes, false, start + ms(1400)),
            Press::Answer(true)
        );
    }

    #[test]
    fn an_injected_press_answers_nothing() {
        let start = Instant::now();
        let card = Card::new(physical("Alt+y", "Alt+n"), false, start);
        let later = start + Duration::from_secs(5);
        assert_eq!(card.press(Button::Yes, true, later), Press::Injected);
        assert_eq!(card.press(Button::Yes, true, start), Press::Injected);
    }

    #[test]
    fn a_card_stays_where_it_appeared() {
        let now = Instant::now();
        let mut card = Card::new(physical("Alt+y", "Alt+n"), true, now);
        assert_eq!(card.place(), Place::Chat);
        card.moved(now);
        assert_eq!(card.place(), Place::Chat);
        let other = Card::new(physical("Alt+y", "Alt+n"), false, now);
        assert_eq!(other.place(), Place::Figure);
    }

    #[test]
    fn events_without_a_question_give_none() {
        assert!(Question::from_event(&json!({"id": 1, "done": true}), None).is_none());
        assert!(Question::from_event(&json!({"prompt": "x"}), None).is_none());
        assert!(Question::from_event(&json!({"id": 1}), None).is_none());
        let anon = Question::from_event(&json!({"id": 1, "prompt": "x"}), None).unwrap();
        let parts = Card::new(anon, false, Instant::now()).parts(Language::En);
        assert_eq!(parts[0].1, "Question for you");
    }
}
