//! The admin calls, the checks of peers and the hello of a connection.

use super::{
    Event, Hello, RpcError, State, Value, check_protocol, name_taken, params_of, rejected,
    socket_rule,
};

impl State {
    /// The methods of grants and client sockets. Only the admin socket of
    /// `choractl` changes them (D40); any connection lists the grants.
    pub(super) fn ipc_admin(
        &mut self,
        conn: u64,
        method: &str,
        params: Value,
    ) -> Result<Value, RpcError> {
        if method != "grants.list" && !self.ipc.admin(conn) {
            return Err(RpcError::new(
                RpcError::ADMIN_SOCKET_ONLY,
                format!("{method} works only on the admin socket of choractl"),
            ));
        }
        if method == "clients.create" {
            return params_of(params).and_then(|c| self.ipc_client_socket(&c));
        }
        if method == "camera.release" {
            let output = params
                .get("output")
                .and_then(Value::as_str)
                .ok_or_else(|| rejected("camera.release wants {\"output\": NAME}"))?;
            return match self.scene.release_camera(output) {
                Some(owner) => {
                    tracing::warn!(output, %owner, "the admin released the camera");
                    Ok(serde_json::json!({"output": output, "was": owner}))
                }
                None => Err(rejected(format!("no client owns the camera of {output:?}"))),
            };
        }
        self.ipc_grants(method, params)
    }

    /// Checks a `hello` of `conn` against the live connections of its
    /// name: they share effective confinement (a connection on a client
    /// socket is confined whatever its `hello` says), `keep_nodes` and
    /// `resume`. Resume is for a name without a live connection: a second
    /// instance must not take the nodes of a running one. Answers whether
    /// the name is connected.
    pub(super) fn check_peers(&self, conn: u64, hello: &Hello) -> Result<bool, RpcError> {
        let (peer_conn, peer) = self
            .ipc
            .conns
            .iter()
            .find(|(_, c)| c.name.as_deref() == Some(&hello.name))
            .map_or((None, None), |(id, c)| (Some(*id), Some(c)));
        let confined = hello.confined || self.ipc.fixed(conn).is_some();
        if peer.is_some()
            && self.ipc.fixed(conn).is_none()
            && !self.ipc.same_program(conn, &hello.name)
        {
            return Err(name_taken(format!(
                "{:?} is connected from another program",
                hello.name
            )));
        }
        if let Some(peer_confined) = peer_conn
            .map(|id| self.ipc.confined(id))
            .filter(|p| *p != confined)
        {
            return Err(name_taken(format!(
                "{:?} is in use with confined {peer_confined}",
                hello.name
            )));
        }
        if let Some(peer) =
            peer.filter(|p| (p.keep_nodes, p.resume) != (hello.keep_nodes, hello.resume))
        {
            return Err(name_taken(format!(
                "{:?} is in use with keep_nodes {} and resume {}",
                hello.name, peer.keep_nodes, peer.resume
            )));
        }
        if hello.resume && peer.is_some() {
            return Err(name_taken(format!(
                "{:?} is connected; resume needs a name without a live connection",
                hello.name
            )));
        }
        Ok(peer.is_some())
    }

    /// Names a connection. All connections of a name share one
    /// `keep_nodes` and one `resume` setting. With `resume`, a name in its
    /// grace time gets its nodes back; without, they go first.
    /// A client socket fixes the name (D33).
    pub(super) fn ipc_hello(&mut self, conn: u64, hello: Hello) -> Result<Value, RpcError> {
        check_protocol(hello.protocol)?;
        if self.ipc.name(conn).is_some() {
            return Err(rejected("hello was sent already"));
        }
        if !crate::methods::valid_name(&hello.name) {
            return Err(rejected(
                "a client name has lowercase letters, digits, _, - and ., from a letter",
            ));
        }
        match self.ipc.fixed(conn) {
            Some(fixed) if fixed != hello.name => {
                return Err(socket_rule(format!("this socket belongs to {fixed:?}")));
            }
            None if self.ipc.clients.contains_key(&hello.name) => {
                return Err(name_taken(format!(
                    "{:?} has a client socket and connects only through it",
                    hello.name
                )));
            }
            _ => {}
        }
        let known = self.check_peers(conn, &hello)?;
        if hello.resume {
            self.ipc.graces.remove(&hello.name);
        } else if self.ipc.graces.contains_key(&hello.name) {
            self.drop_nodes(&hello.name);
        }
        if let Some(c) = self.ipc.conns.get_mut(&conn) {
            c.name = Some(hello.name.clone());
            (c.keep_nodes, c.resume) = (hello.keep_nodes, hello.resume);
            c.confined = hello.confined;
        }
        self.ipc.greeted = true;
        if !known {
            self.emit(&Event::ClientConnected { name: hello.name });
        }
        let software = self
            .backend
            .as_mut()
            .and_then(|b| b.with_gles(crate::gl::is_software).ok())
            .unwrap_or(false);
        Ok(serde_json::json!({
            "version": chora_scene::ipc::VERSION,
            "protocol": chora_scene::ipc::PROTOCOL,
            "features": chora_scene::ipc::FEATURES,
            "software": software,
        }))
    }
}
