# State

## Now (2026-11-26 19:06 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-21 04:00 to 11-28 03:59 UTC, no day cap; tokens.py 19:06 PM 5.7 % (11-26 BKK); week est. 51 % (unmeasured, owner reads) vs 76 % time; need (100-51-5)/40 h = 1.1 %/h; lanes 6 build
session branch: 7c6955c8 (t156 17:27); 100 greens. t153 red sh-37; t154, t155 flake sh-44; t156 flake launcher-mc (mc dark), rerun green; t157 runs

lanes: 6 of 6 (tour-fix-r2, window-loss-r1, kit-swallow-r2, rc1-pass-construct-r2, mc-minor-r3, flake-series-r10)
burn-down: F1-4, F6, F8 met; F5 open (reds since r9), r10 runs; F7 open (88 open, 10 regr)

## RC
| # | Criterion | State |
|---|---|---|
| 1, 1b | No BLOCK/MAJOR, independent pass; owner sees v9 | xp, mc pass r2 met; tiling, co open |
| 2 | Persona r4 | xp, mc meet; co r14 not met (co-79 Terminal-row pane path 0/5); tiling: ti-57 fixed 4/4 (pass r2); rpt2-1 open |
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
| tour-fix-r2.md (Q 4, 0.6 %) | sonnet high, 84e31324 | s..LaZa9mxvCe9PHc5BE |
| window-loss-r1.md (rc 9, 0.4 %) | haiku high, 7c6955c8 | s..LJCa3VLviJj9SXMTE |
| kit-swallow-r2.md (Q 3, 0.4 %) | sonnet high, 02e745c3 | s..N37HHuzX8ujDusKFR |
| rc1-pass-construct-r2.md (Q 2, 1.5 %) | opus high, 57ce1480 | starting |
| mc-minor-r3.md (Q 4c, 0.5 %) | sonnet high, 59dd6894 | s..NjzArxCW9sJ9LRKQ1 |
| flake-series-r10.md (Q 5, 0.6 %) | haiku high, 4d8bbe36 | s..MrWJ1FZ4X7WVbaJa6 |
## Queue (% week; 5 % reserve)
1. (rc 3) FLAKE-BAR-FULL r1 t152, TOUR-HARNESS r2 t153 merged.
2. (rc 2) construct r14 not met: co-75, co-76 hold 5/5; co-79: CO-REACH r6 bcecb671 MERGE-OK, t158; rc1 pass co r2 runs on 57ce1480.
3. (rc 2) P2 r12 NO: ti-57; TILING-FOCUS r4 merged t156; pass r2: ti-57 fixed 4/4, but rpt2-1 MAJOR (Super+Alt chords reach clients): KIT-SWALLOW r1 02e745c3 MERGE-OK, t158; r2 (rksr1-1..3) starts; rpt2-2 MINOR backlog.
4. (F7, rc 1, 11) take r2: 88 open, 10 regr; TOUR-ROWS r4 review FIX (rtr4r-1 MAJOR c5d rows; rtr4r-2..4 MINOR): TOUR-FIX r1 MERGE-OK, in t157; TOUR-FIX r2 (rtf1-2, rtr4-1 MAJOR) runs; take r3 after; MC-BOIL r1 merged t154.
4a. (F7, rc 11) KIT-ICONS r1 merged t155.
4b. (rc 3) H-RF2 r3 4b0d327a MERGE-OK (PM), t158 first; rc1 pass co next.
4c. (rc 1b) XP-MINOR r5 merged t156 (rpx2-5 backlog); mc pass r2 met (MC-MINOR r1 1dd7edc9: -1, -5 fixed, -2, -3 part, -4 kit; r2 59dd6894 MERGE-OK, next train; r3 (rmmr2-1) runs; rpm2-6 backlog); take r3 next.
5. (F5, rc 3) FLAKE-SERIES r10 runs on 4d8bbe36 (haiku).

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
