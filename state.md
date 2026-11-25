# State

## Now (2026-11-25 02:08 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 00:16 PM 5.1 % (11-24) + 1.3 % (11-25), week about 33 % vs 54 % pace: under-spending, fill 6 lanes after t133
session branch: 84f66d43 (t133 00:20); 81 greens; t133: stray-super r3, tiling-doors r1 (mc-60, xp-46, ti-50, ti-55, ti-56). t134 stopped: sleep-ratchet 783 vs 779 (sh42 +2, co-close +3)
burn-down: audit r6: F1-F4, F6, F8 met; F5 no series; F7 open (v9 take: 82 open, 3 regr)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: xp, mc: xp-46, mc-60 merged t133; sh-42 (r1 MERGE-OK; t134 sleep breach, SH42-FIX r2); tiling: ti-52, ti-53 t132; construct: co-75 BLOCK, co-76 MAJOR (CO-CLOSE r2 fbdcd6e2 MERGE-OK, train134); co-79 MAJOR (CO-REACH r1) |
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
| review-tour-base-r7.md (F7; 0.5 %) | sonnet high, r7 head 7c2cdc95 | session_016iS3kMVLpZ99tLoguGVStK |
| sh42-fix-r2.md (rc 1; sh-42; t134 sleep-ratchet; 0.6 %) | sonnet medium, claude/sh42-fix-r2 b92de049+84f66d43 | session_016iS3kPqhV59i3btC3QEanV |
| flake-series-r9.md (rc 3, F5; 5/5; 0.6 %) | haiku high, claude/flake-series-r9 c3c37c2e | session_016iS3kL6R9aghhfvTsKiyJx |
| co-reach-r1.md (rc 1, 2; co-79 MAJOR; 1.2 %) | opus high, claude/co-reach-r1 fbdcd6e2 | session_016iS3kLNYWe7A8GuMJ46b8V |
## Queue (% week; 5 % reserve)
1. (rc 1, 2) TRAIN133 green 84f66d43. TRAIN134: SH42-FIX r2 (sleeps to waits) then train; co-close joins co-reach's train (co-reach r2 drops its 3 sleeps).
2. (rc 1, 2) construct: CO-CLOSE r2 MERGE-OK (train134); CO-REACH r1 co-79 running; co-77, co-78 MINOR after. sh-42: SH42-FIX r1 b92de049 MERGE-OK (rsf1-1..3 MINOR), train134 after t133.
3. (rc 3, F5) FLAKE-SERIES r9 5/5 on c3c37c2e running.
4. (F7, rc 11) TOUR-BASE r7 7c2cdc95 done (rtb6-1 strict default; gate in pieces); REVIEW r7 running; then train, v9 take; rv9j-1..5 MINOR after.
5. (F4, F5) after train127: FLAKE-SERIES r8 5/5 on final head (haiku), TOUR-V9 final take + 5 reviews (2.5 %), CI-PR (rc 4, 5). (rc 1) after reviews: co-44, co-45 only when construct has no open MAJOR (rules.md 6); rtd-5, rtd-7 MINOR backlog.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
