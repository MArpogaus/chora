//! The chora Wayland compositor.

use std::process::ExitCode;

#[allow(clippy::print_stderr, reason = "the last words of a failed start")]
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match chora::run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("chora: {error}");
            ExitCode::FAILURE
        }
    }
}
