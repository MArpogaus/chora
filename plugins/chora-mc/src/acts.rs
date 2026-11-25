use chora_kit::actions::System;
use chora_kit::actions::perform::{Failure, Host};
use std::convert::Infallible;

use super::{Action, Mc, PressState};
use crate::config::Look;

impl Mc {
    /// A drag (D55): the press takes the window of mc that asked for a
    /// move, else the window of the scene that shows, and a release on the claim of another window manager hands the
    /// window to that one. Elsewhere the window stays; mc moves none.
    pub(crate) fn drag(
        &mut self,
        binding: &str,
        state: PressState,
        point: [f64; 2],
        on: Option<String>,
    ) {
        if !matches!(self.rt.action(binding), Some(Action::System(System::Drag))) {
            return;
        }
        if state == PressState::Pressed {
            let shown = || self.windows.get(self.current.as_deref()?).cloned();
            self.dragged = on
                .filter(|s| self.windows.values().any(|w| w == s))
                .or_else(shown);
            return;
        }
        let Some(surface) = self.dragged.take() else {
            return;
        };
        let given = match self.rt.conn.claimant_at(point) {
            Ok(Some(to)) => self.give(chora_kit::give::Give {
                surface: surface.clone(),
                to,
                grab: None,
            }),
            Ok(None) => return,
            Err(err) => Err(err),
        };
        if let Err(err) = given {
            tracing::warn!("{surface} stays: {}", err.message);
        }
    }

    pub fn act(&mut self, binding: &str) {
        if let Some(action) = self.rt.action(binding).cloned() {
            self.perform(action);
        }
    }

    pub fn perform(&mut self, action: Action) {
        match action {
            Action::System(system) => {
                let Ok(()) = chora_kit::actions::perform::perform(self, &system);
            }
            Action::Look(look) => self.turn(look),
        }
    }

    /// Does an action of mc: a scene, or a region.
    pub(crate) fn turn(&mut self, look: Look) {
        let names = self.order();
        let at = self
            .current
            .as_ref()
            .and_then(|c| names.iter().position(|n| n == c))
            .unwrap_or(0);
        let next = match look {
            Look::NextScene => names[(at + 1) % names.len()].clone(),
            Look::PreviousScene => names[(at + names.len() - 1) % names.len()].clone(),
            Look::Back => return self.step_back(),
            Look::Swallow => return,
            Look::Scene(name) => name,
            Look::Toggle(region) => {
                if let Err(err) = self.toggle(region, None, true) {
                    tracing::warn!("toggle {}: {err}", region.name());
                }
                return;
            }
            Look::Solo { content, region } => {
                if let Err(err) = self.solo(&content, region, None, true) {
                    tracing::warn!("solo {content}: {err}");
                }
                return;
            }
        };
        if let Err(err) = self.show(&next, true) {
            tracing::warn!("{next}: {}", err.message);
        }
    }
}

/// What mc does for the actions of every look that it has: the volume, the
/// chat and the key list. The rest has no meaning in mc.
impl Host for Mc {
    type Error = Infallible;

    fn volume(&mut self, change: chora_kit::volume::Change) -> Result<(), Infallible> {
        self.volume.change(change, true);
        Ok(())
    }

    /// `fuzzel` is the Apps list of kit, which says "Nothing found" for
    /// text that matches nothing; the real fuzzel closes without a word
    /// (mc-44). Another command runs as it is.
    fn run(&mut self, command: &str) -> Result<(), Infallible> {
        let command = match command.trim() {
            "fuzzel" if self.menu.is_none() => "chora-mc --launch fuzzel".to_owned(),
            other => self.themed(other),
        };
        let command = command.as_str();
        let token = self.rt.conn.activation_token();
        let started =
            chora_kit::launch::start(&chora_kit::shared::launcher(command), token.as_deref());
        match started {
            Some(_) => Ok(()),
            None => self.failed(Failure::NotStarted(command.to_owned())),
        }
    }

    /// The power menu of the shared script with the fuzzel of mc as its
    /// menu, in the colors of the theme (rpx1-10); plain when the config
    /// of the menus is not written.
    fn power(&mut self) -> Result<bool, Infallible> {
        if self.menu.is_none() {
            return Ok(false);
        }
        let Some(command) = chora_kit::shared::command("power", &[]) else {
            return Ok(false);
        };
        let token = self.rt.conn.activation_token();
        let started = chora_kit::launch::start(&self.themed(&command), token.as_deref());
        Ok(started.is_some())
    }

    fn keys(&mut self) -> Result<(), Infallible> {
        self.keys = self.keys.flip();
        if let Err(err) = self.render() {
            tracing::warn!("keys: {err}");
        }
        Ok(())
    }

    fn chat(&mut self) -> Result<(), Infallible> {
        self.chat_with(None);
        Ok(())
    }

    fn failed(&mut self, failure: Failure) -> Result<(), Infallible> {
        match failure {
            Failure::NoScript(script) => {
                self.notice(&script, "The shared script is not installed.");
            }
            Failure::NotStarted(_) => {}
            Failure::NotGiven { to, why } => tracing::warn!("give to {to}: {why}"),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests_moved;
