# flakes-q r1 (F5: no flaky tests): FLK-HUD-TAB, FLK-XP-FACES

Model: sonnet, effort high (timing faults under load; each needs a cause with evidence, not a retry)
Lane: cloud, branch claude/flakes-q from 618ae513 (session head, train 94b). One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true`; `cargo check --workspace`; a failure or missing tool: send_message the PM "flakes-q BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: backlog.md rows FLK-HUD-TAB, FLK-XP-FACES; reports/flake-series.md "## r3"; reports/flakes-o.md and reports/flakes-l.md for the method. Not yours: the cells of the other flake lane (flakes-p: pull; flakes-q: construct-hud, xp-faces).
Items, each: reproduce under load (`--jobs 3` plus a second `cargo build` in a loop), cause with log evidence, red first, fix (a wait on the real state, never a sleep or a retry), then 20 loaded runs of the cell green; a row not reproduced in 20 loaded runs says so with the count and keeps the added failure log (commit it, it is the evidence for the next red):
1. FLK-HUD-TAB: construct-hud "light-1280x720@1.5: Tab opened no list" (once in a full gate at --jobs 2, 0 of 10 alone). First log the HUD state and key route (who got Tab, list node present) on failure.
2. FLK-XP-FACES: xp-faces "MISMATCH opens-under-pointer: measured gap" (once in a full gate at --jobs 2, 0 of 10 alone). First log the pointer position, the window rect and the frame time the measurement used.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,gate-lib,<your cells>`, then the touched cells twice more at --jobs 3; poll it in the same turn; never end a turn to wait.
Done when: per item red sha, fix sha, loaded-run count; backlog rows closed or named; reports/flakes-q.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/flakes-q and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
