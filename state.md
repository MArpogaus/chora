# State

## Now (2026-11-24 14:50 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 14:02 PM 4.9 % (11-22) + 5.2 % (11-23) + 4.3 % (11-24), week about 29 % vs 49 % pace
session branch: 6d336bdf (t129 14:20, co-pull); 77 greens; t130 14:25: xp-38 r2, flake-series r8, tiling-flake r3, tour-base r4.
burn-down: audit r6 01:40: F1-F4, F6, F8 met (F4 no margin); F5 7 rows, no series; F7 open (r4 take stale, 24 acts)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: rc1-pass r1 xp, mc not confirmed (xp-46, xp-47, mc-60 MAJOR); tiling pass; sh-38 |
| 2 | Persona r4 | xp, mc meet; co r9 not met: co-64, co-65 fixed r3; tiling r7 not met (T7 path, ti-27, ti-50): ti-47/48 fixed; TILING-CALM r2 (t128 red) |
| 3 | 3 greens, no flake | r6 0/5; r7 2/3 (sh-38); FLAKE-SERIES r8 |
| 4, 5 | CI green on a PR; packaging | pre-commit green; CI PR at release |
| 6 | Review per area | r1 MERGE-OK (0 MAJOR; MC-FOCUS) |
| 7 | deny, machete; updates | url 2.5.9 GUARD-3 MERGE-OK 49f65382; train118 |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r2 MERGE-OK 37730187 |
| 9 | Flakes, window loss | sh-35 leak-cell train119; sh-33/sl-9 closed soak r3; sh-36 closed soak r4; sl-1, sl-2 |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| co-pull-r2.md (rc 2; co-70, co-71; 1 %) | sonnet high, claude/co-pull-r2 be6e62ec | session_016iS3kQfpC9MnJi7JhzqpHe |
| rc1-pass-tiling-r1.md (rc 1; 1.5 %) | opus high, claude/rc1-pass-tiling d40142e4 | session_016iS3kM4PtnnwHL12RWmh3X |
| p2-r8-tiling.md (rc 2; 0.8 %) | sonnet medium, claude/p2-r8-tiling bb4c07d9 | session_016iS3kNcvWg4qUZ6HRwTWte |
| tour-v9-judge-r1.md (F7, rc 11; 1.5 %) | opus high, claude/tour-v9-judge d40142e4 | starting |
| stray-super-r1.md (rc 1; xp-46, mc-60; 1 %) | sonnet high, claude/stray-super-r1 d40142e4 | session_016iS3kNnjci4yeBhCJRuWif |
| helper-greet-r1.md (rc 1; xp-47; 0.8 %) | sonnet high, claude/helper-greet-r1 d40142e4 | session_016iS3kMT9MeqAv9J4kbg26x |
## Queue (% week; 5 % reserve)
0. (rc 8) VERIFY-MINOR r2 12/14; xp-18.
1. (F4, F5, rc 1, 3) TRAIN130 cfee54b4..09a1419e running; next train: tiling-calm r2 bb4c07d9 MERGE-OK; P2-R8-TILING on it.
2. (rc 2) CO-PULL r1 merged t129; r11-co not met (C2: co-70 MAJOR, co-71); CO-PULL r2, then r12-co.
3. (rc 3, 9; F5) FLAKE-SERIES r8 (rfs7-1, rfs7-2; sh-38 leak slope); TILING-FLAKE r3 MERGE-OK (sh-39, sh-40).
4. (F7, rc 11) TOUR-BASE r4 in t130 (rtb4-1 next); take r1: 1 same, 13 change, 82 open, 3 regr; TOUR-V9-JUDGE r1, then a fix lane per look.
5. (F4, F5) after train127: FLAKE-SERIES r8 5/5 on final head (haiku), TOUR-V9 final take + 5 reviews (2.5 %), CI-PR (rc 4, 5). (rc 1) after reviews: co-44, co-45 only when construct has no open MAJOR (rules.md 6); rtd-5, rtd-7 MINOR backlog.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
