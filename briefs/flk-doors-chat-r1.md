# flk-doors-chat r1 (rc 2-3, F5, shared RC bar "a visible way to start"): doors red twice at tiling launcher-chat

Model: sonnet, effort medium (repro and root cause of one gate cell; two logs given)
Lane: cloud, claude/flk-doors-chat at 32a505db (train 99d head: 1c599c74 + guide-checks + ti-29 + mc-minor + handover-mo 73180551). Cost at most 0.8 %; stop at 250 k context. One area: the doors cell for tiling launcher-chat.
Why: train 99c (c996702a) and train 99d (32a505db) both failed the doors cell with one row: `launcher-chat  pass  missing  pass  pass` (tiling has no result row); every other cell and look passes. Train 99b (same merges plus handover-mo 60314da1) passed doors. The train is blocked on this; FLK-DOORS-TILING-CHAT in backlog.md on claude/session-log.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`; a failure: send_message the PM "flk-doors-chat BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Reproduce: `tests/merge-gate.sh --jobs 1 --only build,doors` five times on 32a505db; count tiling launcher-chat missing of five. Also on 1c599c74 (the session head) three times, and on 80f6ae67 (with ti-29) three times, to name the merge that brings it.
2) Cause with file:line: why does the tiling run produce no launcher-chat row (the check never runs, a timeout before it, or the helper panel of ti-29 covering the launcher)? Read the tiling part of the doors script and its log.
3) Fix KISS: a red test commit first where the fix is product code; no timeout raised without a measured reason; no known-red row.
4) Gate: the full gate `tests/merge-gate.sh --jobs 1` (no --only) green on the fix merged with origin/claude/autonomous-dev-session-3w-i33pg7, plus doors 5 of 5 green alone. A red cell you did not cause: name it with its log line; do not rerun to green. Start runs in the background and poll in the same turn; never end a turn to wait.
Done when: cause named, doors 5 of 5, full gate green.
Report: reports/flk-doors-chat.md "## r1" on claude/session-log (shas, counts per base, cause with file:line, gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/flk-doors-chat and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
