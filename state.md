# State

## Now (2026-11-28 11:37 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-28 04:00 to 12-05 03:59 UTC, no day cap; tokens.py 11:37 PM 0 % (reset); last week 100 % 28 h early; need 95 %/168 h = 0.57 %/h; lanes 5, opus only reviews
session branch: 3098926e (t162 green); 106 greens

lanes: 4 of 6, t163 (co-minor-r3, flake-xp-r2, tiling-minor-r3, review-mc-minor-r6); flake r11 after flake-xp
burn-down: F1-4, F6, F8 met; F5 open (r10 4/5), flake-xp r2, flake-gate merged; F7 open (take r3: 5 regr, 0 open; 12 red co-44)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | xp, mc, co met; tiling met (pass r3; rpt2-1 fix in t159) |
| 2 | Persona r4 | xp, mc meet; co r15 not met (co-79 pane path: C2 0/4, pull key 0/5); tiling: ti-57, rpt2-1 fixed (pass r3) |
| 3 | 3 greens, no flake | open: reds t141, t142, t148, t153 (sh-37); FLAKE-SERIES r10 reruns |
| 4, 5 | CI on a PR; packaging | CI PR at release |
| 6 | Review per area | r1 MERGE-OK (shas: reports/review-*.md) |
| 7 | deny, machete; updates | GUARD-3 49f65382 (t118) |
| 8 | Docs, CHANGELOG, known issues | DOCS-RC r2 MERGE-OK 37730187 |
| 9 | Flakes, window loss | sl-1, sl-2 open |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| co-minor-r3.md (Q 2, 0.6 %) | sonnet high, 830ab836 | s..NHGpKjWnQJ2BEVYY9 |
| flake-xp-r2.md (Q 5, 0.6 %) | sonnet high, 7fc05748 | s..QqGiX8M41oLod17kM |
| tiling-minor-r3.md (Q 3, 0.5 %) | sonnet high, ad07ec70 | s..M6wS9edrNDs7nJTHu |
| review-mc-minor-r6.md (Q 4c, 0.5 %) | opus high, e7e4bca0 | starting |
## Queue (% week; 5 % reserve)
1. (rc 3) t162 green 3098926e (tour-rows-r5, tiling-minor-r1, flake-gate-r1).
2. (rc 1, 2) co-79: r15 NO (co-79 open MAJOR): CO-REACH r7 008714ba review FIX: CO-REACH r8 830ab836 review MERGE-OK (rcrr8-1,-2 MINOR, -3 NIT): t164; CO-MINOR r3 (rcrr8-1..3) runs; then co persona r16.
3. (rc 2) rpt2-1 MAJOR: KIT-SWALLOW r1 merged t158; r2 bbfcf1c0 merged t159; pass r3 confirms rc 1 (rpt2-1 fixed 3/3); TILING-MINOR r1 merged t162 (rtmr1-1..3 backlog); TILING-MINOR r2 ad07ec70 review MERGE-OK: t163 runs; TILING-MINOR r3 (rtmr2-1..5) runs.
4. (F7, rc 1, 11) take r2: 88 open, 10 regr; TOUR-FIX r1 t157, r2 t159; TOUR-ROWS r5 t162; TAKE r3: 5 regr; TOUR-ROWS r6 7b09b623 review MERGE-OK (0 regr; rtr6r-1..5 NIT, rtr4-2 closed): t164 after t163; then TAKE r4.
4c. (rc 1b) XP-MINOR r5 merged t156 (rpx2-5 backlog); mc pass r2 met (MC-MINOR r2 merged t158; r3 merged t159; rpm2-6 backlog); MC-MINOR r6 e7e4bca0 done (mc-63, rpx1-17 closed; rpx1-16 spelling closed, pill open: host): review r6 runs.
5. (F5, rc 3) FLAKE-SERIES r10 4/5 (sh-47, launcher family): FLAKE-LAUNCHER r1 merged t161; FLAKE-GATE r1 merged t162 (sh-41, sh-46 fixed); FLAKE-XP r1 7fc05748 review FIX (rfxr1-1 MAJOR construct drag; -2,-3 MINOR, -4 NIT): FLAKE-XP r2 runs; then r11.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
