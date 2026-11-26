//! The Looks picker of the bar (`chora-tiling --launch looks`) with a
//! stand-in of fuzzel: what it offers and how big its box
//! is (rpt2-2, rpt3-4).
#![allow(clippy::unwrap_used, reason = "a failure is a failed test")]

use std::{os::unix::fs::PermissionsExt as _, path::Path, process::Command};

use chora_kit::testing::{FakeCore, TempDir};
use chora_scene::{Node, NodeKind, Props};
use serde_json::json;

/// A directory with the stand-ins `(name, shell code)` first on a PATH
/// that has the tools of the shell after.
fn stand_ins(name: &str, fakes: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new(&format!("chora-picker-{name}"));
    std::fs::create_dir_all(&dir).unwrap();
    for (fake, code) in fakes {
        let path = dir.join(fake);
        std::fs::write(&path, format!("#!/bin/sh\n{code}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    dir
}

fn path_of(dir: &Path) -> String {
    format!("{}:/usr/bin:/bin", dir.display())
}

/// A core that lists xp and mc as clients and, when `focus`, has a
/// focused window of tiling.
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
        "clients.list" => Some(json!([{"name": "tiling"}, {"name": "xp"}, {"name": "mc"}])),
        "scene.query" if params["filter"]["focused"] == true => Some(if focus {
            json!([serde_json::to_value(&window).unwrap()])
        } else {
            json!([])
        }),
        _ => None,
    });
    core
}

/// What the picker asked fuzzel: its arguments and the rows on stdin.
fn picked(tag: &str, focus: bool) -> (String, Vec<String>) {
    let core = core(focus);
    let bin = stand_ins(
        tag,
        &[(
            "fuzzel",
            r#"echo "$*" >"$(dirname "$0")/args"; cat >"$(dirname "$0")/rows""#,
        )],
    );
    let status = Command::new(env!("CARGO_BIN_EXE_chora-tiling"))
        .args(["--launch", "looks"])
        .env_clear()
        .env(chora_scene::ipc::SOCKET_ENV, core.socket())
        .env("PATH", path_of(&bin))
        .env("LANG", "C")
        .status()
        .unwrap();
    assert!(status.success());
    let args = std::fs::read_to_string(bin.join("args")).unwrap();
    let rows = std::fs::read_to_string(bin.join("rows")).unwrap();
    (args, rows.lines().map(str::to_owned).collect())
}

/// rpt2-2: without a focused window no row moves anything.
#[test]
fn the_picker_offers_no_move_without_a_window_rpt2_2() {
    let (_, rows) = picked("no-window", false);
    assert_eq!(rows, ["No window to move"]);
}

#[test]
fn the_picker_offers_the_moves_with_a_window() {
    let (_, rows) = picked("window", true);
    assert_eq!(
        rows,
        [
            "Move window to the xp desktop",
            "Move window to the media center desktop"
        ]
    );
}

/// rpt3-4: the box of the picker has the lines of its rows, as the power
/// menu passes its `--lines`.
#[test]
fn the_picker_box_fits_its_rows_rpt3_4() {
    let lines = |args: &str| -> Option<usize> {
        let words: Vec<&str> = args.split_whitespace().collect();
        let at = words.iter().position(|w| *w == "--lines")?;
        words.get(at + 1)?.parse().ok()
    };
    let (args, _) = picked("lines-two", true);
    assert_eq!(lines(&args), Some(2), "{args}");
    let (args, _) = picked("lines-one", false);
    assert_eq!(lines(&args), Some(1), "{args}");
}
