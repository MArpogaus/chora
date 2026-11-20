# flakes-o r1 (F5: no flaky tests): FLK-CUBE-STILL, FLK-XP-HW

Model: sonnet, effort high (timing faults under load; each needs a cause with evidence, not a retry)
Lane: cloud, branch claude/flakes-o from ecfa2029 (flakes-m r1 head, PM MERGE-OK, joins train 91). One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true`; `cargo check --workspace`; a failure or missing tool: send_message the PM "flakes-o BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: backlog.md FLK-CUBE-STILL and FLK-XP-HW rows; reports/flakes-m.md "## r1" and "## PM review r1". Not yours: cells of lanes flakes-l (tiling-look ink, acts xp mixer, hud, layout, tiling-audit) and flakes-n (rw-3 panel test, MX-CAP); tests/doors/lib.sh wait_for_field (flakes-m, done).
Items, each: reproduce under load (`--jobs 3` plus a second `cargo build` in a loop), cause with log evidence, red first, fix (a wait on the real state, never a sleep or a retry), then 20 loaded runs of the cell green; a row not reproduced in 20 loaded runs says so with the count:
1. FLK-CUBE-STILL: tests/gate-construct-cube.sh "a still cube draws" (1 of 20 loaded on a6753908). First log what draws (node ids, damage per output) in the failing window, so a red names its cause.
2. FLK-XP-HW: tests/doors/xp.sh "no helper word": evidence only unless reproduced (0 of 20 in flakes-m).
3. If both are done below 150 k: pr1-1 of the flakes-m PM review (the cube queue check with keys spaced in real time during a turn, one inject with a pause inside, if the inject API has one; else say so).
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,gate-lib,cube,doors`, then the touched cells twice more at --jobs 3; poll it in the same turn; never end a turn to wait.
Done when: per item red sha, fix sha, loaded-run count; backlog rows closed or named; reports/flakes-o.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/flakes-o and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
