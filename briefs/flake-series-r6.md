# flake-series r6 (F5, rc 3, rc 9): five full gates in a row at --jobs 2 on the session head 230ea0e5

Model: haiku, effort high (runs and counts tests; no production code)
Lane: cloud, branch claude/flake-series-r6 at 230ea0e5 (session head, train 113 green). One round, stop at 250 k. Cost at most 1 %.
Why: F5 reopened 11-22 (status page F5; a construct test went red twice after series r5). Since r5: CUBE-SETTLE, FLAKE-HINT, FLK-HANG, PIX-CAPSULE, TILING-FIX (t112d), MC-ROUND (t113). This is the second independent five-gate series. No rerun to green.
Step 0: `command -v cargo sccache cargo-llvm-cov foot python3 ffmpeg`, then `git fetch --unshallow`, `cargo check --workspace`; no setup.sh (auto mode refuses it in lanes); a missing tool or failure: send_message the PM "flake-series BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: design/foundation.md row F5, reports/flake-series.md r5.
Runs take long: start each gate in the background and poll it in the same turn; never end a turn to wait. Clean up between runs with tests/battle/kill-session.sh or the gate's own cleanup only (never by process name).
1. Run `tests/merge-gate.sh --jobs 2` (no --push) five times in a row on 230ea0e5. Keep every log. For each run: exit code, the failing steps and the first failing line, minutes. Record `nproc` and free memory once.
2. Any red step: rerun that step alone (`tests/merge-gate.sh --jobs 2 --only build,<step>`) ten times; report count red of ten and the cause line. Do not fix, do not change a timeout, do not mark a test ignored or known-red.
3. After the runs: `ls /tmp/gate-env.* | wc -l` and the count of chora processes left (pgrep -c chora), as leak evidence.
Done when: reports/flake-series.md "## r6" on claude/session-log with a table run 1-5 (exit, red steps, minutes), the reruns of step 2 and the step 3 counts; no commit on the code branch.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message: "flake-series DONE: <greens>/5" plus the red steps (at most 6 lines).
Rules: push claude/session-log only (fetch and merge on reject, no force; change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
