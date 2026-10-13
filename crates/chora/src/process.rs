//! Programs that the core starts: the startup command (D30), and the
//! programs of `process.spawn` and `plugins.restart` (#9, D42). They get
//! the environment of the core, with `WAYLAND_DISPLAY` and `CHORA_SOCKET`
//! of this compositor and without the variables of a host session. Both
//! methods need the grant `process.spawn`, since a program can do
//! anything that the user can.

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

use chora_scene::ipc::{Capability, Restart, RpcError, Spawn, Spawned};
use serde_json::Value;
use smithay::reexports::{
    calloop::timer::{TimeoutAction, Timer},
    rustix::{
        event::{PollFd, PollFlags, Timespec, poll},
        fd::OwnedFd,
        io::Errno,
        process::{Pid, PidfdFlags, Signal, pidfd_open, pidfd_send_signal},
    },
};

use crate::{
    ipc::{params_of, rejected},
    state::State,
};

/// Variables of the host session that would let a program reach the host
/// compositor instead of this one.
const HOST_VARIABLES: [&str; 6] = [
    "WAYLAND_SOCKET",
    "DISPLAY",
    "SWAYSOCK",
    "I3SOCK",
    "NIRI_SOCKET",
    "HYPRLAND_INSTANCE_SIGNATURE",
];

/// The most arguments of a program, its name included.
const MAX_ARGS: usize = 256;
/// The most bytes of all arguments.
const MAX_ARG_BYTES: usize = 64 << 10;
/// How long a plugin has to exit after SIGTERM, before SIGKILL.
const STOP_WAIT: Duration = Duration::from_secs(5);
/// How long after SIGKILL a restart gives up.
const KILL_WAIT: Duration = Duration::from_secs(1);
/// How often a restart looks whether the old process exited.
const POLL: Duration = Duration::from_millis(50);

/// Starts a program with the environment of this compositor. A thread
/// waits for it, so that it never stays a zombie.
pub fn spawn(
    argv: &[String],
    (cwd, stdin): (Option<&Path>, Stdio),
    wayland: &OsStr,
    socket: &Path,
) -> std::io::Result<Child> {
    let (program, rest) = argv
        .split_first()
        .ok_or_else(|| std::io::Error::other("no program"))?;
    let mut command = Command::new(program);
    command
        .args(rest)
        .stdin(stdin)
        .env("WAYLAND_DISPLAY", wayland)
        .env(chora_scene::ipc::SOCKET_ENV, socket);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    for variable in HOST_VARIABLES {
        command.env_remove(variable);
    }
    command.spawn()
}

/// Waits for a child in a thread.
pub fn reap(mut child: Child) {
    std::thread::spawn(move || child.wait());
}

/// Checks the size of an argv.
fn check_argv(argv: &[String]) -> Result<(), RpcError> {
    let bytes: usize = argv.iter().map(String::len).sum();
    if argv.is_empty() || argv[0].is_empty() {
        return Err(rejected("argv needs a program"));
    }
    if argv.len() > MAX_ARGS || bytes > MAX_ARG_BYTES {
        return Err(rejected(format!(
            "argv has at most {MAX_ARGS} arguments and {} KiB",
            MAX_ARG_BYTES >> 10
        )));
    }
    Ok(())
}

/// The argv and the working directory of a process, from `/proc`.
fn argv_of(pid: Pid) -> Result<(Vec<String>, Option<PathBuf>), String> {
    let proc = PathBuf::from(format!("/proc/{pid}"));
    let cmdline = std::fs::read(proc.join("cmdline")).map_err(|e| format!("pid {pid}: {e}"))?;
    // Each argument ends with a NUL; empty arguments stay.
    let argv: Vec<String> = cmdline
        .strip_suffix(&[0])
        .unwrap_or(&cmdline)
        .split(|b| *b == 0)
        .map(|a| String::from_utf8_lossy(a).into_owned())
        .collect();
    if argv.first().is_none_or(String::is_empty) {
        return Err(format!("pid {pid} has no command line"));
    }
    Ok((argv, std::fs::read_link(proc.join("cwd")).ok()))
}

/// Whether the process of a pidfd still runs. A pidfd becomes readable when
/// its process exits, also as a zombie that an init does not reap, and it
/// never points to another process that reuses the pid.
fn alive(pidfd: &OwnedFd) -> bool {
    let mut fds = [PollFd::new(pidfd, PollFlags::IN)];
    let now = Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    poll(&mut fds, Some(&now)).is_ok_and(|ready| ready == 0)
}

/// A restart that waits for the old process to exit.
struct Stopping {
    conn: u64,
    id: Value,
    name: String,
    pid: Pid,
    pidfd: OwnedFd,
    argv: Vec<String>,
    cwd: Option<PathBuf>,
    since: Instant,
    killed: bool,
}

impl State {
    /// `process.spawn` and `plugins.restart`. `None` means that the answer
    /// comes later.
    pub fn ipc_process(
        &mut self,
        conn: u64,
        id: Value,
        method: &str,
        params: Value,
    ) -> Option<Result<Value, RpcError>> {
        if let Err(err) = self.check_grant(conn, Capability::ProcessSpawn) {
            return Some(Err(err));
        }
        if self.lock.is_some() {
            return Some(Err(rejected("the session is locked")));
        }
        if method == "process.spawn" {
            return Some(params_of::<Spawn>(params).and_then(|s| self.start_program(s.argv, None)));
        }
        let restart = match params_of::<Restart>(params) {
            Ok(r) => r,
            Err(err) => return Some(Err(err)),
        };
        match self.stop_for_restart(conn, id, &restart.name) {
            Ok(()) => None,
            Err(err) => Some(Err(err)),
        }
    }

    /// Starts a program and answers with its pid.
    fn start_program(&self, argv: Vec<String>, cwd: Option<&Path>) -> Result<Value, RpcError> {
        check_argv(&argv)?;
        let socket = self
            .ipc
            .path()
            .ok_or_else(|| rejected("chora has no IPC socket"))?;
        let child = spawn(&argv, (cwd, Stdio::null()), &self.socket_name, socket)
            .map_err(|e| rejected(format!("{}: {e}", argv[0])))?;
        let pid = child.id();
        tracing::info!(pid, ?argv, "started a program");
        reap(child);
        Ok(serde_json::json!(Spawned { pid, argv }))
    }

    /// Sends SIGTERM to the process of a client name, and starts its argv
    /// again once it exited.
    fn stop_for_restart(&mut self, conn: u64, id: Value, name: &str) -> Result<(), RpcError> {
        if self.ipc.restarting.contains(name) {
            return Err(rejected(format!("a restart of {name:?} runs already")));
        }
        let pids = self.ipc.peer_pids(name);
        let pid = match pids[..] {
            [] => return Err(rejected(format!("no client {name:?} is connected"))),
            [pid] => pid.ok_or_else(|| {
                rejected(format!("the process of {name:?} is in a hidden namespace"))
            })?,
            _ => {
                return Err(rejected(format!("several processes use the name {name:?}")));
            }
        };
        let failed = |e: Errno| rejected(format!("pid {pid}: {e}"));
        let pidfd = pidfd_open(pid, PidfdFlags::empty()).map_err(failed)?;
        let (argv, cwd) = argv_of(pid).map_err(rejected)?;
        check_argv(&argv)?;
        pidfd_send_signal(&pidfd, Signal::TERM).map_err(failed)?;
        tracing::info!(name, ?argv, "restarts a plugin");
        self.ipc.restarting.insert(name.to_owned());
        let mut stopping = Stopping {
            conn,
            id,
            name: name.to_owned(),
            pid,
            pidfd,
            argv,
            cwd,
            since: Instant::now(),
            killed: false,
        };
        let inserted = self
            .loop_handle
            .insert_source(Timer::from_duration(POLL), move |_, (), state| {
                state.poll_restart(&mut stopping)
            });
        inserted.map(drop).map_err(|e| rejected(e.to_string()))
    }

    /// Looks whether the old process of a restart exited; starts the new one
    /// then, or kills the old one after a while.
    fn poll_restart(&mut self, stopping: &mut Stopping) -> TimeoutAction {
        let waited = stopping.since.elapsed();
        if alive(&stopping.pidfd) {
            if waited < STOP_WAIT {
                return TimeoutAction::ToDuration(POLL);
            }
            if !stopping.killed {
                let _ = pidfd_send_signal(&stopping.pidfd, Signal::KILL);
                stopping.killed = true;
            }
            if waited < STOP_WAIT + KILL_WAIT {
                return TimeoutAction::ToDuration(POLL);
            }
            let error = rejected(format!("pid {} did not exit", stopping.pid));
            self.ipc.restarting.remove(&stopping.name);
            self.answer_late(stopping.conn, &stopping.id, Err(error));
            return TimeoutAction::Drop;
        }
        self.ipc.restarting.remove(&stopping.name);
        let argv = std::mem::take(&mut stopping.argv);
        let result = self.start_program(argv, stopping.cwd.as_deref());
        self.answer_late(stopping.conn, &stopping.id, result);
        TimeoutAction::Drop
    }

    /// Sends a late answer.
    fn answer_late(&mut self, conn: u64, id: &Value, result: Result<Value, RpcError>) {
        let message = match result {
            Ok(value) => chora_scene::ipc::Message::result(id.clone(), value),
            Err(error) => chora_scene::ipc::Message::error(id.clone(), error),
        };
        self.ipc.respond(conn, &message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_argv_needs_a_program_and_has_limits() {
        assert!(check_argv(&[]).is_err());
        assert!(check_argv(&[String::new()]).is_err());
        assert!(check_argv(&vec!["x".into(); MAX_ARGS + 1]).is_err());
        assert!(check_argv(&["x".repeat(MAX_ARG_BYTES + 1)]).is_err());
        assert!(check_argv(&["foot".into(), "-e".into(), "htop".into()]).is_ok());
    }

    #[test]
    fn a_pidfd_sees_the_exit_also_of_a_zombie() {
        let mut child = Command::new("sleep").arg("5").spawn().unwrap();
        let pid = Pid::from_raw(i32::try_from(child.id()).unwrap()).unwrap();
        let pidfd = pidfd_open(pid, PidfdFlags::empty()).unwrap();
        assert!(alive(&pidfd));
        pidfd_send_signal(&pidfd, Signal::TERM).unwrap();
        // Unreaped, the child stays a zombie after its exit.
        let start = Instant::now();
        while alive(&pidfd) && start.elapsed() < Duration::from_secs(5) {
            std::thread::sleep(POLL);
        }
        assert!(!alive(&pidfd));
        child.wait().unwrap();
        assert!(!alive(&pidfd));
    }

    #[test]
    fn a_restart_keeps_empty_arguments() {
        let mut child = Command::new("sleep").args(["5", ""]).spawn().unwrap();
        let pid = Pid::from_raw(i32::try_from(child.id()).unwrap()).unwrap();
        let (argv, _) = argv_of(pid).unwrap();
        child.kill().unwrap();
        child.wait().unwrap();
        assert_eq!(argv, ["sleep", "5", ""]);
    }

    #[test]
    fn the_argv_of_this_process_reads_from_proc() {
        let pid = Pid::from_raw(i32::try_from(std::process::id()).unwrap()).unwrap();
        let (argv, cwd) = argv_of(pid).unwrap();
        assert!(!argv.is_empty());
        assert_eq!(cwd, std::env::current_dir().ok());
    }
}
