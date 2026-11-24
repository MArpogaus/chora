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

## r2

Lane: claude/verify-minor-r2 at 2235551f. Read only. `cargo check --workspace` and `cargo build --workspace` pass. `tests/xp.sh target/debug OUT` ends `== 16c. the start menu opens the Turn Off Computer dialog and Run` / `PASS`; `tests/gate-mc-column.sh` ends `ok: closing the chat brings the key list back` (light and dark ok). Scene reads: the `states.json` of `python3 tests/acts/run.py` act xp-I6 (xp look, 1280x800, per-text `px`), `audit-nodes.json` and `chat-question.png` of the xp.sh run. The run of all four looks was stopped after xp (only xp-I6 evidence needed; no `--only` flag exists in tests/acts/run.py or demo/tour/run.py for this). `pgrep -fa 'chora --headless'` is empty at the end.

| id | sha | check | line | proposed state |
|---|---|---|---|---|
| xp-5 | 20401efb (installed filter, kit) | chora-kit desktop_entry::tests::only_a_program_that_is_there_counts_as_installed and launcher::tests::the_list_is_sorted_and_holds_only_installed_programs ok; xp apps.rs:86-88 filters `e.installed(on_path)`; here LibreOffice is installed (/usr/bin/soffice), so the xp-I6 list shows it legitimately | `xp-item-3 11 'LibreOffice' [38,13,52,11]` (soffice present) | fixed 20401efb (guard desktop_entry::tests::only_a_program_that_is_there_counts_as_installed, verify-minor r2); the 30 s "Cannot open" case needs a host without soffice, not made here |
| xp-6 | 1f764da2 | tests/xp.sh step 17b ("a program that cannot start or is slow gets a message that names it") passes: `wait_for said Notify "Starting slow-manager"` | `== 17b. ... PASS` | fixed 1f764da2 (guard tests/xp.sh step 17b, verify-minor r2) |
| xp-7 | 165a1105 | tests/xp.sh step 16 overlap check (balloon vs paperclip rect) passes; text: audit `layout audit: 33 nodes, no finding`; xp text px in xp-I6 is 11 for chrome | `layout audit: 33 nodes, no finding`; overlap check silent (no `fail`) | fixed 165a1105 (guard tests/xp.sh step 16 "the notification covers the paperclip", verify-minor r2); text-size half is not measured for the notify card, see Seen |
| xp-12 | 5a560e2b | chat-question.png of step 16b: field x 858-1098 y 598-626 inside the balloon 845-1183 x 214-638; paperclip at x >= 1210, no overlap; step 16b asserts chat y <= Yes y and height >= 380; unit the_card_of_a_question_stands_a_gap_above_the_field_when_no_hint_shows ok | field inside balloon, clip right of it | not reproduced (chat-question.png: balloon right edge 1183 < clip 1210, field inside), no guard for the "reopened after the no-helper balloon" sequence |
| xp-14 | 72527394 | chat-question.png of step 16b: "Typing does not answer." at y 576, below the card box (bottom y 560), above the field; field one line (fixed); card text in DejaVu-like sans, Tahoma not installed here | `Typing does not answer.` y 576 vs card bottom 560 | open: reproduced (chat-question.png: typing line outside the card box) although chat::view::tests::the_typing_line_stands_in_the_card_and_not_above_the_field passes; the two-line field half is not reproduced |
| xp-15 | a836cbd6 | startmenu::tests::the_first_rows_of_both_columns_stand_a_gap_under_the_header ok; startmenu::tests::a_long_title_ends_inside_its_row_with_an_ellipsis ok; /usr/share/applications/display-im6.q16.desktop `Name=ImageMagick (color depth=q16)` | Name above | header: fixed a836cbd6 (guard the_first_rows_of_both_columns_stand_a_gap_under_the_header, verify-minor r2); label: open: reproduced (a long title is cut with an ellipsis by design, no wider row) |
| xp-17 | none named | wm.rs:107 `const HELP_TASK`, the only dialog task button (xp.sh 12b asserts Help only); no cell for Sounds or keyboard focus | `src/wm.rs:107 const HELP_TASK: &str = "helppanel"` | open: reproduced for Display (no task button); the Sounds and keyboard-focus halves: verify: needs an act that walks the Start menu with keys (none exists) |
| xp-18 | none named | `chora-probe keys CODE+CODE` (tests/probe/src/main.rs:555) holds keys down and lets them go in reverse at once; no call that keeps Alt held across a capture; tried `grep -rn "hold" tests/acts/harness.py` | none | verify: needs a held-key helper |
| xp-34 | none named | xp-I6 states.json | `xp-tray-HEADLESS-1 11 '7:59 AM' [77,11,40,11]` (11 px, floor 14; scene `xp-tray-winit` image 128x30) | open: reproduced (11 px under the 14 px floor; the replica size of xp-39, design decision) |
| mc-12 | 675ebd55 | gate-mc-column.sh line 242 `has_text "NOTIFICATIONS" && fail "mc has a NOTIFICATIONS head"` passes; chora-mc audit tests (every_baseline_of_a_page_stands_on_one_grid, the_last_line_of_doors_stands_on_the_baseline_of_the_footer) ok; r1's name items_of_one_row_share_a_baseline_and_rows_have_a_gap is not in the head (0 matched) | `ok: notify.push shows in chora-notify and mc draws no column of notifications` | fixed 675ebd55 (guard tests/gate-mc-column.sh "mc has a NOTIFICATIONS head", verify-minor r2) |

Counts: fixed 5 (xp-5, xp-6, xp-7, xp-15 header half, mc-12); not reproduced 1 (xp-12); open 4 (xp-14, xp-17, xp-34, xp-15 label half); verify 1 (xp-18).

## Seen

- chat::view::tests::the_typing_line_stands_in_the_card_and_not_above_the_field passes, yet the capture chat-question.png of tests/xp.sh step 16b shows "Typing does not answer." below the card box: the unit test and the real layout disagree.
- The notify card text (xp-7) is not in the xp-I6 texts list; its px is unmeasured.
- tests/acts/run.py has no --help or per-act flag: `--help` ran the full selftest and all four looks.
