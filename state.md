# State

## Now (2026-11-24 19:47 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 18:58 PM 4.9 % (11-22) + 5.2 % (11-23) + 5.1 % (11-24), week about 31 % vs 51 % pace: under-spending, fill 6 lanes
session branch: c3c37c2e (t132 19:20); 80 greens; t132: tiling-honest r1 (ti-52, ti-53), co-pull r2 (co-70, co-71).
burn-down: audit r6: F1-F4, F6, F8 met; F5 no series; F7 open (v9 take: 82 open, 3 regr)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: xp, mc: xp-46, mc-60 (review r2 FIX rss-7; STRAY-SUPER r3); sh-42 MAJOR new; xp-47 merged t131; tiling: ti-52, ti-53 merged t132; construct: co-75 BLOCK, co-76 MAJOR (rc1-pass r1; CO-CLOSE r1); sh-38 merged t130 |
| 2 | Persona r4 | xp, mc meet; co r9 not met; tiling r8 not met (ti-55 T7, ti-56 L1) |
| 3 | 3 greens, no flake | t130, t131, t132 green; FLAKE-SERIES r9 5/5 on c3c37c2e running |
| 4, 5 | CI green on a PR; packaging | pre-commit green; CI PR at release |
| 6 | Review per area | r1 MERGE-OK (0 MAJOR; MC-FOCUS) |
| 7 | deny, machete; updates | url 2.5.9 GUARD-3 MERGE-OK 49f65382; train118 |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r2 MERGE-OK 37730187 |
| 9 | Flakes, window loss | sh-35 leak-cell train119; sh-33/sl-9 closed soak r3; sh-36 closed soak r4; sl-1, sl-2 |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| tour-base-r6.md (F7; rtb5-1 MAJOR; 1 %) | sonnet high, claude/tour-base-r6 61d0ef7a | session_016iS3kNhSNkSHNMgkjDGCwZ |
| stray-super-r3.md (rc 1; rss-7 MAJOR; 1.5 %) | opus high, claude/stray-super-r3 1eb30643 | session_016iS3kQeFgVrTUeBWbwLj1s |
| flake-series-r9.md (rc 3, F5; 5/5; 0.6 %) | haiku high, claude/flake-series-r9 c3c37c2e | session_016iS3kL6R9aghhfvTsKiyJx |
| tiling-doors-r1.md (rc 2; ti-55, ti-56, ti-50; 1 %) | sonnet high, claude/tiling-doors-r1 b02dbea9 | session_016iS3kLja4YEVy42LaEmr7i |
| sh42-repro-r2.md (rc 1; sh-42 5x; 0.6 %) | sonnet medium, claude/sh42-repro-r2 c3c37c2e | session_016iS3kLEhyJvsbyMbKk9pzN |
| co-close-r1.md (rc 1; co-75 BLOCK, co-76 MAJOR; 1.5 %) | opus high, claude/co-close-r1 280b9dcf | session_016iS3kN4bp8VEeTupxTZ58i |
## Queue (% week; 5 % reserve)
1. (rc 1, 2) TRAIN132 green c3c37c2e (ti-52, ti-53, co-70, co-71). TILING-DOORS r1 running.
2. (rc 1, 2) construct after CO-CLOSE r1: co-79 MAJOR round (+co-77, co-78 MINOR confirmed by CO-REPRO r1). sh-42: SH42-REPRO r2.
3. (rc 3, F5) FLAKE-SERIES r9 5/5 on c3c37c2e running.
4. (F7, rc 11) TOUR-BASE r6 running (rtb5-1..3); then v9 take; rv9j-1..5 MINOR after.
5. (F4, F5) after train127: FLAKE-SERIES r8 5/5 on final head (haiku), TOUR-V9 final take + 5 reviews (2.5 %), CI-PR (rc 4, 5). (rc 1) after reviews: co-44, co-45 only when construct has no open MAJOR (rules.md 6); rtd-5, rtd-7 MINOR backlog.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
