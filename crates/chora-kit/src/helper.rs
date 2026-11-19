//! The `helper` key of `state.get` (design 3.1): the state word of the
//! helper that the voice agent announces, or `none` while no helper runs.
//! An adapter feeds it the `voice.state` and `voice.remove` events and
//! `client.disconnected` of `voice`, and only reads it.

use chora_scene::Event;
use serde_json::Value;

/// The word of `state.get` while no helper runs.
pub const NONE: &str = "none";

/// The helpers that voice announced: the voice agent itself (no name) and
/// the named ones, the latest report last.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Helpers {
    seen: Vec<(String, String)>,
}

impl Helpers {
    /// Takes in the data of a `voice.state`.
    pub fn follow(&mut self, data: &Value) {
        let name = data["name"].as_str().unwrap_or_default().to_owned();
        let state = data["state"].as_str().unwrap_or_default().to_owned();
        self.seen.retain(|(n, _)| *n != name);
        self.seen.push((name, state));
    }

    /// Takes in the data of a `voice.remove`: the named helper is gone.
    pub fn forget(&mut self, data: &Value) {
        let name = data["name"].as_str().unwrap_or_default();
        self.seen.retain(|(n, _)| n != name);
    }

    /// Follows an event of the client `voice`: its `voice.state` and
    /// `voice.remove`, and its disconnect.
    pub fn hear(&mut self, event: &Event) {
        match event {
            Event::Emitted {
                event,
                client,
                data,
            } if client == "voice" => match event.as_str() {
                "voice.state" => self.follow(data),
                "voice.remove" => self.forget(data),
                _ => {}
            },
            Event::ClientDisconnected { name } if name == "voice" => self.clear(),
            _ => {}
        }
    }

    /// Voice ended: no helper runs without it.
    pub fn clear(&mut self) {
        self.seen.clear();
    }

    /// The state word of the helper that reported last, else `none`.
    #[must_use]
    pub fn word(&self) -> &str {
        match self.seen.last() {
            Some((_, state)) if !state.is_empty() => state,
            _ => NONE,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn no_report_is_none() {
        assert_eq!(Helpers::default().word(), "none");
    }

    #[test]
    fn the_latest_report_is_the_word() {
        let mut h = Helpers::default();
        h.follow(&json!({"state": "idle"}));
        assert_eq!(h.word(), "idle");
        h.follow(&json!({"name": "atlas", "state": "working"}));
        assert_eq!(h.word(), "working");
        h.follow(&json!({"state": "listening"}));
        assert_eq!(h.word(), "listening");
    }

    #[test]
    fn a_removed_helper_falls_back_to_the_one_before() {
        let mut h = Helpers::default();
        h.follow(&json!({"state": "idle"}));
        h.follow(&json!({"name": "atlas", "state": "working"}));
        h.forget(&json!({"name": "atlas"}));
        assert_eq!(h.word(), "idle");
        h.clear();
        assert_eq!(h.word(), "none");
    }
}
