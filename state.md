# State

## Now (2026-11-24 07:25 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 06:06 PM 4.9 % (11-22) + 5.2 % (11-23) + 2.1 % (11-24), week about 25 % vs 43 % pace
session branch: ce866b0d (t124 green 04:34: co-reach-r2); 74 greens; train125 running 06:05 (xp-title f65e7f70).
burn-down: audit r6 01:40: F1-F4, F6, F8 met (F4 no margin: W9 779 = RC; GUARD-4); F5 7 rows, no series; F7 open (r4 take stale, 24 acts)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: ti-47, ti-48, sh-38, co-67 (ledger-audit r1: 11 closed); RECORD-V9 |
| 2 | Persona r4 | xp, mc meet; co r9 not met: co-61 ok, co-64, co-65 MAJOR, CO-REACH r3; tiling r6 not met (T7 reach, T6 ti-49 MINOR; L1, L2 hold): ti-47, ti-48 MAJOR, TILING-REACH r1 |
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
| tiling-reach-r1.md (rc 1, 2; 1 %) | sonnet high, claude/tiling-reach | session_016iS3kMq4yU2VPBXarSuEkP 01:52 |
| co-reach-r4.md (rc 1, 2; co-67; 1 %) | sonnet high, claude/co-reach-r4 909bc57b | session_016iS3kQqSPtSAjLg9NseHmk 06:31 |
| tour-base-r3.md (F7, rc 11; 0.8 %) | sonnet high, claude/tour-base 883214b5 | session_016iS3kLPAifTAZd7mvFfrRp 07:18 |
| flake-series-r8.md (F5, F4; 1.2 %) | sonnet high, claude/flake-series-r7 6f7bcb26 | session_016iS3kPwWVUYse7EqYxr313 05:12 |
| verify-minor-r1.md (rc 8; 0.4 %) | haiku medium, claude/verify-minor ce866b0d | starting |
## Queue (% week; 5 % reserve)
1. (F7) TRAIN125 xp-title r1 (rxt-1, rxt-2 MINOR); then train126 guard-4.
2. (rc 1, 2; 2 %) co-64, co-65 fixed (CO-REACH r3 MERGE-OK, train127 r3b); co-67 MAJOR candidate -> CO-REACH r4, then persona r10; TILING-REACH r1, then persona r7.
3. (rc 3, 9; F5) FLAKE-SERIES r7 2/3, PM review FIX (rfs7-1 W9 787, rfs7-2 no red); r8 + sh-38 leak slope.
4. (F7, rc 11; 0.8 %) TOUR-BASE r2 review FIX (rtb2-1 MAJOR: 9 unproven rows as change): r3; sh-31 closed (notify-body r1: already fixed 60a38634); --check 19 red on ce866b0d; MERGE-OK -> own train.
5. (F4, F5) GUARD-4 r2 MERGE-OK 284ce9a5: own train126 after train125; then FLAKE-SERIES r8 5/5 on final head (haiku 1 %), TOUR-V9 final take + 5 reviews (2.5 %), CI-PR (rc 4, 5). (rc 1) after reviews: co-44, co-45 only when construct has no open MAJOR (rules.md 6); rtd-5, rtd-7 MINOR backlog.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
