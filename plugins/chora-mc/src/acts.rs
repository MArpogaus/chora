use super::{Action, Mc, NAME, PressState, launch};

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
        if !matches!(self.rt.action(binding), Some(Action::Drag)) {
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

    /// Runs the actions of the system: the locker, the power menu, the
    /// screenshot and the volume.
    pub(crate) fn system(&mut self, action: &Action) {
        match action {
            Action::Lock => self.run_shared("power", &["lock"]),
            Action::Power => self.run_shared("power", &[]),
            Action::Screenshot => self.run_shared("screenshot", &[]),
            Action::Volume(change) => self.volume.change(*change, true),
            _ => {}
        }
    }

    pub fn perform(&mut self, action: Action) {
        self.system(&action);
        self.turn(action);
    }

    /// Does an action of a binding that is not one of the system.
    pub(crate) fn turn(&mut self, action: Action) {
        let names = self.order();
        let at = self
            .current
            .as_ref()
            .and_then(|c| names.iter().position(|n| n == c))
            .unwrap_or(0);
        let next = match action {
            Action::NextScene => names[(at + 1) % names.len()].clone(),
            Action::PreviousScene => names[(at + names.len() - 1) % names.len()].clone(),
            Action::Scene(name) => name,
            Action::Run(command) => {
                let mint = self.rt.conn.activation_token();
                let _ = launch(&chora_kit::shared::launcher(&command), mint.as_deref());
                return;
            }
            Action::Give(to) => {
                if let Err(err) = chora_kit::give::focused(self, &to) {
                    eprintln!("chora-mc: give to {to}: {}", err.message);
                }
                return;
            }
            Action::Drag
            | Action::Lock
            | Action::Power
            | Action::Screenshot
            | Action::Volume(_) => return,
            Action::Toggle(region) => {
                if let Err(err) = self.toggle(region, None, true) {
                    eprintln!("chora-mc: toggle {}: {err}", region.name());
                }
                return;
            }
            Action::Solo { content, region } => {
                if let Err(err) = self.solo(&content, region, None, true) {
                    eprintln!("chora-mc: solo {content}: {err}");
                }
                return;
            }
            Action::Chat => {
                self.chat_with(None);
                return;
            }
            Action::Keys => {
                self.keys = self.keys.flip();
                if let Err(err) = self.render() {
                    eprintln!("chora-mc: keys: {err}");
                }
                return;
            }
        };
        if let Err(err) = self.show(&next, true) {
            eprintln!("chora-mc: {next}: {}", err.message);
        }
    }
}
