# State

## Now (2026-11-24 07:59 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 06:06 PM 4.9 % (11-22) + 5.2 % (11-23) + 2.1 % (11-24), week about 25 % vs 43 % pace
session branch: 2235551f (t125 green 07:50: xp-title); 75 greens; train126 07:45 (guard-4 284ce9a5).
burn-down: audit r6 01:40: F1-F4, F6, F8 met (F4 no margin: W9 779 = RC; GUARD-4); F5 7 rows, no series; F7 open (r4 take stale, 24 acts)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: ti-47, ti-48, sh-38, co-67 (ledger-audit r1: 11 closed) |
| 2 | Persona r4 | xp, mc meet; co r9 not met: co-64, co-65 fixed r3; tiling r6 not met: ti-47, ti-48 MAJOR: r1 4e257c06 MERGE-OK (8 MINOR rtr), train127 |
| 3 | 3 greens, no flake | r6 0/5; r7 2/3 (sh-38); FLAKE-SERIES r8; mc-58 fixed t121 |
| 4, 5 | CI green on a PR; packaging | pre-commit green; CI PR at release |
| 6 | Review per area | r1 MERGE-OK (0 MAJOR; MC-FOCUS) |
| 7 | deny, machete; updates | url 2.5.9 GUARD-3 MERGE-OK 49f65382; train118 |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r2 MERGE-OK 37730187 (pr-1..5) |
| 9 | Flakes, window loss | sh-35 leak-cell train119; sh-33/sl-9 closed soak r3; sh-36 closed soak r4; sl-1, sl-2 |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| co-reach-r4.md (rc 1, 2; co-67; 1 %) | sonnet high, claude/co-reach-r4 909bc57b | session_016iS3kQqSPtSAjLg9NseHmk |
| tour-base-r3.md (F7, rc 11; 0.8 %) | sonnet high, claude/tour-base 883214b5 | session_016iS3kLPAifTAZd7mvFfrRp |
| flake-series-r8.md (F5, F4; 1.2 %) | sonnet high, claude/flake-series-r7 6f7bcb26 | session_016iS3kPwWVUYse7EqYxr313 |
| p2-r7-tiling.md (rc 2; 0.8 %) | sonnet medium, claude/p2-r7-tiling ad0093ba | session_016iS3kNwmLnhQzJYTaRrbNv |
| verify-minor-r2.md (rc 8; 0.5 %) | sonnet medium, claude/verify-minor-r2 2235551f | session_016iS3kNQHFDd8Sfe5DbUGAM |
## Queue (% week; 5 % reserve)
0. (rc 8) VERIFY-MINOR r1: 4 fixed (xp-8, mc-9..11); r2 runs for the 10 verify rows.
1. (F4, F5) TRAIN126 guard-4; then train127 co-reach-r3b 909bc57b + tiling-reach 4e257c06; TILING persona r7 runs.
2. (rc 1, 2; 2 %) co-64, co-65 fixed (CO-REACH r3 MERGE-OK, train127 r3b); co-67 MAJOR candidate -> CO-REACH r4, then persona r10; TILING-REACH r1, then persona r7.
3. (rc 3, 9; F5) FLAKE-SERIES r8 (rfs7-1, rfs7-2; sh-38 leak slope).
4. (F7, rc 11; 0.8 %) TOUR-BASE r2 review FIX (rtb2-1 MAJOR: 9 unproven rows as change): r3: --check 19 red on ce866b0d; MERGE-OK -> own train.
5. (F4, F5) GUARD-4 r2 MERGE-OK 284ce9a5: own train126 after train125; then FLAKE-SERIES r8 5/5 on final head (haiku 1 %), TOUR-V9 final take + 5 reviews (2.5 %), CI-PR (rc 4, 5). (rc 1) after reviews: co-44, co-45 only when construct has no open MAJOR (rules.md 6); rtd-5, rtd-7 MINOR backlog.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
