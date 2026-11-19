//! Doing a [`System`] action (KIT2b): the part that is the same in every
//! look is here, the part that needs the look's own state is a hook of
//! [`Host`]. Two looks call it, mc and construct.

use crate::actions::{Step, System, Turn};
use crate::give::{self, Windows};
use crate::volume::Change;
use crate::{launch, shared};

/// What did not happen, for the look to say in its own way.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failure {
    /// The shared script is not installed.
    NoScript(String),
    /// The program or script did not start.
    NotStarted(String),
    /// The window cannot go to the window manager `to`.
    NotGiven {
        /// The window manager that was to get the window.
        to: String,
        /// Why not.
        why: String,
    },
}

/// A look that does system actions. The hooks with a body do nothing by
/// default: an action that has no meaning in the look.
pub trait Host: Windows {
    /// What a hook answers with: the error of the look's connection.
    type Error;

    /// Changes the volume of the default sound output.
    ///
    /// # Errors
    ///
    /// The look's own.
    fn volume(&mut self, change: Change) -> Result<(), Self::Error>;

    /// Shows or hides the key list.
    ///
    /// # Errors
    ///
    /// The look's own.
    fn keys(&mut self) -> Result<(), Self::Error>;

    /// Opens the chat with a helper.
    ///
    /// # Errors
    ///
    /// The look's own.
    fn chat(&mut self) -> Result<(), Self::Error>;

    /// Says that an action did not happen.
    ///
    /// # Errors
    ///
    /// The look's own.
    fn failed(&mut self, failure: Failure) -> Result<(), Self::Error>;

    /// Starts the terminal of the look.
    ///
    /// # Errors
    ///
    /// The look's own.
    fn terminal(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Starts the launcher of the look.
    ///
    /// # Errors
    ///
    /// The look's own.
    fn launcher(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Asks the focused window to close.
    ///
    /// # Errors
    ///
    /// The look's own.
    fn close(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Steps through the desks.
    ///
    /// # Errors
    ///
    /// The look's own.
    fn desk(&mut self, _step: Step) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Turns the cube.
    ///
    /// # Errors
    ///
    /// The look's own.
    fn cube(&mut self, _turn: Turn) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Gives the focused window to the window manager `to`; a refusal is
    /// a [`Failure::NotGiven`].
    ///
    /// # Errors
    ///
    /// The look's own.
    fn give(&mut self, to: &str) -> Result<(), Self::Error>
    where
        Self: Sized,
    {
        match give::focused(self, to) {
            Ok(_) => Ok(()),
            Err(err) => self.failed(Failure::NotGiven {
                to: to.to_owned(),
                why: err.message,
            }),
        }
    }
}

/// Starts a command as an app with the activation token of the look.
fn start<H: Host>(host: &mut H, command: Option<String>, name: &str) -> Result<(), H::Error> {
    let token = host.conn().activation_token();
    match command.and_then(|command| launch::start(&command, token.as_deref())) {
        Some(_) => Ok(()),
        None => host.failed(Failure::NotStarted(name.to_owned())),
    }
}

/// Does a system action in the look: the shared scripts, a command and a
/// give here, the rest through the hooks of the look.
///
/// # Errors
///
/// A hook of the look fails.
pub fn perform<H: Host>(host: &mut H, system: &System) -> Result<(), H::Error> {
    if let Some((script, args)) = system.script() {
        return match shared::command(script, args) {
            Some(command) => start(host, Some(command), script),
            None => host.failed(Failure::NoScript(script.to_owned())),
        };
    }
    match system {
        System::Run(command) => start(host, Some(shared::launcher(command)), command),
        System::Volume(change) => host.volume(*change),
        System::Keys => host.keys(),
        System::Chat => host.chat(),
        System::Give(to) => host.give(to),
        System::Terminal => host.terminal(),
        System::Launcher => host.launcher(),
        System::Close => host.close(),
        System::Desk(step) => host.desk(*step),
        System::Cube(turn) => host.cube(*turn),
        System::Drag | System::Lock | System::Power | System::Screenshot => Ok(()),
    }
}
