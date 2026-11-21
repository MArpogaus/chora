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
