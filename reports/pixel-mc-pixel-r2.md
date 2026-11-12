# Pixel review: mc round 2 (branch claude/mc-pixel @ b50159dd; read-only; measured with PIL)

Captures: `media/2026-11-12/mc-pixel-r2/*.png` (scene-head, hover, chat-question, all-keys, terminal; light/dark; 1x and 2x),
compared with `media/2026-11-12/mc-pixel/` (r1) and the findings of `reports/pixel-mc-v2.md`.
Coordinates are CSS px (1x). 2x geometry equals 1x doubled for every capture (downsampled 2x vs 1x differs only by anti-aliasing;
no layout drift). Light and dark share identical geometry (no theme drift). Heights are inclusive pixel rows.

Not captured in this round (cannot be verified, treated as still open): notification list with items (7), urgent footer line (8),
numbered chat entry (13), chat-door focus bar (B4), "Listening" dot (right edge part of 9).

## Old findings

| # | Old | Now | Evidence |
|---|---|---|---|
| B1 scene-head bar | MAJOR | FIXED | bar y 95..127 (33 rows) vs "AGENDA" cap 94..126 (33 rows) and digit 93..126; same in light, dark, 1x, 2x (2x: 190..255 vs 189..255). Residual 1 px at 1x, see N7. Note: r1 already had this geometry. |
| B2 bar centred | MINOR | FIXED | digit ends 182, bar 201, title starts 220: gaps 19/19. Terminal: 185/204/223: 19/19. |
| B3 hover bar | MINOR | FIXED | hover-*: bar x 876..877 (2 px), y 683..692 (10 rows); "Super+Space" S y 682..692, flat cap 683..692; offset 12 px from text (x 888). Same at 2x (683..692.5). |
| B4 chat-door focus bar | COSMETIC | OPEN (not captured) | no capture of the focus on the Pagella row. |
| B5 hairline extent | COSMETIC | OPEN | hairline x 860, y 28..771; header cap top 22, footer baseline 776. Still 6 px short at top, 5 px at bottom. |
| 1 F1 list overlap | MAJOR | FIXED | all-keys: chat/notifications hidden, hairline y 408, head "ALL KEYS" 430..440, 12 rows pitch 28 (458..779), Esc visible, no rule through text. |
| 2 terminal page | MAJOR | FIXED (residual N5) | foot config errors gone; one warning line + prompt; column, hairline, footer, scene head "5 | TERMINAL" present. Mono size still under 14 px (N5). |
| 3 "Stop" orphan | MINOR | OPEN | chat-question: "Stop" x 1193..1222, y 256..269 alone under "Esc Close" (218..228). Now also 38 px below the head line, off the 28 grid. Fix: put Stop on the CHAT head line (y 218) right of "Esc Close". |
| 4 message field | MINOR | OPEN | box x 888..1223, y 730..785 with 1 px border all round; placeholder in an Arial-like third face; at 2x the field text is soft (max edge step 54 vs 108/182 for crisp text: 1x bitmap scaled). No "Write to Atlas", no flush-right "Enter". |
| 5 card order / stray line | MINOR | OPEN | "Question from Atlas" has no time-left; "Typing does not answer." y 702..715 floats outside the card, text colour. |
| 6 chat line pitch | MINOR | FIXED (phase: N3) | message rows 286, 312, 342, 368, 398: pairs repeat every 56 (2 x 28). Notification/helper rows 28. |
| 7 notification item spacing | MINOR | OPEN (not captured) | |
| 8 urgent footer / x size | MINOR | OPEN (not captured) | |
| 9 right/left ink edges | MINOR | FIXED (dot not captured) | footer right ink 830, clock 829 (light)/830 (dark), column labels 1222, card/field 1223: all within 1 px. |
| 10 footer vs key baseline | MINOR | FIXED | "Esc" baseline 776, "Overview" / "The Morning" baseline 776 (r1: keys 765). |
| 11 "?" sign | MINOR | OPEN | "?" x 888..894 (7 px) y 106..116, amber; Atlas text starts 906. Still a thin glyph, not a disc. Contrast 4.7:1 light, 8.7:1 dark. |
| 12 helper head | MINOR | FIXED | "HELPERS" head at y 78..88, row at 106..119, same rhythm as other heads. |
| 13 label columns | MINOR | OPEN (not captured) | |
| 14 duplicate head | COSMETIC | OPEN | "1  AGENDA" y 22..32 and scene head "1 | AGENDA" y 94..126 on every page; "5 TERMINAL" likewise. |
| 15 arrows | COSMETIC | FIXED | "Super+ <- ->" arrows now drawn in Inter, centred on the caps (655..664 vs cap 654..664). Arrow stroke is thinner than the semibold text (cosmetic). |
| 16 serif vs Inter baseline | COSMETIC | OPEN | "Chat with a helper ..." baseline ~200, "Super+a" baseline ~202 (C 188..201, S 192..203 with overshoot). 1-2 px. |
| 17 empty-state presence | COSMETIC | OPEN | muted text 4.93:1 (dark) / 4.98:1 (light); it is the only content on page 1 ("Nothing planned today."), clock, kickers, footer, speaker labels, placeholder are all this colour at 14 px. Below 7:1 for 3 m. |
| 18 14 px at the floor | COSMETIC | OPEN | labels and keys 14 px; mono terminal below (N5). |
| 19 theme drift | COSMETIC | NO DRIFT | geometry identical light/dark; palette tokens only. |

Totals old: FIXED 10 (B1, B2, B3, 1, 2, 6, 9, 10, 12, 15), OPEN 14 (B4*, B5, 3, 4, 5, 7*, 8*, 11, 13*, 14, 16, 17, 18; * = not captured), REGRESSED 0 of the old items (but see N1), no drift 1.

## New findings

| # | Rate | File | Coordinates | Defect | Fix |
|---|---|---|---|---|---|
| N1 | MINOR | chat-question-*-s1/s2 | field box y 730..785, card y 502..691, "Typing" 702..715; r1: field 716..771 | Regression vs r1: the added HELPERS head pushed the stack down. Field bottom 785 = bottom gutter 15 px (r1 29), 14 px below the column hairline end (771) and below the footer baseline (776); the field is the lowest element on the screen. Gaps card to "Typing" 11, "Typing" to field 15, neither a 28 multiple. | Keep the stack bottom at 771 (drop the "NOTIFICATIONS 0" row while a question is open, or reduce the gap above the card); gaps 14/14 or 28. |
| N2 | MINOR | chat-question-* | field text: line 1 x 900..1071 y 742..755, line 2 x 896..1021 y 762..775 | The two lines in the field start 4 px apart (900 vs 896): placeholder and second text use different insets; both in the third face. | One inset (x 902, same as the card text) and Inter 14. |
| N3 | MINOR | all captures | baselines: column top block 88, 116, 172, 228 (= 4 mod 28); bottom block KEYS 608 ... Esc 776 (= 20 mod 28); title/digit 126 (= 14); body "Nothing planned today." 187 (= 19); chat labels/text 297, 322, 353, 378, 409 (= 17/14) | Goal "one baseline grid": at least four phases (4, 14, 17-20). Rows are on a 28 pitch inside each block, but blocks are not on a common grid, so left and right column lines never share a baseline. | Define baseline k*28+C once; snap title, body, chat messages and bottom-anchored keys/footer to it (footer 776 and keys already agree, move the top block +16 or the bottom block -16). |
| N4 | MINOR | terminal-* vs scene-head-* | column first head cap top y 22 on terminal, y 78 on agenda/hover/all-keys | The column starts 56 px lower on the agenda (reserved, empty helper slot) than on the terminal; the right column jumps between page turns. Terminal head aligns to the running head (y 22), agenda does not. | Same column top on every page (reserve the helper slot everywhere or nowhere). |
| N5 | MINOR | terminal-*-s1/s2 | mono text x 160..632, y 169..201; "C" cap 170..179 (10 px), advance 7.7 px, pitch 17 | Terminal text is about 13 px (cap 10 px), under the 14 px floor; pitch 17 is off the 28 grid; first line at y 169. | Terminal font >= 14 px (advance 8.4), line height 28 or 14 (=2 lines per grid row). |
| N6 | COSMETIC | scene-head-*-s1 | bar y 95..127 vs "AGENDA" cap 94..126 | At 1x the bar is exactly 1 px low (top and bottom); at 2x it is within half a CSS px. | Position the bar from the device-pixel cap rect (translateY -1px at 1x). |
| N7 | COSMETIC | scene-head, terminal | digit "1" ink x 165, "5" ink x 163; body text and running head x 160/161 | The numeral is optically indented 3..5 px from the left margin (side bearing), body and running head hang at 160. | Pull the numeral left by its side bearing (hang it). |
| N8 | COSMETIC | terminal-* | y 169..183 text "warning: 'C' is not a UTF-8 locale" | The capture shows a locale warning from the environment as terminal content. | Start the capture with LANG=C.UTF-8 so the state shows only the prompt. |
| N9 | COSMETIC | chat-question-* | "Stop" y 256, "You" 286, helpers row 106 | Spacing between CHAT head (218), "Stop" (256) and "You" (286): 38 and 30, not 28 (partly 3). | After moving Stop, start messages at head + 56. |

## Verdict: FIX

No MAJOR left. Clean tally: MAJOR 0; MINOR 12 (old 7 + new 5); COSMETIC 10 (old 6 + new 4).
Top 3: (1) chat-question stack overflow and 15 px bottom gutter (N1) with the still-boxed, soft, third-face field (4, N2); (2) baseline grid in four phases (N3) and column top jumping 22 vs 78 (N4); (3) terminal mono ~13 px (N5). All B1-B3, F1 and footer/baseline fixes verified at 1x and 2x.
