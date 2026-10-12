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
    collections::{BTreeSet, HashMap},
    time::Duration,
};

use chora_scene::{
    Event,
    ipc::{Export, Message, MethodCall, MethodReturn, RpcError},
};
use serde_json::Value;
use smithay::reexports::calloop::{
    RegistrationToken,
    timer::{TimeoutAction, Timer},
};

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
/// The longest method name.
const MAX_NAME: usize = 64;

/// The exported methods and the calls that wait for their answers.
#[derive(Default)]
pub struct Methods {
    exports: HashMap<u64, BTreeSet<String>>,
    pending: HashMap<u64, Pending>,
    next: u64,
}

struct Pending {
    caller: u64,
    /// The id of the `methods.call` request.
    id: Value,
    callee: u64,
    timer: Option<RegistrationToken>,
}

impl Methods {
    /// Whether a call of the connection waits for its answer.
    pub fn waits_for(&self, conn: u64) -> bool {
        self.pending.values().any(|p| p.caller == conn)
    }
}

/// Whether a method name is valid: lowercase letters, digits, `_`, `-` and
/// `.`, starting with a letter.
fn valid_name(name: &str) -> bool {
    name.len() <= MAX_NAME
        && name.starts_with(|c: char| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "_-.".contains(c))
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
        if let Some(bad) = export.names.iter().find(|n| !valid_name(n)) {
            return Err(rejected(format!(
                "invalid method name {bad:?}: use a-z, 0-9, _, - and ."
            )));
        }
        let name = self.ipc.name(conn).unwrap_or_default().to_owned();
        let methods = &self.methods;
        let taken = export.names.iter().find(|m| {
            methods.exports.iter().any(|(other, names)| {
                *other != conn && self.ipc.name(*other) == Some(&name) && names.contains(*m)
            })
        });
        if let Some(taken) = taken {
            return Err(rejected(format!(
                "{name}.{taken} is exported by another connection"
            )));
        }
        let names = self.methods.exports.entry(conn).or_default();
        names.extend(export.names);
        if names.len() > MAX_EXPORTS {
            let names: Vec<_> = names.iter().take(MAX_EXPORTS).cloned().collect();
            self.methods
                .exports
                .insert(conn, names.into_iter().collect());
            return Err(rejected(format!(
                "a connection exports at most {MAX_EXPORTS} methods"
            )));
        }
        Ok(Value::Null)
    }

    /// The connection that exported a method of a client.
    fn exporter(&self, client: &str, method: &str) -> Option<u64> {
        self.methods
            .exports
            .iter()
            .find(|(conn, names)| self.ipc.name(**conn) == Some(client) && names.contains(method))
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
            return Err(rejected("too many waiting calls"));
        }
        let number = self.methods.next;
        self.methods.next += 1;
        let timer = self
            .loop_handle
            .insert_source(Timer::from_duration(TIMEOUT), move |_, (), state| {
                if let Some(pending) = state.methods.pending.get_mut(&number) {
                    pending.timer = None;
                }
                state.end_call(number, Err(rejected("the method gave no answer in time")));
                TimeoutAction::Drop
            })
            .ok();
        let from = self.ipc.name(conn).unwrap_or_default().to_owned();
        self.methods.pending.insert(
            number,
            Pending {
                caller: conn,
                id,
                callee,
                timer,
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
    fn end_call(&mut self, number: u64, result: Result<Value, RpcError>) {
        let Some(pending) = self.methods.pending.remove(&number) else {
            return;
        };
        if let Some(timer) = pending.timer {
            self.loop_handle.remove(timer);
        }
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
                methods.extend(names.iter().map(String::as_str));
            }
        }
        let list: Vec<Value> = clients
            .into_iter()
            .map(|(name, methods)| serde_json::json!({"name": name, "methods": methods}))
            .collect();
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
    use super::*;

    #[test]
    fn method_names_are_plain() {
        assert!(valid_name("set_layout"));
        assert!(valid_name("layout.set-gap2"));
        assert!(!valid_name("Set"));
        assert!(!valid_name("2go"));
        assert!(!valid_name("a b"));
        assert!(!valid_name(""));
        assert!(!valid_name(&"a".repeat(MAX_NAME + 1)));
    }
}
