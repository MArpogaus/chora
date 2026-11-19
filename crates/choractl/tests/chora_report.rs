//! `dist/chora-report` on a fixture: the bundle keeps the lines of the
//! logs and of the settings, and none of the user name, the home path,
//! the host name or a token, also in block-style YAML, under header
//! names and in settings files that are symlinks; only the person reads
//! the bundle.

#![allow(clippy::unwrap_used, reason = "a test: a failed step fails it")]

#[path = "../src/temp_dir.rs"]
mod temp_dir;

use std::{
    fmt::Write as _,
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    process::Command,
};

const KEPT: [&str; 10] = [
    "chora runs wayland=wayland-1",
    "owner USER:USER on HOST,HOST",
    "llm_url: http://localhost:8080",
    "cmd: coder --fast",
    "tiling_gap: 8",
    "~/.config/chora/tiling.yaml: bad key",
    "refused client=\"agent\" code=4 user USER on HOST",
    "the last run",
    "asr_url: http://localhost:8000",
    "kernel: ",
];

const GONE: [&str; 18] = [
    "some+one",
    "abc123secretzz",
    "AIzaSy",
    "hunter4",
    "hunter5",
    "hunter6",
    "hunter7",
    "hunter8",
    "dotfiles",
    "abc.def-123",
    "sk-proj",
    "ghp_",
    "QUJDREVG",
    "hunter2",
    "hunter3",
    "t0k3n",
    "secret-value",
    "not a settings file",
];

fn output(program: &str, args: &[&str]) -> String {
    let out = Command::new(program).args(args).output().unwrap();
    String::from_utf8(out.stdout).unwrap().trim().to_owned()
}

fn write(path: PathBuf, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn read_all(dir: &Path) -> String {
    let mut text = String::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            text += &read_all(&path);
        } else {
            let _ = writeln!(text, "== {}", path.display());
            text += &fs::read_to_string(&path).unwrap();
        }
    }
    text
}

/// A home with two log files, the settings of the voice agent and of
/// the agents, and a settings file that is a symlink into dotfiles.
fn fixture(home: &Path, user: &str, host: &str) {
    let home_text = home.display();
    let log = format!(
        "2026-11-15T10:00:00Z  INFO chora: chora runs wayland=wayland-1\n\
         2026-11-15T10:00:01Z  WARN chora::ipc: {home_text}/.config/chora/tiling.yaml: bad key\n\
         2026-11-15T10:00:02Z  INFO chora::ipc: refused client=\"agent\" code=4 user {user} on {host}\n\
         2026-11-15T10:00:02Z  INFO chora::ipc: owner {user}:{user} on {host},{host}\n\
         2026-11-15T10:00:03Z DEBUG chora_voice: Authorization: Bearer abc.def-123\n\
         2026-11-15T10:00:04Z DEBUG chora_voice: key sk-proj-a1b2c3d4e5f6g7h8 and ghp_A1b2C3d4E5f6G7h8I9j0\n\
         2026-11-15T10:00:05Z DEBUG chora_voice: blob QUJDREVGR0hJSktMTU5PUFFSU1RVVldYWVphYmNkZWZnaGlqa2xtbm9w\n"
    );
    write(home.join(".local/state/chora/logs/chora.log"), &log);
    write(
        home.join(".local/state/chora/logs/chora.log.1"),
        "the last run\n",
    );
    write(
        home.join(".config/chora/voice.yaml"),
        "asr_url: http://localhost:8000\nllm_api_key: hunter2\ntoken: \"t0k3n\"\n\
         llm_headers: {X-Key: secret-value}\npassword=hunter3\n\
         llm_headers:\n  X-Api-Key: abc123secretzz\n  x-goog-api-key: AIzaSyZZZZZZZZZZZZZZZZZZZZ\n\
         llm_url: http://localhost:8080\n",
    );
    write(
        home.join(".config/chora/agents.yaml"),
        "Api-Key: hunter4\nAUTHORIZATION: hunter5\nagents:\n  - name: coder\n    env:\n\
         \x20     X-Api-Key: hunter6\n      token: |\n        multi-line-hunter7\n\
         \x20   cmd: coder --fast\n",
    );
    write(
        home.join("dotfiles/tiling.yaml"),
        "tiling_gap: 8\ntiling_token: hunter8\n",
    );
    symlink(
        home.join("dotfiles/tiling.yaml"),
        home.join(".config/chora/tiling.yaml"),
    )
    .unwrap();
    write(
        home.join(".config/chora/notes.txt"),
        "not a settings file\n",
    );
}

/// Runs the script with `home` and gives the text of the unpacked bundle.
fn report(work: &Path, home: &Path) -> String {
    let bundle = work.join("report.tar.gz");
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dist/chora-report");
    let status = Command::new("sh")
        .arg("-c")
        .arg("umask 022 && exec sh \"$0\" -o \"$1\"")
        .arg(&script)
        .arg(&bundle)
        .env("HOME", home)
        .env_remove("XDG_STATE_HOME")
        .env_remove("XDG_CONFIG_HOME")
        .env("PATH", "/usr/bin:/bin")
        .status()
        .unwrap();
    assert!(status.success());
    assert_eq!(
        fs::metadata(&bundle).unwrap().permissions().mode() & 0o077,
        0,
        "only the person reads the bundle"
    );
    let unpacked = work.join("unpacked");
    fs::create_dir_all(&unpacked).unwrap();
    let tar = Command::new("tar")
        .arg("-xzf")
        .arg(&bundle)
        .arg("-C")
        .arg(&unpacked)
        .status()
        .unwrap();
    assert!(tar.success());
    read_all(&unpacked.join("chora-report"))
}

/// The words of the files, without the `== PATH` heads.
fn words(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .filter(|l| !l.starts_with("== "))
        .flat_map(|l| l.split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_'))
}

#[test]
fn the_bundle_keeps_the_lines_and_hides_the_person_and_the_tokens() {
    let work = temp_dir::TempDir::unmade("chora-report");
    let home = work.join("home/some+one(1)");
    let (user, host) = (output("id", &["-un"]), output("uname", &["-n"]));
    fixture(&home, &user, &host);
    let text = report(&work, &home);
    for kept in KEPT {
        assert!(text.contains(kept), "{kept:?} is missing:\n{text}");
    }
    let home_text = home.display().to_string();
    for gone in GONE.iter().chain([&home_text.as_str()]) {
        assert!(!text.contains(gone), "{gone:?} is in the bundle:\n{text}");
    }
    let named = words(&text).find(|w| *w == user || *w == host);
    assert_eq!(
        named, None,
        "the user or host name is in the bundle:\n{text}"
    );
    let _ = fs::remove_dir_all(&work);
}
