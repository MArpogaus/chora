# co-keys

## r1 (rc 2, F7; findings/construct.md co-38, co-40, co-41)

Branch `claude/co-keys`, base a76476be plus the session head a5b77488 merged in (a5384bde). Step 0: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace` ok.

Commits (author Marcel Arpogaus, no attribution lines):
- `c98457e8` red: the tour rows cb-1 and cb-3 lose their `known`; `tests/acts/looks/construct.py` `keys_acts` (new acts).
- `e6365f50` fix(core) co-38: `crates/chora/src/state.rs` `focus_of_scene` calls `tell_key_node` before it emits `focus.changed`; unit test `a_node_that_takes_the_keyboard_is_told_before_the_surface_goes` (crates/chora/src/ipc.rs).
- `f5bdc61f` fix(construct) co-40, co-41: `plugins/chora-construct/src/overlay.rs` (`help_takes_keyboard`, `hint_items`), `camera.rs` (`grab_keys`), `construct.yaml`, docs, acts known-red row for co-39.
- `e965470a` test(doors): the cell `f1-free` of construct (head).

**Causes**
- co-38 (cb-1): the step back sends `release()` then `FocusNode(look)` (`modes.rs:757` `the_keyboard_to_the_look`). The core told `focus.changed{null}` at once and `focus.node` only in `after_events` (`state.rs:521` `tell_key_node`). xp (`plugins/chora-xp/src/wm/keys.rs:333` `focus_moved`) heard the surface go while its `key_node` was still empty, saw a foreign focus leave, and sent `desktop_keys()`, so the core focus node was `xp-icons` of xp; construct still said it held the focus (`state.rs:66`). Seen in the tour: focused node `xp-icons` after the key, `construct-look` after the fix. The pulled window must be a foreign focus of xp (pane by drag); `cube.pull` by method does not show it (xp keeps the window in its book).
- co-40 (cb-3): not a consequence of co-38 (cb-1 green, cb-3 still red after the core fix). After the pane went back to its face while the cube stood back, xp activated `s1` (core focus `s1`, construct holder null). `help_takes_keyboard` (`overlay.rs:324`) took the keyboard only in a face or a pane, so in the stood-back cube Escape and Return went to `s1`. It now takes it with any cube (no `help_back` while stood back, so the close does not move the keyboard to a pane).
- co-41: `F1` was no binding (`construct.yaml`, test `super_f1_shows_the_help_and_f1_is_free`), and no hint named a key list in a face or in the stood-back cube.

**Change.** `F1: keys` beside `Super+F1` (same `construct-help`). In the walk and stood back it is a key of the look; in a face it is a grab: the core grabs a key without a modifier only with the grant `input.grab-any` (refused: logged once, F1 reaches the program, as before), so `grab_keys` adds `F1`. The hint `F1 All keys` (`construct-hint-keys`, the word `Word::AllKeys` of the walk) is in the Set::Flown capsule (face, 3 s or the resting pointer) and in the Set::Cube capsule of the stood-back cube, and a click opens the list. Not a "Keys F1" chip of the mode row: the hint capsule is the existing row of both places.

**Red on a76476be (3 of 3 each)**
- (a) cb-1 and (b) cb-3: `demo/tour/run.py --check construct` with the three `known` removed, built binaries of c98457e8 in a worktree (`TOUR_REPO`): red act 4 (I3 construct and xp each say they hold the focus), act 12 (open ['construct-help'] after Escape), act 13 (mode navigate, expected use), in all three runs.
- (c) acts, three runs: `F1 opens the key list in a face`, `the chip Keys of the stood back cube opens the key list by a click`, `the hint Keys of a face opens the key list by a click` red (then the Escape closes the cube, which cascades into I1/I14/I4 and later cells; 59 pass, 9 blocking each time).
- Not red in the acts: the acts of (a) and (b) (`Alt+Escape with a pane in the space leaves one client with the focus`, `Escape shuts the key list of the stood back cube`, `Return steps in after the key list shut`) pass on a76476be too, as the acts cannot make xp see a foreign focus (see co-38 cause); they stay as guards. The core test of the order was not run red.

**Green (e965470a)**
- `tests/acts/run.py construct`, 5 runs: 72 pass, 1 known red, 0 blocking each (the known row is co-39, I5 in `Super+F1 shows the key list of the stood back cube`).
- `demo/tour/run.py --check construct`, 2 runs: 18 acts, 17 green, 1 known (act 11, cb-2 co-39, unchanged), 0 red, `tour: check: 0 red, 0 broken`. The `known` of the rows after act 11 went (their overlap is gone with the shut sheet).
- `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,cube,construct-hud,construct-extra,pull,doors,acts`: all PASS (pre-commit 4 s, ci-check 39 s, build 25 s, test 209 s, clippy 17 s, doors 368 s, acts 305 s, cube 117 s, pull 82 s, construct-hud 111 s, construct-extra 159 s). A first gate run before `e965470a` failed `doors`: the cell `f1-free` of construct asserted the old rule (F1 shows no list in the walk); it now asserts the list in the walk and F1 reaching the program in a pane without the grant (the gate stopped there and later cells read `missing`). gate-lib left out as briefed.

**Open / said plainly**
- co-39 (cb-2) stays known: the sheet of the stood-back cube still overlaps the chip row (acts: `Turn left` over `construct-help-body-note`); the new hint widens the capsule, nothing fixed it.
- F1 in a face needs `choractl grant construct input.grab-any`; without it the hint click is the way. The acts grant it in the cell `construct may grab F1 without a modifier`.
- A stood-back cube whose pane went back to xp still loses the keyboard to xp until the next key list or step; the brief did not ask for more.
