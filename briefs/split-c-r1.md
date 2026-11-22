# split-c r1 (F4 headroom: compositor core)

Model: sonnet, effort high (pure moves in the compositor core; no behaviour change; core needs care with visibility)
Lane: cloud, claude/split-c at 3b14c825 (session head after train 107). Cost at most 1.6 %; stop at 250 k context. One area: crates/chora/src/ipc.rs and crates/chora/src/mesh.rs and the new modules split from them; no other file except mod lines, tools/size-ratchet.list and the lint-ratchet lists if a count moves.
Why: F4 is met with little headroom: core ipc.rs 1,398 and mesh.rs 1,387 production lines, close to the 1,500 limit (tools/size-ratchet, CONTRIBUTING.md:358); sim-long sl-3 (W12 core dies 8/8) fixes will land in the core.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "split-c BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) core ipc.rs: move one or two cohesive groups (by topic, with their tests) into new modules so ipc.rs has at most 1,200 production lines. Pure moves: no renamed item, no changed body, visibility only as wide as the move needs (pub(crate) at most). One commit per file.
2) core mesh.rs: the same, at most 1,200 production lines.
3) `tools/size-ratchet` stays ok (no file over 1,500); cut-check and lint-ratchet counts do not grow.
Proof of a pure move: `git diff --color-moved=zebra --stat` shows moved blocks only; in the report name each moved group (from lines, to file) and the production line counts before and after.
Gate: full `tests/merge-gate.sh --jobs 1` (no --only) green on claude/split-c merged with origin/claude/autonomous-dev-session-3w-i33pg7. A red cell you did not cause: name it with its log line; do not rerun to green. Run it in the background and poll in the same turn; never end a turn to wait.
Done when: both files at most 1,200 production lines by pure moves, cargo test -p chora green, cut-check and lint-ratchet counts not grown, the full gate green.
Report: reports/split-c.md "## r1" on claude/session-log (shas, moved groups, counts before and after, gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 4 lines with the head sha.
Rules: push claude/split-c and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare `git stash` (WIP commit instead; log-down lc-3).
