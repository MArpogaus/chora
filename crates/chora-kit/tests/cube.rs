//! The cube role on the fake core (`kit::cube`): who exports the role, the
//! listing of a cube host, and a turn that the host refuses.
#![allow(clippy::unwrap_used, reason = "a failure is a failed test")]

use chora_kit::{
    cube::{self, Face, Listing, Turn},
    testing::FakeCore,
};
use serde_json::{Value, json};

/// The exporter is the client that lists the method `cube.list`; the other
/// methods of other clients are not the role.
#[test]
fn the_exporter_is_the_client_that_lists_the_cube_method() {
    let core = FakeCore::start();
    core.respond(|method, _| {
        (method == "methods.list").then(|| {
            json!([
                {"name": "input.grab", "client": "shell"},
                {"name": cube::LIST, "client": "xp"},
            ])
        })
    });
    let mut conn = core.connect("look");
    assert_eq!(cube::exporter(&mut conn).unwrap(), Some("xp".to_owned()));
}

/// Without a client that lists `cube.list`, there is no exporter, and the
/// look switches its desks flat.
#[test]
fn no_exporter_when_no_client_lists_the_cube_method() {
    let core = FakeCore::start();
    core.respond(|method, _| {
        (method == "methods.list").then(|| json!([{"name": "input.grab", "client": "shell"}]))
    });
    let mut conn = core.connect("look");
    assert_eq!(cube::exporter(&mut conn).unwrap(), None);
}

/// The listing of a host is its faces in turn order, the front, whether it
/// turns and the mode; the request goes to the host with no parameters.
#[test]
fn a_listing_is_the_faces_in_turn_order_with_the_front_and_the_mode() {
    let core = FakeCore::start();
    let sent = core.record(|method, params| (method == "methods.call").then(|| params.to_string()));
    core.respond(|method, _| {
        (method == "methods.call").then(|| {
            json!({
                "faces": [
                    {"output": "VIRTUAL-1", "manager": "xp", "health": "up"},
                    {"output": "VIRTUAL-2", "manager": "xp", "health": "starting"},
                ],
                "front": 1,
                "turning": true,
                "mode": "navigate",
            })
        })
    });
    let mut conn = core.connect("look");
    let listing = cube::list(&mut conn, "xp").unwrap();
    let face = |output: &str, health: &str| Face {
        output: output.into(),
        manager: "xp".into(),
        health: health.into(),
    };
    assert_eq!(
        listing,
        Listing {
            faces: vec![face("VIRTUAL-1", "up"), face("VIRTUAL-2", "starting")],
            front: 1,
            turning: true,
            mode: "navigate".into(),
        }
    );
    let sent = sent.lock().unwrap();
    assert_eq!(sent.len(), 1, "one request for the listing");
    let call: Value = serde_json::from_str(&sent[0]).unwrap();
    assert_eq!(
        call,
        json!({"client": "xp", "method": cube::LIST, "params": {}})
    );
}

/// A turn goes to the host with its parameters; a host that refuses it
/// answers false and the turn is not taken; a call that the core refuses
/// is an error.
#[test]
fn a_turn_the_host_refuses_is_not_taken() {
    let core = FakeCore::start();
    let sent = core.record(|method, params| (method == "methods.call").then(|| params.to_string()));
    core.respond(|method, _| (method == "methods.call").then_some(json!(false)));
    let mut conn = core.connect("look");
    let by = Turn {
        by: Some(1),
        ..Turn::default()
    };
    assert!(
        !cube::turn(&mut conn, "xp", by).unwrap(),
        "the host refuses the turn"
    );
    let call: Value = serde_json::from_str(&sent.lock().unwrap()[0]).unwrap();
    assert_eq!(
        call,
        json!({"client": "xp", "method": cube::TURN, "params": {"by": 1}}),
        "only the parameters that are set are sent"
    );

    core.refuse_next("methods.call");
    assert!(
        cube::turn(&mut conn, "xp", by).is_err(),
        "a refused call is an error"
    );
}
