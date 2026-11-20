# co-majors

## r1

Lane: cloud, branch claude/co-majors from 30727c82, head db4d703b (pushed). Setup:
`git fetch --unshallow`, `cargo check --workspace` and `cargo build --workspace` passed.

| id | on the head 30727c82 | red | fix | cell |
|----|----------------------|-----|-----|------|
| co-19 | gone: stood back, F1 shows the bindings overlay | f59df8fb (guard of construct r1) | 929b64e7 (gone before) | `out` (tests/gate-construct-out.sh) PASS |
| co-20 | gone: stood back, Escape closes the cube and no window is lost; the overlay lists "Close the cube, stood back Esc" and "Show these keys, stood back F1"; stepped in, Alt+Escape steps back (listed) | f59df8fb | 929b64e7 (gone before) | `out` PASS |
| co-21 | reproduced in the tour v7 scene (demo/scene-cube.sh, 1280x720 + 640x720 edge) and in the pull gate | 85691f86 | db4d703b | `pull` (2a, closed-cube check) PASS |

### co-21: what was broken

Not the cube. On the head, stood back after the pull-out of the Sunset window, scene.boxes
gives faces 0 and 1 a shared edge (792,229)-(792,491) on both faces, and the mirrors line up
with them (x 416-792 and 792-864). The "neighbour face" of the owner's still is the pane of the
pulled Sunset window, `construct-pane-s1`. It was dropped in front of the face, 0.9 of the
depth of the face. The step back moves the cube 2 focal lengths away from the eye. The pane is
placed by the person, so it does not give way to the cube (d33bbc8e), and it stays near the eye.
It covered x 599-1257 at 1.8 times the height of the face, over face 0, the seam and face 1,
and was drawn above them, so it read as a face that juts out. The reflection still lay under
the real faces, so it looked as though it did not line up.

Fix (construct plugin only, new plugins/chora-construct/src/cube/aside.rs): when the cube
starts to step back, a placed pane that is nearer than the cube and would overlap its
stood-back screen box gets a place beside the cube. The place is at the depth of the cube, on
the side the pane was on, half a face diagonal plus 0.15 face widths plus half the pane width
from the centre. The pane glides there with the stand. It goes home on step in or close (the
rect is the same as before the step back), and a pane that the person moves meanwhile stays
where it was put. Drag back onto a face still works (pull 2a, super and plain). docs/cube-modes.md
says so.

The red check (tests/gate-construct-pull.sh 2a, `closed_cube_breaches`, by scene.boxes on
HEADLESS-1): the faces that show share an edge within 1 px, at a length equal within 2 %, each
mirror lies under its face within 1 px, and no pane is drawn over a face. On the head:
`construct-pane-s1 [-43, 18, 715, 764] lies over face 0 [416, 255, 375, 290]`.

Captures (reports/co-majors/):
- co-21-tour-30727c82-before.png: the tour v7 still reproduced, Sunset over the cube
- co-21-tour-db4d703b-after.png: same steps, cube closed, Sunset beside it with its mirror
- co-21-gate-pull-30727c82-before.png / co-21-gate-pull-db4d703b-after.png: the gate case
- co-19-20-out-30727c82-keys.png: F1 stood back, the overlay with the Esc and F1 rows

### Gate

Merged origin/claude/autonomous-dev-session-3w-i33pg7 (30727c82; already in, nothing new), then
`tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,cube,construct-hud,pull,placed,keys,doors,out`
on db4d703b: all 12 PASS (pre-commit 3 s, build 25 s, test 182 s, clippy 18 s, ci-check 89 s,
cube 114 s, placed 14 s, pull 63 s, out 14 s, keys 18 s, construct-hud 115 s, doors 333 s).
`out` is the extra cell for co-19 and co-20.

## Review r1

Reviewer lane: cloud, read-only on code. claude/co-majors head db4d703b, base 30727c82. Verdict: **FIX** (0 BLOCK, 1 MAJOR, 2 MINOR, 1 NIT).

Checks run: `cargo check --workspace` and `cargo build --workspace` ok; `cargo test -p chora-construct` 231 passed; `cargo clippy -p chora-construct --all-targets -- -D warnings` clean; `tests/gate-construct-pull.sh` on 85691f86 FAIL with `construct-pane-s1 [-43, 18, 715, 764] lies over face 0 [416, 255, 375, 290]`, on db4d703b PASS (check 1); `tests/gate-construct-out.sh` on db4d703b PASS and 929b64e7 is an ancestor of 30727c82 (check 7). Check 5: only construct, docs/cube-modes.md and the gate changed; new `pub(crate)` items all have a doc line. Check 6: docs/cube-modes.md matches the behaviour; the two co-21 PNGs show the cube closed with Sunset beside it at 950-1113 of 1280.

Check 2: `closed_cube_breaches` would catch a gap or an overlap between faces (no edge pair within 1 px of both endpoints gives "share no edge"), but see rv-3.

Check 4, run with a scratch copy of the gate (not committed): pane rect before the step back [-42.7, 17.9, 714.9, 764.2], stood back [203.3, 317.4, 172.7, 165.3], after Return (step in) [-42.7, 17.9, 714.9, 764.2], so the home rect is exact. A `pane.move` during the stand-back gives [244.9, 317.7, 169.9, 164.6], and it is the same after step in, so the person's move stays (POLS). The pane remains a pane throughout, no window lost.

Check 3: at 1280x800 the pane is on screen on both sides (203-376 left, 950-1113 right). The 640x720 case was not run; the lens scales with the output width (focal = 0.65 w), so a pane of the same relative size sits in the same relative place. Two or more panes: see rv-1.

| id | severity | where | finding | fix |
|----|----------|-------|---------|-----|
| rv-1 | MAJOR | plugins/chora-construct/src/cube/aside.rs:83-95 | `make_way` gives every pane the same `host.beside(centre, side, width)` with no account of panes already placed on that side, so two placed panes on one side have the same inner edge at the same depth and overlap each other (read from the code, not run). | Keep a running offset per side in the loop and advance it by the pane width plus `GAP` for each pane placed. |
| rv-2 | MINOR | plugins/chora-construct/src/cube/aside.rs:139 | The only unit test covers `Aside::at`; nothing covers `make_way` (placed, in front, overlap, side), `step_asides` (exact home, cleared on close or step in), or the removal at panes.rs:230, and gate 2a never steps in again (I checked these by hand, they hold). | Add unit tests for these in the construct test harness and a step-in rect compare to gate 2a. |
| rv-3 | MINOR | tests/gate-construct-pull.sh:131-135 | `closed_cube_breaches` compares the mirror in x only (not its top or height), its 2 % length rule cannot fire for an edge over about 140 px when both ends are within 1 px, and it never checks pane against pane, so rv-1 passes the gate. | Compare the mirror y range with the face, drop or tighten the length rule, add a pane-pane overlap check and a second pane to the scene. |
| rv-4 | NIT | plugins/chora-construct/src/cube/aside.rs:95 | The aside place is not clamped to the leash `radius()` that `move_pane_facing` applies, nor to the screen, so a pane much wider than a face lands partly off screen (not run). | Clamp `to` to the radius, and shrink the offset when the pane would leave the screen. |

## r2

Lane: cloud, branch claude/co-majors from db4d703b; head 8e24dcb6 (pushed). Step 0:
`git fetch --unshallow` and `cargo check --workspace` passed. Review r1 items rv-1 to rv-4.

| id | red | fix | test |
|----|-----|-----|------|
| rv-3 | d155d807: gate `closed_cube_breaches` compares the mirror corners with the face corners (x, height, depth at both edges), takes 1 px for the length of the shared edge, checks pane against pane, and the new case `pair` has a second pane on the same side. Fails on db4d703b: `construct-pane-s1 [203, 317, 173, 165] and construct-pane-s2 [-9, 283, 387, 235] overlap` | d155d807 itself (the gate is the fix) | tests/gate-construct-pull.sh, case 2c |
| rv-1 MAJOR | d155d807 (gate) and 1ed59e8e (unit) | 3afe9285 | gate case 2c; `two_panes_on_one_side_go_beside_the_cube_without_overlap`, `a_pane_with_no_room_on_its_side_goes_to_the_other_side` |
| rv-2 | 1ed59e8e (tests of the existing behaviour, green on db4d703b) and c5b143b1 (gate) | none needed | `a_placed_pane_in_front_of_the_cube_goes_beside_it_on_its_side` (placed, in front, overlap, side), `a_pane_that_does_not_cover_the_cube_stays`, `panes_on_both_sides_go_to_their_own_sides`, `a_pane_that_gave_way_goes_home_exactly_when_the_cube_steps_in_or_closes`, `a_pane_is_aside_until_the_step_in_has_ended`, `a_pane_that_the_person_moves_while_aside_stays_there` (the removal at panes.rs:230); gate 2a steps in with view.step_in and compares the home rect, then steps back and compares the aside rect |
| rv-4 NIT | 1ed59e8e | 3afe9285 | `the_place_beside_the_cube_is_inside_the_leash`, `a_wide_pane_beside_the_cube_stays_on_the_screen` |

Red run on db4d703b's aside.rs with the new tests: 4 FAILED (the two rv-1, the leash and the screen
test), the other 6 pass. After the fix `cargo test -p chora-construct` 241 passed, clippy
`--all-targets -D warnings` clean.

### What changed in aside.rs

- `make_way` takes the covering panes nearest the cube first (by their distance across the view) and keeps
  `taken` per side, advanced by the pane width plus GAP (0.15 face widths) for each pane placed; the pane
  centre is `clearance + taken + width / 2` from the cube centre, so no two places overlap.
- First try (found by the gate, not by the review): with only the shrink of rv-4, the second pane of the gate
  (a 1280 px window next to a 600 px one) did not fit on screen at its running offset and shrank back over the
  first one. So a pane that does not show in full on its side takes the other side (with that side's
  running offset); only a pane that fits on neither shrinks its offset on its own side until it shows
  (20 steps), and never moves out of the leash.
- The place is clamped to `radius()` like `move_pane_facing`.

### Gate

Merged origin/claude/autonomous-dev-session-3w-i33pg7 (bb16f2ff, 8e24dcb6 is the merge), then
`tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,cube,construct-hud,pull,placed,keys,doors,out`:
all 12 PASS (pre-commit 4 s, build 1 s, test 225 s, clippy 17 s, ci-check 111 s, cube 120 s, placed 14 s,
pull 80 s, out 15 s, keys 18 s, construct-hud 120 s, doors 341 s). The brief names a step `construct`; the
gate has no such step, so the 12 cells of r1 stand for it. Not run: the 640x720 case and the screenshots (no
capture of the pair case is kept).

Head: 8e24dcb6.

## Review r2

Reviewer lane: cloud, read-only on code. claude/co-majors head 8e24dcb6 (merge of bb16f2ff), r2 range db4d703b..c5b143b1. Verdict: **MERGE-OK** (0 BLOCK, 0 MAJOR, 2 MINOR, 2 NIT).

Checks run: `cargo check --workspace` ok; `cargo test -p chora-construct` 241 passed; `cargo clippy -p chora-construct --all-targets -- -D warnings` clean.

1. Red first: db4d703b's code with the tests of 1ed59e8e (worktree): 4 FAILED (`two_panes_on_one_side_go_beside_the_cube_without_overlap`, `a_pane_with_no_room_on_its_side_goes_to_the_other_side`, `the_place_beside_the_cube_is_inside_the_leash`, `a_wide_pane_beside_the_cube_stays_on_the_screen`), the rest pass; at 3afe9285 all 241 pass. Not run by me: gate case 2c on db4d703b (needs the compositor gate); I read the gate instead (check 6).
2. rv-1 `make_way` (aside.rs:143-175): sorted by `across.abs()` with `total_cmp` on a Vec in pane order, so nearest first and deterministic; `taken[k]` advances by pane width + GAP per side. Scratch unit tests (not committed) at 1280x800 and 640x720: 3 panes of 400 px on one side, 2 panes of 700 px on one side: no pane over a face or another pane, all on screen. The other-side fallback uses `taken[other]`; a mixed case (1000 px panes on both sides plus a 600 px one) breaks only through the shrink path (rv2-1).
3. rv-1 shrink path: it can end over the cube and over earlier panes (rv2-1). Deterministic (no HashMap order in the placement; `asides` is a BTreeMap).
4. rv-4: the place goes through `leashed` (as `move_pane_facing`) and `on_screen`. `the_place_beside_the_cube_is_inside_the_leash` tests the leash only (radius 1000, which is smaller than the distance to the cube); `a_wide_pane_beside_the_cube_stays_on_the_screen` and `a_pane_with_no_room_on_its_side_goes_to_the_other_side` test the screen bound at the default lens only, not 640x720. Scratch at 640x720: same results as at 1280x800 for the ordinary cases.
5. rv-2: the six named tests assert exact `(at, yaw)` with `assert_eq!`, and the person-moved pane stays (POLS); gate 2a compares the home rect by string equality after `view.step_in` (tests/gate-construct-pull.sh:333), and the aside rect again after the second step back (:337).
6. rv-3: `closed_cube_breaches` takes an edge pair only when both ends lie within 1 px of the neighbour's, so a 2 px gap between two faces prints "share no edge"; mirror x, height and depth at both edges, the shared edge length within 1 px, pane against pane, and pane over face (draw order) are checked. Not run.
7. docs/cube-modes.md (rv2-3): r2 did not touch it; it says the pane "moves beside the cube", with no per-side order and no other-side fallback.
8. Tests and clippy: green as above.

| id | severity | where | finding | fix |
|----|----------|-------|---------|-----|
| rv2-1 | MINOR | plugins/chora-construct/src/cube/aside.rs:119,128 | The shrink path multiplies the whole offset (clearance + taken + half width) by 1.0 down to 0, so it can place a pane over the cube or over panes already placed; scratch at 1280x800: one 3000 px pane lies over the cube; two 2000 px panes lie over the cube and each other; two 1000 px panes beside the cube plus a 600 px pane put the third over the first. Only for panes too wide for either side; ordinary panes never reach it. | Shrink only the part beyond `clearance + taken[k]`, never below it, and let such a pane stay partly off screen. |
| rv2-2 | MINOR | plugins/chora-construct/src/cube/tests_construct.rs:786 | No test covers 3 panes on one side, wide panes on both sides, or the shrink path against a pane or the cube (only against the screen), so rv2-1 passes; the 640x720 lens is not tested at all. | Add the scratch cases above (shrink, wide on both sides, 3 panes) and one at `Lens::new(640.0, 720.0)`. |
| rv2-3 | NIT | docs/cube-modes.md:203 | Check 7 not met: nothing on the nearest-first order per side, the GAP, the other-side fallback or the shrink. | One sentence: the nearest pane takes the inner place on its side; a pane that does not show in full goes to the other side. |
| rv2-4 | NIT | plugins/chora-construct/src/cube/aside.rs:120,161 | `leashed` runs after the running offset, so a leash smaller than the distance to the cube clamps places together and they overlap (scratch: radius 1000 or 1200 with the cube 3136 away; no breach at 3500 to 6000). Also `taken` starts at 0 although `make_way` can run again (step back during a step in) with panes still aside (filtered at :155), so a new covering pane may take the place of an aside. Both far from default use. | Start `taken` from the asides already beside the cube, and skip `leashed` when the cube is outside the leash. |

## r3

Lane: cloud, branch claude/co-majors-r3 from fc404a90 (debt-f4) plus the merge of co-p2-fix ab4cbb4f (step 0: `cargo check` and `cargo build --workspace` green). Red first for each item, on the head of the branch.

| item | red | fix | cell | state |
|------|-----|-----|------|-------|
| co-27 MAJOR | fc4cf8c0 (unit test `an_overlay_shows_the_window_of_its_own_program_reserved_for_the_manager_below`; gate k2a red 3 of 3 before) | 193039bf | construct-extra (k2a added, header line dropped in 68790912) | fixed for k2a; windows: see co-30 |
| co-29 MAJOR | b0ea6e26 (unit test `a_row_hovered_while_the_list_fades_in_leaves_it_fully_opaque`, 0.578 after 1 s); gate red 857f9408 (pointer move dropped, red 3 of 3 at 1280x720@1.5 light) | 8313efab | construct-hud | fixed |
| co-28 MINOR | not started | none | none | open: rule 6 order (items 1 and 2 first) and the budget; the 3d gate stays out of construct-extra |
| co-30 MINOR (new) | tests/gate-construct-windows.sh:511 red with and without co-p2-fix | none | none | open, see below |

### co-27 cause
218c2a6b ("adopt only a window that the book claimed") made `Construct::created` adopt a window only when `book.claim` returned a change. The core reserves the window of a program that the overlay starts for the window manager below (tiling), and the book claims no window reserved for another manager, so the overlay (which only shows such a window, D66) took no pane for its own program. The flavor path stays green because nothing is reserved. Fix (adopt.rs): an overlay takes the pane of a window whose token it waited for without the claim (it is shown, not attached, so no phase, origin or give is involved); the flavor still refuses a reserved window (`a_launch_adopts_no_window_reserved_for_another_manager` stays green).

### co-29 cause
Not the keyboard: the core keeps the focus on construct-look and the construct state says `open: construct-list`. The list node stood at opacity 0.87 for 4 s. The list opens under a pointer that stood still, so `pointer.entered` (moved false) hovers row 0 at once; `draw_list(false)` updates the node, and an update (or animate) of a node ends its running animation where it is (chora-scene `change_props`), so the 160 ms fade-in stopped at about half. The pointer move to [4, 300] only kept the pointer off the rows. Fix (overlay.rs, list.rs, main.rs): the list remembers when it opened and a redraw within the fade-in issues the fade again for the time left. The workaround line and its header note are gone. kit7-a2 has no line in tests/acts/known-red.toml any more (nothing to remove); `tests/gate-acts.sh --no-selftest construct`: 55 pass, 0 known red, 0 blocking.

### co-30 (new, found behind co-27)
With co-27 fixed, windows passes sections 1 to 4 and fails at line 511 (section 5): the Super drag of d1 on the flavor's output never reaches the construct (no `release` in its log). A/B on fc404a90 plus the co-27 fix without co-p2-fix fails the same way, so co-p2-fix is not the cause. `claimant_at` takes the smallest claim among the grabbers of the binding: mc with `--manage-windows` claims [0,56,1280,688] and grabs the drag, the construct claims [0,0,1280,800]. With `--manage-windows` left out the d1 drop passes (and d2 to mc then fails as it must). Decision for the PM: a stale setup (a window manager of its own output) or a D55 rule for two window managers on one output. windows stays out of construct-extra, its header names co-30.

### Gate
`tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,gate-lib,construct-hud,construct-extra,cube,keys,pull,out,doors` on the merge with origin/claude/autonomous-dev-session-3w-i33pg7 (already up to date): all PASS (run /tmp/merge-gate.dGKMG5): pre-commit 10 s, gate-lib 70, build 32, test 242, clippy 23, ci-check 106, cube 129, pull 80, doors 365, construct-extra 63 (k1, mcpane, nested, k2a), construct-hud 123, out 15, keys 22. No red cell, no FLK. Head 68790912.

## Review r1

Lane: cloud review of claude/co-majors-r3 head 68790912 against base c839a11c (9 files, +94 -17). One round, sonnet. `cargo check --workspace`, `cargo test -p chora-construct` (243 pass) and `cargo clippy --workspace --all-targets -- -D warnings` are green; the touched files are at most 996 lines. I ran no gate cell (brief), so the gate checks are by reading the diff.

**Verdict: MERGE-OK** (0 BLOCK, 0 MAJOR, 4 MINOR/NIT).

### Checks
1. Red first, in scratch worktrees: `an_overlay_shows_the_window_of_its_own_program_reserved_for_the_manager_below` fails at fc4cf8c0 (adopt.rs of c839a11c; panics at lifecycle.rs:339) and passes at 193039bf. `a_row_hovered_while_the_list_fades_in_leaves_it_fully_opaque` fails at b0ea6e26 ("the list fades in to the end: 0.578125") and passes at 8313efab. 857f9408 deletes only the pointer-move line of tests/gate-construct-hud.sh, so its red is the gate itself (not run here).
2. co-27, adopt.rs:230 `shown_not_owned = self.overlay && pane.is_some()`. `pane` is Some only through `adopt.rs:223-226`: `waiting.position(|w| token.is_some() && w.token == token)` then `waiting.remove(i)`. A stranger window, a window without a token, and a Waiting without a token (plugin and manager panes are `Waiting::new(None, ..)`) cannot match. Stale Program entries go at `adopt.rs:206` (`stale()`, main.rs:421, LAUNCHED), the list clears at `scene.rs:load` and `leave`, and each token is consumed by its first window, so a second window with the same token is gated by `free` as before. The launcher branch (adopt.rs:207-222) returns first. The branch adopts with `Hold::Shown` (`started()`, list.rs:294), which attaches nothing: tiling keeps the owner and the placing (the unit test asserts owner None for the book, and k2a:178-181 asserts the owners are tiling alone). Give-back: `leave` (scene.rs:176) removes only the construct root and clears `panes`; `load` removes the shown panes' nodes (scene.rs:139 partition); neither touches the window, so no window is lost. The flavor still refuses a reserved window (`a_launch_adopts_no_window_reserved_for_another_manager` green).
3. co-29, overlay.rs `left`. `opened` is set in `open_list` (list.rs:154), so open, close, open fast restarts it; `close_list` sets `list = None`, so `draw_list` returns at its first line and a closing list's fade to 0 is never overridden; a redraw after FADE_IN has `left <= 0` and pushes no fade; `draw_list(true)` keeps its full FADE_IN. The pointer move is gone from the gate and the unit test pins the cause (a hover redraw under a still pointer), so the cell passes for the right reason.
4. merge-gate.sh:243 runs k2a last in construct-extra; `grep -iE 'known|skip|todo'` on tests/gate-construct-k2a.sh finds nothing; the former red is the plain `[ -n "$pane" ] || fail` at line 178.
5. co-30, see rv-4. 6. Tests, clippy and size: green, see above.

### Findings
- rv-1 | MINOR | plugins/chora-construct/src/lifecycle.rs:378 | the co-29 unit test cannot tell `left` from a fresh FADE_IN: `opened` uses the wall clock while the test advances the scene clock (`advance(40.0)`), so a fix that always pushed `fade(LIST, 1.0, FADE_IN)` passes too | fix: assert the animation duration of the redraw (about FADE_IN minus elapsed), or backdate `opened` in the test.
- rv-2 | MINOR | plugins/chora-construct/src/lifecycle.rs:339 | co-27 has a positive test only: nothing pins that an overlay still adopts no reserved window without a waited token (a stranger, a stale Waiting), which is the guard of the new branch | fix: add a test that creates a reserved window without a token on an overlay and asserts no pane.
- rv-3 | NIT | tests/gate-construct-hud.sh:137 | the `sleep 0.5` that followed the removed pointer inject stays and now delays Tab for nothing | fix: delete the line (or keep it if the pane move needs the settle time, and say so).
- rv-4 | MINOR | crates/chora/src/ipc.rs:463-480 (co-30) | see below.

### co-30: real rule fault, not a stale setup, MINOR
Evidence. The setup is deliberate: tests/gate-construct-windows.sh:479 starts `chora-mc --manage-windows --output "${outs[0]}"`, the output of the flavor, and section 5 needs mc as a window manager there to drop d2 onto the mc pane (a drop to mc needs its claim). The construct claims the whole output (panes.rs:333, `claim = [x,y,w,h]` for a flavor) and mc its work area (chora-mc build.rs:78, `claim: manage_windows.then_some(work)`). The core routes a Super+button press with `button_grab_for` (ipc.rs:463-480; input.rs:760), which takes `claimant_at` = the smallest claim containing the pointer (scene.rs:257-269), and the unit test at crates/chora/src/input.rs:1823 ("The smallest claim wins") fixes that as the rule. The rule is a flat 2D one: the construct draws a 3D space over its claim, so a pane that stands over the work area of mc is under the pointer for the user, but the claim of mc is smaller and mc takes the press. The same cause makes the d2 drag of line 520 fail, so the drop onto an mc pane cannot work on a shared output at all. Fix (one sentence): in `button_grab_for`, prefer the grabber that owns the node hit under the pointer (the pane node, then its ancestors) and keep the smallest claim as the tiebreak, then update input.rs:1823 and put windows into construct-extra. Severity MINOR: no window is lost (the lane's `no_lost` holds in sections 1 to 4, and the press goes to a client that did not own the window), the case needs mc with `--manage-windows` on the output of a construct, and the shared output is supported (D55, PM 06:05), so it is a fault in a narrow case and not a gate defect; it becomes MAJOR only if the PM counts a drag that does nothing as a lost-window path.

## r4

Lane: cloud, branch claude/co-r4 from 68790912 (co-majors-r3). Step 0 green (`cargo check`, `cargo build --workspace`). Red first for each item; rv-1 and rv-2 are pins of code that is already right, so their red is a mutant of the code, shown and reverted.

| item | red | fix | cell | state |
|------|-----|-----|------|-------|
| co-30 MINOR | e758dcfd (unit test `a_shared_button_grab_goes_to_the_owner_of_the_node_under_the_pointer`: the press over a construct pane inside the claim of mc reached mc) | 5a782e58 | construct-extra (windows added, header line dropped in 0355c2bc); windows green 3 of 3 | fixed |
| rv-1 MINOR | mutants: a fresh FADE_IN on the redraw (0.9925) and no fade (0.756) fail the pin; the head passes | c98adb18 | test (cargo test -p chora-construct) | fixed |
| rv-2 MINOR | mutants: `shown_not_owned = self.overlay`, and a None token that matches a waiting pane without one, fail the pin; the head passes | 51175f9a | test | fixed |
| rv-3 NIT | none (a settle) | 31b2d114 | construct-hud green 2 of 2 without the sleep | fixed |
| co-28 MINOR | the gate itself: 3 cells red 3 of 3 on 31b2d114 | b11cda2a | construct-extra (3d added in 42c9304b) | fixed, stale checks |

### co-30
`Ipc::button_grab_for` (ipc.rs) takes a `Pointer { point, node }`; `open_button_grab` (input.rs) passes `render::node_under`. Among the grabbers of the binding, the one that owns the node under the pointer, or its nearest ancestor owned by a grabber, wins; over no node of a grabber, the smallest claim breaks the tie, else the first grab. The pins of the smallest-claim rule (`a_shared_button_grab_goes_to_the_claim_under_the_pointer`, `a_button_grab_without_a_claim_has_one_owner`) keep their cases unchanged: they hit no node. The new pin covers the pane over the work area (construct), a node of mc (mc), no node inside the claim of mc (mc, the tie) and outside it (construct). Windows section 5 passes: d1 drops on tiling, d2 on the mc pane, d3 moves on empty space.

### co-28: stale checks, not faults
- Label (:187): the label became a glass pill (`construct-label-body-text`) with a neighbour pill `construct-label-n-<k>` for every other pane; `construct-label-text` stays hidden. The check now wants the title of pane b (read from its neighbour pill before the hover) in the focus pill, no neighbour pill for b and one for c (the pane ahead), so a label that follows the pane ahead fails it.
- Recording (:272): the walk goes in pane steps of 800 per tap (D8, 0934fb76), not STEP 90. The first `w` put the camera at z 800, so the fixed window 470-560 x 380-420 lay inside pane c in frames 1 to 3 (equal hashes), and the second tap carried the camera to z 1600, past pane b, which then had no rect (the empty `pointer_move`). The recording now starts after an `s` back at the start point, `view_of` hashes the view below the bar, and an `s` after it brings pane b back for the last check. The frames 1, 2, 3 and the end differ.

### Gate
`tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,gate-lib,construct-hud,construct-extra,cube,keys,pull,out,doors,mc-paths,mc-handover,matrix,acts` on 42c9304b (origin/claude/autonomous-dev-session-3w-i33pg7 at 79c8d2b5 already contained): all PASS (run /tmp/merge-gate.xk9Dw8): pre-commit 11 s, gate-lib 71, build 30, test 250, clippy 42, ci-check 64, matrix 292, doors 369, acts 254, cube 130, pull 88, construct-hud 130, out 15, keys 23, mc-paths 67, mc-handover 12, construct-extra 183 (k1, mcpane, nested, k2a, windows, 3d). No red cell, no FLK.

Head 42c9304b.

## Review r4

Lane: cloud review, read-only, claude/co-r4 head 42c9304b against 68790912. Step 0 green (`cargo check --workspace`).

Verdict: **MERGE-OK** (BLOCK 0, MAJOR 0, MINOR 1, NIT 0).

Checks:
1. Red first: `a_shared_button_grab_goes_to_the_owner_of_the_node_under_the_pointer` fails on e758dcfd (input.rs:1874, the press over the construct pane reached mc: `left: []`) and passes on 5a782e58.
2. co-30: `open_button_grab` passes `render::node_under`, the top node that accepts input (overlay of the output first; None over the lock screen, a layer surface above, a Block or left-out node), and `button_grab_for` walks its ancestors to the first node whose owner is a grabber; otherwise `claimant_at` and the first grab as before. Per look: xp, tiling (bar, launcher) and mc remote have a single grabber of their bindings, so the rule does not change who gets a press. In mc with the construct, the hud, a pane and an own window attached under a pane (owner None, D66) go to the construct, an mc page goes to mc, and no node falls back to the smallest claim. The chat and notify overlays grab no buttons; a press over their nodes finds no grabber among the ancestors and falls back to the claim rule, as before. Stacked overlays: the top node wins and nothing below it is consulted. Press and release stay on one grabber: `state.button_grab` keeps the conn of the press, so removing the node or moving the pointer to another claim during the drag changes nothing (pin `a_shared_button_grab_goes_to_the_claim_under_the_pointer`, "the drag stays with the client of the press"). The old pins keep their cases: `a_shared_button_grab_goes_to_the_claim_under_the_pointer` and `a_button_grab_without_a_claim_has_one_owner` create only `group` nodes, which nothing hits.
3. rv-1: the pin moves `opened` back by 60 ms and checks opacity 1 after 110 ms; a fresh 160 ms FADE_IN and a stopped fade both stay below 1, and a slow runner only shortens the remaining fade, so it cannot go red falsely. rv-2: s1 (no token, right after a launch, beside a waiting Manager pane without a token) catches both `shown_not_owned = self.overlay` and a dropped `token.is_some()` guard; s2 catches a token mismatch; `waiting.len() == 2` catches a consumed wait. Neither pin is a tautology.
4. co-28 label (:187): still catches a wrong pane. Titles of the probe panes can match, but the check also requires that b has no neighbour pill and c has one, so a label of the pane ahead fails it. The recording (:272) widened its window too far: see rv-5.
5. merge-gate: construct-extra grows from 63 s to 183 s (+120 s, about +60 s of wall time at --jobs 2). The matrix ran 292 s next to it in the lane run (cap 315 s; earlier runs 270-296 s), so this run gives no evidence that the change breaks the cap. MX-CAP stays at the edge and has its own owner.
6. `cargo test -p chora --lib` 624 passed; `cargo test -p chora-construct` 244 passed; `cargo clippy --workspace --all-targets -- -D warnings` clean; `tools/size-ratchet --base 68790912` ok (0 files over 1,500 production lines).

Findings:
- rv-5 | MINOR | tests/gate-construct-3d.sh:280 | `view_of` hashes every row below 100 at full width, so frames 1, 2 and end differ on any change of the HUD (focus or neighbour pills, a fade after the `w`), even with a camera that never moved; the check no longer proves that the walk follows the steps of the clock, and the lane showed it red against no mutant of the walk. | Assert the camera itself per frame: read the pose (or the rect of pane c via `box`) from frames 1, 2 and end, or hash a strip that holds no HUD node, and show it red with the walk frozen.

## r5

Lane: cloud, branch claude/co-r5 from 42c9304b (co-r4). Step 0 green (`cargo check --workspace`, `cargo build --workspace`).

| item | red | fix | cell | state |
|------|-----|-----|------|-------|
| 1 K5 wobble race (FLK in train 87c) | 39a709b5 (scratch mutant: the sleep before the read at 0; red 2 of 5 under 8 busy loops at load 12-19, once with the message of 87c, `441 False`; 8 busy loops with the sleep at 0.1 and 24 loops: green 5 of 5 each, the race does not show here without the mutant) | 1454c349 | construct-extra (3d); 5 of 5 green under the 8 loops of the red | fixed |
| 2 rv-5 MINOR | mutant, scratch, reverted: the stride of the walk key not started (`camera.rs`, `self.stride = Some(...)` dropped): the new check gives 0, 0, 0, 0 and fails | 6bd3b6f7 | construct-extra (3d); 5 of 5 green under 8 loops; poses 0, 225, 403, 800 | fixed |

### Item 1
`mesh_of` was read once after `sleep 0.1`. The wait now polls both meshes (pane and mirror) while the button is held, until both show a depth, 20 polls of 0.25 s at most; each poll first moves the held pane by 30 px so that the springs do not rest while the poll waits (wobble.rs ticks a frame and settles when the target stops). Other fixed sleeps followed by one read, now polled: the baseline `view` (two reads 0.5 s apart agree, instead of `sleep 2`), "the drag moves pane a" (until_ok), "a pane dropped off the output shows where it was again" (a `pane_at` call in front as barrier, then until_ok on the box), "after a click on the empty space, w walks" (until_ok). Left as they are, on purpose: `sleep 0.6` before "the click does not move the view" and the `sleep 1` before the check that an injected click gives no keyboard (both prove an absence, which needs a time), `sleep 0.5` after a click before a read of a camera that no click moves, and the `sleep 0.5` before `s`.

### Item 2
The script of `choractl record` now has two `methods.call` cues, `construct.save` at frames 2 and 3 (sent before the step of their frame, so they hold the state of the pictures 1 and 2). The check reads the pose z of the camera from those saves, from the camera at the start and from the camera at rest after the recording, and wants start < frame 1 < frame 2 < end. The picture hash (`view_of`, `end.png`) is gone. Mutants: with the stride not started the old check was red too in this run (no HUD node changed), so the blindness of rv-5 is not reproduced here; the new check names the poses in its message and proves the walk by the camera, not by pixels.

### Gate
`tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,gate-lib,construct-hud,construct-extra,cube,layout` on c2b0310f (origin/claude/autonomous-dev-session-3w-i33pg7 merged; the one conflict, the comment of `construct-extra` in merge-gate.sh, keeps both texts: launcher-click from the integration branch, k2a, windows and 3d from co-r4): all PASS (run /tmp/merge-gate.zEUbYE): pre-commit 4 s, gate-lib 69, ci-check 66, build 19, test 161, clippy 12, cube 119, layout 173, construct-hud 106, construct-extra 155. construct-extra alone 2 more times: PASS 155 s, PASS 153 s. No red cell, no FLK.

Head c2b0310f.
