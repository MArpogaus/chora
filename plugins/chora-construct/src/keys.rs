//! The keys that every look has: the launcher, the key list, the chat
//! with a helper, lock, the power menu, a screenshot and the volume keys.
//! A key works in the walk, and while the camera is in a pane it is a
//! grab (`Grab::Key`). The shared actions come from `chora_kit::shared`.

use std::time::Instant;

use chora_kit::{shared, volume::Change, words::Language};

use crate::{ClientError, Construct, config::Action};

/// The notice of a key that did not start its program.
#[must_use]
pub fn not_started(language: Language) -> &'static str {
    match language {
        Language::En => "Could not start that. A program or a shared script is missing.",
        Language::De => "Konnte nicht gestartet werden. Ein Programm oder Skript fehlt.",
    }
}

fn volume_change(action: Action) -> Change {
    match action {
        Action::VolumeUp => Change::Up,
        Action::VolumeDown => Change::Down,
        _ => Change::Mute,
    }
}

impl Construct {
    /// A key of every look from a grab while the camera is in a pane: the
    /// key list steps out of the pane first, since the list takes the
    /// keyboard.
    pub(crate) fn grabbed_key(&mut self, action: Action) -> Result<(), ClientError> {
        if action == Action::Help && self.in_pane() {
            self.fly_out()?;
        }
        self.shared_key(action)
    }

    /// A key of every look. A program or script that does not start is a
    /// notice.
    pub(crate) fn shared_key(&mut self, action: Action) -> Result<(), ClientError> {
        let command = match action {
            Action::Launcher => {
                self.start_launcher();
                return Ok(());
            }
            Action::Help => return self.toggle_help(),
            Action::VolumeUp | Action::VolumeDown | Action::Mute => {
                self.mixer.change(volume_change(action), true);
                self.doors.ask_for_reading(Instant::now());
                return Ok(());
            }
            Action::Chat => Some("chora-chat".to_owned()),
            Action::Lock => shared::command("power", &["lock"]),
            Action::Power => shared::command("power", &[]),
            Action::Screenshot => shared::command("screenshot", &[]),
            _ => return Ok(()),
        };
        let mint = self.rt.conn.activation_token();
        let started = command.and_then(|c| chora_kit::launch(&c, mint.as_deref()));
        if started.is_some() {
            return Ok(());
        }
        let words = not_started(Language::of_session());
        self.voice.notice(words, Instant::now());
        self.draw_hud()
    }
}
