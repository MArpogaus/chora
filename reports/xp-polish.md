# xp-polish

## r1

Lane: claude/xp-polish, from 2235551f to 64478c66 (pushed). Three MINORs of verify-minor r2, each with a red commit before its fix. Findings rows (xp-14, xp-15, xp-17) are left to the PM.

### Commits

| sha | what |
|---|---|
| 11007d3a | red: three unit cells and step 12b of tests/xp.sh (fail on 2235551f) |
| 86343e3f | fix xp-14: the typing line stands in the card |
| 63fff561 | fix xp-15: a cut Start menu name shows in full in a tooltip |
| a0c8ce36 | fix xp-17: Display Properties has a task button |
| 2e777d0e | test: xp.sh step 12b opens Display from the Control Panel and asserts the task button (and its removal); step 17c reads the tooltip of a long program name |
| 64478c66 | test: gate-xp-question asked for the old hint under the card; it now asks for the line in the card and no hint outside it |

### Red cells (on 2235551f, the red commit 11007d3a)

- `chat::view::tests::no_hint_of_the_typing_line_stands_under_the_card_of_the_xp_balloon` and `card::tests::the_xp_card_holds_the_line_that_typing_does_not_answer`: failed. The old unit test read the margin look (a plain view in a slot); the xp balloon draws the line as a hint between the card and the field, and `page_rows` left it out of the xp card (`!face.is_xp()`). The new cells read the hint of the xp balloon frame and the laid text of the xp card.
- `startmenu::tests::a_cut_title_has_its_full_name_for_a_tooltip`: failed (stub returned none).
- `wm::tests_moved::press::display_properties_has_a_task_button_that_raises_it`: failed (no `xp-task-displaypanel-X` create on open).
- tests/xp.sh 12b failed on the missing task button only after the wiring; with the fix it passes.

### The fixes

1. xp-14 (plugins/chora-assistant): the card of xp holds "Typing does not answer." as its last line like the card of a page (`card.rs` `page_rows`); `view.rs` `hint_of` and `hint_room` drop the hint while the card stands in the chat (before it was dropped in the margin only). The old test that said "leaves typing to the field" lost that half; the old margin test stays. The card is one line taller, the field is no longer held a hint row below it. tests/gate-xp-question.sh follows (see 64478c66).
2. xp-15 label (plugins/chora-xp, `startmenu.rs`, `wm/windows.rs`): the row still ends with one ellipsis (`a_long_title_ends_inside_its_row_with_an_ellipsis` unchanged); `Menu::cut_title` returns the full name and the scene row of an entry whose title is wider than its text box; `rest_tip` rests the existing xp tooltip on it (500 ms, goes on a press or a leave). No new widget. The text box maths moved to `text_box`/`icon_box` so the drawing and the check share it.
3. xp-17 Display (plugins/chora-xp): `DISPLAY_TASK = "displaypanel"` beside `HELP_TASK`; `sync_tasks` lists it while a Display panel is open; a press raises Display (its Effects above it) and takes the key focus (`focus_display`, as `focus_help`); close removes it (`close_panel` already syncs). Not done: Alt+Tab entry, Sounds, key focus on open (left out by the brief).

### Outputs

- `cargo test -p chora-xp -p chora-assistant`: 351 + 249 pass.
- `tests/xp.sh target/debug OUT`: PASS (12b Display task button, 17c tooltip, 16b card).
- `SCALE=1 tests/gate-xp-question.sh` and `SCALE=2`: ok.
- clippy `-D warnings` clean.

### Gate (tests/merge-gate.sh --jobs 2, head 64478c66, base origin/claude/autonomous-dev-session-3w-i33pg7)

First full run, before 64478c66, stopped at `xp-question` (the gate itself, fixed in 64478c66). Second full run on 64478c66: 55 cells PASS, then `tiling-windows` FAIL ("the slide is not between the places": assertion on tw-s1/tw-s2 positions in the tiling slide; not xp, not assistant; it passed on rerun of the same head, so a flake of a foreign cell). The steps that the run stopped before (watchdog-starved, leak, xp-run, demo, tiling-bar, tiling-theme, tiling-disc, mc-host, e2e) and `tiling-windows` ran again with `--only ... --jobs 2` on the same head: all PASS. Foreign red: `tiling-windows` once (flaky), none other. SKIP: none.

### Captures (captures/xp-polish-r1/ on claude/session-log)

- `chat-question.png`: the card holds "Typing does not answer." under the key lines, the field directly below.
- `menu-row-tip.png`: the Start menu row "ImageMagick (color depth=..." with the tooltip "ImageMagick (color depth=q16) and other settings of the display" (a program with that name set by xp.sh 17c).
- `display-task.png`: Display Properties open with its task button in the taskbar.

`pgrep -fa 'chora --headless'` is empty at the end.

### Seen

- tests/xp.sh could not use `choractl inject` (needs a grant) and no right-click helper exists, so 12b opens Display from the Control Panel cascade with the probe.
- The Run box of 16c stays open in menu-row-tip.png (xp.sh step 16c does not close it).
- The 7b menu is unchanged; the tooltip of a cut name shows at x 4 over the left edge of the menu when the row is at the left; it stays inside the area.

## Review r1

Reviewer lane (sonnet): claude/xp-polish 64478c66 on base 2235551f, 11 files, +277 -43. Step 0 passed: `cargo check --workspace` clean; `cargo test -p chora-xp -p chora-assistant` 351 + 249 pass at head.

Verdict: MERGE-OK (0 BLOCK, 0 MAJOR, 3 MINOR, 3 NIT).

### Checks

1. Red first: at 11007d3a (tests only, ahead of 86343e3f, 63fff561, a0c8ce36) four unit cells fail: `card::tests::the_xp_card_holds_the_line_that_typing_does_not_answer`, `chat::view::tests::no_hint_of_the_typing_line_stands_under_the_card_of_the_xp_balloon` (hint is "Typing does not answer."), `startmenu::tests::a_cut_title_has_its_full_name_for_a_tooltip`, `wm::tests_moved::press::display_properties_has_a_task_button_that_raises_it`; all pass at head. xp.sh red is not shown (rxp1).
2. xp-14: the line is the last row of the xp card (card.rs `page_rows`, `!face.is_xp()` dropped); the hint is dropped while the card stands in the chat (view.rs `hint_of`, `hint_room`); capture chat-question.png shows it in the card with the field directly below. gate-xp-question.sh: dropped asserts are "card showed without the hint", "card bottom <= hint top" and "hint shown once", all about the removed hint; replaced by the card text contains the line and no `hud-chat-hint`. Kept: card in balloon, Yes/No in card, 28 px, under the transcript, 14 px lines. No assert dropped without a replacement; two are weaker (rxp3).
3. xp-15: `Menu::cut_title` returns a name only when `text_width > text_box` width (the same box the drawing uses, shared `text_box`); uncut rows give None (unit cell); `rest_tip` uses the existing xp tooltip (500 ms DELAY, cancel on leave or press); xp.sh 17c asserts the full text and that no tooltip stays after the menu closes. menu-row-tip.png reads correct.
4. xp-17: `DISPLAY_TASK` beside `HELP_TASK`, `sync_tasks` rebuilds the list from the panels (no second button on reopen), `close_panel` syncs (removal asserted in the unit cell and in xp.sh 12b via Cancel); a press raises Display with its Effects. display-task.png shows the button.
5. KISS/POLS: no new mode or setting; no new widget; the other costumes are untouched: card.rs changes only the xp branch of one condition; glass chat uses `self.hint` and a fixed card origin (`card_origin` returns early for Glass), so `hint_room` does not move its card; the margin look drops the hint as before (`chat::view::tests` margin cell stays; `the_xp_card_keeps_its_keys_under_the_buttons` split, not weakened).
6. The diff touches no file the tiling-windows cell runs (gate-tiling-focus/follow/doors/swallow/gaps/motion, gate-lib.sh: none changed, none loads chora-xp or chora-assistant in a changed path), so the one "slide is not between the places" (gate-tiling-motion.sh:63) is foreign.

### Findings

- rxp1, MINOR, tests/xp.sh:409-419 (at 11007d3a), the red of the xp.sh cells is unproven: 11007d3a drove 12b with `choractl inject`, which the lane report says needs a grant, and step 17c (tooltip) has no red commit at all (it came with 2e777d0e after the fix); only the four unit cells are shown red. Fix: say in the report that the unit cells carry the red, or run 12b and 17c at 11007d3a with the probe.
- rxp2, MINOR, plugins/chora-assistant/src/chat/view.rs:610 and :633 with card.rs:372, the hint is dropped for every card in the chat but the xp (and page) card holds the line only when `buttons && !too_long`; an xp card without buttons (voice.confirm with no `"buttons": true`) or a too-long one now shows no "Typing does not answer." at all (read from the code, not run). Fix: drop the hint only when the card holds the line, or let `page_rows` add the line whenever the card shows in the chat.
- rxp3, NIT, tests/gate-xp-question.sh:61-71, the old assert that the card stops above the hint is gone and nothing asserts the card stops above the field; `contains` does not say "once". Fix: assert the card bottom is at or above the top of `hud-chat-field` (if the node exists) and count the line in the card text once.
- rxp4, NIT, plugins/chora-xp/src/wm/tests_moved.rs:1104, the unit cell opens Display once and closes it; no cell opens it twice (one button) or raises Effects with Display. Fix: reopen after close and count the task creates, and open Effects then press the button.
- rxp5, NIT, captures/xp-polish-r1/display-task.png, with Display on top the Display task button is not shown pressed and the foot button stays pressed (`display_active` needs `focused.is_none()`, and opening Display does not clear the focus; the brief leaves key focus on open out). XP shows the button of the foreground dialog pressed. Fix: leave as the brief says, or clear the window focus on open.
- rxp6, NIT, captures/xp-polish-r1/menu-row-tip.png, the tooltip of a row at the left edge stands at x 4 above the row and covers the line of the row above; XP draws it under the pointer. This is the existing xp tooltip placement, not new. Fix: none for this branch; note for a tooltip polish.
