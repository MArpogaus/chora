# stray-grab (sh-24, sh-23 focus half)

## r1

Branch claude/stray-grab from bb16f2ff, head 9859071b (merge of the session branch, pushed).

Found on the base: most of sh-24 was fixed by earlier lanes (573f788c: a held-back grab is swallowed, not passed on; 767249d7: the key place follows the keyboard before the pointer). Left open: injected keys skipped the place rule entirely, and the click half of sh-23 (the old look's window kept the wl_keyboard after a click into another look's empty pane, so typed keys strayed into it; the grab half already routed Super+Return to tiling, gate pane-keys).

1. sh-24, injected keys follow the keyboard place.
   Red 25ecc50c, fix cc940fec. Test `input::tests::an_injected_key_follows_the_keyboard_place_as_a_real_one` (keyboard on a node of the screen manager; a face manager's Alt+y, real and injected, is swallowed: reaches no client). Rule: place = manager a click chose, else owner of the keyboard focus, else (real keys only) the claim under the pointer; an injected key never follows the pointer. Gate cells: test, keys.
2. sh-23 focus half, a click into another look's empty claim takes the keyboard.
   Red 1ce94726 (unit) and 9821d96f (gate), fix 363276e8 (`State::choose_manager`). Tests `input::tests::a_click_into_the_empty_claim_of_another_look_takes_the_keyboard`; `tests/gate-construct-pane-keys.sh` now opens a terminal in the xp pane first (the tour), clicks the empty tiling pane, checks no xp surface is focused, then Super tap leaves the xp Start menu closed and Super+Return opens a tiling window. Gate cells: test, pane-keys, wm-pane.
3. Docs 01dafc44: docs/api.md (who gets a key; the click takes the keyboard) and a D55 design amendment in docs/decisions.md (2026-11-20).

Gate (`--jobs 3`, pre-commit, ci-check, build, test, clippy, keys, pane-keys, wm-pane, chat-keys, doors, matrix, layout): all 12 PASS on 9859071b (run /tmp/merge-gate.DZs9fS; pane-keys 8 s, wm-pane 21 s, keys 18 s, doors 353 s, matrix 275 s, layout 192 s). No red cell.

Not touched: plugins/chora-tiling drawing (black-pane lane). Open choice for the PM: a real key with no keyboard place still follows the pointer (keyboard-only use with nothing focused); kept, documented.

## Review r1

Reviewer lane review-stray-grab-r1 on 9859071b (base b97369d1). `cargo check --workspace` ok; `cargo test -p chora input::` 51 passed.

Checks:
1. sh-24: pass. An injected key takes `key_place(state, false)`: click choice, else focus owner, never the pointer; with no place it goes to the grab. The safety layer is still decided before `grab_action` for real keys, and injected keys never reach it (input.rs:451).
2. sh-23 focus: pass, except rv-1. A click on a client is never "empty" (surface under it), so it still focuses; a click on the empty claim of the focused look is filtered by `Some(*manager) != focused`. `choose_manager` takes the choice snapshot after the unfocus, so the choice holds.
3. Two screens: the D55 amendment as written, not a regression. It has no virtual-only limit, and a click on the empty area of the same look changes nothing (same guard). With a BELOW layer (wallpaper) under the click, the press is not "empty" (picking.rs:70), so on real outputs the effect shows only without a background client.
4. Pointer rule: cleared. With no keyboard place, the pointer only picks which manager's grab wins. A held-back grab is swallowed, and raw keys go only to the wl_keyboard focus, so no key reaches a client through it. The same holds while a clicked layer surface has the keyboard (window_owner filtered).
5. Red first: pass. 25ecc50c, 1ce94726 < cc940fec; 9821d96f < 363276e8. At 1ce94726 both new unit tests FAIL (rerun here); the gate red was not rerun.
6. Docs: match the code, except rv-3.

Findings:
- rv-1 | MAJOR | crates/chora/src/state.rs:925-944 | "empty" means only "no surface under the press" (`surface_under`, Target::Blocked gives None). So a press on an input node (construct HUD chip, door, hint, any Accept rect) over another manager's claim now takes the keyboard from the focused window. Local repro: the sh-23 unit fixture plus an `input: accept` rect of `space` at the click point; the window loses the keyboard. This breaks "nothing else loses focus". | Fix: clear the focus only when `render::node_under` also finds no input node (tiling draws none, so pane-keys stays green), and add that unit test.
- rv-2 | MINOR | crates/chora/src/state.rs:937 | `choose_manager` still runs while the session is locked. On an output where the lock client has no surface, the press counts as empty and `focus(None)` clears `focus_wanted`, so after unlock no window has the keyboard (by reading, not run). | Fix: return early from `choose_manager` while `self.lock.is_some()`.
- rv-3 | NIT | docs/decisions.md:233, docs/api.md:424 | The docs say the press takes the keyboard "of the look" and that "the keys go to the manager chosen". The code clears any focused node or window, also one of a non-manager plugin such as the command bar. On real outputs a binding keeps its one owner. | Fix: say "from whatever had it", and that only the grabs of managers of virtual outputs wait for the chosen one.
- rv-4 | NIT | crates/chora/src/input.rs:2639 | No test pins the guard that a click on the empty claim of the look that has the focus keeps the keyboard. | Fix: add a unit test next to `a_click_into_the_empty_claim_of_another_look_takes_the_keyboard`.

Verdict: FIX (1 MAJOR open: rv-1). Counts: BLOCK 0, MAJOR 1, MINOR 1, NIT 2.
