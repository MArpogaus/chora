# testwatch r1 (rc 3, 4): cheap continuous tests after every merge

Model: haiku, effort low (runs tests and files results, writes no product code)
Lane: cloud, read-only on code, works on the session branch claude/autonomous-dev-session-3w-i33pg7.
One round of at most 8 hours or 250 k context, whichever comes first.
Read dev-rules.md, conventions.md, backlog.md (F17, F24, F26 lines) once.

Loop: every 20 minutes `git fetch origin`; when the session branch head changed (or at the
start), check it out detached and run, in this order:
1. `tests/merge-gate.sh --jobs 3` (no --push).
2. `tests/agent.sh` five times (the CI e2e step), counting passes.
Append one line per head to reports/testwatch-r1.md on claude/session-log: UTC time, head sha,
gate PASS or the failing step and cell, agent.sh n of 5, log file names. Push it after each head.
On any failure: copy the failing step log to logs/testwatch/<sha>-<step>.log on claude/session-log,
send the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN one send_message (at most 4 lines: sha, step,
cell, first error line), then continue the loop. Never fix code, never retry a failing cell to
make it green, never change a test. Between heads, while nothing changed: run the doors step
(`tests/merge-gate.sh --only doors`) once per loop and count it (F26 rate).
At the end: a summary block in the report (heads tested, failures per cell, F26 rate) and one
send_message to the PM. Push only claude/session-log, author Marcel Arpogaus
<38564291+MArpogaus@users.noreply.github.com>, no attribution lines, never --no-verify.
Never end a turn to wait: poll with sleep in the loop.
