# State

## Now (2026-11-26 06:00 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC, no day cap; tokens.py 05:57 PM 2.9 % (11-26 BKK); week about 48 % vs 72 % time; need (100-48-5)/46 h = 1.02 %/h; lanes 6 build
session branch: a93bc75b (t149 05:55); 95 greens. TRAIN150 = t149 + tour-harness 06c3f7cc (gate runs, --allow-skip docs); then t151 xp-blink + mc-minor

burn-down: F1-F4, F6, F8 met; F5 met (series r9), rerun on final head; F7 open (take r2: 88 open, 10 regr; judges r2)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | co-79, rv9ka-1 fixed |
| 2 | Persona r4 | xp, mc meet; co r9 not met; tiling r11 not met: T7 2/2 green, ti-57 focus after T7 (L3 red) |
| 3 | 3 greens, no flake | met: FLAKE-SERIES r9 5/5 |
| 4, 5 | CI on a PR; packaging | CI PR at release |
| 6 | Review per area | r1 MERGE-OK |
| 7 | deny, machete; updates | GUARD-3 49f65382 (t118) |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r2 MERGE-OK 37730187 |
| 9 | Flakes, window loss | sl-1, sl-2 open |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| train150 | PM, 168d5cbe | gate runs 05:56 |
| tour-rows-r3.md (0.6 %) | sonnet med, f992fe6a + 157be195 | running s..QgZbQTLiR8q7MVDgf |
| co-drop-r3.md (0.6 %) | sonnet high, 636287c0 + 6daaaab8 | running s..N8SWFfbTNnRgBU7v1 |
| mc-minor-r5.md (0.6 %) | sonnet med, 157be195 | MERGE-OK 1ab2e6a7 (PM review): t151 |
| tiling-focus-r3.md (1.0 %) | opus high, 712a01f5 + 6daaaab8 + 1ab2e6a7 | running s..QXm22Bg2sK5TKXfXg |
| xp-blink-r3.md (0.3 %) | sonnet med, 312a5ff1 + 6daaaab8 | MERGE-OK 3fd3a3bb (PM r3): t151 |
| flake-bar-full-r1.md (0.8 %) | sonnet high, 6daaaab8 | running s..MRoVkQ1EbwsGcjcGg |
| tour-harness-r1.md (0.4 %) | sonnet med | MERGE-OK 06c3f7cc: t150 |
| mc-resume-r1.md (0.5 %) | sonnet high, 3e3b1b2d + 1ab2e6a7 | running s..MaFjpYTR7g5JAHQ1J |
| xp-busy-r1.md (0.5 %) | sonnet high, 3e3b1b2d + 3fd3a3bb | running s..LpwG2o9CLX3eG7mHu |
## Queue (% week; 5 % reserve)
1. (rc 3) FLAKE-PERSONA-NOTEPAD r1 merged t149; FLAKE-BAR-FULL r1 runs. TOUR-HARNESS r1 in t150 (--allow-skip docs, rth1-1), then lanes merge head; t151 xp-blink + mc-minor.
2. (rc 2) construct r13 not met: co-75, co-76 hold; CO-DROP r2 FIX (rcdr2-1 MAJOR sheet at 1024x768): r3 runs.
3. (rc 2) P2 r11: T7 2/2; ti-57: TILING-FOCUS r2 FIX (rtf2-1 MAJOR mc keys): r3 runs.
4. (F7, rc 1, 11) take r2: 88 open, 10 regr; judges r2: XP-BLINK r3 MERGE-OK 3fd3a3bb: t151 (rxb3-1 NIT); XP-BUSY r1 (Stop at send); TOUR-ROWS r2 INCOMPLETE: r3; rtl1-1, -2, rhm1-1: TOUR-HARNESS r1. MC-MINOR r5 MERGE-OK 1ab2e6a7: t151 (rmm5-1 MINOR filed).
4b. (rc 9) rfs2-1 mc host restart keeps old config, rmm5-2: MC-RESUME r1.
5. (F5, rc 3, 4, 5) after t141: FLAKE-SERIES r10 on the final head (haiku);

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
