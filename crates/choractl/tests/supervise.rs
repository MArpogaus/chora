//! `choractl supervise` and `choractl restart` without a core (D42):
//! restart ends the supervised program, and supervise starts it again.

use std::{path::Path, process::Command, time::Duration, time::Instant};

/// Sends a signal to a pid we started ourselves.
#[allow(unsafe_code)] // one `kill`, async-signal-safe by definition
fn signal(pid: u32, sig: libc::c_int) {
    // SAFETY: `kill` of our own child.
    unsafe { libc::kill(pid.cast_signed(), sig) };
}

/// Reads the pid of the supervised program; `None` while the file is
/// missing or unreadable.
fn supervised_pid(runtime: &Path, name: &str) -> Option<u32> {
    let file = runtime
        .join("chora")
        .join("supervise")
        .join(format!("{name}.pid"));
    std::fs::read_to_string(file).ok()?.trim().parse().ok()
}

/// Checks every 50 ms for at most 10 s, until the answer is `Some`.
fn until(mut check: impl FnMut() -> Option<u32>) -> Option<u32> {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if let Some(found) = check() {
            return Some(found);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    None
}

/// A pid runs when `/proc` knows it and its command line contains the
/// given word.
fn running(pid: u32, word: &str) -> bool {
    std::fs::read(format!("/proc/{pid}/cmdline"))
        .is_ok_and(|line| String::from_utf8_lossy(&line).contains(word))
}

/// A private runtime directory for one test process.
fn runtime_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("chora-supervise-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the runtime directory exists");
    dir
}

/// `restart` ends the supervised `sleep 60`, and `supervise` starts it
/// again with a new pid that runs.
#[test]
fn restart_starts_the_supervised_program_again() {
    let runtime = runtime_dir("restart");
    let bin = env!("CARGO_BIN_EXE_choractl");
    let mut supervised = Command::new(bin)
        .args(["supervise", "sleepy", "--", "sleep", "60"])
        .env("XDG_RUNTIME_DIR", &runtime)
        .spawn()
        .expect("supervise starts");
    let first = until(|| supervised_pid(&runtime, "sleepy")).expect("a first pid appears");

    let restarted = Command::new(bin)
        .args(["restart", "sleepy"])
        .env("XDG_RUNTIME_DIR", &runtime)
        .output()
        .expect("restart runs");
    assert!(
        restarted.status.success(),
        "restart: {}",
        String::from_utf8_lossy(&restarted.stderr),
    );

    let second = until(|| supervised_pid(&runtime, "sleepy").filter(|pid| *pid != first))
        .expect("a new pid runs after the restart");
    assert!(running(second, "sleep"), "the new program runs");

    // SIGTERM to `supervise` ends its program too, so nothing sleeps on.
    signal(supervised.id(), libc::SIGTERM);
    supervised.wait().expect("supervise is our child");
    assert!(
        until(|| (!running(second, "sleep")).then_some(1)).is_some(),
        "the program ends with the supervisor",
    );
    let _ = std::fs::remove_dir_all(&runtime);
}

/// A program that ends at once is started six times in total: the
/// first start and five restarts. Then `supervise` gives up with an
/// error.
#[test]
fn supervise_gives_up_after_five_restarts_in_a_minute() {
    let runtime = runtime_dir("limit");
    let bin = env!("CARGO_BIN_EXE_choractl");
    let gave_up = Command::new(bin)
        .args(["supervise", "greedy", "--", "sh", "-c", "exit 3"])
        .env("XDG_RUNTIME_DIR", &runtime)
        .output()
        .expect("supervise runs");
    assert!(
        !gave_up.status.success(),
        "giving up is an error: {}",
        String::from_utf8_lossy(&gave_up.stderr),
    );
    assert!(
        String::from_utf8_lossy(&gave_up.stderr).contains("gave up"),
        "the error names the restart limit",
    );
    assert!(
        supervised_pid(&runtime, "greedy").is_none(),
        "the pid file is gone after giving up",
    );
    let _ = std::fs::remove_dir_all(&runtime);
}
