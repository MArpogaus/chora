//! The connection skeleton of a plugin (`kit::conn`) on the fake core: the
//! name of the connection, its activation tokens, its queries and the
//! transactions that a surface gone away may fail.
#![allow(clippy::unwrap_used, reason = "a failure is a failed test")]

use std::time::Duration;

use chora_kit::testing::FakeCore;
use chora_scene::{Command, NodeKind, PropsUpdate, ROOT, ipc::RpcError};
use serde_json::{Value, json};

#[test]
fn a_connection_is_named_for_its_log_lines() {
    let core = FakeCore::start();
    assert_eq!(core.connect("probe").name(), "probe");
}

/// Without a minting core there is no token; with one, each call mints a
/// new token.
#[test]
fn a_token_is_minted_only_when_the_core_mints_one() {
    let core = FakeCore::start();
    let mut conn = core.connect("probe");
    assert_eq!(conn.activation_token(), None, "the core mints none");
    core.mint_tokens();
    let first = conn.activation_token().expect("a token");
    let second = conn.activation_token().expect("a token");
    assert!(first.starts_with("fake-token-"), "{first}");
    assert_ne!(first, second, "each call mints a new token");
}

/// A query finds the node of its id, and an id that no node has finds
/// nothing.
#[test]
fn a_query_finds_the_node_of_its_id() {
    let core = FakeCore::start();
    let mut conn = core.connect("probe");
    conn.apply(vec![Command::Create {
        id: "n1".into(),
        kind: NodeKind::Group,
        parent: ROOT.into(),
        props: PropsUpdate::default(),
    }])
    .unwrap();
    let found = conn.query("id:n1").unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, "n1");
    assert_eq!(
        conn.query("id:missing").unwrap(),
        [] as [chora_scene::Node; 0]
    );
}

/// A transaction that the core rejects is returned by `try_apply`, and
/// `apply` logs it and passes: a surface may be gone before the answer.
#[test]
fn a_rejected_transaction_is_returned_or_logged_not_failed() {
    let core = FakeCore::start();
    let mut conn = core.connect("probe");
    let attach = || {
        vec![Command::Attach {
            surface: "nope".into(),
            parent: ROOT.into(),
        }]
    };
    let err = conn.try_apply(attach()).unwrap();
    assert_eq!(err.map(|e| e.code), Some(RpcError::REJECTED));
    assert!(
        conn.apply(attach()).is_ok(),
        "apply only logs the rejection"
    );
}

/// A call answers within its time, and the answer is the one of the core.
#[test]
fn a_call_answers_within_its_time() {
    let core = FakeCore::start();
    let mut conn = core.connect("probe");
    let answer: Value = conn
        .call_within(
            "input.grab",
            json!({"binding": "Super+x"}),
            Duration::from_secs(5),
        )
        .unwrap();
    assert_eq!(answer, json!("Super+x"));
}
