# flakes-m r1 (F5: no flaky tests; the flakes no lane owns): FLAKE-CUBE-QUEUE, FJ-R2 doors Escape, xp "no helper word"

Model: sonnet, effort high (timing faults under load; each needs a cause with evidence, not a retry)
Lane: cloud, branch claude/flakes-m from d81c8293 (session head, train 87e). One round, stop at 250 k. Cost about 1.2 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true`; `cargo check --workspace`; a failure or missing tool: send_message the PM "flakes-m BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: backlog.md FLAKE-CUBE-QUEUE (line ~301), FJ-R2 and FLAKES-K-RV (~309, ~320); reports/audit-f.md "## r3" table row 2; reports/flakes-k.md. Not yours: tiling-look ink, acts xp mixer, hud, layout, tiling-audit (lane flakes-l) and tests/gate-construct-3d.sh (co-r5 in train 88).
Items, each: reproduce under load (`--jobs 3` plus a second `cargo build` in a loop), cause with log evidence, red first, fix (a wait on the real state, never a sleep or a retry), then 20 loaded runs of the cell green; a row not reproduced in 20 loaded runs says so with the count:
1. FLAKE-CUBE-QUEUE: tests/gate-construct-cube.sh "a turn key during a turn did not queue one turn".
2. FJ-R2: tiling, mc and construct doors press Escape before the chat field holds the keyboard (xp fixed 6192f696; take its pattern); FLAKES-K-RV rv-1 doors/lib.sh:187 field_has_keyboard must see the chat take the keyboard, not any leave.
3. tests/doors/xp.sh "no helper word": evidence only unless reproduced.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,gate-lib,cube,doors,chat-keys`, then the touched cells twice more at --jobs 3.
Done when: per item red sha, fix sha, loaded-run count; backlog rows closed or named; reports/flakes-m.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/flakes-m and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
