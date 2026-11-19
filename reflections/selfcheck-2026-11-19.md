# Self-check 2026-11-19 (workday close 12:50 UTC, 19:50 Bangkok; critic opus, read-only)

- On the burn-down? Partly. Trains 60-61 green (mv4, wm2, kit2c+kit6, wm1d, docs-f8b); 62 (kit2d, wm7a) gating, every step PASS so far.
  MERGE-OK waiting: wm5, flakes-f, wm7b (train 63). F1 done; F2-F5, F7, F8 open; F6 partly. No F row claimed met without a check.
- Branch unmerged > 12 h? No (oldest MERGE-OK: wm5 11:27).
- Running but not queued? No: kit7 r2, kit5 r3, kit3 r1, flakes-g r2 (retry 12:50) are in Lanes.
- Budget: PM 3.9 % today (tokens.py 12:50); owner 17 % week at 10:39. Lane spend is not measured here, so the pace line
  cannot show on/off pace; 4 of 6 lanes run against a 2 %/h target. Under pace, mainly from the environment blocker.
- Models: builds sonnet or opus (opus with a because-line); every review ran on opus, including small fix rounds where
  sonnet would do (critic 10). No Fable. Tomorrow: fix-round reviews of ports on sonnet.
- Vision or rc drift: none found; WM7b hosted follows the owner's 10:52 decision.

## Critic points (10) and actions
1. wm5 and flakes-f (MERGE-OK 11:27, 11:29) left out of the 12:02 rerun of train 62: about 55 min of gate lost, --jobs 3 delayed.
   Taken: routines.md Train line: each MERGE-OK sha joins the next unstarted train, reruns too. Train 63 starts the moment 62 ends.
2. "Every new container reuses the 12:15 snapshot" was not proven (wm7b r3 at 12:26 had setup). Taken: Needs owner wording
   corrected to "some new containers"; the 12:50 retry is the measure.
3. Idle lanes, no workaround. Taken in part: Needs owner default with deadline 14:00 UTC: step 0 runs setup.sh itself
   (lifts conventions 9 for setup only), owner may veto. Not taken: lifting it now (an environment item is the owner's, rules.md 3).
4. The PM broke its own "new lanes wait" default by restarting flakes-g. Taken: the default now says retry; the
   deadline the critic missed is added.
5. Owner messages 06:01, 06:52, 06:58, 06:59 missing in decisions-now.md: they are in answers.md (the full log); they are
   readings and questions, not standing decisions. Not taken. Time stamps 06:12/06:21 to check at the next decisions edit.
6. The pace line lacks the actual spend. Taken: ask the owner for a week reading with the next merge message.
7. ci-check guard not built; kit3 r1 --only lacks ci-check. Taken: guard built (below); kit3's review checks cut-check first.
8. Stop hook vs lanes.py 3 commits/h: closed with reason: hook-forced commits are the exception; the PM batches edits.
9. flakes-g r2 brief asks for --jobs 3 before flakes-f merged: no scope message to a running worker (rules.md 5); the lane's
   gate is advisory, the train is the authority. Noted.
10. Status page stale since 11:26. Taken: republish after train 62.

## Frictions resolved
All seven 11-19 entries marked resolved (fixed, guarded or closed with reason) in friction.md.

## Workflow change
tools/start_lane.sh refuses a brief without a "Step 0 ... cargo check" setup line and a brief whose merge-gate --only list
lacks ci-check (friction 11-19 kit7 rw-1 and flakes-g x3); 3 new tests, 16 green. routines.md Train line (critic 1).
