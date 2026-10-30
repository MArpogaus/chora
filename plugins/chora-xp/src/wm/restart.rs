//! Restart with `resume`: the core keeps the nodes of `chora-xp` for a
//! grace time. Before Restart, `chora-xp` writes its model into the
//! `meta` of its nodes. The root group keeps the current desktop. The
//! group of each window keeps its frame, desktop, frame before
//! maximizing and stacking. The new instance reads it,
//! removes the kept nodes, builds its scene again and puts the windows
//! back.

use std::collections::BTreeMap;

use chora_scene::{Command, Node, NodeKind, PropsUpdate, ipc::ClientError};
use serde::{Deserialize, Serialize};

use super::{NAME, Wm};
use crate::{decoration, theme::Rect};

/// The key of the meta.
const KEY: &str = "xp";

/// A window before the restart.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct KeptWindow {
    /// The frame.
    pub frame: Rect,
    /// The desktop, a face of the cube.
    pub desktop: usize,
    /// The frame before maximizing, for a maximized window.
    pub restore: Option<Rect>,
    /// The place in the stack, the lowest first.
    pub z: usize,
    /// The place on the taskbar, the first first.
    pub opened: usize,
    /// The window is minimized.
    pub minimized: bool,
}

/// A value as the meta of a node.
fn meta(value: &impl Serialize) -> PropsUpdate {
    PropsUpdate {
        meta: serde_json::to_string(value)
            .ok()
            .map(|json| BTreeMap::from([(KEY.into(), json)])),
        ..PropsUpdate::default()
    }
}

/// The value in the meta of a node.
fn read<T: for<'a> Deserialize<'a>>(node: &Node) -> Option<T> {
    serde_json::from_str(node.props.meta.get(KEY)?).ok()
}

impl Wm {
    /// The meta of the nodes for the next instance, before Restart.
    pub(super) fn keep_for_restart(&self) -> Vec<Command> {
        let mut commands = vec![Command::Update {
            id: NAME.into(),
            props: meta(&self.desktop),
        }];
        for (z, w) in self.windows.iter().enumerate().filter(|(_, w)| w.shown) {
            let kept = KeptWindow {
                frame: w.rect(),
                desktop: w.desktop,
                restore: w.restore,
                z,
                opened: self
                    .opened
                    .iter()
                    .position(|s| *s == w.surface)
                    .unwrap_or(usize::MAX),
                minimized: w.minimized,
            };
            commands.push(Command::Update {
                id: decoration::group_id(&w.surface),
                props: meta(&kept),
            });
        }
        commands
    }

    /// Removes the scene of the plugin at its end, but for Restart: nodes
    /// of a plugin with resume outlive it for the grace time.
    pub fn leave(&mut self) -> Result<(), ClientError> {
        let commands = self.remove_scene()?;
        self.apply(commands)
    }

    /// Removes the nodes of the plugin that sit on nodes of others, the
    /// roots of its trees.
    fn remove_scene(&mut self) -> Result<Vec<Command>, ClientError> {
        let nodes = self.conn.query(&format!("owner:{NAME}"))?;
        let owned = |id: &str| nodes.iter().any(|n| n.id == id);
        Ok(nodes
            .iter()
            .filter(|n| n.parent.as_deref().is_none_or(|p| !owned(p)))
            .map(|n| Command::Remove {
                id: n.id.clone(),
                to: None,
            })
            .collect())
    }

    /// Reads the windows that a restart kept into `kept`, and removes the
    /// kept nodes, whose surfaces the new scene adopts. Sets the current
    /// desktop. Nodes without the meta of Restart, as after a crash, go
    /// too. Returns whether the root group is still there: it holds the
    /// claim of the plugin, so it goes only with the creation of the new
    /// one.
    pub(super) fn take_kept(&mut self) -> Result<bool, ClientError> {
        let nodes = self.conn.query(&format!("owner:{NAME}"))?;
        if let Some(desktop) = nodes.iter().find(|n| n.id == NAME).and_then(read) {
            self.desktop = desktop;
        }
        for node in &nodes {
            if let (Some(surface), Some(kept)) = (decoration::surface_of(&node.id), read(node)) {
                self.kept.insert(surface.into(), kept);
            }
        }
        // The surfaces stay reserved for chora-xp, so that no other window
        // manager takes them before the new scene adopts them (D55).
        let mut commands: Vec<Command> = nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Surface)
            .map(|n| Command::Remove {
                id: n.id.clone(),
                to: Some(NAME.into()),
            })
            .collect();
        let old_root = nodes.iter().any(|n| n.id == NAME);
        commands.extend(
            self.remove_scene()?
                .into_iter()
                .filter(|c| !matches!(c, Command::Remove { id, .. } if id == NAME)),
        );
        self.apply(commands)?;
        Ok(old_root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_kept_window_reads_back_from_its_meta() {
        let kept = KeptWindow {
            frame: [10, 20, 300, 200],
            desktop: 1,
            restore: Some([5, 6, 70, 80]),
            z: 2,
            opened: 1,
            minimized: true,
        };
        let meta = meta(&kept).meta.expect("meta");
        assert_eq!(
            serde_json::from_str::<KeptWindow>(&meta[KEY]).ok(),
            Some(kept)
        );
        // One window fits into the 1 KiB of a value of meta.
        assert!(meta[KEY].len() < 1024);
    }
}
