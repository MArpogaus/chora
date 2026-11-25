# State

## Now (2026-11-25 17:00 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC, no day cap; tokens.py 16:02 PM 4.0 % (11-25); week about 40 % vs 64 % time; need (100-40-5)/60 h = 0.9 %/h; 6 build lanes
session branch: 62a04be0 (t140 14:49); 88 greens. TRAIN142 6d512f1d: xp-minor r4, tiling-minor r2, co-polish r1, mc-minor r3, tiling-doors r4 (gate)
burn-down: F1-F4, F6, F8 met; F5 met (series r9), rerun on final head; F7 open (take r2: 88 open, 10 regr; judges r2)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | co-79 MAJOR reopened (CO-REACH r5); mc rv9ka-1 MAJOR |
| 2 | Persona r4 | xp, mc meet; co r9 not met; tiling r10 not met (ti-55 T7 red r10) |
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
| train142 | PM, 6d512f1d on 62a04be0 | gate 16:27 |
| flake-sofa-r1.md (F5; 0.6 %) | sonnet medium, flake-sofa-r1 62a04be0 | running s..M6gKEkxgv57K4Vkaf |
| co-reach-r5.md (rc 1, 2; co-79 MAJOR pane path; 1 %) | opus high, co-reach-r5 62a04be0 | running s..P6b9i3yJSwFStLwno |
| mc-page-end-r1.md (rv9ka-1 MAJOR; 1.2 %) | opus high, mc-page-end-r1 42418379 | running s..QKeFuc8m3ZmGDdTbd |
| xp-tail-r1.md (F7; rv9kb-2; 0.6 %) | sonnet medium, xp-tail-r1 62a04be0 | running s..PV7VUWhrb7C6ZgYQ7 |
| tour-rows-r1.md (F7; 1 %) | sonnet medium, tour-rows-r1 62a04be0 | running s..QUL8RNU6CJCyGVdBb |
| session-layout-r1.md (rtd4-1; 0.6 %) | sonnet medium, session-layout-r1 62a04be0 | running s..QnvanhG5MxiqVLtqK |
## Queue (% week; 5 % reserve)
1. (rc 1, 2) t140 green 62a04be0. TRAIN141 red: mc-sofa volume-wheel flaky on base (e6a17d59 1/3, 62a04be0 2/3); FLAKE-SOFA r1 running. TRAIN142 = t141 + mc-minor r3, tiling-doors r4; rmm3-1, -2 to backlog.
2. (rc 2) construct: co-45 fixed (co-polish r1, t141); co-44 open (backlog, rules 6); r13 not met: co-75, co-76 hold; co-79 reopened (pane 0/5): CO-REACH r5.
3. (rc 2) ti-55: TILING-DOORS r4 MERGE-OK e1ec02ee (goal order T7 before L3; harness) to t142. SESSION-LAYOUT r1 (rtd4-1) running, then P2-R11-tiling.
4. (F7, rc 1, 11) take r2: 1 same, 88 open, 10 regr. Judges r2: rv9ka-1 MAJOR (MC-PAGE-END r1), rv9kb-2 (XP-TAIL r1), rows, tools (TOUR-ROWS r1) running; rest backlog.
5. (F5, rc 3, 4, 5) after t141: FLAKE-SERIES r10 on the final head (haiku); CI-PR dry run (sonnet). ACT-WAIT r1 MERGE-OK 861d58d5 (9.4 s -> 0.05 s) to next train.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
