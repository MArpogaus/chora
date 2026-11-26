# State

## Now (2026-11-26 21:11 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC, no day cap; tokens.py 21:11 PM 5.7 % (11-26 BKK); week est. 51 % (unmeasured, owner reads) vs 76 % time; need (100-51-5)/40 h = 1.1 %/h; lanes 6 build
session branch: 0e18f14c (t157 19:53); 101 greens. t158 runs (h-rf2-r3, mc-minor-r2, kit-swallow-r1, co-reach-r6)

lanes: 5 of 6 (co-minor-r1, rc1-pass-tiling-r3, flake-launcher-r1, tour-rows-r5, flake-gate-r1)
burn-down: F1-4, F6, F8 met; F5 open (r10 4/5), flake-launcher runs; F7 open (24 MINOR regr, 0 open)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | xp, mc met; co met after t158; tiling open |
| 2 | Persona r4 | xp, mc meet; co r15 not met (co-79 pane path: C2 0/4, pull key 0/5); tiling: ti-57 fixed 4/4 (pass r2); rpt2-1 open |
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
| co-minor-r1.md (Q 2, 0.5 %) | sonnet high, 57ce1480 | s..QfdXoTJwAHB4ThLKn |
| rc1-pass-tiling-r3.md (Q 3, 1.2 %) | opus high, 7822a66c | s..M3tsq3SF7RgUqsCwy |
| flake-launcher-r1.md (Q 5, 0.7 %) | sonnet high, 0e18f14c | s..LmRg8qngXai5j1EAo |
| tour-rows-r5.md (Q 4, 0.6 %) | sonnet high, b16c888f | starting |
| flake-gate-r1.md (Q 5, 0.7 %) | sonnet high, 0e18f14c | starting |
## Queue (% week; 5 % reserve)
1. (rc 3) t158 runs; t159: kit-swallow-r2, mc-minor-r3, tour-fix-r2.
2. (rc 1, 2) co r14: co-75, co-76 5/5; co-79: CO-REACH r6 bcecb671 MERGE-OK, t158; pass r2 57ce1480 confirms rc 1; CO-MINOR r1 runs; r15 NO (co-79 open MAJOR): CO-REACH r7 after co-minor-r1.
3. (rc 2) P2 r12 NO: ti-57; TILING-FOCUS r4 merged t156; pass r2: ti-57 fixed 4/4, but rpt2-1 MAJOR (Super+Alt chords reach clients): KIT-SWALLOW r1 02e745c3 MERGE-OK, t158; r2 bbfcf1c0 MERGE-OK (PM review), t159 (with mc-minor-r3); rc1 pass tiling r3 runs on 7822a66c; rpt2-2 MINOR backlog.
4. (F7, rc 1, 11) take r2: 88 open, 10 regr: TOUR-ROWS r4 review FIX (rtr4r-1 MAJOR c5d rows; rtr4r-2..4 MINOR): TOUR-FIX r1 t157; TOUR-FIX r2 b16c888f MERGE-OK (PM review; mc-63 MINOR), t159; 24 regr all MINOR; TOUR-ROWS r5 judges them; take r3 after; MC-BOIL r1 t154.
4c. (rc 1b) XP-MINOR r5 merged t156 (rpx2-5 backlog); mc pass r2 met (MC-MINOR r2 in t158; r3 d8eb907c MERGE-OK (review r3; rmmr3-1 MINOR backlog), t159; rpm2-6 backlog); take r3 next.
5. (F5, rc 3) FLAKE-SERIES r10 4/5 (sh-47, launcher family): FLAKE-LAUNCHER r1, FLAKE-GATE r1 (sh-41,43,45,46) run; then r11.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
