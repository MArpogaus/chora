# ux r2 (rc 1): the review of ux r1 closed, claude/ux-r1 at 90513573

Gate: PASSED as one run of `tests/merge-gate.sh` on 90513573, the merge of the session branch c2d0a116
(train 11, 63ab1724) into claude/ux-r1: 27 steps, first run, no rerun, no flake (F16/F17/F18 did not show).

1. M1 fixed (22e28034 red, 1a732e23): the text is in `Mode::List(picks, Option<String>)`; `Session::held` is
   gone. `show` sends no draft; only `pick` (a click or a number) takes the text and sends it. Red: 4 unit
   tests with the fake Link, `sends` 1 not 0: hide then show (the reviewer's scenario, the question balloon),
   Escape then show with a token (chat.show), a show with a name over the list, the only-helper path. Green.
2. m2 fixed (cc806b4b red, a32eb349): after a text a helper name is text ("ask atlas", "atlas"); a number
   that is no entry ("hallo9", "9", "0") comes back as text with the words "Pick a helper first". KISS: the
   existing notice, no new word or message. Red: 2 unit tests. Green.
3. m3 fixed (9bed45ef red, 6009cebc): only the side that holds the key list gives way to all keys; Left and
   Drawer keep their content. KISS: one rule in `items`, and the docs already said "the column".
4. m6 (89b715b6, test only): header-SIZE waits for two equal readings, then each reading of one second
   counts; an overlap or a cut date fails. Green at 1280x800 and 1024x768, 2 runs each; red shown on forged
   readings. The first run was red at 1024x768 on mc's hidden `-probe` copies (opacity 0): excluded by id.
5. c1 (12477d48): unit test for give::keys_row. Docs: hud.md, voice.md, configuration.md, mc-style.md.
Open: the mc page head is still cut at 1024x768 (`7  CHORA-…`), old. After Escape the field keeps the unsent
text as a visible draft when the last helper opens again. Train 12 (98291088) is not merged here.
