# State

## Now (2026-11-25 22:02 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC, no day cap; tokens.py 21:01 PM 4.4 % (11-25 BKK), 0.9 % (11-26); week about 43 % vs 67 % time; need (100-43-5)/55 h = 0.95 %/h; lanes 6 build + 1 review
session branch: 5646b8fd (t144 21:00); 90 greens. TRAIN145 df3d1ccf = t144 + ci-dry r1, flake-sofa r2, co-reach r5, mc-minor r4 (gate)

burn-down: F1-F4, F6, F8 met; F5 met (series r9), rerun on final head; F7 open (take r2: 88 open, 10 regr; judges r2)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | co-79 fixed r5 (t145); mc rv9ka-1 fixed (t144) |
| 2 | Persona r4 | xp, mc meet; co r9 not met; tiling r11 not met: T7 2/2 green, ti-57 focus after T7 (L3 red) |
| 3 | 3 greens, no flake | met: t130-t132 green; FLAKE-SERIES r9 5/5 c3c37c2e |
| 4, 5 | CI green on a PR; packaging | pre-commit green; CI PR at release |
| 6 | Review per area | r1 MERGE-OK (0 MAJOR; MC-FOCUS) |
| 7 | deny, machete; updates | GUARD-3 49f65382 (t118) |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r2 MERGE-OK 37730187 |
| 9 | Flakes, window loss | sh-33, sh-35, sh-36, sl-9 closed; sl-1, sl-2 |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| train145 | PM, df3d1ccf on 5646b8fd | gate 21:01, 34 PASS 22:02 |
| flake-cube-r1.md (0.8 %) | sonnet high, flake-cube-r1 16977e22 | running s..PWJBMrkPqrTGzEz4N |
| mc-launcher-r1.md (0.8 %) | sonnet high, 5646b8fd | running s..QDsyyTVKrVvGrALpf |
| tour-rows-r2.md (0.8 %) | sonnet med, 1ac365f1 | running s..MZSPbKahubVfGgjop |
| xp-blink-r1.md (0.5 %) | sonnet med, d3d28cfa | running s..QQ5tzfg5ZNj9Rx1VT |
| harness-minor-r1.md (0.6 %) | sonnet med, 5646b8fd | running s..NgK9wKeey3B3chSbH |
| co-drop-r1.md (1.0 %) | opus high, co-drop-r1 7d9269ae | running s..QFQWDJPVE889fjtNT |
| review-tiling-focus-r1.md | opus high, 6261d3e0 | running s..QGnr8CVRJk9i8H2qA |
## Queue (% week; 5 % reserve)
1. (rc 3) FLAKE-SOFA r1 (t144); r2 MERGE-OK 7db16e74 (t145). rfs2-2 cube no keyboard 3 reds: FLAKE-CUBE r1.
2. (rc 2) construct: r13 not met: co-75, co-76 hold; co-79 CO-REACH r5 MERGE-OK 7d9269ae (t145); rcr5-1 MINOR, rcr5-2 MINOR/MAJOR if a run confirms (drop on a face stays in navigate): CO-DROP r1 runs.
3. (rc 2) ti-55: TILING-DOORS r4 (t142). SESSION-LAYOUT r1 MERGE-OK 691f0c8e (t144). P2 r11: T7 2/2; ti-57: TILING-FOCUS r1 6261d3e0 (66 PASS): review r1 runs.
4. (F7, rc 1, 11) take r2: 1 same, 88 open, 10 regr. Judges r2: MC-PAGE-END r1 (t144), MC-MINOR r4 MERGE-OK 2ad0c45c (t145), rv9kb-2 XP-TAIL r1 e4157dbd (t144); XP-MINOR r1 d3d28cfa 0 MAJOR, t146, TOUR-ROWS r1 FIX: TOUR-ROWS r2 runs; rtr1-3: MC-LAUNCHER r1 runs.
5. (F5, rc 3, 4, 5) after t141: FLAKE-SERIES r10 on the final head (haiku); CI-DRY r1 MERGE-OK e36b6078 (43 PASS; docs strict fixed; NOT RUN vkms, rpm, arch, mike) next train.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
