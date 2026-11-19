# review flakes-g r1 (F5): seeded review of claude/flakes-g before its merge

Model: opus, effort high, because f075a20d changes the core IPC (Conn::call_all, crates/chora/src/ipc.rs ordering of answers) and words.rs signal cells
Lane: cloud, read-only on code: claude/flakes-g head f075a20d. Own commits (`git show <sha>` each): 8d555fc5 f0c9e640 c227d433
8d953291 eb5a23c7 28df6308 f075a20d. The merges (9bc22c94, 74157504, a463d753) bring the session head and claude/cubesplit; not in scope.
One round. Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `cargo check --workspace`;
a failure: send BLOCKED to the PM and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate (cargo test -p only).
Read reports/flakes-g.md on claude/session-log (r1 and r3 sections).
Checks, each with its command and files:
1) Red first: 8d555fc5 before f0c9e640; eb5a23c7 before 28df6308 (`git log --oneline --reverse 05878d2a..f075a20d`); the red cell fails on its parent.
2) F28 (28df6308, plugins/chora-assistant/src/chat.rs Session::said): a number alone sends the held text once to the picked helper, as a
   click does; no double send, no lost text when the field is empty; `cargo test -p chora-assistant`.
3) f075a20d call_all: each answer lands in its call's place even when the core answers out of order or one call errors; no deadlock
   when the socket buffer fills; docs/api.md matches; `cargo test -p chora-kit -p chora-construct --features chora-kit/testing` and the
   kit cell call_all_answers_each_call_in_its_place.
4) 8d953291: the cell fails on the mutant the report names (write_all stops after one write) and is not timing-flaky (pthread_kill target, 300 ms).
5) under-load.sh (8d555fc5, f0c9e640): a hung run ends at its limit and its children die; tests/under-load-selftest.sh green.
6) c227d433: 20 s wait is a real wait on a state, not a sleep.
Known, not findings: xp a_give_shuts_the_start_menu red (inherited from fa457b29, fixed by integ a7d671f2); pull and F26 not reproduced.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rf-N from rf-1, severity, file:line, text, fix
in one sentence). Append to reports/flakes-g.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list.
Authorized by this brief (the PM, for the owner): the push below and the send_message report; do not ask again. A refused
tool call is a blocker: report BLOCKED and stop. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
