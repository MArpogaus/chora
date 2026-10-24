//! Methods that plugins offer to other clients (D31), for example
//! `tiling.set_layout`. An agent then asks the window manager for a change
//! instead of fighting it.
//!
//! A connection exports method names. A caller names the client and the
//! method; the core sends the exporting connection a `method.called` event
//! with a call id and answers the caller when the exporter returns, after
//! `methods.return` with that id. A call that gets no answer within
//! `TIMEOUT` fails, and so does a call whose exporter disconnects.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    time::{Duration, Instant},
};

pub(crate) use chora_scene::ipc::valid_name;
use chora_scene::{
    Event,
    ipc::{Export, ExportMethod, Message, MethodCall, MethodReturn, RpcError},
};
use serde_json::Value;

use crate::{
    ipc::{params_of, rejected},
    state::State,
};

/// How long a caller waits for the answer.
const TIMEOUT: Duration = Duration::from_secs(10);
/// How many methods a connection can export.
const MAX_EXPORTS: usize = 64;
/// How many calls of one connection can wait for their answers.
const MAX_PENDING: usize = 64;

/// The exported methods and the calls that wait for their answers.
#[derive(Default)]
pub struct Methods {
    exports: HashMap<u64, BTreeMap<String, ExportMethod>>,
    pending: HashMap<u64, Pending>,
    next: u64,
}

struct Pending {
    caller: u64,
    /// The id of the `methods.call` request.
    id: Value,
    callee: u64,
    deadline: Instant,
}

impl Methods {
    /// Whether a call of the connection waits for its answer.
    pub fn waits_for(&self, conn: u64) -> bool {
        self.pending.values().any(|p| p.caller == conn)
    }

    /// The calls whose deadline passed.
    pub fn due(&self, now: Instant) -> Vec<u64> {
        let due = self.pending.iter().filter(|(_, p)| p.deadline <= now);
        due.map(|(number, _)| *number).collect()
    }
}

impl State {
    /// `methods.export`, `methods.call` and `methods.return`. `None` means
    /// that the answer of a call comes later.
    pub fn ipc_methods(
        &mut self,
        conn: u64,
        id: Value,
        method: &str,
        params: Value,
    ) -> Option<Result<Value, RpcError>> {
        if self.ipc.name(conn).is_none() {
            return Some(Err(rejected("send hello first")));
        }
        Some(match method {
            "methods.export" => params_of(params).and_then(|e| self.export(conn, e)),
            "methods.call" => {
                let call = match params_of(params) {
                    Ok(c) => c,
                    Err(e) => return Some(Err(e)),
                };
                return self.call_method(conn, id, call).err().map(Err);
            }
            _ => params_of(params).and_then(|r| self.return_method(conn, r)),
        })
    }

    fn export(&mut self, conn: u64, export: Export) -> Result<Value, RpcError> {
        if let Some(bad) = export.names.iter().find(|m| !valid_name(m.name())) {
            return Err(rejected(format!(
                "invalid method name {:?}: use a-z, 0-9, _, - and .",
                bad.name()
            )));
        }
        let name = self.ipc.name(conn).unwrap_or_default().to_owned();
        let taken = export.names.iter().find(|m| {
            self.methods.exports.iter().any(|(other, known)| {
                *other != conn
                    && self.ipc.name(*other) == Some(&name)
                    && known.contains_key(m.name())
            })
        });
        if let Some(taken) = taken {
            return Err(rejected(format!(
                "{name}.{} is exported by another connection",
                taken.name()
            )));
        }
        let known = self.methods.exports.entry(conn).or_default();
        let new = export
            .names
            .iter()
            .filter(|m| !known.contains_key(m.name()))
            .count();
        if known.len() + new > MAX_EXPORTS {
            return Err(RpcError::new(
                RpcError::LIMIT_REACHED,
                format!("a connection exports at most {MAX_EXPORTS} methods"),
            ));
        }
        known.extend(export.names.into_iter().map(|m| (m.name().to_owned(), m)));
        Ok(Value::Null)
    }

    /// The connection that exported a method of a client.
    fn exporter(&self, client: &str, method: &str) -> Option<u64> {
        self.methods
            .exports
            .iter()
            .find(|(conn, names)| {
                self.ipc.name(**conn) == Some(client) && names.contains_key(method)
            })
            .map(|(conn, _)| *conn)
    }

    /// Sends the call to the exporter. The answer follows later.
    fn call_method(&mut self, conn: u64, id: Value, call: MethodCall) -> Result<(), RpcError> {
        let Some(callee) = self.exporter(&call.client, &call.method) else {
            return Err(rejected(format!(
                "{}.{} is no exported method",
                call.client, call.method
            )));
        };
        if callee == conn {
            return Err(rejected("a connection cannot call its own methods"));
        }
        let waiting = self
            .methods
            .pending
            .values()
            .filter(|p| p.caller == conn)
            .count();
        if waiting >= MAX_PENDING {
            return Err(RpcError::new(
                RpcError::LIMIT_REACHED,
                "too many waiting calls",
            ));
        }
        let number = self.methods.next;
        self.methods.next += 1;
        let deadline = self.time.now() + TIMEOUT;
        self.expire_at(deadline).map_err(rejected)?;
        let from = self.ipc.name(conn).unwrap_or_default().to_owned();
        self.methods.pending.insert(
            number,
            Pending {
                caller: conn,
                id,
                callee,
                deadline,
            },
        );
        let event = Event::MethodCalled {
            call: number,
            caller: from,
            method: call.method,
            params: call.params,
        };
        self.ipc.send_to(callee, &event);
        Ok(())
    }

    fn return_method(&mut self, conn: u64, answer: MethodReturn) -> Result<Value, RpcError> {
        let mine = self
            .methods
            .pending
            .get(&answer.call)
            .is_some_and(|p| p.callee == conn);
        if !mine {
            return Err(rejected(format!(
                "no call {} waits for this connection",
                answer.call
            )));
        }
        let result = answer.error.map_or(Ok(answer.result), Err);
        self.end_call(answer.call, result);
        Ok(Value::Null)
    }

    /// Answers the caller of a call and forgets the call.
    pub fn end_call(&mut self, number: u64, result: Result<Value, RpcError>) {
        let Some(pending) = self.methods.pending.remove(&number) else {
            return;
        };
        let message = match result {
            Ok(value) => Message::result(pending.id, value),
            Err(error) => Message::error(pending.id, error),
        };
        self.ipc.respond(pending.caller, &message);
    }

    /// The client names that are connected, with the methods that each
    /// exported, sorted by name.
    pub fn clients(&self) -> Value {
        let mut clients: std::collections::BTreeMap<&str, BTreeSet<&str>> =
            std::collections::BTreeMap::new();
        for conn in self.ipc.conn_ids() {
            let Some(name) = self.ipc.name(conn) else {
                continue;
            };
            let methods = clients.entry(name).or_default();
            if let Some(names) = self.methods.exports.get(&conn) {
                methods.extend(names.keys().map(String::as_str));
            }
        }
        let list: Vec<Value> = clients
            .into_iter()
            .map(|(name, methods)| serde_json::json!({"name": name, "methods": methods}))
            .collect();
        Value::Array(list)
    }

    /// `methods.list`: every exported method with its description and
    /// input schema, by connection and sorted by name. Plain exports
    /// answer with `null` for both.
    pub fn list_methods(&self) -> Value {
        let mut list: Vec<Value> = Vec::new();
        for conn in self.ipc.conn_ids() {
            let Some(client) = self.ipc.name(conn) else {
                continue;
            };
            let Some(methods) = self.methods.exports.get(&conn) else {
                continue;
            };
            for method in methods.values() {
                list.push(serde_json::json!({
                    "client": client,
                    "name": method.name(),
                    "description": method.description(),
                    "input_schema": method.input_schema(),
                }));
            }
        }
        Value::Array(list)
    }

    /// Forgets the exports and the calls of a closed connection. Its
    /// callers get an error.
    pub fn methods_closed(&mut self, conn: u64) {
        self.methods.exports.remove(&conn);
        let ended: Vec<u64> = self
            .methods
            .pending
            .iter()
            .filter(|(_, p)| p.callee == conn || p.caller == conn)
            .map(|(n, _)| *n)
            .collect();
        for number in ended {
            self.end_call(
                number,
                Err(rejected("the client of the method disconnected")),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::testing::{Client, Core};

    /// A core with an exporter `wm` of `set_layout` and a caller.
    fn exported() -> (Core, Client, Client) {
        let mut core = Core::new();
        let mut wm = core.named("wm");
        wm.ok(
            &mut core,
            "methods.export",
            json!({"names": ["set_layout"]}),
        );
        let caller = core.named("agent");
        (core, wm, caller)
    }

    fn call(caller: &mut Client, method: &str) -> u64 {
        let call = json!({"client": "wm", "method": method, "params": {"name": "max"}});
        caller.send("methods.call", call)
    }

    #[test]
    fn method_names_are_plain() {
        assert!(valid_name("set_layout"));
        assert!(valid_name("layout.set-gap2"));
        assert!(!valid_name("Set"));
        assert!(!valid_name("2go"));
        assert!(!valid_name("a b"));
        assert!(!valid_name(""));
        assert!(!valid_name(&"a".repeat(chora_scene::ipc::MAX_NAME + 1)));
    }

    #[test]
    fn a_call_reaches_the_exporter_and_its_answer_the_caller() {
        let (mut core, mut wm, mut caller) = exported();
        let id = call(&mut caller, "set_layout");
        assert_eq!(caller.answer(&mut core, id), None, "the answer waits");
        let event = wm.events_now(&mut core);
        assert_eq!(event.len(), 1);
        let params = &event[0]["params"];
        assert_eq!(params["event"], "method.called");
        assert_eq!(params["caller"], "agent");
        assert_eq!(params["method"], "set_layout");
        assert_eq!(params["params"], json!({"name": "max"}));
        let answer = json!({"call": params["call"], "result": "done"});
        wm.ok(&mut core, "methods.return", answer);
        assert_eq!(caller.answer(&mut core, id), Some(Ok(json!("done"))));
    }

    /// The sequence numbers count every event of a connection, also the
    /// events that arrive before it subscribes (D32).
    #[test]
    fn the_sequence_numbers_every_event_of_a_connection() {
        let (mut core, mut wm, mut caller) = exported();
        call(&mut caller, "set_layout");
        assert_eq!(wm.events_now(&mut core)[0]["params"]["seq"], 1);
        wm.ok(&mut core, "events.subscribe", json!({"events": []}));
        call(&mut caller, "set_layout");
        assert_eq!(wm.events_now(&mut core)[0]["params"]["seq"], 2);
    }

    #[test]
    fn an_error_of_the_exporter_reaches_the_caller() {
        let (mut core, mut wm, mut caller) = exported();
        let id = call(&mut caller, "set_layout");
        let number = wm.events_now(&mut core)[0]["params"]["call"].clone();
        let error = json!({"code": 7, "message": "no such layout"});
        wm.ok(
            &mut core,
            "methods.return",
            json!({"call": number, "error": error}),
        );
        let answer = caller.answer(&mut core, id).expect("an answer");
        assert_eq!(answer, Err(error));
    }

    #[test]
    fn a_call_without_an_answer_fails_at_its_deadline() {
        let (mut core, mut wm, mut caller) = exported();
        let id = call(&mut caller, "set_layout");
        let number = wm.events_now(&mut core)[0]["params"]["call"].clone();
        core.advance(TIMEOUT.saturating_sub(Duration::from_millis(1)));
        assert_eq!(caller.answer(&mut core, id), None);
        core.advance(Duration::from_millis(1));
        let answer = caller.answer(&mut core, id).expect("an answer");
        assert_eq!(
            answer.unwrap_err()["message"],
            "the method gave no answer in time"
        );
        let late = json!({"call": number, "result": "late"});
        let (_, message) = wm.fails(&mut core, "methods.return", late);
        assert_eq!(message, "no call 0 waits for this connection");
    }

    #[test]
    fn the_callers_get_an_error_when_the_exporter_closes() {
        let (mut core, wm, mut caller) = exported();
        let id = call(&mut caller, "set_layout");
        core.dispatch();
        drop(wm);
        let answer = caller.answer(&mut core, id).expect("an answer");
        assert_eq!(
            answer.unwrap_err()["message"],
            "the client of the method disconnected"
        );
        let (_, message) = caller.fails(
            &mut core,
            "methods.call",
            json!({"client": "wm", "method": "set_layout"}),
        );
        assert_eq!(message, "wm.set_layout is no exported method");
    }

    #[test]
    fn only_the_exporter_returns_a_call_and_nobody_calls_itself() {
        let (mut core, mut wm, mut caller) = exported();
        let id = call(&mut caller, "set_layout");
        let number = wm.events_now(&mut core)[0]["params"]["call"].clone();
        let answer = json!({"call": number, "result": "forged"});
        let (_, message) = caller.fails(&mut core, "methods.return", answer);
        assert_eq!(message, "no call 0 waits for this connection");
        assert_eq!(caller.answer(&mut core, id), None);
        let own = json!({"client": "wm", "method": "set_layout"});
        let (_, message) = wm.fails(&mut core, "methods.call", own);
        assert_eq!(message, "a connection cannot call its own methods");
    }

    #[test]
    fn a_method_of_a_name_has_one_exporter() {
        let (mut core, mut wm, _caller) = exported();
        let mut second = core.named("wm");
        let names = json!({"names": ["set_layout"]});
        let (_, message) = second.fails(&mut core, "methods.export", names.clone());
        assert_eq!(message, "wm.set_layout is exported by another connection");
        wm.ok(&mut core, "methods.export", names);
        let mut other = core.named("panel");
        other.ok(
            &mut core,
            "methods.export",
            json!({"names": ["set_layout"]}),
        );
        let clients = other.ok(&mut core, "clients.list", Value::Null);
        assert_eq!(
            clients,
            json!([
                {"name": "agent", "methods": []},
                {"name": "panel", "methods": ["set_layout"]},
                {"name": "wm", "methods": ["set_layout"]},
            ])
        );
    }

    /// `methods.list` answers the exports with description and schema
    /// (T5), and a plain export without a schema still works.
    #[test]
    fn listed_methods_carry_description_and_schema() {
        let mut core = Core::new();
        let mut wm = core.named("wm");
        let schema = json!({"type": "object", "properties": {"gap": {"type": "integer"}}});
        let export = json!({"names": [
            "restart",
            {"name": "set_layout", "description": "Sets the layout.", "input_schema": schema},
        ]});
        wm.ok(&mut core, "methods.export", export);
        let mut other = core.named("agent");
        let listed = other.ok(&mut core, "methods.list", Value::Null);
        assert_eq!(
            listed,
            json!([
                {"client": "wm", "name": "restart", "description": null, "input_schema": null},
                {
                    "client": "wm",
                    "name": "set_layout",
                    "description": "Sets the layout.",
                    "input_schema": schema,
                },
            ])
        );
        let id = other.send("methods.call", json!({"client": "wm", "method": "restart"}));
        core.dispatch();
        assert_eq!(wm.events_now(&mut core)[0]["params"]["method"], "restart");
        assert_eq!(other.answer(&mut core, id), None, "the answer waits");
        let (_, message) = wm.fails(
            &mut core,
            "methods.export",
            json!({"names": [{"name": "Bad", "description": "d"}]}),
        );
        assert!(
            message.starts_with("invalid method name \"Bad\""),
            "{message}"
        );
        let (_, message) = wm.fails(
            &mut core,
            "methods.export",
            json!({"names": [{"description": "no name"}]}),
        );
        assert!(message.contains("variant"), "{message}");
        let (_, message) = wm.fails(
            &mut core,
            "methods.export",
            json!({"names": [{"name": "x", "desc": "typo for description"}]}),
        );
        assert!(message.contains("variant"), "{message}");
    }

    #[test]
    fn exports_and_waiting_calls_have_limits() {
        let mut core = Core::new();
        let mut wm = core.named("wm");
        let names: Vec<String> = (0..MAX_EXPORTS).map(|i| format!("m{i}")).collect();
        wm.ok(&mut core, "methods.export", json!({"names": names}));
        let (error_code, message) =
            wm.fails(&mut core, "methods.export", json!({"names": ["one_more"]}));
        assert_eq!(error_code, RpcError::LIMIT_REACHED);
        assert_eq!(message, "a connection exports at most 64 methods");
        let (_, message) = wm.fails(&mut core, "methods.export", json!({"names": ["Bad"]}));
        assert!(message.starts_with("invalid method name"));
        let mut caller = core.named("agent");
        for _ in 0..MAX_PENDING {
            call(&mut caller, "m0");
        }
        core.dispatch();
        let (error_code, message) = caller.fails(
            &mut core,
            "methods.call",
            json!({"client": "wm", "method": "m0"}),
        );
        assert_eq!(error_code, RpcError::LIMIT_REACHED);
        assert_eq!(message, "too many waiting calls");
    }
}
