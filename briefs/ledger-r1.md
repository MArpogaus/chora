# ledger r1 (rc 1): verify every open BLOCK and MAJOR on the new head

Lane: opus, cloud, read-only on code; writes only the findings ledger on claude/session-log.
One round. Stop at 250 k context and hand over. Read dev-rules.md and conventions.md first.
Setup: clone MArpogaus/chora to /home/user/chora, fetch all branches, worktree /home/user/wt/log on
claude/session-log, `bash /home/user/wt/log/setup.sh`; build the session branch head d349a6da in
its own worktree.
Task: for every line in findings/*.md with BLOCK or MAJOR that is not fixed/closed, decide on
d349a6da: fixed (name the commit or merge and the gate cell or capture that proves it), still open
(reproduce: the command, the cell or a capture at 1x, and what you saw), or not reproducible (what
you tried). Many were addressed by trains 5-8 (mc r1-r5, doors r1-r4, xtc, core r1-r3, flakes r2-r3);
check, do not assume. ti-1 stays open until fuzzel 1.11 (backlog PK1). Run real sessions with
tests/gate-lib.sh helpers or the look's gate scripts; captures stay in the worker (list paths).
Output: update the state column of each line in findings/*.md (fixed <sha> <cell> / open <evidence>
/ not reproduced <how>), and write reports/ledger-r1.md: counts per look before and after, the
list still open with evidence, and new BLOCK/MAJOR you met (new lines in the ledger, next free id).
Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines).
Rules: push only claude/session-log (fetch and rebase on reject, no force), only findings/ and that
report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI attribution
lines; never --no-verify; never end a turn to wait for a long background job.
