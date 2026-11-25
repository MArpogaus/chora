# State

## Now (2026-11-25 20:08 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC, no day cap; tokens.py 20:07 PM 4.4 % (11-25 BKK), 0.7 % (11-26); week about 42 % vs 66 % time; need (100-42-5)/56 h = 0.95 %/h; lanes 3 build + 2 reviews
session branch: 16977e22 (t143 19:15); 89 greens. TRAIN144 5646b8fd = t143 + session-layout r1, flake-sofa r1, mc-page-end r1, xp-tail r1 (gate)
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
| train144 | PM, 5646b8fd on 16977e22 | gate 19:17 |
| flake-sofa-r2.md (F5; rfs1-2; 0.6 %) | sonnet medium, flake-sofa-r2 3d555e4f | running s..MNzLQKASxWUjMFfX8 |
| review-co-reach-r5.md (rc 1, 2; co-79; 0.5 %) | opus high, co-reach-r5 7d9269ae | starting |
| mc-minor-r4.md (rpe-1..3; 0.6 %) | sonnet medium, mc-minor-r4 23cd2b5a | running s..QU1Y2wj95EZqLt6ks |
| review-tour-rows-r1.md (F7; 0.5 %) | opus high, tour-rows-r1 1ac365f1 | starting |
| tiling-focus-r1.md (rc 1, 2; ti-57 MAJOR; 1.2 %) | opus high, tiling-focus-r1 79ed970f | running s..LQrg3Gt2nuQWTHwkr |
| xp-minor-r1.md (rxt-1..3; 0.6 %) | sonnet medium, xp-minor-r1 e4157dbd | starting |
## Queue (% week; 5 % reserve)
1. (rc 1, 2) FLAKE-SOFA r1 3d555e4f (t144); r2 rooms-pill light.
2. (rc 2) construct: co-45 fixed; co-44 backlog; r13 not met: co-75, co-76 hold; co-79 CO-REACH r5 done 7d9269ae (pane 5/5, gate green): review r5.
3. (rc 2) ti-55: TILING-DOORS r4 MERGE-OK e1ec02ee (goal order T7 before L3; harness) to t142. SESSION-LAYOUT r1 MERGE-OK 691f0c8e (t144). P2 r11: T7 2/2; ti-57 MAJOR: TILING-FOCUS r1; Kemal T3 unstable.
4. (F7, rc 1, 11) take r2: 1 same, 88 open, 10 regr. Judges r2: rv9ka-1 MC-PAGE-END r1 MERGE-OK 23cd2b5a (t144), MC-MINOR r4, rv9kb-2 XP-TAIL r1 e4157dbd (t144; rxt-1..3: XP-MINOR r1), TOUR-ROWS r1 done 1ac365f1 (0 open, 8 regr = rv9ka-1; mc act 4 launcher red 2/2): review r1.
5. (F5, rc 3, 4, 5) after t141: FLAKE-SERIES r10 on the final head (haiku); CI-DRY r1 MERGE-OK e36b6078 (43 PASS; docs strict fixed; NOT RUN vkms, rpm, arch, mike) next train.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
