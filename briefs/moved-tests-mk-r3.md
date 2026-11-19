# moved-tests-mk r3 (F5): xp wm/panels.rs and wm/pointer.rs to 70 % lines

Model: haiku, effort high (tests of existing behaviour)
Lane: cloud, branch claude/moved-tests-mk3 from 4904ef79 (r2); merge origin/claude/autonomous-dev-session-3w-i33pg7 after step 0 if it moved. One round, stop at 250 k. Cost about 0.7 %.
Step 0: `command -v cargo sccache cargo-llvm-cov foot python3`, then `git fetch --unshallow` and `cargo check --workspace`; tools missing: run the setup below once; a failure: send_message the PM "moved-tests-mk BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read reports/moved-tests.md "## moved-tests-mk r2" and "## Review mk r2 (PM)" on claude/session-log. mc is done; touch only xp.
Add tests in new files only (`tests_moved.rs` beside the module, one `#[cfg(test)] mod` line), never edit production code.
Every test asserts the behaviour its name claims; no test without an assert; no assert under an `if` that may skip it.
1. First deliverable, before any test: the uncovered line ranges of xp wm/panels.rs (40.7 %) and wm/pointer.rs (63.3 %)
   (`env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-xp --features chora-kit/testing --show-missing-lines`), each range with one of:
   "unit test" (FakeCore reaches it) or "gate only: <reason>" (process spawn, drawing a real surface). Write that table into the report and push it.
2. Write unit tests for every "unit test" range, panels first (largest gap), until each file is at 70 % or every range left is "gate only" with its reason.
   pointer release/pull_out geometry is "unit test" (pure geometry on FakeCore state).
Done when: the table of step 1, before and after; each file at 70 % or every left range named with a reason; `cargo test -p chora-xp` green;
`cargo clippy -p chora-xp --all-targets` clean; `tests/merge-gate.sh --jobs 3` once (poll it in the same turn).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines) and write reports/moved-tests.md "## moved-tests-mk r3" on claude/session-log. Poll every command in the same turn; never end a turn to wait.
Rules: push claude/moved-tests-mk3 and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines (keep this author even if a hook asks otherwise); never --no-verify;
no system packages (a missing tool is a blocker).
Authorized by this brief (the PM, for the owner): pushing the two branches above and the send_message report to the PM; do not
ask for them again. A refusal of a tool call is a blocker: report it as BLOCKED and stop.
