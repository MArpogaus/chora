# sim-long r1 (plan slice 23, SIM-LONG: leaks and stalls over hours)

Model: haiku, effort high (a test-only run and a table; no product code)
Lane: cloud, claude/sim-long at 7a6c0eb8 (session head). Cost at most 1.0 %; stop at 250 k context. One area: tests/battle (scripts and RESULTS.md); no product code. A fault you find goes into the report as a finding, not a fix.
Why: plans/week-3.md slice 23 on claude/session-log; design/rc.md row 9 (no known window loss, no stall) and the vision card (stability over new features). No run of several hours exists on this week's head.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, then `cargo build --workspace`; a failure: send_message the PM "sim-long BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Read tests/battle/README.md. Run the plugin days W9, W10, W11, W12 and W16 in a loop for at least 3 hours of wall time, one workflow at a time, each with its own RUN log and SHOTS dir under /tmp/sim-long; detach the loop (`setsid nohup bash -c '...' >/dev/null 2>&1 </dev/null &`) and poll it in the same turn; never end a turn to wait. Clean up with kill-session.sh only (never by process name).
2) Every 10 minutes sample RSS (kB) and open fds of the core and each plugin process by pid (/proc/<pid>/status VmRSS, ls /proc/<pid>/fd | wc -l) into /tmp/sim-long/samples.csv (time, workflow, process, rss_kb, fds).
3) A stall: a workflow step that exceeds the watchdog bound named in the README or its script; a death: the core or a plugin exits. Record each with its log lines.
4) Write a section "## sim-long 2026-11-22" at the end of tests/battle/RESULTS.md: hours run, passes per workflow, growth per hour of RSS and fds per process (first vs last hour, linear fit), deaths, stalls, findings. Commit it with samples.csv summarised (not the raw csv; keep the csv out of git).
Gate: `tests/merge-gate.sh --only pre-commit` on claude/sim-long (only RESULTS.md changes; no product code, so no full gate).
Done when: at least 3 hours run; the RESULTS.md section with the table; every growth over 5 % per hour or any stall or death named as a finding (look, cell, log line).
Report: reports/sim-long.md "## r1" on claude/session-log (hours, table summary, findings with ids sl-1..); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/sim-long and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
