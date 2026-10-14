//! Grants of the user for the IPC methods that see or drive the whole
//! session: `input.inject` and `screen.capture` (D33).
//!
//! `choractl grant NAME CAPABILITY` grants a capability to a client name,
//! until `choractl revoke` or the end of the session. A grant protects
//! against mistakes of agents, not against hostile code of the same user:
//! every client of the user can call `grants.set`. The MCP bridge offers no
//! tool for it, so an agent cannot grant itself by accident. The emergency
//! reset revokes all grants.

use std::collections::{BTreeMap, BTreeSet};

use chora_scene::ipc::{Capability, GrantChange, RpcError};
use serde_json::Value;

use crate::{ipc::rejected, state::State};

/// The grants by client name.
#[derive(Default)]
pub struct Grants(BTreeMap<String, BTreeSet<Capability>>);

impl Grants {
    /// Whether a client name holds a capability.
    pub fn allows(&self, client: &str, capability: Capability) -> bool {
        self.0.get(client).is_some_and(|c| c.contains(&capability))
    }

    fn set(&mut self, change: GrantChange) -> Result<(), RpcError> {
        if change.capabilities.is_empty() {
            return Err(rejected("name at least one capability"));
        }
        self.0
            .entry(change.client)
            .or_default()
            .extend(change.capabilities);
        Ok(())
    }

    /// Revokes some capabilities, or all for an empty list.
    fn revoke(&mut self, change: &GrantChange) {
        let Some(held) = self.0.get_mut(&change.client) else {
            return;
        };
        if change.capabilities.is_empty() {
            held.clear();
        }
        for capability in &change.capabilities {
            held.remove(capability);
        }
        if held.is_empty() {
            self.0.remove(&change.client);
        }
    }

    /// Revokes every grant.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    fn list(&self) -> Value {
        serde_json::json!(self.0)
    }
}

impl State {
    /// `grants.set`, `grants.revoke` and `grants.list`.
    pub fn ipc_grants(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        if method == "grants.list" {
            return Ok(self.grants.list());
        }
        let change: GrantChange = crate::ipc::params_of(params)?;
        tracing::info!(method, client = change.client, capabilities = ?change.capabilities);
        if method == "grants.set" {
            self.grants.set(change)?;
        } else {
            self.grants.revoke(&change);
        }
        Ok(Value::Null)
    }

    /// Checks that the client of a connection holds a capability.
    pub fn check_grant(&self, conn: u64, capability: Capability) -> Result<(), RpcError> {
        let name = self
            .ipc
            .name(conn)
            .ok_or_else(|| rejected("send hello first"))?;
        if self.grants.allows(name, capability) {
            Ok(())
        } else {
            Err(rejected(format!(
                "{capability} needs a grant: choractl grant {name} {capability}"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(client: &str, capabilities: &[Capability]) -> GrantChange {
        GrantChange {
            client: client.into(),
            capabilities: capabilities.to_vec(),
        }
    }

    #[test]
    fn grants_add_up_and_revoke_one_by_one_or_all() {
        let mut grants = Grants::default();
        grants
            .set(change("agent", &[Capability::ScreenCapture]))
            .unwrap();
        grants
            .set(change("agent", &[Capability::InputInject]))
            .unwrap();
        assert!(grants.allows("agent", Capability::ScreenCapture));
        assert!(grants.allows("agent", Capability::InputInject));
        assert!(!grants.allows("other", Capability::ScreenCapture));
        grants.revoke(&change("agent", &[Capability::InputInject]));
        assert!(!grants.allows("agent", Capability::InputInject));
        assert!(grants.allows("agent", Capability::ScreenCapture));
        grants.revoke(&change("agent", &[]));
        assert_eq!(grants.list(), serde_json::json!({}));
    }

    #[test]
    fn a_grant_names_a_capability() {
        let mut grants = Grants::default();
        assert!(grants.set(change("agent", &[])).is_err());
    }
}
