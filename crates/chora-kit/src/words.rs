//! The words of the screen: one table in English and German, chosen by
//! the language setting. A HUD folder or a look never changes a word.

/// The language of the words.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    /// English.
    En,
    /// German.
    De,
}

impl Language {
    /// The language of a tag such as `de`, `de_DE.UTF-8` or `en-US`;
    /// every language but German gets English.
    #[must_use]
    pub fn from_tag(tag: &str) -> Self {
        let language = tag.split(['_', '-', '.', '@']).next().unwrap_or_default();
        if language.eq_ignore_ascii_case("de") {
            Self::De
        } else {
            Self::En
        }
    }
}

/// A word of the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    /// The word `Asleep`; see [`TABLE`].
    Asleep,
    /// The word `Listening`; see [`TABLE`].
    Listening,
    /// The word `Thinking`; see [`TABLE`].
    Thinking,
    /// The word `Working`; see [`TABLE`].
    Working,
    /// The word `Speaking`; see [`TABLE`].
    Speaking,
    /// The word `Question`; see [`TABLE`].
    Question,
    /// The word `VoiceOff`; see [`TABLE`].
    VoiceOff,
    /// The word `Muted`; see [`TABLE`].
    Muted,
    /// The word `MicOn`; see [`TABLE`].
    MicOn,
    /// The word `MicError`; see [`TABLE`].
    MicError,
    /// The word `ChatRow`; see [`TABLE`].
    ChatRow,
    /// The word `ChatLauncher`; see [`TABLE`].
    ChatLauncher,
    /// The word `Helpers`; see [`TABLE`].
    Helpers,
    /// The word `HelperOne`; see [`TABLE`].
    HelperOne,
    /// The word `Stop`; see [`TABLE`].
    Stop,
    /// The word `Close`; see [`TABLE`].
    Close,
    /// The word `You`; see [`TABLE`].
    You,
    /// The word `CardTitle`; see [`TABLE`].
    CardTitle,
    /// The word `KeyLine`; see [`TABLE`].
    KeyLine,
    /// The word `NoVoiceYes`; see [`TABLE`].
    NoVoiceYes,
    /// The word `Yes`; see [`TABLE`].
    Yes,
    /// The word `No`; see [`TABLE`].
    No,
    /// The word `Allowed`; see [`TABLE`].
    Allowed,
    /// The word `Refused`; see [`TABLE`].
    Refused,
    /// The word `NoAnswer`; see [`TABLE`].
    NoAnswer,
    /// The word `Injected`; see [`TABLE`].
    Injected,
    /// The word `TypedYes`; see [`TABLE`].
    TypedYes,
    /// The word `Waiting`; see [`TABLE`].
    Waiting,
    /// The word `FieldHint`; see [`TABLE`].
    FieldHint,
    /// The word `Offline`; see [`TABLE`].
    Offline,
    /// The word `CloseNote`; see [`TABLE`].
    CloseNote,
    /// The word `NoHelper`; see [`TABLE`].
    NoHelper,
    /// The word `HudNotice`; see [`TABLE`].
    HudNotice,
}

/// The last variant, for the table check.
pub const LAST: Word = Word::HudNotice;

/// One row per word, in the order of [`Word`]: English, German. `{name}`
/// style fields are filled by [`say`].
pub const TABLE: &[(Word, &str, &str)] = &[
    (
        Word::Asleep,
        "Asleep. Say \"Hey Chora\".",
        "Schläft. Sagen Sie „Hallo Chora“.",
    ),
    (Word::Listening, "Listening", "Hört zu"),
    (Word::Thinking, "Thinking", "Denkt nach"),
    (Word::Working, "Working", "Arbeitet"),
    (Word::Speaking, "Speaking", "Spricht"),
    (Word::Question, "Question for you", "Frage an Sie"),
    (Word::VoiceOff, "Voice off", "Sprache aus"),
    (
        Word::Muted,
        "Microphone off. Click to turn it on.",
        "Mikrofon aus. Zum Einschalten klicken.",
    ),
    (Word::MicOn, "Microphone on.", "Mikrofon an."),
    (
        Word::MicError,
        "I could not switch the microphone.",
        "Ich konnte das Mikrofon nicht schalten.",
    ),
    (Word::ChatRow, "Chat ...", "Chat ..."),
    (
        Word::ChatLauncher,
        "Chat with a helper",
        "Mit einem Helfer chatten",
    ),
    (Word::Helpers, "{n} helpers working", "{n} Helfer arbeiten"),
    (Word::HelperOne, "1 helper working", "1 Helfer arbeitet"),
    (Word::Stop, "Stop", "Anhalten"),
    (Word::Close, "Close", "Schließen"),
    (Word::You, "You", "Sie"),
    (Word::CardTitle, "Question from {name}", "Frage von {name}"),
    (
        Word::KeyLine,
        "Press {yes} to allow or {no} to refuse.",
        "{yes} zum Erlauben, {no} zum Ablehnen drücken.",
    ),
    (
        Word::NoVoiceYes,
        "A spoken \"yes\" does not count.",
        "Ein gesprochenes „Ja“ zählt hier nicht.",
    ),
    (Word::Yes, "Yes", "Ja"),
    (Word::No, "No", "Nein"),
    (Word::Allowed, "Allowed.", "Erlaubt."),
    (Word::Refused, "Refused.", "Abgelehnt."),
    (
        Word::NoAnswer,
        "No answer in time. Refused.",
        "Keine Antwort rechtzeitig. Abgelehnt.",
    ),
    (
        Word::Injected,
        "Please use your own mouse or keyboard.",
        "Bitte mit eigener Maus oder Tastatur antworten.",
    ),
    (
        Word::TypedYes,
        "Typing does not answer. Press {yes} or {no}, or click Yes or No.",
        "Tippen beantwortet die Frage nicht. {yes} oder {no} drücken oder Ja oder Nein klicken.",
    ),
    (
        Word::Waiting,
        "A question is waiting.",
        "Eine Frage wartet.",
    ),
    (
        Word::FieldHint,
        "Type a message, then Enter.",
        "Nachricht eingeben, dann Enter.",
    ),
    (
        Word::Offline,
        "Cannot reach chora-voice. Trying again every 5 s.",
        "chora-voice nicht erreichbar. Neuer Versuch alle 5 s.",
    ),
    (
        Word::CloseNote,
        "{name} goes on working.",
        "{name} arbeitet weiter.",
    ),
    (
        Word::NoHelper,
        "No helper is set up. Ask the person who set up this computer.",
        "Kein Helfer eingerichtet. Fragen Sie, wer den Computer eingerichtet hat.",
    ),
    (
        Word::HudNotice,
        "Look \"{name}\" has {n} errors; the old look stays. Run: chora-assistant --check-hud {name}",
        "Aussehen „{name}“ hat {n} Fehler; das alte bleibt. Prüfen: chora-assistant --check-hud {name}",
    ),
];

impl Word {
    /// The text in a language, with its `{fields}` unfilled.
    #[must_use]
    pub fn text(self, language: Language) -> &'static str {
        let (_, en, de) = TABLE[self as usize];
        match language {
            Language::En => en,
            Language::De => de,
        }
    }
}

/// The word with its `{fields}` filled.
#[must_use]
pub fn say(language: Language, word: Word, fields: &[(&str, &str)]) -> String {
    crate::hud::expand(word.text(language), |name| {
        fields
            .iter()
            .find(|(field, _)| *field == name)
            .map(|(_, value)| (*value).to_owned())
    })
}

/// The word of a `voice.state` value such as `working`, if it is one.
#[must_use]
pub fn state(name: &str) -> Option<Word> {
    Some(match name {
        "asleep" => Word::Asleep,
        "listening" => Word::Listening,
        "thinking" => Word::Thinking,
        "working" => Word::Working,
        "question" => Word::Question,
        "speaking" => Word::Speaking,
        _ => return None,
    })
}

/// A row of the agents list: `Atlas · working · Write file (2/2)`.
#[must_use]
pub fn row(language: Language, name: &str, state_name: &str, task: &str) -> String {
    let state_text = state(state_name).map_or_else(
        || state_name.to_owned(),
        |word| word.text(language).to_lowercase(),
    );
    format!("{name} · {state_text} · {task}")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    fn fields(text: &str) -> BTreeSet<&str> {
        text.split('{')
            .skip(1)
            .filter_map(|rest| rest.split_once('}').map(|(name, _)| name))
            .collect()
    }

    fn avoids_jargon(text: &str) -> bool {
        !text.contains("HUD") && !text.contains("ACP") && !text.to_lowercase().contains("agent")
    }

    #[test]
    fn every_word_has_en_and_de() {
        assert_eq!(TABLE.len(), LAST as usize + 1, "a word without a row");
        for (index, (word, en, de)) in TABLE.iter().enumerate() {
            assert_eq!(*word as usize, index, "{word:?} is out of order");
            assert!(!en.is_empty() && !de.is_empty(), "{word:?}");
            assert_eq!(fields(en), fields(de), "{word:?}: fields differ");
            assert_eq!(word.text(Language::En), *en);
            assert_eq!(word.text(Language::De), *de);
            assert!(avoids_jargon(en) && avoids_jargon(de), "{word:?}");
        }
    }

    #[test]
    fn the_language_follows_the_tag() {
        assert_eq!(Language::from_tag("de"), Language::De);
        assert_eq!(Language::from_tag("de_DE.UTF-8"), Language::De);
        assert_eq!(Language::from_tag("de-AT"), Language::De);
        assert_eq!(Language::from_tag("en_US.UTF-8"), Language::En);
        assert_eq!(Language::from_tag("fr"), Language::En);
        assert_eq!(Language::from_tag(""), Language::En);
    }

    #[test]
    fn fields_are_filled_once() {
        let en = say(Language::En, Word::CardTitle, &[("name", "Atlas")]);
        assert_eq!(en, "Question from Atlas");
        let de = say(Language::De, Word::Helpers, &[("n", "2")]);
        assert_eq!(de, "2 Helfer arbeiten");
        let echo = say(Language::En, Word::CardTitle, &[("name", "{name}")]);
        assert_eq!(echo, "Question from {name}");
        assert_eq!(
            row(Language::De, "Atlas", "working", "Datei schreiben (2/2)"),
            "Atlas · arbeitet · Datei schreiben (2/2)"
        );
        assert_eq!(state("question"), Some(Word::Question));
        assert_eq!(state("nonsense"), None);
    }
}
