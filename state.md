# State

## Now (2026-11-24 01:53 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 01:53 PM 4.9 % (11-22) + 5.2 % (11-23) + 1.4 % (11-24), week about 24 % vs 41 % pace
session branch: cae62f50 (t122 green 00:58: notify-text, tiling-disc, xp-front-bar-r2); 72 greens; train123 running (soak-r4).
burn-down: audit r6 01:40: F1-F4, F6, F8 met (F4 no margin: W9 779 = RC; GUARD-4); F5 7 rows, no series; F7 open (r4 take stale, 24 acts)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: RECORD-V9 (ti-38 next train) |
| 2 | Persona round 4 | xp, mc meet; co r8 not met (reach; co-52, co-54 hold): co-61, co-62 MAJOR, CO-REACH r2; tiling r6 not met (T7 reach, T6 ti-49 MINOR; L1, L2 hold): ti-47, ti-48 MAJOR, TILING-REACH r1 |
| 3 | 3 greens, no flake | r6 0/5; FLAKE-SERIES r7; mc-58 fixed t121 |
| 4, 5 | CI green on a PR; packaging | pre-commit green; CI PR at release |
| 6 | Review per area | r1 MERGE-OK (0 MAJOR; MC-FOCUS) |
| 7 | deny, machete; updates | url 2.5.9 GUARD-3 MERGE-OK 49f65382; train118 |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r2 MERGE-OK 37730187 (pr-1..5) |
| 9 | Flakes, window loss | sh-35 leak-cell train119; sh-33/sl-9 closed soak r3; sh-36 closed soak r4; sl-1, sl-2 |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| train123: soak-r4 f0c45fcf | PM | on cae62f50; gate 01:00 |
| tiling-reach-r1.md (rc 1, 2; 1 %) | sonnet high, claude/tiling-reach cae62f50 | session_016iS3kMq4yU2VPBXarSuEkP 01:52 |
| guard-4-r1.md (F4 W9, F5 sh-37; 0.5 %) | sonnet medium, claude/guard-4 cae62f50 | session_016iS3kLiaHH34dzQpNbYZDk 01:40 |
| co-reach-r2.md (rc 1, 2; 2 %) | opus high, claude/co-reach-r2 8c411416 | session_016iS3kQjgrwECodJywoZZU8 00:48 |
| tour-base-r1.md (F7, rc 11; 1.5 %) | sonnet high, claude/tour-base 8c411416 | session_016iS3kLwgpLsqzbCZvy458S 23:54 |
| flake-series-r7.md (rc 3; 1 %) | sonnet high, claude/flake-series-r7 1ec04694 | session_016iS3kLkxQHS3DxoJnqt8Xk 19:00 |
## Queue (% week; 5 % reserve)
1. (rc 9) TRAIN123 running: soak-r4 f0c45fcf; then lanes merge cae62f50+.
2. (rc 1, 2; 2 %) CO-REACH r2 (co-61, co-62), then construct persona r9; TILING-REACH r1 (ti-47 double bar, ti-48 handover words), then tiling persona r7.
3. (rc 3, 9; F5) FLAKE-SERIES r7 running (sh-34, stall, xp-42); soak-r4 MERGE-OK f0c45fcf (sh-36 closed) joins train123.
4. (F7, rc 11; 1.5 %) TOUR-BASE: I6 baseline, 0 unexplained acts.
5. (F4, F5; 0.5 %) GUARD-4 running (sleep ratchet, sh-37); then FLAKE-SERIES r8 5/5 on final head (haiku 1 %), TOUR-V9 final take + 5 reviews (2.5 %), CI-PR (rc 4, 5). (rc 1) after reviews: co-44, co-45 only when construct has no open MAJOR (rules.md 6); rtd-5, rtd-7 MINOR backlog.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
