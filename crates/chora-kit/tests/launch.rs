//! The `FakeCore` script of `kit::launch` (slice KIT6): a look that
//! starts programs through a connection to the fake core and follows them
//! until their window maps. The programs are real processes of `sh`; the
//! clock of the look never advances, so a change that shows is no timeout.
#![allow(clippy::unwrap_used, reason = "a failure is a failed test")]

use std::{
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use chora_kit::{
    conn::Conn,
    launch::{Candidate, Change, Launches, Starter, Why},
    testing::FakeCore,
};

/// The processes that the starter of the look started.
#[derive(Default)]
struct Children(Mutex<Vec<Child>>);

impl Drop for Children {
    fn drop(&mut self) {
        for child in self.0.lock().unwrap().iter_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// A look that starts its programs as `choractl run` does: the command
/// runs under `sh` with the token of the look, and the answer names the
/// token and the pid.
struct Look {
    conn: Conn,
    launches: Launches,
    _children: Arc<Children>,
    began: Instant,
}

impl Look {
    fn new(core: &FakeCore) -> Self {
        let children = Arc::new(Children::default());
        let held = children.clone();
        let start: Starter = Arc::new(move |command, token| {
            let child = Command::new("sh")
                .args(["-c", command])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .ok()?;
            let pid = child.id();
            held.0.lock().unwrap().push(child);
            Some((token.unwrap_or("none").to_owned(), Some(pid)))
        });
        Self {
            conn: core.connect("look"),
            launches: Launches::with_starter(start),
            _children: children,
            began: Instant::now(),
        }
    }

    fn token(&mut self) -> String {
        self.conn.activation_token().unwrap()
    }

    fn run(&mut self, command: &str) -> String {
        let token = self.token();
        let candidate = (command.to_owned(), Some(token.clone()));
        let change = self.launches.run(command, candidate, self.began);
        assert_eq!(change, None, "{command} did not start");
        token
    }

    fn chain(&mut self, commands: [&str; 2]) -> String {
        let token = self.token();
        let chain = commands
            .iter()
            .zip(["writer", "gedit"])
            .map(|(command, name)| Candidate {
                name: name.into(),
                command: (*command).into(),
            })
            .collect();
        let changes =
            self.launches
                .run_chain("Notepad", (chain, None), Some(token.clone()), self.began);
        assert_eq!(changes, [], "the first editor started");
        token
    }

    /// The changes that the look hears at the start of its clock, once
    /// there are some.
    fn heard(&mut self) -> Vec<Change> {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            let mut changes = self.launches.collect();
            changes.extend(self.launches.check(self.began));
            if !changes.is_empty() {
                return changes;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("no change within 5 s at the start of the clock");
    }

    fn mapped(&mut self, token: &str) -> Option<u32> {
        self.launches.window(Some(token))
    }
}

#[test]
fn a_program_that_exits_before_its_window_gives_exited_at_once() {
    let core = FakeCore::start();
    let mut look = Look::new(&core);
    look.run("exit 1");
    let changes = look.heard();
    assert!(
        matches!(
            changes.as_slice(),
            [Change::Failed {
                why: Why::Exited,
                ..
            }]
        ),
        "{changes:?}"
    );
    assert_eq!(look.launches.check(look.began), []);
}

#[test]
fn an_editor_that_exits_before_its_window_gives_way_to_the_next_at_once() {
    let core = FakeCore::start();
    let mut look = Look::new(&core);
    let token = look.chain(["exit 1", "sleep 30"]);
    let changes = look.heard();
    assert!(
        matches!(changes.as_slice(), [Change::Done { note: None }]),
        "{changes:?}"
    );
    assert_eq!(look.mapped(&token), None);
    let told = look.launches.collect();
    assert_eq!(
        told.first().map(Change::text),
        Some((
            "Notepad opened with gedit".into(),
            "writer ended without a window.".into()
        )),
        "{told:?}"
    );
}

#[test]
fn a_launch_that_maps_a_window_ends_its_wait_and_closes_its_message() {
    let core = FakeCore::start();
    let mut look = Look::new(&core);
    let token = look.run("sleep 30");
    let late = look.began + Duration::from_secs(60);
    assert_eq!(look.mapped(&token), None);
    assert_eq!(look.launches.check(late), []);
    assert_eq!(look.launches.collect(), []);
}

#[test]
fn a_program_that_exits_after_its_window_is_no_failure() {
    let core = FakeCore::start();
    let mut look = Look::new(&core);
    let token = look.run("sleep 0.2");
    assert_eq!(look.mapped(&token), None);
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(look.launches.collect(), []);
    assert_eq!(look.launches.check(look.began), []);
}
