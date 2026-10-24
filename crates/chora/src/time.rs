//! The time of the deadlines of the core. A unit test moves it by hand, so
//! that it checks a timeout without waiting.

use std::time::Instant;

use smithay::reexports::calloop::timer::{TimeoutAction, Timer};

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

    /// Stops the time, so that it moves only with `advance`.
    #[cfg(test)]
    pub fn stop(&mut self) {
        self.fixed = Some(self.now());
    }

    /// Moves the stopped time forward.
    #[cfg(test)]
    pub fn advance(&mut self, by: std::time::Duration) {
        let fixed = self.fixed.as_mut().expect("a stopped time");
        *fixed += by;
    }
}

impl crate::state::State {
    /// Ends what passed its deadline: connections without `hello`, grace
    /// times, method calls without an answer, transactions whose clients
    /// did not commit in time, and unused activation tokens of no client.
    pub fn expire(&mut self) {
        let now = self.time.now();
        for conn in self.ipc.silent(now) {
            self.ipc_close(conn);
        }
        for name in self.ipc.graces_over(now) {
            self.drop_nodes(&name);
        }
        self.repeat_key();
        if self.inject_mark.is_some_and(|until| now >= until) {
            self.unmark_injection();
        }
        for call in self.methods.due(now) {
            self.end_call(call, Err(rejected("the method gave no answer in time")));
        }
        self.complete_transactions();
        self.forget_unused_tokens(crate::protocols::MAX_TOKEN_AGE);
    }

    /// Runs `expire` at a deadline. `None` when the event loop takes no
    /// timer.
    pub fn expire_at(&self, deadline: Instant) -> Result<(), String> {
        let timer = Timer::from_deadline(deadline);
        let inserted = self.loop_handle.insert_source(timer, |_, (), state| {
            state.expire();
            TimeoutAction::Drop
        });
        inserted.map(drop).map_err(|err| {
            tracing::warn!(?err, "no timer for a deadline");
            "no timer for a deadline".to_owned()
        })
    }
}
