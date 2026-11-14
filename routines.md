# Routines of the PM (restored 11-14 from the archive audit; injected with the card)

Time: `date -u`; logs in UTC; owner-facing times (chat, status page) also in Bangkok.

## By time
- Every 2 h: watchdog (tools/triggers.md); heartbeat checks the PM from outside.
- Daily 12:50 UTC: self-check (rules.md) plus one independent opus critic, read-only, on
  the last 24 h of state.md, answers.md and commits; the PM answers each point in
  reflections/selfcheck-<date>.md the same day (owner 11-09, charter 14).
- Friday: reports/week-N.md, critical, no overselling (owner 11-09 "Don't overstate
  anything"): done, missed, rc burn-down, budget, owner decisions taken by the PM.
- Weekly (Friday): `review_due.py` for split reviews (5,000 lines, again after 50 %
  growth) and the extraction check (code to move into chora-kit or a small plugin,
  duplication across plugins); results as lines in state.md or backlog items.

## By event
- A usage limit stops work: send_later at the reset time to resume every lane; before
  the stop merge first, push everything, rewrite state.md (rules.md 8).
- create_session: start check after 6 minutes (start_lane.sh); still PENDING after 15
  minutes: check the environment's setup (friction 41, 45).
- A lane hands back: review, merge or next round; archive the worker (if the permission
  check refuses, leave it idle and note it).
- A merge: status page (Running, Next, Needs you match state.md; reviewed captures that
  name their build; owner 11-10, 11-12 01:26); remove the lane worktree and its target/.
- A persona round ends: `triage.py` on feedback/, findings ranked by severity.
- A rule broken twice: a friction entry and a guard before the next lane start.
- An owner message: classify (rules.md 2), record verbatim the same turn.
- An architecture record (docs/decisions.md) changes: list it in the week report, the
  owner may veto (charter: A1-A22 were his; 11-14 17:17 delegated).

## Session end (11-30)
session-finish skill; HANDOFF.md on claude/session-log with the owner's tasks: hardware
checks (3D bench, gate-0.9), delete trigger trig_012VGJoVzsJ81hFa6AufBc9S and the PM
triggers, open owner questions (backlog M1), merge state of dev and main.
