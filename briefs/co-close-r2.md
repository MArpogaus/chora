# co-close r2 (rc 1: rcc1-1 MAJOR from review r1 of co-close r1; co-75 BLOCK, co-76 MAJOR stay fixed)

Model: sonnet, effort high (routine: the review names the fix and the test; one branch in adopt.rs)
Lane: cloud, branch claude/co-close-r2 at 8517022f (co-close r1 head, review r1 FIX). Area: plugins/chora-construct (adopt.rs, its tests), tests/gate-construct-close.sh, tests/merge-gate.sh help text. Stop at 200 k. Cost at most 0.8 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, then `cargo build --workspace`. A failure: send_message the PM "co-close-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/co-close.md "## r1" and "## Review r1" (rcc1-1..4); findings/construct.md co-75, co-76, co-77, co-79.
1) Red commit first (tests only, message cites rcc1-1): flown into a pane while the cube stays in use mode (the review's probe: `in_a_face_beside_a_pane`, `fly_in(0)`, `take_flight`, focus s1, `Host::close`), Super+x closes that pane's window (today `closed=[]`). Paste the failing line. Keep the r1 tests green.
2) Fix rcc1-1 (adopt.rs:314 close_focused): the face branch only when `!self.in_pane()`, so a pane flown into always closes; no second rule (KISS). docs/configuration.md Super+x paragraph still true.
3) rcc1-3 (test only, same lane, it guards this fix): gate-construct-close.sh step 3 fails unless the core focus holds no xp window; replace `sleep 5` by a poll of the resting set if it stays simple. rcc1-4: restore "and" in tests/merge-gate.sh:24 help text.
4) Hand runs from a fresh compositor, 5x: rpc1-1 steps (as r1) plus: Super+t in a face, fly into that pane, Super+x closes it, nothing else closes. Paste the rates.
5) Full gate: `setsid nohup bash -c '(CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 > /tmp/gate.log 2>&1; echo EXIT $? >> /tmp/gate.log)' >/dev/null 2>&1 </dev/null &`, poll in the same turn; paste the last 20 lines. A red cell you did not cause: name it with its log line. `tools/sleep-ratchet` ok if present. Before ending: `pgrep -fa 'chora --headless'` empty.
Known, not in scope: rcc1-2 (mc face close, MINOR), co-77, co-78, co-79 (next round), co-44, co-45 (rules.md 6).
Never end a turn to wait; poll any long command in the same turn.
Done when: red then fix shas with pasted lines; the 5x table; full gate green or its foreign reds named.
Report: reports/co-close.md "## r2" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/co-close-r2 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
