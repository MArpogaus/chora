use chora_kit::actions::System;

use super::{Action, Mc, NAME, PressState, launch};
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
            Ok(Some(to)) if to != NAME => self.give(chora_kit::give::Give {
                surface: surface.clone(),
                to,
                grab: None,
            }),
            Ok(_) => return,
            Err(err) => Err(err),
        };
        if let Err(err) = given {
            eprintln!("chora-mc: {surface} stays: {}", err.message);
        }
    }

    pub fn act(&mut self, binding: &str) {
        if let Some(action) = self.rt.action(binding).cloned() {
            self.perform(action);
        }
    }

    /// Runs a shared script as an app, or says that it is not installed.
    pub(crate) fn run_shared(&mut self, script: &str, args: &[&str]) {
        let Some(command) = chora_kit::shared::command(script, args) else {
            self.notice(script, "The shared script is not installed.");
            return;
        };
        let mint = self.rt.conn.activation_token();
        let _ = launch(&command, mint.as_deref());
    }

    /// Runs an action of every look that mc has: the locker, the power
    /// menu, the screenshot, the volume, a command, a give, the chat and
    /// the key list. The rest has no meaning in mc.
    pub(crate) fn system(&mut self, system: &System) {
        if let Some((script, args)) = system.script() {
            self.run_shared(script, args);
            return;
        }
        match system {
            System::Volume(change) => self.volume.change(*change, true),
            System::Run(command) => {
                let mint = self.rt.conn.activation_token();
                let _ = launch(&chora_kit::shared::launcher(command), mint.as_deref());
            }
            System::Give(to) => {
                if let Err(err) = chora_kit::give::focused(self, to) {
                    eprintln!("chora-mc: give to {to}: {}", err.message);
                }
            }
            System::Chat => self.chat_with(None),
            System::Keys => {
                self.keys = self.keys.flip();
                if let Err(err) = self.render() {
                    eprintln!("chora-mc: keys: {err}");
                }
            }
            System::Drag
            | System::Terminal
            | System::Launcher
            | System::Close
            | System::Desk(_)
            | System::Cube(_)
            | System::Lock
            | System::Power
            | System::Screenshot => {}
        }
    }

    pub fn perform(&mut self, action: Action) {
        match action {
            Action::System(system) => self.system(&system),
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
            Look::Scene(name) => name,
            Look::Toggle(region) => {
                if let Err(err) = self.toggle(region, None, true) {
                    eprintln!("chora-mc: toggle {}: {err}", region.name());
                }
                return;
            }
            Look::Solo { content, region } => {
                if let Err(err) = self.solo(&content, region, None, true) {
                    eprintln!("chora-mc: solo {content}: {err}");
                }
                return;
            }
        };
        if let Err(err) = self.show(&next, true) {
            eprintln!("chora-mc: {next}: {}", err.message);
        }
    }
}
