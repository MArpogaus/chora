# State

## Now (2026-11-28 07:45 UTC)
pm: session_016iS3kLFcYmUQ3S8Zp7sDaN
budget: week 11-28 04:00 to 12-05 03:59 UTC, no day cap; tokens.py 07:45 PM 0 % (reset); last week hit 100 % at 11-27 00:16, 28 h early; need 95 %/168 h = 0.57 %/h; lanes 5, opus only co-reach
session branch: 3098926e (t162 green); 106 greens

lanes: 4 of 6 (co-reach-r7, flake-xp-r1, tiling-minor-r2, take-r3); flake r11 after flake-xp
burn-down: F1-4, F6, F8 met; F5 open (r10 4/5), flake-xp runs, flake-gate merged; F7 open (24 MINOR regr, 0 open)

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
| co-reach-r7.md (Q 2, 1 %) | opus high, dc532d60 | s..Lt3YWo6KSF9sma4g1 |
| tiling-minor-r2.md (Q 3, 0.5 %) | sonnet high, bd641acb | s..e5m7T8bV2j2Lb7at |
| flake-xp-r1.md (Q 5, 0.7 %) | sonnet high, 8b6f35dc | s..kL3XGUoheq6ZEp6h9E |
| tour-v9-take-r3.md (Q 4, 1 %) | sonnet medium, 3098926e | starting |
## Queue (% week; 5 % reserve)
1. (rc 3) t162 green 3098926e (tour-rows-r5, tiling-minor-r1, flake-gate-r1). Limit stop 11-27 00:16 to 11-28 04:00.
2. (rc 1, 2) co r14: co-75, co-76 5/5; co-79: CO-REACH r6 bcecb671 merged t158; pass r2 57ce1480 confirms rc 1; CO-MINOR r1 dc532d60 merged t160 (rcmr1-1 MINOR, -2 NIT backlog); r15 NO (co-79 open MAJOR): CO-REACH r7 runs.
3. (rc 2) rpt2-1 MAJOR: KIT-SWALLOW r1 merged t158; r2 bbfcf1c0 merged t159; pass r3 confirms rc 1 (rpt2-1 fixed 3/3); TILING-MINOR r1 merged t162 (rtmr1-1..3 backlog); TILING-MINOR r2 (rtmr1-1..3) runs.
4. (F7, rc 1, 11) take r2: 88 open, 10 regr: TOUR-ROWS r4 review FIX (rtr4r-1 MAJOR c5d rows; rtr4r-2..4 MINOR): TOUR-FIX r1 t157; TOUR-FIX r2 merged t159; TOUR-ROWS r5 merged t162 (regr 3: c5d, c5e, m8); TAKE r3 runs; MC-BOIL r1 t154.
4c. (rc 1b) XP-MINOR r5 merged t156 (rpx2-5 backlog); mc pass r2 met (MC-MINOR r2 merged t158; r3 merged t159; rpm2-6 backlog); take r3 next.
5. (F5, rc 3) FLAKE-SERIES r10 4/5 (sh-47, launcher family): FLAKE-LAUNCHER r1 merged t161; FLAKE-GATE r1 merged t162 (sh-41, sh-46 fixed; sh-43, sh-45 open; rfgr1-1 NIT); FLAKE-XP r1 (sh-44, sh-47) runs; then r11.

## Needs owner
- Delete claude/release-rc4 (403).
- rc.4 tag: kept local (owner).
