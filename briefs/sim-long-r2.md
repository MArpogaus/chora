# sim-long r2 (rc 9 recheck: leaks and core death after core-fix and mesh-churn merged)

Model: haiku, effort high (a test-only run and a table; no product code)
Lane: cloud, claude/sim-long-r2 at 230ea0e5 (session head, t113; core-fix 207fe5ad and mesh-churn 356e22ef merged). Cost at most 1.0 %; stop at 250 k context. One area: tests/battle (scripts and RESULTS.md); no product code. A fault you find goes into the report as a finding, not a fix.
Why: design/rc.md row 9 (no leak, no stall, no window loss) is open on sl-1 (W9 core RSS +236 %), sl-3 (core abort 18/18) and sl-2 (W11 +73 %); the fixes are merged and unmeasured (reports/review-mesh-churn.md, reports/core-fix.md on claude/session-log).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, then `cargo build --workspace` (debug, as in r1); a failure: send_message the PM "sim-long-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/sim-long.md (r1 method, sl-1..sl-8), reports/review-mesh-churn.md (the last section: what the recheck must measure), reports/core-fix.md.
1) Repros, each 5 times: tests/battle/repro-sl3.sh (pass: no core abort, 5/5) and tests/battle/repro-sl1.sh (pass: RSS after the last close at most start + 60 MB; record the anon split and the peak with 12 windows, expected at most about 266 MB).
2) Full W9 end RSS 3 times against core-diag's 288 MB; W11 3 times against r1's 243 MB (sl-2).
3) Frame time p99 during a relayout and a resize drag (the stat the battle scripts or choractl give; name the source); r1 or core-diag values as the baseline where they exist, else record only.
4) One core kept alive for 3 h (sl-7): loop W9, W10, W11, W12, W16 against the same core, detached (`setsid nohup bash -c '...' >/dev/null 2>&1 </dev/null &`), poll in the same turn, never end a turn to wait; every 10 minutes sample RSS (kB) and fds of the core and each plugin by pid into /tmp/sim-long-r2/samples.csv. Clean up with kill-session.sh only (never by process name).
5) RESULTS.md section "## sim-long r2 2026-11-23": repro table, W9/W11 ends, p99, 3 h table (first vs last hour, linear fit per process), deaths, stalls. Commit it (the raw csv stays out of git).
Gate: `tests/merge-gate.sh --only pre-commit,ci-check` on claude/sim-long-r2 (only RESULTS.md changes).
Done when: items 1-4 measured with numbers and log paths; each of sl-1, sl-2, sl-3 marked closed or still open against its bar; every growth over 5 % per hour, any stall or death a finding (id sl-9..), with look, cell and log line.
Report: reports/sim-long.md "## r2" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/sim-long-r2 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
