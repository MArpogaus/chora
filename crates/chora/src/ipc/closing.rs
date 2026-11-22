//! The end of a connection: its nodes, its grace time and its output.

use chora_scene::{
    Event,
    ipc::{Message, RpcError},
};
use serde_json::Value;

use super::{GRACE, MAX_OUTPUT, State};

impl State {
    /// Removes the nodes, the cameras and the assets of a client name, and
    /// ends its grace time.
    pub fn drop_nodes(&mut self, name: &str) {
        self.ipc.graces.remove(name);
        self.drop_virtuals_of(name);
        let effects = self.scene.disconnect(name);
        self.execute_gone(effects, name);
        self.ipc.uploads.drop_client(name);
    }

    /// Ends every grace time and removes the kept nodes.
    pub fn end_graces(&mut self) {
        let names: Vec<String> = self.ipc.graces.keys().cloned().collect();
        for name in names {
            self.drop_nodes(&name);
        }
    }

    /// Ends a connection for good with an error: the core reads no more
    /// of it, the client reads the error, then the connection closes
    /// without a grace time.
    pub fn ipc_end(&mut self, conn: u64, error: &RpcError) {
        self.ipc
            .respond(conn, &Message::error(Value::Null, error.clone()));
        let Some(c) = self.ipc.conns.get_mut(&conn) else {
            return;
        };
        (c.keep_nodes, c.resume) = (false, false);
        if let Some(reader) = c.reader.take() {
            self.loop_handle.remove(reader);
        }
    }

    /// Closes a connection. Without `keep_nodes`, the nodes and the assets
    /// of its name go when the last connection of the name closes; with
    /// `resume`, they go after the grace time.
    pub fn ipc_close(&mut self, conn: u64) {
        let Some(closed) = self.ipc.conns.remove(&conn) else {
            return;
        };
        // The client sees the end at once, also while a child that the core
        // forks holds a copy of the socket until its exec.
        let _ = closed.stream.shutdown(std::net::Shutdown::Both);
        if let Some(reader) = closed.reader {
            self.loop_handle.remove(reader);
        }
        let dropped = self.ipc.drop_grabs(conn);
        self.tell_ungrabbed(dropped);
        if self.button_grab.take_if(|g| g.conn == conn).is_some() {
            self.refresh_pointer();
        }
        if !closed.reserves.is_empty() {
            self.output_changed();
        }
        self.methods_closed(conn);
        self.recording_closed(conn);
        let held = self.transactions.drop_conn(conn);
        self.release(&held);
        // A dropped head lets the next transaction start.
        self.complete_transactions();
        let Some(name) = closed.name else { return };
        if self
            .ipc
            .conns
            .values()
            .any(|c| c.name.as_ref() == Some(&name))
        {
            return;
        }
        if self.scene.focused_node_owner() == Some(name.as_str()) {
            self.scene.unfocus_node();
            self.update_focus();
        }
        if closed.resume && !closed.keep_nodes {
            let end = self.time.now() + GRACE;
            let _ = self.expire_at(end);
            self.ipc.graces.insert(name.clone(), end);
        } else if !closed.keep_nodes {
            self.drop_nodes(&name);
        } else {
            // Kept nodes stay, but a surface reserved for the name is free.
            self.drop_virtuals_of(&name);
            let effects = self.scene.unreserve(&name);
            self.execute_gone(effects, &name);
        }
        self.emit(&Event::ClientDisconnected { name });
    }

    /// Writes all pending IPC output. Closes the connections that failed or
    /// overflowed, and those without a reader once their answers are out. A
    /// close can send events to the other connections, so this repeats until
    /// no connection closes.
    pub fn ipc_flush(&mut self) {
        loop {
            let closing = self.ipc_write();
            if closing.is_empty() {
                return;
            }
            for conn in closing {
                self.ipc_close(conn);
            }
        }
    }

    /// Writes the output of every connection and returns those to close.
    fn ipc_write(&mut self) -> Vec<u64> {
        let transactions = &self.transactions;
        let methods = &self.methods;
        self.ipc
            .conns
            .iter_mut()
            .filter_map(|(id, conn)| {
                let broken = !conn.flush();
                if broken && conn.overflow {
                    tracing::warn!(
                        id,
                        name = ?conn.name,
                        "the output passed {} MiB; the connection closes",
                        MAX_OUTPUT >> 20,
                    );
                }
                let answered = conn.reader.is_none()
                    && conn.output.is_empty()
                    && !transactions.waits_for(*id)
                    && !methods.waits_for(*id);
                (broken || answered).then_some(*id)
            })
            .collect()
    }
}
