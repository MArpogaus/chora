# State

## Now (2026-11-25 03:51 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC; pace 0.6 %/h = 14 %/day, no day cap; tokens.py 03:50 PM 1.7 % (11-25), week about 34 % vs 55 % pace: under-spending, fill 6 lanes
session branch: 721be592 (t134 03:50, tour-base r7); 82 greens. Lane gates miss the sleep ratchet: harness backlog
burn-down: audit r6: F1-F4, F6, F8 met; F5 no series; F7 open (v9 take: 82 open, 3 regr)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | open: sh-42 (SH42-FIX r2); construct co-75 BLOCK, co-76, co-79 MAJOR (CO-CLOSE r2 + CO-REACH r1 7f519bc8 MERGE-OK; CO-CLOSE-WAITS r1, then train); xp, mc, tiling: none open |
| 2 | Persona r4 | xp, mc meet; co r9 not met; tiling r8 not met (ti-55 T7, ti-56 L1) |
| 3 | 3 greens, no flake | met: t130-t132 green; FLAKE-SERIES r9 5/5 c3c37c2e |
| 4, 5 | CI green on a PR; packaging | pre-commit green; CI PR at release |
| 6 | Review per area | r1 MERGE-OK (0 MAJOR; MC-FOCUS) |
| 7 | deny, machete; updates | url 2.5.9 GUARD-3 MERGE-OK 49f65382; train118 |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r2 MERGE-OK 37730187 |
| 9 | Flakes, window loss | sh-33, sh-35, sh-36, sl-9 closed; sl-1, sl-2 |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| tour-base-r8.md (F7, rc 11; rtb7-1..5; 0.8 %) | sonnet medium, claude/tour-base-r8 721be592 | session_016iS3kPb7eVDhLpTwJfZ2Mc |
| stray-super-minor-r1.md (rc 1, 9; rss-12, rss-13; 0.8 %) | sonnet high, claude/stray-super-minor-r1 84f66d43 | session_016iS3kLMiAHZnqw4HmY1kpE |
| tiling-minor-r1.md (rc 1; rth-1..8, rtd1-2..5; 1.0 %) | sonnet high, claude/tiling-minor-r1 84f66d43 | session_016iS3kNuX9zyEYHgoVk9R8Y |
| sh42-fix-r2.md (rc 1; sh-42; t134 sleep-ratchet; 0.6 %) | sonnet medium, claude/sh42-fix-r2 b92de049+84f66d43 | session_016iS3kPqhV59i3btC3QEanV |
| co-close-waits-r1.md (rc 1; t134 sleeps; 0.5 %) | sonnet medium, claude/co-close-waits-r1 7f519bc8 | session_016iS3kPi6qRiSRUWdnKxSmS |
## Queue (% week; 5 % reserve)
1. (rc 1, 2) TRAIN135: SH42-FIX r2, CO-CLOSE-WAITS r1.
2. (rc 1, 2) construct: CO-REACH r1 7f519bc8 MERGE-OK (holds co-close r2); CO-CLOSE-WAITS r1 then train; co-77, co-78, rcr1-1 MINOR after. sh-42: SH42-FIX r2 then train.
3. (rc 3, F5) met: FLAKE-SERIES r9 5/5 green c3c37c2e; next series on the final head (item 5).
4. (F7, rc 11) t134 merged r7; TOUR-BASE r8 (rtb7-1..5), review, train; then v9 take; rv9j-1..5 MINOR after.
5. (F4, F5) after train127: FLAKE-SERIES r8 5/5 on final head (haiku), TOUR-V9 final take + 5 reviews (2.5 %), CI-PR (rc 4, 5). (rc 1) after reviews: co-44, co-45 only when construct has no open MAJOR (rules.md 6); rtd-5, rtd-7 MINOR backlog.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
