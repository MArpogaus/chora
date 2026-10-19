//! Sets `CHORA_VERSION` to the crate version and, in a git checkout, the
//! commit, so that the core and its clients tell their builds apart.

use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    out.status.success().then(|| text.trim().to_owned())
}

fn main() {
    // A new commit or another branch changes the version.
    let branch = git(&["symbolic-ref", "-q", "HEAD"]);
    for name in ["HEAD"].into_iter().chain(branch.as_deref()) {
        if let Some(path) = git(&["rev-parse", "--git-path", name]) {
            println!("cargo:rerun-if-changed={path}");
        }
    }
    let package = env!("CARGO_PKG_VERSION");
    let version = match git(&["rev-parse", "--short=10", "HEAD"]) {
        Some(commit) if !commit.is_empty() => format!("{package}+{commit}"),
        _ => package.to_owned(),
    };
    println!("cargo:rustc-env=CHORA_VERSION={version}");
}
