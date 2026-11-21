# flakes-q (F5: no flaky tests)

## r1

Lane: cloud, branch `claude/flakes-q` from `618ae513` (train 94b); head `fb85b156` (pushed). Written 11-21.
Step 0: `git fetch --unshallow`, `cargo check --workspace` exit 0. Load: 2 busy loops (6 for the second xp
count), a second `cargo build -p chora-kit` loop in its own target dir (clean each round), and 3 or 4
parallel streams of the cell, load average 5 to 8 on 4 cores. Logs are under `/tmp/claude-0` of the
container only.

| Item | Result | Commits | Loaded runs after the fix |
|---|---|---|---|
| 1. FLK-HUD-TAB | cause shown, fixed | diagnostic `e830d569`, red `ec5c52b3`, fix `26ad8411` | cell 21 of 21 (3 streams of 7); before: 3 of the first 6 red |
| 2. FLK-XP-FACES | not reproduced, evidence only | diagnostic `fb85b156` | 45 of 45 (21 at 2 busy loops, 24 at 6); gate 3 of 3 |

### 1. FLK-HUD-TAB
Diagnostic `e830d569`: a failed Tab (both lines, `light-...: Tab opened no list`) now says which nodes hold
the focus, the state of `construct-list` (opacity, visible), the surfaces and the tail of the session log.
Reproduction: the first loaded run of the cell (light-1280x720@1.5 only, 3 streams, 2 busy loops, build loop)
was red in 3 of the first 6 runs, every one the second Tab (after the long-title window), every one the same
state: `focused ['construct-look']` (the key went to the look, as it should), the `construct-list` node
visible with opacity **0.99989**, **0.99804**, and one more partial value, not 1.0. The test waits for
exactly 1.0 for 10 s, and the opacity did not move in that time.
Cause: the fade is not slow, it is cut. For the `EaseOut` curve (`1 - (1-t)^3`) the two values are the
fade at 95 % (152 of 160 ms) and at 87.5 % (140 of 160 ms). `draw_list(false)` (a redraw of the open list,
here the hover of a row under a pointer that stood still) carries the fade on for `left = FADE_IN - opened`
only while `left > 0` by the plugin's clock, and otherwise sends only the `update` of the list. In the scene
`apply` removes the running animation of a node for any update (`scene.rs:752`, `animations.remove`), and an
update without opacity leaves the opacity at the value of that moment. The core starts and steps the fade
later than the plugin counts it under load (the animation starts when the core applies the commands and
advances at loop turns), so a redraw can come when the plugin says 160 ms are over and the core's fade has 8
to 20 ms left: the fade ends there for good. This is the co-29 case (`a_row_hovered_while_the_list_fades_in...`,
which carries the fade for the time left) with the clock of the plugin ahead of the clock of the core.
Red `ec5c52b3`: `a_redraw_after_the_fade_time_leaves_a_late_list_fully_opaque` backdates `opened` by 200 ms,
steps the core's scene by 140 ms, hovers a row, steps by 1000 ms: opacity 0.998046875 (the failing run read
0.998044).
Fix `26ad8411`: when the redraw has no fade to carry on (`!open && left <= 0`), its `update` sets opacity
1.0. No sleep, no retry; the list is at its target whatever the clocks say. Product change of one line in
`plugins/chora-construct/src/overlay.rs`. `cargo test -p chora-construct` 261 of 261.
Loaded after the fix: the cell 21 of 21 in three streams (93 to 98 s each at load 5 to 8).

### 2. FLK-XP-FACES
Not reproduced in 45 loaded runs of the whole cell (21 in 3 streams at 2 busy loops, 24 in 4 streams at 6
busy loops, the build loop beside; load average up to 8.3) and 3 of 3 in the merge gate at --jobs 3. The
backlog line (one red in a full gate at --jobs 2, 0 of 10 alone) stays open on evidence only.
Diagnostic `fb85b156`: a failed `opens-under-pointer` now says the time of the injected pointer move to
960,200, the time the window showed, the time the cell gave up, the window node (owner, parent, focused,
rects per output), `render.stats` and the tails of `pane-host.log` and `xp.log`, so the next sighting tells a
window on the wrong face (rects name another output) from one that is not focused or has no rect yet.
Looked at, not shown: xp opens a window on `home_bar(pointer_output)`, the output of the last
`pointer.entered` node (`wm/windows.rs:551`), so the cell passes only if xp heard the entered event before the
window's `created`. Freezing the pane host (`SIGSTOP`) around the injected move, with the window shown a
second later, still passed in one run, so a late pane host is not that path. Nothing is changed in the cell
or in xp.

## Gate
`tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,gate-lib,construct-hud,xp-faces`
at `fb85b156` (PATH with ~/.local/bin): pre-commit 3 s, gate-lib 89 s, ci-check 93 s, build 27 s, test 215 s,
clippy 17 s, xp-faces 15 s, construct-hud 112 s: all PASS. Then `--only construct-hud,xp-faces` twice more at
--jobs 3: xp-faces 17 s and 15 s, construct-hud 113 s and 112 s: PASS.

## Open
- FLK-XP-FACES: evidence only (0 of 45 loaded, 3 of 3 gate); the next red names the face of the window.
- The pre-commit hook of this repository refuses a new comment in a test script, so the `hud_state` and
  `opens_state` helpers carry no header comment; their names say the case.
