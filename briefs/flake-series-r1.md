# flake-series r1 (F5): five full gates in a row, and the chora-kit coverage figure

Model: haiku, effort high (runs and counts tests; no production code)
Lane: cloud, branch claude/flake-series from 7d9ec18e (train 67, matrix cap in). One round, stop at 250 k. Cost about 1 %.
Step 0: `command -v cargo sccache cargo-llvm-cov foot python3`, then `git fetch --unshallow` and `cargo check --workspace`; tools missing: run the setup below once; a failure: send_message the PM "flake-series BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read design/foundation.md row F5 and backlog.md F17 and MX1 on claude/session-log.
Runs take long: start each gate in the background and poll it in the same turn; never end a turn to wait.
1. Run `tests/merge-gate.sh --jobs 3` (no --push) five times in a row on 7d9ec18e. Keep every log. For each run: exit code, the failing steps and the first failing line.
2. Any red step: rerun that step alone (`--only <step>`) ten times; report count red of ten and the cause line. Do not fix, do not change a timeout, do not mark a test ignored or known-red.
3. Coverage: `env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-kit --features testing --summary-only`; report the lines figure (F5 needs 80 %) and the five files with the most uncovered lines.
Done when: reports/flake-series.md "## r1" on claude/session-log with a table run 1-5 (exit, red steps), the reruns of step 2, the coverage figure and file list; no commit on the code branch; send_message the PM "flake-series DONE: <greens>/5, kit <n> %" and stop.
