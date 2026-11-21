# flake-series r4 (F5): five full gates in a row at --jobs 2 on the session head 1c599c74

Model: haiku, effort high (runs and counts tests; no production code)
Lane: cloud, branch claude/flake-series-r4 at 1c599c74 (session head, train 98 green). One round, stop at 250 k. Cost at most 1 %.
Why: F5 needs five full gate runs green in a row at --jobs 2 (design/foundation.md row F5); r3 was 2 of 5 (reports/flake-series.md). Flakes-r merged since. This runs in the cloud so trains on the PM machine are not blocked.
Step 0: `command -v cargo sccache cargo-llvm-cov foot python3 ffmpeg`, then `git fetch --unshallow` and `cargo check --workspace`; no setup.sh (auto mode refuses it in lanes); a missing tool or failure: send_message the PM "flake-series BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: design/foundation.md row F5, reports/flake-series.md r3, backlog.md FLK-XP-FACES and FLK-DOORS-TILING-CHAT.
Runs take long: start each gate in the background and poll it in the same turn; never end a turn to wait.
1. Run `tests/merge-gate.sh --jobs 2` (no --push) five times in a row on 1c599c74. Keep every log. For each run: exit code, the failing steps and the first failing line. Record `nproc` and free memory once.
2. Any red step: rerun that step alone (`tests/merge-gate.sh --jobs 2 --only build,<step>`) ten times; report count red of ten and the cause line. Do not fix, do not change a timeout, do not mark a test ignored or known-red.
Done when: reports/flake-series.md "## r4" on claude/session-log with a table run 1-5 (exit, red steps, minutes) and the reruns of step 2; no commit on the code branch.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message: "flake-series DONE: <greens>/5" plus the red steps (at most 6 lines).
Rules: push claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
