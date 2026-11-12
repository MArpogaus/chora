# Pixel review: mc round 5 (branch claude/mc-pixel @ efa3569c; read-only; measured with PIL)

Captures: `media/2026-11-12/mc-pixel-r5/*.png` (24: agenda, scene-head, hover, chat-question, all-keys, terminal; light/dark; 1x 1280x800, 2x 2560x1600), compared with `media/2026-11-12/mc-pixel-r4/` and `reports/pixel-mc-pixel-r4.md`. Coordinates are CSS px (1x); baselines are the exclusive row under flat letter bottoms.
Method: all 24 PNGs measured; 12 opened and read (agenda, chat-question, terminal, all-keys, hover, scene-head in light and dark at 1x) and the 2x set compared run by run against 1x (23/19/17/20 ink runs per page, same count at both scales, max 1x/2x delta 1.0 px, no theme drift). Baseline estimator: last row with >=45% of the run's peak ink density; baseline mod 28 on every page at 1x and 2x. Contrast from the extreme ink pixel of each text box against the page background. Full pixel diff r4 vs r5 at 1x: the only changed regions are the new empty-agenda line (y 200..217, x 161..406), the urgent line/X (x 425..564, y 764..774), the dark title (x 220..456, y 96..130), the helper disc (x 889..899, y 120..130) and the clock digits (time of capture). Everything else is pixel-identical to r4. Scripts in `/tmp/claude-0/-home-user/8a28ab73-2182-54dd-8448-69a0fbb58e8a/scratchpad/mcrev5/` (m.py, c.py, all.py).

## Worker claims, checked

- N1 "all accent text >= 7.1:1 in light and dark with the same hue": PARTLY FALSE. Measured: light title/Listening (140,36,31) 8.06; light urgent line and X (160,30,28) 7.13; light amber disc (113,74,15) 7.16; dark title (216,152,139) 7.13; dark urgent line and X (239,141,133) 7.16; dark amber (224,179,97) 8.76. BUT dark "Listening" text and dot are still (204,120,102) = 5.23:1 (x 1137..1220, y 36..51, agenda, terminal, chat-question dark; byte-identical to r4, not in the r4/r5 diff). "Same hue" is also loose: light 1..3 deg (urgent 0.9, title 2.8), dark 4.5 (urgent), 10.1 (title), 10.6 (Listening); amber 36 light / 39 dark. Margins of the three new tokens are thin (7.13..7.16) but pass.
- N2 "empty agenda shows a second muted line": TRUE. Serif 20 px, muted (84,80,74) 7.33:1 light, (173,171,163) 7.41:1 dark, x 161..407, y 200..218, baseline 214 = 18 mod 28 (the serif phase), pitch 28 from "Nothing planned today." (base 186); 1x and 2x identical (214.5 at 2x), no widow, no wrap. Present on agenda, scene-head, hover, all-keys, all five page types with a left column, light and dark.

## Old findings (r4)

| # | r4 | Now | Evidence |
|---|---|---|---|
| N1 accent text < 7:1 | MINOR OPEN | OPEN (reduced: 1 of 5 tokens left) | urgent 6.10 -> 7.13 light, 5.52 -> 7.16 dark FIXED; light Listening 8.06 ok; amber 4.75 -> 7.16 light FIXED (8.76 dark); dark title 5.23 -> 7.13 FIXED. Dark Listening text and dot still 5.23:1 (see New N5). |
| N2 empty first screen has no voice hint | MINOR OPEN | FIXED (new wording issue N6 below) | line 'Say "Atlas" or type below.' present, on grid, 7.33/7.41:1. |
| N3 Yes/No 32 px buttons off grid | COSMETIC OPEN | OPEN | boxes y 630..662 x 902..990 / 998..1086, label base 652 (8 mod 28), unchanged pixels. |
| N4 "a" ends line 1 of notification 2 | COSMETIC OPEN | OPEN | "Backup done and a / longer line that wraps" x 1000..1213, y 256..302, unchanged. |
| B4 chat-door focus bar | COSMETIC OPEN | OPEN (not captured) | no capture with focus on "Chat with a helper". |
| 14 duplicate head | COSMETIC OPEN | OPEN | "1  AGENDA" (36..47) above "1 | AGENDA" (96..131); terminal "5 TERMINAL" likewise. |
| 16 serif vs Inter baseline | COSMETIC OPEN | OPEN | "Chat with a helper ..." base 438 vs "Super+a" base 439. |
| N6 bar 1 px low at 1x | COSMETIC OPEN | OPEN | no pixel change in the title row (diff shows only colour). |
| N7 numeral side bearing | COSMETIC OPEN | OPEN | numeral ink x 165 vs body 161. |
| C-A serif +-1 px phase | COSMETIC OPEN | OPEN | serif 18 mod 28 (186, 214 new, 270, 298, 354, 438), sans 19; the new line joins the serif phase, consistent with the rest of the serif lines, so no new drift. |
| C-B same question stated several times | COSMETIC OPEN | OPEN | helper row, chat "Write notes.txt?", card label + heading, footer: unchanged. |
| C-C Listening dot | COSMETIC OPEN | OPEN | dot x 1208..1220 vs right edges 1222..1224. |
| C-D caret | COSMETIC OPEN | OPEN | caret x 884..885, y 734..751. |
| C-E Stop/Esc/Close gaps | COSMETIC OPEN | OPEN | Stop 1109..1139, Esc 1154..1177, Close 1185..1223. |
| C-F 15 px terminal mono vs 14 px card mono | COSMETIC OPEN | OPEN | unchanged; sizes in use: ~48, serif 20, sans 14, mono 14, mono 15. |
| C-G measure 24..41 chars | COSMETIC OPEN | OPEN | notifications 224 px, card 218..259 px; the new line is 26 characters (246 px) in a 673 px column. |
| all r4 FIXED items (M1, 5, 8, 17 and earlier) | FIXED | no regression | all sans baselines 19 mod 28 on every page, 1x and 2x (all-keys rows pitch 28, 467..775); title base 130/131; card edges 378..686; field/footers base 775; hairline x 860; hover bar and Super+Space rows unchanged; muted token 7.33/7.41 everywhere. |

Old totals: FIXED 1 (r4 N2), OPEN 15 (N1 residual as N5, N3, N4, B4, 14, 16, N6, N7, C-A..C-G), REGRESSED 0.

## New findings

| # | Rate | File | Coordinates | Defect | Fix |
|---|---|---|---|---|---|
| N5 | MINOR | agenda-dark, terminal-dark, chat-question-dark (s1, s2) | "Listening" x 1137..1195 and dot x 1208..1220, y 36..51 | The worker's N1 fix missed this token: dark Listening text and dot (204,120,102) = 5.23:1, below the 7:1 three-metre rule, at the top-right where the voice state is shown (the voice-first status). Light is 8.06. Hue is also 10.6 deg vs 4.5 deg for the dark urgent line. | Use the dark title token (216,152,139) = 7.13:1 (or (239,141,133) 7.16). |
| N6 | MINOR | agenda-* (first screen) | line 2 x 161..407, y 200..218 | 'or type below.' points at nothing: no input field exists on the empty agenda (the field appears only inside the chat, y 728..775, in chat-question). The keyboard path is the chat door "Chat with a helper ... Super+a" (x 888..1223, y 424..442). A first-screen hint must name a real control. | Say 'Say "Atlas" or press Super+a.' (same length, 26 characters), or show the field on the agenda. |
| N7 | COSMETIC | agenda-*, all-keys-*, hover-*, scene-head-* | quotes around Atlas, x 213..272 | Straight ASCII quotes (") in the serif line; the card already uses typographic quotes (curly "yes", y 568..582). Traditional typesetting wants curly (or guillemet) quotes. | Use U+201C/U+201D; hang them if the line ever starts with a quote. |

## Verdict: MERGE-OK

MAJOR 0; MINOR 2 (N5 dark Listening 5.23:1, N6 "type below" with no field); COSMETIC 15 (old open 14: B4, 14, 16, N6, N7, C-A..C-G = 12, plus N3, N4; new N7 straight quotes).
Fixed this round: r4 N2 and the urgent/amber/title parts of N1; none regressed; no 1x/2x or theme drift; nothing moved except the intended colour and line changes.
Top 3: (1) dark "Listening" still 5.23:1 while the worker states all accent text passes (N5); (2) the new hint says "type below" but the empty agenda has no field (N6); (3) the shared 1 px residuals (serif phase C-A, bar N6, 4 px numeral indent N7, duplicate head 14, 32 px buttons N3) are the only grid defects left.
