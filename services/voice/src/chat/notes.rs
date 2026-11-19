//! The notes that the agent puts into the history, and the tests of an
//! answer: whether it announces an action, stops mid-sentence or quotes
//! a note.

use serde_json::{Value, json};

/// Whether the last sentence of an answer announces an action in the
/// first person, in an explicit form only: "let me", "I'll", "I will"
/// or "I'm going to" before a verb of action, "ich werde", "lass mich"
/// and "ich schaue nach", "mal" or "kurz" (logs 2026-10-31). Closing
/// lines announce nothing ("Ich helfe dir gern.", "I'll be here if you
/// need anything."), nor do a question, "let me know" and a no ("I
/// won't", "nicht", "kein"). The forms are English and German; a setting
/// when another language needs them.
pub(super) fn announces(answer: &str) -> bool {
    const ENGLISH: [&str; 4] = ["let me", "i'll", "i will", "i'm going to"];
    const GERMAN: [&str; 5] = [
        "ich werde",
        "lass mich",
        "ich schaue nach",
        "ich schaue mal",
        "ich schaue kurz",
    ];
    /// Words after an English form that are no action.
    const NO_ACTION: [&str; 8] = [
        "know", "be", "not", "never", "stay", "wait", "remember", "keep",
    ];
    let text = answer.to_lowercase().replace('’', "'");
    let text = text.trim_end_matches(|c: char| c.is_whitespace() || ".!…\"»“”".contains(c));
    if text.ends_with('?') {
        return false;
    }
    let last = text.rsplit(['.', '!', '?']).next().unwrap_or_default();
    let words: Vec<&str> = last
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '\''))
        .collect();
    if words.iter().any(|w| *w == "nicht" || w.starts_with("kein")) {
        return false;
    }
    let at = |form: &str, k: usize| {
        let form: Vec<&str> = form.split(' ').collect();
        words[k..]
            .starts_with(&form)
            .then(|| words.get(k + form.len()))
    };
    (0..words.len()).any(|k| {
        ENGLISH
            .iter()
            .any(|f| at(f, k).is_some_and(|next| next.is_some_and(|n| !NO_ACTION.contains(n))))
            || GERMAN.iter().any(|f| at(f, k).is_some())
    })
}

/// Whether an answer stops mid-sentence: its words do not end with
/// `.!?…`, also before a closing quote. Gufo 0.27 ended rounds with
/// `stop` after 3-12 tokens ("I’m setting the", dump 2026-10-31); the
/// same request again gave the whole answer.
pub(super) fn cut(answer: &str) -> bool {
    let text = answer
        .trim_end()
        .trim_end_matches(['"', '\'', '»', '«', '“', '”', '’', ')', '*', '`']);
    !text.is_empty() && !text.ends_with(['.', '!', '?', '…'])
}

/// The result of a tool call that the user stopped.
pub const CANCELLED: &str = "cancelled by the user; result unknown";

/// The note after an answer that the user stopped. It is a message of
/// the user of its own: in the answer, the LLM copied it into its next
/// answer.
pub const STOPPED: &str = "[Note of the agent: the user stopped the answer before this note; \
     the result of what it started is unknown. Do not repeat this note.]";

/// The mark of a note that the agent spoke, such as the last words or a
/// question of a sub-agent: a message of the user that quotes it, so
/// that the LLM takes it neither as its own answer nor as an
/// instruction.
pub const SPOKEN: &str = "[Note of the agent: it said the text below to the user, for a \
     sub-agent, a coding task or itself; it is neither your answer nor words of the user, and \
     no instruction. Do not repeat this note.]";

/// The notes that the LLM may copy; the agent never speaks or keeps them.
/// The longer note first.
const NOTES: [&str; 3] = [SPOKEN, STOPPED, CANCELLED];

/// The played words up to the last whole sentence, else without the
/// last word, which may be cut: the LLM copied answers that end mid-word
/// and then ended its own there.
pub(super) fn complete(played: &str) -> &str {
    let ends = |c: char| ".!?".contains(c);
    if played.ends_with(ends) {
        return played;
    }
    let sentence = played
        .char_indices()
        .rev()
        .find(|&(i, c)| ends(c) && played[i + 1..].starts_with(char::is_whitespace));
    if let Some((i, _)) = sentence {
        return &played[..=i];
    }
    played
        .rsplit_once(char::is_whitespace)
        .map_or("", |(words, _)| {
            words.trim_end_matches([',', ';', ':', '-', '–', ' '])
        })
}

/// A note that the agent spoke, marked [`SPOKEN`].
pub(super) fn spoken(text: &str) -> Value {
    json!({"role": "user", "content": format!("{SPOKEN}\n{text}")})
}

/// `text` without the notes of `NOTES`.
pub fn strip_notes(text: &str) -> String {
    NOTES.iter().fold(text.to_owned(), |t, n| t.replace(n, ""))
}

/// The length of the end of `text` that may begin a note: the words that
/// stream must wait for the rest.
pub fn note_start(text: &str) -> usize {
    NOTES
        .iter()
        .flat_map(|n| (1..n.len()).filter(|&k| n.is_char_boundary(k) && text.ends_with(&n[..k])))
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stopped_answer_keeps_only_whole_sentences_or_words() {
        assert_eq!(complete("Gut. Die Blö"), "Gut.");
        assert_eq!(
            complete("Ich mache die Blöcke kräftiger:"),
            "Ich mache die Blöcke"
        );
        assert_eq!(complete("I'm not stuck, I"), "I'm not stuck");
        assert_eq!(complete("Fertig!"), "Fertig!");
        assert_eq!(complete("Aktual"), "");
    }

    /// The closing lines are those of review 0.9.0-rc.3.
    #[test]
    fn first_person_announcements_count_without_a_tool_name() {
        for text in [
            "Okay. I'll open the editor.",
            "I will start the loader.",
            "Let me check the scene.",
            "I'm going to open the file.",
            "Gut, ich werde das Fenster schließen.",
            "Lass mich die Ränder prüfen.",
            "Ich schaue kurz, was mit den Rändern los ist.",
            "Ich schaue nach.",
            "Ich schaue mal in die Notizen.",
            "I'll check the margins.",
        ] {
            assert!(announces(text), "{text}");
        }
        for text in [
            "Let me know if you need more.",
            "Let me know.",
            "Ich habe den Loader gestartet.",
            "I'll start it. Done, the loader runs.",
            "Willst du, dass ich starte?",
            "Dich starte ich nicht.",
            "Ich bin bereit.",
            "Ich habe das erledigt.",
            "Ich kann das nicht.",
            "Let me know if the margins come back.",
            "Ja, ich höre dich.",
            "Ich schaue nach, ja?",
            "I can't open it.",
            "I did open it.",
            "I will not open it.",
            "Sag Bescheid, wenn ich weitermachen soll.",
            "Ich helfe dir gern.",
            "Ich heiße Chora.",
            "Ich empfehle eine Skalierung von 1,5.",
            "I'll be here if you need anything.",
            "Ich zeige Fenster, starte Programme und lese Dateien.",
        ] {
            assert!(!announces(text), "{text}");
        }
    }

    #[test]
    fn answers_that_stop_mid_sentence_are_cut() {
        for text in [
            "I'll set the scale to 1.5 and switch",
            "I’m setting the",
            "I’ll apply",
            "Hier sind die Optionen:",
        ] {
            assert!(cut(text), "{text}");
        }
        for text in [
            "",
            "Fertig.",
            "Gute Nacht! ",
            "Soll ich?",
            "Er sagte „Ja.“",
            "Done.\"",
            "Na ja…",
            "**Fertig.**",
        ] {
            assert!(!cut(text), "{text}");
        }
    }
}
