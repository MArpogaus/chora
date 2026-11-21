# handover-mo: the open MAJOR rows of the two-output handover

## r1

Lane: cloud, claude/handover-mo from a5b77488. Step 0 green (unshallow,
`cargo check --workspace`, `cargo build --workspace`). PM merge of the
session head 598a34ff before the gate: f7dded7a.

Shas: 8a3c2bc9 red cells (tests/gate-start-tap.sh, wired into the
`matrix` step of tests/merge-gate.sh), 20e4ad6e fix, f7dded7a merge of
598a34ff. Head: f7dded7a.

New gate tests/gate-start-tap.sh: xp on HEADLESS-1, tiling on HEADLESS-2,
both take injected keys, core with --test-injected-press-is-real (an
injected click chooses a manager as a person's does, D55). Drags inject
press, steps and release apart about 0.1 s, as the tour does; a drag in
one inject hides ca-4 (measured: cube passed that way on a5b77488).
Cells: tap (control: a lone Super opens and closes Start), cube, drag,
return, give.

| row | cell | a5b77488 | after 20e4ad6e |
|---|---|---|---|
| ca-4 | cube | red 4/4 | pass 3/3 |
| sd-h2 | give (Start open, window given to xp) | red 4/4 | pass 3/3 |
| sd-h2 | drag (Super drag of a tiling foot onto xp) | pass 5/5 | pass |
| sh-23 key half | return (click empty tiling output, Super+Return) | pass 5/5 | pass |
| sd-h1, sh-23 render half | gate-tiling-empty-output.sh outputs, panes | pass 5/5 each | pass |

Causes and fixes:
- ca-4: plugins/chora-xp/src/wm/keys.rs `button()` sent the Cube press
  to `grab_cube` without marking the pending Super tap (`self.tap`) used;
  `key_released` then toggled Start. Fix: a grabbed button press marks
  the tap used. The core already ends taps of other clients at their
  grabs (crates/chora/src/input.rs:310 `end_lone_taps_of_others`), not
  xp's own. Unit test `keys::a_super_drag_opens_no_start_menu`
  (tests_moved.rs), red without the fix.
- sd-h2: plugins/chora-xp/src/wm/windows.rs `adopt` focused the window
  that came but left the Start menu open over it (only a window that
  leaves xp shut the menus, pointer.rs `give_dragged` via
  `shut_surfaces`). Fix: adopt applies `shut_surfaces` once the core took
  the window. Unit test `a_window_that_comes_shuts_the_start_menu`
  (state.rs), red without the fix.
- sh-23 key half: not red. With presses that count as a person's, the
  click on the empty claim of tiling chooses tiling
  (crates/chora/src/state.rs:948 `choose_manager`) and Super+Return
  gives a focused foot of tiling; Start stays shut. Without the flag an
  injected click chooses nobody and the foot goes to xp, as designed
  (gate-click-routes.sh case 2); a tour without the flag shows the old
  picture.
- sd-h1 / sh-23 render half: not red in the product. The bar and the
  wallpaper of tiling are companions (waybar.sh, wallpaper.sh) that the
  session units, `--with-companions` or a pane manager start; with them
  the empty output paints its bar and Apps door at once (gate
  tiling-empty). A bare `chora-tiling --output` is black: that is the
  tour harness, demo/tour/chapters/handover.py:35 (start_multi), which
  is lane chapters-a's file: add `--with-companions` there. No product
  code for it.

No new dependency, no new config key; construct focus and the tour
untouched.

Gate on f7dded7a: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,xp,matrix,virtual,doors,acts`
green: pre-commit 4 s, ci-check 700 s, build 33 s, test 239 s, clippy
20 s, xp 86 s, matrix 326 s (gate-start-tap: tap cube drag return give
pass), virtual 48 s, doors 376 s, acts 300 s; EXIT 0. No red cell.
