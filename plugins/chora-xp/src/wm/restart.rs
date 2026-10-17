//! What a restart of `chora-xp` keeps: the core keeps the surfaces of
//! the clients but drops the nodes of the plugin with their meta, so
//! the old instance writes the places of the windows and the Explorer
//! windows into a file of its cache, and the new one reads them.
//!
//! ponytail: a stopgap until the core keeps the nodes of a plugin that
//! restarts for a grace period; then rebuild from the plugin's own nodes
//! and delete this file.

use std::collections::HashMap;

use chora_scene::Command;
use serde::{Deserialize, Serialize};

use super::Wm;
use crate::{folder::Place, theme::Rect, window::Window};

/// The file in the cache directory.
const FILE: &str = "restart.json";

/// The windows of the old instance.
#[derive(Serialize, Deserialize)]
pub(super) struct Kept {
    /// The windows of clients, by their surfaces.
    windows: HashMap<String, KeptWindow>,
    /// The Explorer windows, the lowest first.
    explorers: Vec<(Place, KeptWindow)>,
}

/// Where a window was.
#[derive(Serialize, Deserialize)]
pub(super) struct KeptWindow {
    /// The frame.
    pub frame: Rect,
    /// The desktop, a face of the cube.
    pub desktop: usize,
    /// The frame before maximizing, for a maximized window.
    pub restore: Option<Rect>,
}

impl Wm {
    /// Writes the windows for the next instance, before a restart.
    pub(super) fn keep_for_restart(&self) {
        let keep = |w: &Window| {
            w.shown.then(|| KeptWindow {
                frame: w.rect(),
                desktop: w.desktop,
                restore: w.restore,
            })
        };
        let mut kept = Kept {
            windows: HashMap::new(),
            explorers: Vec::new(),
        };
        for w in &self.windows {
            let Some(k) = keep(w) else { continue };
            match self.explorers.iter().find(|e| e.id == w.surface) {
                Some(e) => kept.explorers.push((e.history.current.clone(), k)),
                None => {
                    kept.windows.insert(w.surface.clone(), k);
                }
            }
        }
        let written = serde_json::to_string(&kept)
            .map_err(std::io::Error::other)
            .and_then(|json| self.theme.cache_file(FILE, &json));
        if let Err(err) = written {
            eprintln!("chora-xp: keep the windows for the restart: {err}");
        }
    }

    /// Reads the windows of the old instance, if it restarted, and opens
    /// its Explorer windows again where they were.
    pub(super) fn reopen_kept(&mut self) -> Vec<Command> {
        let Some(kept) = self
            .theme
            .take_cache_file(FILE)
            .and_then(|json| serde_json::from_str::<Kept>(&json).ok())
        else {
            return Vec::new();
        };
        self.kept = kept.windows;
        let mut commands = Vec::new();
        let current = self.desktop;
        for (place, k) in kept.explorers {
            self.desktop = k.desktop;
            commands.extend(self.open_explorer(place));
            let id = format!("xpx{}", self.explorer_count);
            commands.extend(self.put_back(&id, &k));
        }
        self.desktop = current;
        commands.extend(self.relayout_explorers());
        commands
    }

    /// Puts an Explorer window back where it was: its frame, its
    /// maximized state.
    fn put_back(&mut self, surface: &str, k: &KeptWindow) -> Vec<Command> {
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        let active = self.is_active(surface);
        let area = self.bars.first().map(super::Bar::usable);
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
