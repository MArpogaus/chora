//! The words of the screen: one `match` per language, English and
//! German, chosen by the language setting. A HUD folder or a look never changes a word.

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

/// A word of the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    /// The voice is asleep and waits for its name.
    Asleep,
    /// The voice listens.
    Listening,
    /// The voice thinks.
    Thinking,
    /// A turn runs.
    Working,
    /// The voice speaks.
    Speaking,
    /// The title of a question without a helper.
    Question,
    /// A helper that waits for work.
    Idle,
    /// A helper that finished.
    Done,
    /// A helper that cannot go on.
    Blocked,
    /// The voice is switched off.
    VoiceOff,
    /// The badge of a muted microphone.
    Muted,
    /// The microphone was switched on.
    MicOn,
    /// The microphone did not switch.
    MicError,
    /// The row that opens the chat.
    ChatRow,
    /// The name of the chat launcher.
    ChatLauncher,
    /// Several helpers work.
    Helpers,
    /// One helper works.
    HelperOne,
    /// The button that stops a turn.
    Stop,
    /// The button that closes the chat.
    Close,
    /// The name of the person in the chat.
    You,
    /// The title of a question of a helper.
    CardTitle,
    /// The keys that answer a question.
    KeyLine,
    /// A spoken yes does not answer.
    NoVoiceYes,
    /// The left button.
    Yes,
    /// The right button.
    No,
    /// The question was allowed.
    Allowed,
    /// The question was refused.
    Refused,
    /// The question timed out.
    NoAnswer,
    /// A press that a program made.
    Injected,
    /// The detail of a question does not fit on the card.
    TooLong,
    /// Typed text does not answer.
    TypedYes,
    /// A question waits.
    Waiting,
    /// The hint of the input field.
    FieldHint,
    /// The voice control does not answer.
    Offline,
    /// Closing the chat does not stop a helper.
    CloseNote,
    /// No helper exists.
    NoHelper,
    /// Messages of the transcript are missing.
    Dropped,
    /// Typed text waits and was changed after Enter.
    NotSent,
    /// The draft has more lines than show.
    Lines,
    /// Enter again sends a draft of many lines.
    SendLines,
    /// A look with one problem.
    HudNoticeOne,
    /// A look with several problems.
    HudNotice,
    /// The control that steps back from a face of a cube.
    StepBack,
    /// The control that steps into a face of a cube.
    StepIn,
    /// The control that turns a cube to the left.
    TurnLeft,
    /// The control that turns a cube to the right.
    TurnRight,
    /// The control that opens a cube.
    OpenCube,
}

/// Every word, for the checks of the tests.
pub const ALL: [Word; 47] = [
    Word::Asleep,
    Word::Listening,
    Word::Thinking,
    Word::Working,
    Word::Speaking,
    Word::Question,
    Word::Idle,
    Word::Done,
    Word::Blocked,
    Word::VoiceOff,
    Word::Muted,
    Word::MicOn,
    Word::MicError,
    Word::ChatRow,
    Word::ChatLauncher,
    Word::Helpers,
    Word::HelperOne,
    Word::Stop,
    Word::Close,
    Word::You,
    Word::CardTitle,
    Word::KeyLine,
    Word::NoVoiceYes,
    Word::Yes,
    Word::No,
    Word::Allowed,
    Word::Refused,
    Word::NoAnswer,
    Word::Injected,
    Word::TooLong,
    Word::TypedYes,
    Word::Waiting,
    Word::FieldHint,
    Word::Offline,
    Word::CloseNote,
    Word::NoHelper,
    Word::Dropped,
    Word::NotSent,
    Word::Lines,
    Word::SendLines,
    Word::HudNoticeOne,
    Word::HudNotice,
    Word::StepBack,
    Word::StepIn,
    Word::TurnLeft,
    Word::TurnRight,
    Word::OpenCube,
];

impl Word {
    /// The text in a language, with its `{fields}` unfilled.
    #[must_use]
    pub fn text(self, language: Language) -> &'static str {
        match language {
            Language::En => self.en(),
            Language::De => self.de(),
        }
    }

    fn en(self) -> &'static str {
        match self {
            Self::Asleep => "Asleep. Say \"Hey Chora\".",
            Self::Listening => "Listening",
            Self::Thinking => "Thinking",
            Self::Working => "Working",
            Self::Speaking => "Speaking",
            Self::Question => "Question for you",
            Self::Idle => "Idle",
            Self::Done => "Done",
            Self::Blocked => "Blocked",
            Self::VoiceOff => "Voice off",
            Self::Muted => "Microphone off. Click to turn it on.",
            Self::MicOn => "Microphone on.",
            Self::MicError => "I could not switch the microphone.",
            Self::ChatRow => "Chat ...",
            Self::ChatLauncher => "Chat with a helper",
            Self::Helpers => "{n} helpers working",
            Self::HelperOne => "1 helper working",
            Self::Stop => "Stop",
            Self::Close => "Close",
            Self::You => "You",
            Self::CardTitle => "Question from {name}",
            Self::KeyLine => "Press {yes} to allow or {no} to refuse.",
            Self::NoVoiceYes => "A spoken \"yes\" does not count.",
            Self::Yes => "Yes",
            Self::No => "No",
            Self::Allowed => "Allowed.",
            Self::Refused => "Refused.",
            Self::NoAnswer => "No answer in time. Refused.",
            Self::Injected => "Please use your own mouse or keyboard.",
            Self::TooLong => "Too long to show. Refused.",
            Self::TypedYes => "Typing does not answer. Press {yes} or {no}, or click Yes or No.",
            Self::Waiting => "A question is waiting.",
            Self::FieldHint => "Type a message, then Enter.",
            Self::Offline => "Cannot reach voice control. Trying again every 5 s.",
            Self::CloseNote => "{name} goes on working.",
            Self::NoHelper => "No helper is set up. Ask the person who set up this computer.",
            Self::Dropped => "Earlier messages were dropped.",
            Self::NotSent => "Not sent. Press Enter to send.",
            Self::Lines => "{n} lines",
            Self::SendLines => "Press Enter again to send {n} lines",
            Self::HudNoticeOne => {
                "The look \"{name}\" has 1 problem, so the old look stays. Choose another look in the settings, or ask who made it."
            }
            Self::HudNotice => {
                "The look \"{name}\" has {n} problems, so the old look stays. Choose another look in the settings, or ask who made it."
            }
            Self::StepBack => "Step back",
            Self::StepIn => "Step in",
            Self::TurnLeft => "Turn left",
            Self::TurnRight => "Turn right",
            Self::OpenCube => "Open cube",
        }
    }

    fn de(self) -> &'static str {
        match self {
            Self::Asleep => "Schläft. Sagen Sie „Hallo Chora“.",
            Self::Listening => "Hört zu",
            Self::Thinking => "Denkt nach",
            Self::Working => "Arbeitet",
            Self::Speaking => "Spricht",
            Self::Question => "Frage an Sie",
            Self::Idle => "Bereit",
            Self::Done => "Fertig",
            Self::Blocked => "Blockiert",
            Self::VoiceOff => "Sprache aus",
            Self::Muted => "Mikrofon aus. Zum Einschalten klicken.",
            Self::MicOn => "Mikrofon an.",
            Self::MicError => "Ich konnte das Mikrofon nicht schalten.",
            Self::ChatRow => "Chat ...",
            Self::ChatLauncher => "Mit einem Helfer chatten",
            Self::Helpers => "{n} Helfer arbeiten",
            Self::HelperOne => "1 Helfer arbeitet",
            Self::Stop => "Anhalten",
            Self::Close => "Schließen",
            Self::You => "Sie",
            Self::CardTitle => "Frage von {name}",
            Self::KeyLine => "{yes} zum Erlauben, {no} zum Ablehnen drücken.",
            Self::NoVoiceYes => "Ein gesprochenes „Ja“ zählt hier nicht.",
            Self::Yes => "Ja",
            Self::No => "Nein",
            Self::Allowed => "Erlaubt.",
            Self::Refused => "Abgelehnt.",
            Self::NoAnswer => "Keine Antwort rechtzeitig. Abgelehnt.",
            Self::Injected => "Bitte mit eigener Maus oder Tastatur antworten.",
            Self::TooLong => "Zu lang zum Anzeigen. Abgelehnt.",
            Self::TypedYes => {
                "Tippen beantwortet die Frage nicht. {yes} oder {no} drücken oder Ja oder Nein klicken."
            }
            Self::Waiting => "Eine Frage wartet.",
            Self::FieldHint => "Nachricht eingeben, dann Enter.",
            Self::Offline => "Sprachsteuerung nicht erreichbar. Neuer Versuch alle 5 s.",
            Self::CloseNote => "{name} arbeitet weiter.",
            Self::NoHelper => {
                "Kein Helfer eingerichtet. Fragen Sie, wer den Computer eingerichtet hat."
            }
            Self::Dropped => "Frühere Nachrichten wurden verworfen.",
            Self::NotSent => "Nicht gesendet. Zum Senden Enter drücken.",
            Self::Lines => "{n} Zeilen",
            Self::SendLines => "Zum Senden von {n} Zeilen noch einmal Enter drücken",
            Self::HudNoticeOne => {
                "Das Aussehen „{name}“ hat 1 Fehler, das alte bleibt. Wählen Sie in den Einstellungen ein anderes oder fragen Sie, wer es gemacht hat."
            }
            Self::HudNotice => {
                "Das Aussehen „{name}“ hat {n} Fehler, das alte bleibt. Wählen Sie in den Einstellungen ein anderes oder fragen Sie, wer es gemacht hat."
            }
            Self::StepBack => "Zurücktreten",
            Self::StepIn => "Hineingehen",
            Self::TurnLeft => "Nach links drehen",
            Self::TurnRight => "Nach rechts drehen",
            Self::OpenCube => "Würfel öffnen",
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

/// The word of a state such as `working`, if it is one: the states of
/// `voice.state` and of a helper.
#[must_use]
pub fn state(name: &str) -> Option<Word> {
    Some(match name {
        "asleep" => Word::Asleep,
        "listening" => Word::Listening,
        "thinking" => Word::Thinking,
        "working" => Word::Working,
        "question" => Word::Question,
        "speaking" => Word::Speaking,
        "idle" => Word::Idle,
        "done" => Word::Done,
        "blocked" => Word::Blocked,
        _ => return None,
    })
}

/// A row of the agents list: `Atlas · working · Write file (2/2)`; no
/// task, no last part.
#[must_use]
pub fn row(language: Language, name: &str, state_name: &str, task: &str) -> String {
    let state_text = state(state_name).map_or_else(
        || state_name.to_owned(),
        |word| word.text(language).to_lowercase(),
    );
    if task.is_empty() {
        format!("{name} · {state_text}")
    } else {
        format!("{name} · {state_text} · {task}")
    }
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
        ["chora-", "Look", "HUD", "ACP"]
            .iter()
            .all(|word| !text.contains(word))
            && !text.to_lowercase().contains("agent")
    }

    fn has_command(text: &str) -> bool {
        text.contains("--") || text.contains("chora") && text.contains(':')
    }

    #[test]
    fn every_word_has_en_and_de() {
        for word in ALL {
            let (en, de) = (word.text(Language::En), word.text(Language::De));
            assert!(!en.is_empty() && !de.is_empty(), "{word:?}");
            assert_eq!(fields(en), fields(de), "{word:?}: fields differ");
            assert!(avoids_jargon(en) && avoids_jargon(de), "{word:?}");
            assert!(!has_command(en) && !has_command(de), "{word:?}: a command");
        }
    }

    #[test]
    fn the_notice_counts_in_the_singular() {
        for language in [Language::En, Language::De] {
            let one = say(language, Word::HudNoticeOne, &[("name", "fancy")]);
            let many = say(language, Word::HudNotice, &[("name", "fancy"), ("n", "3")]);
            assert!(one.contains("fancy") && many.contains("fancy") && many.contains('3'));
            assert!(!one.contains("errors"), "{one}");
            assert!(one.contains("1 "), "{one}");
        }
        let one = say(Language::En, Word::HudNoticeOne, &[("name", "x")]);
        assert!(one.contains("1 problem"), "{one}");
    }

    #[test]
    fn helper_states_have_words() {
        for name in [
            "asleep",
            "listening",
            "thinking",
            "working",
            "question",
            "speaking",
            "idle",
            "done",
            "blocked",
        ] {
            assert!(state(name).is_some(), "{name}");
        }
        assert_eq!(row(Language::En, "Atlas", "idle", "x"), "Atlas · idle · x");
        assert_eq!(
            row(Language::De, "Atlas", "done", "x"),
            "Atlas · fertig · x"
        );
        assert_eq!(
            row(Language::En, "Atlas", "blocked", "x"),
            "Atlas · blocked · x"
        );
    }

    #[test]
    fn a_row_without_a_task_has_no_tail() {
        assert_eq!(row(Language::En, "Atlas", "working", ""), "Atlas · working");
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
