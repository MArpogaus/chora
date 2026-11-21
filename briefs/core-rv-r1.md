# core-rv r1 (rc 9 core safety): CORE-L-RV rv-1, rv-2, rv-5 and CM-R1

Model: sonnet, effort high (core safety code; the watchdog and the IPC resume; test first)
Lane: cloud, claude/core-rv at 1c599c74. Cost at most 1.4 %; stop at 250 k context. One area: crates/chora safety.rs, logs.rs, ipc.rs (core).
Why: plans/week-3.md slice 33 (claude/session-log); backlog.md CORE-L-RV and CM-R1; reports/core-l.md "Review r1" and the core-minors report name each finding:
- rv-1 safety.rs:274: an unmeasured first starved frame sets run.last, so a suspect never clears under intermittent load and a later run disconnects a stale suspect.
- rv-2 logs.rs:131: a symlinked log folder says "Not a directory"; the test does not check the message.
- rv-5 safety.rs:260: removed outputs are never forgotten.
- CM-R1 (rw-2 option a): docs/api.md "Window managers resume": kept nodes stay shown during the grace; a pinning test in ipc.rs.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`; a failure: send_message the PM "core-rv BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Per finding: a red unit test commit first (cargo test -p chora names it red), then the fix, KISS; no new lint allow; the line numbers above may have moved, find the code by name.
2) Gate: the full gate `tests/merge-gate.sh --jobs 1` (no --only) green on the branch merged with origin/claude/autonomous-dev-session-3w-i33pg7, and the opt-in watchdog-starved gate once if it exists (name its result). A red cell you did not cause: name it with its log line; do not rerun to green. Start runs in the background and poll in the same turn; never end a turn to wait.
Done when: four findings closed each with a red-then-green test, full gate green.
Report: reports/core-rv.md "## r1" on claude/session-log (shas, red test per finding, gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/core-rv and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
