# verify-minor

## r1

Lane: claude/verify-minor at ce866b0d. Read only: no product code changed. Build: `cargo check --workspace` and `cargo build --workspace` pass. Unit tests: `cargo test -p chora-xp -p chora-assistant -p chora-chat -p chora-mc` pass (348 + 247 + 42 + 196 tests, 0 failed); the guard tests below were re-run by name and each printed `ok`. Headless gate cells (tests/xp.sh, tests/gate-mc-column.sh) and captures were not run in this lane: rows that need them are `verify`.

Ancestry: all nine named shas are ancestors of ce866b0d (yes).

| id | sha | in head | check | line | proposed state |
|---|---|---|---|---|---|
| xp-5 | none named | - | no cell; Start list has no installed-program filter test | - | verify: needs headless capture of the Start programs list at 1280x800 |
| xp-6 | none named | - | no cell; no start-feedback test | - | verify: needs capture of a Notepad start (busy cursor or taskbar button) |
| xp-7 | 165a1105 | yes | tests/xp.sh step "notify covers the paperclip" (headless); text floor not measured by any cell | not run | verify: needs headless tests/xp.sh for the overlap; text-size half needs a capture |
| xp-8 | 8e1cf99d (spec now 21 px via fd93053d) | yes | decoration::tests::every_caption_button_fills_its_21_pixels_active_or_not; tests/xp.sh step 3b asserts 21x21, 2 apart, 4 from edge | ok | fixed 8e1cf99d (guard decoration::tests::every_caption_button_fills_its_21_pixels_active_or_not, verify-minor r1) |
| xp-12 | 5a560e2b | yes | chat::view::tests::the_card_of_a_question_stands_a_gap_above_the_field_when_no_hint_shows ok; the balloon-to-field placement is in a tests/xp.sh step (headless, not run) | ok (unit) | verify: needs headless tests/xp.sh for the balloon placement |
| xp-14 | 72527394 | yes | chat::view::tests::the_typing_line_stands_in_the_card_and_not_above_the_field | ok | verify: typing line fixed 72527394; two-line field and card font need a question-card capture |
| xp-15 | a836cbd6 | yes | startmenu::tests::the_first_rows_of_both_columns_stand_a_gap_under_the_header | ok | verify: header gap fixed a836cbd6; the "color depth" label truncation needs a Start menu capture |
| xp-17 | none named | - | no cell | - | verify: needs capture of Start keyboard focus, Sounds and Display items |
| xp-18 | none named | - | no cell; switcher.rs tests cover stepping only, no held-Alt test | - | verify: needs a headless test that holds Alt |
| xp-34 | none named | - | no cell; clock-width.py covers the tiling bar clock, not the xp tray | - | verify: needs one xp capture at 1280x800 for the tray clock height |
| mc-9 | 84f21def, 6902a2d9 | yes | draw::tests::a_field_in_the_column_is_lines_of_the_grid_without_a_box_and_sharp_at_two; draw::tests::a_field_with_a_radius_leaves_its_corners_clear; draw::tests::the_lines_of_a_field_in_the_column_start_at_one_inset; face::tests::the_lines_of_the_margin_fall_on_the_grid_of_28 | all ok | fixed 6902a2d9 (guard draw::tests::a_field_in_the_column_is_lines_of_the_grid_without_a_box_and_sharp_at_two, verify-minor r1) |
| mc-10 | 72527394 | yes | card::tests::a_card_in_a_page_has_no_widow_and_holds_the_typing_line; card::tests::the_last_word_of_a_line_goes_down_to_a_lone_word; audit::tests::no_text_of_the_column_is_set_under_14_pixels | all ok | fixed 72527394 (guard card::tests::a_card_in_a_page_has_no_widow_and_holds_the_typing_line, verify-minor r1) |
| mc-11 | 6902a2d9 (Stop), b50159dd (head) | yes | chat::view::tests::stop_stands_on_the_head_line_left_of_close_and_the_hint_on_the_grid; audit::tests::every_section_of_the_column_has_a_head | ok | fixed 6902a2d9 (guard chat::view::tests::stop_stands_on_the_head_line_left_of_close_and_the_hint_on_the_grid, verify-minor r1) |
| mc-12 | 346a6a98 | yes | test items_of_one_row_share_a_baseline is not in the head; tests/gate-mc-column.sh was changed by 346a6a98 but not run | - | verify: needs a headless MC capture of the NOTIFICATIONS count with the chat open |

Counts: fixed 4 (xp-8, mc-9, mc-10, mc-11); not reproduced 0; open 0; verify 10 (xp-5, xp-6, xp-7, xp-12, xp-14, xp-15, xp-17, xp-18, xp-34, mc-12).

## Seen

- tests/xp.sh step 3b asserts 21 px caption buttons, while the 8e1cf99d message says 24 px. fd93053d moved the chrome to 96 DPI and the test was renamed to 21 px, so the spec is 21 px now. The xp-8 row text (22 px, spec 24 px) is stale.
