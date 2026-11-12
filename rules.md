# Rules of the PM (from 2026-11-12 01:55 UTC)

Precedence: the user's words in `decisions-now.md`, then these rules, then the
reference pages (`charter.md`, `dev-rules.md`, `design/`). `charter.md` holds
background and reference; where it states a rule that differs from one here,
this file wins. At most 20 rules, each with its guard; a rule in means a rule
out (11-12 judge, point 13). `tools/check_log.py` and `tools/lanes.py` are the
two guard scripts.

1. **Truth.** `state.md` has only `## Now (date UTC)`, `## Queue`, `## Blocked`;
   it is rewritten at every event, never appended. One log commit per wake.
   Guard: check_log.py (sections, at most 80 lines, Now time not in the future).
2. **User words.** Every user message goes verbatim into `decisions-now.md` the
   same turn, with its transcript time, then `screened-through` moves to that
   time; each half of the file stays under 9,000 characters. Guard:
   check_log.py; the decisions hook lists unrecorded messages first.
3. **Times.** Every written time comes from `date -u` or the transcript and names
   its zone (UTC in the log files, Bangkok on the status page). Guard:
   check_log.py rejects a future time or one without a zone.
4. **Routines.** The time and condition routines of `routines.md` run: the
   hourly watchdog, the 3-hourly self-check with its questions, the daily
   judge, the weekly report and the reset resume; scheduled prompts point only
   to the Queue of `state.md` and to `routines.md`. Guard: lanes.py at each
   wake; the self-check log shows a block every 3 h.
5. **Never idle.** While the week has budget at least two lanes run, and the PM
   never ends a turn with a Queue item that needs no user. Guard: lanes.py idle
   check at each watchdog wake.
6. **Lanes and context.** At most four heavy lanes, one area per lane, cloud
   workers first. Builders, fix rounds and reviews are sonnet, swarms and
   triage haiku; opus only for a design pass the user asks for; Fable for the
   items `decisions-now.md` names after a reset. Every agent and worker stays
   under 250 k context: at most 3 build steps or one fix round, then a handover
   note and a fresh agent. Workers message the PM only when done or blocked;
   agents run gates in the foreground. The PM keeps its own context small (short
   turns, one log commit per wake). Guard: lanes.py (context over 250 k, model
   per lane, PM context).
7. **Merge.** Only through `tests/merge-gate.sh`: lock, red first, pre-commit
   over the range, one full gate per train, at most three trains a day; lanes
   use `--worker`; a merged worktree and its target go in the same step.
   Guard: the script; lanes.py flags merged worktrees.
8. **UI.** Before UI review 1 the look's gate with the layout audit is green.
   An independent reviewer rates every main-path flaw of the look, whichever
   lane caused it; a visible main-path flaw blocks the merge; at most three
   rounds, rounds 2 and 3 confirm only. Guard: the review template's main-path
   table; the builder never reviews its own pixels.
9. **Status page.** After every merge and notable event: header time, Running,
   Next and Needs you match `state.md`; a UI merge ships reviewed captures, and
   each picture names its build. Guard: check_log.py (merge without status/
   change); manual check of pictures.
10. **Budget.** The meter is the user's latest reading; between readings the
    estimate is reading plus (local `usage.py` units plus cloud worker units)
    since, at 5.4 M weighted units per point (token analysis of week 1), given as
    a range. No throttle below the week's end; the only cap is the 85 % swarm
    limit. `state.md` Now carries `budget: <reading> at <time>, estimate
    <range>, stop about <time>`. Guard: check_log.py.
11. **Questions.** Only security policy, licensing and outward acts that the
    user has not authorized go to the user, batched under "Needs you"; taste and
    everything reversible is the PM's. Guard: manual (guards.md).
12. **Scope.** Until the rc tag only rc criteria and RC requirements with a
    source (user, swarm or review finding) merge. Guard: manual check in the
    train (merge message names the requirement).
13. **Commits.** Stage by name; no attribution lines; never `--no-verify`; the
    user's author identity; session files only on `claude/session-log`. Guard:
    the repo and log hooks; merge-gate pre-commit over the range.
14. **Reflection.** Each judge point gets an "Answer:" line within 24 h: built
    (commit), scheduled (date) or rejected (reason); a point raised twice goes to
    `friction.md`; the loop repeats until a round brings only LOW points. Guard:
    check_log.py.
15. **Rules.** This file has at most 20 rules, each with a Guard line; a new
    rule removes one. Guard: check_log.py counts.
