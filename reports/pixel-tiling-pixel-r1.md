# Pixel review: chora-tiling, branch claude/tiling-pixel @ 5ec90579, round 1 (2026-11-12)

Reviewer did not build it. Captures: media/2026-11-12/tiling-pixel/*.png (desktop, question, chat, fullscreen, keys; Ink and Paper; 1280x800 and 2560x1600@2). Measured with PIL (1x px). 2x was compared with a BOX-downsample of itself against the 1x capture: layout is pixel-identical (only text antialiasing differs; max diff bbox is glyphs), so every coordinate below holds at both scales. No toast, launcher, layout chip, tooltip or fractional-scale capture exists in this set.

Not in the set, so not judged: backdrop is pure black (0,0,0) in every capture (no wallpaper), so shadow and glow depth cannot be seen (see N3).

## Old findings (v2 report #1-#20)

| # | Was | Now | Evidence |
|---|---|---|---|
| 1 chat panel | MAJOR | FIXED (residuals N1, N2, chat field) | Island x868..1268, y52..788: 12 from right/bottom, 12 below the bar, radius ~10 (first panel px per row 878,874,872,871,871,870...), fill (26,29,39) = surface, 1 px hairline (46,51,68), text/muted tokens, Close = surface-2 pill (36,40,54) radius ~8. Paper: #fff, hairline (213,216,225). |
| 2 question card | MAJOR | FIXED | Card x696..1255, y574..743, radius ~10, surface fill, hairline, 14 px padding; Yes/No pills x710..798 and 806..894, y694..726, surface-2, radius ~8; Paper variant correct. |
| 3 fullscreen | MAJOR | FIXED | No bar in either theme; terminal text starts at 0,0; no ghost text under islands. Tooltip-over-content not capturable here. |
| 4 chat pill | MAJOR | FIXED | Pill x1185..1255, y64..95: 12 from tile right (1267) and 12 from tile top (52); surface fill, hairline, text token; no overlap with bar; same at 2x (no ghost under Keys island). Paper: white pill. |
| 5 toast | MAJOR | UNVERIFIED | mako config not checked by pixels, no toast capture. Needs a capture before it can be marked fixed. |
| 6 keys panel size | MINOR | OPEN | Panel x233..1046, y54..746 = 813 x 692 (spec 880 x 560), top 54 is 2 off the 52 grid line. |
| 7 overlay backdrop/scrim | MINOR | OPEN | Backdrop black. Scrim darkens tiles to (27,27,27) from (36,36,36) but not the bar, pill or chips (in keys-paper the Chat pill and Listening chip go grey while the bar stays bright). Paper panel fill is (237,237,237), not #fff. Ink panel (26,29,38) ok. |
| 8 key list columns | MINOR | OPEN | "Super+r / Super+Space" still one cap x257..432 (keys-ink y592..609); caps still ragged (cap right edges 525/524/537/564 per row); row pitch 22. Label x constant (443/444) is good. |
| 9 left island dead space | MINOR | OPEN | Island x12..459 (448 wide), content ends ~391 ("Tile"); 68 px empty; "Tile" muted (154,161,180), no affordance. |
| 10 bar icons | MINOR | OPEN | Apps 14x14 (x26..39,y17..30), keyboard 16x12 (x1170..1185,y18..29), power 14x14 (x1240..1253,y17..30); power icon muted (154,161,180) while "Keys" is text (230,232,239); Keys label to power gap 15. |
| 11 fuzzel selection radius | MINOR | UNVERIFIED | No launcher capture. |
| 12 launcher content | MINOR | UNVERIFIED | No launcher capture. |
| 13 layout chip + marker | MINOR | UNVERIFIED | No chip capture. No yellow marker anywhere in these 20 PNGs. |
| 14 status chips/orbs | MINOR | OPEN | "Question for you" chip x1138..1224, y753..773 and "Listening" chip x1173..1224: still white (250,250,250) with ~11 px text and radius ~3; orbs still (224,101,58) orange and green, not accent/urgent tokens; in Ink these white chips are the brightest objects on screen. Chips are 12 from the tile edge now (orb right 1255), improved. |
| 15 border vs token | MINOR | FIXED | Bar and keys/chat/pill hairline = (46,51,68) = line exactly; Paper (213,216,225) exact. Unfocused tile border (42,46,61) on black, within alpha of the token. |
| 16 empty tag colour | COSMETIC | OPEN | Empty tags (154,161,180) full muted; spec says 50 %. Update spec. |
| 17 terminal fill | COSMETIC | OPEN (changed) | Terminal is now neutral grey (36,36,36), still not a token (base #11131a, surface #1a1d27); it looks warm next to the blue-grey islands, and in Paper it is a dark block. |
| 18 focus shadow | COSMETIC | OPEN / not assessable | Black backdrop hides any shadow; gap 635..644 is (0,0,0). |
| 19 tooltip theme | COSMETIC | UNVERIFIED | No tooltip capture. |
| 20 fractional scale | COSMETIC | OPEN | 1x and 2x are identical in logical px, edges integer (borders 2 px, gaps 12/10: focused tile x12..634, gap 635..644, next tile 645..). No 1.25x/1.5x set. |

Verified OK, no finding: bar 32 high at y8..39, 12 from sides, islands 12..459 / 566..713 / 986..1267; tile gaps 12 outer, 10 inner; focus border 2 px gradient (violet to teal; TL (148,163,240), BR ~(112,211,221)); clock island centred; Paper and Ink tokens on islands, panel, card and pill match.

## New findings

| # | Rating | File / coords | What is wrong | Fix hint |
|---|---|---|---|---|
| N1 | MINOR | chat-*-1280x800.png: Close pill x976..1064, y94..119; title x882..1217, y61..89 | The empty/error state is a bold 14 px sentence acting as title, and the Close button floats at x976 (centre 1020): neither left-aligned with the text (882) nor right-aligned in the island (1254). Reads as a layout bug. There is no panel header. | Put a header row (title muted, Close right-aligned at x1254) and render the error as a normal 14 px body line. |
| N2 | MINOR | Paper captures: bar fill (235,235,235), keys panel (237,237,237), chat panel/question card/chat pill (255,255,255) | Paper surfaces are three different values; Ink is one (24..26,26..29,36..39). Pill and panel look brighter than the bar in Paper. | One `surface` token for all islands (bar included); keys panel solid, not translucent. |
| N3 | MINOR | desktop-ink x0..11, y400: all (0,0,0) up to border (148,163,240) at x12; gap x635..644 black | No outer halo or glow at all around the focused tile, and the capture backdrop is pure black, so the "focus glow" and "smooth depth" of the brief cannot be seen in any capture. | Capture on the real wallpaper; if the glow is intended, add 0 0 18 accent 35 % and verify a coloured ramp outside the 2 px border. |
| N4 | MINOR (carry from worker list) | chat-*: field x882..1254, y738..773 | Chat input is a square box (corners 90 degrees, 1 px border) inside a radius-10 island with radius-8 pills. | border-radius 8, 12 px inset, accent focus ring. |
| N5 | COSMETIC | fullscreen: pill y24..56; tiled: y64..95 | Chat pill jumps 40 px vertically between fullscreen and tiled; in fullscreen it is 24 from the top, not 12. | Anchor to output top + 12 in fullscreen. |
| N6 | COSMETIC | chat/question/desktop: label "Chat ..." | Three ASCII dots, not an ellipsis; a "Chat" button that ends in "..." reads as loading. | "Chat" or "Chat with a helper". |
| N7 | COSMETIC | chat-*: panel x868..1268 over right tile | Panel overlays the tile and cuts terminal text mid-glyph at x868 instead of reflowing. Acceptable, but the tile border and text peek at the panel's rounded corners (x868..878, y52..62). | Optional: shrink tiles when the panel is open, or add a drop shadow. |

## Verdict: MERGE-OK

The four MAJOR items of v2 (#1-#4) are fixed on both themes and both scales. No new MAJOR. Remaining visible work: status chips (#14), bar icon sizes (#10), left island width (#9), keys panel size/caps (#6, #8), chat panel Close and input (N1, N4), Paper surface consistency (N2).

Counts: old 20 = 5 FIXED (#1-#4, #15), 10 OPEN (#6-#10, #14, #16-#18, #20), 5 UNVERIFIED for lack of capture (#5, #11, #12, #13, #19), 0 REGRESSED. New: MAJOR 0, MINOR 4, COSMETIC 3.
Open by severity (old OPEN plus new): MAJOR 0, MINOR 10 (#6-#10, #14, N1-N4), COSMETIC 7 (#16-#18, #20, N5-N7).
