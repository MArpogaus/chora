//! Which windows are the look's (arch plan 1.1, 2.2).
#![allow(
    missing_docs,
    dead_code,
    clippy::missing_docs_in_private_items,
    clippy::unused_self,
    clippy::needless_pass_by_value,
    clippy::unnecessary_wraps,
    reason = "red: the cells come before the book"
)]

use std::collections::BTreeMap;

use chora_scene::{Command, Event, ipc::Emit};
use serde::Serialize;

/// Where a window of the look came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase", tag = "origin")]
pub enum Origin {
    New {
        #[serde(skip_serializing_if = "Option::is_none")]
        token: Option<String>,
    },
    Given,
    Kept,
}

/// Why a window left the look.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Why {
    Destroyed,
    Detached,
    Given(String),
    Released,
}

/// A step in the life of a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Opened,
    Closed,
    Given,
}

/// What the look does after an event.
#[derive(Debug, Clone, PartialEq)]
pub enum Change {
    Adopt { id: String, origin: Origin },
    Forget { id: String, why: Why },
    Outputs,
    Phase { id: String, phase: Phase },
    Apply(Vec<Command>),
}

#[derive(Debug, Clone)]
pub(super) enum Pending {
    Adopt(Origin),
    Give(String),
}

/// The book of a look.
#[derive(Debug)]
pub struct Book {
    pub(super) name: String,
    pub(super) pending: BTreeMap<String, Pending>,
    windows: BTreeMap<String, Origin>,
}

impl Book {
    #[must_use]
    pub fn new(name: &str, _filters: bool) -> Self {
        Self {
            name: name.into(),
            pending: BTreeMap::new(),
            windows: BTreeMap::new(),
        }
    }
    #[must_use]
    pub fn with_groups(self, _prefix: &str) -> Self {
        self
    }
    #[must_use]
    pub fn with_legacy(self, _legacy: fn(&str) -> bool) -> Self {
        self
    }
    #[must_use]
    pub fn has(&self, id: &str) -> bool {
        self.windows.contains_key(id)
    }
    #[must_use]
    pub fn origin(&self, id: &str) -> Option<&Origin> {
        self.windows.get(id)
    }
    pub fn set_output(&mut self, _name: &str) -> Vec<Change> {
        Vec::new()
    }
    pub fn claim(&mut self, _id: &str, _to: Option<&str>, _token: Option<String>) -> Vec<Change> {
        Vec::new()
    }
    pub fn keep(&mut self, _id: &str) {}
    pub fn apply(&mut self, _event: &Event) -> Vec<Change> {
        Vec::new()
    }
    pub fn confirm(&mut self, _id: &str) -> Vec<Change> {
        Vec::new()
    }
    pub fn refused(&mut self, _id: &str) -> Vec<Change> {
        Vec::new()
    }
    pub fn forget(&mut self, _id: &str) -> bool {
        false
    }
    #[must_use]
    pub fn phase_event(&self, id: &str, phase: Phase, rect: Option<[f64; 4]>) -> Emit {
        let _ = (id, phase, rect);
        Emit {
            event: String::new(),
            data: serde_json::Value::Null,
        }
    }
}
