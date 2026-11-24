# State

## Now (2026-11-24 17:37 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 17:37 PM 4.9 % (11-22) + 5.2 % (11-23) + 5.1 % (11-24), week about 31 % vs 51 % pace: under-spending, fill 6 lanes
session branch: 73925ee2 (t131 17:30); 79 greens; t130+t131: xp-38 r2, flake-series r8, tiling-flake r3, tour-base r4, tiling-calm r2, helper-greet r1.
burn-down: audit r6: F1-F4, F6, F8 met; F5 no series; F7 open (v9 take: 82 open, 3 regr)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: xp, mc: xp-46, mc-60 (STRAY-SUPER r2); xp-47 merged t131; tiling: ti-52, ti-53 in train132; sh-38 merged t130 |
| 2 | Persona r4 | xp, mc meet; co r9 not met; tiling r8 not met (ti-55 T7, ti-56 L1; ti-27, ti-47 fixed) |
| 3 | 3 greens, no flake | r8 merged t130; t130, t131 green; 5/5 series after train132 |
| 4, 5 | CI green on a PR; packaging | pre-commit green; CI PR at release |
| 6 | Review per area | r1 MERGE-OK (0 MAJOR; MC-FOCUS) |
| 7 | deny, machete; updates | url 2.5.9 GUARD-3 MERGE-OK 49f65382; train118 |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r2 MERGE-OK 37730187 |
| 9 | Flakes, window loss | sh-35 leak-cell train119; sh-33/sl-9 closed soak r3; sh-36 closed soak r4; sl-1, sl-2 |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| tour-base-r5.md (F7; rv9j-4, rv9j-5, rtb4-1; 1 %) | sonnet high, claude/tour-base-r5 d0801b7e | session_016iS3kQ1eXAcuS9WmgxHfSS |
| stray-super-r2.md (rc 1; rss-1, rss-2 MAJOR; 1.5 %) | opus high, claude/stray-super-r2 e1183766 | session_016iS3kM7GXtGirZm9qkxpMf |
| tiling-doors-r1.md (rc 2; ti-55, ti-56, ti-50; 1 %) | sonnet high, claude/tiling-doors-r1 b02dbea9 | session_016iS3kLja4YEVy42LaEmr7i |
## Queue (% week; 5 % reserve)
0. (rc 8) VERIFY-MINOR r2 12/14
1. (rc 1, 2) TRAIN131 green 73925ee2. TRAIN132 running: tiling-honest r1 b02dbea9 (MERGE-OK 16:59), co-pull r2 280b9dcf (MERGE-OK 17:23). (rc 2) TILING-DOORS r1 running.
2. (rc 2) CO-PULL r1 merged t129; r11-co not met (C2: co-70 MAJOR, co-71); CO-PULL r2 MERGE-OK (train132), then r12-co.
3. (rc 3, F5) FLAKE-SERIES r8, TILING-FLAKE r3 merged t130; next: haiku 5/5 series on the train132 head.
4. (F7, rc 11) JUDGE r1: no new BLOCK/MAJOR; rv9j-1..5 MINOR; open stays until rows prove: TOUR-BASE r5; then the v9 take again.
5. (F4, F5) after train127: FLAKE-SERIES r8 5/5 on final head (haiku), TOUR-V9 final take + 5 reviews (2.5 %), CI-PR (rc 4, 5). (rc 1) after reviews: co-44, co-45 only when construct has no open MAJOR (rules.md 6); rtd-5, rtd-7 MINOR backlog.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
