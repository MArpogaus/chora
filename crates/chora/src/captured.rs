//! The output of programs that `process.spawn` starts with `capture` (#22):
//! the core keeps stdout, stderr and the exit status until the client takes
//! them with `process.wait`. Threads read the pipes, so that a program that
//! writes much never blocks the core.

use std::{
    collections::HashMap,
    io::Read,
    os::unix::process::ExitStatusExt,
    process::Child,
    sync::{Arc, Mutex, PoisonError},
    thread,
};

use chora_scene::ipc::ProcessOutput;

/// The most bytes that the core keeps of each stream; the rest goes.
const MAX_OUTPUT: usize = 64 << 10;
/// The most programs whose output waits for a client.
const MAX_CAPTURED: usize = 32;

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
}

impl Output {
    /// Whether the program ended and its output is all there.
    fn finished(&self) -> bool {
        self.status.is_some() && self.open == 0
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
        if self.captured.len() >= MAX_CAPTURED {
            // A child that cannot be kept still needs a reaper.
            crate::process::reap(child);
            return Err(format!(
                "{MAX_CAPTURED} programs wait for process.wait already"
            ));
        }
        let output = Arc::new(Mutex::new(Output::default()));
        let streams: [Option<Box<dyn Read + Send>>; 2] = [
            child
                .stdout
                .take()
                .map(|s| Box::new(s) as Box<dyn Read + Send>),
            child
                .stderr
                .take()
                .map(|s| Box::new(s) as Box<dyn Read + Send>),
        ];
        for (index, stream) in streams.into_iter().enumerate() {
            let Some(stream) = stream else { continue };
            lock(&output).open += 1;
            let output = Arc::clone(&output);
            thread::spawn(move || read(stream, &output, index == 0));
        }
        let pid = child.id();
        let waited = Arc::clone(&output);
        thread::spawn(move || {
            let status = child.wait().map_or(-1, |s| {
                s.code()
                    .unwrap_or_else(|| 128 + s.signal().unwrap_or_default())
            });
            lock(&waited).status = Some(status);
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

    /// The output of a program of a client so far, and whether it ended.
    /// An ended program goes with its output.
    pub fn take(&mut self, owner: &str, pid: u32) -> Result<ProcessOutput, String> {
        let captured = self
            .captured
            .get(&pid)
            .filter(|c| c.owner == owner)
            .ok_or_else(|| format!("no program {pid} with output of this client"))?;
        let output = lock(&captured.output);
        let answer = ProcessOutput {
            pid,
            running: !output.finished(),
            status: output.status.filter(|_| output.finished()),
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

/// Reads a stream until its end, and keeps up to `MAX_OUTPUT` bytes.
fn read(mut stream: Box<dyn Read + Send>, output: &Mutex<Output>, stdout: bool) {
    let mut chunk = [0; 4096];
    loop {
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
    fn much_output_is_cut() {
        let mut processes = Processes::default();
        let pid = run(&mut processes, "head -c 200000 /dev/zero");
        let output = wait(&mut processes, pid);
        assert_eq!(output.stdout.len(), MAX_OUTPUT);
        assert!(output.truncated);
    }
}
