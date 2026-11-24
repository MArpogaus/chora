# sh42-fix r1 (rc 1, shared bar "no stray keys, no lost window in any handover"): sh-42 MAJOR, keys lost when tiling joins

Model: opus, effort high because the fault crosses core focus, chora-kit wm/focus and two looks' join paths (complex; earlier handover rounds rw-1..rw-5 touched the same code)
Lane: cloud, branch claude/sh42-fix-r1 at c3c37c2e (session head, t132). Stop at 250 k. Cost about 1.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "sh42-fix-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/shared.md sh-42; reports/sh42-repro.md "## r2" (steps, rate table: lost 5/5 xp, 5/5 mc; click recovers xp 5/5, mc 0/5; controls 0/3) and media/2026-11-24/sh42-repro/run.sh; reports/handover-ti.md "## r1", "## Review r1", "## r2" (rw-1: pointer_focus, taken_by_other); docs/decisions.md D119 on the branch.
Items, in order:
1. Red first: a gate cell (extend tests/gate-handover-shared.sh, reuse its helpers, no copies; F2) that opens a Foot in xp, joins chora-tiling --no-companions, types, and asserts the text reaches Foot and core focus is still that surface; the same for mc (page window after Return). Commit it red (it fails on c3c37c2e).
2. Root cause in the report (file:line): who clears the core focus at the join (tiling's join, core seat on a new client, or chora-kit focus) and why mc's holder becomes null and no click or Return gives it back.
3. Fix, smallest change at the cause (KISS, POLS): a joining look never takes the keyboard from a focused foreign window; in mc a click or Return must give the keys back. Unit test first where the cause is in Rust logic.
Not in scope: tiling look visuals and doors (TILING-DOORS r1 runs there; touch plugins/chora-tiling only at the join path), stacking order, MINOR items.
Gate: full `tests/merge-gate.sh --jobs 1` (product code: no --only, t99b). Poll the gate in the same turn; never end a turn to wait. Before ending: `pgrep -fa 'chora --headless'` empty.
Done when: red cell commit sha, then fix sha(s) with the cell green; the repro table rerun (5x xp, 5x mc, 3x control) shows 0/5 lost; gate PASS lines; reports/sh42-fix.md "## r1" with root cause, shas, cells and the table.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/sh42-fix-r1 and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash.
