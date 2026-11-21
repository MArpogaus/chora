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

## Review r1

Reviewer: review-handover-mo-r1 (sonnet, read-only on code). Range 598a34ff..f7dded7a (red 8a3c2bc9, fix 20e4ad6e). Step 0 green: unshallow, `cargo check --workspace`, `cargo build --workspace`. No merge gate run.

Checks:
1. Red first holds. 8a3c2bc9 holds only tests/gate-start-tap.sh and its merge-gate.sh wiring (2 files). Built there (product = a5b77488): `tests/gate-start-tap.sh` red cube ("the Start menu opens after a Super drag on the desktop") and red give ("the Start menu stays open over the given window"), tap/drag/return pass. At f7dded7a: all five cells pass, 11 s.
2. ca-4 holds. `cargo test -p chora-xp` 320 passed, among them `a_super_drag_opens_no_start_menu`. keys.rs `button()` marks the tap used on any pressed button that reaches xp; a lone tap (`tap` cell) still opens and closes Start; a key between press and release is already covered by `key()`. One gap, rhm-4.
3. sd-h2 holds. windows.rs `adopt` runs `shut_surfaces` after `try_apply` took the window. The book proposes only client `surface.created`/`detached` surfaces (kit wm/book.rs:280), so a balloon or a dialog of xp's own (nodes, not surfaces) never reaches `adopt`; modal and `stays` surfaces survive `shut_at`. Test `a_window_that_comes_shuts_the_start_menu` passes.
4. Gate script is not vacuous for the red cells (it is red on the old product); missing windows fail loudly (`ctl wait`, `need_bins`, `fail`); it is the last command of the `matrix` step. Residual weak spots: rhm-2, rhm-3.
5. sh-23 / sd-h1: a person CAN reach tiling without companions. README.md:31 (quick start table) is `chora -- chora-tiling`, bare. Verified on f7dded7a: headless chora, bare `chora-tiling`, 4 s later: 0 surfaces, no waybar, no swaybg, capture 1280x800 of one colour (0,0,0). Only a login session (units: chora-tiling.target pulls waybar and swaybg units) or a construct pane (default manager `chora-tiling --with-companions`, construct/config.rs:326) is dressed. gate-tiling-empty-output.sh starts the companions itself, so its pass does not cover the README path. Rows stay open; the tour line is a chapters-a fix too, but the product path is the README one (rhm-1).
6. KISS holds: only xp (keys.rs, windows.rs, two tests, state.rs is a test), the new gate and one line of merge-gate.sh. No new dependency or key.

Findings (id, severity, file:line, text, fix):
- rhm-1 MAJOR README.md:31 and plugins/chora-tiling/src/main.rs:390: sh-23 render half / sd-h1 are not "not red": the documented start `chora -- chora-tiling` (and a bare `chora-tiling --output` beside xp) gives a black screen with no bar and no start door; findings/shared.md sh-23 and shared-doors.md sd-h1 now say "not red in the product" and docs/known-issues.md:119 says Open. Fix: keep both rows open; start the companions by default in chora-tiling (the pid-file guard `companion()` already stops doubles; `--with-companions` is then a no-op) or, smaller, change the README line to `chora-tiling --with-companions`, plus a gate cell that runs bare chora-tiling with no pre-started companions.
- rhm-2 nit tests/gate-start-tap.sh:58,111,127: `menu_after closed` returns at the first 0.2 s poll, so a Start menu that opens later passes; the `drag` cell also asserts nothing about the window having reached xp (it passes on a5b77488 too). Fix: sleep 0.5 before the closed poll, and in `drag` wait `owner:xp` for the window.
- rhm-3 nit tests/gate-start-tap.sh:136: a fixed `sleep 0.5` lets the click choose tiling before Super+Return; under load the foot goes to xp and `return` is red with no product fault. Fix: poll the chosen manager (or retry the click) instead of the sleep.
- rhm-4 nit plugins/chora-xp/src/wm/keys.rs:155: the tap is marked used only by a press that xp's own grab receives; a Super+button that no one grabs (a user-removed or unbound button such as Super+BTN_SIDE) goes to the client and Super's release still opens Start, because the core ends lone taps of others only at a grab (crates/chora/src/input.rs:310). Fix: end every held lone-modifier tap at any button press in the core, not only at grabs.
- rhm-5 nit plugins/chora-xp/src/wm/keys.rs:151: `button` became `pub(super)` only so tests_moved.rs can call it. Fix: drive the test with `wm.input(Event::InputButton{..})` and keep it private.

Verdict: FIX (computed: one open MAJOR, rhm-1). Counts: BLOCK 0, MAJOR 1, nit 4. The two fixes (ca-4, sd-h2) and the gate are correct and merge-ready by themselves; rhm-1 is a product row that stays open, not a regression of this branch.

## r2

Branch claude/handover-mo head 60314da1 (ti-29 1f9a5180 merged clean at 4a078baf).
- rhm-1 FIXED. Cause: plugins/chora-tiling/src/main.rs:395 started the companions (waybar.sh, wallpaper.sh) only with `--with-companions`, so the documented `chora -- chora-tiling` painted nothing. Now they start by default (the pid-file guard `companion()` stops doubles); `--with-companions` is accepted and hidden, a no-op; `--no-companions` added because ten gates and the audit start their own waybar with a test config and the default one raced them (tiling-audit red "no layout pill", tiling-look red "focus border"); those gates pass it. Cell: `bare` of tests/gate-tiling-empty-output.sh, red 938f9382 (pixel 0 0 0 at the bar), green ccf7ebab; it also clicks the Apps door. README, docs/configuration.md, docs/known-issues.md:119, construct default manager command `chora-tiling` updated.
- rhm-2 fixed (a3c006fa): `menu_after closed` polls 1 s and fails if the menu opens at any poll.
- rhm-3 fixed (a3c006fa): the `sleep 0.5` is a poll until no node has the focus (the press took the keyboard).
- Gate: pre-commit, ci-check, build, test, clippy, xp, matrix, virtual PASS on ccf7ebab; tiling-audit, tiling-look, doors, acts, launcher-click, tiling-empty PASS after the --no-companions gate change (60314da1 only changes the Args bool to a counter, built and run in tiling-empty 5/5, not rerun in the full gate). Red cells on the way, caused by this fix and corrected: tiling-audit and tiling-look (above).
- Pre-commit clippy forbade a fourth bool (struct_excessive_bools); no allow added.
