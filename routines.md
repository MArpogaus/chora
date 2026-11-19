# Routines of the PM

Time: `date -u`; logs in UTC; owner-facing times (chat, status page) also in Bangkok.

## By time
- Budget (owner 11-15, "your main job"): week resets at the owner's reset; check the pace line at each lane start; model and effort in create_session.
- Every 2 h: watchdog (tools/triggers.md), prunes idle children (11-17); heartbeat checks the PM.
- Daily 12:50 UTC (19:50 Bangkok), workday close (owner 11-15 09:35): self-check and main-duty
  questions, one independent opus critic, every open friction resolved, one workflow change;
  reflections/selfcheck-<date>.md.
- Friday: reports/week-N.md (done, missed, burn-down, budget, PM decisions; no overselling);
  `review_due.py`; extraction check (chora-kit, duplication) to state.md or backlog.
- Refactor proposals (owner 11-17): ranked in backlog.md per handover; a release takes
  the top ones; debt-<tag>.md counts never grow; a sprint starts with a plan.

## By event
- Each handback, merge, train (11-19): realign Queue 4-5 (rank, F moved, unmerged <= 4, budget), same turn.
- Train (owner 11-19 07:01): all MERGE-OK branches in one train; red: bisect; --jobs 3 after flakes-f 10/10.
  `df -h /`; under 20 G one build at a time, clear sccache (friction 53).
- A harness branch is MERGE-OK: its own train at once, before other trains; then tell the running
  lanes to merge the session branch. A new hook or check shows it never blocks a valid commit.
- A FIX verdict or a CI-only failure: name the earliest gate that could have caught it; if none
  exists, a harness backlog item (owner 11-15 02:10: strong compile-time gates).
- A usage limit stops work: send_later at the reset time to resume every lane; before
  the stop merge first, push everything, rewrite state.md (rules.md 8).
- create_session: start check after 6 minutes (start_lane.sh); still PENDING after 15
  minutes: check the environment's setup (friction 41, 45).
- A lane hands back: review, merge or next round; archive the worker (if the permission
  check refuses, leave it idle and note it).
- A merge: status page matches state.md (reviewed captures name their build); remove the
  lane worktree and its target/.
- A rule broken twice: a friction entry and a guard before the next lane start.
- Tour review: fresh-eyes pass per look first; the PM watches tour-all before a verdict.
- Budget ledger (11-17): Queue lines and briefs name % cost; open lines <= % left - 5 %.
- An owner message: classify (rules.md 2), record verbatim, same turn.
- An architecture record (docs/decisions.md) changes: list it in the week report, the
  owner may veto (charter: A1-A22 were his; 11-14 17:17 delegated).

## Session end (11-30)
session-finish skill; HANDOFF.md on claude/session-log with the owner's tasks: hardware
checks (3D bench, gate-0.9), delete trigger trig_012VGJoVzsJ81hFa6AufBc9S and the PM
triggers, open owner questions (backlog M1), merge state of dev and main.
