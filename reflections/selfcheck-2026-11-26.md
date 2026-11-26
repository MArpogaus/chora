# Self-check 2026-11-26 (workday close, 12:50 UTC)

## Main-duty questions
- Burn-down: partly. F1-4, F6, F8 hold; F5 back to open (trains t141, t142, t148, t153 red; t153 mc-scale = sh-37); F7 open (88 open, 10 regr). RC 1: xp confirmed r2; mc, tiling, co have no current independent pass.
- Branch unmerged > 12 h: none. MERGE-OK shas in t154 (gate running, green through keys) or t155 (kit-icons abdd0158).
- Running but not queued: none; every Lanes row now names its Queue line.
- Pace: not kept. Week estimate 51 % (not measured; tokens.py measures the PM only) at 76 % time; about 0.56 %/h against 1.1 %/h needed. Lanes held 6 only 01:31-06:52.
- Models: kept (haiku tests, sonnet routine, opus reviews and ti-57 root cause; no Fable).
- Vision drift: yes. About 9 xp rounds since 11-25 while mc, tiling, co rc1 passes waited.

## Independent opus critic (12:51), points and actions
1. Pace 0.56 vs 1.1 %/h; 51 % unmeasured. Action: label it unmeasured in state.md; 6 lanes now.
2. Under-spend correction broken again; lanes-short as escape. Action: lanes-short only with an end time; lanes 5, 6 started now (rc1-pass-mc-r2 opus high, flake-series-r10 haiku high).
3. 11-25 promises broken (6 lanes, pace, tour tooling freeze, ranking). Action: tour take r3 next, no tour tooling before it.
4. Ranking not by RC row. Action: rc1 passes mc (now), tiling, co ahead of any xp round (Queue 4b).
5. Rule 6 broken and excused (tiling-chip while ti-57 open; 4 opus rounds). Action: tiling-focus r4 is the last round, else the owner decides the shared-focus rule.
6. Red-first broken and approved 5 times. Action: H-RF2 before train 156; by hand until then.
7. Claims without evidence (RC 3 met, xp-focus as fix, RC 6 no sha, OK vs MERGE-OK). Action: RC 3 open; xp-focus = diagnosis; MERGE-OK with sha.
8. lanes.py guards ignored. Action: Lanes trimmed to running rows plus one batch row, each with (Q n); making lanes.py fail the log commit goes to backlog (harness).
9. Log commit cap exceeded (hour 12: 4). Action: one log commit per hour, batched.
10. Frictions not logged (t148, t153, t150, rule 6, red-first waivers). Action: five entries in friction.md, each resolved with a change.

## Frictions resolved
- 11-26 07:08 xp-pager (fixed finding rebuilt): guarded by tools/start_lane.sh "Red on head:" check (tested: refuses a brief without it, accepts "none (<reason>)").
- Five critic entries 12:51: resolved with the changes above.

## Workflow change
tools/start_lane.sh refuses a brief that lacks a "Red on head: <cell> red on <sha>" or "Red on head: none (<reason>)" line.
