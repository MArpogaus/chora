# xp-fields

## r1

Lane `claude/xp-fields`, rc 2, xp look. Rows: xp-24 (MAJOR, text fields on the XP grid) and ca-1 (Help key list cut at 720). Pixel round 1 of 2 for xp.

**Shas** (branch `claude/xp-fields`, pushed): base a5b77488; red `fc936a43` (tests only); fix `9a6506d6`. Author and committer Marcel Arpogaus, no attribution lines.

**Asserts** (cell `tests/gate-xp-fields.sh`; tools `tests/xp-field-text.py`, `xp-runbox-baseline.py`, `xp-help-rows.py`). Measured on a5b77488 (before) and 9a6506d6 (after), logical px:

| assert | before a5b77488 | after 9a6506d6 |
|---|---|---|
| year field of Date and Time: height 21, border 1 px | 21, 1 (green) | 21, 1 |
| year field: text 3 px inside the border | **1x 4.0 (red)** | 1x 3.0, 2x 3.5 (one device px of the glyph; the assert allows 3 to 3.5) |
| time field of Date and Time: height 21 | **1x 20 (red)** | 1x 21, 2x 21 |
| Wait field of Display, Screen Saver: height 21, border 1 px | 21, 1 (green) | 21, 1 |
| Wait: label and field text on one baseline (+-1) | 14, 14 (green) | 1x 14, 14; 2x 14.5, 14.5 |
| Run box (fuzzel): label and typed text on one baseline (+-1) | 1x 29, 29; 2x 30, 30 (green) | same |
| Help at 1280x720 after a wheel to the end: no ink below the last rule, at least 12 px of padding below it | **last rule 7 px above the pane bottom, 1 row of the row Mute below it (red)** | 37 px, 0 rows of ink |

Asserts already green on a5b77488 stay (no fake red). The 2x values before are not measured: the cell stops at its first red per scale; the 2x column shows the after values only.

**What the rows say**
- xp-24: the shared text place of a field is now `dialog::field_text` (3 px inside the 1 px border, one place; combo, spin, year, time and the time zone box use it) and `FIELD_HEIGHT` 21 px for the time box. The chat box (28 px, level with Send, xp-style 13) is unchanged and its two cells stay green.
- The Run box is fuzzel: it draws no box around its input (xp-style 13), so height 21, border and padding have no pixels to measure there; the baseline of its label and typed text is asserted. The brief names a "search field": at a5b77488 xp has none (the entry Search... of the Start menu is gray, Help has no search, the launcher is the Run box). If the owner meant another widget, it needs naming.
- ca-1: the table of Help scrolls by lines when it is higher than its window: a Luna scroll bar (arrows, track, thumb; the helpers moved from display.rs to dialog.rs) and the wheel over the page (`pointer.scroll` joins the events of xp, an injected wheel counts). At 800 the table fits and has no bar (unit test `a_table_that_fits_has_no_scroll_bar`; a second test scrolls at 690 to the last row). The last row can stand up to one row above the bottom (scroll by whole lines).

**Gate** `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,xp,xp-fields,xp-faces,xp-question,xp-notepad,acts` on 9a6506d6: pre-commit, ci-check, build, test (204 s), clippy, xp, xp-faces, xp-fields, xp-question, xp-notepad, acts all PASS (partial run: nothing pushed by the gate). A first run failed `lint-ratchet` (allow 361 against the list 360): a redundant `#[expect]` of mine in help.rs; removed (the file allows the cast at its top) and the fix commit amended before the push.

**Captures** `reports/evidence/xp-fields-r1/`: runbox, datetime, display (Screen Saver), help-720, each `-before-a5b77488` and `-after-9a6506d6` (display before as well; the Run box capture is the one with typed text, 1x). The Run box looks the same before and after.

## Review r1

Reviewer: cloud lane (sonnet), read-only on `claude/xp-fields` 9a6506d6 (base a5b77488, 11 files, +582 -115). `cargo check -p chora-xp` ok, `cargo test -p chora-xp` 319 passed. No gate run (the lane ran it green).

**Verdict: MERGE-OK** (0 BLOCK, 0 MAJOR, 5 MINOR, 1 NIT).

Checks:
1. Red first: fc936a43 holds only the 4 test files and comes before 9a6506d6; its reds (year pad 4 px, time field 20 px, Help last rule 7 px above the pane bottom with row Mute cut) match the code (`YEAR[0]+5` text rect, `TIME` height 20) and the before capture (Mute cut at the pane bottom). Not rerun.
2. Asserts are on real pixels (field height from border rows #7F9DB9, thickness, pad, baselines, rule/ink rows). A missing widget fails: year, Wait, Run box typed text and Help `none` call `fail`; a missing time field reads empty and goes red. Only the Run-box baseline can skip (rxf-3).
3. `dialog::field_text` serves year, time, time zone box, Wait and every `dialog::combo`; no copied field text left except the month box (rxf-1). Scroll helpers moved once to dialog.rs; display.rs and help.rs both use them.
4. The report's "unmeasured" claims hold: the Run box capture shows no box (docs/xp-style.md 13: fuzzel draws none); xp has no search field (`grep` finds no input widget besides the chat box, measured at 28 px by the earlier cell, and the question card). Unmeasured fields: see rxf-1, rxf-4.
5. ca-1: help-720-before shows Mute cut at the pane bottom; help-720-after shows the Luna bar, all rows to Mute, 37 px below it. Wheel is asserted in the cell, the bar click is not (rxf-2). At 800 and above the table fits by `fit()` (unit test at 800; no capture at 1024, rxf-6).
6. Elsewhere unchanged: pixel diff of display-before/after and runbox-before/after is the clock only; datetime differs in the year text (1 px) and the time field only. No core change; `pointer.scroll` only joins the xp event list and `scrolled` ignores every panel but Help.

Findings:
- rxf-1 MINOR plugins/chora-xp/src/datetime.rs:203: `month_box` still draws its own field and text (`Text::new(name, ... [5, 0, w - 26, h])`), not `dialog::field_text`, and no assert measures it; fix: draw its text through `field_text` (blue selection inset 3 px) or add the month box to the cell.
- rxf-2 MINOR plugins/chora-xp/src/help.rs:348: `click_bar` copies the arrow/page/track logic of display.rs:557-575 and has no unit test (the list has `the_scroll_bar_scrolls_the_list`); the thumb cannot be dragged; fix: add a test for an arrow and a page click, and share one bar-click helper in dialog.rs.
- rxf-3 MINOR tests/gate-xp-fields.sh:112: the Run-box baseline assert lives in `runbox_cell`, which `need_fuzzel 1.11` skips (an environment with fuzzel 1.9.2, see xp-36, never checks it); fix: print an explicit SKIP line that names the baseline assert so a green cell is not read as measured.
- rxf-4 MINOR tests/gate-xp-fields.sh:166: the time field is asserted for height only (not border or padding), the Wait field has no padding assert, and the time zone box, month box and the `dialog::combo` fields of display.rs are not measured (covered only through the shared helper); fix: add border and pad of the time field and one combo to `dialog_cell`.
- rxf-5 MINOR findings/xp.md:25: xp-24 is marked fixed, but the Run box (named by the owner) looks the same before and after and has no field box (fuzzel, xp-style 13); the row says so, yet closes; fix (PM): keep a note "dialogs only; Run box has no box by design" on the row or ask the owner whether a box around the Run input is wanted.
- rxf-6 NIT plugins/chora-xp/src/help.rs:628: the no-bar unit test runs at 800 only; the 1024 case of ca-1 is not tested or captured; fix: assert `max_top() == 0` at 1024 too.
