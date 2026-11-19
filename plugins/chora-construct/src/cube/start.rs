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
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use chora_kit::geometry::Cube;
use serde_json::Value;

use super::{CUBE, ClientError, Construct, Health, MANAGER, NodeKind, PANES, PropsUpdate, create};

/// The last line that a window manager wrote to its error output.
pub(super) type Tail = Arc<Mutex<String>>;

/// How often the construct reads the error output of the manager.
const FOLLOW: Duration = Duration::from_millis(100);

/// A new file for the error output of a manager.
fn log_file() -> PathBuf {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::var_os("XDG_RUNTIME_DIR").map_or_else(std::env::temp_dir, PathBuf::from);
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
            match reader.read_line(&mut line) {
                Ok(0) => std::thread::sleep(FOLLOW),
                Ok(_) if line.ends_with('\n') => {
                    let text = line.trim_end().to_owned();
                    eprintln!("chora-construct: cube manager: {text}");
                    if let (false, Ok(mut kept)) = (text.trim().is_empty(), last.lock()) {
                        *kept = text;
                    }
                    line.clear();
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
        let _ = std::fs::remove_file(&path);
    });
    Ok((child, tail))
}

impl Construct {
    /// At the start: the virtual outputs that an earlier construct left
    /// show the cube again when its window manager still runs on them, as
    /// after a crash; else they go.
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
            eprintln!("chora-construct: the cube of {MANAGER} again: {err}");
            self.remove_outputs(&left);
            return Ok(());
        }
        host.since = Instant::now();
        self.cube = Some(host);
        self.open_modes()?;
        self.frame()
    }

    /// Whether a client of the name of the window manager is connected.
    pub(super) fn manager_runs(&mut self) -> Result<bool, ClientError> {
        let clients: Value = self.rt.conn.call("clients.list", Value::Null)?;
        let list = clients.as_array().cloned().unwrap_or_default();
        Ok(list.iter().any(|c| c["name"].as_str() == Some(MANAGER)))
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
        while tail.lock().map(|t| t.clone()).unwrap_or_default() != "the last line" {
            assert!(
                Instant::now() < end,
                "the tail is {:?}",
                tail.lock().map(|t| t.clone())
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}
