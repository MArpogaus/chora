//! The notice over a fake core: show, replace, dismiss, and a server that
//! is gone. The fake core records each call and answers a planted id.

#![allow(clippy::unwrap_used, reason = "a test: a failure is a failed test")]

use chora_kit::{
    notice::{self, Gone, Notice, Urgency},
    testing::FakeCore,
};
use serde_json::json;

fn notice(summary: &str) -> Notice<'_> {
    Notice {
        app: "probe",
        icon: "",
        summary,
        body: "the text",
        expire_ms: -1,
        urgency: Urgency::Normal,
    }
}

fn calls(core: &FakeCore) -> Vec<serde_json::Value> {
    core.model().lock().unwrap().called.clone()
}

/// A notice is one call of `notify.show` on the client `notify`, with
/// every field of the notice, and its answer is the id.
#[test]
fn a_notice_shows_and_gives_its_id() {
    let core = FakeCore::start();
    core.answer_with(notice::SHOW, json!({"id": 7}));
    let mut conn = core.connect("probe");
    assert_eq!(notice::show(&mut conn, None, &notice("Hello")), Ok(7));
    assert_eq!(
        calls(&core),
        [
            json!({"client": "notify", "method": "notify.show", "params": {
            "app": "probe", "replaces": null, "icon": "", "summary": "Hello",
            "body": "the text", "expire_ms": -1, "urgency": "normal"}})
        ]
    );
}

/// With the id of an earlier notice, the call replaces it, and a critical
/// notice says so.
#[test]
fn a_notice_replaces_an_earlier_one() {
    let core = FakeCore::start();
    core.answer_with(notice::SHOW, json!({"id": 7}));
    let mut conn = core.connect("probe");
    let urgent = Notice {
        urgency: Urgency::Critical,
        ..notice("Again")
    };
    assert_eq!(notice::show(&mut conn, Some(7), &urgent), Ok(7));
    let sent = &calls(&core)[0]["params"];
    assert_eq!(
        (sent["replaces"].clone(), sent["urgency"].clone()),
        (json!(7), json!("critical"))
    );
}

/// A dismissal is one call of `notify.close` with the id.
#[test]
fn a_notice_is_dismissed_by_its_id() {
    let core = FakeCore::start();
    let mut conn = core.connect("probe");
    assert_eq!(notice::dismiss(&mut conn, 7), Ok(()));
    assert_eq!(
        calls(&core),
        [json!({"client": "notify", "method": "notify.close", "params": {"id": 7}})]
    );
}

/// A server that is gone is an honest `Gone`, not a panic, for a show
/// and for a dismissal, and the next call goes through again.
#[test]
fn a_server_that_is_gone_is_told() {
    let core = FakeCore::start();
    core.answer_with(notice::SHOW, json!({"id": 8}));
    let mut conn = core.connect("probe");
    core.refuse_next("methods.call");
    assert!(matches!(
        notice::show(&mut conn, None, &notice("Lost")),
        Err(Gone(_))
    ));
    assert_eq!(
        notice::show_or_log(&mut conn, None, &notice("Back")),
        Some(8)
    );
    core.refuse_next("methods.call");
    assert!(matches!(notice::dismiss(&mut conn, 8), Err(Gone(_))));
    assert_eq!(
        notice::show_or_log(&mut conn, None, &notice("Back")),
        Some(8)
    );
    core.refuse_next("methods.call");
    assert_eq!(notice::show_or_log(&mut conn, None, &notice("Lost")), None);
}
