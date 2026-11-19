//! Which windows are the look's (arch plan 1.1, 2.2): the claim filter,
//! adopt and forget as proposals that the look settles, the outputs of
//! `--output`, the origin of each window, the phase events
//! `<look>.window`, and the pass-through of a foreign look on the nodes of
//! the look (D58).
//!
//! The book does no IO. [`Book::apply`] turns an event into changes; an
//! [`Change::Adopt`] is a proposal: the look attaches the surface and
//! answers [`Book::confirm`] when the core took it, or [`Book::refused`]
//! when another window manager was first. Where a window shows, and what
//! a detached own window does, stay the look's.

use std::collections::BTreeMap;

use chora_scene::{Command, Event, PropsUpdate, ipc::Emit};
use serde::Serialize;
use serde_json::{Value, json};

use super::{give::pass_on, kept};

/// Where a window of the look came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase", tag = "origin")]
pub enum Origin {
    /// A new window, with the activation token that mapped it.
    New {
        /// The activation token, when one mapped the window.
        #[serde(skip_serializing_if = "Option::is_none")]
        token: Option<String>,
    },
    /// A window that the core reserved for the look: another window
    /// manager gave it (D55), or the core kept it for the look.
    Given,
    /// A window that the look kept over a restart.
    Kept,
}

/// Why a window left the look.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Why {
    /// The client destroyed or unmapped it.
    Destroyed,
    /// It lost its parent and owner.
    Detached,
    /// The look gave it to this window manager.
    Given(String),
    /// The last output of `--output` went: the look let it go to any
    /// window manager.
    Released,
}

/// A step in the life of a window, as `<look>.window` tells it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    /// The window became the look's.
    Opened,
    /// The window closed.
    Closed,
    /// The window went to another window manager.
    Given,
}

/// What the look does after an event, in order. A phase comes before
/// the forget of its window, so the look still knows where it showed.
#[derive(Debug, Clone, PartialEq)]
pub enum Change {
    /// A proposal: the look attaches the surface, then answers
    /// [`Book::confirm`] or [`Book::refused`].
    Adopt {
        /// The surface id.
        id: String,
        /// Where it comes from.
        origin: Origin,
    },
    /// The window is no longer the look's: the look drops its data.
    Forget {
        /// The surface id.
        id: String,
        /// Why it left.
        why: Why,
    },
    /// The outputs of the look changed.
    Outputs,
    /// The look emits the phase with [`Book::phase_event`].
    Phase {
        /// The surface id.
        id: String,
        /// The step.
        phase: Phase,
    },
    /// The look applies these commands as they are.
    Apply(Vec<Command>),
}

/// A proposal that the look has not settled yet.
#[derive(Debug, Clone)]
pub(super) enum Pending {
    /// The look attaches the surface.
    Adopt(Origin),
    /// The look gives the window to this window manager.
    Give(String),
}

/// The book of a look: which windows are its own.
#[derive(Debug)]
pub struct Book {
    /// The client name of the look, and the id of its root group.
    pub(super) name: String,
    /// Whether `--output` names outputs.
    filters: bool,
    /// The outputs of the look.
    outputs: Vec<String>,
    /// The own windows and where they came from.
    windows: BTreeMap<String, Origin>,
    /// The proposals that wait for the look.
    pub(super) pending: BTreeMap<String, Pending>,
    /// The prefix of the id of the group that carries a window.
    groups: Option<String>,
    /// The meta keys of the look from before [`kept`]; foreign updates do
    /// not touch them either.
    legacy: fn(&str) -> bool,
}

impl Book {
    /// The book of the look `name`; `filters` when `--output` names
    /// outputs, so that the look lets its windows go when the last of them
    /// goes.
    #[must_use]
    pub fn new(name: &str, filters: bool) -> Self {
        Self {
            name: name.into(),
            filters,
            outputs: Vec::new(),
            windows: BTreeMap::new(),
            pending: BTreeMap::new(),
            groups: None,
            legacy: |_| false,
        }
    }

    /// The group of a window has the id `prefix` and the surface id, so
    /// that a foreign look passes through on it too.
    #[must_use]
    pub fn with_groups(mut self, prefix: &str) -> Self {
        self.groups = Some(prefix.into());
        self
    }

    /// The meta keys of the look from before [`kept`], which a foreign
    /// update does not change.
    #[must_use]
    pub fn with_legacy(mut self, legacy: fn(&str) -> bool) -> Self {
        self.legacy = legacy;
        self
    }

    /// Whether the window is the look's.
    #[must_use]
    pub fn has(&self, id: &str) -> bool {
        self.windows.contains_key(id)
    }

    /// Where an own window came from.
    #[must_use]
    pub fn origin(&self, id: &str) -> Option<&Origin> {
        self.windows.get(id)
    }

    /// The own windows, by surface id.
    pub fn windows(&self) -> impl Iterator<Item = &str> {
        self.windows.keys().map(String::as_str)
    }

    /// Whether the look has no output of `--output`: it takes no window
    /// then and passes a reserved one on.
    #[must_use]
    pub fn released(&self) -> bool {
        self.filters && self.outputs.is_empty()
    }

    /// An output of the look came or changed.
    pub fn set_output(&mut self, name: &str) -> Vec<Change> {
        if !self.outputs.iter().any(|o| o == name) {
            self.outputs.push(name.into());
        }
        vec![Change::Outputs]
    }

    /// The claim filter for a free surface, at the start or when the core
    /// announces it: a surface that the core reserved for another window
    /// manager, or that the look has or waits for, stays; without an
    /// output a surface reserved for the look passes on.
    pub fn claim(&mut self, id: &str, to: Option<&str>, token: Option<String>) -> Vec<Change> {
        let mine = to == Some(self.name.as_str());
        if to.is_some() && !mine || self.has(id) || self.pending.contains_key(id) {
            return Vec::new();
        }
        if self.released() {
            return if mine {
                vec![Change::Apply(pass_on(id, &self.name))]
            } else {
                Vec::new()
            };
        }
        let origin = if mine {
            Origin::Given
        } else {
            Origin::New { token }
        };
        self.pending
            .insert(id.into(), Pending::Adopt(origin.clone()));
        vec![Change::Adopt {
            id: id.into(),
            origin,
        }]
    }

    /// A window that the look kept over a restart: it is attached already.
    pub fn keep(&mut self, id: &str) {
        self.windows.insert(id.into(), Origin::Kept);
    }

    /// What an event changes.
    pub fn apply(&mut self, event: &Event) -> Vec<Change> {
        match event {
            Event::SurfaceCreated {
                surface, to, token, ..
            } => self.claim(surface, to.as_deref(), token.clone()),
            Event::SurfaceDetached { surface } => {
                let mut changes = self.leave(surface, Why::Detached);
                changes.extend(self.claim(surface, None, None));
                changes
            }
            Event::SurfaceDestroyed { surface } => {
                let mut changes = self.leave(surface, Why::Destroyed);
                if !changes.is_empty() {
                    changes.insert(
                        0,
                        Change::Phase {
                            id: surface.clone(),
                            phase: Phase::Closed,
                        },
                    );
                }
                changes
            }
            Event::OutputChanged { output } => self.set_output(&output.name),
            Event::OutputRemoved { name } => self.output_removed(name),
            Event::NodeRequested {
                node, action, args, ..
            } if action == "update" && self.holds(node) => self.pass_through(node, args),
            _ => Vec::new(),
        }
    }

    /// Whether an event is one of the life of a window, which
    /// [`Book::apply`] answers: a surface created, detached or destroyed,
    /// an output changed or removed, a foreign `update` of a node (D58).
    #[must_use]
    pub fn hears(event: &Event) -> bool {
        matches!(
            event,
            Event::SurfaceCreated { .. }
                | Event::SurfaceDetached { .. }
                | Event::SurfaceDestroyed { .. }
                | Event::OutputChanged { .. }
                | Event::OutputRemoved { .. }
        ) || matches!(event, Event::NodeRequested { action, .. } if action == "update")
    }

    /// The app of a surface that an event announces: `Some` for a
    /// surface created, with the app id that it has, if any.
    #[must_use]
    pub fn announced_app(event: &Event) -> Option<Option<&str>> {
        match event {
            Event::SurfaceCreated { app_id, .. } => Some(app_id.as_deref()),
            _ => None,
        }
    }

    /// The core took the commands of a proposal.
    pub fn confirm(&mut self, id: &str) -> Vec<Change> {
        match self.pending.remove(id) {
            Some(Pending::Adopt(origin)) => {
                self.windows.insert(id.into(), origin);
                vec![Change::Phase {
                    id: id.into(),
                    phase: Phase::Opened,
                }]
            }
            Some(Pending::Give(to)) => {
                self.windows.remove(id);
                vec![
                    Change::Phase {
                        id: id.into(),
                        phase: Phase::Given,
                    },
                    Change::Forget {
                        id: id.into(),
                        why: Why::Given(to),
                    },
                ]
            }
            None => Vec::new(),
        }
    }

    /// The core refused the commands of a proposal: another window
    /// manager attached the surface first, or a give failed and the
    /// window stays.
    pub fn refused(&mut self, id: &str) -> Vec<Change> {
        self.pending.remove(id);
        Vec::new()
    }

    /// The look let a window go another way, for example by a move that
    /// another client asked for. True when it was the look's.
    pub fn forget(&mut self, id: &str) -> bool {
        self.pending.remove(id);
        self.windows.remove(id).is_some()
    }

    /// The event `<look>.window` of a phase: the surface, the node that
    /// carries it and its rect, if the look knows one.
    #[must_use]
    pub fn phase_event(&self, id: &str, phase: Phase, rect: Option<[f64; 4]>) -> Emit {
        Emit {
            event: format!("{}.window", self.name),
            data: json!({"surface": id, "node": self.node_of(id), "phase": phase, "rect": rect}),
        }
    }

    /// The node that carries a window: its group, or the surface.
    fn node_of(&self, id: &str) -> String {
        self.groups
            .as_ref()
            .map_or_else(|| id.into(), |prefix| format!("{prefix}{id}"))
    }

    /// Takes a window out of the book.
    fn leave(&mut self, id: &str, why: Why) -> Vec<Change> {
        self.pending.remove(id);
        if self.windows.remove(id).is_none() {
            return Vec::new();
        }
        vec![Change::Forget { id: id.into(), why }]
    }

    /// An output went; the last one of `--output` lets every window go.
    fn output_removed(&mut self, name: &str) -> Vec<Change> {
        self.outputs.retain(|o| o != name);
        let mut changes = vec![Change::Outputs];
        if !self.released() || self.windows.is_empty() {
            return changes;
        }
        let ids: Vec<String> = self.windows.keys().cloned().collect();
        changes.push(Change::Apply(
            ids.iter()
                .map(|id| Command::Remove {
                    id: id.clone(),
                    to: None,
                })
                .collect(),
        ));
        for id in ids {
            changes.extend(self.leave(&id, Why::Released));
        }
        changes
    }

    /// Whether a node is the root of the look, an own window or the group
    /// of one.
    fn holds(&self, node: &str) -> bool {
        let group = (self.groups.as_deref()).and_then(|prefix| node.strip_prefix(prefix));
        node == self.name || self.has(node) || group.is_some_and(|id| self.has(id))
    }

    /// The update of opacity, effects and meta that another client asks
    /// for on a node of the look (D58), without the meta keys of the look.
    /// No relayout sends opacity or effects, so the look of the other
    /// client stays across layouts and restarts.
    fn pass_through(&self, node: &str, args: &Value) -> Vec<Change> {
        let Ok(asked) = serde_json::from_value::<PropsUpdate>(args.clone()) else {
            return Vec::new();
        };
        let meta = asked.meta.map(|mut meta| {
            meta.retain(|key, _| !kept::reserved(key) && !(self.legacy)(key));
            meta
        });
        let props = PropsUpdate {
            opacity: asked.opacity,
            effects: asked.effects,
            meta: meta.filter(|m| !m.is_empty()),
            ..PropsUpdate::default()
        };
        if props == PropsUpdate::default() {
            return Vec::new();
        }
        vec![Change::Apply(vec![Command::Update {
            id: node.into(),
            props,
        }])]
    }
}
