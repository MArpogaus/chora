//! The notices of `flavors/tiling/give.sh` and `look.sh` when `choractl`
//! refuses (rpt2-2, rtmr1-1): in words of the person, no tool name, no
//! English reason.
#![allow(clippy::unwrap_used, reason = "a failure is a failed test")]

use std::{os::unix::fs::PermissionsExt as _, path::PathBuf, process::Command};

use chora_scene::testing::TempDir;

fn flavor_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../flavors/tiling")
}

/// The notice that give.sh shows when a stand-in of `choractl give`
/// refuses, one argument a line, in the language `lang`.
fn refused_notice(lang: &str) -> Vec<String> {
    notice_of("give.sh", "xp", lang)
}

/// The notice that the script `script` of the tiling flavor shows with
/// the argument `arg` when a stand-in of `choractl` refuses, one argument
/// a line, in the language `lang`.
fn notice_of(script: &str, arg: &str, lang: &str) -> Vec<String> {
    let bin = TempDir::new(&format!("chora-notice-{script}-{lang}"));
    std::fs::create_dir_all(&bin).unwrap();
    for (name, code) in [
        (
            "choractl",
            r#"echo "choractl: no focused window" >&2; exit 1"#,
        ),
        (
            "notify-send",
            r#"for a in "$@"; do echo "$a"; done >"$(dirname "$0")/notice""#,
        ),
    ] {
        let path = bin.join(name);
        std::fs::write(&path, format!("#!/bin/sh\n{code}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let status = Command::new("sh")
        .arg(flavor_dir().join(script))
        .arg(arg)
        .env_clear()
        .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
        .env("LANG", lang)
        .status()
        .unwrap();
    assert!(!status.success());
    let notice = std::fs::read_to_string(bin.join("notice")).unwrap();
    notice.lines().map(str::to_owned).collect()
}

/// No tool name in a notice, and its words follow the language of the
/// title.
#[test]
fn a_refused_give_names_no_tool_in_its_notice_rpt2_2() {
    for lang in ["C", "de_DE.UTF-8"] {
        let text = refused_notice(lang).join(" ");
        assert!(!text.contains("choractl"), "{lang}: {text}");
        assert!(!text.contains("no focused window"), "{lang}: {text}");
    }
    assert_eq!(
        refused_notice("C"),
        [
            "-a",
            "Desktop",
            "Window not moved",
            "The window stayed where it was."
        ]
    );
    let german = refused_notice("de_DE.UTF-8").join(" ");
    assert!(german.contains("Fenster nicht verschoben"), "{german}");
    assert!(german.contains("Das Fenster ist geblieben"), "{german}");
}

/// rtmr1-1: a refused look change says what stayed, as a person says it,
/// and not the command line of the tool.
#[test]
fn a_refused_look_change_names_no_tool_in_its_notice_rtmr1_1() {
    for lang in ["C", "de_DE.UTF-8"] {
        let text = notice_of("look.sh", "dark", lang).join(" ");
        assert!(!text.contains("choractl"), "{lang}: {text}");
        assert!(!text.contains("settings.set"), "{lang}: {text}");
        assert!(!text.contains("no focused window"), "{lang}: {text}");
    }
    assert_eq!(
        notice_of("look.sh", "dark", "C"),
        [
            "-a",
            "Look",
            "Theme not changed",
            "The look stayed as it was."
        ]
    );
    let german = notice_of("look.sh", "dark", "de_DE.UTF-8").join(" ");
    assert!(german.contains("Design nicht geändert"), "{german}");
    assert!(
        german.contains("Das Design ist geblieben, wie es war."),
        "{german}"
    );
}
