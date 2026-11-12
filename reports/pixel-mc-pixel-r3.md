# Pixel review: mc round 3 (branch claude/mc-pixel @ cd5ae779; read-only; measured with PIL)

Captures: `media/2026-11-12/mc-pixel-r3/*.png` (24: agenda, scene-head, hover, chat-question, all-keys, terminal; light/dark; 1x 1280x800, 2x 2560x1600),
compared with `media/2026-11-12/mc-pixel-r2/` and `reports/pixel-mc-pixel-r2.md`. Coordinates are CSS px (1x), baselines are the exclusive row under the flat letter bottoms.
Method: ink-row runs per column, baseline mod 28 on every page, 1x vs 2x and light vs dark compared run by run (all runs agree within 1 px on all 6 pages: no layout drift, no theme drift), crops at 2x for the field, the helper dot and the urgent line.

Baseline grid (N3): sans 14 px, title, numeral, body, footer, keys, kickers, field, "Typing", "A question is waiting." all end at y = 19 mod 28 (title 131, body 187, header 47, keys 635..775 step 28, field 747, waiting 775, footer 775). Column top is 92 (first kicker cap top, baseline 103) on agenda, hover, scene-head, terminal and chat-question; r2 had 22/78. Serif lines sit at 18 (notifications 214, 270, 298, 354; "Chat with a helper" 438) or 20 (chat messages 300, 356; card heading 468): see C-A.

## Old findings

| # | r2 | Now | Evidence |
|---|---|---|---|
| B1 scene-head bar | FIXED | FIXED (1 px at 1x: N6) | bar x 201..202, y 98..130; "E" cap 97..129 at 1x, 97.5..130.5 at 2x; bar bottom = baseline 131 in all four captures. |
| B2 bar centred | FIXED | FIXED | digit ink ends 182, bar 201, title starts 220: 19/19. |
| B3 hover bar | FIXED | FIXED | hover-*: bar x 876..877, y 681..690 (10 rows) vs "Super+Space" flat cap 681..690, offset 11 px; same at 2x. |
| B4 chat-door focus bar | OPEN (not captured) | OPEN (not captured) | no capture of focus on the "Chat with a helper" row. |
| B5 hairline extent | OPEN | FIXED | hairline x 860, y 37..774 (agenda, chat-question, terminal); header cap top 36, footer baseline 775. 1 px short at top, flush at bottom. |
| 1 F1 list | FIXED | FIXED | all-keys: 12 rows pitch 28 (456..777), head "ALL KEYS" 428..438, baselines 19 mod 28, nothing under the rule. |
| 2 terminal page | FIXED (N5) | FIXED | prompt only, no warning; column, hairline, scene head, urgent footer present; text 15 px (N5). |
| 3 "Stop" orphan | OPEN | FIXED | "Stop" x 1110..1139, baseline 243 on the CHAT head line (head 232..243), left of "Esc Close". |
| 4 message field | OPEN | FIXED | no box: 1 px hairline x 888..1223 at y 728; "Write to Atlas" x 889..975, baseline 747 on the grid, Inter, same face as the card; "Enter" right ink 1222 (Close 1222, card 1223); caret hangs at x 884..885 (C-D). At 2x crisp: max edge step 110, equal to the other muted 14 px text (110) and below "Typing" (184, full-contrast text). |
| 5 card order / stray line | OPEN | OPEN | "Question from Atlas" still has no time-left; "Typing does not answer." (y 708..722, full-contrast text, x 889) is outside the card, now on the grid and directly above the hairline of the field, so it reads as the field's label. |
| 6 chat line pitch | FIXED | FIXED | label baselines 271, 327, 383 (pitch 56); message baselines 300, 356 (56); see C-A for the 1 px serif offset. |
| 7 notification spacing | OPEN | FIXED | items at baselines 214, 270 (wrap line 298), 354: 56 between items, 28 within a wrapped item; time column and text 112 apart. |
| 8 urgent footer "x" | OPEN | OPEN | agenda/terminal footers: "x" glyph x 423..430, y 767..774 (7 px, x-height only) next to caps 764..775 (11 px), 14 px text; reads as a fallback "x" and is small and thin. Text "DISK IS FULL" 6.1:1 light, 5.5:1 dark. Footer centred (423..567 around 495 vs measure centre 496). |
| 9 right/left ink edges | FIXED (dot not captured) | FIXED (dot: C-C) | footer right ink 830, clock 830, Show 1223, Close 1222, Enter 1222, card/hairline 1223/1224; left: footer 162, run head 161, body 161, title 220. |
| 10 footer vs key baseline | FIXED | FIXED | "Esc" 775, footer 775, field-zone 775. |
| 11 "?" sign | OPEN | FIXED | helper row now an amber disc, 11 px (x 889..900, y 120..131) = cap height of "Atlas"; state also in words ("has a question"). Disc 5.5:1 dark, 5.1:1 light. |
| 12 helper head | FIXED | FIXED | "HELPERS" 92..102, row 120..131; same rhythm as NOTIFICATIONS/CHAT. |
| 13 label columns | OPEN (not captured) | FIXED | helpers description, notification text, key descriptions all at x 1000 (label width 112 from 888); times and keys at 888. |
| 14 duplicate head | OPEN | OPEN | "1  AGENDA" (36..47) above the scene head "1 | AGENDA" (97..130) and "5 TERMINAL" likewise. |
| 15 arrows | FIXED | FIXED | arrows 766..775 vs cap 764..775. |
| 16 serif vs Inter baseline | OPEN | OPEN | "Chat with a helper ..." serif bottom 438 vs "Super+a" baseline 439 (1 px, 1.5 with C overshoot). |
| 17 empty-state presence | OPEN | OPEN | "Nothing planned today." and clock, kickers, times, key descriptions, footer are the muted token: 4.99:1 light (110,105,97 on 247,245,237), 4.93:1 dark; below 7:1 for 3 m. Dark title/numeral/Listening 5.23:1 (large). |
| 18 14 px floor | OPEN | FIXED | terminal 15 px (advance 9.0, text 175..190, cursor 168..195 = 28 rows); card mono 14 px (advance 8.4); no text below 14. |
| 19 theme drift | NO DRIFT | NO DRIFT | light/dark runs identical within 1 px on all pages. |
| N1 bottom gutter | MINOR | FIXED | field zone ends at baseline 775 (= footer = hairline end); "A question is waiting." 765..778. Gaps: card 686, "Typing" 719, hairline 728, field 747, waiting 775. |
| N2 field insets | MINOR | FIXED | placeholder 889, "waiting" 889, "Typing" 889; caret hangs 884. |
| N3 one baseline | MINOR | FIXED (residual C-A) | all sans on 19 mod 28 on both columns; one phase for every page. |
| N4 column top | MINOR | FIXED | 92 on every page. |
| N5 terminal size | MINOR | FIXED | 15 px on a 28 pitch. |
| N6 bar 1 px low at 1x | COSMETIC | OPEN | bar 98..130 vs "E" cap 97..129 at 1x; 2x within 0.5. |
| N7 numeral side bearing | COSMETIC | OPEN | numeral ink x 165 vs body 161, run head 161 (4 px indent). |
| N8 locale warning | COSMETIC | FIXED | not present. |
| N9 spacing under Stop | COSMETIC | FIXED | head 243, "You" 271, "Please" 300: 28/29. |

Old totals: FIXED 25 (B1, B2, B3, B5, 1, 2, 3, 4, 6, 7, 9, 10, 11, 12, 13, 15, 18, N1, N2, N3, N4, N5, N8, N9, plus 19 no drift), OPEN 7 (B4*, 5, 8, 14, 16, 17, N6, N7; * not captured; 5 and 8 MINOR, rest COSMETIC), REGRESSED 0.

## New findings

| # | Rate | File | Coordinates | Defect | Fix |
|---|---|---|---|---|---|
| M1 | MINOR | chat-question-*-s1/s2 | card mono lines x 902..1162, y 512..526 and "bytes)" x 903..950, y 540..554 | The path line wraps at 31 chars and leaves "bytes)" alone on the second line: a widow in the permission card, on the main path of the question flow; the mono block takes two 28 rows for 36 characters. Card measure (308 px, ~36 chars) is also under 45. | Do not wrap the path: break at "/" or middle-ellipsise ("~/projects/.../notes.txt"), put "(412 bytes)" on its own line, or use `text-wrap: balance/pretty`. |
| C-A | COSMETIC | all pages | serif baselines 214 (notif), 438 (chat door) = 18; 300, 356, 468 (chat, card) = 20; sans 19 | Serif lines sit -1/+1 px off the sans phase; 06:02 baseline 215 vs "Disk is full" 214; message pitch 29/27 instead of 28. | Set serif line-height/offset from the font metrics so its baseline lands on 19 mod 28. |
| C-B | COSMETIC | chat-question | helper row "has a question · Write file" (y 120), chat line "Write notes.txt?" (y 373..383), card heading (y 453..496) | The same question is stated three times in one column. | Drop the bold "Write notes.txt?" chat line or the helper-row suffix. |
| C-C | COSMETIC | agenda, terminal, chat-question | "Listening" dot x 1208..1220 | Right ink of the dot 1220, other right edges 1222..1224. | Hang the dot (margin-right -3). |
| C-D | COSMETIC | chat-question | caret x 884..885, y 734..751 | Caret hangs 4 px outside the 888 margin; its height 18 vs text 10. | Keep, or inset the caret at 888 and start the text after it. |
| C-E | COSMETIC | chat-question | "Stop" 1110..1139, "Esc" 1154..1176, "Close" 1186..1222 | Gaps 15 and 10 px (uneven). | One gap (12). |
| C-F | COSMETIC | terminal vs card | terminal mono 15 px, card mono 14 px | Two near-equal mono sizes (and 14 sans): the "at most 4 sizes" count (48, serif 20, 15, 14) is met but 14/15 is an unnecessary pair. | Use 14 for both (advance 8.4). |
| C-G | COSMETIC | right column, all pages | notification text 1000..1224 (224 px, 24..28 chars), card 308 px | Measure 24..36 characters, below 45..75; unavoidable in a 336 px column but wraps ("Backup done and a / longer line that wraps") are choppy. | Widen the text column (label width 112 already; reduce to 84 for times) or accept. |

## Verdict: MERGE-OK

MAJOR 0; MINOR 3 (old 5, 8; new M1); COSMETIC 14 (old B4, 14, 16, 17, N6, N7; new C-A..C-G). Fixed from r2: 25 of 32 items, none regressed.
Top 3: (1) M1 card path wraps to a lone "bytes)" (widow); (2) old 8 urgent footer "x" is a 7 px fallback glyph and old 5 "Typing does not answer." sits outside the card; (3) one remaining contrast gap, 17: the muted token at 4.9..5.0:1 carries the empty state, clock, times and keys, short of 7:1 for three metres.
