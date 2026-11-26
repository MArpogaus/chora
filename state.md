# State

## Now (2026-11-26 20:32 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC, no day cap; tokens.py 20:32 PM 5.7 % (11-26 BKK); week est. 51 % (unmeasured, owner reads) vs 76 % time; need (100-51-5)/40 h = 1.1 %/h; lanes 6 build
session branch: 0e18f14c (t157 19:53); 101 greens. t153 red sh-37; t154, t155 flake sh-44; t156 flake launcher-mc; t158 runs (h-rf2-r3, mc-minor-r2, kit-swallow-r1, co-reach-r6)

lanes: 4 of 6 (tour-fix-r2, co-minor-r1, review-mcm3-r1, flake-series-r10)
burn-down: F1-4, F6, F8 met; F5 open (reds since r9), r10 runs; F7 open (88,10 regr)

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
| 9 | Flakes, window loss | A13, A17 closed; sl-1, sl-2 open |
| 11 | Tour v9 | open = F7 |

## Lanes
| Lane | Model, branch | State |
|---|---|---|
| tour-fix-r2.md (Q 4, 0.6 %) | sonnet high, 84e31324 | s..LaZa9mxvCe9PHc5BE |
| co-minor-r1.md (Q 2, 0.5 %) | sonnet high, 57ce1480 | s..QfdXoTJwAHB4ThLKn |
| review-mcm3-r1.md (Q 4c, 0.4 %) | opus high, d8eb907c | starting |
| flake-series-r10.md (Q 5, 0.6 %) | haiku high, 4d8bbe36 | s..MrWJ1FZ4X7WVbaJa6 |
## Queue (% week; 5 % reserve)
1. (rc 3) FLAKE-BAR-FULL r1 t152, TOUR-HARNESS r2 t153 merged.
2. (rc 1, 2) co r14: co-75, co-76 5/5; co-79: CO-REACH r6 bcecb671 MERGE-OK, t158; pass r2 57ce1480 confirms rc 1; CO-MINOR r1 runs; r15 NO (co-79 open MAJOR): CO-REACH r7 after co-minor-r1.
3. (rc 2) P2 r12 NO: ti-57; TILING-FOCUS r4 merged t156; pass r2: ti-57 fixed 4/4, but rpt2-1 MAJOR (Super+Alt chords reach clients): KIT-SWALLOW r1 02e745c3 MERGE-OK, t158; r2 bbfcf1c0 MERGE-OK (PM review), t159; then rc1 pass tiling r3; rpt2-2 MINOR backlog.
4. (F7, rc 1, 11) take r2: 88 open, 10 regr: TOUR-ROWS r4 review FIX (rtr4r-1 MAJOR c5d rows; rtr4r-2..4 MINOR): TOUR-FIX r1 t157; TOUR-FIX r2 (rtf1-2, rtr4-1 MAJOR) runs; take r3 after; MC-BOIL r1 t154.
4a. (F7, rc 11) KIT-ICONS r1 merged t155.
4b. (rc 3) H-RF2 r3 in t158.
4c. (rc 1b) XP-MINOR r5 merged t156 (rpx2-5 backlog); mc pass r2 met (MC-MINOR r1 1dd7edc9: -1, -5 fixed, -2, -3 part, -4 kit; r2 in t158; r3 d8eb907c done, review runs; rpm2-6 backlog); take r3 next.
5. (F5, rc 3) FLAKE-SERIES r10 runs on 4d8bbe36 (haiku).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
