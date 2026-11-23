# soak r3 (rc 9; sl-9 MAJOR): the 3 h soak again on the session head, probes ended each round

Model: haiku, effort high (a test-only run and a table; no product code)
Lane: cloud, claude/soak-r3 at 1ec04694 (session head, t117: leak 677790dc merged). Cost at most 1.0 %; stop at 250 k context. One area: tests/battle/soak-core.sh and tests/battle/RESULTS.md; no product code. A fault you find goes into the report as a finding, not a fix.
Why: findings/shared.md sh-33 and reports/sim-long.md "## r2" sl-9 (RSS 273 MB to 2.25 GB in 67 rounds, fds 32 to 823); reports/leak.md "Still open": the soak must end its probes each round (`end_probes`, tests/battle/lib.sh:110) and be measured again.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, then `cargo build --workspace` (debug, as in r2); a failure: send_message the PM "soak-r3 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) tests/battle/soak-core.sh: call `end_probes` at the end of every round (after the probe windows close) and log the open window count per round to the run file. One commit `test(battle): soak ends probes each round (sl-9)`; nothing else changes.
2) Run it once, SOAK_MIN=180, SAMPLE_S=600, detached (`setsid nohup bash tests/battle/soak-core.sh >/dev/null 2>&1 </dev/null &`); poll in the same turn, never end a turn to wait. Clean up with tests/battle/kill-session.sh soak only (never by process name).
3) Table: per process (core, mc, tiling) first vs last hour RSS and fds, linear fit per hour, the window count per round (flat or not), deaths, watchdog disconnects (grep the session log), rounds done.
4) Verdict per bar (design/rc.md row 9, sim-long r2 bar: growth at most 5 % per hour with the window count flat): sl-9 closed or still open with the numbers. Any growth over the bar, death or disconnect: a finding (id sl-10..) with process, log path and line.
5) RESULTS.md section "## soak r3 2026-11-23" with the table; commit it (the raw csv stays out of git).
Gate: `tests/merge-gate.sh --only pre-commit,ci-check` on claude/soak-r3 (test script and RESULTS.md only).
Done when: 3 h measured with numbers and log paths; sl-9 marked closed or open against the bar; new findings filed in the report.
Report: reports/sim-long.md "## soak r3" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha and the sl-9 verdict.
Rules: push claude/soak-r3 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
