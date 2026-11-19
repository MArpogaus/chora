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
