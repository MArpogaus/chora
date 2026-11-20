# persona-begin r1 (RC 2 persona round 4; harness only): each goal compares with its own begin

Model: sonnet, effort medium (contained harness fix in tests/persona with selftests)
Lane: cloud, claude/persona-begin at dcb77733 (session head). Cost at most 0.8 %; stop at 200 k context.
Why: reports/p2-r3-tiling.md line 5 (on claude/session-log): "L1 compares with `start`, L2 vacuous (empty config), T7 and L3 compare with a `begin` taken before the run (one `end` for several goals). A repeat `end` compares with the end state of the previous `end`, so a second pass of T7 means nothing." Rows 10, 17, 18, 20: L1, L2, L3 and T7 were red by the harness, not the product. Read tests/persona/check.py (around lines 554, 590, 634), round.sh, README, tests/persona-selftest.sh.
Step 0 (no setup script): `git fetch --unshallow`, `cargo check --workspace`, then `cargo build --workspace`; a failure: send_message the PM "persona-begin r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Items (a red case in tests/persona-selftest.sh first, then the fix; no product code):
1. Every goal (L1, L2, L3, T7 and the rest) compares with a begin snapshot taken right before that goal; `end` for goal N never reuses the begin or end of goal N-1. round.sh and the README run begin and end per goal.
2. L1 compares with its own begin, not `start`; a window closed by the persona is "closed", not "gone".
3. L2 with an empty or missing config: the contract says "not measured (no config)", never red or green.
4. L3 holders: windows given away by a later goal do not count against L3.
5. T7: a given window that is shown but under another window is a pass for "not lost" and a separate MINOR line "covered" (known root-order MINOR), not "lost".
Not yours: product code, gate cells outside tests/persona and tests/persona-selftest.sh.
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,gate-lib,doors,acts`, `tests/persona-selftest.sh`, and `tests/persona/smoke.sh` once. Poll in the same turn; never end a turn to wait.
Report: append "## r1" to reports/persona-begin.md on claude/session-log (per item red sha, fix sha, selftest cases, gate cells); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/persona-begin and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
