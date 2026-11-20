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
