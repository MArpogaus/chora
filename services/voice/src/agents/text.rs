//! Text helpers of the agents: cut, redact, title and summary.

use serde_json::{Value, json};

/// The text with at most `limit` bytes, cut on a char boundary and
/// marked with an ellipsis.
pub(super) fn cut(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_owned();
    }
    let room = limit - '…'.len_utf8();
    let end = (text.char_indices().map(|(i, _)| i))
        .take_while(|&i| i <= room)
        .last()
        .unwrap_or(0);
    format!("{}…", &text[..end])
}

/// The text without the secrets that `chat::hide` hides: a JSON text
/// by its keys and each of its texts as below, any other text by the
/// `key: value` and `key=value` of each line. A JSON text without
/// secrets stays as it is.
pub(super) fn redact(text: &str) -> String {
    if let Ok(value) = serde_json::from_str::<Value>(text)
        && (value.is_object() || value.is_array())
    {
        let mut hidden = value.clone();
        crate::chat::hide(&mut hidden);
        redact_texts(&mut hidden);
        return if hidden == value {
            text.to_owned()
        } else {
            hidden.to_string()
        };
    }
    (text.lines().map(redact_line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Each text of a JSON value through [`redact`].
pub(super) fn redact_texts(value: &mut Value) {
    match value {
        Value::String(text) => *text = redact(text),
        Value::Array(items) => items.iter_mut().for_each(redact_texts),
        Value::Object(map) => map.values_mut().for_each(redact_texts),
        _ => {}
    }
}

pub(super) fn redact_line(line: &str) -> String {
    let Some(at) = line.find([':', '=']) else {
        return line.to_owned();
    };
    let key = (line[..at].split_whitespace().last())
        .unwrap_or_default()
        .trim_matches(['-', '"', '\'']);
    let mut probe = json!({ key: "" });
    crate::chat::hide(&mut probe);
    if probe[key] == "(hidden)" {
        format!("{} (hidden)", &line[..=at])
    } else {
        line.to_owned()
    }
}

/// A name with a capital first letter, as spoken and as mc shows it.
pub fn title(name: &str) -> String {
    let mut chars = name.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// The first sentence of a long text and `details`; a short text as it
/// is.
pub fn summary(text: &str, limit: usize, details: &str) -> String {
    let text = text.trim();
    if text.chars().count() <= limit {
        return text.to_owned();
    }
    let end = text
        .char_indices()
        .find(|&(i, c)| {
            c == '\n' || (".!?".contains(c) && text[i + 1..].starts_with(char::is_whitespace))
        })
        .map_or(text.len(), |(i, c)| i + c.len_utf8());
    let first: String = text[..end].chars().take(limit).collect();
    let first = first.trim();
    format!("{first} {details}").trim().to_owned()
}
