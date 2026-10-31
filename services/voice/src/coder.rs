//! The coding tasks of `coder_cmd` in clones of the projects of the
//! inventory (docs/voice.md section 3, WP7; ported from the Python
//! agent). One task runs at a time, in the background, in
//! `WORKTREES/NAME-N` on the branch `voice/N`, or in `WORKTREES/N`
//! without a project. Its output goes to the file of the clone with
//! `.log` after its name, outside the repository. A line of JSON, as the
//! stream-json of Claude Code, gives the steps and the result; plain lines
//! count as steps. The harness is any command that takes the task as its
//! last argument.

use std::{
    path::{Path, PathBuf},
    process::Stdio,
    sync::{Arc, Mutex, PoisonError, mpsc::Sender},
    time::{Duration, Instant},
};

use serde_json::{Value, json};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    sync::oneshot,
};

use crate::core::Input;

/// The tools of the LLM.
pub const TOOLS: [&str; 4] = ["code_task", "code_status", "code_cancel", "list_projects"];

/// The longest line of the coder, such as a tool result of stream-json.
const LINE_LIMIT: u64 = 16 << 20;

/// The longest result that the agent speaks, in characters.
const SAID_CHARS: usize = 400;

/// What the coder tells the core.
#[derive(Debug, PartialEq)]
pub enum Event {
    /// A task started.
    Started(String),
    /// The last step of the task.
    Step(String),
    /// The task ended: whether it is done, and the sentence to speak.
    Done { ok: bool, said: String },
}

/// The coder; clones share the task.
#[derive(Clone)]
pub struct Coder {
    inner: Arc<Mutex<Inner>>,
    /// The directory of `inventory` and `worktrees`.
    data: PathBuf,
    tx: Sender<Input>,
}

struct Inner {
    cmd: Vec<String>,
    job: Option<Job>,
    /// The spoken result of the last task.
    last: String,
}

struct Job {
    dir: PathBuf,
    started: Instant,
    step: String,
    stop: Option<oneshot::Sender<()>>,
}

/// `$XDG_DATA_HOME/chora`, else `~/.local/share/chora`.
pub fn data_dir() -> PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .filter(|d| !d.is_empty())
        .map_or_else(|| crate::expand("~/.local/share"), PathBuf::from);
    base.join("chora")
}

impl Coder {
    pub fn new(cmd: Vec<String>, data: PathBuf, tx: Sender<Input>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                cmd,
                job: None,
                last: String::new(),
            })),
            data,
            tx,
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().expect("no panic with the lock")
    }

    /// A new harness; an empty one turns the tools off. A running task
    /// goes on.
    pub fn set_cmd(&self, cmd: Vec<String>) {
        self.lock().cmd = cmd;
    }

    /// The function specs for the LLM; none without a harness.
    pub fn specs(&self) -> Vec<Value> {
        if self.lock().cmd.is_empty() {
            return Vec::new();
        }
        let function = |name: &str, description: &str, parameters: Value| {
            json!({"type": "function", "function": {
                "name": name, "description": description, "parameters": parameters}})
        };
        let none = json!({"type": "object", "properties": {}});
        vec![
            function(
                "code_task",
                "Start a coding task in the background, in a clone of a project, or in an \
                 empty repository without one. Its result is spoken when it ends.",
                json!({"type": "object", "required": ["task"], "properties": {
                    "task": {"type": "string", "description": "the task in full"},
                    "project": {"type": "string", "description": "a name of list_projects"}}}),
            ),
            function(
                "code_status",
                "Whether a coding task runs, for how long, and its last step; else the last \
                 result.",
                none.clone(),
            ),
            function("code_cancel", "Stop the running coding task.", none.clone()),
            function(
                "list_projects",
                "The projects that code_task can clone.",
                none,
            ),
        ]
    }

    fn projects(&self) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(self.data.join("inventory"))
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        names.sort();
        names
    }

    /// Runs one of `TOOLS` and answers the text for the LLM.
    pub fn call(&self, name: &str, args: &Value) -> Value {
        match name {
            "code_task" => {
                let Some(task) = args["task"].as_str() else {
                    return "error: code_task needs a task".into();
                };
                self.start(task, args["project"].as_str().filter(|p| !p.is_empty()))
            }
            "list_projects" => {
                let names = self.projects();
                if names.is_empty() {
                    format!("no projects in {}", self.data.join("inventory").display())
                } else {
                    names.join(", ")
                }
            }
            "code_cancel" => self.cancel(),
            _ => self.status(),
        }
        .into()
    }

    /// Stops the running task; its result comes as for any end.
    pub fn cancel(&self) -> String {
        let mut inner = self.lock();
        match inner.job.as_mut().and_then(|j| j.stop.take()) {
            Some(stop) => {
                let _ = stop.send(());
                "the coding task stops".into()
            }
            None => "no coding task runs".into(),
        }
    }

    fn status(&self) -> String {
        let inner = self.lock();
        match &inner.job {
            Some(job) => format!(
                "it runs for {} min in {}; last step: {}",
                job.started.elapsed().as_secs() / 60,
                job.dir.display(),
                job.step
            ),
            None if inner.last.is_empty() => "no coding task runs".into(),
            None => format!("no coding task runs; the last one: {}", inner.last),
        }
    }

    /// Starts a task in a new clone of `project`, or in an empty
    /// repository.
    fn start(&self, task: &str, project: Option<&str>) -> String {
        let mut inner = self.lock();
        if inner.job.is_some() {
            return "a coding task runs already; ask for its status or cancel it".into();
        }
        if inner.cmd.is_empty() {
            return "error: no coder_cmd in the settings".into();
        }
        if let Some(p) = project
            && !self.projects().iter().any(|n| n == p)
        {
            return format!("no project {p}; list_projects names them");
        }
        if task.trim_start().starts_with('-') {
            return "a task must not start with -".into();
        }
        let stem = project.map(|p| format!("{p}-")).unwrap_or_default();
        let worktrees = self.data.join("worktrees");
        let number = (1..=u32::MAX)
            .find(|n| !worktrees.join(format!("{stem}{n}")).exists())
            .expect("a free number");
        let dir = worktrees.join(format!("{stem}{number}"));
        let (stop, stopped) = oneshot::channel();
        inner.job = Some(Job {
            dir: dir.clone(),
            started: Instant::now(),
            step: "starting".into(),
            stop: Some(stop),
        });
        let source = project.map(|p| self.data.join("inventory").join(p));
        let started = format!("the coding task started in {}", dir.display());
        let (coder, cmd, task) = (self.clone(), inner.cmd.clone(), task.to_owned());
        crate::log::info(&format!("coder: task in {}: {task}", dir.display()));
        let _ = self.tx.send(Input::Code(Event::Started(task.clone())));
        tokio::spawn(async move {
            let run = async {
                clone(source.as_deref(), &dir, number).await?;
                coder.run(&cmd, &task, &dir).await
            };
            let (ok, state, result) = tokio::select! {
                result = run => match result {
                    Ok((true, r)) => (true, "is done", r),
                    Ok((false, r)) => (false, "failed", r),
                    Err(e) => (false, "failed", e),
                },
                _ = stopped => (false, "was cancelled", String::new()),
            };
            coder.end(&dir, ok, state, &result);
        });
        started
    }

    /// Ends the task: the result for `code_status` and for the core.
    fn end(&self, dir: &Path, ok: bool, state: &str, result: &str) {
        let mut inner = self.lock();
        let step = inner.job.take().map(|j| j.step).unwrap_or_default();
        let name = dir.file_name().unwrap_or_default().to_string_lossy();
        let detail: String = if result.is_empty() { &step } else { result }
            .chars()
            .take(SAID_CHARS)
            .collect();
        inner.last = if detail.trim().is_empty() {
            format!("The coding task in {name} {state}.")
        } else {
            format!("The coding task in {name} {state}. {}", detail.trim())
        };
        crate::log::info(&format!("coder: {}", inner.last));
        let said = inner.last.clone();
        let _ = self.tx.send(Input::Code(Event::Done { ok, said }));
    }

    /// Runs the harness in `dir` and reads its output; answers whether it
    /// ended well, and its result.
    async fn run(&self, cmd: &[String], task: &str, dir: &Path) -> Result<(bool, String), String> {
        let log_path = dir.with_file_name(format!(
            "{}.log",
            dir.file_name().unwrap_or_default().to_string_lossy()
        ));
        let log =
            std::fs::File::create(&log_path).map_err(|e| format!("{}: {e}", log_path.display()))?;
        let err = log.try_clone().map_err(|e| e.to_string())?;
        let (program, args) = cmd.split_first().ok_or("coder_cmd is empty")?;
        // Its own process group, so that a stop ends what it started too.
        let mut child = tokio::process::Command::new(program)
            .args(args)
            .arg(task)
            .current_dir(dir)
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(err)
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("{program}: {e}"))?;
        let group = Group::new(child.id());
        let out = BufReader::new(child.stdout.take().expect("piped"));
        let result = self.read(out, tokio::fs::File::from_std(log)).await?;
        let status = child.wait().await.map_err(|e| e.to_string())?;
        drop(group);
        Ok((status.success(), result))
    }

    /// Writes the lines of the harness to the log and reports its steps;
    /// answers its result.
    async fn read(
        &self,
        mut out: impl AsyncBufReadExt + Unpin,
        mut log: tokio::fs::File,
    ) -> Result<String, String> {
        let mut result = String::new();
        while let Some(line) = read_line(&mut out).await? {
            log.write_all(&line).await.map_err(|e| e.to_string())?;
            let step;
            (step, result) = self::step(&String::from_utf8_lossy(&line), result);
            if step.is_empty() {
                continue;
            }
            if let Some(job) = &mut self.lock().job {
                job.step.clone_from(&step);
            }
            let _ = self.tx.send(Input::Code(Event::Step(step)));
        }
        Ok(result)
    }
}

/// A clone of `source` in `dir` on the branch `voice/N` without its
/// remote, or an empty repository without a source.
async fn clone(source: Option<&Path>, dir: &Path, number: u32) -> Result<(), String> {
    let branch = format!("voice/{number}");
    let d = dir.to_string_lossy();
    let Some(source) = source else {
        return git(&["init", "--quiet", "-b", &branch, &d]).await;
    };
    let s = source.to_string_lossy();
    git(&["clone", "--quiet", "--no-hardlinks", &s, &d]).await?;
    git(&["-C", &d, "switch", "--quiet", "-c", &branch]).await?;
    // No remote: a push from the clone must not reach the project.
    git(&["-C", &d, "remote", "remove", "origin"]).await
}

/// The process group of the harness: its drop ends what is left of it,
/// also after a stop, so that no coder runs on next to the next task.
pub struct Group(Option<u32>);

/// The groups that may run, for `end_groups` at the exit.
static GROUPS: Mutex<Vec<u32>> = Mutex::new(Vec::new());

impl Group {
    pub fn new(pid: Option<u32>) -> Self {
        if let Some(pid) = pid {
            let mut groups = GROUPS.lock().unwrap_or_else(PoisonError::into_inner);
            groups.retain(|g| group_alive(*g));
            groups.push(pid);
        }
        Self(pid)
    }
}

/// Sends `signal` to the group `pgid`; false when no process got it.
fn kill_group(pgid: u32, signal: &str) -> bool {
    std::process::Command::new("kill")
        .args(["-s", signal, "--", &format!("-{pgid}")])
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

impl Drop for Group {
    fn drop(&mut self) {
        let Some(pid) = self.0 else { return };
        if kill_group(pid, "TERM") {
            // ponytail: a fixed grace in a thread; the group may ignore
            // TERM. At the exit `end_groups` waits instead.
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(5));
                kill_group(pid, "KILL");
            });
        }
    }
}

/// At the exit: ends the groups of `Group` and `more`.
pub fn end_groups(more: &[u32]) {
    let mut groups = std::mem::take(&mut *GROUPS.lock().unwrap_or_else(PoisonError::into_inner));
    groups.extend(more);
    end(groups);
}

/// TERM to `groups`, up to 5 s for all of their processes to end, then
/// KILL to the rest and up to 1 s more.
pub(crate) fn end(mut groups: Vec<u32>) {
    groups.sort_unstable();
    groups.dedup();
    groups.retain(|g| group_alive(*g));
    let wait = |groups: &[u32], limit: Duration| {
        let start = Instant::now();
        while groups.iter().any(|g| group_alive(*g)) && start.elapsed() < limit {
            std::thread::sleep(Duration::from_millis(50));
        }
    };
    for g in &groups {
        kill_group(*g, "TERM");
    }
    wait(&groups, Duration::from_secs(5));
    groups.retain(|g| group_alive(*g));
    for g in &groups {
        crate::log::warn(&format!("process group {g} ignored TERM; KILL"));
        kill_group(*g, "KILL");
    }
    wait(&groups, Duration::from_secs(1));
}

/// Whether a process of the group `pgid` runs; a zombie has ended.
pub fn group_alive(pgid: u32) -> bool {
    let pgid = pgid.to_string();
    let entries = std::fs::read_dir("/proc").into_iter().flatten().flatten();
    entries
        .filter_map(|e| std::fs::read_to_string(e.path().join("stat")).ok())
        .any(|stat| {
            // After the name: the state, the parent and the group.
            let rest = stat.rsplit_once(") ").map_or("", |(_, r)| r);
            let fields: Vec<&str> = rest.split(' ').take(3).collect();
            fields.len() == 3 && fields[0] != "Z" && fields[2] == pgid
        })
}

/// One line of at most `LINE_LIMIT` bytes; none at the end.
pub async fn read_line(
    out: &mut (impl AsyncBufReadExt + Unpin),
) -> Result<Option<Vec<u8>>, String> {
    let mut line = Vec::new();
    let n = out
        .take(LINE_LIMIT + 1)
        .read_until(b'\n', &mut line)
        .await
        .map_err(|e| e.to_string())?;
    if n as u64 > LINE_LIMIT {
        return Err(format!(
            "a line of the coder is longer than {LINE_LIMIT} bytes"
        ));
    }
    Ok((n > 0).then_some(line))
}

/// Runs git and answers its error.
async fn git(args: &[&str]) -> Result<(), String> {
    let out = tokio::process::Command::new("git")
        .args(args)
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|e| format!("git: {e}"))?;
    if out.status.success() {
        return Ok(());
    }
    Err(format!(
        "git {}: {}",
        args[0],
        String::from_utf8_lossy(&out.stderr).trim()
    ))
}

/// The step that a line of coder output shows, and the result so far. A
/// stream-json event of a tool call or a text gives a step, the result
/// event the result, other stream-json events no step. A line that is
/// no JSON object is a step as it is.
fn step(line: &str, result: String) -> (String, String) {
    let short = |s: &str| s.trim().chars().take(200).collect::<String>();
    let Ok(Value::Object(event)) = serde_json::from_str::<Value>(line) else {
        return (short(line), result);
    };
    if event.get("type").and_then(Value::as_str) == Some("result") {
        let r = event
            .get("result")
            .and_then(Value::as_str)
            .unwrap_or_default();
        return (
            String::new(),
            if r.is_empty() { result } else { r.to_owned() },
        );
    }
    let parts = event
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(Value::as_array);
    for part in parts.into_iter().flatten() {
        match part["type"].as_str() {
            Some("tool_use") => {
                let target = part["input"]
                    .as_object()
                    .and_then(|i| i.values().next())
                    .map(|v| v.as_str().map_or_else(|| v.to_string(), str::to_owned))
                    .unwrap_or_default();
                let name = part["name"].as_str().unwrap_or_default();
                return (short(&format!("{name} {target}")), result);
            }
            Some("text") => return (short(part["text"].as_str().unwrap_or_default()), result),
            _ => {}
        }
    }
    (String::new(), result)
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    #[test]
    fn steps_come_from_stream_json_and_plain_lines() {
        let tool = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Edit","input":{"file_path":"a.rs"}}]}}"#;
        assert_eq!(step(tool, "r".into()), ("Edit a.rs".into(), "r".into()));
        let text =
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Ich lese."}]}}"#;
        assert_eq!(step(text, String::new()).0, "Ich lese.");
        let done = r#"{"type":"result","result":"Fertig."}"#;
        assert_eq!(step(done, String::new()), (String::new(), "Fertig.".into()));
        assert_eq!(
            step(r#"{"type":"system"}"#, "r".into()),
            (String::new(), "r".into())
        );
        assert_eq!(step("compiling\n", String::new()).0, "compiling");
        assert_eq!(step("[1, 2]", String::new()).0, "[1, 2]");
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("chora-voice-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sh(script: &str) -> Vec<String> {
        vec!["sh".into(), "-c".into(), script.into(), "coder".into()]
    }

    /// Waits for the end of the task and answers its events.
    fn events(rx: &mpsc::Receiver<Input>) -> Vec<Event> {
        let mut events = Vec::new();
        loop {
            let Ok(Input::Code(event)) = rx.recv_timeout(Duration::from_secs(10)) else {
                panic!("no end: {events:?}");
            };
            let done = matches!(event, Event::Done { .. });
            events.push(event);
            if done {
                return events;
            }
        }
    }

    #[tokio::test]
    async fn a_task_needs_a_known_project_and_no_option() {
        let data = temp("coder-check");
        std::fs::create_dir_all(data.join("inventory/demo")).unwrap();
        let (tx, _rx) = mpsc::channel();
        let coder = Coder::new(sh("true"), data.clone(), tx);
        assert_eq!(coder.call("list_projects", &json!({})), "demo");
        let answer = |args| coder.call("code_task", &args).as_str().unwrap().to_owned();
        assert!(answer(json!({"task": " -rf"})).contains("must not"));
        assert!(answer(json!({"task": "x", "project": "nope"})).contains("no project"));
        assert!(answer(json!({})).contains("needs a task"));
        let _ = std::fs::remove_dir_all(&data);
    }

    #[tokio::test]
    async fn a_task_runs_in_a_clone_without_origin_and_logs_next_to_it() {
        let data = temp("coder");
        let project = data.join("inventory/demo");
        git(&["init", "--quiet", &project.to_string_lossy()])
            .await
            .unwrap();
        let (tx, rx) = mpsc::channel();
        // The task is the last argument; $0 is "coder".
        let coder = Coder::new(
            sh(
                r#"echo "$1" > task.txt; git remote; echo working; echo '{"type":"result","result":"Alles gut."}'"#,
            ),
            data.clone(),
            tx,
        );
        let started = coder.call(
            "code_task",
            &json!({"task": "Mach es schön.", "project": "demo"}),
        );
        assert!(
            started.as_str().unwrap().ends_with("worktrees/demo-1"),
            "{started}"
        );
        let events = tokio::task::spawn_blocking(move || events(&rx))
            .await
            .unwrap();
        assert_eq!(events[0], Event::Started("Mach es schön.".into()));
        assert!(
            events.contains(&Event::Step("working".into())),
            "{events:?}"
        );
        assert_eq!(
            events.last().unwrap(),
            &Event::Done {
                ok: true,
                said: "The coding task in demo-1 is done. Alles gut.".into()
            }
        );
        let clone = data.join("worktrees/demo-1");
        assert_eq!(
            std::fs::read_to_string(clone.join("task.txt")).unwrap(),
            "Mach es schön.\n"
        );
        let log = std::fs::read_to_string(data.join("worktrees/demo-1.log")).unwrap();
        assert!(
            log.starts_with("working\n"),
            "no remote, so git remote prints nothing: {log}"
        );
        assert!(
            coder
                .call("code_status", &json!({}))
                .as_str()
                .unwrap()
                .ends_with("Alles gut.")
        );
        let _ = std::fs::remove_dir_all(&data);
    }

    #[tokio::test]
    async fn a_cancel_stops_the_task_and_its_children() {
        let data = temp("coder-cancel");
        let (tx, rx) = mpsc::channel();
        let coder = Coder::new(sh("sleep 30 & echo $! > pid; wait"), data.clone(), tx);
        coder.call("code_task", &json!({"task": "warte"}));
        let dir = data.join("worktrees/1");
        let start = Instant::now();
        while !dir.join("pid").exists() {
            assert!(start.elapsed() < Duration::from_secs(5));
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert!(
            coder
                .call("code_task", &json!({"task": "noch eine"}))
                .as_str()
                .unwrap()
                .contains("runs already")
        );
        assert!(
            coder
                .call("code_status", &json!({}))
                .as_str()
                .unwrap()
                .starts_with("it runs")
        );
        assert_eq!(coder.cancel(), "the coding task stops");
        let events = tokio::task::spawn_blocking(move || events(&rx))
            .await
            .unwrap();
        let Event::Done { ok, said } = events.last().unwrap() else {
            unreachable!()
        };
        assert!(!ok && said.contains("was cancelled"), "{said}");
        // The sleep in the group ends with it.
        let pid = std::fs::read_to_string(dir.join("pid")).unwrap();
        // A zombie counts as gone.
        let alive = || {
            std::fs::read_to_string(format!("/proc/{}/stat", pid.trim()))
                .is_ok_and(|stat| !stat.contains(") Z "))
        };
        let start = Instant::now();
        while alive() {
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "the harness runs on"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert_eq!(coder.cancel(), "no coding task runs");
        let _ = std::fs::remove_dir_all(&data);
    }
}
