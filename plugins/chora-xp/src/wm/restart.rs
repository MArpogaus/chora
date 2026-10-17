//! Restart with `resume`: the core keeps the nodes of `chora-xp` for a
//! grace time. Before Restart, `chora-xp` writes what its model needs
//! into the `meta` of its nodes: the current desktop in its root group,
//! and the frame, desktop, frame before maximizing, stacking and the
//! place of an Explorer window in the group of each window. The new
//! instance reads it, removes the kept nodes, builds its scene again
//! and puts the windows back.

use std::collections::BTreeMap;

use chora_scene::{Command, PropsUpdate, ipc::ClientError};

use super::{Bar, NAME, Wm};
use crate::{decoration, folder::Place, theme::Rect};

/// A window before the restart.
pub(super) struct KeptWindow {
    /// The frame.
    pub frame: Rect,
    /// The desktop, a face of the cube.
    pub desktop: usize,
    /// The frame before maximizing, for a maximized window.
    pub restore: Option<Rect>,
    /// The place in the stack, the lowest first.
    z: usize,
    /// The place of an Explorer window.
    place: Option<Place>,
}

/// A rectangle as meta: `x y w h`.
fn rect_meta([x, y, w, h]: Rect) -> String {
    format!("{x} {y} {w} {h}")
}

fn meta_rect(text: &str) -> Option<Rect> {
    let numbers: Vec<i32> = text.split(' ').filter_map(|n| n.parse().ok()).collect();
    numbers.try_into().ok()
}

impl KeptWindow {
    fn of(meta: &BTreeMap<String, String>) -> Option<Self> {
        Some(Self {
            frame: meta_rect(meta.get("frame")?)?,
            desktop: meta.get("desktop")?.parse().ok()?,
            restore: meta.get("restore").and_then(|r| meta_rect(r)),
            z: meta.get("z")?.parse().ok()?,
            place: meta.get("place").and_then(|p| serde_json::from_str(p).ok()),
        })
    }
}

impl Wm {
    /// The meta of the nodes for the next instance, before Restart.
    pub(super) fn keep_for_restart(&self) -> Vec<Command> {
        let meta = |pairs: Vec<(&str, String)>| PropsUpdate {
            meta: Some(pairs.into_iter().map(|(k, v)| (k.into(), v)).collect()),
            ..PropsUpdate::default()
        };
        let mut commands = vec![Command::Update {
            id: NAME.into(),
            props: meta(vec![("desktop", self.desktop.to_string())]),
        }];
        for (z, w) in self.windows.iter().enumerate().filter(|(_, w)| w.shown) {
            let mut pairs = vec![
                ("frame", rect_meta(w.rect())),
                ("desktop", w.desktop.to_string()),
                ("z", z.to_string()),
            ];
            if let Some(restore) = w.restore {
                pairs.push(("restore", rect_meta(restore)));
            }
            if let Some(e) = self.explorers.iter().find(|e| e.id == w.surface)
                && let Ok(place) = serde_json::to_string(&e.history.current)
            {
                pairs.push(("place", place));
            }
            commands.push(Command::Update {
                id: decoration::group_id(&w.surface),
                props: meta(pairs),
            });
        }
        commands
    }

    /// Removes the nodes of the plugin that sit on nodes of others, the
    /// roots of its trees: at Turn Off, and after a resume.
    pub(super) fn remove_scene(&mut self) -> Result<Vec<Command>, ClientError> {
        let nodes = self.conn.query(&format!("owner:{NAME}"))?;
        let owned = |id: &str| nodes.iter().any(|n| n.id == id);
        Ok(nodes
            .iter()
            .filter(|n| n.parent.as_deref().is_none_or(|p| !owned(p)))
            .map(|n| Command::Remove { id: n.id.clone() })
            .collect())
    }

    /// Reads the windows that a restart kept, and removes the kept nodes,
    /// whose surfaces the new scene adopts. Sets the current desktop.
    pub(super) fn take_kept(&mut self) -> Result<Vec<(String, KeptWindow)>, ClientError> {
        let nodes = self.conn.query(&format!("owner:{NAME}"))?;
        // Only Restart writes the desktop: nodes without it belong to an
        // instance that still runs, and the new scene fails as before.
        let Some(desktop) = nodes
            .iter()
            .find(|n| n.id == NAME)
            .and_then(|n| n.props.meta.get("desktop")?.parse().ok())
        else {
            return Ok(Vec::new());
        };
        self.desktop = desktop;
        let mut kept: Vec<(String, KeptWindow)> = nodes
            .iter()
            .filter_map(|n| {
                let surface = n.id.strip_prefix("xp-win-")?;
                Some((surface.to_string(), KeptWindow::of(&n.props.meta)?))
            })
            .collect();
        kept.sort_by_key(|(_, k)| k.z);
        let commands = self.remove_scene()?;
        self.apply(commands)?;
        Ok(kept)
    }

    /// Puts the kept windows back in their order: adopts the surfaces of
    /// the clients and opens the Explorer windows again. Only a window of
    /// the current desktop gets the focus.
    pub(super) fn restore_kept(
        &mut self,
        kept: Vec<(String, KeptWindow)>,
    ) -> Result<(), ClientError> {
        let current = self.desktop;
        for (surface, k) in kept {
            if let Some(place) = k.place.clone() {
                self.desktop = k.desktop;
                let mut commands = self.open_explorer(place);
                let id = format!("xpx{}", self.explorer_count);
                commands.extend(self.put_back(&id, &k));
                self.desktop = current;
                self.apply(commands)?;
                continue;
            }
            let Some(node) = self.conn.query(&format!("id:{surface}"))?.pop() else {
                continue;
            };
            if node.owner.is_some() {
                continue;
            }
            self.kept.insert(surface.clone(), k);
            let info = node.surface.unwrap_or_default();
            self.adopt(&surface, info.app_id, info.title.as_deref())?;
        }
        let top = self.topmost_except("");
        let mut commands = self.focus(top.as_deref());
        commands.extend(self.sync_tasks());
        commands.extend(self.relayout_explorers());
        self.apply(commands)
    }

    /// Puts an Explorer window back where it was: its frame, its
    /// maximized state.
    fn put_back(&mut self, surface: &str, k: &KeptWindow) -> Vec<Command> {
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        let active = self.is_active(surface);
        let area = self.bars.first().map(Bar::usable);
        let w = &mut self.windows[i];
        let mut commands: Vec<Command> = w.fit(k.frame, active).into_iter().collect();
        if let (Some(restore), Some(area)) = (k.restore, area) {
            w.maximize(area);
            w.restore = Some(restore);
        }
        w.desktop = k.desktop;
        commands.extend(w.style(&mut self.theme, active));
        commands.push(w.place());
        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_meta_of_a_window_reads_back() {
        let meta: BTreeMap<String, String> = [
            ("frame", "10 20 300 200"),
            ("desktop", "1"),
            ("restore", "5 6 70 80"),
            ("z", "2"),
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect();
        let k = KeptWindow::of(&meta).expect("a window");
        assert_eq!(k.frame, [10, 20, 300, 200]);
        assert_eq!((k.desktop, k.restore, k.z), (1, Some([5, 6, 70, 80]), 2));
        assert!(k.place.is_none());
    }
}
