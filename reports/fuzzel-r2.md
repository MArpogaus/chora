# fuzzel r2: report

Result: ti-1 is not a fault of chora. fuzzel 1.11.1 needs two pointer motions before it selects a row, and the r1 cell moved to the wrong row with one move. With three moves and the right row the click starts the clicked row: tests/gate-tiling-apps-click.sh passes 3/3 on claude/fuzzel-click, no wait in the cell.

## Items, earliest gate each
1. Red cell: 9d6331e (tests/gate-tiling-apps-click.sh, one move to y 496, click). Red 3/3 on fuzzel 1.11.1 (earliest gate: the cell itself).
2. Cause, from a WAYLAND_DEBUG=1 of the real fuzzel in the run (wrapper in PATH, stderr to a file):
   - r1 clicked y 496. A capture of the list (8 rows: alpha, Chat with a helper, Dark look, Light look, Screenshot, Screenshot of an area, System look, Zebra) puts System look at y 496 and Zebra at y 533. The r1 "Zebra row" was System look.
   - fuzzel 1.11.1 wayland.c `wl_pointer_enter` (1066-1090) does not store the position ("do *not* set seat->pointer.{x,y} here"). `wl_pointer_motion` (1104-1119) stores x, y and calls `select_hovered_match` only when the old x and y are non-zero, so the first motion only starts and the second selects. `wl_pointer_button` (1123-1135) on release runs `execute_selected`, the selection, which stays the first row until a selecting motion.
   - Log, one pointer_move: `enter(215, 326)` then `button`, no motion; first row runs. Two moves: `enter` then one `motion`; first row runs (still the start motion). Three moves: `enter`, two `motion`, button; zebra starts.
   - `wl_pointer.frame` is not the cause: fuzzel binds wl_seat at version 4 (`bind(1, "wl_seat", 4)`; the compositor offers 9), and frame exists from version 5, so no frame event is sent to it and its frame listener is empty (1167) anyway. Scale: fractional scale 120 (1.0), set_buffer_scale 1; scale 2 not tried, not needed.
   - Not tried: a wait of one frame between move and click (the cause is the count of motions, not the timing).
3. Fix: none in chora (the cause is fuzzel's design plus the wrong row of the cell). Cell corrected in f11eb6c: Zebra row y 533, three pointer_move in one inject (enter, approach, row), click. Passes 3/3.
   Note, not a fix: smithay sends enter without a motion on a focus change, so a hand needs two motions after entering and the first inject move only enters. A real mouse sends many motions; ti-1's "click and hover leave the selection" came from fuzzel 1.9.2 (no mouse selection) and ends with the 1.11 pin.

## Gate
- tests/gate-tiling-apps-click.sh: 3/3 ok with fuzzel 1.11.1 (from the scratch prefix, see below).
- tests/merge-gate.sh on claude/fuzzel-click (f11eb6c), serial, four runs; the last passes ("all steps passed", 28 PASS; logs /tmp/merge-gate.JsYSoe/):
  1. 2d-guard FAIL at 0 s: the clone was shallow, base 8742dee missing (`git fetch --unshallow` fixed it; environment, not code).
  2. tiling-audit FAIL "the bar does not show" (ink-1280x800, then every cell); the same cell alone with THEMES=ink SIZES=1280x800 passes. Not reproduced, not investigated.
  3. doors FAIL: mc screenshot-door and volume-door "missing", known flake F16 (reports/harness-r3.md); tiling-audit passed in this run.
  4. all steps passed (tiling-audit 317 s, doors, e2e 274 s).
  The diff against the session branch is one new test file, so runs 2 and 3 are flakes of lanes outside this one.

## setup.sh defect (not fixed here, session-log owns it)
setup.sh fails to build fuzzel 1.11.1: meson setup fails with `Dependency "scdoc" not found` (fuzzel's doc/meson.build needs scdoc) and the script sends meson's output to /dev/null, so the log only shows "ninja: error: loading 'build.ninja'". Container still had apt fuzzel 1.9.2. I built 1.11.1 from the same tarball in the scratchpad (`sed -i "/^subdir('doc')/d" meson.build`, prefix in the scratchpad) and ran the cell with it in PATH. Proposal: `sed` away `subdir('doc')` in setup.sh (or install scdoc by apt) and keep meson's stderr in the log.

## Skips and gaps
- tests/gate-tiling-apps-click.sh, like gate-tiling-apps.sh, is not in any gate list (merge-gate steps, ci.yaml e2e): it runs by hand. Say whether to register the tiling cells.
- The cell needs fuzzel >= 1.11 for the selection; with 1.9.2 it fails (no mouse selection there).
- The cell has no window audit at its end (the cell of r1 had none); not added.
