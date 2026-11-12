# Pixel review: chora-tiling, branch claude/tiling-pixel @ cbdf1eba, round 2 (2026-11-12)

Reviewer did not build it. Captures: media/2026-11-12/tiling-pixel-r2/*.png (chat, desktop, fullscreen, keys, question, toast; Ink and Paper; 1280x800 and 2560x1600@2 = 24 PNGs). Measured with PIL (1x px). Each 2x was BOX-downsampled and diffed against its 1x: only glyph antialiasing differs; at 2x borders are exactly 2 logical px = 4 device px (tile border x26..29 = 4 px, bar hairline 2 px, panel edges 466/467 and 2090/2091), so all coordinates hold at both scales. No launcher, layout chip, tooltip or fractional-scale (1.25/1.5) capture exists.

Backdrop is now a real wallpaper (dark violet/teal gradient on Ink, lilac/mint on Paper); focus glow and shadow are visible: left of the focused tile x0..11 y400 ramps (31,28,47) to (52,47,79) up to the 2 px accent border (148,163,240); gap x635..644 carries the same violet bleed. Paper shows grey drop shadows under tiles and the chat panel.

## Earlier findings (r1 table, #1-#20 and N1-N7)

| # | Was | Now | Evidence |
|---|---|---|---|
| 1 chat panel | FIXED | FIXED | Island x868..1267, y52..787, 12 from edges, hairline (46,51,68), fill (26,29,39) = surface; Paper (255,255,255)/(213,216,225); shadow now visible in Paper (x1268 (203,204,214)). |
| 2 question card | FIXED | FIXED | Card x696..1255, y574..743 surface + hairline; Yes x710..798, No x806..894 y694..725 surface-2 (36,40,54); Paper (242,243,247). |
| 3 fullscreen | FIXED | FIXED | No bar in either theme, content at x16; same at 2x. |
| 4 chat pill | FIXED | FIXED | Pill x1181..1255, y64..95, surface/line, 12 from tile top and right. |
| 5 toast | UNVERIFIED | FIXED | New captures: toast x896..1255, y664..737, fill surface (26,29,39), edge (46,51,68); Paper white/(213,216,225); sits 12 from tile right, 14 above the chip (y751), clear of the dot. Residual types in R3/R4. |
| 6 keys panel size | OPEN | OPEN (top fixed) | Panel x233..1045, y52..747 = 813 x 696: top now on the 52 grid, centred (233 left / 234 right, 52 top / 52 bottom). Still not the spec 880 x 560 (spec or build, pick one). COSMETIC. |
| 7 overlay scrim | OPEN | OPEN | Panel fill now (26,29,39)/(253,253,253) (0.97 alpha, ghost under it is 1 level, invisible). Scrim still treats helpers unevenly: bar island stays full, but the chat pill goes (26,29,39) to (19,22,29) and in Paper (255) to (191,191,191) grey, the chips likewise; the wallpaper tint is flattened to neutral grey in Paper. MINOR. |
| 8 key columns | OPEN | PARTLY FIXED, OPEN | All caps in a column now share one width (col 1 x257..432 = 176, col 2 x673..856 = 184, every row, both themes); label x constant 443/867. Still: "Super+r / Super+Space" is one cap x257..432 y592..609 with " / " in it (split into two caps with "or"), and row pitch 22 not 24. COSMETIC. |
| 9 left island | OPEN | FIXED | Island x12..410 (398), shrunk to content; "Tile" is a surface-2 pill x361..404, y13..36 with text (230,232,239). |
| 10 bar icons | OPEN | PARTLY FIXED, OPEN | Power icon and keyboard now text colour (230,232,239) like labels (Paper (27,30,39)); sizes still 14x14 (apps), 16x12 (keyboard x1170..1185 y18..29), 14x14 (power); gaps kbd-to-Keys 10, Keys-to-power 15. COSMETIC (keyboard glyph size). |
| 11 fuzzel radius | UNVERIFIED | UNVERIFIED | No launcher capture. |
| 12 launcher content | UNVERIFIED | UNVERIFIED (worker: open) | No capture. |
| 13 layout chip + marker | UNVERIFIED | UNVERIFIED | No chip capture; no yellow marker in any of the 24 PNGs. |
| 14 chips/orbs | OPEN | FIXED | Chips "Question for you" x1094..1224, "Listening" x1144..1224, y751..775 (24 high), fill surface (26,29,39), hairline, 14 px text; orbs accent (167,139,250) and urgent (248,113,113); orb right edge 1255 = 12 from tile. Paper: white chip, hairline. |
| 15 border token | FIXED | FIXED | (46,51,68) / (213,216,225) everywhere. |
| 16 empty tag colour | OPEN | OPEN | Tags 2..9 brightest (154,161,180) = muted, spec says 50 %. Update spec. COSMETIC. |
| 17 terminal fill | OPEN | OPEN | Ink tile fill (22,25,34) is neither base (17,19,26) nor surface (26,29,39); Paper is white = surface (fine). COSMETIC. |
| 18 focus shadow | OPEN | FIXED | Glow and depth visible on the wallpaper (see top), same ramp at 2x. |
| 19 tooltip theme | UNVERIFIED | UNVERIFIED | No tooltip capture. |
| 20 fractional scale | OPEN | OPEN | 1x/2x identical; no 1.25/1.5 capture. COSMETIC. |
| N1 chat Close | MINOR | FIXED | Close x1166..1253, y92..119 right-aligned flush with the field's right edge (1253); text left x882. Residual R5. |
| N2 Paper surfaces | MINOR | FIXED | Bar, clock, pill, card, chat, toast all (255,255,255) in Paper; Ink bar (26,29,39) = panel surface; keys panel 0.97 alpha (252-253), imperceptible. |
| N3 glow / wallpaper | MINOR | FIXED | See top. |
| N4 round chat field | MINOR | FIXED | Field x882..1253, y738..773, corner radius ~6-8 (first px per row 887,885,884,883,883,882), surface-2 fill, hairline. |
| N5 pill jump in fullscreen | COSMETIC | OPEN | Fullscreen pill still y24..55 (tiled y64); 24 from top, not 12. |
| N6 "Chat ..." | COSMETIC | OPEN | Still three ASCII dots on the pill (desktop, question, toast, fullscreen). |
| N7 panel over tile | COSMETIC | FIXED | Panel now has a visible shadow in Paper and a clean hairline; text cut at x868 is accepted overlay behaviour. |

## New findings

| # | Rating | File / coords | What is wrong | Fix hint |
|---|---|---|---|---|
| R1 | MINOR | chat-ink/paper: panel x868..1267, field y738..773 | The open chat panel covers the status chip and orb (x1094..1255, y751..775). While chatting the user cannot see "Listening" or, worse, a pending "Question for you" orb. The question card was captured only with chat closed, so the combination is untested. | Anchor chips above the panel (x = panel left - 12) or inside its header; capture chat+question together. |
| R2 | COSMETIC | toast-*: text x909..1230, y677..725 | Toast text is drawn with LCD subpixel antialiasing: max channel spread 228 (red/blue fringes visible on Paper at 1x and 2x), while all other helper text is greyscale (spread 12-13). | mako/pango: `font-options antialias=gray` or set the cairo option for the notifier. |
| R3 | COSMETIC | toast/fullscreen-paper: orb x1231..1253, y753..775 | The accent orb has a 1 px ring in Ink `base` (17,19,26) at x1231: on Paper it is a black outline (the only black outline on screen); on Ink it is invisible. | Ring = theme `line`/surface, not fixed base. |
| R4 | COSMETIC | toast-*: title/body | Toast title 16 px bold and body 16 px light; the rest of the helper type is 14 px regular/medium. Light weight at 16 px on white is thin. | 14 px, regular, to match the type scale. |
| R5 | COSMETIC | chat-*: title x882..1227, y70..83 and Close y92..119 | The error line is a bold 14 px sentence acting as title, with Close on its own row under it; there is still no header row. Alignment is now correct. | Header row: muted title left, Close right on one line, error as body text. |

## Verdict: MERGE-OK

No MAJOR in either theme at 1x or 2x. The r1 chip, icon-colour, left-island, shadow/glow, Paper-surface, Close and chat-field findings are fixed and the toast is verified. Remaining visible work: scrim consistency (#7) and chat panel hiding the status chip (R1), then cosmetics. The "nothing above COSMETIC" bar is not yet met (2 MINOR), and the launcher, layout chip, tooltip and fractional-scale captures are still missing.

Counts. Old 20 plus 7 r1-new: FIXED 14 (#1-#5, #9, #14, #15, #18, N1-N4, N7), OPEN 11 (#6, #7, #8, #10, #16, #17, #20, N5, N6; of these #8 and #10 partly fixed), UNVERIFIED 4 (#11, #12, #13, #19), REGRESSED 0. New in r2: MAJOR 0, MINOR 1 (R1), COSMETIC 4 (R2-R5). Open by severity: MAJOR 0, MINOR 2 (#7, R1), COSMETIC 13 incl. unverified-by-capture items not counted.
