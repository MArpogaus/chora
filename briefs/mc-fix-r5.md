# mc-fix-r5 (RC 2 persona round 4; rule 6, mc MAJOR): review r4 rv-r4-1 I5 cover rule, NITs

Model: sonnet, effort high (one named MAJOR with file:line, probe and fix from the seeded review)
Lane: cloud, branch claude/mc-fix at a8cd0de0 (merge origin/claude/autonomous-dev-session-3w-i33pg7 first when it moved past d81c8293). One round, stop at 250 k. Cost about 0.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true`; `cargo check --workspace`; a failure: send_message the PM "mc-fix r5 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/mc-fix.md "## r4" and "## Review r4" (rv-r4-1 to rv-r4-3).
Items (a red test commit first, then the fix):
1. rv-r4-1 MAJOR, tests/acts/invariants.py:395 `hides`: a whole-output picture counts as a cover only when its owner is the look that is shown. Red: the review's probe (mc drawn over xp: xp text, mc-bg, mc text) as a failing case in tests/acts/test_acts.py; it must breach I5. Keep the r4 cases green. Touch only `hides` and its tests (flakes-l owns the I5 timing rows).
2. rv-r4-2, rv-r4-3 NIT only if cheap: drop the moot name match at windows.rs:188 or say why it stays; log a failed refresh query once, not on every state.get (main.rs:660).
Gate: `tests/merge-gate.sh --jobs 2 --only pre-commit,gate-lib,ci-check,build,test,clippy,mc-paths,acts`; poll it in the same turn; never end a turn to wait. A red cell you did not cause: rerun it alone once and name it.
Done when: red commit before the fix; cells pass; reports/mc-fix.md "## r5" on claude/session-log (per item: red sha, fix sha, test, cell); head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/mc-fix and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
