//! The Looks picker of the bar (`chora-tiling --launch looks`) with stand-ins
//! of fuzzel, choractl and notify-send: what it offers and how big its box
//! is (rpt2-2, rpt3-4), and what a refused give says (rpt2-2).
#![allow(clippy::unwrap_used, reason = "a failure is a failed test")]

use std::{
    os::unix::fs::PermissionsExt as _,
    path::{Path, PathBuf},
    process::Command,
};

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
        .env(chora_scene::ipc::SOCKET_ENV, core.socket())
        .env("PATH", path_of(&bin))
        .env("LANG", "C")
        .env_remove("LC_ALL")
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

fn flavor_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../flavors/tiling")
}

/// The notice that give.sh shows when `choractl give` refuses.
fn refused_notice(lang: &str) -> Vec<String> {
    let bin = stand_ins(
        &format!("give-{lang}"),
        &[
            (
                "choractl",
                r#"echo "choractl: no focused window" >&2; exit 1"#,
            ),
            (
                "notify-send",
                r#"for a in "$@"; do echo "$a"; done >"$(dirname "$0")/notice""#,
            ),
        ],
    );
    let status = Command::new("sh")
        .arg(flavor_dir().join("give.sh"))
        .arg("xp")
        .env_clear()
        .env("PATH", path_of(&bin))
        .env("LANG", lang)
        .status()
        .unwrap();
    assert!(!status.success());
    std::fs::read_to_string(bin.join("notice"))
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect()
}

/// rpt2-2: no tool name in a notice, and its words follow the language
/// of the title.
#[test]
fn a_refused_give_names_no_tool_in_its_notice_rpt2_2() {
    for lang in ["C", "de_DE.UTF-8"] {
        let notice = refused_notice(lang);
        let text = notice.join(" ");
        assert!(!text.contains("choractl"), "{lang}: {text}");
        assert!(!text.contains("no focused window"), "{lang}: {text}");
    }
    let german = refused_notice("de_DE.UTF-8").join(" ");
    assert!(german.contains("Fenster"), "{german}");
}
