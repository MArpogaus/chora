# mc-round r2 (rc 1, 3, 9): resolve the MH-2 clash with a3037cb2 on the session head

Model: sonnet, effort low (one semantic merge conflict in one plugin, the decision is given)
Lane: cloud, claude/mc-round at 9ed1fbc8 (r1 head, MERGE-OK). Cost at most 0.4 %; stop at 250 k context. One area: plugins/chora-mc and its gate cells.
Why: train t112 (03:10 UTC) red in `test`: plugins/chora-mc/src/main.rs:1693 `the_doors_leave_while_the_chat_is_open` left 5 right 0. a3037cb2 (on the session head since 11-22, I6) hides the doors while the chat is open (region/items.rs `door_items` returns nothing on `Chat::Open`); mc-round MH-2 (bf23d49d, 97efc46a, reviewed) keeps them, the chat box ends one baseline above the foot. 9ed1fbc8 lacks a3037cb2, so neither gate saw both.
Decision (PM): MH-2 holds (the doors are the mouse path of the rc bar). Keep a3037cb2's `question_pending` and its chat head rule.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "mc-round BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Merge fresh origin/claude/autonomous-dev-session-3w-i33pg7 (86e619f3 or newer) into claude/mc-round. Red: `cargo test -p chora-mc` shows the clash; paste the line.
2) Resolve: `door_items` keeps the doors under MH-2's box geometry; the doc comment of `door_items` says so; `the_doors_leave_while_the_chat_is_open` goes (superseded by MH-2's test; name it in the commit body). Check a3037cb2's I6 rows (tests/acts/known-red.toml removals) still hold with the doors shown: the acts mc rows green.
3) `cargo test -p chora-mc`, gate-mc-column.sh 5/5 light and dark.
Gate: full `tests/merge-gate.sh --jobs 2` (no --only), detached: `setsid nohup bash -c '(tests/merge-gate.sh --jobs 2 > /tmp/gate.log 2>&1; echo EXIT $? >> /tmp/gate.log)' >/dev/null 2>&1 </dev/null &`, poll /tmp/gate.log in the same turn; never end a turn to wait. A red cell you did not cause: name it with its log line; do not rerun to green. Before ending: `pgrep -fa 'chora --headless'` empty.
Done when: the merge and resolve commits, the unit and cell counts, the gate green or its foreign reds named.
Report: reports/mc-round.md "## r2" on claude/session-log (shas, the red line, counts, gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 4 lines with the head sha.
Rules: push claude/mc-round and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
