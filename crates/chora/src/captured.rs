//! The output of programs that `process.spawn` starts with `capture`:
//! the core keeps stdout, stderr and the exit status until the client takes
//! them with `process.wait`. Threads read the pipes, so that a program that
//! writes much never blocks the core. A thread closes its pipe shortly after
//! the end of the program, also when a child in the background holds it
//! open, so that threads and file descriptors stay bounded.

use std::{
    collections::HashMap,
    io::Read,
    os::{fd::AsFd, unix::process::ExitStatusExt},
    process::Child,
    sync::{Arc, Mutex, PoisonError},
    thread,
    time::{Duration, Instant},
};

use chora_scene::ipc::ProcessOutput;
use smithay::reexports::rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    io::Errno,
};

/// The most bytes that the core keeps of each stream; the rest goes.
const MAX_OUTPUT: usize = 64 << 10;
/// The most programs of one client whose output waits for it.
const MAX_CAPTURED: usize = 32;
/// How long after its end a program may still write through a pipe that a
/// child in the background holds open.
const GRACE: Duration = Duration::from_millis(100);
/// How often a reader looks at the end of the program while the pipe is
/// quiet.
const TICK: Timespec = Timespec {
    tv_sec: 0,
    tv_nsec: 20_000_000,
};

/// What a program wrote, and how it ended.
#[derive(Default)]
struct Output {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    truncated: bool,
    /// The exit code, or 128 and the signal that ended the program.
    status: Option<i32>,
    /// The streams that a thread still reads.
    open: u8,
    /// When the program ended.
    ended: Option<Instant>,
}

impl Output {
    /// Whether the program ended and its pipes are closed.
    fn finished(&self) -> bool {
        self.status.is_some() && self.open == 0
    }

    /// Whether the grace time after the end of the program is over.
    fn past_grace(&self) -> bool {
        self.ended.is_some_and(|t| t.elapsed() >= GRACE)
    }
}

/// A program and the client that started it.
struct Captured {
    owner: String,
    output: Arc<Mutex<Output>>,
}

/// The programs whose output waits for their clients.
#[derive(Default)]
pub struct Processes {
    captured: HashMap<u32, Captured>,
}

impl Processes {
    /// Keeps the output of a child that has piped stdout and stderr, and
    /// waits for it in a thread. Returns its pid.
    pub fn capture(&mut self, owner: &str, mut child: Child) -> Result<u32, String> {
        if let Err(err) = self.room(owner) {
            // A child that cannot be kept still needs a reaper, and no pipe
            // that nobody reads.
            drop((child.stdout.take(), child.stderr.take()));
            crate::process::reap(child);
            return Err(err);
        }
        let output = Arc::new(Mutex::new(Output::default()));
        if let Some(stdout) = child.stdout.take() {
            start_reader(stdout, &output, true);
        }
        if let Some(stderr) = child.stderr.take() {
            start_reader(stderr, &output, false);
        }
        let pid = child.id();
        let waited = Arc::clone(&output);
        thread::spawn(move || {
            let status = child.wait().map_or(-1, |s| {
                s.code()
                    .unwrap_or_else(|| 128 + s.signal().unwrap_or_default())
            });
            let mut output = lock(&waited);
            output.status = Some(status);
            output.ended = Some(Instant::now());
        });
        self.captured.insert(
            pid,
            Captured {
                owner: owner.to_owned(),
                output,
            },
        );
        Ok(pid)
    }

    /// Whether a client may start one more program with capture.
    pub fn room(&self, owner: &str) -> Result<(), String> {
        let kept = self.captured.values().filter(|c| c.owner == owner).count();
        if kept >= MAX_CAPTURED {
            return Err(format!(
                "{MAX_CAPTURED} programs of this client wait for process.wait already"
            ));
        }
        Ok(())
    }

    /// The output of a program of a client so far, and whether it ended.
    /// An ended program goes with its output.
    pub fn take(&mut self, owner: &str, pid: u32) -> Result<ProcessOutput, String> {
        let captured = self
            .captured
            .get(&pid)
            .filter(|c| c.owner == owner)
            .ok_or_else(|| format!("no program {pid} with output of this client"))?;
        let output = lock(&captured.output);
        let finished = output.finished();
        let answer = ProcessOutput {
            pid,
            running: !finished,
            status: output.status.filter(|_| finished),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            truncated: output.truncated,
        };
        drop(output);
        if !answer.running {
            self.captured.remove(&pid);
        }
        Ok(answer)
    }

    /// Whether a program of a client ended, or is unknown.
    pub fn done(&self, pid: u32) -> bool {
        self.captured
            .get(&pid)
            .is_none_or(|c| lock(&c.output).finished())
    }

    /// Drops the output of the programs of a client that went.
    pub fn drop_client(&mut self, owner: &str) {
        self.captured.retain(|_, c| c.owner != owner);
    }
}

fn lock(output: &Mutex<Output>) -> std::sync::MutexGuard<'_, Output> {
    output.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Starts a thread that reads a pipe of a program.
fn start_reader<S>(stream: S, output: &Arc<Mutex<Output>>, stdout: bool)
where
    S: Read + AsFd + Send + 'static,
{
    lock(output).open += 1;
    let output = Arc::clone(output);
    thread::spawn(move || read(stream, &output, stdout));
}

/// Reads a stream until its end, or until the grace time after the end of
/// the program, and keeps up to `MAX_OUTPUT` bytes.
fn read(mut stream: impl Read + AsFd, output: &Mutex<Output>, stdout: bool) {
    let mut chunk = [0; 4096];
    while !lock(output).past_grace() {
        let mut fds = [PollFd::new(&stream, PollFlags::IN)];
        // A poll that a signal cut short looks again.
        if poll(&mut fds, Some(&TICK)).is_err_and(|e| e != Errno::INTR) {
            break;
        }
        if fds[0].revents().is_empty() {
            continue;
        }
        let n = match stream.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        let mut output = lock(output);
        let kept = if stdout {
            &mut output.stdout
        } else {
            &mut output.stderr
        };
        let room = MAX_OUTPUT.saturating_sub(kept.len());
        kept.extend_from_slice(&chunk[..n.min(room)]);
        if n > room {
            output.truncated = true;
        }
    }
    lock(output).open -= 1;
}

#[cfg(test)]
mod tests {
    use std::{
        process::{Command, Stdio},
        time::{Duration, Instant},
    };

    use super::*;

    fn run(processes: &mut Processes, script: &str) -> u32 {
        let child = Command::new("sh")
            .args(["-c", script])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        processes.capture("agent", child).unwrap()
    }

    fn wait(processes: &mut Processes, pid: u32) -> ProcessOutput {
        let start = Instant::now();
        while !processes.done(pid) && start.elapsed() < Duration::from_secs(5) {
            thread::sleep(Duration::from_millis(20));
        }
        processes.take("agent", pid).unwrap()
    }

    #[test]
    fn the_output_and_the_status_come_back_once() {
        let mut processes = Processes::default();
        let pid = run(&mut processes, "echo out; echo err >&2; exit 3");
        assert!(processes.take("other", pid).is_err());
        let output = wait(&mut processes, pid);
        assert!(!output.running);
        assert_eq!(output.status, Some(3));
        assert_eq!(
            (output.stdout.as_str(), output.stderr.as_str()),
            ("out\n", "err\n")
        );
        assert!(
            processes.take("agent", pid).is_err(),
            "it goes after the end"
        );
    }

    #[test]
    fn a_child_in_the_background_does_not_hold_the_end() {
        let mut processes = Processes::default();
        let pid = run(&mut processes, "sleep 5 & echo ok");
        let start = Instant::now();
        let output = wait(&mut processes, pid);
        assert!(!output.running);
        assert_eq!((output.status, output.stdout.as_str()), (Some(0), "ok\n"));
        assert!(start.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn much_output_is_cut() {
        let mut processes = Processes::default();
        let pid = run(&mut processes, "head -c 200000 /dev/zero");
        let output = wait(&mut processes, pid);
        assert_eq!(output.stdout.len(), MAX_OUTPUT);
        assert!(output.truncated);
    }
}
