//! The local word matcher (docs/voice.md section 4): short utterances
//! that are a command word, without the LLM, and the echo filter.

use crate::config::Config;

/// A command word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    CodeCancel,
    Cancel,
    Sleep,
    Wake,
    Yes,
    No,
}

/// At most this many words make a command word.
const MAX_WORDS: usize = 4;

/// Lowercase words without punctuation; `ä→ae`, `ö→oe`, `ü→ue`, `ß→ss`.
/// A hyphen splits words.
pub fn normalize(text: &str) -> Vec<String> {
    let mut out = String::new();
    for c in text.to_lowercase().chars() {
        match c {
            'ä' => out.push_str("ae"),
            'ö' => out.push_str("oe"),
            'ü' => out.push_str("ue"),
            'ß' => out.push_str("ss"),
            c if c.is_alphanumeric() => out.push(c),
            _ => out.push(' '),
        }
    }
    out.split_whitespace().map(str::to_owned).collect()
}

/// Whether two words are equal, or differ by one edit when both have
/// four letters or more.
fn near(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    if a.len().min(b.len()) < 4 || a.len().abs_diff(b.len()) > 1 {
        return false;
    }
    // One substitution, insertion or deletion.
    let (short, long) = if a.len() <= b.len() {
        (&a, &b)
    } else {
        (&b, &a)
    };
    let start = short
        .iter()
        .zip(long.iter())
        .take_while(|(x, y)| x == y)
        .count();
    if short.len() == long.len() {
        short[start + 1..] == long[start + 1..]
    } else {
        short[start..] == long[start + 1..]
    }
}

/// Whether the words equal a phrase, word by word or written together
/// ("Codingaufgabe" for "Coding-Aufgabe").
fn phrase(words: &[String], phrase: &str) -> bool {
    let want = normalize(phrase);
    (want.len() == words.len() && want.iter().zip(words).all(|(w, x)| near(w, x)))
        || want.concat() == words.concat()
}

/// The words of `text` without the filler words.
fn content(config: &Config, text: &str) -> Vec<String> {
    let fillers: Vec<Vec<String>> = config.filler_words.iter().map(|f| normalize(f)).collect();
    normalize(text)
        .into_iter()
        .filter(|w| !fillers.iter().any(|f| f.len() == 1 && f[0] == *w))
        .collect()
}

/// The command word of an utterance of `length_ms`, if it is one. Yes
/// and no count only when `confirming`, as the [`answer`] of any length.
pub fn matches(config: &Config, text: &str, length_ms: u32, confirming: bool) -> Option<Word> {
    let words = content(config, text);
    let lists = [
        (Word::CodeCancel, &config.code_cancel_words),
        (Word::Cancel, &config.cancel_words),
        (Word::Sleep, &config.sleep_words),
        (Word::Wake, &config.wake_words),
    ];
    let command = (length_ms <= config.word_max_ms && (1..=MAX_WORDS).contains(&words.len()))
        .then(|| {
            lists
                .into_iter()
                .find(|(_, list)| list.iter().any(|p| phrase(&words, p)))
        })
        .flatten()
        .map(|(word, _)| word);
    command.or_else(|| match answer(config, text) {
        Answer::Yes if confirming => Some(Word::Yes),
        Answer::No if confirming => Some(Word::No),
        _ => None,
    })
}

/// A text as the answer to a spoken confirmation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    Yes,
    No,
    /// A short text without a clear yes or no.
    Unclear,
    /// A long text without a yes or no word: a new request.
    Request,
}

/// Whether two words are near: equal, or one edit apart from five
/// letters on, so "mich" is not "mach".
fn close(a: &str, b: &str) -> bool {
    a == b || (a.chars().count().min(b.chars().count()) >= 5 && near(a, b))
}

/// Whether a phrase of `list` starts `words` at `at`.
fn has_at(words: &[String], at: usize, list: &[String]) -> bool {
    list.iter().any(|p| {
        let want = normalize(p);
        !want.is_empty()
            && words.len() >= at + want.len()
            && want.iter().zip(&words[at..]).all(|(w, x)| close(w, x))
    })
}

/// The answer in `text`: a yes or no word first counts whatever
/// follows ("Ja, die Skalierung"), unless a short answer also has the
/// other ("Ja, aber nicht"); a short answer (at most four words) counts
/// by the yes or no word in it. A text of filler words only is read with
/// them ("Bitte.").
pub fn answer(config: &Config, text: &str) -> Answer {
    let mut words = content(config, text);
    if words.is_empty() {
        words = normalize(text);
    }
    let anywhere = |list| (0..words.len()).any(|at| has_at(&words, at, list));
    let (yes, no) = (anywhere(&config.yes_words), anywhere(&config.no_words));
    let short = words.len() <= MAX_WORDS;
    let first_no = has_at(&words, 0, &config.no_words);
    let first_yes = has_at(&words, 0, &config.yes_words) && !(short && no);
    match (first_no, first_yes, short, yes, no) {
        (true, ..) => Answer::No,
        (_, true, ..) | (_, _, true, true, false) => Answer::Yes,
        (_, _, true, false, true) => Answer::No,
        (_, _, false, false, false) => Answer::Request,
        _ => Answer::Unclear,
    }
}

/// The agent of `names` whose name starts `text` ("Coder, mach weiter"),
/// near as a command word, and the rest of the text after it.
pub fn addressed<'a>(
    text: &str,
    names: impl IntoIterator<Item = &'a String>,
) -> Option<(String, String)> {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    names.into_iter().find_map(|name| {
        let want = normalize(name);
        let n = want.len();
        if n == 0 || tokens.len() < n {
            return None;
        }
        let head = normalize(&tokens[..n].join(" "));
        // A name word of five letters or more may differ by one edit: so
        // "oder" is not "coder".
        let near_all = head.len() == n && want.iter().zip(&head).all(|(w, h)| close(w, h));
        (near_all || head.concat() == want.concat()).then(|| (name.clone(), tokens[n..].join(" ")))
    })
}

/// Whether `text` is the agent's own voice: more than 80 % of its words
/// are in `said`, the words spoken in the last seconds.
pub fn is_echo(text: &str, said: &str) -> bool {
    let words = normalize(text);
    if words.is_empty() {
        return false;
    }
    let said = normalize(said);
    let found = words.iter().filter(|w| said.contains(w)).count();
    found * 5 > words.len() * 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_at_the_start_addresses_an_agent() {
        let names = ["coder".to_owned(), "chora-docs".to_owned()];
        let to = |text| addressed(text, &names);
        assert_eq!(
            to("Coder, mach weiter."),
            Some(("coder".into(), "mach weiter.".into()))
        );
        assert_eq!(to("Koder: stopp"), Some(("coder".into(), "stopp".into())));
        assert_eq!(
            to("Chora Docs, lies vor"),
            Some(("chora-docs".into(), "lies vor".into()))
        );
        assert_eq!(to("Coder"), Some(("coder".into(), String::new())));
        assert_eq!(to("Mach weiter, Coder"), None);
        assert_eq!(to("Cola, bitte"), None, "too far");
        assert_eq!(to("Oder doch nicht"), None, "a short word, one edit");
    }

    #[test]
    fn the_matcher_finds_words_of_asr_variants_and_rejects_near_misses() {
        let config = Config::default();
        let m = |text, confirming| matches(&config, text, 1000, confirming);
        for (text, word) in [
            ("Stopp.", Word::Cancel),
            ("Stop!", Word::Cancel),
            ("Stopp bitte", Word::Cancel),
            ("Vergiss es.", Word::Cancel),
            ("Coding-Aufgabe abbrechen.", Word::CodeCancel),
            ("Codingaufgabe abbrechen", Word::CodeCancel),
            ("Programmieraufgabe abbrechen!", Word::CodeCancel),
            ("Schlaf.", Word::Sleep),
            ("Schlaf jetzt!", Word::Sleep),
            ("Hör zu.", Word::Wake),
            ("Hoer zu", Word::Wake),
            ("Hallo Cora.", Word::Wake),
            ("Hallo, Chora!", Word::Wake),
        ] {
            assert_eq!(m(text, false), Some(word), "{text}");
        }
        for text in [
            "Schlafen gehen",
            "Stopp das Video",
            "Hallo",
            "Zeig mir bitte die Agenda für heute.",
            "",
            "...",
        ] {
            assert_eq!(m(text, false), None, "{text}");
        }
    }

    #[test]
    fn yes_and_no_count_only_in_a_confirmation() {
        let config = Config::default();
        let m = |text, confirming| matches(&config, text, 1000, confirming);
        assert_eq!(m("Ja", false), None);
        assert_eq!(m("Ja.", true), Some(Word::Yes));
        assert_eq!(m("Ja, mach das.", true), Some(Word::Yes));
        assert_eq!(m("Mach das bitte.", true), Some(Word::Yes));
        assert_eq!(m("Okay", true), Some(Word::Yes));
        assert_eq!(m("Nein.", true), Some(Word::No));
        assert_eq!(m("Lieber nicht.", true), Some(Word::No));
        assert_eq!(m("Stopp.", true), Some(Word::Cancel), "cancel comes first");
        assert_eq!(
            matches(&config, "Stopp.", 2500, false),
            None,
            "longer than word_max_ms"
        );
    }

    #[test]
    fn a_confirmation_answer_counts_by_its_first_or_only_yes_or_no() {
        let config = Config::default();
        let a = |text| answer(&config, text);
        for text in [
            "Ja, die Skalierung des Bildschirms.",
            "Bitte.",
            "Ja, versuche es bitte noch mal.",
            "Yes, do it",
            "Please.",
            "Sure.",
            "Go ahead.",
            "Mach.",
            "Okay, mach mal.",
            "Genau so",
            "Das passt, genau.",
        ] {
            assert_eq!(a(text), Answer::Yes, "{text}");
        }
    }

    #[test]
    fn short_forms_of_yes_are_yes_and_near_no_no_word() {
        let config = Config::default();
        let short = [
            "yeah", "yep", "yup", "jo", "jap", "jep", "klar", "gern", "gerne",
        ];
        for word in short {
            for text in [format!("{word}."), format!("{word}, mach das")] {
                assert_eq!(answer(&config, &text), Answer::Yes, "{text}");
            }
            for no in &config.no_words {
                for no in normalize(no) {
                    assert!(!close(word, &no) && !near(word, &no), "{word} {no}");
                }
            }
        }
        assert_eq!(answer(&config, "Yeah."), Answer::Yes);
        assert_eq!(answer(&config, "Klar, aber nicht"), Answer::Unclear);
    }

    #[test]
    fn an_answer_with_both_or_none_is_unclear_or_a_request() {
        let config = Config::default();
        let a = |text| answer(&config, text);
        for (text, want) in [
            ("Nein, lieber nicht", Answer::No),
            ("No, leave the screen as it is.", Answer::No),
            ("Nein.", Answer::No),
            ("Bitte nicht.", Answer::No),
            ("Ja, aber nicht", Answer::Unclear),
            ("1.5", Answer::Unclear),
            ("Lass mich kurz", Answer::Unclear),
            ("Die Skalierung, ja, die vom Bildschirm.", Answer::Unclear),
            ("Zeig mir die Agenda für heute.", Answer::Request),
        ] {
            assert_eq!(a(text), want, "{text}");
        }
        // As a command word in a confirmation, of any length.
        let m = |text, ms, confirming| matches(&config, text, ms, confirming);
        let long = "Ja, die Skalierung des Bildschirms.";
        assert_eq!(
            [
                m(long, 2500, true),
                m("Bitte.", 600, true),
                m("Bitte.", 600, false)
            ],
            [Some(Word::Yes), Some(Word::Yes), None]
        );
    }

    #[test]
    fn near_allows_one_edit_from_four_letters() {
        assert!(near("chora", "cora"));
        assert!(near("stopp", "stop"));
        assert!(near("schlaf", "schlav"));
        assert!(!near("ja", "da"), "short words must be equal");
        assert!(!near("schlaf", "schlafen"));
        assert!(!near("halt", "hallo"));
    }

    #[test]
    fn a_text_of_the_agents_own_words_is_echo() {
        let said = "Heute hast du drei Termine. Um zehn Uhr triffst du das Team.";
        assert!(is_echo("drei Termine um zehn", said));
        assert!(!is_echo("Stopp, warte mal.", said));
        assert!(!is_echo("Termine verschieben bitte", said));
        assert!(!is_echo("", said));
    }
}
