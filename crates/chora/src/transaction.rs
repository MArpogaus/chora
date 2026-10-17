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
//! A transaction that names an image file waits aside until a worker
//! thread decoded the file; transactions of other connections go on
//! meanwhile, and the later transactions of the same connection wait
//! behind it. A file that fails to decode rejects the transaction.
//!
//! A surface that misses a deadline is late. No transaction waits for it
//! until it commits a buffer for its newest configure, so a hung client
//! delays the queue once, not once per transaction.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    time::{Duration, Instant},
};

use chora_scene::{
    Command, Effect, TransactionError,
    ipc::{Message, RpcError},
};
use serde_json::Value;
use smithay::{
    reexports::wayland_server::{Resource, protocol::wl_surface::WlSurface},
    utils::Serial,
    wayland::{
        compositor::{Barrier, CompositorHandler, add_blocker, with_states},
        shell::xdg::XdgToplevelSurfaceData,
    },
};

use crate::{images::Decoded, ipc::rejected, state::State};

/// How long a transaction waits for its clients.
pub const DEADLINE: Duration = Duration::from_millis(150);
/// How many transactions of one connection can wait in the queue.
const MAX_PENDING: usize = 16;

/// The queue of transactions.
#[derive(Default)]
pub struct Transactions {
    queue: VecDeque<Pending>,
    /// The transactions that wait for their images, or behind one of their
    /// connection that does, in the order of their arrival.
    staged: Vec<Pending>,
    next_ticket: u64,
    /// The late surfaces, and the newest configure serial of each.
    late: HashMap<String, Serial>,
}

struct Pending {
    conn: u64,
    id: Value,
    owner: String,
    commands: Vec<Command>,
    /// Names the images that the worker decodes for the transaction.
    ticket: u64,
    /// How many images the worker still decodes for it.
    decoding: usize,
    /// The first image that failed to load.
    failed: Option<String>,
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
    /// transaction at the head of the queue, if it waited for the commit.
    fn answered(&mut self, id: &str, acked: Option<Serial>) -> Option<&mut Pending> {
        let caught_up = |serial: &Serial| acked.is_some_and(|a| a >= *serial);
        if self.late.get(id).is_some_and(caught_up) {
            self.late.remove(id);
        }
        let head = self.queue.front_mut()?;
        if !head.waiting.get(id).is_some_and(caught_up) {
            return None;
        }
        head.waiting.remove(id);
        Some(head)
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

    fn all(&self) -> impl Iterator<Item = &Pending> {
        self.queue.iter().chain(&self.staged)
    }

    /// Whether a transaction of the connection waits.
    pub fn waits_for(&self, conn: u64) -> bool {
        self.all().any(|p| p.conn == conn)
    }

    /// The image files that the waiting transactions name, which the cache
    /// keeps for them.
    fn images(&self) -> HashSet<&str> {
        self.all()
            .flat_map(|p| &p.commands)
            .filter_map(|c| match c {
                Command::Create { props, .. }
                | Command::Update { props, .. }
                | Command::Animate { props, .. } => props.image.as_deref(),
                _ => None,
            })
            .collect()
    }

    /// Moves the transactions whose images are in to the queue, each
    /// connection in order.
    fn promote(&mut self) {
        let mut blocked = HashSet::new();
        let mut i = 0;
        while i < self.staged.len() {
            let pending = &self.staged[i];
            if pending.decoding == 0 && !blocked.contains(&pending.conn) {
                self.queue.push_back(self.staged.remove(i));
            } else {
                blocked.insert(pending.conn);
                i += 1;
            }
        }
    }

    /// Drops the transactions of a closed connection. Returns the surfaces
    /// whose commits they held, for `release`.
    pub fn drop_conn(&mut self, conn: u64) -> Vec<WlSurface> {
        self.staged.retain(|p| p.conn != conn);
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
        let waiting = self.transactions.all().filter(|p| p.conn == conn).count();
        if waiting >= MAX_PENDING {
            let error = rejected("too many waiting transactions");
            self.ipc.respond(conn, &Message::error(id, error));
            return;
        }
        let ticket = self.transactions.next_ticket;
        self.transactions.next_ticket += 1;
        let (decoding, failed) = match self.assets.images.decode_for(ticket, &commands) {
            Ok(n) => (n, None),
            Err(error) => (0, Some(error)),
        };
        let pending = Pending {
            conn,
            id,
            owner,
            commands,
            ticket,
            decoding,
            failed,
            waiting: HashMap::new(),
            deadline: None,
            barrier: Barrier::new(false),
            held: Vec::new(),
        };
        if decoding > 0 || self.transactions.staged.iter().any(|p| p.conn == conn) {
            self.transactions.staged.push(pending);
        } else {
            self.transactions.queue.push_back(pending);
            self.complete_transactions();
        }
    }

    /// Puts an image that the worker decoded into the cache, and lets its
    /// transaction go on.
    pub fn decoded(&mut self, decoded: Decoded) {
        let ticket = decoded.ticket;
        let Some(pending) = self
            .transactions
            .staged
            .iter_mut()
            .find(|p| p.ticket == ticket)
        else {
            // The connection closed.
            return;
        };
        pending.decoding -= 1;
        if let Err(error) = self.assets.images.insert(decoded) {
            pending.failed.get_or_insert(error);
        }
        self.transactions.promote();
        self.complete_transactions();
    }

    /// Drops the images that no node shows and no waiting transaction
    /// names.
    fn prune_images(&mut self) {
        let named = self.transactions.images();
        self.assets.images.prune(&self.scene, &named);
    }

    /// Starts and completes the transactions at the head of the queue, as
    /// far as their clients allow.
    pub fn complete_transactions(&mut self) {
        while let Some(mut head) = self.transactions.queue.pop_front() {
            if head.deadline.is_none()
                && let Err(error) = self.start(&mut head)
            {
                self.prune_images();
                self.ipc.respond(head.conn, &Message::error(head.id, error));
                continue;
            }
            let due = head.deadline.is_some_and(|d| self.time.now() >= d);
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
    fn start(&mut self, head: &mut Pending) -> Result<(), RpcError> {
        let effects = self
            .scene
            .check(&head.owner, &head.commands)
            .map_err(|e| transaction_error(&e))?;
        if let Some(error) = head.failed.take() {
            return Err(rejected(error));
        }
        self.check_effects(&head.commands).map_err(rejected)?;
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
        let deadline = self.time.now() + DEADLINE;
        head.deadline = Some(deadline);
        if !head.waiting.is_empty() {
            // A missing timer is logged; the clients still end the wait.
            let _ = self.expire_at(deadline);
        }
        Ok(())
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
        let Some(head) = self.transactions.answered(&id, acked) else {
            return;
        };
        head.held.push(surface.clone());
        let complete = head.waiting.is_empty();
        add_blocker(surface, head.barrier.clone());
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
        self.sync_default_cameras();
        let applied = self.scene.apply(&head.owner, &head.commands);
        self.prune_images();
        let message = match applied {
            Ok(effects) => {
                self.execute(effects);
                self.refresh_pointer();
                Message::result(head.id, Value::Null)
            }
            Err(error) => Message::error(head.id, transaction_error(&error)),
        };
        self.ipc.respond(head.conn, &message);
        head.barrier.signal();
        let surfaces = std::mem::take(&mut head.held);
        self.release(&surfaces);
    }
}

/// The answer to a transaction that the scene rejected. A surface that is
/// gone gets its own code, at the check as at the end of a wait, so that a
/// plugin can tell a closed window from its own mistake.
fn transaction_error(error: &TransactionError) -> RpcError {
    let code = if error.is_gone_surface() {
        RpcError::GONE_SURFACE
    } else {
        RpcError::REJECTED
    };
    RpcError::new(code, error.to_string())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::testing::{Client, Core, rect};

    /// A transaction of a connection that waits for the configures of
    /// surfaces.
    fn waiting(conn: u64, id: u64, surfaces: &[(&str, u32)], deadline: Instant) -> Pending {
        Pending {
            conn,
            id: json!(id),
            owner: "wm".into(),
            commands: Vec::new(),
            ticket: id,
            decoding: 0,
            failed: None,
            waiting: surfaces
                .iter()
                .map(|(s, serial)| ((*s).to_owned(), Serial::from(*serial)))
                .collect(),
            deadline: Some(deadline),
            barrier: Barrier::new(false),
            held: Vec::new(),
        }
    }

    /// A core whose first connection `wm` waits for `s1` and `s2`.
    fn blocked() -> (Core, Client) {
        let mut core = Core::new();
        let wm = core.named("wm");
        let deadline = core.state.time.now() + DEADLINE;
        let head = waiting(0, 99, &[("s1", 1), ("s2", 1)], deadline);
        core.state.transactions.queue.push_back(head);
        (core, wm)
    }

    /// Writes a gray PNG file of a size and returns its path.
    fn png(name: &str, side: u32) -> String {
        let path = std::env::temp_dir().join(format!("chora-{}-{name}.png", std::process::id()));
        let file = std::io::BufWriter::new(std::fs::File::create(&path).unwrap());
        let mut encoder = png::Encoder::new(file, side, side);
        encoder.set_color(png::ColorType::Grayscale);
        let mut writer = encoder.write_header().unwrap();
        writer
            .write_image_data(&vec![128; (side * side) as usize])
            .unwrap();
        path.to_str().unwrap().to_owned()
    }

    fn image(id: &str, path: &str) -> Value {
        json!({"commands": [{"create": {"id": id, "kind": "image", "parent": "root",
            "props": {"image": path}}}]})
    }

    /// The responses of a client, once the worker decoded the images and
    /// `n` arrived.
    fn decoded(core: &mut Core, client: &mut Client, n: usize) -> Vec<Value> {
        let mut responses = Vec::new();
        for _ in 0..1000 {
            core.dispatch();
            responses.extend(client.responses());
            if responses.len() >= n {
                return responses;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("no answer after the decode");
    }

    #[test]
    fn a_decode_does_not_stop_other_clients() {
        let mut core = Core::new();
        let mut loader = core.named("loader");
        let mut panel = core.named("panel");
        let path = png("large", 2048);
        let first = loader.send("scene.apply", image("picture", &path));
        let second = loader.send("scene.apply", rect("after"));
        let other = panel.send("scene.apply", rect("p"));
        assert_eq!(panel.answer(&mut core, other), Some(Ok(Value::Null)));
        assert_eq!(loader.responses(), Vec::<Value>::new(), "both wait");
        let responses = decoded(&mut core, &mut loader, 2);
        let ids: Vec<Value> = responses.iter().map(|r| r["id"].clone()).collect();
        assert_eq!(ids, [json!(first), json!(second)], "in order");
        assert!(
            responses.iter().all(|r| r["result"].is_null()),
            "{responses:?}"
        );
        assert_eq!(core.state.assets.images.get(&path).unwrap().size, [2048; 2]);
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn an_image_that_fails_to_decode_rejects_the_transaction() {
        let mut core = Core::new();
        let mut loader = core.named("loader");
        let path = std::env::temp_dir().join(format!("chora-{}-bad.png", std::process::id()));
        std::fs::write(&path, b"no png").unwrap();
        let path = path.to_str().unwrap();
        let id = loader.send("scene.apply", image("picture", path));
        let response = decoded(&mut core, &mut loader, 1).remove(0);
        assert_eq!(response["id"], json!(id));
        let message = response["error"]["message"].as_str().unwrap();
        assert!(message.starts_with(path), "{message}");
        let nodes = loader.ok(&mut core, "scene.query", json!({"selector": "id:picture"}));
        assert_eq!(nodes, json!([]));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn a_transaction_applies_all_or_nothing() {
        let mut core = Core::new();
        let mut wm = core.named("wm");
        let mut commands = rect("a")["commands"].as_array().unwrap().clone();
        commands.extend(rect("a")["commands"].as_array().unwrap().clone());
        let (error_code, _) = wm.fails(&mut core, "scene.apply", json!({"commands": commands}));
        assert_eq!(error_code, RpcError::REJECTED);
        let nodes = wm.ok(&mut core, "scene.query", json!({"selector": "id:a"}));
        assert_eq!(nodes, json!([]));
    }

    #[test]
    fn a_gone_surface_has_its_own_code() {
        let mut core = Core::new();
        let mut wm = core.named("wm");
        let update = json!({"commands": [{"update": {"id": "s9", "props": {"visible": true}}}]});
        let (error_code, _) = wm.fails(&mut core, "scene.apply", update);
        assert_eq!(error_code, RpcError::GONE_SURFACE);
    }

    #[test]
    fn a_commit_answers_only_the_newest_configure() {
        let mut transactions = Transactions::default();
        let deadline = Instant::now() + DEADLINE;
        transactions
            .queue
            .push_back(waiting(0, 1, &[("s1", 5), ("s2", 5)], deadline));
        assert!(transactions.answered("s1", None).is_none());
        assert!(transactions.answered("s1", Some(Serial::from(4))).is_none());
        let head = transactions
            .answered("s1", Some(Serial::from(5)))
            .expect("the head");
        assert_eq!(head.waiting.keys().collect::<Vec<_>>(), ["s2"]);
        assert!(transactions.answered("s3", Some(Serial::from(9))).is_none());
    }

    #[test]
    fn a_late_surface_waits_for_nobody_until_it_catches_up() {
        let mut transactions = Transactions::default();
        transactions.late.insert("s1".into(), Serial::from(3));
        let mut waits = HashMap::new();
        transactions.configured(&mut waits, "s1", Serial::from(7));
        transactions.configured(&mut waits, "s2", Serial::from(8));
        assert_eq!(waits.keys().collect::<Vec<_>>(), ["s2"]);
        transactions.answered("s1", Some(Serial::from(5)));
        assert_eq!(transactions.late.get("s1"), Some(&Serial::from(7)));
        transactions.answered("s1", Some(Serial::from(7)));
        assert!(transactions.late.is_empty());
    }

    #[test]
    fn a_destroyed_surface_holds_no_transaction() {
        let mut transactions = Transactions::default();
        let deadline = Instant::now() + DEADLINE;
        transactions
            .queue
            .push_back(waiting(0, 1, &[("s1", 1)], deadline));
        transactions
            .queue
            .push_back(waiting(0, 2, &[("s1", 2)], deadline));
        transactions.late.insert("s1".into(), Serial::from(1));
        transactions.forget("s1");
        assert!(transactions.late.is_empty());
        assert!(transactions.queue.iter().all(|p| p.waiting.is_empty()));
    }

    #[test]
    fn transactions_wait_behind_the_head_and_answer_in_order() {
        let (mut core, mut wm) = blocked();
        let mut panel = core.named("panel");
        let first = wm.send("scene.apply", rect("a"));
        let second = panel.send("scene.apply", rect("b"));
        let third = wm.send("scene.apply", rect("c"));
        assert_eq!(wm.answer(&mut core, first), None);
        assert_eq!(panel.answer(&mut core, second), None);
        core.state.transactions.forget("s1");
        core.state.transactions.forget("s2");
        core.state.complete_transactions();
        core.dispatch();
        let ids: Vec<Value> = wm.responses().iter().map(|r| r["id"].clone()).collect();
        assert_eq!(ids, [json!(99), json!(first), json!(third)]);
        assert_eq!(panel.answer(&mut core, second), Some(Ok(Value::Null)));
    }

    #[test]
    fn a_missed_deadline_makes_the_surfaces_late() {
        let (mut core, mut wm) = blocked();
        let first = wm.send("scene.apply", rect("a"));
        core.advance(DEADLINE.saturating_sub(Duration::from_millis(1)));
        assert_eq!(wm.answer(&mut core, first), None);
        core.advance(Duration::from_millis(1));
        let ids: Vec<Value> = wm.responses().iter().map(|r| r["id"].clone()).collect();
        assert_eq!(ids, [json!(99), json!(first)]);
        let mut late: Vec<&String> = core.state.transactions.late.keys().collect();
        late.sort();
        assert_eq!(late, ["s1", "s2"]);
    }

    #[test]
    fn a_connection_has_a_limit_of_waiting_transactions() {
        let (mut core, mut wm) = blocked();
        for i in 1..MAX_PENDING {
            wm.send("scene.apply", rect(&format!("r{i}")));
        }
        core.dispatch();
        assert_eq!(wm.responses(), Vec::<Value>::new());
        let (_, message) = wm.fails(&mut core, "scene.apply", rect("over"));
        assert_eq!(message, "too many waiting transactions");
        let mut panel = core.named("panel");
        let id = panel.send("scene.apply", rect("p"));
        assert_eq!(
            panel.answer(&mut core, id),
            None,
            "the queue is the same for all"
        );
    }

    #[test]
    fn a_closed_connection_lets_the_next_transaction_go() {
        let (mut core, _wm) = blocked();
        let mut panel = core.named("panel");
        let id = panel.send("scene.apply", rect("p"));
        assert_eq!(panel.answer(&mut core, id), None);
        assert!(core.state.transactions.waits_for(0));
        core.state.ipc_close(0);
        assert!(!core.state.transactions.waits_for(0));
        assert_eq!(panel.answer(&mut core, id), Some(Ok(Value::Null)));
    }

    #[test]
    fn the_end_of_the_stream_waits_for_the_transactions_of_the_connection() {
        let (mut core, mut wm) = blocked();
        let id = wm.send("scene.apply", rect("a"));
        wm.shut_down_writes();
        assert!(!wm.closed(&mut core));
        core.advance(DEADLINE);
        let ids: Vec<Value> = wm.responses().iter().map(|r| r["id"].clone()).collect();
        assert_eq!(ids, [json!(99), json!(id)]);
        assert!(wm.closed(&mut core));
    }
}
