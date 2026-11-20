
## r1

Head: claude/co-hud-load 06ea05e4 (fix on 618ae513). No red check without load: the fault needs a redraw after the fade time.

Cause: Tab does not leak. Under 8 busy loops, before Tab the construct held the focus (holder construct), and after Tab the state showed construct-list open. The list node sat at opacity 0.976 / 0.998 for 4 s while the gate waited for 1.0. A redraw of the list (row hover) after FADE_IN (160 ms wall) sends no new fade, and its Update stops the running scene animation (scene.rs:752) short of 1. Under load the animation lags wall time. co-29 fixed only the redraw within the fade time.

Fix: overlay.rs draw_list sets opacity 1.0 when the fade time is over. Red unit test: a_row_hovered_after_the_fade_time_leaves_the_list_fully_opaque (0.99976 before).

Load runs (gate-construct-hud dark 1280x720@1.5): 2 busy loops 5/5 green (no repro); 8 busy loops before: 3/3 red; after fix 5/5 green.

Merge gate --jobs 1: pre-commit, ci-check, build, test, clippy, gate-lib, construct-hud, construct-extra, cube, pull, placed, wm-pane all PASS.
