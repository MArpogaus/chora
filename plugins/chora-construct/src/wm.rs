//! Window manager panes (D69 step 9, K7): a pane that shows a virtual
//! output of the construct, with a window manager on it as a child of
//! the construct. The pane forwards the pointer into the output, so the
//! windows of the manager take clicks at any place and turn of the pane.
//! The managers are those of the setting `managers`, by their client
//! name; each has one pane at most, and none while the cube has it. A
//! manager that ends, or does not connect in time, takes its pane and
//! its output with it; the windows on the output follow the rules of the
//! core for an output that goes.

use std::{
    process::{Child, Stdio},
    time::{Duration, Instant},
};

use chora_scene::Event;
use serde_json::{Value, json};

use super::{ClientError, Construct, Content, Pane, PaneFile, RpcError, cube};

/// The size of a new pane without one, in logical pixels.
const SIZE: [f64; 2] = [1024.0, 768.0];
/// The smallest side of the output of a pane.
const SMALLEST: f64 = 100.0;
/// How long a window manager may take to connect.
const CONNECT: Duration = Duration::from_secs(10);

/// A window manager in a pane: its client name, its output, its process
/// and, until it connects, the time it has.
pub(crate) struct Managed {
    manager: String,
    output: String,
    child: Child,
    connect_by: Option<Instant>,
}

fn refused(message: String) -> ClientError {
    ClientError::Rpc(RpcError::new(RpcError::REJECTED, message))
}

/// Why a manager pane cannot run `manager`: it is no key of the setting
/// `managers`.
pub(crate) fn unknown_manager(
    _managers: &std::collections::BTreeMap<String, String>,
    _manager: &str,
) -> Option<String> {
    None
}

impl Construct {
    /// The method `pane.manager`, and a manager pane of a construct
    /// file: adds a virtual output of `size`, starts the manager on it
    /// and shows the output in a pane ahead, or where the file has it.
    pub(crate) fn open_manager(
        &mut self,
        manager: &str,
        size: Option<[f64; 2]>,
        file: Option<&PaneFile>,
    ) -> Result<Value, ClientError> {
        let managers = &self.settings.value.managers;
        let Some(command) = managers.get(manager).cloned() else {
            let reason = unknown_manager(managers, manager);
            return Err(refused(reason.unwrap_or_else(|| {
                format!("{manager} is no window manager of the setting managers")
            })));
        };
        if self.manager_taken(manager)? {
            return Err(refused(format!("a client {manager} runs already")));
        }
        let wanted = size.or(file.and_then(|f| f.size)).unwrap_or(SIZE);
        let size = [
            wanted[0].clamp(SMALLEST, self.lens.w.max(SMALLEST)).round(),
            wanted[1].clamp(SMALLEST, self.lens.h.max(SMALLEST)).round(),
        ];
        let output = (self.add_outputs(1, size)?.pop())
            .ok_or_else(|| refused("the core added no output".into()))?;
        let command = format!("exec {command} --output {output}");
        let spawned = std::process::Command::new("sh")
            .args(["-c", &command])
            .stdin(Stdio::null())
            .spawn();
        let mut child = match spawned {
            Ok(child) => child,
            Err(err) => {
                self.remove_outputs(std::slice::from_ref(&output));
                return Err(refused(format!("cannot start {command:?}: {err}")));
            }
        };
        let at = self.place(file.and_then(|f| f.at), size, true);
        let at = at.unwrap_or_default();
        let pane = Pane {
            name: self.label(manager),
            source: output.clone(),
            content: Content::Manager {
                manager: manager.to_owned(),
            },
            at,
            yaw: self.yaw_at(file, at),
            size,
            taken: false,
            attached: false,
        };
        if let Some(err) = self.rt.conn.try_apply(self.pane_nodes(&pane))? {
            cube::end(&mut child);
            self.remove_outputs(std::slice::from_ref(&output));
            return Err(ClientError::Rpc(err));
        }
        let answer = json!({"name": pane.name, "output": output});
        self.panes.push(pane);
        self.managed.push(Managed {
            manager: manager.to_owned(),
            output,
            child,
            connect_by: Some(Instant::now() + CONNECT),
        });
        self.frame()?;
        Ok(answer)
    }

    /// Whether a window manager of that client name runs: in a pane, on
    /// the cube, or as a client that the construct did not start.
    fn manager_taken(&mut self, manager: &str) -> Result<bool, ClientError> {
        if self.has_manager(manager) || (self.cube.is_some() && manager == cube::MANAGER) {
            return Ok(true);
        }
        let clients: Value = self.rt.conn.call("clients.list", Value::Null)?;
        let list = clients.as_array().cloned().unwrap_or_default();
        Ok(list.iter().any(|c| c["name"].as_str() == Some(manager)))
    }

    /// Whether a pane has the window manager of that client name.
    pub(crate) fn has_manager(&self, manager: &str) -> bool {
        self.managed.iter().any(|m| m.manager == manager)
    }

    /// The output of the pane of a window manager.
    pub(crate) fn manager_output(&self, manager: &str) -> Option<&str> {
        let held = self.managed.iter().find(|m| m.manager == manager)?;
        Some(held.output.as_str())
    }

    /// Whether the construct runs the window manager of that client
    /// name, in a pane or on the cube.
    pub(crate) fn manages(&self, client: &str) -> bool {
        self.has_manager(client) || (self.cube.is_some() && client == cube::MANAGER)
    }

    /// A window manager of the construct connected, and no longer has
    /// to; or it went, and its pane or the cube goes with it. A manager
    /// that goes before the one of a pane connected is the one of
    /// before, which the construct ended.
    pub(crate) fn manager_client(&mut self, event: &Event) -> Result<(), ClientError> {
        let (Event::ClientConnected { name } | Event::ClientDisconnected { name }) = event else {
            return Ok(());
        };
        if !self.has_manager(name) {
            return self.cube_client(event);
        }
        if matches!(event, Event::ClientDisconnected { .. }) {
            let connected = |m: &Managed| m.manager == *name && m.connect_by.is_none();
            if self.managed.iter().any(connected) {
                return self.drop_manager(name);
            }
            return Ok(());
        }
        for managed in self.managed.iter_mut().filter(|m| m.manager == *name) {
            managed.connect_by = None;
        }
        Ok(())
    }

    /// Removes the pane of a window manager; its child and its output go
    /// with it.
    fn drop_manager(&mut self, manager: &str) -> Result<(), ClientError> {
        let held = |p: &Pane| matches!(&p.content, Content::Manager { manager: m } if m == manager);
        if let Some(i) = self.panes.iter().position(held) {
            return self.remove_pane(i);
        }
        self.end_manager(manager);
        Ok(())
    }

    /// Ends the child of a window manager and removes its output, after
    /// its pane went.
    pub(crate) fn end_manager(&mut self, manager: &str) {
        let (gone, kept): (Vec<Managed>, Vec<Managed>) = std::mem::take(&mut self.managed)
            .into_iter()
            .partition(|m| m.manager == manager);
        self.managed = kept;
        for mut managed in gone {
            cube::end(&mut managed.child);
            self.remove_outputs(std::slice::from_ref(&managed.output));
        }
    }

    /// Ends every window manager of a pane: the space goes, or another
    /// construct loads.
    pub(crate) fn end_managers(&mut self) {
        let names: Vec<String> = self.managed.iter().map(|m| m.manager.clone()).collect();
        for name in names {
            self.end_manager(&name);
        }
    }

    /// The time by which a window manager of a pane must connect.
    pub(crate) fn manager_deadline(&self) -> Option<Instant> {
        self.managed.iter().filter_map(|m| m.connect_by).min()
    }

    /// Removes the panes of the window managers that did not connect in
    /// time.
    pub(crate) fn check_managers(&mut self) -> Result<(), ClientError> {
        let now = Instant::now();
        let late = |m: &&Managed| m.connect_by.is_some_and(|by| by <= now);
        let names: Vec<String> = (self.managed.iter().filter(late))
            .map(|m| m.manager.clone())
            .collect();
        for name in names {
            eprintln!("chora-construct: the window manager {name} did not connect");
            self.drop_manager(&name)?;
        }
        Ok(())
    }
}
