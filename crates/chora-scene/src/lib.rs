//! Scene model, commands, events and IPC types of the chora compositor.
//!
//! This crate has no Smithay dependency. The core, `choractl` and plugins
//! share it as the one definition of the API.

mod binding;
mod command;
mod event;
pub mod ipc;
mod node;
mod scene;
mod selector;
mod transform;

pub use binding::{Binding, BindingError, Modifiers};
pub use command::{Command, Effect, ToplevelState};
pub use event::{Event, OutputInfo};
pub use node::{InputMode, Node, NodeKind, Props, PropsUpdate, ROOT, SurfaceInfo};
pub use scene::{Placed, Scene, SceneError, TransactionError};
pub use selector::{Selector, SelectorError};
pub use transform::Transform;

#[cfg(test)]
mod tests;
