# Pixel review: mc round 4 (branch claude/mc-pixel @ 72527394; read-only; measured with PIL)

Captures: `media/2026-11-12/mc-pixel-r4/*.png` (24: agenda, scene-head, hover, chat-question, all-keys, terminal; light/dark; 1x 1280x800, 2x 2560x1600), compared with `media/2026-11-12/mc-pixel-r3/` and `reports/pixel-mc-pixel-r3.md`. Coordinates are CSS px (1x); baselines are the exclusive row under flat letter bottoms.
Method: every one of the 24 PNGs was opened or measured (the worker did not open dark and 2x; I did). Ink-row runs per column with a 45% row-density baseline estimator, baseline mod 28 on every page at 1x and 2x, light vs dark and 1x vs 2x compared run by run (no layout drift, no theme drift: all runs agree within 0.5..1 px). Contrast ratios from the darkest (light) or brightest (dark) ink pixel of each text box against the page background. Crops at 2x of the card, the field and the urgent footer. Scripts in `/tmp/claude-0/-home-user/8a28ab73-2182-54dd-8448-69a0fbb58e8a/scratchpad/mcrev4/`.

Worker claims, checked:
- Card path no widow: TRUE. Path `~/projects/notes/notes.txt` is one mono line (x 902..1120, base 495); "(412 bytes)" is its own line (x 905..992, base 523); pitch 28.
- "Typing does not answer." inside the card on the grid: TRUE. Inside the border (378..686), x 902..1063, base 607 = 19 mod 28, muted 7.33:1 light / 7.41:1 dark.
- Urgent line has a drawn X: TRUE. 10 x 10 px at x 425..435, y 765..775 (cap 764..775), 11 px gap to the text (446), both themes, 1x and 2x. It is 10 px, not 14, and 1 px short of cap top (COSMETIC, not logged).
- Muted text >= 7:1: TRUE. Muted token (84,80,74) on (247,245,237) = 7.33:1; dark (173,171,163) on (28,28,28) = 7.41:1; same pixel colour in kickers, header, clock, "Nothing planned", helper description, times, key descriptions, footer arrows/labels, "Atlas" label, Stop/Esc/Close, Show, Enter, placeholder, "waiting", "spoken yes", "Typing". Accent text is NOT at 7:1 (N1).

## Old findings (r3)

| # | r3 | Now | Evidence |
|---|---|---|---|
| M1 card path widow | MINOR OPEN | FIXED | card mono: path base 495, "(412 bytes)" base 523, both full lines, x 902..1120 / 905..992 (light, dark, 1x, 2x identical). Card is 308 px = 11 rows (378..686, both edges 14 mod 28). |
| 5 "Typing does not answer." outside card | MINOR OPEN | FIXED (time-left still absent) | now card line 6, y 596..610, base 607 (19 mod 28), under "A spoken yes does not count." (579). Field zone now holds only hairline 728, placeholder 747, "A question is waiting." 775. "Question from Atlas" still has no time-left: not in the brief, noted only. |
| 8 urgent footer "x" | MINOR OPEN | FIXED | drawn 10 px cross, x 425..435, y 765..775; footer text 446..565; block 425..565 centre 495 vs measure centre 496. Contrast of the line: N1. |
| 17 muted token < 7:1 | COSMETIC OPEN | FIXED | 4.99/4.93 -> 7.33/7.41 (table above). |
| B4 chat-door focus bar | COSMETIC OPEN | OPEN (not captured) | no capture with focus on "Chat with a helper". |
| 14 duplicate head | COSMETIC OPEN | OPEN | "1  AGENDA" (36..47) above "1 | AGENDA" (96..131); terminal "5 TERMINAL" likewise. |
| 16 serif vs Inter baseline | COSMETIC OPEN | OPEN | "Chat with a helper ..." base 438 (438.5 at 2x) vs "Super+a" 439 (right, same row). |
| N6 bar 1 px low at 1x | COSMETIC OPEN | OPEN | bar y 98..131 vs "E" cap 97..130 at 1x (1 px low); at 2x cap 97.5..131, bar 98..131 (0.5 short at top). Terminal "T" 97..130 same. |
| N7 numeral side bearing | COSMETIC OPEN | OPEN | numeral ink x 165 (agenda), 163 (terminal) vs body 161: 4 / 2 px indent. |
| C-A serif +-1 px | COSMETIC OPEN | OPEN | serif bases 214/270/298/354 (18 mod 28, notifications), 300 (20, chat message), 440/468 (20, card heading), 438 (18); sans 19 everywhere; times 215 vs serif 214. |
| C-B same question stated several times | COSMETIC OPEN | OPEN (worse by count) | helper row "has a question . Write file" (y 120), chat "Write notes.txt?" (316..328), card "Question from Atlas" + heading "Atlas wants to write the file notes.txt.", footer "A question is waiting." = 5 statements in one column. |
| C-C Listening dot | COSMETIC OPEN | OPEN | dot x 1208..1220 vs right edges 1222..1224. |
| C-D caret | COSMETIC OPEN | OPEN | caret x 884..885, y 734..751 (17 px) vs placeholder cap 10.5, left margin 888. |
| C-E Stop/Esc/Close gaps | COSMETIC OPEN | OPEN | Stop 1109.5..1139, Esc 1154..1177, Close 1185.5..1223: gaps 15 and 8.5. |
| C-F 15 px terminal mono vs 14 px card mono | COSMETIC OPEN | OPEN | terminal advance 8.97 (15 px), card 8.4 (14 px). Sizes in use: ~48, serif 20, sans 14, mono 14, mono 15 = 4 distinct families of size counting 14/14 as one: still within 4 but 15 is wasteful. |
| C-G measure 24..41 chars | COSMETIC OPEN | OPEN | notifications 224 px (24..28 chars), card lines 218..259 px (26..41 chars). |
| all other r3 FIXED items (B1, B2, B3, B5, 1, 2, 3, 4, 6, 7, 9..13, 15, 18, 19, N1..N5, N8, N9) | FIXED | no regression | all baselines mod 28 = 19 (sans), title 130/131, hairline x 860 y 37..775, hover bar x 876..878 y 681..691 vs "Super+Space" 681..691 (flat cap), all-keys 12 rows pitch 28 (456..777 baselines 467..775), field/footers base 775, column top 92 on all pages, terminal 15 px on a 28 cursor, light/dark and 1x/2x identical. |

Old totals: FIXED 4 (M1, 5, 8, 17), OPEN 12 (B4, 14, 16, N6, N7, C-A..C-G), REGRESSED 0. Previously FIXED items all hold.

## New findings

| # | Rate | File | Coordinates | Defect | Fix |
|---|---|---|---|---|---|
| N1 | MINOR | agenda-*, terminal-*, chat-question-* | urgent line x 425..565, y 764..775; "Listening" x 1137..1195, y 37..51 (dark); helper disc x 889..900, y 120..131 (light) | Accent colours miss the 7:1 rule for three metres on 14 px text, and the urgent line is the highest-priority text on the page: light 6.10:1 (179,33,31), dark 5.52:1 (235,107,97); "Listening" dark 5.23:1 (204,120,102) (light 8.06 ok); amber disc light 4.75:1 (150,98,20), dark 8.76 ok; dark title/numeral accent 5.23 (large type, still below 7). | Light urgent: use the same red as title/Listening (140,36,31) = 8.06:1 (or 150,30,26 = 7.70). Dark red token (250,140,128) = 7.41:1 for urgent, Listening and title. Light amber (115,70,0) = 7.39:1. |
| N2 | MINOR | agenda-* (first screen) | left column y 172..190 and top-right "Listening" | Voice-first brief (RC: first screen says how to talk to it): the empty first screen offers no voice hint at all, only "Nothing planned today." and a 14 px "Listening" in the corner; a 600 px empty column. | Under the empty-state line add one serif/sans line with a sample utterance (e.g. "Say: what is on today?") at 20 px on the grid, in the left column. |
| N3 | COSMETIC | chat-question-* | Yes/No boxes x 888+14..1086, y 630..662; label ink y 640.5..651.5 | Buttons are 32 px tall: box top 630 (14 mod 28) but bottom 662 (18 mod 28); label baseline 652 = 8 mod 28, 11 px off the 19 phase of every other sans line; card bottom padding 24 vs top 22. | Make the buttons 28 px (or 56) tall with the label base at 19 mod 28; or set box 630..658. |
| N4 | COSMETIC | agenda-*, terminal-* | notification 2, x 1000..1213, y 256..302 | "Backup done and a / longer line that wraps": the first line ends in the single-letter article "a" (bad break, not a widow). | Non-breaking space after "a" / `text-wrap: pretty`; or balance lines. |

## Verdict: MERGE-OK

MAJOR 0; MINOR 2 (new N1, N2; all r3 MINOR fixed); COSMETIC 14 (old B4, 14, 16, N6, N7, C-A..C-G = 12; new N3, N4). Fixed this round: 4 of 16 open items; none regressed; no theme or scale drift.
Top 3: (1) N1 accent text (urgent line 6.10/5.52:1, dark Listening 5.23:1) below 7:1 while muted is now 7.33/7.41; (2) N2 empty first screen says nothing about how to talk to it; (3) the shared 1 px residuals (serif phase C-A, bar N6, 4 px numeral indent N7, duplicate head 14) remain the only grid defects.
