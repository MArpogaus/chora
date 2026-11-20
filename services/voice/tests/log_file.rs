//! Only the agent writes `voice.log`: a CLI call of chora-voice leaves
//! the log of a running agent where it is.

#![allow(clippy::unwrap_used, reason = "a test: a failed step fails it")]

use std::{fs, process::Command};

#[test]
fn a_cli_call_leaves_the_log_of_the_agent() {
    let work = chora_kit::testing::TempDir::unmade("chora-voice-log");
    let logs = work.join("state/chora/logs");
    fs::create_dir_all(&logs).unwrap();
    fs::create_dir_all(work.join("config")).unwrap();
    let log = logs.join("voice.log");
    fs::write(&log, "the agent runs\n").unwrap();
    for args in [
        &["--version"][..],
        &["--help"],
        &["--print-config"],
        &["gate", "/nonexistent.wav"],
    ] {
        Command::new(env!("CARGO_BIN_EXE_chora-voice"))
            .args(args)
            .env("XDG_STATE_HOME", work.join("state"))
            .env("XDG_CONFIG_HOME", work.join("config"))
            .env_remove("RUST_LOG")
            .output()
            .unwrap();
        assert_eq!(
            fs::read_to_string(&log).unwrap(),
            "the agent runs\n",
            "chora-voice {args:?}"
        );
        assert!(!logs.join("voice.log.1").exists(), "chora-voice {args:?}");
    }
    let _ = fs::remove_dir_all(&work);
}
