# State

## Now (2026-11-26 00:01 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC, no day cap; tokens.py 23:01 PM 1.2 % (11-26 BKK); week about 45 % vs 68 % time; need (100-45-5)/53 h = 0.94 %/h; lanes 6 build
session branch: df3d1ccf (t145 22:43); 91 greens. TRAIN146 157be195 = t145 + xp-minor r1 d3d28cfa (gate 22:43)

burn-down: F1-F4, F6, F8 met; F5 met (series r9), rerun on final head; F7 open (take r2: 88 open, 10 regr; judges r2)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | co-79 fixed r5 (t145 green); mc rv9ka-1 fixed (t144) |
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
| train146 | PM, 157be195 on df3d1ccf | gate 22:43 |
| flake-cube-r1.md (0.8 %) | sonnet high, flake-cube-r1 16977e22 | running s..PWJBMrkPqrTGzEz4N |
| launcher-notice-r1.md (0.6 %) | sonnet high, 53789bea | starting |
| tour-rows-r2.md (0.8 %) | sonnet med, 1ac365f1 | running s..MZSPbKahubVfGgjop |
| xp-blink-r1.md (0.5 %) | sonnet med, d3d28cfa | running s..QQ5tzfg5ZNj9Rx1VT |
| harness-minor-r1.md (0.6 %) | sonnet med, 5646b8fd | running s..NgK9wKeey3B3chSbH |
| tiling-focus-r2.md (1.2 %) | opus high, 6261d3e0 | running s..PNcZM4JvQiQQUQXh1 |
| review-co-drop-r1.md (0.5 %) | opus high, 75d4623e | running s..MUcBEL6p9xHpAsN9B |
## Queue (% week; 5 % reserve)
1. (rc 3) FLAKE-SOFA r1 (t144); r2 MERGE-OK 7db16e74 (t145 green). rfs2-2 cube no keyboard 3 reds: FLAKE-CUBE r1.
2. (rc 2) construct: r13 not met: co-75, co-76 hold; co-79 CO-REACH r5 MERGE-OK 7d9269ae (t145 green); CO-DROP r1 done 75d4623e (rcr5-2 fixed): review r1. FLAKE-PERSONA-NOTEPAD new.
3. (rc 2) ti-55: TILING-DOORS r4 (t142). SESSION-LAYOUT r1 MERGE-OK 691f0c8e (t144). P2 r11: T7 2/2; ti-57: TILING-FOCUS r1 6261d3e0 FIX (rtf1-1, -2 MAJOR): r2.
4. (F7, rc 1, 11) take r2: 1 same, 88 open, 10 regr. Judges r2: t144, t145 merged; XP-MINOR r1 d3d28cfa 0 MAJOR, t146, TOUR-ROWS r1 FIX: TOUR-ROWS r2 runs; rtr1-3: MC-LAUNCHER r1 MERGE-OK 53789bea t147; rml1-2: LAUNCHER-NOTICE r1; rml1-1 tour fuzzel: queued.
5. (F5, rc 3, 4, 5) after t141: FLAKE-SERIES r10 on the final head (haiku); CI-DRY r1 MERGE-OK e36b6078 (43 PASS; docs strict fixed; NOT RUN vkms, rpm, arch, mike) next train.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
