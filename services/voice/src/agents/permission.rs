//! The permission requests of an agent: the question, the outcome and
//! what the request asks.

use std::path::Path;

use serde_json::{Value, json};
use tokio::sync::oneshot;

use super::title;

/// A question of the agent that waits for the user.
pub(super) struct Permission {
    pub(super) id: Value,
    pub(super) options: Vec<Value>,
    /// The core drops its sender after `agent_permission_s` from when
    /// it spoke the question, or when it drops the question.
    pub(super) answer: oneshot::Receiver<bool>,
}

/// The spoken question of a permission request: `asks`, or "Always
/// allow?" when the only allowing option is `allow_always`, so that a
/// yes never allows always unheard.
pub(super) fn question(
    asks: &str,
    name: &str,
    what: &str,
    options: &[Value],
    german: bool,
) -> String {
    let has = |kind: &str| options.iter().any(|o| o["kind"] == kind);
    if !has("allow_once") && has("allow_always") {
        // ponytail: English and German, as the verbs of `asked`.
        return if german {
            format!("{} möchte {what}. Immer erlauben?", title(name))
        } else {
            format!("{} wants to {what}. Always allow?", title(name))
        };
    }
    asks.replace("{name}", &title(name)).replace("{what}", what)
}

/// The answer to a permission request: the first option of an allowing
/// kind for yes, of a rejecting kind for no; cancelled without one. Yes
/// takes `allow_always` only without `allow_once`, after `question`
/// asked for it.
pub(super) fn outcome(options: &[Value], yes: bool) -> Value {
    let kinds = if yes {
        ["allow_once", "allow_always"]
    } else {
        ["reject_once", "reject_always"]
    };
    kinds
        .iter()
        .find_map(|k| options.iter().find(|o| o["kind"] == *k))
        .map_or_else(
            || json!({"outcome": {"outcome": "cancelled"}}),
            |o| json!({"outcome": {"outcome": "selected", "optionId": o["optionId"]}}),
        )
}

/// Whether a permission request takes a key or a button of the HUD, as
/// a security question of the reply does: all but a read, a search and
/// a thought, so a command, a write, a move, a delete, a fetch and a
/// kind that is unknown; and any request without an `allow_once`
/// option, which a plain yes would turn into `allow_always`.
pub(super) fn is_physical(tool: &Value, options: &[Value]) -> bool {
    let once = options.iter().any(|o| o["kind"] == "allow_once");
    !once || !matches!(tool["kind"].as_str(), Some("read" | "search" | "think"))
}

/// What the user hears when a request is rejected because its detail
/// does not show whole and as it is: too long, or with characters that
/// hide or reorder text (sh-26); none when it shows.
pub(super) fn unshowable(detail: &str, name: &str, german: bool) -> Option<String> {
    let who = title(name);
    let told = if crate::chat::too_long(detail) {
        if german {
            format!("{who} wollte etwas tun, das zu lang ist, um es zu zeigen. Abgelehnt.")
        } else {
            format!("{who} asked for something too long to show. Rejected.")
        }
    } else if chora_kit::chat::hides(detail) {
        if german {
            format!(
                "{who} wollte etwas tun, das Zeichen enthält, die Text verbergen oder umstellen. Abgelehnt."
            )
        } else {
            format!(
                "{who} asked for something with characters that hide or reorder text. Rejected."
            )
        }
    } else {
        return None;
    };
    Some(told)
}

/// What a permission request asks, in full, for the screen.
pub(super) fn detail(tool: &Value) -> String {
    let title = tool["title"].as_str().unwrap_or_default().trim();
    let mut input = tool["rawInput"].clone();
    crate::chat::hide(&mut input);
    if input.is_null() {
        title.to_owned()
    } else {
        format!("{title}\n{input}")
    }
}

/// What a permission request asks, short enough to speak: the title of
/// its tool call, a path as its file name for an edit, a read, a delete
/// and a move, and the kind as a verb.
pub(super) fn asked(tool: &Value, german: bool) -> String {
    let title = tool["title"].as_str().unwrap_or_default().trim();
    let short = matches!(
        tool["kind"].as_str(),
        Some("edit" | "read" | "delete" | "move")
    );
    let title = if short && title.starts_with('/') {
        Path::new(title)
            .file_name()
            .map_or(title.into(), |f| f.to_string_lossy())
    } else {
        title.into()
    };
    let title: String = title.chars().take(80).collect();
    // ponytail: English and German verbs, as the default phrases; a
    // phrase per kind in the settings when another language needs it.
    let (en, de) = match tool["kind"].as_str() {
        Some("edit") => ("edit", "ändern"),
        Some("delete") => ("delete", "löschen"),
        Some("move") => ("move", "verschieben"),
        Some("execute") => ("run", "ausführen"),
        Some("fetch") => ("fetch", "abrufen"),
        Some("read") => ("read", "lesen"),
        _ => ("", ""),
    };
    match (title.is_empty(), en.is_empty(), german) {
        (true, true, false) => "do something".into(),
        (true, true, true) => "etwas tun".into(),
        (true, false, false) => en.into(),
        (true, false, true) => de.into(),
        (false, true, _) => title,
        (false, false, false) => format!("{en} {title}"),
        (false, false, true) => format!("{title} {de}"),
    }
}

/// A plan entry as a line: done, in progress or waiting.
pub(super) fn plan_line(entry: &Value) -> String {
    let sign = match entry["status"].as_str() {
        Some("completed") => "✓",
        Some("in_progress") => "●",
        _ => "○",
    };
    format!("{sign} {}", entry["content"].as_str().unwrap_or_default())
}
