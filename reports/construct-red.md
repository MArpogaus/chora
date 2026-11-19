# construct-red: the construct rows of known-red.toml go green (F7, F3)

## r1
Lane r1 (opus, high) on claude/construct-red from ee53e09c. Step 0: cargo, python3, jq present (setup.sh not needed);
`git fetch --unshallow`, `cargo check --workspace` green.

Commits: a9c870b6 (kit7-a2 red), df070c01 (co-21 red), e46862fc (kit7-a2 fix, D107, 2 rows out), d8f80eaa (co-21 fix,
1 row out), 28254eb6 (wm7a-m3 cell).
- kit7-a2: **fixed, decision D107**. mc, tiling and xp give a new window the keyboard (tiling windows.rs:23, mc manage.rs:526,
  xp focus_moved after Created). In the construct a new window that comes as a pane without a flight now takes the keys
  from a pane window that a press gave them (input.rs:301) and gives them to the walk: `lifecycle.rs` `keys_back_for` sends
  `modes::the_keyboard_to_the_look()` (`kit::wm::focus::release` + FocusNode look). It stays put while the camera is in or
  flies into a window, and for a foreign, launcher or hidden-window focus. Red cell `a_new_pane_takes_the_keyboard_back_to_the_walk_after_a_press`
  (last focus `s1`, wanted none), guard cell `a_new_pane_leaves_a_foreign_focus`. D107 is the next free number; DOCS-F8c's
  D107+ plan (audit-f) must start at D108.
- co-21: **fixed in the invariant, not the construct**. Cause: `Host::faces` (cube.rs:607) hides each face that does not
  face the eye; halfway through the 4 s turn faces 1 and 2 show and meet, 0 and 3 face away (cube.rs:1510 already asserts the
  stood-back "not the one before" hidden). A closed cube from outside shows at most two side faces, so I13's held clause
  (a box for every ring neighbour) could not pass. I13 now wants, held in a turn, a shown face with a neighbour that shows
  with corners (a lone face is the gap net3 saw) and a neighbour box with corners; the seam check is unchanged. Selftest:
  red cell `test_held_in_a_turn_a_closed_cube_hides_its_back_faces`, the planted breach is now a lone face held (ring(1)).
- wm7a-m3: **already fixed on the base by 1ae3f243** (`close_cube` calls `tell_end`); only `Host::last_word` had a cell. New
  FakeCore cell `a_cube_closed_in_a_turn_tells_its_rest_last` reads the emits: red with the `tell_end` call taken out
  (last data `turning: true`), green on the head. Backlog row can close.
- Acts: `tests/acts/run.py --no-selftest construct` (BIN=target/debug): 53 pass, 0 known red, 0 blocking; I13, I4 and
  "Tab right after a new window opens the window list" pass. known-red.toml keeps only net3-m1 (mc I5, KIT3).
- `cargo test -p chora-construct -p chora-kit --features chora-kit/testing`: 206 + 274 + doc cells pass; clippy of both, all
  targets, -D warnings: clean. `tests/test_acts.py` 109 pass.
- Gate at 28254eb6, `tests/merge-gate.sh --jobs 3`, once: **green**, 41 steps PASS, no flake, no rerun (acts 173 s, pull, cube, construct-hud, pixels, e2e included).
- Open: none of the three. DOCS-F8c starts its records at D108.

## Review r1
Review lane (opus, high) of claude/construct-red 28254eb6 against ee53e09c; read-only on code, no gate run. Verdict: **FIX** (rc-1 MAJOR).

Checks:
1) Red first: holds. `a_new_pane_takes_the_keyboard_back_to_the_walk_after_a_press` fails on a9c870b6 (last focus `Some("s1")`,
   wanted `None`) and passes on e46862fc (own target dir; a shared target dir across worktrees reuses a stale binary).
   `tests/test_acts.py -k I13`: 2 fail on df070c01, 9 pass on d8f80eaa.
2) kit7-a2: the three looks do give a new window the keys: tiling windows.rs:23 (adopt, "takes the focus"), mc manage.rs:526
   (cell, except a window of app.open), xp wm/windows.rs:49 (`adopt` -> `focus(Some(surface))`). The construct gives them to the walk
   instead, which the act wants (Tab opens the list). No path leaves the keys with no one: `the_keyboard_to_the_look` releases and
   focuses LOOK in one apply, and a focus of another manager, the launcher or a flown-into window stays. See rc-1 for the POLS gap.
3) co-21: a real wrong invariant, not a weakened check. `Host::faces` (cube.rs:607) hides each face that faces away, so held halfway
   through a turn only two adjacent faces show and the old "box for every ring neighbour" could not hold. The new clause flags a face
   shown alone held in a turn, which is what net3 saw (reports/net3.md:36, "only face 1 shows", rw-1 at dc00d33c); the
   corners-of-a-boxed-neighbour clause and the seam check between shown neighbours (1 and 2 mid-turn) are unchanged.
4) known-red.toml: three rows out (co-21 I13, kit7-a2 act and I4), each named passing in "## r1" (53 pass); one row left, net3-m1 (mc I5, KIT3).
5) wm7a-m3: `a_cube_closed_in_a_turn_tells_its_rest_last` passes on the head and fails with `self.tell_end(&host)` (cube/construct.rs:288)
   commented out (last data `turning: true`); restored.
6) D107 content: see rc-2 (the number clash with docs-f8c is the PM's at merge).

Findings:
- rc-1 MAJOR plugins/chora-construct/src/lifecycle.rs:37: a window that the pressed window's own app opens (a dialog with an xdg parent,
  which the core reserves for the construct, chora-scene scene.rs:457; or Ctrl+N in a pressed terminal) comes as a pane without a
  flight, so `keys_back_for` moves the keys to the walk and the text typed for that dialog drives the walk, while mc, tiling and xp
  give that new window the keys. Fix: give the keys (`focus::give`) to a new pane that is a child of the pressed window (carry the
  toplevel parent in `surface.created`), the walk otherwise; name the case in D107 and add a FakeCore cell.
- rc-2 MINOR docs/decisions.md:156: D107 says "Accepted, construct-red r1" (a lane accepting its own design; other records are accepted
  by the PM after review) and "in mc ... a new window takes the keyboard" without mc's app.open exception (manage.rs:526); its cost
  omits the child-window case. Fix: mark it proposed until the PM accepts it at merge and name both cases.
- rc-3 NIT tests/acts/looks/construct.py:86: the docstring of `tab_after_a_new_window` still says "(known red under WM2)". Fix: drop it.

## r2
Lane r2 (sonnet, high) on claude/construct-red from 28254eb6. Step 0: cargo, python3, jq present; `git fetch --unshallow`, build green.

Commits: 6c065581 (rc-1 red), da68737d (rc-1 fix, rc-2, rc-3).
- rc-1: **fixed**. `surface.created` had no xdg parent, so the event gets an optional `parent` (chora-scene `Event::SurfaceCreated`,
  set in `crates/chora/src/handlers.rs` `announce` from `toplevel.parent()`; `kit::wm::Created.parent`; `testing::created_child`;
  api.md names it). `lifecycle.rs` `keys_back_for` became `keys_for_new`: a new pane that is a child of the pressed window gets
  `kit::wm::focus::give`; any other new pane keeps the r1 rule (keys to the walk); the guards (camera in a window, foreign,
  launcher, hidden focus) are unchanged. Red cell `a_dialog_of_the_pressed_window_gets_the_keyboard_and_an_unrelated_window_does_not`
  (fails on the red commit at the dialog assert, passes on the fix), guard cell `a_child_of_another_window_gives_the_keyboard_to_the_walk`.
- rc-2: D107 is now **D116**, "Proposed (PM accepts at merge)", names the child-window rule and the mc app.open exception, and the
  cost of the new `parent` field. The docs-f8c plan keeps D107-D115.
- rc-3: the "known red under WM2" docstring of `tab_after_a_new_window` is gone.
- `cargo test -p chora-construct -p chora-kit --features chora-kit/testing`: 208 + 274 + doc cells pass; clippy (construct, kit, scene,
  chora, xp; all targets, -D warnings) clean. `BIN=target/debug tests/acts/run.py --no-selftest construct`: 53 pass, 0 known red.
  Gate at da68737d, `tests/merge-gate.sh --jobs 3`, once: green, 41 steps PASS, no flake, no rerun.
- Open: none. The event gained a field, so the PM should note it in the merge (api.md updated; clients that ignore it are unaffected).

## Review r2
Review lane (opus, high) of claude/construct-red da68737d against 28254eb6; read-only on code, no gate run. Verdict: **MERGE-OK** (no BLOCK, no MAJOR).

Checks:
1) Red first: holds. `cargo test -p chora-construct --features chora-kit/testing` on 6c065581: 207 pass, 1 fail,
   `a_dialog_of_the_pressed_window_gets_the_keyboard_and_an_unrelated_window_does_not` at the dialog assert (lifecycle.rs:389,
   last focus `Some(None)`, wanted `Some(Some("s2"))`); on da68737d: 208 pass.
2) rc-1: `keys_for_new` gives `focus::give` only when `created.parent` is the pressed window (core focus, an own window, camera in
   no window); any other new pane keeps the walk. Nested child (dialog of a dialog): holds once `focus.changed` names the first dialog,
   which is then the pressed own window. A child that maps after the press moved to another window: parent is not the focus, keys to
   the walk (the D116 rule for an unrelated window). A child of a window on another cube face: the same rule, keyed on the focus, not
   the face. An xdg_popup never reaches it: `new_popup` (handlers.rs:484) only tracks the popup, no window, no `surface.created`.
   No path leaves the keys with no one: the child branch is one `focus` command; the walk branch is release + look focus in one apply.
   Gap: a new window of the same app without an xdg parent (Ctrl+N) is not covered, though the docstring and D116 say so (rc-4).
3) Core API: additive. `parent` is `#[serde(default, skip_serializing_if = "Option::is_none")]`, `Event` has no deny_unknown_fields,
   api.md names it. A throwaway probe in `a_child_toplevel_goes_to_the_manager_of_its_parent` (ipc.rs:4402, reverted) saw
   `surface.created` with `"parent": "s1"` for the dialog. A `set_parent` after the batch of the first commit is not reported (rc-5).
4) Other looks: `cargo check --workspace` green (mc, tiling compile; neither builds a `Created` or `SurfaceCreated` literal);
   `cargo test -p chora-kit --features testing` all pass (274 + integration + doc); `cargo test -p chora-xp` 252 pass.
   xp startup.rs sets `parent: None` for nodes it adopts at start, which xp never reads.
5) rc-2: D116 is "Proposed (PM accepts at merge)", names the child rule, the mc app.open exception and the `parent` field; rc-3: the
   docstring in tests/acts/looks/construct.py:84 is clean.
6) D-number clash: the PM's at merge.

Findings:
- rc-4 MINOR plugins/chora-construct/src/lifecycle.rs:34 and docs/decisions.md:156 (D116): both say a "new window of the same app"
  counts as a child, but the code matches only the xdg `parent`; a Ctrl+N window (foot, GTK) has none and gives the keys to the walk.
  Fix: drop "or a new window of the same app" from the docstring and D116 (or, if the PM wants Ctrl+N covered, also match the pid of
  the pressed window).
- rc-5 MINOR crates/chora/src/handlers.rs:194: `parent` is read once at `announce`; a later `xdg_toplevel.set_parent` sends no event
  (no `parent_changed` handler), so such a dialog gets the r1 rule (keys to the walk); no look reads it after creation.
  Fix: say in api.md that `parent` is the parent at the first commit, and leave a change event to a later need.
- rc-6 NIT crates/chora/src/ipc.rs:4412: no core cell asserts the new field (the FakeCore cells script it). Fix: subscribe `a` to
  `surface.created` in `a_child_toplevel_goes_to_the_manager_of_its_parent` and assert `"parent": "s1"`.
