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

/// A responder answers the requests that it knows and leaves the rest to
/// the fake core; a recorder keeps a line per request, in order.
#[test]
fn a_responder_answers_and_a_recorder_keeps_the_requests() {
    let core = FakeCore::start();
    core.respond(|method, params| match method {
        "output.list" => Some(json!([{"name": "X"}])),
        "scene.query" => Some(json!([params["selector"]])),
        _ => None,
    });
    let log = core.record(|method, _| method.starts_with("output.").then(|| method.to_owned()));
    let mut conn = core.connect("probe");
    let outputs: Value = conn.call("output.list", Value::Null).unwrap();
    assert_eq!(outputs, json!([{"name": "X"}]));
    let found: Value = conn
        .call("scene.query", json!({"selector": "kind:text"}))
        .unwrap();
    assert_eq!(found, json!(["kind:text"]));
    let echoed: Value = conn.call("input.grab", json!({"binding": "a"})).unwrap();
    assert_eq!(echoed, "a", "the fake core answers what no responder does");
    assert_eq!(*log.lock().unwrap(), ["output.list"], "one line a request");
}

/// A held binding is refused every time, another is granted, and the
/// fake core keeps the grabs that went through.
#[test]
fn a_held_binding_is_refused_and_a_grab_is_kept() {
    let core = FakeCore::start();
    core.hold("Super+x");
    let mut conn = core.connect("probe");
    let held: Result<Value, _> = conn.call("input.grab", json!({"binding": "Super+x"}));
    assert!(
        matches!(&held, Err(ClientError::Rpc(e)) if e.code == RpcError::REJECTED),
        "{held:?}"
    );
    let granted: Result<Value, _> = conn.call("input.grab", json!({"binding": "Super+y"}));
    assert!(granted.is_ok());
    assert_eq!(core.model().lock().unwrap().grabs, [json!("Super+y")]);
}

/// The window manager under a point is another client's: the smallest claim
/// of the connection itself is left out, so a look needs no filter of its
/// own name.
#[test]
fn the_claimant_under_a_point_is_never_the_connection_itself() {
    use chora_scene::{Command, NodeKind, PropsUpdate, ROOT};
    let claim = |id: &str, size: f64| Command::Create {
        id: id.into(),
        kind: NodeKind::Group,
        parent: ROOT.into(),
        props: PropsUpdate {
            claim: Some(Some([0.0, 0.0, size, size])),
            ..PropsUpdate::default()
        },
    };
    let core = FakeCore::start().selecting();
    let mut other = core.connect("other");
    other.apply(vec![claim("other", 800.0)]).unwrap();
    let mut mine = core.connect("mine");
    mine.apply(vec![claim("mine", 100.0)]).unwrap();
    assert_eq!(
        mine.claimant_at([10.0, 10.0]).unwrap().as_deref(),
        Some("other"),
        "under both claims, the other one"
    );
    assert_eq!(
        mine.claimant_at([900.0, 900.0]).unwrap(),
        None,
        "outside every claim of the others"
    );
}

/// The runtime of the helper is a plugin on a connection to the fake core:
/// its requests reach the core, and it holds no grab of its own.
#[test]
fn a_runtime_of_the_fake_core_reaches_the_core() {
    let core = FakeCore::start();
    let methods = core.record(|method, _| Some(method.to_owned()));
    let mut rt: chora_kit::runtime::Runtime<()> = core.runtime("probe");
    assert!(!rt.holds("Super+x"), "no grab of its own");
    rt.subscribe(&["window.opened"]).unwrap();
    assert_eq!(
        *methods.lock().unwrap(),
        ["hello", "events.subscribe"],
        "the connect says hello, then the request follows"
    );
}

/// `outputs` plants the answer of `output.list`; without it the list is
/// empty.
#[test]
fn the_fake_core_answers_output_list_with_its_outputs() {
    let core = FakeCore::start();
    let mut rt: chora_kit::runtime::Runtime<()> = core.runtime("probe");
    assert!(rt.all_outputs().unwrap().is_empty(), "none by default");
    core.outputs(serde_json::json!([{
        "name": "DP-1", "position": [0, 0], "size": [1920, 1080],
        "scale": 1.0, "camera": {}, "work_area": [0, 0, 1920, 1080]
    }]));
    let outputs = rt.all_outputs().unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].name, "DP-1");
}
