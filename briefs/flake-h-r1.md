# flake-h r1 (F5): the three gate cells that went red under load

Model: sonnet, effort high (routine root causes of test waits)
Lane: cloud, branch claude/flake-h from the train 72 merge 219d5afe. One round, stop at 250 k. Cost about 1 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "flake-h r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read on claude/session-log: reports/matrix-cap.md "## mx1-m2 r1" (runs 1-2 red: layout HUD audit, doors helper cells), backlog.md rows "flake (11-19 13:32, wm5 r3)" (pull pane 169 px for 200) and FF1 (rw-1 wait counts iterations, rw-4 no per-run timeout), reports/flake-series.md.
1. Reproduce: run `tests/merge-gate.sh --jobs 3 --only matrix,layout,doors,construct-pull` (names from `--list`) 5 times; log each red cell with its line and the time it waited.
2. Each red cell: find the wait or the assert that races (a wait in iterations instead of seconds, a fixed sleep, a size read before the layout settles); fix the test wait, not the product, unless the product is wrong (then a red test first and say so). FF1 rw-1 (gate-tiling-audit.sh:86 waits in seconds) and rw-4 (a per-run timeout in under-load.sh) are in scope.
3. After the fixes: 5 more runs of the same command, all green.
Poll each run in the same turn; never end a turn to wait.
Done when: 5 of 5 green after the fixes; reports/flake-h.md "## r1" on claude/session-log with the red cells found, the cause of each, the fix commit and the 5 run results; head sha. A cell that never went red in 10 runs: say so, no change.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/flake-h and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
