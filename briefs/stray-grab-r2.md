# stray-grab-r2 (RC shared bar "no stray keys"): fix review r1 (rv-1 MAJOR, rv-2 MINOR, rv-3/4 NIT)

Model: sonnet, effort high (the fix and its test are named by the review; small diff in state.rs; a seeded review follows)
Lane: cloud, branch claude/stray-grab at 9859071b (r1 head). One round, stop at 250 k. Cost about 0.8 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "stray-grab BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/stray-grab.md "## Review r1" (findings rv-1 to rv-4, with file, line, repro and fix). On the branch: crates/chora/src/state.rs 925-944 (choose_manager), crates/chora/src/render.rs node_under, crates/chora/src/input.rs tests near 2639.
Items:
1. rv-1 MAJOR: a press on an input node (Accept rect, construct HUD chip, door, hint) over another manager's claim takes the keyboard from the focused window. Red unit test first (the review's repro: sh-23 fixture plus an `input: accept` rect of `space` at the click point; the window keeps the keyboard), then the fix: clear focus only when `render::node_under` also finds no input node.
2. rv-2 MINOR: `choose_manager` returns early while the session is locked. Red unit test first (locked, press on an output without a lock surface, unlock: focus_wanted kept), then the fix.
3. rv-4 NIT: a unit test that a click on the empty claim of the look that has the focus keeps the keyboard (guards an existing rule; no red needed).
4. rv-3 NIT: docs/decisions.md:233 and docs/api.md:424 wording as the review says.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7 first, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,keys,pane-keys,wm-pane,chat-keys,doors,construct-hud,matrix`; poll it in the same turn; never end a turn to wait. A red cell you did not cause: rerun it alone once and name it.
Done when: red commits before fixes, gate cells pass, reports/stray-grab.md "## r2" on claude/session-log (per item: red sha, fix sha, test, gate cell).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/stray-grab and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
