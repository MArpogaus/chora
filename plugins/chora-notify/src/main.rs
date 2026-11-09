//! chora-notify: the notification server of the freedesktop specification on the
//! session bus.
//!
//! A block of the XP split (D73, `docs/xp-split.md`). It connects through
//! `chora-kit`, subscribes to no event and exports no method. The code of
//! the block moves here in step X2.

use std::process::ExitCode;

use chora_kit::runtime::{self, Plugin, Runtime};
use chora_scene::{
    Event,
    ipc::{ClientError, RpcError},
};
use serde_json::Value;

/// The client name of the plugin, the prefix of its methods.
const NAME: &str = "notify";

/// The plugin.
struct Notify {
    rt: Runtime<()>,
}

impl Plugin for Notify {
    type Action = ();

    fn runtime(&mut self) -> &mut Runtime<()> {
        &mut self.rt
    }

    fn event(&mut self, _: Event) -> Result<(), ClientError> {
        Ok(())
    }

    fn method(&mut self, method: &str, _: Value) -> Result<Value, RpcError> {
        Err(no_method(method))
    }
}

/// The error of a call of a method that the plugin does not export.
fn no_method(method: &str) -> RpcError {
    RpcError::new(
        RpcError::METHOD_NOT_FOUND,
        format!("{NAME} has no method {method}"),
    )
}

fn main() -> ExitCode {
    runtime::run(NAME, || {
        let rt = Runtime::start(NAME, Vec::new(), &[], Vec::new())?;
        Ok(runtime::serve(&mut Notify { rt })?)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_call_of_any_method_fails_with_its_name() {
        let err = no_method("show");
        assert_eq!(err.code, RpcError::METHOD_NOT_FOUND);
        assert_eq!(err.message, "notify has no method show");
    }
}
