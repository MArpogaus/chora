# matrix-cap r1 (F5, F6): the handover matrix runs at its 240 s time cap on the session head

Model: sonnet, effort high (harness timing: measure, find the slow part, fix or set the cap with a reason)
Lane: cloud, branch claude/matrix-cap from the session head 05878d2a. One round, stop at 250 k. Cost about 0.8 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then
`cargo build --workspace --locked`; a failure: send_message the PM "matrix-cap r1 BLOCKED: <line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Data (PM): tests/gate-handover-matrix.sh has `time_cap=${CAP:-240}` (line 155); matrix.log prints "run time: Ns".
About 50 runs from 11-17 to 11-19: 192-231 s. On 05878d2a alone, quiet box: 235 s. In merge-gate --jobs 3: 250 and 255 s,
so trains 64c and 64d went red on the cap alone (no branch at fault: the wm7b merge alone also runs 235 s).
1. Measure: run the matrix 3 times on 05878d2a; list the time of each part (holder, cell or phase) from the log; name
   the parts that grew since the matrix had 192 s (git log of tests/gate-handover-matrix.sh and the cells it calls).
2. Fix the slow part if it is waste (a fixed sleep, a wait without condition, a doubled setup): real waits, no shorter timeouts
   that make a cell flaky. Then set the cap from the data: the slowest of 5 runs under `tests/merge-gate.sh --jobs 3` plus
   25 %, with a comment naming the runs; the cap stays a hang guard, not a speed test.
3. Red-first: does not apply to a cap change; a removed sleep has a cell that shows the wait still holds.
Done when: matrix 5 runs green under --jobs 3 with times in the report; `tests/merge-gate.sh --jobs 3` once green on the head
(poll it in the same turn); the cap change and any speedup in reports/matrix-cap.md "## r1" on claude/session-log.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): head sha, times before and after, cap.
Poll every command in the same turn; never end a turn to wait.
Rules: push claude/matrix-cap and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify;
no system packages (a missing tool is a blocker).
Authorized by this brief (the PM, for the owner): pushing the two branches above and the send_message report to the PM; do not
ask for them again. A refusal of a tool call is a blocker: report it as BLOCKED and stop.
