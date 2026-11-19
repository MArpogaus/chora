//! The keys that every look has: the launcher, the key list, the chat
//! with a helper, lock, the power menu, a screenshot and the volume keys.
//! A key works in the walk, and while the camera is in a pane it is a
//! grab (`Grab::Key`). The actions are done by `chora_kit::actions::perform`.

use std::time::Instant;

use chora_kit::{
    actions::{
        System,
        perform::{Failure, Host, perform},
    },
    volume::Change,
    words::Language,
};

use crate::{ClientError, Construct};

/// The notice of a key that did not start its program.
#[must_use]
pub fn not_started(language: Language) -> &'static str {
    match language {
        Language::En => "Could not start that. A program or a shared script is missing.",
        Language::De => "Konnte nicht gestartet werden. Ein Programm oder Skript fehlt.",
    }
}

impl Construct {
    /// A key of every look from a grab while the camera is in a pane: the
    /// key list steps out of the pane first, since the list takes the
    /// keyboard.
    pub(crate) fn grabbed_key(&mut self, system: &System) -> Result<(), ClientError> {
        if *system == System::Keys && self.in_pane() {
            self.fly_out()?;
        }
        perform(self, system)
    }

    /// The notice of a key that did not start its program, a script that
    /// is missing, or a window manager that did not take a window.
    fn say_failed(&mut self, failure: Failure) -> Result<(), ClientError> {
        let language = Language::of_session();
        let words = match failure {
            Failure::NoScript(_) | Failure::NotStarted(_) => not_started(language).to_owned(),
            Failure::NotGiven { to, why } => {
                eprintln!("chora-construct: give to {to}: {why}");
                crate::input::refusal(language, &to, &why)
            }
        };
        self.voice.notice(&words, Instant::now());
        self.draw_hud()
    }
}

/// What the construct does for the actions of every look: the key list in
/// the space, the volume with its door, the chat, the launcher, the
/// terminal, a close and a give of its panes. A step through the desks and
/// a turn of the cube have no meaning here.
impl Host for Construct {
    type Error = ClientError;

    fn volume(&mut self, change: Change) -> Result<(), ClientError> {
        self.mixer.change(change, true);
        self.doors.ask_for_reading(Instant::now());
        Ok(())
    }

    fn keys(&mut self) -> Result<(), ClientError> {
        self.toggle_help()
    }

    fn chat(&mut self) -> Result<(), ClientError> {
        let mint = self.rt.conn.activation_token();
        if chora_kit::launch::start("chora-chat", mint.as_deref()).is_some() {
            return Ok(());
        }
        self.say_failed(Failure::NotStarted("chora-chat".into()))
    }

    fn failed(&mut self, failure: Failure) -> Result<(), ClientError> {
        self.say_failed(failure)
    }

    fn terminal(&mut self) -> Result<(), ClientError> {
        self.start_terminal();
        Ok(())
    }

    fn launcher(&mut self) -> Result<(), ClientError> {
        self.start_launcher();
        Ok(())
    }

    fn close(&mut self) -> Result<(), ClientError> {
        self.close_ahead()
    }

    fn give(&mut self, to: &str) -> Result<(), ClientError> {
        self.give_key(to)
    }
}
