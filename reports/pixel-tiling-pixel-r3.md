# Pixel review: chora-tiling, branch claude/tiling-pixel @ b39f5d8a, round 3 (2026-11-12)

Reviewer did not build it. Captures: media/2026-11-12/tiling-pixel-r3/*.png = 38 PNGs (chat, desktop, fullscreen, keys, launcher, layout, question, toast, tooltip; Ink and Paper; 1280x800 and 2560x1600@2; plus desktop at 1600x1000@1.25 in both themes). Every PNG was read or measured with PIL (1x px unless stated). Each 2x was BOX-downsampled and diffed against its 1x: only glyph antialiasing differs, except the launcher (see R8). Hairlines are exactly 2 device px at 2x (tooltip, layout chip, launcher), so coordinates hold.

## r2 findings: status

| # | r2 | Now | Evidence |
|---|---|---|---|
| R1 chips under chat panel | MINOR | FIXED (chips), combo untested | chat-ink/paper: panel x868..1267, y52..787. "Listening" chip now x744..820, orb x833..850, y751..777, 17 px left of the panel, fully visible, same fill (26,29,39) / white. The question card (x696..1255) and toast (x896..1255) still sit under the panel footprint; no chat+question capture exists. See R13. |
| R2 toast LCD AA | COSMETIC | OPEN | toast text x909..1230 y677..725: channel spread 213 Ink, 228 Paper (colour fringes visible in the Paper crop); every other helper text 12-14. |
| R3 orb ring | COSMETIC | OPEN | Paper toast, chat, launcher: orb ring (17,19,26) at x1231 / x835..836 is a black outline; Ink gives a dark 3 px halo (17,19,26)..(21,24,31). |
| R4 toast type | COSMETIC | OPEN | "Build finished" 16 px bold, body 16 px light. |
| R5 chat header | COSMETIC | OPEN | Bold error sentence y70..83, Close alone on y92..119, no header row. |
| #6 keys panel size | COSMETIC | OPEN | x233..1045, y52..747 = 813 x 696 (spec 880 x 560). |
| #7 scrim | MINOR | FIXED | keys-ink: tile (22,25,34) to (21,24,32), chip (26,29,39) to (24,27,36), pill likewise, orb (167,139,250) to (137,115,205) = exactly 0.20 Ink base (17,19,26). keys-paper: chip/pill 255 to 251, orb lightens to (181,158,248) (blend with Paper base, no grey flattening), wallpaper tint kept. Uniform and theme-correct. Residual R12 (bar not dimmed). |
| #8 key caps | COSMETIC | OPEN | "Super+r / Super+Space" is still one cap with " / " inside; row pitch 22. Column widths equal (OK). |
| #10 bar icons | COSMETIC | OPEN | Sizes unchanged (14, 16x12, 14). New: they are also soft, see R6. |
| #12 launcher content | UNVERIFIED | OPEN, COSMETIC | Now captured: plain "Apps" prompt + caret, 9 rows, pitch 37, no search hint, no scroll cue; list is noisy (Foot / Foot Client / Foot Server, ImageMagick (color depth=q16), LibreOffice x4). Order is not alphabetical (Screenshot first), fine if it is recency. |
| #11 launcher radius/theme | UNVERIFIED | FIXED | Launcher panel x385..894, y214..625 (510 x 412), hairline (46,51,68) / (213,216,225), fill (26,29,39) / white = surface, corner arc ~13 px (hairline starts x398 at y214) matching the other islands; input text x406, row text x407 (aligned), selection x399..880 y276..312 accent (167,139,250) with base text (17,19,26) 6.8:1 on Ink, white on (109,74,255) 5.2:1 on Paper. Residuals R8. |
| #13 layout chip | UNVERIFIED | FIXED | layout-*: chip x580..700, y372..428 (120 x 56), centred at (640,400) exactly on the tile gap and screen centre, surface fill + hairline + radius ~14, 20 px semibold "Tile left" (Ink (230,232,239) / Paper dark), hairline 2 device px at 2x. No yellow marker anywhere (accepted: not in design). |
| #19 tooltip theme | UNVERIFIED | FIXED | tooltip-*: x1111..1267, y49..84 (157 x 36), surface + line hairline, 14 px text "Every key (Super+/)", right edge flush with the bar and tile (1267), 9 px under the bar. Residuals R9. |
| #20 fractional scale | COSMETIC | FIXED (verified) | 1.25 run exists; see 1.25 section. Residuals R6, R10. |
| #16 tag colour | COSMETIC | OPEN | Unchanged (muted, spec says 50 %). |
| #17 terminal fill | COSMETIC | OPEN | Ink tile (22,25,34) is neither base nor surface. |
| N5 pill jump | COSMETIC | OPEN | Fullscreen pill y24..55 vs tiled y64. |
| N6 "Chat ..." | COSMETIC | OPEN | Three ASCII dots still. |
| #1-#5, #9, #14, #15, #18, N1-N4, N7 | | FIXED, held | Island, card, fullscreen, chips, border (46,51,68)/(213,216,225), glow and shadow all unchanged and correct at 1x/2x/1.25. |

Unverified in r2 (#11, #12, #13, #19): #11 FIXED, #12 OPEN (COSMETIC), #13 FIXED, #19 FIXED.

## 1.25 run (desktop ink + paper, 1600x1000)

Text is crisp (hinted per device px). Tile border is 2 device px solid on top and left (x15..16, 2.5 logical-to-device rounding), bottom border is 2 px plus one partial row (81,131,149)/(112,162,192) = 2.5 px, a slight top/bottom asymmetry. Bar hairline is 1 solid px plus a faint 25 % neighbour ((28,31,42) at y11 and y48): a soft double edge. Island/pill geometry scales correctly (bar y10..49, tiles inset 15). The bar icons (Apps grid, keyboard, power) are visibly soft next to crisp text; the separators are 2 px soft grey bars. Chip, pill and tag text were fine. No misalignment found.

## New findings

| # | Rating | File / coords | What is wrong | Fix hint |
|---|---|---|---|---|
| R6 | MINOR | desktop-*-2560x1600@2 and 1.25, bar x24..42 y14..33 (Apps), keyboard x1170..1185, power x1238..1256 | Bar icons are 1x rasters scaled up: at 2x the Apps grid has a ~4 device px blurred edge (mid-tone pixel count 336 vs ~144 expected) beside perfectly sharp text; same at 1.25. Sharp at 1x. | Ship icons as SVG (or render at the output scale), do not bitmap-scale. |
| R7 | MINOR | question-*: card x696..1255 y574..743, all text | Question card is set in a fallback sans (Arial/Liberation, narrow, tall x-height); the bar, toast, chat, tooltip, launcher are Inter. The security prompt is the one helper with a different typeface; all lines are one size, title only bold. | Set the card's font-family to the token font; title 14 semibold, body regular, path monospace or muted. |
| R8 | COSMETIC | launcher-*: panel x385..894 y214..625 (1x) vs x376..904 y212..629 (2x) | Launcher is 510 x 412 at 1x but 264 x 209 logical at 2x... i.e. 528 x 418 logical-equivalent: about 3.5 % wider and 1.5 % taller at 2x, row pitch 37 vs 36; size follows font cells, not px. Selection bar has square corners inside a 13 px radius panel; panel has no shadow (flat on a white Paper tile, hairline only); panel sits 20 px below centre (189 above / 175 below). | Fixed width/height in config, round the selection (6-8 px), add the island shadow if the launcher can draw one. |
| R9 | COSMETIC | tooltip-*: x1111..1267 y49..84 | Tooltip has no shadow (Paper y90 identical to desktop) and covers the Chat pill (the pill label is hidden, only its lower hairline shows) and the tile's top border. | Add the island shadow; anchor below the pill (y>=100) or left of the bar button. |
| R10 | COSMETIC | 1.25 captures, bar y10..49, tile bottom y982..984 | Faint 25 % hairline neighbour on the bar edges and a 2.5 px bottom tile border (see 1.25 section). | Snap hairlines to device px (round the 1 logical px to 1 device px); tile border 2 px or 3 px both sides. |
| R11 | COSMETIC | bar left island: x12..436 (layout label "Tile left") vs x12..410 ("Tile") | The island width changes with the layout name, so its right edge and the label jump when the layout changes (toast/chat/keys captures show "Tile", launcher/layout/tooltip "Tile left"). | min-width on the layout pill sized for the longest name. |
| R12 | COSMETIC | keys-*: bar y8..40, clock x566..714 | The bar and clock stay undimmed (26,29,39) while the chat pill, chips and orb take the 0.20 scrim (24,27,36), a 2-level step. Intentional "bar above scrim" is fine; the others could match. | Decide one: dim all helpers or none. |
| R13 | unverified | chat + question/toast | The chat panel (x868..1267) overlaps the question card (x696..1255) and toast (x896..1255) footprints; the chips moved aside but the card/toast did not, and there is no capture of both together. A hidden yes/no prompt would be a MAJOR. | Capture chat+question; stack the card above the panel or give it a higher layer. |

## Verdict: MERGE-OK

No MAJOR at 1x, 2x or 1.25 in either theme. The r2 MINORs are closed (R1 chips step aside, #7 scrim uniform 0.20 theme-base), and the launcher, layout chip, tooltip and 1.25 run are in and on-token (surface, line, accent, 2 device px hairlines at 2x, exact centring). Two new MINORs keep the "nothing above COSMETIC" bar unmet: soft raster bar icons at 2x and 1.25 (R6) and the fallback typeface on the question card (R7). R13 needs one capture.

Counts. r2 items (27 = 20 old + 7 r1-new): FIXED 19 (#1-#5, #7, #9, #11, #13, #14, #15, #18, #19, #20, N1-N4, N7), OPEN 8 (#6, #8, #10, #12, #16, #17, N5, N6; all COSMETIC), REGRESSED 0, UNVERIFIED 0. r2-new (R1-R5): FIXED 1 (R1), OPEN 4 (R2-R5, COSMETIC). New in r3: MAJOR 0, MINOR 2 (R6, R7), COSMETIC 5 (R8-R12), plus 1 unverified (R13). Open by severity: MAJOR 0, MINOR 2 (R6, R7), COSMETIC 17 (#6, #8, #10, #12, #16, #17, N5, N6, R2-R5, R8-R12), 1 unverified (R13).
