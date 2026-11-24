# State

## Now (2026-11-24 11:58 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 11:58 PM 4.9 % (11-22) + 5.2 % (11-23) + 3.6 % (11-24), week about 28 % vs 47 % pace
session branch: 2235551f (t125 green 07:50: xp-title); 75 greens; t126 red: sh-39 flake (e2e apps-click, alone 3/3); t127 red: xp-38 flake (acts); t127b 11:05 + tour-base, xp-polish.
burn-down: audit r6 01:40: F1-F4, F6, F8 met (F4 no margin); F5 7 rows, no series; F7 open (r4 take stale, 24 acts)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: ti-47, ti-48, sh-38 co-68 (co-67 MERGE-OK) |
| 2 | Persona r4 | xp, mc meet; co r9 not met: co-64, co-65 fixed r3; tiling r7 not met (T7 path, ti-27, ti-50): ti-47/48 train127; TILING-CALM r1 |
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
| co-pull-r1.md (rc 1, 2; co-68 MAJOR; 1 %) | sonnet high, claude/co-pull f1457035 | session_016iS3kQUTaRWZ2y3if8gWjd |
| flake-series-r8.md (F5, F4; 1.2 %) | sonnet high, claude/flake-series-r7 6f7bcb26 | session_016iS3kPwWVUYse7EqYxr313 |
| review-tiling-calm-r1.md (0.5 %) | sonnet high, claude/tiling-calm e9387fb2 | session_016iS3kMBzYfYVkv4YQDUxEQ |
| tiling-flake-r3.md (sh-39, sh-40; 1 %) | sonnet high, claude/tiling-flake-r3 4e257c06 | session_016iS3kPxH8AyTt14UBrf3hi |
| xp-38-r2.md (rc 3; 1 %) | sonnet high, claude/xp-38-r2 2235551f | session_016iS3kLvpV2kdtwhHXTnNRh |
## Queue (% week; 5 % reserve)
0. (rc 8) VERIFY-MINOR r2: 9 of 14 closed; xp-14, xp-15, xp-17 XP-POLISH r1 64478c66 (rxp1, rxp2 MINOR) in t127b; xp-34 known; xp-18 verify.
1. (F4, F5, rc 1) TRAIN127 guard-4, co-reach-r4 f1457035, tiling-reach 4e257c06; TILING-CALM r1 e9387fb2 done, REVIEW r1; then persona r8.
2. (rc 1, 2; 2 %) co-64, co-65, co-67 in train127; persona r10 not met: co-68 MAJOR (pull key told, does nothing) -> CO-PULL r1, then persona r11; TILING-REACH r1, then persona r7.
3. (rc 3, 9; F5) FLAKE-SERIES r8 (rfs7-1, rfs7-2; sh-38 leak slope); TILING-FLAKE r3 (sh-39, sh-40).
4. (F7, rc 11) TOUR-BASE r3 cc71e5d9 (rtb3-1 MINOR) in t127b.
5. (F4, F5) after train127: FLAKE-SERIES r8 5/5 on final head (haiku), TOUR-V9 final take + 5 reviews (2.5 %), CI-PR (rc 4, 5). (rc 1) after reviews: co-44, co-45 only when construct has no open MAJOR (rules.md 6); rtd-5, rtd-7 MINOR backlog.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
