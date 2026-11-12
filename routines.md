# Routines of the PM (from 2026-11-12, user 02:09 UTC)

Reference page of rules.md 4. Each routine names its trigger and what it does.
Times are UTC.

## Time routines (scheduled tasks)

| Routine | Trigger | Does |
|---|---|---|
| Watchdog | hourly at :35 (trig_01VxAcAFayNMtGiWDVuzJuRz) | lanes.py; cloud workers via get_session; act on finished or stuck lanes; keep two or more lanes running; refresh state.md Now |
| Self-check | every 3 h at :50 (see state.md Now for the id) | answer the questions below in `reflections/selfcheck-DATE.md`, one block per check, each answer with the action taken or "none, because ..." |
| Daily judge | 16:50 UTC (23:50 Bkk) via the self-check of that hour | fresh opus judge (charter 11a), answers within 24 h (rules.md 14) |
| Weekly report | Friday's last self-check | `reports/week-N.md` from the log files |
| Reset resume | Sat 04:05 UTC (trig_01Cbw7Sgz7WRVpTnjsN1zdwq) | work the Queue of state.md |

Self-check questions (be self-critical; a "yes" needs an action now):
1. On track? Compare the Queue and the rc dates with what merged since the last check.
2. Aligned? Read decisions-now.md: does each running lane serve a user decision or an rc criterion?
3. New data needed before a decision? (a swarm, a capture, a measurement, a user reading)
4. Cleanup due? (merged worktrees, targets, leftover chora sessions, stale sessions, disk under 8 GB, stale status page)
5. Tests due? (a gate not run since the last merge, a flaky test seen twice, a swarm on a newly merged look)
6. Efficient? (agents above 250 k context, idle lanes, duplicated work, PM context above 400 k, reviews that could be confirm-only)
7. Another reflection round? (a judge point unanswered, or the last round had findings above LOW)
8. What did I get wrong since the last check, and which rule or guard stops it next time? (goes to friction.md)

## Condition routines

| Condition | Routine |
|---|---|
| A user message | verbatim into decisions-now.md and answers.md the same turn; screened-through to its transcript time |
| A lane reports done | review per rules.md 8 (UI) or a code review (sonnet), then a fix round or the merge train |
| A merge on the session branch | state.md Now, status page (captures marked reviewed), remove the worktree and its target, swarm on a newly merged look |
| A swarm finishes | triage the same hour; new data yes: fix lane; no: stop swarms |
| A review says FIX | a fresh fix agent with the finding list; confirm-only re-review |
| Disk under 8 GB | remove merged targets and worktrees before any build |
| PM context above 400 k | keep turns short; the hooks make compaction safe |
| A usage reading | answers.md, state.md budget line, recalibrate usage.py |
| A rule broken twice | friction entry and a guard before the next lane starts |
| A 5-hour or weekly limit stop | resume task exists for the reset time |
