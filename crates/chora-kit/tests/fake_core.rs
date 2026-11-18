//! The fake core that plugin tests share: what it answers and what it
//! refuses on demand.

use chora_kit::testing::FakeCore;
use chora_scene::ipc::{ClientError, RpcError};
use serde_json::{Value, json};

fn call(conn: &mut chora_kit::conn::Conn) -> Result<Value, ClientError> {
    conn.call("methods.call", json!({"client": "other", "method": "open"}))
}

/// A refusal that a test plants is seen once, by the next call of that
/// method, and the call after it passes again.
#[test]
fn a_planted_refusal_is_seen_once() {
    let core = FakeCore::start();
    let mut conn = core.connect("probe");
    assert!(call(&mut conn).is_ok(), "no refusal planted");
    core.refuse_next("methods.call");
    let err = call(&mut conn).unwrap_err();
    assert!(
        matches!(&err, ClientError::Rpc(e) if e.code == RpcError::REJECTED),
        "{err:?}"
    );
    assert!(call(&mut conn).is_ok(), "the refusal is used up");
}

/// The selftest: a refusal that is planted for another method does not
/// touch this one, and the fake core keeps what the plugin called.
#[test]
fn a_refusal_of_another_method_is_not_seen() {
    let core = FakeCore::start();
    let mut conn = core.connect("probe");
    core.refuse_next("events.emit");
    assert!(call(&mut conn).is_ok());
    assert_eq!(core.model().lock().unwrap().calls, ["open"]);
}
