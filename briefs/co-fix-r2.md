# co-fix r2 (RC 2 persona stop rule; construct MAJOR co-33 from p2-r2-co): the one-screen pull hint

Model: sonnet, effort medium (routine: a hint line and a gate measure; the decision is made).
Lane: cloud, branch claude/co-fix from e97b55f0 (co-fix r1, MERGE-OK, waiting for train 91). One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true`; `cargo check --workspace`; a failure or missing tool: send_message the PM "co-fix BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/co-fix.md "## r1" (Item 2) and "## PM review r1" (the co-33 decision); feedback/2026-11-20-p2r2co-{max,demo}.md. On the branch: tests/gate-construct-pull.sh, plugins/chora-construct (pull, HUD, input), docs/cube-modes.md, chora-xp wm/pointer.rs `pull_out`.
Decision (PM, fixed): no new gesture before the tag. On one output, where the designed title drag past the face edge cannot leave the face, the construct shows a one-line hint where such a drag starts that names the path that works today: ask the helper to pull this window out (`cube.pull`). Plain words, the look's HUD style, quiet, gone after the drag; never shown with a second output where the drag works.
Items (red gate test commit first, then the fix; no sleep, no retry):
1. Measure in gate-construct-pull, one output: a title drag in use mode to the screen edge and back (demo r2) shows the hint and moves no window across; a stood-back drag still turns the cube (K12) with no hint; with two outputs the designed pull still works and no hint shows. Red first.
2. The hint itself (smallest change in the construct HUD); docs/cube-modes.md one sentence beside line 198.
Not yours: a one-screen pull gesture (backlog), Super+C stepping back (backlog), xp face code beyond reading the drag.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7 if it moved; `tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,gate-lib,keys,construct-hud,construct-extra,cube,pull,doors`; poll in the same turn; never end a turn to wait. A red cell you did not cause: name it with its log line (FLK-* in backlog.md), do not rerun to green.
Done when: red sha, fix sha, cell, capture of the hint; findings/construct.md co-33 `fixed <sha> (red <sha>) cell:pull`; reports/co-fix.md "## r2" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/co-fix and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
