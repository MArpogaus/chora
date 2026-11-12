# Routines of the PM (from 2026-11-12, user 02:09 UTC)

Reference page of rules.md 4. Each routine names its trigger and what it does.
Times are UTC.

## Time routines (scheduled tasks)

| Routine | Trigger | Does |
|---|---|---|
| Watchdog | every 2 h at :15 (trig_01VxAcAFayNMtGiWDVuzJuRz); one-line reply when nothing changed | lanes.py; cloud workers via get_session; act on finished or stuck lanes; keep two or more lanes running; refresh state.md Now |
| Self-check | 00:50, 06:50, 12:50, 18:50 UTC (trig_01X5NqMSsvxMSpYFJxo5Wxsg) | answer the questions below in `reflections/selfcheck-DATE.md`, one block per check, each answer with the action taken or "none, because ..." |
| Daily judge | the 18:50 UTC self-check (01:50 Bkk) | fresh opus judge (charter 11a), answers within 24 h (rules.md 14) |
| Weekly report | the 12:50 UTC self-check on a Bangkok Friday | `reports/week-N.md` from the log files |
| Reset resume | Sat 04:05 UTC (trig_01Cbw7Sgz7WRVpTnjsN1zdwq) | work the Queue of state.md |

Self-check questions (every 6 h; be self-critical; a worry gets its action now). The
mechanical checks (lanes, disk, worktrees, context, leftover sessions) are `lanes.py`: run it
first and act on its flags; the questions cover what a script cannot see.

1. On track and aligned? Compare the Queue and the rc dates with what merged since the last
   check; does every running lane serve a user decision or an rc criterion?
2. New data needed before a decision? (a swarm on a newly merged look, a capture, a measurement)
3. Tests due? (a gate not run since the last merge, a flaky test seen twice)
4. Quality and honesty: did anything merge with a known main-path flaw or without red first;
   does the status page or state.md claim more than the evidence shows?
5. Risks in the next 12 h (limit stop, permission refusal, stuck worker, flaky gate), and is the
   fallback ready? Before an expected weekly stop: run the pre-stop routine.
6. Waiting on me: an unread report or verdict, a lane silent for 2 h, the user's last chat
   question not answered?
7. Cost: the PM's own spend since the last check (usage.py for the PM transcript), agents above
   250 k context, reviews that could be confirm-only; is the wake cadence still worth it?
8. User value: what would the user try first now, and is it in its best shape?
9. Challenge: did I follow a user suggestion or an agent report without checking it; where do I
   disagree and did I say so?
10. Faults and rules: what did I get wrong since the last check, which rule or guard stops it;
    did a friction pattern repeat; another reflection round needed?

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
| Pre-stop (estimate at 97 % or more, or a usage warning) | write state.md Now and Queue for the reset, disable the watchdog and the self-check (the reset resume re-enables them), push everything |
| After a compaction | the hooks inject card, state and decisions; read rules.md; resume the Queue at once |
| Before an outward or destructive act (delete, push --delete, tag, PR, publish) | check the user's authorization and the permission scope first; do it as one small step; on refusal, log it under Blocked with the exact command for the user |
| A lane silent for 2 h (no push, no report) | get_session or the agent's last output; nudge once, then replace it with a fresh agent and a handover |
| Before briefing a worker | the brief template of design/product-briefs.md: goal, brief section, design note, area, acceptance checks, gates incl. `cargo test --workspace`, do-nots, report form |
- After create_session: a send_later check after 6 minutes that the worker left PENDING; if not, archive it and run the task as a local agent (friction 41).
