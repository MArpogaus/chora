# State

## Now (2026-11-26 04:00 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC, no day cap; tokens.py 03:33 PM 2.3 % (11-26 BKK); week about 48 % vs 71 % time; need (100-48-5)/48 h = 0.98 %/h; lanes 5 build + review
lanes-short: construct in review, other areas taken
session branch: 6daaaab8 (t147 02:02); 93 greens. TRAIN148 3e3b1b2d = t147 + harness-minor b1a6465c, tour-launch 07eb539f

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
| train148 | PM, 3e3b1b2d | red 02:33 tiling-audit (flake, 3/3 alone green); rerun 02:47 |
| flake-persona-notepad-r1.md (0.5 %) | sonnet med, ef55b4fb | MERGE-OK: own train after t148 |
| tour-rows-r3.md (0.6 %) | sonnet med, f992fe6a + 157be195 | running s..QgZbQTLiR8q7MVDgf |
| co-drop-r2.md (0.6 %) | sonnet high, 75d4623e | done 636287c0 (gate green): review |
| review-co-drop-r2.md (0.5 %) | opus high, 636287c0 | starting |
| mc-minor-r5.md (0.6 %) | sonnet med, 157be195 | MERGE-OK 1ab2e6a7 (PM review): t149 |
| tiling-focus-r3.md (1.0 %) | opus high, 712a01f5 + 6daaaab8 + 1ab2e6a7 | running s..QXm22Bg2sK5TKXfXg |
| xp-blink-r3.md (0.3 %) | sonnet med, 312a5ff1 + 6daaaab8 | running s..LbEm47JYWeUTPVQyF |
| flake-bar-full-r1.md (0.8 %) | sonnet high, 6daaaab8 | running s..MRoVkQ1EbwsGcjcGg |
| tour-harness-r1.md (0.4 %) | sonnet med, 07eb539f + 6daaaab8 + b1a6465c | running s..Nr2Jefb9u8CQ7MGbz |
## Queue (% week; 5 % reserve)
1. (rc 3) FLAKE-PERSONA-NOTEPAD r1 MERGE-OK ef55b4fb: t149. t148 tiling-audit fullscreen ink red: FLAKE-BAR-FULL r1. HARNESS-MINOR r1, TOUR-LAUNCH r1 in t148.
2. (rc 2) construct r13 not met: co-75, co-76 hold; CO-DROP r1 (t147); rcdr1-1, -2, rv9kb-3: CO-DROP r2 636287c0: review r2 runs.
3. (rc 2) P2 r11: T7 2/2; ti-57: TILING-FOCUS r2 FIX (rtf2-1 MAJOR: mc keeps and yields keys; rtf2-2 nit): TILING-FOCUS r3 runs.
4. (F7, rc 1, 11) take r2: 88 open, 10 regr; judges r2: XP-BLINK r2 MERGE-OK 312a5ff1: t149; rxb2-1: r3; TOUR-ROWS r2 INCOMPLETE: r3; rtl1-1, -2, rhm1-1: TOUR-HARNESS r1. MC-MINOR r5 MERGE-OK 1ab2e6a7: t149 (rmm5-1 MINOR filed).
5. (F5, rc 3, 4, 5) after t141: FLAKE-SERIES r10 on the final head (haiku); CI-DRY r1 merged (NOT RUN vkms, rpm, arch, mike).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
