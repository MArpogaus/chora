# co-p2-fix r1 (RC shared bar: no stray keys, key list by keys): construct co-25, co-26

Model: sonnet, effort high (key grabs of one plugin with a known precedent in chora-tiling; the PM reviews seeded)
Lane: cloud, branch claude/co-p2-fix from 79c8d2b5 (session branch, train 81). One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "co-p2-fix BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/construct.md co-2, co-25, co-26; reports/p2-r1-construct.md "## r1b"; feedback/2026-11-20-p2r1b-*.md. On the branch: docs/configuration.md:324-330 (how chora-tiling swallows unbound Super chords), plugins/chora-construct (key grabs), tests/gate-construct-keys.sh, tests/gate-doors.sh.
Items:
1. co-25 MAJOR: in a construct face (and stood back), no Super chord that the construct does not bind reaches the focused client. Red gate test first (gate-construct-keys: a chora-probe window --report focused in a face; Super+d, Super+q, Super+Tab, Super+1: the report holds none of their keys; Super+T, C, Space, F1 keep their actions). Fix as chora-tiling does (grab Super+KEY, Super+Shift+KEY, Super+Ctrl+KEY; swallow the unbound). Plain keys (no Super) still reach the client. Document it in docs/configuration.md beside the tiling paragraph.
2. co-26 MAJOR (verify): Super+F1 inside a face opens the key list (construct-help). Reproduce first in gate-construct-keys; if it already works, report the cell output and the r1b cause (likely the harness), no change. If red: fix, cell green.
Do not touch crates/chora input.rs (the stray-grab lane) or chora-tiling.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7 first, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,keys,doors,construct-hud,cube,out,pull,matrix`; poll it in the same turn; never end a turn to wait. A red cell you did not cause: rerun it alone once and name it.
Done when: red commits before fixes, gate cells pass, reports/co-p2-fix.md "## r1" on claude/session-log (per item: red sha, fix sha, test, gate cell, capture or probe report).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/co-p2-fix and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
