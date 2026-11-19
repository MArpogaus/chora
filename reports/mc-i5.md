# mc-i5: the key column row no longer overprints (net3-m1, I5)

## r1
Branch claude/mc-i5 from ee53e09c, head f7a69761 (3 commits, pushed).

Cause (not the row height of the layout, as kit3 guessed): `Mc::update` (plugins/chora-mc/src/region.rs) measures the new texts on hidden probe copies, and it gave the probes `max_width` = the width of the whole side. The items themselves wrap at the side less their left place (`width - item.x - lead`). The words "Start a program or run a command" stood in one line for the probe, so the layout put the next row one line below, and on the screen they wrapped to two lines over "Chat with a helper". Only the update path (the same items, new texts, e.g. after a window closes) was wrong; a fresh build used the right width.

Commits:
1. f1a52402 test(mc): red. `a_wrapped_row_of_the_key_column_leaves_room_for_the_next_row`: FakeCore with a planted `scene.query` responder that wraps text (0.6 em a letter, 1.2 em a line) at `max_width`, output 1280 x 800 (the gate), `show` then `regions()` (update path), and each words row must end above the next key row. Fails: `row 2 ("Start a program or run a command", 2 lines) ends at 616 over the next row at 588` (the 616 and 588 are the y of the act's breach).
2. 1a5d744b fix(mc): one fn `room(item, width)` for the item and its probe. The I5 row of mc is removed from tests/acts/known-red.toml in this commit. No new wording; spacing is the grid of the column's other rows.
3. f7a69761 test(acts): tests/test_acts.py `test_a_second_overprint_of_the_mc_key_column_blocks` read the real mc I5 row (the harness selftest of the gate step `acts` failed without it; `--no-selftest` hid it). Removed it and its helper; the `match` of a row stays tested by `test_match_limits_a_row_to_its_breach`.

Checks:
- `cargo test -p chora-mc`: 133 pass (the new test red on f1a52402, green on 1a5d744b). `cargo clippy -p chora-mc --all-targets -- -D warnings` clean.
- `tests/acts/run.py --no-selftest mc`: `cell mc I5 pass`, 33 pass, 0 known red, 0 blocking.
- Capture of the key column, `choractl capture` of the headless session at 1280x800 after the window act: reports/mc-i5-keys.png (on claude/session-log). "command" stands on its own line and "Chat with a helper" below it, no overprint.
- `tests/merge-gate.sh --jobs 3`: stopped at `xp` ("timeout: said Notify Starting slow-manager", a 3-job load flake of the xp notify cell, nothing near mc); `xp` reran alone with `--only xp`: PASS (87 s). Steps before it passed in that run (red-first .. xp-fields). The steps it had not reached ran with `--only` (matrix, hud, layout, tiling-audit, chat-keys); there `acts` failed on the selftest above (fixed by commit 3); then red-first, pre-commit, tiling-look, doors, acts, last-words, cube, pull, placed, construct-hud, out, winit, wm-pane, pane-keys, walk, keys, pixels, mc-paths, mc-sofa, mc-handover, e2e all PASS on f7a69761. No step red on the final head; the gate was not one single run (the xp flake and the selftest finding).

Open: none for I5. The mc row of known-red.toml is gone; the cube row (co-21, I13) stays.
