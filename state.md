# State

## Now (2026-11-26 02:17 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC, no day cap; tokens.py 01:31 PM 1.7 % (11-26 BKK); week about 46 % vs 68 % time; need (100-46-5)/50 h = 0.98 %/h; lanes 6 build
session branch: 6daaaab8 (t147 02:02: flake-cube, mc-launcher, xp-blink r1, co-drop r1); 93 greens. TRAIN148 3e3b1b2d = t147 + harness-minor b1a6465c, tour-launch 07eb539f

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
| train148 | PM, 3e3b1b2d on 6daaaab8 | gate 02:03 |
| flake-persona-notepad-r1.md (0.5 %) | sonnet med, df3d1ccf | running s..PVzKu2DC4f86sGRkY |
| tour-rows-r3.md (0.6 %) | sonnet med, f992fe6a + 157be195 | running s..QgZbQTLiR8q7MVDgf |
| co-drop-r2.md (0.6 %) | sonnet high, 75d4623e | running s..MVitc1ELinZfjMSPQ |
| mc-minor-r5.md (0.6 %) | sonnet med, 157be195 | running s..PUjQt7D2R4B1JfAFV (red 01:58) |
| xp-blink-r3.md (0.3 %) | sonnet med, 312a5ff1 + 6daaaab8 | starting |
| review-tiling-focus-r2.md (0.5 %) | opus high, 712a01f5 | running s..Nm4A8Cm62BvUuxcMX |
## Queue (% week; 5 % reserve)
1. (rc 3) FLAKE-SOFA r1 (t144); r2 MERGE-OK 7db16e74 (t145 green). rfs2-2: FLAKE-CUBE r1 (t147); FLAKE-PERSONA-NOTEPAD r1 runs. HARNESS-MINOR r1, TOUR-LAUNCH r1 in t148; then lanes merge the head.
2. (rc 2) construct: r13 not met: co-75, co-76 hold; co-79 CO-REACH r5 MERGE-OK 7d9269ae (t145 green); CO-DROP r1 (t147); rcdr1-1, -2, rv9kb-3: CO-DROP r2 runs.
3. (rc 2) ti-55: TILING-DOORS r4 (t142). SESSION-LAYOUT r1 MERGE-OK 691f0c8e (t144). P2 r11: T7 2/2; ti-57: TILING-FOCUS r2 712a01f5 done (gate 66 PASS): review r2.
4. (F7, rc 1, 11) take r2: 88 open, 10 regr; judges r2: XP-BLINK r1 (t147), r2 MERGE-OK 312a5ff1 (PM review): t149; rxb2-1: XP-BLINK r3; TOUR-ROWS r2 INCOMPLETE (rtr2-1 counts 15/9/10): r3; rtr1-3: MC-LAUNCHER r1 (t147); rml1-1: TOUR-LAUNCH r1 in t148. rmm3-1..-4: MC-MINOR r5 runs.
5. (F5, rc 3, 4, 5) after t141: FLAKE-SERIES r10 on the final head (haiku); CI-DRY r1 merged (NOT RUN vkms, rpm, arch, mike).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
