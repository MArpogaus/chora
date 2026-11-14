//! The word for the microphone on the state line of the margin column:
//! `Mute` while the voice agent listens, and the way back while it is
//! muted, where the HUD host does not show its badge for that. A press
//! of the person on the word calls `mute` of the agent.

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

/// The hit of the word: `mic:mute` or `mic:on`.
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

    /// The word of the state line and what a press on it asks for.
    fn word(self) -> Option<(&'static str, Option<bool>)> {
        match self {
            Self::Unknown => None,
            Self::On => Some(("Mute", Some(true))),
            Self::Off => Some(("Turn microphone on", Some(false))),
            Self::Failed => Some(("The microphone did not switch", None)),
            Self::Refused => Some(("Use your own mouse", None)),
        }
    }
}

impl Mc {
    /// Takes the state of the microphone from an event of the voice
    /// agent; true when the word of the state line changes.
    pub fn mic_follows(&mut self, mic: Mic) -> bool {
        std::mem::replace(&mut self.mic, mic) != mic
    }

    /// The word for the microphone as an item of the state line `id`: in
    /// the line of the state word, flush right, or alone at the left edge
    /// when mc draws no state word there. Beside the HUD host a muted
    /// microphone has no word of mc: the badge of the host says so there
    /// and turns it on.
    pub fn mic_item(&self, id: &str, slot: i32, alone: bool) -> Option<Item> {
        let host = self.settings.value.voice_hud == crate::config::VoiceHud::Host;
        if host && self.mic == Mic::Off {
            return None;
        }
        let (word, mute) = self.mic.word()?;
        let props = self.ty("margin", None).props(word);
        Some(Item {
            put: if alone { Put::Stack } else { Put::End },
            hit: mute.map(|mute| format!("{HIT}:{}", if mute { "mute" } else { "on" })),
            ..Item::text(format!("{id}-mic"), slot, props, alone)
        })
    }

    /// A press on the word: a person's press switches the microphone with
    /// `voice.mute`; the line says so when the agent refuses, and when the
    /// press was injected. True when `token` is a hit of the word.
    pub fn press_mic(&mut self, token: &str, injected: bool) -> bool {
        let mute = match token.split_once(':') {
            Some((HIT, "mute")) => true,
            Some((HIT, "on")) => false,
            _ => return false,
        };
        self.mic = if injected {
            Mic::Refused
        } else {
            let call = json!({"client": crate::VOICE, "method": "mute", "params": {"mic": mute}});
            match self.rt.conn.call::<_, Value>("methods.call", call) {
                Ok(_) if mute => Mic::Off,
                Ok(_) => Mic::On,
                Err(err) => {
                    eprintln!("chora-mc: voice.mute: {err}");
                    Mic::Failed
                }
            }
        };
        if let Err(err) = self.render() {
            eprintln!("chora-mc: mic: {err}");
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

    /// Beside the HUD host the word Mute stands alone at the left of the
    /// state line once the agent told its state, in 14 pixels or more,
    /// and goes while the microphone is muted.
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
        state(&mut mc, true);
        assert!(word(&mc).is_none(), "a word beside the badge of the host");
    }

    /// Without the host mc draws the state word, the word for the
    /// microphone stands flush right in its line, and a muted microphone
    /// has the way back there.
    #[test]
    fn without_the_host_the_word_stands_in_the_line_of_the_state() {
        let (mut mc, _model) = crate::testing::mc("mic-own", [1280, 800]);
        mc.settings.value.voice_hud = crate::config::VoiceHud::Own;
        mc.show("agenda", false).unwrap();
        mc.method("hud.set", json!({"state": "Listening", "text": ""}))
            .unwrap();
        mc.mic = Mic::On;
        assert_eq!(word(&mc).expect("the word").put, Put::End);
        mc.mic = Mic::Off;
        let item = word(&mc).expect("the way back");
        assert_eq!(item.props.text.as_deref(), Some("Turn microphone on"));
        assert_eq!(item.hit.as_deref(), Some("mic:on"));
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
