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

/// The command word of an utterance of `length_ms`, if it is one. Yes
/// and no count only when `confirming`.
pub fn matches(config: &Config, text: &str, length_ms: u32, confirming: bool) -> Option<Word> {
    if length_ms > config.word_max_ms {
        return None;
    }
    let fillers: Vec<Vec<String>> = config.filler_words.iter().map(|f| normalize(f)).collect();
    let words: Vec<String> = normalize(text)
        .into_iter()
        .filter(|w| !fillers.iter().any(|f| f.len() == 1 && f[0] == *w))
        .collect();
    if words.is_empty() || words.len() > MAX_WORDS {
        return None;
    }
    let lists = [
        (Word::CodeCancel, &config.code_cancel_words),
        (Word::Cancel, &config.cancel_words),
        (Word::Sleep, &config.sleep_words),
        (Word::Wake, &config.wake_words),
        (Word::Yes, &config.yes_words),
        (Word::No, &config.no_words),
    ];
    lists
        .into_iter()
        .filter(|(word, _)| confirming || !matches!(word, Word::Yes | Word::No))
        .find(|(_, list)| list.iter().any(|p| phrase(&words, p)))
        .map(|(word, _)| word)
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
        let near_all = head.len() == n && want.iter().zip(&head).all(|(w, h)| near(w, h));
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
        assert_eq!(m("Ja, mach das.", true), None, "two phrases");
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
