//! Transactions that wait for client commits (D22).
//!
//! A transaction with `configure` commands waits until every configured
//! client commits a buffer for its configure, or until a deadline passes.
//! Then the core applies the whole transaction to the scene in one step.
//! Transactions run one after the other, in the order of their arrival.

use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

use chora_scene::{Command, Effect, ipc::Message};
use serde_json::Value;
use smithay::{
    reexports::calloop::timer::{TimeoutAction, Timer},
    utils::Serial,
    wayland::shell::xdg::ToplevelSurface,
};

use crate::{ipc::rejected, state::State};

/// How long a transaction waits for its clients.
const DEADLINE: Duration = Duration::from_millis(150);

/// The queue of transactions.
#[derive(Default)]
pub struct Transactions {
    queue: VecDeque<Pending>,
}

struct Pending {
    conn: u64,
    id: Value,
    owner: String,
    commands: Vec<Command>,
    /// The configure serial that each surface still has to commit.
    waiting: HashMap<String, Serial>,
    /// Set when the configure events are out.
    deadline: Option<Instant>,
}

impl Transactions {
    /// Notes a commit of a toplevel.
    pub fn committed(&mut self, id: &str, toplevel: &ToplevelSurface) {
        let Some(head) = self.queue.front_mut() else {
            return;
        };
        let Some(serial) = head.waiting.get(id) else {
            return;
        };
        let acked = toplevel.with_cached_state(|s| s.last_acked.as_ref().map(|c| c.serial));
        if acked.is_some_and(|a| a >= *serial) {
            head.waiting.remove(id);
        }
    }

    /// Stops waiting for a destroyed surface.
    pub fn forget(&mut self, id: &str) {
        for pending in &mut self.queue {
            pending.waiting.remove(id);
        }
    }

    /// Drops the transactions of a closed connection.
    pub fn drop_conn(&mut self, conn: u64) {
        self.queue.retain(|p| p.conn != conn);
    }
}

impl State {
    /// Queues a transaction. The response follows when it completes.
    pub fn submit_transaction(
        &mut self,
        conn: u64,
        id: Value,
        owner: String,
        commands: Vec<Command>,
    ) {
        self.transactions.queue.push_back(Pending {
            conn,
            id,
            owner,
            commands,
            waiting: HashMap::new(),
            deadline: None,
        });
        self.complete_transactions();
    }

    /// Starts and completes the transactions at the head of the queue, as
    /// far as their clients allow.
    pub fn complete_transactions(&mut self) {
        while let Some(mut head) = self.transactions.queue.pop_front() {
            if head.deadline.is_none()
                && let Err(error) = self.start(&mut head)
            {
                self.ipc
                    .respond(head.conn, &Message::error(head.id, rejected(error)));
                continue;
            }
            let due = head.deadline.is_some_and(|d| Instant::now() >= d);
            if !head.waiting.is_empty() && !due {
                self.transactions.queue.push_front(head);
                break;
            }
            self.finish(head);
        }
    }

    /// Checks the transaction on a copy of the scene and sends its
    /// configure events.
    fn start(&mut self, head: &mut Pending) -> Result<(), String> {
        let effects = self
            .scene
            .clone()
            .apply(&head.owner, &head.commands)
            .map_err(|e| e.to_string())?;
        for effect in &effects {
            if let Effect::Configure {
                surface,
                size,
                states,
            } = effect
                && let Some(serial) = self.configure(surface, *size, states)
            {
                head.waiting.insert(surface.clone(), serial);
            }
        }
        let deadline = Instant::now() + DEADLINE;
        head.deadline = Some(deadline);
        if !head.waiting.is_empty() {
            self.wake_at(deadline);
        }
        Ok(())
    }

    /// Runs `complete_transactions` again at the deadline.
    fn wake_at(&self, deadline: Instant) {
        let timer = Timer::from_deadline(deadline);
        let inserted = self.loop_handle.insert_source(timer, |_, (), state| {
            state.complete_transactions();
            TimeoutAction::Drop
        });
        if let Err(err) = inserted {
            tracing::warn!(
                ?err,
                "no deadline timer; the transaction waits for its clients"
            );
        }
    }

    /// Applies the transaction to the scene and answers the client.
    fn finish(&mut self, head: Pending) {
        let message = match self.scene.apply(&head.owner, &head.commands) {
            Ok(effects) => {
                self.execute(effects);
                Message::result(head.id, Value::Null)
            }
            Err(error) => Message::error(head.id, rejected(error.to_string())),
        };
        self.ipc.respond(head.conn, &message);
    }
}
