# State

## Now (2026-11-24 16:51 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 15:41 PM 4.9 % (11-22) + 5.2 % (11-23) + 4.8 % (11-24), week about 30 % vs 50 % pace
session branch: 6d336bdf (t129 14:20); 77 greens; t130 14:25: xp-38 r2, flake-series r8, tiling-flake r3, tour-base r4.
burn-down: audit r6: F1-F4, F6, F8 met; F5 no series; F7 open (v9 take: 82 open, 3 regr)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: rc1-pass r1 xp, mc not confirmed (xp-46, mc-60 MAJOR; xp-47 MERGE-OK); tiling not confirmed (ti-52, ti-53 MAJOR); sh-38 |
| 2 | Persona r4 | xp, mc meet; co r9 not met; tiling r8 not met (ti-55 T7, ti-56 L1; ti-27, ti-47 fixed) |
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
| review-tiling-honest-r1.md (rc 1; ti-52, 53; 0.5 %) | opus high, claude/tiling-honest-r1 b02dbea9 | starting |
| tour-base-r5.md (F7; rv9j-4, rv9j-5, rtb4-1; 1 %) | sonnet high, claude/tour-base-r5 d0801b7e | session_016iS3kQ1eXAcuS9WmgxHfSS |
| stray-super-r2.md (rc 1; rss-1, rss-2 MAJOR; 1.5 %) | opus high, claude/stray-super-r2 e1183766 | session_016iS3kM7GXtGirZm9qkxpMf |
## Queue (% week; 5 % reserve)
0. (rc 8) VERIFY-MINOR r2 12/14
1. (F4, F5, rc 1, 3) TRAIN131 = t130 (mc-scale 5101 ms flake, sh-37; 6 reruns green) + tiling-calm r2 + helper-greet r1, running. (rc 2) TILING-DOORS r1 (plan fca236a9 accepted) on the TILING-HONEST r1 head.
2. (rc 2) CO-PULL r1 merged t129; r11-co not met (C2: co-70 MAJOR, co-71); CO-PULL r2, then r12-co.
3. (rc 3, 9; F5) FLAKE-SERIES r8 (rfs7-1, rfs7-2; sh-38 leak slope); TILING-FLAKE r3 MERGE-OK (sh-39, sh-40).
4. (F7, rc 11) JUDGE r1: no new BLOCK/MAJOR; rv9j-1..5 MINOR; open stays until rows prove: TOUR-BASE r5; then the v9 take again.
5. (F4, F5) after train127: FLAKE-SERIES r8 5/5 on final head (haiku), TOUR-V9 final take + 5 reviews (2.5 %), CI-PR (rc 4, 5). (rc 1) after reviews: co-44, co-45 only when construct has no open MAJOR (rules.md 6); rtd-5, rtd-7 MINOR backlog.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
