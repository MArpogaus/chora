# Self-check 2026-11-25 (12:52 UTC, 19:52 Bangkok)

## Answers
- RC burn-down on track? Partly. t139 is green (12:55): ti-55 and ti-56 are fixed with red tests first, but the persona recheck (P2-R10) has not run, so rc 2 for co and tiling stays open. F7 has had no v9 retake since 11-24.
- Reviewed branch unmerged > 12 h? No. tiling-minor r1 (MERGE-OK 05:20) merged in t139 at 12:55, inside the limit. xp r3, mc r2 and co r2 are in t140.
- Running but not in the Queue? No.
- Pace: no. The week stands at about 38 % against 62 % time. 11-24 spent 5.1 % against 14 %/day. The 11-24 promise of at least 0.8 %/h and 6 lanes was not kept: the table never held more than 4 build lanes.
- Models: kept. Sonnet built, haiku runs the test lane, opus ran the mc review and this critic. No Fable.
- Vision drift: yes. Effort went to MINOR rounds on xp and mc, which already meet rc 1 and 2, while the rc 2 recheck for co and tiling waited. Tour-base r5 to r8 broke the 11-24 tooling freeze.

## Critic points and actions
1. Under-spending, corrected a fifth time: now 5 build lanes (mc-minor r3, co-polish r1, xp-minor r4, tour-v9 take r2, tiling-minor r2); P2-R10 becomes the 6th lane on the t140 head.
2. Wrong pace figures: the budget line now gives the needed rate, (100 - 38 - 5) / 63 h = 0.9 %/h, worked out from `date -u`.
3. Tooling freeze broken: tour-v9 take r2 starts now. No tooling round runs before it, and 4 fresh-eyes judges (opus high) follow it.
4. Ranking: Queue lines rank by RC row. MINOR rounds on looks that already meet the RC only fill leftover lanes.
5. The design/rc.md table rewrite, promised three times, is dropped openly. The RC table in state.md is the live view; rc.md keeps the criteria. Logged in backlog.md.
6. Overstatements fixed:
   - F5 now reads the same in state.md and on the status page (met r9, rerun on the final head).
   - The F6 claim "86 green merges, all reviewed" now names the 689bff45 red-first exception.
   - rxm3-1: the fix works (the gate cell passes); only its unit test can pass vacuously on a host that already has the file. It stays MINOR, and the xp-minor r4 lane (haiku) closes it now, before the RC. This is a judgment against the critic, recorded here.
7. Queue 4-5 rewritten (no more "after train127").
8. Guard: see the workflow change below.

## Frictions resolved
No entry is open in friction.md. The recurring cause (under-spending under a manual guard) is fixed by the guard below.

## Workflow change
tools/check_log.py `check_lanes_full`: a log commit fails when the week share is more than 5 points under the time share and fewer than 6 build lanes are running or starting. The only escape is a `lanes-short: <reason>` line in Now. Its first run caught today's state (5 lanes).
