# co-pane r1 (RC 2, construct round 3 fails): a dragged pane stays reachable; the key sheet closes; the checker reads the xp child

Model: sonnet, effort high
Lane: cloud, branch claude/co-pane at bbaad12b (co-fix r2, MERGE-OK). Step 1: `git merge --no-ff origin/claude/autonomous-dev-session-3w-i33pg7`; a conflict you cannot resolve in a few lines: send_message the PM and stop. Stop at 250 k. Cost about 1.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "co-pane BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/p2-r3-co.md "## r3", feedback/2026-11-20-p2r3co-{max,demo}.md, media/2026-11-20/persona-r3-{max,demo}/.
Items, each a red test commit first, then the fix:
1. (MAJOR candidate, I3) Max, walk room, no cube: a drag on the pane of window s1 lifts it so its top is cut off; three put-back drags change nothing; I3 x23 "the focused s1 is no shown window of construct". Find what the drag does (move in the picture or a pull). The pane must stay fully in view (clamp) or be put back by a drag, and the focused window must be a shown one. Confirm against a scene read (choractl state) first; if the scene shows s1 shown and only check.py disagrees, fix the checker instead and say so.
2. (medium) Key sheet: Escape stands the cube back and leaves the sheet open; its Close button does nothing; afterwards a Start menu is open unasked. Escape and Close close the sheet first (POLS), nothing else opens.
3. (harness) tests/persona/check.py reads construct's xp child: the demo run gives I1 "windows lost ['s2']" and I3 x48 for windows xp owns inside a construct face (lead since r1b). The checker must count windows of a child look as shown; a persona-selftest case first.
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,gate-lib,cube,pull,placed,construct-hud,construct-extra,walk,wm-pane` plus persona-selftest. Poll the gate in the same turn; never end a turn to wait.
Done when: each item red then green with shas; gate PASS lines; reports/co-pane.md "## r1" with per item sha, test, cell, capture.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/co-pane and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
