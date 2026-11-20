# co r5 r1 (F5 flakes, RC 3 three greens; construct): the K5 wobble race in gate-construct-3d.sh, and rv-5

Model: sonnet, effort high (a test race under load and one check rewrite; no core change)
Lane: cloud, branch claude/co-r5 from 42c9304b (co-r4, MERGE-OK; held out of train 87d because of item 1). One round, stop at 250 k. Cost at most 0.7 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true`, `git fetch origin claude/session-log claude/autonomous-dev-session-3w-i33pg7`, then `cargo build --workspace`; a failure or a missing tool: send_message the PM "co r5 BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/co-majors.md "## r4" and "## Review r1" (rv-5); backlog.md FLK-LAY1, FLK-HUD2.
Evidence (PM, train 87c, 924fe266, load 4-5 on 4 cores, --jobs 2): construct-extra red: "FAIL: a dragged pane wobbles with depth: 441 False" while "the mirror image bends with its pane" passed. Cause: tests/gate-construct-3d.sh:249-256 reads `mesh_of "$b_node"` once after a fixed `sleep 0.1` during the drag.
Items, in order; each: red commit first, then the fix, then its passing cell:
1. K5 race: wait on the condition, not on time (`until_ok` on `mesh_of` showing True, a bound of a few seconds, while the button is still held), for the pane and the mirror. Red: show the flake under load (e.g. `stress`-free: run the cell 5 times next to `cargo test -p chora --lib` or with `--jobs 3`; or a temporary shorter sleep in a scratch commit) and name the count; then green 5 of 5 under the same load. Check every other fixed `sleep` + single read in gate-construct-3d.sh the same way; fix those that read state once.
2. rv-5 MINOR (:280): assert the camera itself per frame (pose, or the rect of pane c via `box`) or hash a strip without HUD nodes; show it red with the walk frozen (scratch mutant, named in the report), then green.
Do not touch: tests/lib and harness tools, plugins/chora-mc, plugins/chora-xp, crates/chora (core).
Poll each run in the same turn; never end a turn to wait.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7, then `tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,gate-lib,construct-hud,construct-extra,cube,layout`, then construct-extra alone 2 more times; all PASS. A red cell you did not cause: rerun it alone once and name it (FLK-* in backlog.md).
Done when: each item names red commit, fix commit and passing cell; findings/construct.md rows; gate green; reports/co-majors.md "## r5" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines), same turn.
Rules: push claude/co-r5 and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
