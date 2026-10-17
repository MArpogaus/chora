//! Grants of the user for the IPC methods that see or drive the whole
//! session, or start programs: `input.inject`, `screen.capture` and
//! `process.spawn`.
//!
//! `choractl grant NAME CAPABILITY` grants a capability to a client name,
//! until `choractl revoke` or the end of the session, through the admin
//! socket (D40). A client of a client socket cannot reach it, a process of
//! the user on the host can. The MCP bridge offers no tool for grants, so
//! an agent cannot grant itself by accident. The emergency reset revokes
//! all grants.

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
            // A Wayland client got its protocols at its connection.
            self.disconnect_wayland(&change.client, &change.capabilities);
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
    use serde_json::json;

    use super::*;
    use crate::testing::Core;

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

    #[test]
    fn grants_change_and_list_over_ipc() {
        let mut core = Core::new();
        let mut ctl = core.admin();
        let both = json!({"client": "agent", "capabilities": ["screen.capture", "input.inject"]});
        ctl.ok(&mut core, "grants.set", both);
        let list = ctl.ok(&mut core, "grants.list", Value::Null);
        assert_eq!(list, json!({"agent": ["input.inject", "screen.capture"]}));
        let one = json!({"client": "agent", "capabilities": ["input.inject"]});
        ctl.ok(&mut core, "grants.revoke", one);
        let list = ctl.ok(&mut core, "grants.list", Value::Null);
        assert_eq!(list, json!({"agent": ["screen.capture"]}));
        let unknown = json!({"client": "agent", "capabilities": ["disk.write"]});
        let (error_code, _) = ctl.fails(&mut core, "grants.set", unknown);
        assert_eq!(error_code, RpcError::INVALID_PARAMS);
    }

    #[test]
    fn only_the_admin_socket_changes_grants_and_makes_client_sockets() {
        let mut core = Core::new();
        let mut agent = core.named("agent");
        let grant = json!({"client": "agent", "capabilities": ["input.inject"]});
        for (method, params) in [
            ("grants.set", grant.clone()),
            ("grants.revoke", grant),
            ("clients.create", json!({"name": "agent"})),
        ] {
            let (_, message) = agent.fails(&mut core, method, params);
            assert!(
                message.contains("only on the admin socket"),
                "{method}: {message}"
            );
        }
        assert_eq!(agent.ok(&mut core, "grants.list", Value::Null), json!({}));
    }

    #[test]
    fn each_method_of_a_capability_needs_its_grant() {
        let mut core = Core::new();
        let mut agent = core.named("agent");
        for (method, capability) in [
            ("input.inject", "input.inject"),
            ("screen.capture", "screen.capture"),
            ("process.spawn", "process.spawn"),
            ("process.wait", "process.spawn"),
            ("plugins.restart", "process.spawn"),
        ] {
            let (error_code, message) = agent.fails(&mut core, method, json!({}));
            assert_eq!(error_code, RpcError::REJECTED, "{method}");
            let wanted = format!("{capability} needs a grant: choractl grant agent {capability}");
            assert_eq!(message, wanted, "{method}");
        }
    }

    #[test]
    fn a_grant_belongs_to_a_name_and_outlives_its_connections() {
        let mut core = Core::new();
        let mut ctl = core.admin();
        let grant = json!({"client": "agent", "capabilities": ["input.inject"]});
        ctl.ok(&mut core, "grants.set", grant);
        let inject = json!({"events": []});
        for _ in 0..2 {
            let mut agent = core.named("agent");
            agent.ok(&mut core, "input.inject", inject.clone());
        }
        let mut other = core.named("other");
        let (error_code, _) = other.fails(&mut core, "input.inject", inject);
        assert_eq!(error_code, RpcError::REJECTED);
    }

    #[test]
    fn the_emergency_reset_revokes_every_grant_and_disconnects() {
        let mut core = Core::new();
        let mut ctl = core.admin();
        let all = json!({"client": "agent", "capabilities": ["input.inject", "process.spawn"]});
        ctl.ok(&mut core, "grants.set", all);
        core.state.reset();
        assert!(ctl.closed(&mut core));
        assert!(!core.state.grants.allows("agent", Capability::InputInject));
        let mut ctl = core.admin();
        assert_eq!(ctl.ok(&mut core, "grants.list", Value::Null), json!({}));
    }
}
