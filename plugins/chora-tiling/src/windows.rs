//! Adopting, laying out and forgetting the windows of tiling.

use std::collections::{BTreeMap, HashMap};

use chora_kit::wm::kept;
use chora_scene::{Command, NodeKind, PropsUpdate, ipc::ClientError};

use crate::{
    NAME, Tiling,
    commands::{changes, clip_of, frame_nodes, group_of, meta_change, styled},
    dress::Dress,
    theme,
    wm::{Placement, WINDOW_KEY},
};

impl Tiling {
    /// Takes a surface without an owner. A surface that another client took
    /// first, or whose app id `--ignore` names, stays out of the layout.
    ///
    /// The size of a window that comes with one, as from another window
    /// manager: it floats at that size, centered, and a drag hits it.
    ///
    /// A new window of tiling takes the focus.
    ///
    /// The book proposed it; true when the core took the attach, so that
    /// the window is tiling's.
    pub(super) fn adopt(&mut self, surface: &str) -> Result<bool, ClientError> {
        if self.without_outputs() {
            return self.pass_on_reserved(surface).map(|()| false);
        }
        let nodes = self.query(&format!("id:{surface}"))?;
        let ignored = nodes.iter().any(|node| {
            node["surface"]["app_id"]
                .as_str()
                .is_some_and(|app| self.ignore.iter().any(|i| i == app))
        });
        let size = nodes
            .first()
            .and_then(|n| serde_json::from_value(n["surface"]["size"].clone()).ok());
        if ignored {
            return self.pass_on_reserved(surface).map(|()| false);
        }
        let mut commands = Vec::new();
        if !self.groups.contains(surface) {
            commands.extend(frame_nodes(
                surface,
                self.border.as_deref(),
                self.shadow.as_deref(),
            ));
        }
        commands.push(Command::Attach {
            surface: surface.into(),
            parent: clip_of(surface),
        });
        match self.call_apply(commands) {
            Ok(()) => {
                self.groups.insert(surface.into());
                self.wm.add_window(surface);
                if let Some(size) = size {
                    self.wm.set_size(surface, size);
                }
                self.focus.take();
                self.style(surface).map(|()| true)
            }
            Err(ClientError::Rpc(err)) => {
                eprintln!("chora-tiling: {surface}: {err}");
                Ok(false)
            }
            Err(err) => Err(err),
        }
    }

    /// Gives a new window the style of the settings. A shader that fails
    /// only logs, and the window stays.
    pub(super) fn style(&mut self, surface: &str) -> Result<(), ClientError> {
        self.apply(vec![styled(&self.wm.config.style, surface)])
    }

    /// Sends what changed since the last transaction: sizes and states,
    /// positions, visibility, the stacking order and the focus.
    ///
    /// No outputs: no claim, so `null`.
    ///
    /// A grab of a pointer binding that another window manager holds
    /// succeeds only with a claim, for example with `--output` for an
    /// output that came later (D55).
    pub(super) fn relayout(&mut self) -> Result<(), ClientError> {
        let placements = self.wm.arrange();
        let mut commands = Vec::new();
        commands.extend(self.window_changes(&placements));
        let order: Vec<String> = placements.iter().map(|p| p.id.clone()).collect();
        if order != self.order {
            commands.extend(order.iter().enumerate().map(|(index, id)| Command::Move {
                id: group_of(id),
                parent: None,
                index: Some(index),
            }));
        }
        let focus = self.wm.focused().map(String::from);
        let send_focus = self.focus.sends(focus.as_deref());
        if send_focus {
            commands.push(Command::Focus {
                surface: focus.clone(),
            });
        }
        let saved = self.meta();
        for (id, meta) in &saved {
            commands.extend(meta_change(self.saved.get(id), id, meta));
        }
        let claim = self.wm.claim();
        if claim != self.claimed {
            commands.push(Command::Update {
                id: NAME.into(),
                props: PropsUpdate {
                    claim: Some(claim),
                    ..PropsUpdate::default()
                },
            });
        }
        commands.append(&mut self.meshes);
        commands.extend(self.stale.drain(..).map(|id| Command::Remove {
            id: group_of(&id),
            to: None,
        }));
        if commands.is_empty() {
            return Ok(());
        }
        let gained = self.claimed.is_none() && claim.is_some();
        self.saved = saved;
        self.claimed = claim;
        self.sent = placements.into_iter().map(|p| (p.id.clone(), p)).collect();
        self.last_tag = self.wm.current_tag_index();
        self.order = order;
        if send_focus {
            self.focus.sent(focus);
        }
        self.apply(commands)?;
        if gained && self.claimed.is_some() {
            self.rt.start_grabs();
            self.grab_all()?;
            self.rt.end_grabs()?;
        }
        Ok(())
    }

    /// The commands that bring the windows from what was sent to their
    /// placements.
    pub(super) fn window_changes(&self, placements: &[Placement]) -> Vec<Command> {
        let c = &self.colors;
        let colors = [&c.accent, &c.accent_2, &c.line, &c.urgent].map(|h| theme::channels(h));
        let look = &self.wm.config.look;
        let width = self.wm.current_output().map_or(0.0, |(_, size)| size[0]);
        let tag = self.wm.current_tag_index();
        let dress = Dress {
            colors: self.border.is_some().then_some(&colors),
            shadows: look.shadows,
            animate: look.animations && !self.still,
            slide: match tag.cmp(&self.last_tag) {
                std::cmp::Ordering::Greater => width,
                std::cmp::Ordering::Less => -width,
                std::cmp::Ordering::Equal => 0.0,
            },
        };
        placements
            .iter()
            .flat_map(|p| changes(self.sent.get(&p.id), p, &dress))
            .collect()
    }

    /// The meta of the group and of each window, by node, under the
    /// versioned key of `kit::wm::kept`; the keys of before go.
    pub(super) fn meta(&self) -> HashMap<String, BTreeMap<String, String>> {
        let window_keys = [WINDOW_KEY.to_owned()];
        let windows = self.wm.window_meta().into_iter().map(|(id, text)| {
            let sub = BTreeMap::from([(WINDOW_KEY.to_owned(), text)]);
            (id, kept::write(&sub, &window_keys))
        });
        let group = self.wm.group_meta();
        let group_keys: Vec<String> = group.keys().cloned().collect();
        windows
            .chain([(NAME.to_owned(), kept::write(&group, &group_keys))])
            .collect()
    }

    /// Forgets a window that left tiling: its place in the model, and
    /// what the core shows of it.
    ///
    /// The focus on it is no longer that of tiling: when the window
    /// comes back, tiling sends it again.
    pub(super) fn forget_window(&mut self, id: &str) {
        self.book.forget(id);
        self.wm.remove_window(id);
        self.sent.remove(id);
        self.saved.remove(id);
        if self.groups.remove(id) {
            self.stale.push(id.into());
        }
        self.effects.cancel_wobble(id);
        self.bent.remove(id);
        if self.focus.surface.as_deref() == Some(id) {
            self.focus.surface = None;
            self.focus.sent = false;
        }
    }

    /// Forgets what the core shows, so that the next transaction sends all.
    pub(super) fn forget(&mut self) {
        self.sent.clear();
        self.saved.clear();
        self.claimed = None;
        self.order.clear();
        self.focus.sent = false;
    }

    /// Has the core compile the shaders of the style, on a node that the
    /// same transaction removes again, so that `settings.set` refuses a
    /// shader that does not compile instead of each new window.
    pub(super) fn check_shaders(&mut self) -> Result<(), ClientError> {
        let effects = self.settings.value.style.effects.clone();
        if effects.is_empty() {
            return Ok(());
        }
        let id = "tiling-shader-check";
        self.call_apply(vec![
            Command::Create {
                id: id.into(),
                kind: NodeKind::Rect,
                parent: NAME.into(),
                props: PropsUpdate {
                    effects: Some(effects),
                    ..PropsUpdate::default()
                },
            },
            Command::Remove {
                id: id.into(),
                to: None,
            },
        ])
    }

    pub(super) fn call_apply(&mut self, commands: Vec<Command>) -> Result<(), ClientError> {
        match self.rt.conn.try_apply(commands)? {
            None => Ok(()),
            Some(err) => Err(ClientError::Rpc(err)),
        }
    }

    /// Applies a transaction. A rejected transaction only logs, and the next
    /// one sends everything again: a surface can disappear between an event
    /// and the answer to it, and the event that follows corrects the layout.
    pub(super) fn apply(&mut self, commands: Vec<Command>) -> Result<(), ClientError> {
        self.apply_report(commands).map(|_| ())
    }

    /// Like `apply`, but `Ok(false)` when the core rejected the transaction.
    pub(super) fn apply_report(&mut self, commands: Vec<Command>) -> Result<bool, ClientError> {
        match self.call_apply(commands) {
            Err(ClientError::Rpc(err)) => {
                if !err.is_gone_surface() {
                    eprintln!("chora-tiling: {err}");
                }
                self.forget();
                Ok(false)
            }
            Ok(()) => Ok(true),
            Err(err) => Err(err),
        }
    }
}
