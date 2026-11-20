
## r1

Head: claude/co-hud-load 06ea05e4 (fix on 618ae513). No red check without load: the fault needs a redraw after the fade time.

Cause: Tab does not leak. Under 8 busy loops, before Tab the construct held the focus (holder construct), and after Tab the state showed construct-list open. The list node sat at opacity 0.976 / 0.998 for 4 s while the gate waited for 1.0. A redraw of the list (row hover) after FADE_IN (160 ms wall) sends no new fade, and its Update stops the running scene animation (scene.rs:752) short of 1. Under load the animation lags wall time. co-29 fixed only the redraw within the fade time.

Fix: overlay.rs draw_list sets opacity 1.0 when the fade time is over. Red unit test: a_row_hovered_after_the_fade_time_leaves_the_list_fully_opaque (0.99976 before).

Load runs (gate-construct-hud dark 1280x720@1.5): 2 busy loops 5/5 green (no repro); 8 busy loops before: 3/3 red; after fix 5/5 green.

Merge gate --jobs 1: pre-commit, ci-check, build, test, clippy, gate-lib, construct-hud, construct-extra, cube, pull, placed, wm-pane all PASS.

## PM check r1 (23:58 UTC; PM not the builder; diff 618ae513..06ea05e4, 2 files, +43)
- ch-1 MINOR: test and fix in one commit (06ea05e4); red-first passes ("a test before it or in it"), but the unit test's red at 618ae513 is not shown on its own; the lane's 3/3 red under load at 618ae513 stands in for it.
- Checked: overlay.rs draw_list, past the fade time, now sets opacity 1.0 through panes::update (before, no command, so an update stopped the running fade short of 1); the new test moves `opened` back 200 ms and asserts opacity 1.0 after a late hover; no other path changed.
Verdict (computed): no BLOCK or MAJOR: MERGE-OK, head 06ea05e4, joins train 95; co-35 fixed when train 95 is green.
