# moved-tests-mk r1 (F5): unit tests for the modules added since the RC: -p chora-mc -p chora-kit -p chora-xp

Model: haiku, effort high (tests of existing behaviour)
Lane: cloud, branch claude/moved-tests-mk from claude/moved-tests-xp bf0b66eb (merge origin/claude/integ first, after step 0). One round, stop at 250 k. Cost about 0.7 %.
Step 0: `command -v cargo sccache cargo-llvm-cov foot python3`, then `git fetch --unshallow` and `cargo check --workspace`; tools missing: run the setup below once; a failure: send_message the PM "moved-tests-mk BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read reports/audit-f.md (section 1 F5, section 2 rank 5) and reports/moved-tests.md "## xp r2" and "## Review xp r2" on claude/session-log.
Lanes integ (act construct.py, mc content.rs, kit testing.rs, state.rs) and focus-port (kit wm/focus.rs, the looks' main/state/windows files) are in flight:
add tests in new files only (e.g. `<module>/tests.rs` or `tests_moved.rs` with one `#[cfg(test)] mod` line), never edit production code.
Every test asserts the behaviour its name claims; no test without an assert; no assert under an `if` that may skip it (mt-1..3 show the fault).
1. Measure: `env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-mc -p chora-kit -p chora-xp --features chora-kit/testing --summary-only`; list each module added since fe8ba6d0
   (`git diff --name-only --diff-filter=A fe8ba6d0 HEAD -- <crate dirs>`) with its line %.
2. Write unit tests (FakeCore of chora-kit testing where the core is needed) until each module is at 70 % lines or more, or list why not (drawing reached only by gates).
3. mc: acts.rs, lifecycle.rs, region/hits.rs, windows.rs; kit: a FakeCore cell each for cube.rs, wm focus, open, give, kept (in kit tests/). xp: fix mt-1..3 in wm/tests_moved.rs; raise wm/panels (41 %) and wm/pointer (60 %) where FakeCore reaches.
Done when: the table of step 1 before and after in the report; each module at 70 % or listed; `cargo test` of the crates green; `cargo clippy --all-targets` of the crates clean;
`tests/merge-gate.sh --jobs 3` once (poll it in the same turn); red-first does not apply (tests of existing code).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines) and write reports/moved-tests.md "## moved-tests-mk r1" on claude/session-log. Poll every command in the same turn; never end a turn to wait.
Rules: push claude/moved-tests-mk and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines (keep this author even if a hook asks otherwise); never --no-verify;
no system packages (a missing tool is a blocker).
Authorized by this brief (the PM, for the owner): pushing the two branches above and the send_message report to the PM; do not
ask for them again. A refusal of a tool call is a blocker: report it as BLOCKED and stop.
