# cube-tests r1 (F5): construct cube/construct.rs to 70 % lines; kit cube IO gets FakeCore cells

Model: haiku, effort high (tests only; no production code)
Lane: cloud, branch claude/cube-tests from the session head f2b7112a (train 69). One round, stop at 250 k. Cost about 0.8 %.
Step 0: `command -v cargo sccache cargo-llvm-cov foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "cube-tests r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read on claude/session-log: reports/audit-f.md "## r2" (rank 1 and row F5), design/foundation.md row F5, reports/moved-tests.md (row 66).
Before: `env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-construct --features chora-kit/testing --summary-only`; note cube/construct.rs lines (41.81 %, 421 lines).
1. New file plugins/chora-construct/src/cube/tests_construct.rs with one `#[cfg(test)] mod tests_construct;` line in cube.rs. Through FakeCore cover `open_cube`, `check_cube`, `retry_cube`, `step_cube`, `close_cube`, `face_nodes`, `edge_nodes`, `add_outputs`, `remove_outputs`. Each test asserts a state or a request, not only that it runs.
2. New file crates/chora-kit/tests/cube.rs: FakeCore cells for `cube::exporter` (one found, none found), `list` (a listing), `turn` (a refused turn).
3. No production code change. If a range cannot be reached in a unit test (a child process), name it with the reason in the report.
Gate: `tests/merge-gate.sh --jobs 3` on the head, green (start it in the background and poll it in the same turn; never end a turn to wait). Do not mark tests ignored.
Done when: cube/construct.rs >= 70 % lines (or each left range named with why), the four kit cells pass, gate green; reports/cube-tests.md "## r1" on claude/session-log with before/after figures, the commands, head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): head sha, before/after %, gate.
Rules: push claude/cube-tests and claude/session-log only (fetch and merge on reject, no force); author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
