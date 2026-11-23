//! The chora Wayland compositor.

use std::io::Write;
use std::process::ExitCode;

/// Runs the core; the last words of a failed start go to stderr, and a failed
/// write has nowhere to go.
fn main() -> ExitCode {
    chora::heap::tune();
    let args: Vec<String> = std::env::args().skip(1).collect();
    match chora::run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(std::io::stderr(), "chora: {error}");
            ExitCode::FAILURE
        }
    }
}
