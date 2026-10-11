//! The safety layer: the only built-in behavior (D23).

use chora_scene::Effect;

use crate::state::State;

impl State {
    /// The emergency reset: disconnects all clients of the IPC socket and
    /// puts all surfaces in a grid that fills the output.
    pub fn reset(&mut self) {
        for conn in self.ipc.conn_ids() {
            self.ipc_close(conn, true);
        }
        let effects = self.scene.reset(self.output_area());
        self.send_configures(&effects);
        self.focus(None);
        tracing::info!("emergency reset");
    }

    fn send_configures(&self, effects: &[Effect]) {
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

    /// The logical size of the output.
    fn output_area(&self) -> [f64; 2] {
        let size = self
            .output
            .as_ref()
            .and_then(smithay::output::Output::current_mode)
            .map_or((0, 0).into(), |m| m.size.to_logical(1));
        [f64::from(size.w), f64::from(size.h)]
    }
}
