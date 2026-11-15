//! The core writes its log to `$XDG_STATE_HOME/chora/logs/chora.log`: the
//! start, and the last words of a panic, which stderr has once.

#![allow(clippy::unwrap_used, reason = "a test: a failed step fails it")]

use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

fn wait_for(file: &Path, words: &str) -> String {
    let start = Instant::now();
    loop {
        let text = fs::read_to_string(file).unwrap_or_default();
        if text.contains(words) || start.elapsed() > Duration::from_secs(20) {
            return text;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn the_log_file_has_the_start_and_the_last_words_of_a_panic() {
    let work = std::env::temp_dir().join(format!("chora-log-file-{}", std::process::id()));
    let _ = fs::remove_dir_all(&work);
    let runtime = work.join("runtime");
    fs::create_dir_all(&runtime).unwrap();
    let stderr = work.join("stderr.log");
    let mut chora = Command::new(env!("CARGO_BIN_EXE_chora"))
        .args(["--headless", "320x240"])
        .env("XDG_STATE_HOME", work.join("state"))
        .env("XDG_RUNTIME_DIR", &runtime)
        .env("CHORA_TEST_PANIC", "1")
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("RUST_LOG")
        .stdout(Stdio::null())
        .stderr(fs::File::create(&stderr).unwrap())
        .spawn()
        .unwrap();
    let log = work.join("state/chora/logs/chora.log");
    let text = wait_for(&log, "the core goes on without this thread");
    let _ = chora.kill();
    let _ = chora.wait();
    assert!(text.contains("INFO chora: chora runs"), "{text}");
    assert!(
        text.contains("ERROR chora::panic: chora: the thread chora-test panicked"),
        "{text}"
    );
    let stderr = fs::read_to_string(&stderr).unwrap();
    assert_eq!(
        stderr.matches("the thread chora-test panicked").count(),
        1,
        "{stderr}"
    );
    let _ = fs::remove_dir_all(&work);
}
