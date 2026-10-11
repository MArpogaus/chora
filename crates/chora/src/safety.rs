//! The safety layer: the only built-in behavior (D23).

use calloop::{
    LoopHandle,
    signals::{Signal, Signals},
};
use chora_scene::Effect;

use crate::state::State;

/// SIGUSR1 starts the emergency reset, for example from a text console when
/// the keyboard no longer reaches the compositor.
pub fn reset_on_signal(
    handle: &LoopHandle<'static, State>,
) -> Result<(), Box<dyn std::error::Error>> {
    let signals = Signals::new(&[Signal::SIGUSR1])?;
    handle
        .insert_source(signals, |_, (), state| state.reset())
        .map_err(|e| e.error)?;
    Ok(())
}

impl State {
    /// The emergency reset: disconnects all clients of the IPC socket, puts
    /// all surfaces in a grid that fills the output and focuses the first
    /// one.
    pub fn reset(&mut self) {
        for conn in self.ipc.conn_ids() {
            self.ipc_close(conn);
        }
        let effects = self.scene.reset(self.reset_area());
        self.send_configures(&effects);
        let first = self
            .scene
            .node(chora_scene::ROOT)
            .and_then(|r| r.children.first().cloned());
        self.focus(first.as_deref());
        tracing::info!("emergency reset");
    }

    fn send_configures(&mut self, effects: &[Effect]) {
        for effect in effects {
            if let Effect::Configure {
                surface,
                size,
                states,
            } = effect
            {
                self.configure(surface, *size, states);
            }
        }
    }
}
