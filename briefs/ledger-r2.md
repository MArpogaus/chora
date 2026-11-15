# ledger r2 (rc 1): verify every open BLOCK and MAJOR on be205f89 (train 13)

Lane: opus, cloud, read-only on code; writes only the findings ledger on claude/session-log.
One round. Stop at 250 k context and hand over. Read dev-rules.md and conventions.md first.
Setup: clone MArpogaus/chora to /home/user/chora, fetch all branches, worktree /home/user/wt/log on
claude/session-log, `bash /home/user/wt/log/setup.sh`; build the session branch head be205f89 in
its own worktree.
Task: for every line in findings/*.md with BLOCK or MAJOR that is not fixed/closed, decide on
be205f89: fixed (name the commit or merge and the gate cell or capture that proves it), still open
(reproduce: the command, the cell or a capture at 1x, and what you saw), or not reproducible (what
you tried). Start from reports/ledger-r1.md (7 open: ti-1, ti-4, mc-30, sh-5, sh-7, sh-10, sh-11); trains 9-13
(ux r1, ux r2, tiling focus fade, flakes r4) addressed several; check, do not assume. The 25 fixed ones
stand on the passing train 13 merge gate; do not rerun it. ti-1: check `fuzzel --version`; if
>= 1.11 click it, else it stays open (PK1). ti-4: mako is installed now; run gate-tiling-notice. Run real sessions with
tests/gate-lib.sh helpers or the look's gate scripts; captures stay in the worker (list paths).
Output: update the state column of each line in findings/*.md (fixed <sha> <cell> / open <evidence>
/ not reproduced <how>), and write reports/ledger-r2.md: counts per look before and after, the
list still open with evidence, and new BLOCK/MAJOR you met (new lines in the ledger, next free id).
Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines).
Rules: push only claude/session-log (fetch and rebase on reject, no force), only findings/ and that
report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI attribution
lines; never --no-verify; never end a turn to wait for a long background job.
