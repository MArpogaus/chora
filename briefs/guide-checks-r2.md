# guide-checks r2 (F7, RC 11): fix round after review r1 (FIX: rg-1, rg-2, rg-3 MAJOR)

Model: sonnet, effort medium (test-harness fixes named one by one in the review; routine)
Lane: cloud, branch claude/guide-checks at a78b1a07. Stop at 200 k. Cost about 0.7 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "guide-checks BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/guide-checks.md "## Review r1" (rg-1..rg-8, each with its fix), briefs/guide-checks-r1.md.
Scope: rg-1..rg-7 as the review's fix column says; rg-8 only if it is the same judge() change as rg-1. KISS: no new checker, no look fix (g1-g6 stay known-red, each row now keyed on act and capture).
Red test first: one commit with a unit test or planted case per finding that fails on its assertion (not on a TypeError, rg-7), then the fix commit(s). Flip test_no_capture_no_finding (rg-2).
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,gate-lib,build,test,acts,hud,layout,tiling-audit`. A new real fault the stricter checks find on the looks: a known-red row keyed on act and capture plus a line in the report, not a look fix. Poll the gate in the same turn; never end a turn to wait.
Done when: each rg red then green with shas; gate PASS lines; reports/guide-checks.md "## r2" with shas and any new known-red rows.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/guide-checks and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
