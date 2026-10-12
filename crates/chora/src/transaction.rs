//! Transactions that wait for client commits (D22).
//!
//! A transaction with `configure` commands waits until every configured
//! client commits a buffer for its configure, or until a deadline passes.
//! Then the core applies the whole transaction to the scene in one step.
//! Transactions run one after the other, in the order of their arrival.
//!
//! A commit that answers a configure of a transaction waits as a commit
//! blocker until the transaction applies, as in niri and Pinnacle. So the
//! new buffers of a transaction and its scene changes show in one frame.
//!
//! A surface that misses a deadline is late. No transaction waits for it
//! until it commits a buffer for its newest configure, so a hung client
//! delays the queue once, not once per transaction.

use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

use chora_scene::{Command, Effect, ipc::Message};
use serde_json::Value;
use smithay::{
    reexports::{
        calloop::timer::{TimeoutAction, Timer},
        wayland_server::{Resource, protocol::wl_surface::WlSurface},
    },
    utils::Serial,
    wayland::{
        compositor::{Barrier, CompositorHandler, add_blocker, with_states},
        shell::xdg::XdgToplevelSurfaceData,
    },
};

use crate::{ipc::rejected, state::State};

/// How long a transaction waits for its clients.
const DEADLINE: Duration = Duration::from_millis(150);
/// How many transactions of one connection can wait in the queue.
const MAX_PENDING: usize = 16;

/// The queue of transactions.
#[derive(Default)]
pub struct Transactions {
    queue: VecDeque<Pending>,
    /// The late surfaces, and the newest configure serial of each.
    late: HashMap<String, Serial>,
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
    /// Holds the commits that answered, until the transaction applies.
    barrier: Barrier,
    /// The surfaces whose commits the barrier holds.
    held: Vec<WlSurface>,
}

impl Transactions {
    /// Notes a commit of a toplevel, before Smithay applies it, with the
    /// newest configure serial that the client acknowledged. Returns the
    /// barrier that holds the commit, if the transaction at the head of the
    /// queue waits for it.
    fn answered(
        &mut self,
        id: &str,
        acked: Option<Serial>,
        surface: &WlSurface,
    ) -> Option<Barrier> {
        let caught_up = |serial: &Serial| acked.is_some_and(|a| a >= *serial);
        if self.late.get(id).is_some_and(caught_up) {
            self.late.remove(id);
        }
        let head = self.queue.front_mut()?;
        head.waiting.get(id).is_some_and(caught_up).then(|| {
            head.waiting.remove(id);
            head.held.push(surface.clone());
            head.barrier.clone()
        })
    }

    /// Stops waiting for a destroyed surface.
    pub fn forget(&mut self, id: &str) {
        self.late.remove(id);
        for pending in &mut self.queue {
            pending.waiting.remove(id);
        }
    }

    /// Notes a configure that a transaction sent. The transaction waits for
    /// it, unless the surface is late; a late surface must then catch up
    /// with this newer configure.
    fn configured(&mut self, waiting: &mut HashMap<String, Serial>, surface: &str, serial: Serial) {
        if let Some(missed) = self.late.get_mut(surface) {
            *missed = serial;
        } else {
            waiting.insert(surface.into(), serial);
        }
    }

    /// Whether a transaction of the connection waits in the queue.
    pub fn waits_for(&self, conn: u64) -> bool {
        self.queue.iter().any(|p| p.conn == conn)
    }

    /// Drops the transactions of a closed connection. Returns the surfaces
    /// whose commits they held, for `release`.
    pub fn drop_conn(&mut self, conn: u64) -> Vec<WlSurface> {
        let mut held = Vec::new();
        self.queue.retain_mut(|p| {
            if p.conn != conn {
                return true;
            }
            p.barrier.signal();
            held.append(&mut p.held);
            false
        });
        held
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
        let waiting = self
            .transactions
            .queue
            .iter()
            .filter(|p| p.conn == conn)
            .count();
        if waiting >= MAX_PENDING {
            let error = rejected("too many waiting transactions");
            self.ipc.respond(conn, &Message::error(id, error));
            return;
        }
        self.transactions.queue.push_back(Pending {
            conn,
            id,
            owner,
            commands,
            waiting: HashMap::new(),
            deadline: None,
            barrier: Barrier::new(false),
            held: Vec::new(),
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
                // No other transaction started, so no other needs the
                // images that this one loaded.
                self.assets.images.prune(&self.scene);
                self.ipc
                    .respond(head.conn, &Message::error(head.id, rejected(error)));
                continue;
            }
            let due = head.deadline.is_some_and(|d| Instant::now() >= d);
            if !head.waiting.is_empty() && !due {
                self.transactions.queue.push_front(head);
                break;
            }
            // The surfaces that still wait missed the deadline: now they
            // are late.
            self.transactions.late.extend(head.waiting.drain());
            self.finish(head);
        }
    }

    /// Checks the transaction on a copy of the scene and sends its
    /// configure events.
    fn start(&mut self, head: &mut Pending) -> Result<(), String> {
        let effects = self
            .scene
            .check(&head.owner, &head.commands)
            .map_err(|e| e.to_string())?;
        self.assets.images.load_for(&head.commands)?;
        self.check_effects(&head.commands)?;
        for effect in &effects {
            if let Effect::Configure {
                surface,
                size,
                states,
            } = effect
                && let Some(serial) = self.configure(surface, *size, states)
            {
                self.transactions
                    .configured(&mut head.waiting, surface, serial);
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

    /// Compiles the shaders of the effects and of the shader nodes that the
    /// commands name, unless they compiled before in their current version.
    /// Without a backend, for example before the first output, the check
    /// waits for the first frame.
    fn check_effects(&mut self, commands: &[Command]) -> Result<(), String> {
        for command in commands {
            let (Command::Create { props, .. }
            | Command::Update { props, .. }
            | Command::Animate { props, .. }) = command
            else {
                continue;
            };
            let shaders = props.effects.iter().flatten().map(|e| &e.shader);
            for path in shaders.chain(props.shader.iter()) {
                let modified = std::fs::metadata(path).and_then(|m| m.modified()).ok();
                let checked = self.assets.checked_effects.get(path);
                if modified.is_some() && checked == Some(&modified) {
                    continue;
                }
                if let Some(backend) = &mut self.backend {
                    backend.check_effect(path)?;
                    self.assets.checked_effects.insert(path.clone(), modified);
                }
            }
        }
        Ok(())
    }

    /// Holds a commit of a toplevel that answers the transaction at the
    /// head of the queue. Runs before Smithay applies the commit; the
    /// transaction completes after it.
    pub fn hold_commit(&mut self, surface: &WlSurface) {
        let Some(id) = self.surface_id(surface) else {
            return;
        };
        let acked = with_states(surface, |states| {
            let role = states.data_map.get::<XdgToplevelSurfaceData>()?;
            role.lock().ok()?.last_acked.as_ref().map(|c| c.serial)
        });
        let Some(barrier) = self.transactions.answered(&id, acked, surface) else {
            return;
        };
        add_blocker(surface, barrier);
        let complete = self
            .transactions
            .queue
            .front()
            .is_some_and(|h| h.waiting.is_empty());
        if complete {
            self.loop_handle.insert_idle(Self::complete_transactions);
        }
    }

    /// Applies the commits that a transaction held.
    pub fn release(&mut self, held: &[WlSurface]) {
        let mut clients = Vec::new();
        for client in held.iter().filter_map(Resource::client) {
            if !clients.contains(&client) {
                clients.push(client);
            }
        }
        let display = self.display_handle.clone();
        for client in clients {
            self.client_compositor_state(&client)
                .blocker_cleared(self, &display);
        }
    }

    /// Applies the transaction to the scene, answers the client, and then
    /// applies the commits that the transaction held.
    fn finish(&mut self, mut head: Pending) {
        let applied = self.scene.apply(&head.owner, &head.commands);
        self.assets.images.prune(&self.scene);
        let message = match applied {
            Ok(effects) => {
                self.execute(effects);
                self.refresh_pointer();
                Message::result(head.id, Value::Null)
            }
            Err(error) => Message::error(head.id, rejected(error.to_string())),
        };
        self.ipc.respond(head.conn, &message);
        head.barrier.signal();
        let surfaces = std::mem::take(&mut head.held);
        self.release(&surfaces);
    }
}
