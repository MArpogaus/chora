//! The Apps list of the bar (`chora-tiling --launch fuzzel`) with a
//! stand-in of fuzzel: the rows that move a window are there only when a
//! window has the focus, as the Looks picker has them (rtmr1-3).
#![allow(clippy::unwrap_used, reason = "a failure is a failed test")]

use std::{os::unix::fs::PermissionsExt as _, path::Path, process::Command};

use chora_kit::testing::{FakeCore, TempDir};
use chora_scene::{Node, NodeKind, Props};
use serde_json::json;

/// A core that lists xp as a client and, when `focus`, has a focused
/// window of tiling.
fn core(focus: bool) -> FakeCore {
    let core = FakeCore::start();
    let window = Node {
        id: "s1".into(),
        kind: NodeKind::Surface,
        owner: Some("tiling".into()),
        parent: None,
        children: Vec::new(),
        props: Props::default(),
        surface: None,
        to: None,
    };
    core.respond(move |method, params| match method {
        "clients.list" => Some(json!([{"name": "tiling"}, {"name": "xp"}])),
        "scene.query" if params["filter"]["focused"] == true => Some(if focus {
            json!([serde_json::to_value(&window).unwrap()])
        } else {
            json!([])
        }),
        _ => None,
    });
    core
}

/// The rows that the Apps list showed fuzzel, one a line.
fn rows(tag: &str, focus: bool) -> Vec<String> {
    let core = core(focus);
    let bin = TempDir::new(&format!("chora-apps-{tag}"));
    std::fs::create_dir_all(&bin).unwrap();
    for (name, code) in [
        ("fuzzel", r#"cat >"$(dirname "$0")/rows""#),
        ("chora-tiling", "exit 0"),
    ] {
        let path = bin.join(name);
        std::fs::write(&path, format!("#!/bin/sh\n{code}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let flavor = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flavors/tiling");
    let status = Command::new(env!("CARGO_BIN_EXE_chora-tiling"))
        .args(["--launch", "fuzzel"])
        .env_clear()
        .env(chora_scene::ipc::SOCKET_ENV, core.socket())
        .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
        .env("CHORA_FLAVOR_DIR", flavor)
        .env("LANG", "C")
        .status()
        .unwrap();
    assert!(status.success());
    let rows = std::fs::read_to_string(bin.join("rows")).unwrap();
    rows.lines()
        .map(|l| l.split('\0').next().unwrap_or(l).to_owned())
        .collect()
}

#[test]
fn the_apps_list_offers_the_moves_with_a_window() {
    let rows = rows("window", true);
    let moves = rows
        .iter()
        .filter(|r| r.starts_with("Move window to"))
        .count();
    assert_eq!(moves, 3, "{rows:?}");
}

/// rtmr1-3: without a focused window no row moves anything, and the
/// other rows of the flavor stay.
#[test]
fn the_apps_list_offers_no_move_without_a_window_rtmr1_3() {
    let rows = rows("no-window", false);
    let moves: Vec<&String> = rows
        .iter()
        .filter(|r| r.starts_with("Move window to"))
        .collect();
    assert!(moves.is_empty(), "{moves:?}");
    assert!(rows.iter().any(|r| r == "Dark theme"), "{rows:?}");
}
