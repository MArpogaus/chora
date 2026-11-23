//! The `events.subscribe` and `events.emit` calls of a connection.

use std::time::Duration;

use chora_scene::{
    Event,
    ipc::{Emit, Query, RpcError, Subscribe},
};
use serde_json::Value;

use super::{State, rejected};

/// The most bytes of the data of one emitted event (D65).
const MAX_EMIT: usize = 64 << 10;
/// The most events that one client name emits per `EMIT_WINDOW` (D65).
const MAX_EMITS: u32 = 32;
/// The events that every client may emit, whatever its name: the looks
/// name their theme and their view to whoever draws for them. The core
/// puts the emitter in `data.look`.
const LOOK_EVENTS: &[&str] = &["look.theme", "look.view"];
const EMIT_WINDOW: Duration = Duration::from_secs(1);

/// Whether the first part of a name is that of core events, such as
/// `surface` or `output`.
fn core_namespace(name: &str) -> bool {
    let first = |n: &str| n.split('.').next().unwrap_or_default().to_owned();
    let head = first(name);
    chora_scene::EVENT_NAMES.iter().any(|e| first(e) == head)
}

/// Whether `events.subscribe` takes a name: a core event, the event of a
/// client, `NAME.something`, or a pattern `PREFIX.*` (D65). A typo in the
/// name of a core event is refused.
fn subscribable(name: &str) -> bool {
    use chora_scene::ipc::valid_name;
    if chora_scene::EVENT_NAMES.contains(&name) {
        return true;
    }
    if let Some(prefix) = name.strip_suffix(".*") {
        return valid_name(prefix);
    }
    !core_namespace(name)
        && (name.split_once('.')).is_some_and(|(head, rest)| valid_name(head) && valid_name(rest))
}

/// The data of a shared event of the looks with the field `look` set to
/// the emitter; the data is an object or null.
fn stamped_look(mut data: Value, event: &str, client: &str) -> Result<Value, RpcError> {
    if data.is_null() {
        data = Value::Object(serde_json::Map::new());
    }
    let Some(fields) = data.as_object_mut() else {
        return Err(rejected(format!("the data of {event} is an object")));
    };
    fields.insert("look".into(), Value::from(client));
    Ok(data)
}

impl State {
    /// Streams the named events, or all of them for an empty list. An
    /// unknown name is a mistake that answers, not silence. The answer is
    /// a snapshot of the scene as `scene.query` gives it (D32), so that no
    /// change between the snapshot and the first event is lost.
    pub(super) fn ipc_subscribe(
        &mut self,
        conn: u64,
        subscription: Subscribe,
    ) -> Result<Value, RpcError> {
        if let Some(unknown) = subscription.events.iter().find(|n| !subscribable(n)) {
            return Err(rejected(format!("there is no event {unknown:?}")));
        }
        if let Some(c) = self.ipc.conns.get_mut(&conn) {
            c.events = Some(subscription.events);
        }
        self.query(conn, &Query::new(""))
    }

    /// Sends an event of the client to the connections that subscribed to
    /// its name or a pattern of it (D65). The name starts with the client
    /// name and a dot, so no client sends a core event or one of another
    /// name; the exception is `LOOK_EVENTS`, whose data gets the field `look`
    /// with the client name. The data has at most `MAX_EMIT` bytes, and a name emits at most
    /// `MAX_EMITS` events per second; the core refuses the rest with code 6.
    pub(super) fn ipc_emit(&mut self, conn: u64, emit: Emit) -> Result<Value, RpcError> {
        let client = self
            .ipc
            .name(conn)
            .ok_or_else(|| rejected("send hello first"))?
            .to_owned();
        if core_namespace(&client) {
            return Err(rejected(format!(
                "the events of {client:?} would look like core events; \
                 emit as a client whose name starts otherwise"
            )));
        }
        let shared = LOOK_EVENTS.contains(&emit.event.as_str());
        let own = shared
            || (emit.event.strip_prefix(client.as_str()))
                .and_then(|rest| rest.strip_prefix('.'))
                .is_some_and(chora_scene::ipc::valid_name);
        if !own {
            return Err(rejected(format!(
                "an event of {client:?} is named {client}.NAME, with NAME in lowercase \
                 letters, digits, _, - and ., from a letter, not {:?}",
                emit.event
            )));
        }
        let data = if shared {
            stamped_look(emit.data, &emit.event, &client)?
        } else {
            emit.data
        };
        let size = serde_json::to_vec(&data).map_or(usize::MAX, |d| d.len());
        if size > MAX_EMIT {
            return Err(RpcError::new(
                RpcError::LIMIT_REACHED,
                format!("the data has {size} bytes, more than {MAX_EMIT}"),
            ));
        }
        let now = self.time.now();
        let emits = &mut self.ipc.emits;
        emits.retain(|_, (start, _)| now.duration_since(*start) < EMIT_WINDOW);
        let (_, count) = emits.entry(client.clone()).or_insert((now, 0));
        if *count >= MAX_EMITS {
            return Err(RpcError::new(
                RpcError::LIMIT_REACHED,
                format!(
                    "{client:?} emitted {MAX_EMITS} events in this second; the core drops \
                     this one, so send only the last state of a burst"
                ),
            ));
        }
        *count += 1;
        let event = Event::Emitted {
            event: emit.event,
            client,
            data,
        };
        self.ipc.send_subscribed(None, &event, |_| None);
        Ok(Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::testing::{Core, rect};

    /// A typo in a name is an answer, not silence until the end of time.
    #[test]
    fn subscribe_wants_known_event_names() {
        let mut core = Core::new();
        let mut client = core.named("client");
        client.ok(&mut core, "events.subscribe", json!({"events": []}));
        client.ok(
            &mut core,
            "events.subscribe",
            json!({"events": ["surface.created", "output.changed"]}),
        );
        let (_, message) = client.fails(
            &mut core,
            "events.subscribe",
            json!({"events": ["surface.created", "surface.creatd"]}),
        );
        assert!(message.contains("no event \"surface.creatd\""), "{message}");
        // Outside the core names, a name is the event of a client (D65).
        let clients = json!({"events": ["voice.state", "voice.*", "surface.*"]});
        client.ok(&mut core, "events.subscribe", clients);
        for bad in ["voice", "Voice.state", ".*", "output.state"] {
            let events = json!({"events": [bad]});
            client.fails(&mut core, "events.subscribe", events);
        }
    }

    /// D65: a client emits only under its own name, and the core events
    /// stay the core's.
    #[test]
    fn a_client_emits_events_of_its_own_name_only() {
        let mut core = Core::new();
        let mut voice = core.named("voice");
        let state = json!({"event": "voice.state", "data": {"state": "listening"}});
        voice.ok(&mut core, "events.emit", state);
        for event in ["mc.state", "voice", "voice.", "voicex.state", "voice.State"] {
            let (error_code, message) =
                voice.fails(&mut core, "events.emit", json!({"event": event}));
            assert_eq!(error_code, RpcError::REJECTED, "{event}: {message}");
        }
        let mut impostor = core.named("surface");
        let created = json!({"event": "surface.created", "data": {"surface": "s1"}});
        let (_, message) = impostor.fails(&mut core, "events.emit", created);
        assert!(message.contains("core events"), "{message}");
        let mut silent = core.connect();
        silent.fails(&mut core, "events.emit", json!({"event": "voice.state"}));
    }

    /// D65: an emitted event reaches the connections that named it or a
    /// pattern of it, not those of every core event; it carries the
    /// emitter and the data.
    #[test]
    fn an_emitted_event_reaches_its_subscribers_only() {
        let mut core = Core::new();
        let mut exact = core.named("exact");
        exact.ok(
            &mut core,
            "events.subscribe",
            json!({"events": ["voice.state"]}),
        );
        let mut pattern = core.named("pattern");
        pattern.ok(
            &mut core,
            "events.subscribe",
            json!({"events": ["voice.*"]}),
        );
        let mut all = core.named("all");
        all.ok(&mut core, "events.subscribe", json!({"events": []}));
        let mut other = core.named("other");
        other.ok(&mut core, "events.subscribe", json!({"events": ["mc.*"]}));
        let mut voice = core.named("voice");
        let state = json!({"event": "voice.state", "data": {"state": "speaking"}});
        voice.ok(&mut core, "events.emit", state);
        voice.ok(&mut core, "events.emit", json!({"event": "voice.say"}));
        let got = exact.events_now(&mut core);
        assert_eq!(got.len(), 1, "{got:?}");
        let params = &got[0]["params"];
        assert_eq!(params["event"], "voice.state");
        assert_eq!(params["client"], "voice");
        assert_eq!(params["data"]["state"], "speaking");
        assert!(params["seq"].is_u64());
        assert_eq!(pattern.event_names(&mut core), ["voice.state", "voice.say"]);
        let core_only = all.event_names(&mut core);
        assert!(
            core_only.iter().all(|e| e == "client.connected"),
            "{core_only:?}"
        );
        assert_eq!(other.event_names(&mut core), Vec::<String>::new());
    }

    /// The shared events of the looks: any client emits `look.theme` and
    /// `look.view`, and the core names the emitter in `data.look`, so a
    /// fifth look needs no change in whoever listens for `look.*`. No
    /// other name of the namespace goes through.
    #[test]
    fn a_look_names_itself_with_the_shared_look_events() {
        let mut core = Core::new();
        let mut host = core.named("host");
        host.ok(&mut core, "events.subscribe", json!({"events": ["look.*"]}));
        let mut fifth = core.named("fifth");
        let theme = json!({"event": "look.theme", "data": {"theme": "dark", "look": "mc"}});
        fifth.ok(&mut core, "events.emit", theme);
        let view = json!({"event": "look.view", "data": {"pane": true}});
        fifth.ok(&mut core, "events.emit", view);
        let got = host.events_now(&mut core);
        assert_eq!(got.len(), 2, "{got:?}");
        assert_eq!(got[0]["params"]["event"], "look.theme");
        assert_eq!(got[0]["params"]["client"], "fifth");
        assert_eq!(got[0]["params"]["data"]["look"], "fifth", "no spoofed look");
        assert_eq!(got[0]["params"]["data"]["theme"], "dark");
        assert_eq!(got[1]["params"]["data"]["look"], "fifth");
        assert_eq!(got[1]["params"]["data"]["pane"], true);
        let (error_code, _) = fifth.fails(&mut core, "events.emit", json!({"event": "look.other"}));
        assert_eq!(error_code, RpcError::REJECTED);
    }

    /// D65: the data has at most 64 KiB, and a name emits at most 32
    /// events per second; the rest is refused with code 6.
    #[test]
    fn emitted_events_have_a_size_and_a_rate_limit() {
        let mut core = Core::new();
        core.state.time.stop();
        let mut watcher = core.named("watcher");
        watcher.ok(
            &mut core,
            "events.subscribe",
            json!({"events": ["voice.*"]}),
        );
        let mut voice = core.named("voice");
        let big = json!({"event": "voice.big", "data": "x".repeat(MAX_EMIT)});
        // The line is longer than one read of the core.
        let id = voice.send("events.emit", big);
        let answer = std::iter::repeat_with(|| voice.answer(&mut core, id))
            .flatten()
            .next()
            .expect("an answer");
        assert_eq!(answer.unwrap_err()["code"], RpcError::LIMIT_REACHED);
        for _ in 0..MAX_EMITS {
            voice.ok(&mut core, "events.emit", json!({"event": "voice.tick"}));
        }
        let mut twin = core.named("voice");
        let tick = json!({"event": "voice.tick"});
        let (error_code, message) = twin.fails(&mut core, "events.emit", tick.clone());
        assert_eq!(error_code, RpcError::LIMIT_REACHED, "the limit is per name");
        assert!(message.contains("last state"), "{message}");
        core.advance(EMIT_WINDOW);
        voice.ok(&mut core, "events.emit", tick);
        let ticks = watcher.event_names(&mut core);
        assert_eq!(
            ticks.len(),
            MAX_EMITS as usize + 1,
            "refused events never arrive"
        );
    }

    /// Gate check 5 of 0.8: a subscription that starts while transactions
    /// run misses no event and sees no event twice.
    #[test]
    fn a_subscription_answers_with_a_snapshot_and_numbers_its_events() {
        const TOTAL: usize = 1000;
        const BEFORE: usize = 512;
        let mut core = Core::new();
        core.add_output("DP-1", (800, 600));
        let mut watcher = core.named("watcher");
        watcher.ok(&mut core, "scene.apply", rect("kept"));
        // Every camera transaction sends one `output.changed`.
        let camera = json!({"commands": [{"camera": {"output": "DP-1"}}]});
        // The subscription lands between two transactions of a waiting
        // list of `TOTAL`. Batches keep the socket buffers free.
        let mut sub = 0;
        let mut messages: Vec<Value> = Vec::new();
        for sent in 0..TOTAL {
            watcher.send("scene.apply", camera.clone());
            if sent + 1 == BEFORE {
                sub = watcher.send("events.subscribe", json!({"events": ["output.changed"]}));
            }
            if (sent + 1) % 64 == 0 {
                core.dispatch();
                messages.extend(watcher.read().unwrap_or_default());
            }
        }
        loop {
            core.dispatch();
            let more = watcher.read().unwrap_or_default();
            if more.is_empty() {
                break;
            }
            messages.extend(more);
        }
        let events = TOTAL - BEFORE;
        assert_eq!(messages.len(), TOTAL + 1 + events, "all answers and events");
        let at = messages
            .iter()
            .position(|m| m["id"] == sub)
            .expect("the subscribe answer");
        assert!(messages[..at].iter().all(|m| m["method"] != "event"));
        let snapshot = messages[at]["result"].as_array().expect("a snapshot");
        assert!(snapshot.iter().any(|n| n["id"] == "kept"));
        let seqs: Vec<u64> = messages[at..]
            .iter()
            .filter(|m| m["method"] == "event")
            .map(|m| m["params"]["seq"].as_u64().expect("a sequence number"))
            .collect();
        assert_eq!(seqs, (1..=events as u64).collect::<Vec<u64>>());
    }

    #[test]
    fn a_subscription_names_its_events_or_takes_all() {
        let mut core = Core::new();
        let mut some = core.named("some");
        let only = json!({"events": ["client.disconnected"]});
        some.ok(&mut core, "events.subscribe", only);
        let mut quiet = core.named("quiet");
        let mut all = core.named("all");
        all.ok(&mut core, "events.subscribe", json!({"events": []}));
        drop(core.named("bar"));
        assert_eq!(some.event_names(&mut core), ["client.disconnected"]);
        assert_eq!(
            all.event_names(&mut core),
            ["client.connected", "client.disconnected"]
        );
        assert_eq!(quiet.event_names(&mut core), Vec::<String>::new());
    }
}
