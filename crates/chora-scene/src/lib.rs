//! Scene model, commands, events and IPC types of the chora compositor.
//!
//! This crate has no Smithay dependency. The core, `choractl` and plugins
//! share it as the one definition of the API.

mod animation;
mod binding;
mod command;
mod event;
pub mod ipc;
mod node;
mod scene;
mod selector;
mod transform;

pub use animation::{Easing, MAX_DURATION};
pub use binding::{Binding, BindingError, Modifiers};
pub use command::{Command, Effect, ToplevelState};
pub use event::{Event, NAMES as EVENT_NAMES, OutputInfo, ScrollSource, SurfaceRequest};
pub use node::{
    Border, Clip, Decoration, InputMode, MAX_EFFECTS, MAX_MESH_CELLS, MAX_NODE_TEXTURES,
    MAX_UNIFORMS, Mesh, Node, NodeKind, NodeTexture, Props, PropsUpdate, ROOT, ShaderEffect,
    SurfaceInfo, Uniform,
};
pub use scene::{
    MAX_CAMERAS, MAX_NODES, OVERLAY, Placed, Scene, SceneError, TransactionError, overlay_id,
};
pub use selector::{Selector, SelectorError};
pub use transform::Transform;

#[cfg(test)]
mod tests;
