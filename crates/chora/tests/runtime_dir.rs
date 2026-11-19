//! A runtime directory that group or other can write is refused at the
//! start: chora binds no socket in it, not the Wayland one either. The
//! Wayland socket goes away with the process, so the test reads the line
//! that smithay logs when it binds one.

#![allow(clippy::unwrap_used, reason = "a test: a failed step fails it")]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

/// A scratch directory with a mode, for the runtime directory.
fn runtime(name: &str, mode: u32) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("chora-rt-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(mode)).unwrap();
    dir
}

/// The state home beside the runtime directory, not in it.
fn state(runtime: &Path) -> PathBuf {
    let mut name = runtime.file_name().unwrap().to_owned();
    name.push("-state");
    runtime.with_file_name(name)
}

/// Runs a headless chora with `runtime` as its runtime directory.
fn run(runtime: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_chora"))
        .args(["--headless", "320x240"])
        .env("XDG_STATE_HOME", state(runtime))
        .env(chora_scene::paths::RUNTIME_ENV, runtime)
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("RUST_LOG")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn no_socket_is_made_in_a_runtime_dir_that_others_can_write() {
    let dir = runtime("open", 0o777);
    let output = run(&dir);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "chora must refuse to start: {stderr}"
    );
    assert_eq!(entries(&dir), Vec::<String>::new(), "nothing is made");
    assert!(
        !stderr.contains("Created new socket"),
        "no Wayland socket was bound, not even one that ends with the process: {stderr}"
    );
    fs::remove_dir_all(&dir).unwrap();
    let _ = fs::remove_dir_all(state(&dir));
}

#[test]
fn the_refusal_is_printed_as_text_with_its_remedy() {
    let dir = runtime("remedy", 0o777);
    let output = run(&dir);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let last = stderr.lines().last().unwrap_or_default();
    let remedy = format!(
        "chmod 0700 {}, or set XDG_RUNTIME_DIR to a dir of yours",
        dir.display()
    );
    assert!(last.starts_with("chora: "), "{last}");
    assert!(last.contains(&remedy), "{last}");
    assert!(!stderr.contains("Custom {"), "no Debug print: {stderr}");
    fs::remove_dir_all(&dir).unwrap();
    let _ = fs::remove_dir_all(state(&dir));
}
