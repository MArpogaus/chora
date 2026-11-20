# stray-grab-r1 (RC shared bar "no stray keys"; F-none): sh-24 and the focus half of sh-23

Model: opus, effort high, because the fix is in the core input routing (crates/chora input.rs grabs, pointer vs keyboard focus), where a wrong rule leaks keys into clients or strands keyboard-only users
Lane: cloud, branch claude/stray-grab from bb16f2ff (session branch, train 79). One round, stop at 250 k. Cost about 1.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "stray-grab BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/shared.md rows sh-23, sh-24; reports/multilook-r1.md or the review it names (review-multilook-r1, items 1-3); vision-card.md "Shared RC bar". On the branch: crates/chora/src/input.rs around 433 and 466-471, docs/decisions.md on input and focus.
Items:
1. sh-24: a grab gated by the pointer position falls through to the focused client as a raw key; keyboard-only use follows a stale pointer; injected keys are gated too. Red test first (a unit or gate test that sends a grabbed chord with the pointer over another look's pane and sees the key reach no client), then the fix: a grab is decided by the keyboard focus's look, not by the pointer; injected keys follow the same rule.
2. sh-23 focus half: in two panes (xp and tiling), a click into the tiling pane moves keyboard focus there, so Super+Return goes to tiling, not to the xp Start menu. Red test first, then fix. The black tiling pane (render half of sh-23, sd-h1) is another lane's: do not touch plugins/chora-tiling drawing.
3. Docs: the rule in docs (where the input docs say who gets a key) and a decision line if the rule changes a record.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7 first, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,keys,pane-keys,wm-pane,chat-keys,doors,matrix,layout`; poll it in the same turn; never end a turn to wait. A red cell you did not cause: rerun it alone once and name it.
Done when: red commits before fixes, gate cells pass, reports/stray-grab.md "## r1" on claude/session-log (per item: red sha, fix sha, test, gate cell).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/stray-grab and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
