//! The time of the deadlines of the core.

use std::time::Instant;

use smithay::reexports::calloop::{
    RegistrationToken,
    timer::{TimeoutAction, Timer},
};

use crate::ipc::rejected;

/// The time of the deadlines: the monotonic clock, or a fixed time in a
/// test.
#[derive(Default)]
pub struct Time {
    fixed: Option<Instant>,
}

impl Time {
    /// The current time.
    pub fn now(&self) -> Instant {
        self.fixed.unwrap_or_else(Instant::now)
    }
}

impl crate::state::State {
    /// Ends what passed its deadline: connections without `hello`, method
    /// calls without an answer, and transactions whose clients did not
    /// commit in time.
    pub fn expire(&mut self) {
        let now = self.time.now();
        for conn in self.ipc.silent(now) {
            self.ipc_close(conn);
        }
        for call in self.methods.due(now) {
            self.end_call(call, Err(rejected("the method gave no answer in time")));
        }
        self.complete_transactions();
    }

    /// Runs `expire` at a deadline. `None` when the event loop takes no
    /// timer.
    pub fn expire_at(&self, deadline: Instant) -> Option<RegistrationToken> {
        let timer = Timer::from_deadline(deadline);
        let inserted = self.loop_handle.insert_source(timer, |_, (), state| {
            state.expire();
            TimeoutAction::Drop
        });
        inserted
            .inspect_err(|err| tracing::warn!(?err, "no timer for a deadline"))
            .ok()
    }
}
