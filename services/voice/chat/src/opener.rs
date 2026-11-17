//! The opener: `chora-chat` from a launcher or a key asks the host to
//! show the chat, and hands it the activation token of the act that
//! started the opener, so that the field may take the keyboard. With
//! `--door` it prints the word of a helper door instead, for a panel
//! that cannot read the word table.

use chora_kit::{
    conn::Conn,
    words::{self, Language, Word},
};
use chora_scene::ipc::ClientError;
use serde_json::{Value, json};

/// The client name of the opener.
const NAME: &str = "chat-opener";

/// The client name of the host that draws the chat.
const HOST: &str = "assistant";

/// The parameters of `chat.show`: the helper if the person named one,
/// and the token of the environment, if the opener has one.
#[must_use]
pub fn show_params(name: Option<&str>, token: Option<&str>) -> Value {
    let mut params = json!({});
    if let Some(name) = name {
        params["name"] = name.into();
    }
    if let Some(token) = token.filter(|t| !t.is_empty()) {
        params["token"] = token.into();
    }
    params
}

/// Asks the host to show the chat.
///
/// # Errors
///
/// No chora session runs, or the host does not answer.
pub fn open(name: Option<&str>) -> Result<(), ClientError> {
    let token = std::env::var("XDG_ACTIVATION_TOKEN").ok();
    let params = show_params(name, token.as_deref());
    let mut conn = Conn::connect(NAME)?;
    let call = json!({"client": HOST, "method": "chat.show", "params": params});
    conn.call::<_, Value>("methods.call", call).map(drop)
}

/// The client name of the voice agent.
const VOICE: &str = "voice";

/// The client name of the reader of the door word, which a panel runs
/// again and again while an opener may run.
const DOOR: &str = "chat-door";

/// The word of a helper door: `No helper` without a helper, else the name
/// and the state word of the first helper that is not idle, or of the
/// first one: `coder · working`.
#[must_use]
pub fn door_word(language: Language, helpers: &[(String, String)]) -> String {
    let busy = helpers.iter().find(|(_, state)| state != "idle");
    match busy.or(helpers.first()) {
        Some((name, state)) => words::row(language, name, state, ""),
        None => Word::NoHelperDoor.text(language).to_owned(),
    }
}

/// The helpers that the voice agent names, each with its state; none
/// when no session runs or the voice agent does not answer.
#[must_use]
pub fn helpers() -> Vec<(String, String)> {
    let call = json!({"client": VOICE, "method": "state", "params": {}});
    let state = Conn::connect(DOOR)
        .and_then(|mut conn| conn.call::<_, Value>("methods.call", call))
        .unwrap_or_default();
    let word = |agent: &Value, key: &str| agent[key].as_str().map(str::to_owned);
    state["agents"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|agent| Some((word(agent, "name")?, word(agent, "state")?)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_door_names_the_busy_helper_and_says_when_none_exists() {
        let helper = |name: &str, state: &str| (name.to_owned(), state.to_owned());
        assert_eq!(door_word(Language::En, &[]), "No helper");
        assert_eq!(door_word(Language::De, &[]), "Kein Helfer");
        let one = [helper("coder", "idle")];
        assert_eq!(door_word(Language::En, &one), "coder · idle");
        let two = [helper("atlas", "idle"), helper("coder", "working")];
        assert_eq!(door_word(Language::En, &two), "coder · working");
        assert_eq!(door_word(Language::De, &two), "coder · arbeitet");
    }

    #[test]
    fn the_token_and_the_name_go_to_the_host() {
        assert_eq!(show_params(None, None), json!({}));
        assert_eq!(
            show_params(Some("atlas"), Some("tok")),
            json!({"name": "atlas", "token": "tok"})
        );
        assert_eq!(show_params(None, Some("")), json!({}));
    }

    #[test]
    fn the_launcher_entry_carries_the_words_of_the_table() {
        let entry = include_str!("../../../../flavors/shared/applications/chora-chat.desktop");
        let line = |key: &str| entry.lines().find_map(|l| l.strip_prefix(key));
        assert_eq!(line("Name="), Some(Word::ChatLauncher.text(Language::En)));
        assert_eq!(
            line("Name[de]="),
            Some(Word::ChatLauncher.text(Language::De))
        );
        assert_eq!(line("Exec="), Some("chora-chat"));
    }
}
