# State

## Now (2026-11-24 09:32 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 08:18 PM 4.9 % (11-22) + 5.2 % (11-23) + 2.9 % (11-24), week about 26 % vs 45 % pace
session branch: 2235551f (t125 green 07:50: xp-title); 75 greens; t126 red: sh-39 flake (e2e apps-click, alone 3/3); train127 09:03 guard-4+co-reach-r4+tiling-reach.
burn-down: audit r6 01:40: F1-F4, F6, F8 met (F4 no margin: W9 779 = RC; GUARD-4); F5 7 rows, no series; F7 open (r4 take stale, 24 acts)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: ti-47, ti-48, sh-38 (co-67 MERGE-OK) |
| 2 | Persona r4 | xp, mc meet; co r9 not met: co-64, co-65 fixed r3; tiling r7 not met (T7 path, ti-27, ti-50): ti-47/48 train127; TILING-CALM r1 |
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
| p2-r10-co.md (rc 2; co-64/65/67; 0.8 %) | sonnet medium, claude/p2-r10-co f1457035 | session_016iS3kQimoWgLMfuE8TZQyN |
| review-tour-base-r3.md (F7; 0.5 %) | sonnet high, claude/tour-base cc71e5d9 | starting |
| flake-series-r8.md (F5, F4; 1.2 %) | sonnet high, claude/flake-series-r7 6f7bcb26 | session_016iS3kPwWVUYse7EqYxr313 |
| tiling-calm-r1.md (rc 2, 8; 1 %) | sonnet high, claude/tiling-calm ad0093ba | session_016iS3kLdvEtaUtMWPs87vDJ |
| xp-polish-r1.md (rc 8; 1 %) | sonnet high, claude/xp-polish 2235551f | session_016iS3kNmUxToFmubQNgEQYh |
## Queue (% week; 5 % reserve)
0. (rc 8) VERIFY-MINOR r2: 9 of 14 closed; xp-14, xp-15, xp-17 -> XP-POLISH r1; xp-34 known; xp-18 verify.
1. (F4, F5, rc 1) TRAIN127 guard-4, co-reach-r4 f1457035, tiling-reach 4e257c06; TILING-CALM r1 (ti-27, ti-50), then persona r8.
2. (rc 1, 2; 2 %) co-64, co-65 fixed (CO-REACH r3 MERGE-OK, train127 r3b); co-67 fixed r4 MERGE-OK (train127); next persona r10; TILING-REACH r1, then persona r7.
3. (rc 3, 9; F5) FLAKE-SERIES r8 (rfs7-1, rfs7-2; sh-38 leak slope).
4. (F7, rc 11; 0.8 %) TOUR-BASE r2 review FIX (rtb2-1 MAJOR: 9 unproven rows as change): r3 done cc71e5d9 (gate 56/1 foreign xp-38); REVIEW r3; MERGE-OK -> train.
5. (F4, F5) after train127: FLAKE-SERIES r8 5/5 on final head (haiku 1 %), TOUR-V9 final take + 5 reviews (2.5 %), CI-PR (rc 4, 5). (rc 1) after reviews: co-44, co-45 only when construct has no open MAJOR (rules.md 6); rtd-5, rtd-7 MINOR backlog.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
