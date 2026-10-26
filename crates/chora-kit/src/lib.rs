//! Building blocks for chora plugins that are not part of the API of the
//! core: the IPC and event connection to the scene, effects that a window
//! manager computes itself, such as wobbly windows and the magic lamp,
//! the freedesktop files that a shell reads, such as icon themes and
//! desktop entries, and the start of apps.
pub mod conn;
pub mod desktop_entry;
pub mod icons;
pub mod lamp;
pub mod wobbly;
pub mod xdg;

/// Starts a shell command as an app with `choractl run` (D42), so that
/// the app gets a unit of its own and an activation token. `&` lets the
/// shell exit at once, so nothing waits for `run` and no zombie stays.
pub fn run(command: &str) {
    let started = std::process::Command::new("sh")
        .args(["-c", r#"choractl run sh -c "$0" >/dev/null &"#, command])
        .status();
    if let Err(err) = started {
        eprintln!("cannot run {command:?}: {err}");
    }
}
