//! The word for the microphone on the state line of the margin column:
//! `Mute` while the voice agent listens; while it is muted the badge of
//! the HUD host says so and turns it on. A press of the person on the
//! word calls `mute` of the agent.

use serde_json::{Value, json};

use crate::{
    Mc,
    region::{Item, Put},
};

/// What mc knows of the microphone of the voice agent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mic {
    /// No state of the agent came yet, or the agent is gone.
    #[default]
    Unknown,
    /// The agent hears.
    On,
    /// The microphone is muted.
    Off,
    /// The agent did not switch the microphone.
    Failed,
    /// A press that no person made tried to switch it.
    Refused,
}

/// The hit of the word: `mic:mute`.
const HIT: &str = "mic";

impl Mic {
    /// The state of the microphone in the data of `voice.state`.
    pub fn of(data: &Value) -> Self {
        if data["muted"]["mic"].as_bool() == Some(true) {
            Self::Off
        } else {
            Self::On
        }
    }

    /// The word of the state line and whether a press on it mutes. A
    /// muted microphone has no word: the badge of the host has it.
    fn word(self) -> Option<(&'static str, bool)> {
        match self {
            Self::Unknown | Self::Off => None,
            Self::On => Some(("Mute", true)),
            Self::Failed => Some(("The microphone did not switch", false)),
            Self::Refused => Some(("Use your own mouse", false)),
        }
    }
}

impl Mc {
    /// Takes the state of the microphone from an event of the voice
    /// agent; true when the word of the state line changes.
    pub fn mic_follows(&mut self, mic: Mic) -> bool {
        std::mem::replace(&mut self.mic, mic) != mic
    }

    /// The word for the microphone as an item of the state line `id`:
    /// right after a state word of `hud.set`, which the word of the HUD
    /// host would cover at the right edge, or alone at the left edge when
    /// mc draws no state word there.
    pub fn mic_item(&self, id: &str, slot: i32, alone: bool) -> Option<Item> {
        let (word, mutes) = self.mic.word()?;
        let props = self.ty("margin", None).props(word);
        Some(Item {
            put: if alone { Put::Stack } else { Put::After },
            hit: mutes.then(|| format!("{HIT}:mute")),
            ..Item::text(format!("{id}-mic"), slot, props, alone)
        })
    }

    /// A press on the word: a person's press mutes the microphone with
    /// `voice.mute`; the line says so when the agent refuses, and when the
    /// press was injected. True when `token` is a hit of the word.
    pub fn press_mic(&mut self, token: &str, injected: bool) -> bool {
        if token.split_once(':') != Some((HIT, "mute")) {
            return false;
        }
        self.mic = if injected {
            Mic::Refused
        } else {
            let call = json!({"client": crate::VOICE, "method": "mute", "params": {"mic": true}});
            match self.rt.conn.call::<_, Value>("methods.call", call) {
                Ok(_) => Mic::Off,
                Err(err) => {
                    tracing::warn!("voice.mute: {err}");
                    Mic::Failed
                }
            }
        };
        if let Err(err) = self.render() {
            tracing::warn!("mic: {err}");
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use chora_kit::runtime::Plugin as _;
    use serde_json::json;

    use super::Mic;
    use crate::region::{Item, Put};

    fn word(mc: &crate::Mc) -> Option<Item> {
        (mc.items("agenda").into_values())
            .flat_map(|(items, _)| items)
            .find(|i| i.id.ends_with("-state-mic"))
    }

    fn state(mc: &mut crate::Mc, mic: bool) {
        let data = json!({"state": "listening", "muted": {"mic": mic}});
        mc.event(chora_scene::Event::Emitted {
            event: "voice.state".into(),
            client: crate::VOICE.into(),
            data,
        })
        .unwrap();
    }

    /// The word Mute stands alone at the left of the state line once the
    /// agent told its state, in 14 pixels or more, and goes while the
    /// microphone is muted; after a state word of `hud.set` it follows
    /// that word.
    #[test]
    fn the_state_line_says_mute_while_the_agent_listens() {
        let (mut mc, _model) = crate::testing::mc("mic-host", [1280, 800]);
        mc.show("agenda", false).unwrap();
        assert!(word(&mc).is_none(), "a word before any state");
        state(&mut mc, false);
        let item = word(&mc).expect("the word");
        assert_eq!(item.props.text.as_deref(), Some("Mute"));
        assert_eq!(item.hit.as_deref(), Some("mic:mute"));
        assert_eq!(item.put, Put::Stack);
        assert!(item.props.font_size.unwrap() >= 14.0);
        mc.method("hud.set", json!({"state": "Listening", "text": ""}))
            .unwrap();
        assert_eq!(word(&mc).expect("the word").put, Put::After);
        state(&mut mc, true);
        assert!(word(&mc).is_none(), "a word beside the badge of the host");
    }

    /// An injected press does not switch the microphone, and the line
    /// says why; a press of the person calls `mute` of the agent.
    #[test]
    fn only_a_press_of_the_person_switches_the_microphone() {
        let (mut mc, model) = crate::testing::mc("mic-press", [1280, 800]);
        mc.show("agenda", false).unwrap();
        state(&mut mc, false);
        let hit = format!("{}-hit", word(&mc).expect("the word").id);
        let calls = || model.lock().unwrap().calls.len();
        let before = calls();
        assert!(mc.click_mic(&hit, true));
        assert_eq!(calls(), before, "an injected press called the agent");
        assert_eq!(mc.mic, Mic::Refused);
        let item = word(&mc).expect("the refusal");
        assert_eq!(item.props.text.as_deref(), Some("Use your own mouse"));
        assert_eq!(item.hit, None);
        state(&mut mc, false);
        assert!(mc.click_mic(&hit, false));
        assert_eq!(calls(), before + 1);
        assert_eq!(mc.mic, Mic::Off);
    }
}
