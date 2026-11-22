//! The start of the window manager of the cube. Its process outlives the
//! construct (WM7b): when the construct dies, the windows of the faces
//! stay with it, flat on the screens, and a construct that starts again
//! opens the cube around it. Its error output goes to a file, which the
//! construct follows: each line goes on to the log of the construct, and
//! the last one is the reason if the manager fails.

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Child, Stdio},
    sync::{
        Arc, Condvar, Mutex, MutexGuard, PoisonError,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use chora_kit::geometry::Cube;
use serde_json::Value;

use super::{CUBE, ClientError, Construct, Health, NodeKind, PANES, PropsUpdate, create};

/// The last line that a window manager wrote to its error output, and how
/// far the reader of the file got.
pub(super) type Tail = Arc<Followed>;

/// How often the construct reads the error output of the manager.
const FOLLOW: Duration = Duration::from_millis(100);
/// The longest `Followed::settle` waits for the reader: a reader that
/// died never answers.
const SETTLE: Duration = Duration::from_millis(500);

/// What the reader of the error output of a manager keeps, and the event
/// that it reached the end of the file.
#[derive(Default)]
pub(super) struct Followed {
    state: Mutex<Reading>,
    moved: Condvar,
}

/// The state of the reader of the error output.
#[derive(Default)]
struct Reading {
    /// The last line that was not empty.
    last: String,
    /// How many times somebody asked for a look.
    asks: u64,
    /// The newest ask that a finished pass began after: that pass read the
    /// file to its end with the last write of the manager in it.
    covered: u64,
    /// Somebody wants the reader to look at the file now.
    asked: bool,
    /// The reader stopped.
    closed: bool,
}

impl Followed {
    fn state(&self) -> MutexGuard<'_, Reading> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The last line that the manager wrote.
    pub(super) fn last(&self) -> String {
        self.state().last.clone()
    }

    fn keep(&self, line: String) {
        self.state().last = line;
    }

    /// The reader starts a pass over the file: the number of asks so far.
    fn begin_pass(&self) -> u64 {
        self.state().asks
    }

    /// The reader came to the end of the file in a pass that began after
    /// `began` asks: tells it to those who wait, and rests until the next
    /// look, which is at most `FOLLOW` away or when somebody asks for it.
    fn rest_at_the_end(&self, began: u64) {
        let mut state = self.state();
        state.covered = state.covered.max(began);
        self.moved.notify_all();
        let (mut state, _) = (self.moved)
            .wait_timeout_while(state, FOLLOW, |s| !s.asked)
            .unwrap_or_else(PoisonError::into_inner);
        state.asked = false;
    }

    fn close(&self) {
        self.state().closed = true;
        self.moved.notify_all();
    }

    /// Waits until the reader has read the file to its end in a pass that
    /// began after this ask, so that the last line of a manager that has
    /// exited is in `last`; a pass that began before it may have read the
    /// end of the file before the last write. The reader wakes at once; no
    /// fixed time stands in for its work.
    pub(super) fn settle(&self) {
        let mut state = self.state();
        state.asks += 1;
        let ask = state.asks;
        state.asked = true;
        self.moved.notify_all();
        let _ = (self.moved)
            .wait_timeout_while(state, SETTLE, |s| s.covered < ask && !s.closed)
            .unwrap_or_else(PoisonError::into_inner);
    }
}

/// A new file for the error output of a manager.
fn log_file() -> PathBuf {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    let dir = chora_scene::paths::runtime_home().unwrap_or_else(std::env::temp_dir);
    let n = COUNT.fetch_add(1, Ordering::Relaxed);
    dir.join(format!(
        "chora-construct-cube-{}-{n}.log",
        std::process::id()
    ))
}

/// Starts `command` as a process of its own, without a signal at the end
/// of the construct, its error output in a file that a thread follows
/// until the tail goes with the host.
pub(super) fn launch(command: &str) -> std::io::Result<(Child, Tail)> {
    let path = log_file();
    let file = File::create(&path)?;
    let child = std::process::Command::new("sh")
        .args(["-c", command])
        .stdin(Stdio::null())
        .stderr(file)
        .spawn()?;
    let tail = Tail::default();
    let kept = Arc::downgrade(&tail);
    let reader = File::open(&path)?;
    std::thread::spawn(move || {
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        while let Some(last) = kept.upgrade() {
            let began = last.begin_pass();
            match reader.read_line(&mut line) {
                Ok(0) => last.rest_at_the_end(began),
                Ok(_) if line.ends_with('\n') => {
                    let text = line.trim_end().to_owned();
                    crate::log::log_info!("chora-construct: cube manager: {text}");
                    if !text.trim().is_empty() {
                        last.keep(text);
                    }
                    line.clear();
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
        if let Some(last) = kept.upgrade() {
            last.close();
        }
        let _ = std::fs::remove_file(&path);
    });
    Ok((child, tail))
}

impl Construct {
    /// At the start: the virtual outputs that an earlier construct left
    /// show the cube again when its window manager still runs on them, as
    /// after a crash; else they go. A window of that manager may have the
    /// focus already, which no `focus.changed` tells a new construct.
    pub(crate) fn reopen_left_cube(&mut self) -> Result<(), ClientError> {
        let outputs = self.rt.all_outputs()?;
        let left: Vec<String> = (outputs.into_iter())
            .filter(|o| o.is_virtual == Some(true) && o.owner.as_deref() == Some(crate::NAME))
            .map(|o| o.name)
            .collect();
        let reopens = !self.overlay && (1..=super::FACES).contains(&left.len());
        if !reopens || !self.manager_runs()? {
            self.remove_outputs(&left);
            return Ok(());
        }
        let size = [self.lens.w, self.lens.h];
        let cube = Cube { size, caps: false };
        let manager = self.settings.value.cube_manager.clone();
        let flags: Vec<String> = left.iter().map(|o| format!("--output {o}")).collect();
        let command = format!("exec {manager} {}", flags.join(" "));
        let mut host = self.host(left.clone(), None, (&manager, &command), cube);
        host.health.fill(Health::Up);
        let mut commands = vec![create(CUBE, NodeKind::Group, PANES, PropsUpdate::default())];
        commands.extend(self.face_nodes(&left, &cube));
        commands.extend(self.edge_nodes());
        commands.extend(host.faces([self.pose.x, self.pose.z], self.lens.ground));
        if let Some(err) = self.rt.conn.try_apply(commands)? {
            crate::log::log_warn!(
                "chora-construct: the cube of {} again: {err}",
                self.face_manager()
            );
            self.remove_outputs(&left);
            return Ok(());
        }
        host.since = Instant::now();
        self.cube = Some(host);
        self.core_focus = self.focused_surface()?;
        self.open_modes()?;
        self.frame()
    }

    /// Whether a client of the name of the window manager is connected.
    pub(super) fn manager_runs(&mut self) -> Result<bool, ClientError> {
        let clients: Value = self.rt.conn.call("clients.list", Value::Null)?;
        let list = clients.as_array().cloned().unwrap_or_default();
        let client = self.face_manager();
        Ok(list
            .iter()
            .any(|c| c["name"].as_str() == Some(client.as_str())))
    }

    /// The client name of the window manager of the cube: of the open
    /// cube, else of the setting `cube_manager`.
    pub(crate) fn face_manager(&self) -> String {
        let program = crate::names::program_of(&self.settings.value.cube_manager);
        let from_setting = || crate::names::client_of(program).to_owned();
        (self.cube.as_ref()).map_or_else(from_setting, |host| host.client().to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_error_output_of_the_manager_goes_to_the_tail_and_the_manager_outlives_it() {
        let (mut child, tail) = launch("echo first >&2; echo 'the last line' >&2").expect("sh");
        let _ = child.wait();
        let end = Instant::now() + Duration::from_secs(2);
        while tail.last() != "the last line" {
            assert!(Instant::now() < end, "the tail is {:?}", tail.last());
            tail.settle();
        }
    }

    /// A manager that exited is read at once: the construct waits for the
    /// reader of its error output to reach the end of the file, not for a
    /// fixed time that the reader may or may not need (W8).
    #[test]
    fn the_last_line_of_a_manager_that_exited_is_there_without_a_wait() {
        let mut host = super::super::tests::host_of(1);
        let (child, tail) = launch("echo 'the last line' >&2; exit 3").expect("sh");
        host.child = Some(child);
        host.tail = tail;
        host.health = vec![Health::Starting];
        let _ = host.child.as_mut().expect("the child").wait();
        let asked = Instant::now();
        let reason = host.check_start(chora_kit::words::Language::En, Instant::now());
        let waited = asked.elapsed();
        assert!(
            reason.is_some_and(|r| r.contains("the last line")),
            "the reason names the last line"
        );
        assert!(
            waited < Duration::from_millis(140),
            "it waited {waited:?} for a reader that is at the end in a moment"
        );
    }

    /// A reader that came to the end of the file before the ask may not
    /// have seen the last write of the manager: its pass does not settle
    /// the ask. Only a pass that began after it does (rd-3).
    #[test]
    fn a_pass_that_ended_after_the_ask_but_began_before_it_does_not_settle() {
        let followed = Arc::new(Followed::default());
        let before = followed.begin_pass();
        let waiting = {
            let followed = Arc::clone(&followed);
            std::thread::spawn(move || followed.settle())
        };
        while !followed.state().asked {
            std::thread::yield_now();
        }
        followed.rest_at_the_end(before);
        std::thread::sleep(Duration::from_millis(50));
        assert!(
            !waiting.is_finished(),
            "the pass of before the ask settled it"
        );
        let after = followed.begin_pass();
        followed.rest_at_the_end(after);
        waiting.join().expect("settle");
    }
}
