# State

## Now (2026-11-25 19:14 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC, no day cap; tokens.py 18:53 PM 4.4 % (11-25 BKK), 0.4 % (11-26); week about 41 % vs 65 % time; need (100-41-5)/58 h = 0.93 %/h; 6 build lanes
session branch: 62a04be0 (t140 14:49); 88 greens. T142 6d512f1d red only doors tiling helper-none (missing; rerun 3/3 green, disk 4 G: flake). TRAIN143 16977e22 = t142 + act-wait r1 (gate)
burn-down: F1-F4, F6, F8 met; F5 met (series r9), rerun on final head; F7 open (take r2: 88 open, 10 regr; judges r2)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | co-79 MAJOR reopened (CO-REACH r5); mc rv9ka-1 fixed (t144) |
| 2 | Persona r4 | xp, mc meet; co r9 not met; tiling r11 not met: T7 2/2 green, ti-57 focus after T7 (L3 red) |
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
| train143 | PM, 16977e22 on 62a04be0 | gate 17:40 |
| flake-sofa-r2.md (F5; rfs1-2; 0.6 %) | sonnet medium, flake-sofa-r2 3d555e4f | running s..MNzLQKASxWUjMFfX8 |
| co-reach-r5.md (rc 1, 2; co-79 MAJOR pane path; 1 %) | opus high, co-reach-r5 62a04be0 | running s..P6b9i3yJSwFStLwno |
| mc-minor-r4.md (rpe-1..3; 0.6 %) | sonnet medium, mc-minor-r4 23cd2b5a | running s..QU1Y2wj95EZqLt6ks |
| tour-rows-r1.md (F7; 1 %) | sonnet medium, tour-rows-r1 62a04be0 | running s..QUL8RNU6CJCyGVdBb |
| tiling-focus-r1.md (rc 1, 2; ti-57 MAJOR; 1.2 %) | opus high, tiling-focus-r1 79ed970f | running s..LQrg3Gt2nuQWTHwkr |
| ci-dry-r1.md (rc 4, 5; 0.6 %) | sonnet medium, ci-dry-r1 62a04be0 | starting |
## Queue (% week; 5 % reserve)
1. (rc 1, 2) FLAKE-SOFA r1 MERGE-OK 3d555e4f (t144); r2 rooms-pill light; rmm3-1, -2 backlog.
2. (rc 2) construct: co-45 fixed; co-44 backlog; r13 not met: co-75, co-76 hold; co-79 reopened (pane 0/5): CO-REACH r5.
3. (rc 2) ti-55: TILING-DOORS r4 MERGE-OK e1ec02ee (goal order T7 before L3; harness) to t142. SESSION-LAYOUT r1 MERGE-OK 691f0c8e (t144). P2 r11: T7 2/2; ti-57 MAJOR: TILING-FOCUS r1; Kemal T3 unstable.
4. (F7, rc 1, 11) take r2: 1 same, 88 open, 10 regr. Judges r2: rv9ka-1 MC-PAGE-END r1 MERGE-OK 23cd2b5a (t144), MC-MINOR r4, rv9kb-2 XP-TAIL r1 MERGE-OK e4157dbd (t144; rxt-1, -2 MINOR backlog), rows, tools (TOUR-ROWS r1) running; rest backlog.
5. (F5, rc 3, 4, 5) after t141: FLAKE-SERIES r10 on the final head (haiku); CI-DRY r1 running. ACT-WAIT r1 861d58d5 in t143.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
