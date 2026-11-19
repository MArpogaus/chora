# mx1-m2 r1 (F5): the handover-matrix cap measures the run's own wall time

Model: sonnet, effort high (harness timing, small routine change)
Lane: cloud, branch claude/mx1-m2 from the session head f2b7112a (train 69). One round, stop at 250 k. Cost about 0.3 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "mx1-m2 r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read on claude/session-log: reports/audit-f.md "## r2" (rank 2), reports/matrix-cap.md (r1 and its review, finding m2), backlog.md MX1.
Data (audit r2): both train reruns since 14:40 (trains 65, 66) were the matrix cap; none since matrix-cap b71c5096.
1. tests/gate-handover-matrix.sh: the cap measures wall time from the matrix's own first cell (not time waiting in the gate queue or a lock).
2. The PASS line prints that time and the cap; a cap breach prints the five slowest cells with times.
3. Red-first: a gate-lib cell (or a script self-test) that shows a queued start no longer counts against the cap; commit it red first.
Gate: one `tests/merge-gate.sh --jobs 3` on the head while two other cargo jobs run beside it; the printed matrix time under 315 s and green (start in the background, poll it in the same turn; never end a turn to wait).
Done when: the above holds; reports/matrix-cap.md "## mx1-m2 r1" on claude/session-log with times, head sha; review m2 named closed.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): head sha, printed time, cap, gate.
Rules: push claude/mx1-m2 and claude/session-log only (fetch and merge on reject, no force); author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
