## r1

Lane claude/keys-route, from 892d035c, head 541b92bb (merged with origin/claude/autonomous-dev-session-3w-i33pg7: already up to date). Brief briefs/keys-route-r1.md.

| item | red | fix | pinned by |
|---|---|---|---|
| construct rx-1 (a construct chord took the keys from a face window) | 0ac54f23 | 7125cc62 | unit `input::tests::a_chord_of_the_screens_manager_does_not_take_the_keys_from_a_face_window`, `the_test_mode_keeps_the_keys_of_a_face_window_after_a_chord_of_the_screen`, `the_test_mode_lets_an_injected_chord_choose_its_manager` |
| rw-2 (in_pane with a cube swallows unbound Super chords) | none: already closed at the head | 851e8960 (`regrab` swallows only with `cube.is_none()`, camera.rs:550) | existing `cube::tests_moved::a_pane_and_a_cube_hold_no_unbound_super_chord` (red 8357b193) |
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

## r2

Lane claude/keys-route, from 541b92bb, head e16d558e (pushed). Brief briefs/keys-route-r2.md. Step 0: `git fetch --unshallow`, `cargo check --workspace` green. The gate ran on the local merge 4bdb2c23 of e16d558e with origin/claude/autonomous-dev-session-3w-i33pg7 (e87b0ff9, clean merge; the merge is not pushed).

| item | red | fix | pinned by |
|---|---|---|---|
| rw-1 MAJOR (the early return in `choose_manager_of` reversed M4: a face or pane window with the keyboard and the pointer on a screen manager's claim sent that manager's window into the face) | 18436b85, fails at 541b92bb: `ipc::tests::a_grabbed_key_of_the_screen_chooses_it_over_the_window_of_a_face`, "the pointer is in space: left Some(\"face\")" | ac327bc0 | the same unit (real Super+F1 of `space` on fixture `face_window_with_a_chord_on_the_screen`: new toplevel goes to `space`; then the face's own Super+e reaches `face`, which ends the choice: the next toplevel goes to `face`); the three r1 tests in input.rs and the xp HT-rx-1 test stay green |
| rw-2 MINOR (no gate cell reached the key-made choice) | the script run against the core at 18436b85: M4 part red, see below | e16d558e | tests/gate-keys-route.sh, in step `virtual` of tests/merge-gate.sh |
| rw-3 NIT (report sha) | | the "## r1" table row of rw-2 now names 851e8960, not ff617c83 | |
| rw-4 NIT (doc paragraph on the wrong test) | | ac327bc0 | the paragraph "The same in the test mode, where an injected key counts..." now sits on `the_test_mode_keeps_the_keys_of_a_face_window_after_a_chord_of_the_screen` |

### What changed

- state.rs `choose_manager_of`: the early return on a focus owner that manages only virtual outputs is gone; the M4 choice (pointer or focus on the manager's claim) stands. The choice now carries whether a key made it (third field of `chosen_manager`). A grab of the manager that owns the keyboard focus ends a key-made choice, a click-made one stays.
- input/place.rs `grab_action`: the grabs of the owner of the keyboard focus are never held back (new `focus_owner`, shared with `key_place`), so after a screen chord that chose the screen manager, the face's Super+e still reaches the face. `manages_only_virtual_outputs` is private again, the r1 re-export is gone. The escape hatch (`press_action`) is untouched.
- docs/api.md `input.key`: the paragraph on the chosen manager says it (the face-window exception of r1 is replaced by: grabs of the focus owner never wait, and its chord ends a key-made choice).
- tests/gate-keys-route.sh (test mode `--test-injected-press-is-real`, tiling on the screen, xp on a virtual output shown in a pane; tiling starts first because tiling and xp both swallow every Super chord they do not bind, so xp's Super+e cannot be had beside tiling; the face chord is Alt+F4): (1) a terminal of xp has the keyboard, the pointer on the screen, Super+Ctrl+r (tiling's reload, no window) chooses tiling, then Alt+F4 must close the window of xp (rx-1: not swallowed for the chosen tiling); (2) a new face terminal has the keyboard, Super+Return of tiling must open its window in tiling, not in xp (M4). Waits are `until_ok` with bounds, no bare sleep. The brief named Super+e then Super+Return in one run; Super+e is tiling's here, and a window that tiling opens takes the keyboard, so Super+e would then wait for tiling by design, hence the two parts.

### Red for the gate step

At 18436b85 (core without the fix, the same binaries otherwise) the step is red on part 2: "FAIL: M4: Super+Return of tiling with the pointer on its claim opened its window in xp, which has the keyboard". Part 1 is green there (the early return kept the keys of the face) and at the fix; it pins the other side (at 892d035c it would be swallowed). Green at ac327bc0: "ok: rx-1: Alt+F4 reaches xp ...", "ok: M4: Super+Return of tiling opens its window in tiling ...".

### Gate

`tests/merge-gate.sh --jobs 1`, no `--only`, on 4bdb2c23: all 49 steps PASS, no SKIP, no FAIL line. Among them test 240 s (`cargo test --workspace` all ok: chora 648, chora-construct 265), matrix 348 s, virtual 55 s (with gate-keys-route.sh), acts 1495 s, e2e 304 s. Also run alone: `cargo test -p chora` 648, `-p chora-xp` 326, `-p chora-construct` 265 green. No red cell.

## Review r2

Seeded review of claude/keys-route e16d558e against 541b92bb (brief briefs/review-keys-route-r2.md). Step 0 green (`git fetch --unshallow`, `cargo check --workspace`). At e16d558e `cargo test -p chora ipc::` 152/152 and `input::` 60/60 green. No merge gate run (the lane ran it green on 4bdb2c23, 49/49).

- Check 1 (rw-1, ac327bc0): holds. The early return is gone; a key of a manager whose claim holds the pointer or the focus chooses it also over a face or pane window (third field of `chosen_manager` = by key). A grab of the focus owner ends only a key-made choice; a click-made one stays (and a click takes the keyboard anyway, sh-23). No held chord survives a manager change: the choice changes only in `take_grab` before `hold_grab`, and the release follows `held.grabs`, not the place. The unit pins both sides: F1 of `space` chooses `space` (M4), Super+e of `face` reaches `face` (needs the `grab_action` exemption: without it `space` is the place and the face is held back) and ends the choice (`three` goes to `face`).
- Check 2 (place.rs): holds. A binding has one grab (ipc/grabs.rs:51, "is grabbed already"), and `grab_for_where` returns at most one connection, so a key goes to exactly one client, or to none by `Swallow` when its one grabber waits; the exemption only lets the focus owner's own grab fire, which the raw key would have reached anyway. `key_place` and `grab_action` call the same `focus_owner` (live, holds a claim, no window while a layer surface has the keyboard). `press_action` (VT switch, Quit, Reset, RevokeAll) is untouched and still decided before `grab_action`.
- Check 3 (red first): 18436b85 precedes ac327bc0; `cargo test -p chora --lib a_grabbed_key_of_the_screen_chooses_it_over_the_window_of_a_face` at 18436b85 FAILS: "the pointer is in space, left: Some(\"face\"), right: Some(\"space\")".
- Check 4 (gate-keys-route.sh in step `virtual`): reaches the key-made choice in the test mode (`--test-injected-press-is-real`). The deviation is justified by the code: tiling starts first and its swallow list (chora-kit swallow.rs KEYS) grabs every unbound Super chord, Super+e included, and a binding has one grab, so xp cannot hold Super+e beside tiling; Alt+F4 is xp's (xp config.rs:420), Super+Ctrl+r tiling's reload. Part 1 pins the exemption (red at 892d035c: Alt+F4 held back for the chosen tiling), part 2 pins M4 (red at 541b92bb and at 18436b85 as reported: the early return sends the terminal to xp). Waits are `until_ok` with bounds (40, 50, 60), `ctl wait --timeout`; no bare sleep outside the helper.
- Check 5: docs/api.md `input.key` changed in ac327bc0 with the code and states the rule (choice over a face or pane window; grabs of the focus owner never wait; its chord ends a key-made choice). rw-3 fixed (the "## r1" row names 851e8960), rw-4 fixed (the paragraph sits on `the_test_mode_keeps_the_keys_of_a_face_window_after_a_chord_of_the_screen`).

Findings:
- rw-6, MINOR (pre-existing since 892d035c), crates/chora/src/state.rs:991 and docs/api.md:438: a key-made choice from a chord that opens no window (Super+Ctrl+r) lingers while the face window keeps the keyboard; ungrabbed keys typed into that window do not end it, so a window started from the face's terminal opens in tiling, while api.md says the choice ends "with the next press of the user that chooses nobody". No stray key (the keys go to the focused window), a stray window. Fix: end a key-made choice on a real ungrabbed press that goes to the focus owner's window, or say in api.md that only button presses, grabbed chords of the focus owner and a focus change end it.
- rw-7, NIT, crates/chora/src/state.rs:995: `choose_manager_of` computes the focus owner its own way (no `layer_has_keyboard` filter, no live and claim filter) instead of `place::focus_owner`, so the owner whose chord ends a key-made choice can differ from the one whose grabs never wait (a clicked layer surface with the keyboard while `focus_wanted` keeps a face window); it changes only whether a choice ends, not where a key goes. Fix: make `place::focus_owner` `pub(crate)` and use it in `choose_manager_of`.
- rw-8, NIT, tests/gate-keys-route.sh:13: the header says part 1 shows that "no key stays chosen for tiling", but the script asserts only that Alt+F4 closed the xp window (the close moves the focus, which ends any choice anyway); the end of a key-made choice is pinned by the unit only. Fix: drop that clause or name the unit that pins it.

Verdict: MERGE-OK (0 BLOCK, 0 MAJOR; 1 MINOR, 2 NIT).
