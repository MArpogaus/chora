//! The opener: `chora-chat` from a launcher or a key asks the host to
//! show the chat, and hands it the activation token of the act that
//! started the opener, so that the field may take the keyboard. With
//! `--door` it prints the word of a helper door instead, for a panel
//! that cannot read the word table.

use chora_kit::{
    conn::Conn,
    words::{Language, Word},
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

/// The client name of the reader of the door word, which a panel runs
/// again and again while an opener may run.
const DOOR: &str = "chat-door";

/// The words of a helper door as the presenter says them in its
/// `state.get` (the helper machine of the kit); `No helper` without an
/// answer, as no helper can be presented then.
#[must_use]
pub fn door_of(language: Language, presented: Option<&Value>) -> String {
    presented
        .and_then(|state| state["door"].as_str())
        .map_or_else(
            || Word::NoHelperDoor.text(language).to_owned(),
            str::to_owned,
        )
}

/// The word of a helper door: asks the presenter for it.
#[must_use]
pub fn door(language: Language) -> String {
    let call = json!({"client": HOST, "method": "state.get", "params": {}});
    let presented = Conn::connect(DOOR)
        .and_then(|mut conn| conn.call::<_, Value>("methods.call", call))
        .ok();
    door_of(language, presented.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_door_says_the_words_of_the_presenter_and_no_helper_without_it() {
        assert_eq!(door_of(Language::En, None), "No helper");
        assert_eq!(door_of(Language::De, None), "Kein Helfer");
        let presented = json!({"helper": "working", "door": "coder · working"});
        assert_eq!(door_of(Language::En, Some(&presented)), "coder · working");
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
