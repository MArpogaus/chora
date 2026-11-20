# review core-l r1 (F5, stability: WD2, L16): seeded review of claude/core-l before its merge

Model: opus, effort high, because the frame clock of the watchdog (crates/chora/src/safety.rs) is core and wrong excuses would blame or spare a slow plugin (rules.md 7: the builder never reviews itself)
Lane: cloud, read-only on code: claude/core-l head b1d1491d, base 355cdbe5 (8 files, +168 -27). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No full merge gate (the lane ran pre-commit, gate-lib, ci-check, build, test, clippy green).
Step 0 (no setup script): `git fetch --unshallow || true`; `git fetch origin claude/session-log claude/core-l`; `cargo check --workspace`; a failure: send_message the PM "review core-l BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/core-l-r1.md (done criteria), reports/core-l.md "## r1"; then `git diff 355cdbe5..b1d1491d`.
Checks, each with its command and files:
1) Red first: 4d2b133b fails on its parent and passes at f3baf36d; b8b7adf0 fails before b1d1491d and passes after (`cargo test -p chora --lib calm_frames_read_no_threads`, `cargo test -p chora-scene a_symlinked_folder` per sha in a scratch worktree).
2) WD2, safety.rs `Watchdog::clock_in` and `note`: the threads are read only after a frame over budget. Does a starved run still get excused from its second frame on, and can the first unmeasured frame alone lead to Blame or Kill (count frames against the run length in `note`)? Can `over` stay stale for an output that was removed or renamed? Do tty.rs, winit.rs and offscreen.rs all take the clock before the frame and pass it to frame_rendered?
3) The test-only `tasks()` (scratch dir) and the thread-local READS: no effect on a release build; the tests do not depend on order or on other tests' threads.
4) L16, chora-scene logs.rs `private_folder`: O_DIRECTORY|O_NOFOLLOW then fchmod. A symlink higher up the path, a folder replaced between create and open, an existing folder with mode 0755 (gets 0700), and the caller: one "no log file" line and chora goes on (name the caller file:line).
5) F22 closed without a commit: read the evidence in the report (0 of 50 under load); say whether the test e5e44719 names covers the reported race.
6) `cargo test -p chora --lib safety`, `cargo test -p chora-scene logs`, `cargo clippy --workspace --all-targets -- -D warnings`; no file over 1,500 lines.
Known, not findings: e2e red on gate-tiling-apps-click (fuzzel 1.9.2 in the container, F27/ti-1).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rv-N, severity, file:line, text, fix in one sentence). Append to reports/core-l.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
