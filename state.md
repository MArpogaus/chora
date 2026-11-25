# State

## Now (2026-11-25 12:53 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 12:53 PM 3.2 % (11-25), week about 38 % vs 62 % pace: under-spending, fill 6 lanes
session branch: d86be5d0 (t138 11:10: xp-minor r1, mc-minor r1); 86 greens. TRAIN139 tiling-doors r3 35e31662, tiling-minor r1 31af60c5 (gate running)
burn-down: audit r6: F1-F4, F6, F8 met; F5 no series; F7 open (v9 take: 82 open, 3 regr)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | construct co-75, -76, -79 fixed t136 (recheck next persona round); xp, mc: none; tiling ti-55, ti-56 reopened r9 |
| 2 | Persona r4 | xp, mc meet; co r9 not met; tiling r9 not met (ti-55, ti-56; DOORS r3, then P2-R10) |
| 3 | 3 greens, no flake | met: t130-t132 green; FLAKE-SERIES r9 5/5 c3c37c2e |
| 4, 5 | CI green on a PR; packaging | pre-commit green; CI PR at release |
| 6 | Review per area | r1 MERGE-OK (0 MAJOR; MC-FOCUS) |
| 7 | deny, machete; updates | url 2.5.9 GUARD-3 MERGE-OK 49f65382; train118 |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r2 MERGE-OK 37730187 |
| 9 | Flakes, window loss | sh-33, sh-35, sh-36, sl-9 closed; sl-1, sl-2 |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| train139 (Queue 1) | PM, e39448ad on d86be5d0 | gate running |
| co-minor-r2.md (rc 2) | done 466fcd11, PM review r2 MERGE-OK | train140 after t139 |
| xp-minor-r3.md (rc 1) | done cb5fcb7f, PM review MERGE-OK | train140 after t139 |
| mc-minor-r2.md (rc 1) | done 0f68810c, review r2 MERGE-OK (0 MAJOR, 5 MINOR) | train140 after t139 |
| mc-minor-r3.md (rc 1; rmm2-1..6; 0.8 %) | sonnet medium, claude/mc-minor-r3 0f68810c | running session_016iS3kNMXoJFbpfCZT9oZsw |
| co-polish-r1.md (rc 1; co-44, co-45; 0.8 %) | sonnet medium, claude/co-polish-r1 d86be5d0 | starting |
| xp-minor-r4.md (rc 1; rxm3-1, -2; 0.3 %) | haiku medium, claude/xp-minor-r4 cb5fcb7f | starting |
## Queue (% week; 5 % reserve)
1. (rc 1, 2) t138 green d86be5d0 (xp-9, -19, -28, -29 closed). TRAIN139 running: tiling-doors r3 35e31662, tiling-minor r1 31af60c5.
2. (rc 2) construct: CO-MINOR r2 466fcd11 PM review MERGE-OK (rcm1-1 closed); train140 with xp-minor r3, mc-minor r2; co-77 by design D106; persona recheck co-75/76/79 in P2-R10.
3. (rc 2) tiling in train139; then P2-R10 persona recheck (ti-55, ti-56, co-75/76/79; 0.8 %).
4. (F7, rc 11) TOUR-BASE r8 MERGE-OK 27fb2599 (PM), train136; then v9 take; rv9j-1..5 MINOR after.
5. (F4, F5) after train127: FLAKE-SERIES r8 5/5 on final head (haiku), TOUR-V9 final take + 5 reviews (2.5 %), CI-PR (rc 4, 5). (rc 1) after reviews: co-44, co-45 only when construct has no open MAJOR (rules.md 6); rtd-5, rtd-7 MINOR backlog.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
