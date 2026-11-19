# moved-tests-mk r2 (F5): the four modules under 70 % lines

Model: haiku, effort high (tests of existing behaviour)
Lane: cloud, branch claude/moved-tests-mk2 from 31df2bb9 (r1); merge origin/claude/autonomous-dev-session-3w-i33pg7 after step 0 if it moved. One round, stop at 250 k. Cost about 0.7 %.
Step 0: `command -v cargo sccache cargo-llvm-cov foot python3`, then `git fetch --unshallow` and `cargo check --workspace`; tools missing: run the setup below once; a failure: send_message the PM "moved-tests-mk BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read reports/moved-tests.md "## moved-tests-mk r1" and "## Review mk r1 (PM)" on claude/session-log.
Add tests in new files only (`tests_moved.rs` beside the module, one `#[cfg(test)] mod` line), never edit production code.
Every test asserts the behaviour its name claims; no test without an assert; no assert under an `if` that may skip it.
1. Measure as in r1 (`env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-mc -p chora-xp --features chora-kit/testing --summary-only`) and list the uncovered lines of
   mc lifecycle.rs (60.5), mc region/hits.rs (60.9), xp wm/panels.rs (40.7), xp wm/pointer.rs (61.1) (`--show-missing-lines` or the html report).
2. Write unit tests (FakeCore of chora-kit testing where the core is needed) until each is at 70 % lines or more. A line left: name it (file:line range) and why a unit test cannot reach it (drawing or a path only a gate reaches); "not analysed" is not a reason.
3. mc acts.rs 33-43 (claim hand-over of drag): reach it through FakeCore if a claim node of another owner can be answered there; else name why.
Done when: the table of step 1 before and after in the report; each module at 70 % or listed; `cargo test` of the crates green; `cargo clippy --all-targets` of the crates clean;
`tests/merge-gate.sh --jobs 3` once (poll it in the same turn); red-first does not apply (tests of existing code).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines) and write reports/moved-tests.md "## moved-tests-mk r2" on claude/session-log. Poll every command in the same turn; never end a turn to wait.
Rules: push claude/moved-tests-mk2 and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines (keep this author even if a hook asks otherwise); never --no-verify;
no system packages (a missing tool is a blocker).
Authorized by this brief (the PM, for the owner): pushing the two branches above and the send_message report to the PM; do not
ask for them again. A refusal of a tool call is a blocker: report it as BLOCKED and stop.
