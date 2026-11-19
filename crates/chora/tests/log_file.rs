//! The core writes its log to `$XDG_STATE_HOME/chora/logs/chora.log`: the
//! start, and the last words of a panic, which stderr has once. A second
//! chora leaves the log of the first where it is.

#![allow(clippy::unwrap_used, reason = "a test: a failed step fails it")]

use std::{
    fs,
    path::Path,
    process::{Child, Command, Stdio},
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

fn headless(work: &Path, runtime: &str, stderr: &Path) -> Child {
    let runtime = work.join(runtime);
    fs::create_dir_all(&runtime).unwrap();
    Command::new(env!("CARGO_BIN_EXE_chora"))
        .args(["--headless", "320x240"])
        .env("XDG_STATE_HOME", work.join("state"))
        .env(chora_scene::paths::RUNTIME_ENV, &runtime)
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("RUST_LOG")
        .stdout(Stdio::null())
        .stderr(fs::File::create(stderr).unwrap())
        .spawn()
        .unwrap()
}

#[test]
fn a_second_chora_leaves_the_log_of_the_first() {
    let work = chora_scene::testing::TempDir::new("chora-log-second");
    let log = work.join("state/chora/logs/chora.log");
    let mut first = headless(&work, "first", &work.join("first.log"));
    let first_log = wait_for(&log, "chora runs");
    let second_stderr = work.join("second.log");
    let mut second = headless(&work, "second", &second_stderr);
    let words = wait_for(&second_stderr, "chora runs");
    let open = fs::read_dir(format!("/proc/{}/fd", first.id()))
        .unwrap()
        .filter_map(|fd| fs::read_link(fd.ok()?.path()).ok())
        .any(|target| target == log);
    for chora in [&mut first, &mut second] {
        let _ = chora.kill();
        let _ = chora.wait();
    }
    assert!(first_log.contains("chora runs"), "{first_log}");
    assert!(words.contains("no log file"), "{words}");
    assert!(open, "the first keeps chora.log open");
    assert!(
        fs::read_to_string(&log).unwrap().contains("chora runs"),
        "the log of the first stays chora.log"
    );
    assert!(!work.join("state/chora/logs/chora.log.1").exists());
    let _ = fs::remove_dir_all(&work);
}

#[test]
fn the_log_file_has_the_start_and_the_last_words_of_a_panic() {
    let work = chora_scene::testing::TempDir::unmade("chora-log-file");
    let runtime = work.join("runtime");
    fs::create_dir_all(&runtime).unwrap();
    let stderr = work.join("stderr.log");
    let mut chora = Command::new(env!("CARGO_BIN_EXE_chora"))
        .args(["--headless", "320x240"])
        .env("XDG_STATE_HOME", work.join("state"))
        .env(chora_scene::paths::RUNTIME_ENV, &runtime)
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
