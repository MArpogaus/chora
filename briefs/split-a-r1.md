# split-a r1 (F4 headroom, plans/week-3.md row 50)

Model: sonnet, effort medium (pure moves of code into new modules; no behaviour change)
Lane: cloud, claude/split-a at 892d035c (session head after train 101). Cost at most 1.6 %; stop at 250 k context. One area: plugins/chora-mc/src/main.rs and plugins/chora-assistant/src/chat.rs and the new modules split from them; no other file except mod lines, tools/size-ratchet.list and the lint-ratchet lists if a count moves.
Why: F4 is met with little headroom: mc main.rs about 1,430 and assistant chat.rs about 1,419 production lines, close to the 1,500 limit (tools/size-ratchet, CONTRIBUTING.md:358). A fix in either file can push it over.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "split-a BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) mc main.rs: move one or two cohesive groups (by topic, with their tests) into new modules so main.rs has at most 1,200 production lines. Pure moves: no renamed item, no changed body, visibility only as wide as the move needs (pub(crate) at most). One commit per file.
2) assistant chat.rs: the same, at most 1,200 production lines.
3) `tools/size-ratchet --update` if a listed file shrank; no file over 1,500 added; cut-check and lint-ratchet counts do not grow.
Proof of a pure move: `git diff --color-moved=zebra --stat` shows moved blocks only; in the report name each moved group (from lines, to file) and the production line counts before and after.
Gate: full `tests/merge-gate.sh --jobs 1` (no --only) green on claude/split-a merged with origin/claude/autonomous-dev-session-3w-i33pg7. A red cell you did not cause: name it with its log line; do not rerun to green. Run it in the background and poll in the same turn; never end a turn to wait.
Done when: both files at most 1,200 production lines by pure moves, cargo test -p chora-mc and -p chora-assistant green, the ratchet list lowered, the full gate green.
Report: reports/split-a.md "## r1" on claude/session-log (shas, moved groups, counts before and after, gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 4 lines with the head sha.
Rules: push claude/split-a and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
