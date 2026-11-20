# handover-ti r2 (RC 2, shared bar "no lost window in any handover"): fix review r1 findings of 371f0fa1

Model: sonnet, effort high (the review names file, line and fix for each finding; routine follow-up)
Lane: cloud, branch claude/handover-ti at 371f0fa1. Stop at 250 k. Cost about 1.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "handover-ti BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/handover-ti.md "## r1" and "## Review r1" (the probe and findings rw-1..rw-5).
Items, in order:
1. rw-1 MAJOR plugins/chora-tiling/src/main.rs:1018 (crates/chora-kit/src/wm/focus.rs:86): after a click on the xp desktop, a click on tiling's window must take the keys back. Red first: add the click-back step (give s1 to tiling, click xp desktop at 5,400, click s1 at 640,400; core focus is s1, tiling holder, xp none) to tests/gate-handover-shared.sh. Fix: pointer_focus counts taken_by_other(NAME) and a core focus other than the sent surface as lost.
2. rw-2 MINOR plugins/chora-xp/src/wm/press.rs:227: the first xp desktop click after a tiling focus gives the icons the keys (resend desktop_keys in focus_moved when the core focus leaves a foreign window). Red first in the same cell or an xp unit test.
3. rw-4 NIT crates/chora/src/state.rs:1118: derive the told node from key_node(); unit test first.
4. rw-5 NIT crates/chora-kit/src/wm/focus.rs:181: holder() and state() agree for an own untracked node; unit test first.
5. rw-3 NIT: no code; note in the report that the r1 node cell was green-only, and keep test edits in red commits.
Each fix: its red test commit first, then the fix; reuse the existing gate helpers, no new copies (F2). Not in scope: the stacking by root order (known MINOR), I5.
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,gate-lib,matrix,virtual,wm-pane,mc-handover,layout,xp-faces` plus persona-selftest if tests/persona changed. Poll the gate in the same turn; never end a turn to wait.
Done when: each item red then green with shas; gate PASS lines; reports/handover-ti.md "## r2" with per item sha, test, cell, and the probe table rerun.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/handover-ti and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
