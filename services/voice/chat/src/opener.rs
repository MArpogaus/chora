//! The opener: `chora-chat` from a launcher or a key asks the host to
//! show the chat, and hands it the activation token of the act that
//! started the opener, so that the field may take the keyboard.

use chora_kit::conn::Conn;
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

#[cfg(test)]
mod tests {
    use chora_kit::words::{Language, Word};

    use super::*;

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
