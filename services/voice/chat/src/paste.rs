//! Reads the text of a selection offer.

use std::{
    fs::File,
    io::Read,
    os::fd::{AsFd, OwnedFd},
    time::{Duration, Instant},
};

use rustix::event::{PollFd, PollFlags, Timespec, poll};

/// The most bytes of a paste.
const MAX_PASTE: usize = 64 * 1024;
/// How long a source may take.
const WAIT: Duration = Duration::from_secs(1);

/// The mime types that carry text, best first.
const TEXT_TYPES: [&str; 3] = ["text/plain;charset=utf-8", "text/plain", "UTF8_STRING"];

/// The mime type of an offer that carries text.
#[must_use]
pub fn text_type(offered: &[String]) -> Option<&'static str> {
    TEXT_TYPES
        .into_iter()
        .find(|wanted| offered.iter().any(|o| o == wanted))
}

/// Reads what the source writes into the pipe until it ends, the limit
/// is reached or the time is up. Bytes that are no UTF-8 are replaced.
#[must_use]
pub fn read_text(fd: OwnedFd) -> String {
    let deadline = Instant::now() + WAIT;
    let mut file = File::from(fd);
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 4096];
    while bytes.len() < MAX_PASTE {
        let left = deadline.saturating_duration_since(Instant::now());
        let Ok(timeout) = Timespec::try_from(left) else {
            break;
        };
        let ready = {
            let fd = file.as_fd();
            let mut fds = [PollFd::new(&fd, PollFlags::IN)];
            poll(&mut fds, Some(&timeout)).unwrap_or(0)
        };
        if ready == 0 {
            break;
        }
        match file.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => bytes.extend_from_slice(&chunk[..n]),
        }
    }
    bytes.truncate(MAX_PASTE);
    String::from_utf8_lossy(&bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn a_text_type_is_picked_by_rank() {
        let offered = ["image/png".to_owned(), "text/plain".to_owned()];
        assert_eq!(text_type(&offered), Some("text/plain"));
        assert_eq!(text_type(&["image/png".to_owned()]), None);
    }

    #[test]
    fn a_pipe_is_read_to_its_end() {
        let (read, write) = rustix::pipe::pipe().expect("a pipe");
        let mut write = File::from(write);
        write.write_all("grüß dich".as_bytes()).expect("write");
        drop(write);
        assert_eq!(read_text(read), "grüß dich");
    }
}
