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
