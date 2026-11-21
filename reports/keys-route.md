## r1

Lane claude/keys-route, from 892d035c, head 541b92bb (merged with origin/claude/autonomous-dev-session-3w-i33pg7: already up to date). Brief briefs/keys-route-r1.md.

| item | red | fix | pinned by |
|---|---|---|---|
| construct rx-1 (a construct chord took the keys from a face window) | 0ac54f23 | 7125cc62 | unit `input::tests::a_chord_of_the_screens_manager_does_not_take_the_keys_from_a_face_window`, `the_test_mode_keeps_the_keys_of_a_face_window_after_a_chord_of_the_screen`, `the_test_mode_lets_an_injected_chord_choose_its_manager` |
| rw-2 (in_pane with a cube swallows unbound Super chords) | none: already closed at the head | ff617c83 (`regrab` swallows only with `cube.is_none()`, camera.rs:550) | existing `cube::tests_moved::a_pane_and_a_cube_hold_no_unbound_super_chord` (red 8357b193) |
| HT-rx-1 (xp icons take the keys from xp's own Start menu) | 1617cfa6 | 20e849da | unit `wm::state::tests::the_desktop_icons_leave_the_keys_of_the_start_menu_of_xp`; gate-handover-shared.sh step "rx-1" (cell `matrix`), red at 1617cfa6: "FAIL: rx-1: the window of tiling went and the desktop icons of xp took the keys from its open Start menu" |
| CO-P2-FIX r1 (xp opens Start on a swallowed Super chord) | none: already holds | 06c900b6 (xp swallows unbound Super chords on a face, wm.rs:836) with the core's rw-1 (a grab of another client ends xp's tap) | pin 541b92bb `wm::tests_moved::keys::a_swallowed_chord_on_a_face_opens_no_start_menu` (green on the first run) |

### What changed

- construct rx-1, core: `input::key` now chooses the manager of a key grab also for an injected key when `injected_press_is_real` holds (new `take_grab`, input.rs; the button path already did so at input.rs:713). The choice itself: `State::choose_manager_of` (state.rs) chooses nothing when the window with the keyboard belongs to a manager of virtual outputs only (`manages_only_virtual_outputs`, input/place.rs, split out of `holds_other_claim`), because the keys already follow that focus and the choice would hold back every grab of that manager. A key chord is the person's key: it goes to the one client that binds it, or to none; the manager of the face keeps Super, Super+e, Super+d after Super+F1. The escape hatch (`press_action`: VT switch, Quit, Reset, RevokeAll) is untouched. docs/api.md (the paragraph on the chosen manager) says both changes.
- HT-rx-1: `focus_moved` (plugins/chora-xp/src/wm/keys.rs) adds `!self.own_node()`; `own_node` is `pub(super)`. The gate step opens the Start menu of xp over the window of tiling, kills the foot window by its pid, and waits (`until_ok`, bound 2 s) for the icons to take the keys; with the fix they do not and the menu stays open.

### Not pinned by a gate cell (construct rx-1)

A gate cannot reach the choice in `pull`: in use mode construct's claim does not hold the pointer or the focused window (the temporary log in `choose_manager_of` read `claims false false` with the pointer on a face and with it on a second output; the cube fills the screen), so the choice never fires there. The test mode alignment is in, so a cell that puts the pointer on a claim of construct with a window of xp focused would reach it. The unit tests on the core fixture pin it. Cells injected-presses, handover-shared, acts and pull stay green.

### Gate

`tests/merge-gate.sh --jobs 1`, no `--only`, at 541b92bb: all 49 steps PASS, no SKIP line (red-first, identity, pre-commit, gate-lib, ci-check, build, test 219 s, clippy, 2d-guard, ..., acts 1277 s, pull 84 s, matrix 334 s, ..., e2e 293 s). `cargo test -p chora` (647), `-p chora-xp` (326) and `-p chora-construct` green in the `test` step. No red cell.

### Left open

- Gate reach of the construct rx-1 choice (above): needs a persona cell with the pointer on a claim of construct and a window of xp focused (plugins/chora-construct claims, crates/chora/src/state.rs:986).
- xp on its own screen does not swallow unbound Super chords (wm.rs:836 only for a face): a Super+q typed into a window of xp on a screen still reaches the window and opens Start on the release. By design today (docs/cube-modes.md:71); not a stray key of this round.

## Review r1

Seeded review of claude/keys-route 541b92bb against 892d035c (brief briefs/review-keys-route-r1.md). Step 0 green (`cargo check --workspace`). `cargo test -p chora input::` 60/60 green at 541b92bb. Red first holds: 0ac54f23 before 7125cc62 and 1617cfa6 before 20e849da; at 0ac54f23 `a_chord_of_the_screens_manager_does_not_take_the_keys_from_a_face_window` and `the_test_mode_lets_an_injected_chord_choose_its_manager` FAIL, at 1617cfa6 `the_desktop_icons_leave_the_keys_of_the_start_menu_of_xp` FAILS. Escape hatch (`press_action`) untouched: only the `Grab` arm of `input::key` moved into `take_grab`. docs/api.md changed in 7125cc62 with the code. HT-rx-1: only the `!self.own_node()` term changed in `focus_moved`; the gate step uses `until_ok` with bounds (its own `sleep 0.25` sits inside the bounded helper, as in gate-chat-keys.sh). rw-2 of CO-P2-FIX r2 holds at 892d035c (camera.rs:550 `keys_with_pane() && cube.is_none()`, red 8357b193, fix 851e8960). CO-P2-FIX r1 holds (wm.rs:836 swallow on a face, pin 541b92bb; any chord during the tap marks it used, keys.rs:179; the core ends xp's tap on another client's grab).

Checked by a scratch unit test (not committed): the fixture `face_window_with_a_chord_on_the_screen` plus a real Super+F1 of `space` and a new toplevel: at 892d035c the new window is reserved for `space` (pointer, M4); at 541b92bb for `face`.

- rw-1, MAJOR, crates/chora/src/state.rs:1001: the new early return reverses M4 for every manager of virtual outputs: with a window on a face or in a pane focused and the pointer on a screen manager's claim, that manager's chord (Super+Return of tiling) now opens its window in the face or pane (scratch test above), a window in the look the person did not point at; no test pins this and `a_grabbed_key_of_the_user_chooses_its_manager_where_the_person_is` (ipc.rs:2876) has no face case. Fix: keep the choice in `choose_manager_of`, instead let `grab_action` (input/place.rs:31) never hold back the grabs of the keyboard focus owner and let a grab of the focus owner end a key-made choice, then add the face case to the M4 test.
- rw-2, MINOR, tests/ (no cell): no gate cell reaches the key-made choice (construct's claim never holds the pointer in use mode); the unit tests drive the real `input::key` path and pin the rule enough for the RC bar, but with rw-1's fix a cell in the test mode (tiling on the screen, xp in a pane, injected Super+Return with the pointer on tiling, then Super+e) should pin both sides. Fix: add that step to gate-handover-shared.sh or injected-presses with the rw-1 fix.
- rw-3, NIT, reports/keys-route.md "## r1" table: the rw-2 fix sha is 851e8960 ("swallow in a pane only when no cube is open"), not ff617c83. Fix: correct the sha.
- rw-4, NIT, crates/chora/src/input.rs:2844: the doc paragraph "The same in the test mode, where an injected key counts…" sits on `the_test_mode_lets_an_injected_chord_choose_its_manager`; it belongs to `the_test_mode_keeps_the_keys_of_a_face_window_after_a_chord_of_the_screen`. Fix: move it.
- rw-5, NIT, crates/chora/src/input.rs `the_test_mode_keeps_the_keys_of_a_face_window_after_a_chord_of_the_screen`: green at its red commit 0ac54f23 (an injected key chose nobody there), so it guards `take_grab` only after 7125cc62; the other two tests carry the red. Fix: none needed; name it a guard in the report.

Verdict: FIX (1 MAJOR rw-1; 1 MINOR, 3 NIT; no BLOCK).
