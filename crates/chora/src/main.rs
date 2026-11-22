//! The chora Wayland compositor.

use std::process::ExitCode;

#[allow(clippy::print_stderr, reason = "the last words of a failed start")]
#[allow(unsafe_code, reason = "one mallopt call before any thread starts")]
fn main() -> ExitCode {
    // SAFETY: sets a glibc option before any thread starts; a fixed mmap threshold returns freed llvmpipe textures to the system (sl-1).
    unsafe { libc::mallopt(libc::M_MMAP_THRESHOLD, 64 << 10) };
    let args: Vec<String> = std::env::args().skip(1).collect();
    match chora::run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("chora: {error}");
            ExitCode::FAILURE
        }
    }
}
