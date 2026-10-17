//! Restart with `resume`: the core keeps the nodes of `chora-xp` for a
//! grace time. Before Restart, `chora-xp` writes its model into the
//! `meta` of its nodes. The root group keeps the current desktop. The
//! group of each window keeps its frame, desktop, frame before
//! maximizing, stacking and Explorer place. The new instance reads it,
//! removes the kept nodes, builds its scene again and puts the windows
//! back.

use std::collections::BTreeMap;

use chora_scene::{Command, Node, PropsUpdate, ipc::ClientError};
use serde::{Deserialize, Serialize};

use super::{Bar, NAME, Wm};
use crate::{decoration, folder::Place, theme::Rect};

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
    /// The place of an Explorer window.
    pub place: Option<Place>,
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
                place: self
                    .explorers
                    .iter()
                    .find(|e| e.id == w.surface)
                    .map(|e| e.history.current.clone()),
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
    /// too. Returns the Explorer windows to open again.
    pub(super) fn take_kept(&mut self) -> Result<Vec<KeptWindow>, ClientError> {
        let nodes = self.conn.query(&format!("owner:{NAME}"))?;
        if let Some(desktop) = nodes.iter().find(|n| n.id == NAME).and_then(read) {
            self.desktop = desktop;
        }
        let mut explorers = Vec::new();
        for node in &nodes {
            let (Some(surface), Some(kept)) = (decoration::surface_of(&node.id), read(node)) else {
                continue;
            };
            match kept {
                KeptWindow { place: Some(_), .. } => explorers.push(kept),
                kept => {
                    self.kept.insert(surface.into(), kept);
                }
            }
        }
        let commands = self.remove_scene()?;
        self.apply(commands)?;
        Ok(explorers)
    }

    /// Opens a kept Explorer window again where it was: its frame, its
    /// desktop, its maximized state.
    pub(super) fn reopen(&mut self, k: KeptWindow) -> Vec<Command> {
        let Some(place) = k.place else {
            return Vec::new();
        };
        let current = std::mem::replace(&mut self.desktop, k.desktop);
        let mut commands = self.open_explorer(place);
        self.desktop = current;
        let Some(i) = self.explorers.last().and_then(|e| self.index(&e.id)) else {
            return commands;
        };
        let active = self.is_active(&self.windows[i].surface);
        let area = self.bars.first().map(Bar::usable);
        let w = &mut self.windows[i];
        match (k.restore, area) {
            (Some(restore), Some(area)) => {
                w.restore = Some(restore);
                w.maximize(area);
            }
            _ => commands.extend(w.fit(k.frame, active)),
        }
        w.minimized = k.minimized;
        commands.extend(w.style(&mut self.theme, active));
        commands.push(w.place());
        commands
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
            place: Some(Place::Trash),
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
