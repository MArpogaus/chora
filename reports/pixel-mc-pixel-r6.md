# Pixel review: mc round 6 (branch claude/mc-pixel @ 10eb3a7f; read-only; measured with PIL)

Captures: `media/2026-11-12/mc-pixel-r6/*.png` (24: agenda, scene-head, hover, chat-question, all-keys, terminal; light/dark; 1x 1280x800, 2x 2560x1600), compared with `media/2026-11-12/mc-pixel-r5/` and `reports/pixel-mc-pixel-r5.md`. Coordinates are CSS px (1x); baseline = exclusive row under flat letter bottoms.
Method: all 24 PNGs measured; 1x agenda (light, dark), chat-question dark, terminal dark, all-keys light and 2x chat-question light opened and read. Ink runs per column (left 150..855, right 880..1230), baseline = last row with >=45% of peak ink density, mod 28 on every page; contrast from the extreme ink pixel against the page background (WCAG). 2x runs matched 1x by count and position (23/19/17/20 runs per page type, max delta 2.0 px, no theme drift). Pixel diff r5 vs r6 at 1x: changed regions are only the clock digits (x 810..830, y 36..46 and 911..927 in the notification times), the empty-agenda line 2 (y 200..217, x 202..446, agenda/scene-head/hover/all-keys/chat-question) and, in dark only, the "Listening" text and dot (x 1137..1219, y 36..50). Everything else is pixel-identical to r5. Scripts in `/tmp/claude-0/-home-user/8a28ab73-2182-54dd-8448-69a0fbb58e8a/scratchpad/mcrev6/` (m.py, two.py, d.py, out.txt).

## Worker claims, checked

- Dark Listening (216,152,138) = 7.12:1: TRUE (measured 7.12 on agenda, terminal, chat-question dark; text and dot). Thin margin but passes; the title token is (216,152,139), one unit apart, so the hue now matches.
- Empty-agenda line reads Say "Atlas" or press Super+a.: TRUE, with typographic quotes U+201C/U+201D. Serif muted, light (84,80,74) 7.33:1, dark (173,171,163) 7.41:1, x 161..446, baseline 214 = 18 mod 28 (serif phase, pitch 28 from 186), 2x identical, no wrap, no widow. It names a real control (the chat door "Chat with a helper ... Super+a", y 424..442).

## Typesetting audit (all 24)

- Baseline grid: every sans run 19 mod 28 and every serif run 18 mod 28 at 1x and 2x (all-keys rows 467..775 pitch 28; footer/field 775; card edges 378..686). Runs reading 20 (Listening y 36..50 with dot/descender; "I will write the file now." with descender g) are estimator artefacts of descenders, not drift. Hairlines at 575/407/729 are rules, unchanged.
- Contrast, every text colour. Light on (247,245,237): muted 84,80,74 = 7.33; ink 41,38,36 = 13.77; title/Listening 140,36,31 = 8.06; urgent 160,30,28 = 7.13; amber 113,74,15 = 7.16. Dark on (28,28,28): muted 173,171,163 = 7.41; ink 217,212,201 = 11.53; title/Listening 216,152,138 = 7.12; urgent 239,141,133 = 7.16; amber 224,179,97 = 8.76 (dot). All >= 7:1 in both themes; no text colour below the rule. Hairlines/rules 1.4..1.7:1 are not text.
- Sizes: about 48 (title), serif 20, sans 14, mono 14/15 = 4 sizes + one mono variant; smallest text 14 px (small caps labels 11 px cap height, 14 px type).
- Measure: left column hint 26 characters in 673 px; right margin 24..41 characters (unchanged C-G, a narrow margin column, not a text page).
- No widows in the headings/lists; card heading "Atlas wants to write the / file notes.txt." breaks cleanly.

## Old findings (r5)

| # | r5 | Now | Evidence |
|---|---|---|---|
| N5 dark Listening 5.23:1 | MINOR OPEN | FIXED | (204,120,102) -> (216,152,138) = 7.12:1, text x 1137..1195, dot 1208..1220, y 36..50. |
| N6 hint says "type below", no field | MINOR OPEN | FIXED | now "press Super+a", which is the chat door key (y 424..442). |
| N7 straight quotes | COSMETIC OPEN | FIXED | curly quotes at x 213..272 (visible in agenda and all-keys captures). |
| N3 Yes/No 32 px buttons off grid | COSMETIC | OPEN | boxes y 630..662, label base 652 (8 mod 28), unchanged. |
| N4 "a" ends line 1 of notification 2 | COSMETIC | OPEN | "Backup done and a / longer line that wraps" x 1000..1213, y 256..302. |
| B4 chat-door focus bar | COSMETIC | OPEN | not captured. |
| 14 duplicate head | COSMETIC | OPEN | "1 AGENDA" (36..47) above "1 | AGENDA" (96..131); terminal likewise. |
| 16 serif vs Inter baseline | COSMETIC | OPEN | chat door base 438 vs "Super+a" 439. |
| N6 bar 1 px low at 1x | COSMETIC | OPEN | title row pixel-identical apart from colour. |
| N7 numeral side bearing | COSMETIC | OPEN | numeral ink x 165 vs body 161. |
| C-A serif +-1 px phase | COSMETIC | OPEN | serif 18 mod 28, sans 19. |
| C-B question stated several times | COSMETIC | OPEN | helper row, chat "Write notes.txt?", card, footer. |
| C-C Listening dot | COSMETIC | OPEN | dot x 1208..1220 vs right edges 1222..1224. |
| C-D caret | COSMETIC | OPEN | caret x 884..885, y 734..751. |
| C-E Stop/Esc/Close gaps | COSMETIC | OPEN | Stop 1109..1139, Esc 1154..1177, Close 1185..1223. |
| C-F 15 px terminal mono vs 14 px card mono | COSMETIC | OPEN | unchanged. |
| C-G measure 24..41 chars | COSMETIC | OPEN | notifications 224 px, card 218..259 px. |

FIXED 3 (N5, N6, N7 of r5); OPEN 14 (all cosmetic); REGRESSED 0.

## New findings

| # | Rate | File | Coordinates | Defect | Fix |
|---|---|---|---|---|---|
| N8 | COSMETIC | all *-dark, *-light | title/urgent/amber tokens | Contrast margins are thin (7.12 dark Listening, 7.13 light urgent, 7.16 dark urgent/light amber); a rounding change in a token would drop below 7:1. | Add a contrast check (>= 7.05) to the audit so the tokens cannot slip. |

No MAJOR, no MINOR.

## Verdict: PASS

MAJOR 0; MINOR 0; COSMETIC 15 (14 old open, 1 new). Fixed this round: r5 N5, N6, N7. No regression, no 1x/2x or theme drift; only the intended line, colour and clock changed.
Top 3: (1) dark Listening now 7.12:1, all text colours >= 7:1 in both themes; (2) empty-agenda hint is on grid (18 mod 28), curly-quoted and points at a real control; (3) the remaining cosmetic residuals (duplicate head, 32 px buttons, 1 px serif phase, numeral indent, narrow margin measure) are unchanged and carry no weight against the goal.
