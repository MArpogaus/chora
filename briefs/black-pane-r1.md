# black-pane-r1 (RC shared bar "a visible way to start a program"): sd-h1 and the render half of sh-23

Model: sonnet, effort high (one plugin's empty output, a known scene; the PM reviews)
Lane: cloud, branch claude/black-pane from bb16f2ff (session branch, train 79). One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "black-pane BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/shared-doors.md sd-h1, findings/shared.md sh-23, findings/tiling.md (open rows), the tour v4 journey-1 review that names 05-01-two-screens-R.png. On the branch: plugins/chora-tiling (bar, start door, output setup), tests/gate-layout.sh, tests/doors/lib.sh.
Items:
1. sd-h1: an empty tiling output in a two-screen setup is pure black, no bar, no start door. Red gate test first (two outputs, tiling on the second with no window: its bar and launcher door are drawn, a pixel or scene.boxes check), then the fix.
2. sh-23 render half: xp and tiling in two panes, the tiling pane stays black until a click. Red test first, then fix (the tiling pane draws its bar and door at once). Keyboard focus on click is another lane's (stray-grab): do not touch crates/chora input.rs.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7 first, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,layout,matrix,doors,tiling-look,tiling-audit,wm-pane`; poll it in the same turn; never end a turn to wait. A red cell you did not cause: rerun it alone once and name it.
Done when: red commits before fixes, gate cells pass, captures of the two screens before and after in reports/black-pane/, reports/black-pane.md "## r1" on claude/session-log (per item: red sha, fix sha, test, gate cell, capture).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/black-pane and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
