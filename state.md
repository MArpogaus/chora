# State

## Now (2026-11-24 03:34 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 03:02 PM 4.9 % (11-22) + 5.2 % (11-23) + 1.4 % (11-24), week about 24 % vs 41 % pace
session branch: 05a63377 (t123 green 03:07: soak-r4); 73 greens; train124 running (co-reach-r2 efb0f263).
burn-down: audit r6 01:40: F1-F4, F6, F8 met (F4 no margin: W9 779 = RC; GUARD-4); F5 7 rows, no series; F7 open (r4 take stale, 24 acts)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: RECORD-V9 (ti-38 next train) |
| 2 | Persona r4 | xp, mc meet; co r9 not met: co-61 ok, co-64, co-65 MAJOR, CO-REACH r3; tiling r6 not met (T7 reach, T6 ti-49 MINOR; L1, L2 hold): ti-47, ti-48 MAJOR, TILING-REACH r1 |
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
| train124: co-reach-r2 efb0f263 | PM | on 05a63377; gate 03:08 |
| tiling-reach-r1.md (rc 1, 2; 1 %) | sonnet high, claude/tiling-reach cae62f50 | session_016iS3kMq4yU2VPBXarSuEkP 01:52 |
| guard-4-r1.md (F4 W9, F5 sh-37; 0.5 %) | sonnet medium, claude/guard-4 cae62f50 | session_016iS3kLiaHH34dzQpNbYZDk 01:40 |
| co-reach-r3.md (rc 1, 2; 2 %) | opus high, claude/co-reach-r3 b2fa7792 | session_016iS3kNvatmG2tDzUZuAFQy 03:32 |
| xp-title-r1.md (F7; 0.6 %) | sonnet medium, claude/xp-title 05a63377 | session_016iS3kNJs4X9FpcdTqsoc7j 03:32 |
| tour-base-r2.md (F7, rc 11; 1.2 %) | sonnet high, claude/tour-base 645afaf5 | starting |
| flake-series-r7.md (rc 3; 1 %) | sonnet high, claude/flake-series-r7 1ec04694 | session_016iS3kLkxQHS3DxoJnqt8Xk 19:00 |
## Queue (% week; 5 % reserve)
1. (rc 1, 2) TRAIN124 running: co-reach-r2 efb0f263; lanes merge 05a63377.
2. (rc 1, 2; 2 %) co r9 not met: CO-REACH r3 co-64, co-65 MAJOR, then persona r10; TILING-REACH r1, then persona r7.
3. (rc 3, 9; F5) FLAKE-SERIES r7 running (sh-34, stall, xp-42); soak-r4 merged t123 (sh-36 closed).
4. (F7, rc 11; 1.2 %) TOUR-BASE r1 review FIX (rtb-1 MAJOR: 71 mc rows unexplained): r2 runs (bisect mc-59 or open verdict); MERGE-OK -> own train.
5. (F4, F5; 0.5 %) GUARD-4 running (sleep ratchet, sh-37); then FLAKE-SERIES r8 5/5 on final head (haiku 1 %), TOUR-V9 final take + 5 reviews (2.5 %), CI-PR (rc 4, 5). (rc 1) after reviews: co-44, co-45 only when construct has no open MAJOR (rules.md 6); rtd-5, rtd-7 MINOR backlog.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
