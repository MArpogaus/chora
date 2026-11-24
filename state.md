# State

## Now (2026-11-24 21:33 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 20:13 PM 5.2 % (11-23) + 5.1 % (11-24) + 0.8 % (11-25), week about 32 % vs 53 % pace: under-spending, fill 6 lanes
session branch: c3c37c2e (t132 19:20); 80 greens; t132: tiling-honest r1 (ti-52, ti-53), co-pull r2 (co-70, co-71).
burn-down: audit r6: F1-F4, F6, F8 met; F5 no series; F7 open (v9 take: 82 open, 3 regr)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: xp, mc: xp-46, mc-60 (r3 MERGE-OK; train133); sh-42 MAJOR 5/5 (SH42-FIX r1); xp-47 merged t131; tiling: ti-52, ti-53 merged t132; construct: co-75 BLOCK, co-76 MAJOR (r1 review FIX: rcc1-1 MAJOR; CO-CLOSE r2); sh-38 merged t130 |
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
| train133 (rc 1, 2; stray-super r3 73c6b19c, tiling-doors r1 b1ba6743) | PM, 84f66d43 | gate running |
| flake-series-r9.md (rc 3, F5; 5/5; 0.6 %) | haiku high, claude/flake-series-r9 c3c37c2e | session_016iS3kL6R9aghhfvTsKiyJx |
| sh42-fix-r1.md (rc 1; sh-42 MAJOR; 1.5 %) | opus high, claude/sh42-fix-r1 c3c37c2e | session_016iS3kMHuyqcookRmRRXXJB |
| co-close-r2.md (rc 1; rcc1-1 MAJOR; 0.8 %) | sonnet high, claude/co-close-r2 8517022f | starting |
## Queue (% week; 5 % reserve)
1. (rc 1, 2) TRAIN132 green c3c37c2e (ti-52, ti-53, co-70, co-71). TRAIN133 84f66d43 (stray-super r3, tiling-doors r1) running.
2. (rc 1, 2) construct after CO-CLOSE r1: co-79 MAJOR round (+co-77, co-78 MINOR confirmed by CO-REPRO r1). sh-42: SH42-FIX r1 (repro r2 5/5).
3. (rc 3, F5) FLAKE-SERIES r9 5/5 on c3c37c2e running.
4. (F7, rc 11) TOUR-BASE r6 running (rtb5-1..3); then v9 take; rv9j-1..5 MINOR after.
5. (F4, F5) after train127: FLAKE-SERIES r8 5/5 on final head (haiku), TOUR-V9 final take + 5 reviews (2.5 %), CI-PR (rc 4, 5). (rc 1) after reviews: co-44, co-45 only when construct has no open MAJOR (rules.md 6); rtd-5, rtd-7 MINOR backlog.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
