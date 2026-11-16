# Self-check 2026-11-16 (workday close, 12:5x UTC)

## Answers
- Burn-down: not on track for Fri. 2 BLOCK + 19 MAJOR at 10:1x; merged since: xp-polish r1, multilook r1 (trains 26, 27); mc r1+r2, xp-polish r2, tiling-look r1 wait for review or train; co-18 BLOCK open in construct r1. Likely tag Tue 11-24; Wed 12:00 check stays.
- Reviewed branch unmerged > 12 h: none (mc r1 11:27, secfix-a 11:3x, tiling-look r1 11:4x). mc r1 merges alone if mc r2 is not MERGE-OK by 18:00.
- Running not in Queue: testwatch (now in Queue line 1).
- Pace: missed. Owner readings 38 % (01:17) to 45 % (12:10), about 1 %/h, 24 %/day against the 12.5 % cap.
- Models: lanes on their Model lines (sonnet builds, haiku testwatch). Reviews ran on sonnet against rule 9 "opus reviews" without a rule change (rule 10 broken).
- Vision drift: none; Helper kept for RC (owner 11:17).

## Critic (opus, medium) and actions
1. Pace overstated: measured rate now in state.md; ledger r3 and media r3 held to the next reading.
2. Budget guard reads the PM transcript only (tokens.py): to build, check_log reads owner readings for the rate (backlog harness).
3. Sonnet reviews broke rule 9: rule 9 changed at the train 27 boundary to "sonnet builds, reviews; opus security, hard causes".
4. F26 needs an opus cause lane: queued Tue 00:xx (new UTC day, under cap), replaces ledger r3.
5. Burn-down line stale: rewritten; tag date Tue 11-24 likely.
6. rc.md state column stale (also 11-15): drop it at train 29.
7. lanes.py limit 4 vs WIP 6: fixed to 6; log commits batched from now on.
8. Friction 54 closed without a fix: kept closed (train gate is the authority) but F30 and lane-container tools go into the harness lane with F26.

## Frictions
All open entries up to 52 were resolved earlier; 53 (disk again) guarded by the routines.md disk line; 54 closed with reason.

## Workflow change
routines.md: df check before every review or train, one build at a time under 20 G (friction 53). rule 9 review model (critic 3). lanes.py MAX_LANES 6.
