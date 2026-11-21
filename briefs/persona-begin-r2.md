# persona-begin r2 (RC 2 persona round 4; harness only): merge the session head, fix the construct C2 selftest case

Model: sonnet, effort medium (one merge interaction in a selftest; contained)
Lane: cloud, claude/persona-begin at 4f07fe6c (r1 head, MERGE-OK). Cost at most 0.4 %; stop at 150 k context.
Why: train 95 (2026-11-21 01:15 UTC) went red in the gate cell `acts` (tests/persona-selftest.sh): "FAIL: construct: C2 is red with a window in the xp face: check.py: no `begin` for this goal (round.sh begin NAME before each goal)". The case (tests/persona-selftest.sh around line 226-252, commit a3327873 from claude/co-pane) is on the session head 618ae513 and calls `check end C2` without a per-goal begin; r1 made a begin per goal mandatory. The product is not at fault.
Step 0 (no setup script): `git fetch --unshallow || true`, `cargo check --workspace`, then `cargo build --workspace`; a failure: send_message the PM "persona-begin r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Items:
1. `git fetch origin claude/autonomous-dev-session-3w-i33pg7` and merge it (--no-ff, owner identity) into claude/persona-begin. Run tests/persona-selftest.sh: it must be red at the C2 case (that is the red; name the merge sha).
2. Fix the construct case: take the per-goal begin for C2 right before the give (the same form r1 uses for the other goals), so C2 compares with its own begin. Do not weaken check.py's "no begin" error. Grep tests/ for any other `check end`/`check.py end` call without its own begin and fix it the same way.
Not yours: product code, gate cells outside tests/persona and tests/persona-selftest.sh.
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,gate-lib,doors,acts`, and `tests/persona-selftest.sh` twice. Poll in the same turn; never end a turn to wait.
Report: append "## r2" to reports/persona-begin.md on claude/session-log (merge sha, fix sha, cells); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 4 lines with the head sha.
Rules: push claude/persona-begin and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
