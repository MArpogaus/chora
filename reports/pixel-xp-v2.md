# Pixel review: xp v2, build 8f358fa (read-only review, 1x unless said)

Method: all captures viewed; user icon, tray, caption buttons, taskbar, desktop icons cropped at 5-8x nearest-neighbour.
Captures: UX=media/2026-11-12/ux-deep-xp, R3=media/2026-11-12/xp-v2-r3. xp2x = same scene at scale 2 (coords /2).
Counts: MAJOR 6, MINOR 10, COSMETIC 8 (24 findings in the table; the tray padding, clock clip and pager size sit in section B).

## The two things the user saw

**A. Start menu user icon** (UX/xp1x-02-start-menu.png, tile x8-65, y368-420; same in xp2x-s2-02 at x16-130, y736-840, R3/start-menu.png).
The tile is lopsided: a white 2 px rounded ring spans x8-57 but the tile is 8 px wider, so a lilac (#C3D0EA) band shows
right of the ring (x57-65) and a hard dark shadow sits right and below (x65-68, y418-422). Left, top and bottom have no band.
The avatar is centred in the ring horizontally (x16-50) but sits low: head top y379 is 7 px under the ring, body bottom y414 is 1-2 px
from the ring. The body is cut flat at the bottom while the ring has round corners, so the figure looks clipped. Head/shoulder edges are
soft (fractional scale). Looks like a 48 px image drawn into a 56+8 slot. Fix: one square tile (48 or 56), equal 2 px ring all round, figure
with 6 px margin, bottom of the body clipped by the ring radius, shadow 1 px or none.

**B. Tray** (UX/xp1x-02, tray x1118-1280, y765-800; xp2x-s2-02 x1750-2560).
- Speaker is broken, not just small: only a white 3x6 block (x1166-1169, y779-785) is drawn, the cone is white-on-blue half transparent so it
  vanishes, and a 15 px red no-sign (x1169-1184, y774-790, #F03020, off palette, tray icons are otherwise white) is stacked on it, centre
  x1176 vs speaker centre x1168: not concentric. It reads as a stray square plus a red stop sign. Tooltip says "Sound unavailable"
  (xp1x-28b), so the no-sign is permanent on this VM, but the base glyph must still read as a speaker.
- Paperclip glyph x1126-1142, y774-792: 16x18, taller than the 16 px grid, 1 px aliased stroke with uneven thickness, blotchy at 1x;
  a different stroke weight and size from the speaker. At 2x (xp2x-s2-02 x1760-1785) it is thicker and clean, so it is not scaled from one master.
- Spacing: tray edge x1118 to clip 8 px; clip to speaker 24 px; speaker to clock 25 px; clock end x1261 to screen edge 19 px.
  Spec says 12 px ends, 8 between: build still the pre-fix padding. Right end 19 px, left end 8 px: unequal.
- Clock "3:26 AM" last glyph: the right stem of "M" (x1260-1261) is a paler blue column, i.e. a 1 px clipped or sub-pixel edge.
- Pager buttons are about 23x23 with 3 px gaps (x1010-1112) instead of 24 with 2.

## Ranked findings

| # | Rate | Capture | Where (x,y 1x) | What is wrong | Fix | Automatable |
|---|---|---|---|---|---|---|
| 1 | MAJOR | UX xp1x-02, R3 start-menu | 8-65,368-420 | User tile lopsided: 8 px lilac band right, shadow, figure low and cut flat | see A | tile ring width equal on 4 sides; avatar bbox centred in ring (+-1 px) |
| 2 | MAJOR | all with tray | 1166-1184,774-790 | Speaker glyph reduced to a white stub, no-sign off-centre, red off palette | redraw 16 px speaker, cone white, no-sign concentric, palette red #D8321F max | icon bbox centre != glyph centre; icon colour not in palette; glyph ink < 30 % of box |
| 3 | MAJOR | xp1x-10, 11, 12, 28b | 683-757,58-80 | Caption buttons 22x22 with 4 px gaps (spec 24, gap 2); inactive close is dull mauve (#B8667A), not XP red | 24x24, 2 px gap; inactive keep red at lower alpha | measured button size != spec; gap != spec |
| 4 | MAJOR | UX xp2x-s2-06-notify | 1448-1928 (2x:1448-1928,1107-1187) | Notify balloon covers the paperclip body; tail points at the clock/tray, not its source | stack balloon above the clip or move clip; tail to the tray icon | rect overlap between balloon and mascot; tail tip not within 2 px of an anchor bbox |
| 5 | MAJOR | UX xp1x-44 | 1216-1232,620-640 vs clip 1200-1255,655-755 | Chat tail ends 18 px above and right of the head (head x1210-1235): xp-v2 polish #14 not fixed | tail tip on head | tail tip to anchor distance > 4 px |
| 6 | MAJOR | UX xp1x-44 | 946-1246,530-605 | "Typing does not answer." outside the card; field still 2 lines (polish #26 open); grey #E8E8E8 path strip with border not in palette | move into card; one-line field; path strip in #ECE9D8 | text outside its group box; colour not in palette |
| 7 | MINOR | xp1x-02 | 8-38,445-470 and others | Start-menu icon sizes vary: Terminal 30, chat dot 16 (x14-31), Vim 30 uneven, LibreOffice 28, ImageMagick is a dotted 20 px fragment (x8-36,y573-590) with no art | 32 px slot, fallback glyph when art < 24 px, never draw raw SVG scraps | icon size not in {16,24,32,48}; ink bbox < 50 % of slot |
| 8 | MINOR | xp1x-02 | 42-215,589 | "ImageMagick (color depth..." truncated with "..." and runs to x215, past the 210 column edge (clips into the divider) | clip to column minus 8 px, shorten name | text bbox crosses column edge |
| 9 | MINOR | xp1x-02 | 231-420,440-690 | Right-column icons are 3 styles: pale folders, saturated My Computer, grey line-art Run/Control Panel | one 24 px full-colour set | icon mean saturation outlier vs set |
| 10 | MINOR | xp1x-04 control panel | 421-632,423-620 | Submenu icons 14 px, "Effects" has no icon (empty 20 px gutter), Desktop has a purple glyph, others grey; text not aligned to menu parent rows | same size and a placeholder glyph | row with empty icon slot; icon size not in set |
| 11 | MINOR | xp1x-10/11/12 | title icon 61-73,62-75; task icon 134-146,773-786 | Terminal icon 12x12 in title bar and taskbar (spec 16), glyph ">_" unreadable, grey border; at 2x it is 16 | 16 px at 1x, one asset | icon size not in set |
| 12 | MINOR | xp1x-10 | 126-306 vs 310-490, 770-796 | Task buttons: inactive is lighter, active darker but 1 px inset line only; text baseline of "foot" identical, icon 2 px above text centre (y773-786 vs text 777-788) | centre icon and text on one axis | icon centre y != text x-height centre y |
| 13 | MINOR | xp1x-01..51 | 8-80,12-70 desktop | Desktop icons ~28 px (folder x34-62) with a 2-line label; label "My Documents" starts at x13 (centred to 48) while spec grid 80x88 from 8,8; icon art tiny vs 32 px menu icons; Recycle Bin 8 px from edge but label ends 13 px | 32 px icons, 80x88 cells | icon size not in set; cell origin != 8,8 |
| 14 | MINOR | xp1x-44/43 | 940-1270,172-212 | Header "Atlas - Question for you" bold, the "Write file" grey line is 14 px but 11 px of air to the rule at y223; Stop (1176-1232) and close box (1240-1257) not same height (28 vs 18), tops misaligned (175 vs 177) | align on top or centre | adjacent control tops/centres differ |
| 15 | MINOR | R3 turn-off | 483-797,300-500 | Dialog: Cancel is a white 56x16 box (not XP 80x28 button), black 1 px edge square, title bar has no frame bevel, icons 32 px but labels 12-13 px; desktop dim grey also greys the tray text | push button 80x28; 14 px labels | button size != 80x28; text < 14 px |
| 16 | MINOR | R3 run | 405-875,240-560 | Run dialog is a bare list: no title bar, OK/Cancel or icon; blue 2 px border; selection #316AC5 ok; list row 29 px vs 36 spec | give it title and buttons or call it a launcher | window without title bar; row height != spec |
| 17 | COSMETIC | xp1x-02 | 0-420,720-724 | Green 2 px seam at y720 between left column and footer and a 1 px dark line at x419; footer text "Log Off" row 3 px low vs icons | align | -- |
| 18 | COSMETIC | xp1x-02 | 151-166,692-712 | "All Programs" arrow is a flat green triangle with no outline, rest of icons have gradients | gradient arrow | -- |
| 19 | COSMETIC | R3 help-keys-end | 1005-1027,37-58 | Help close button 22x22 (spec 24) bright orange; title text 14 px bold ok; keycap bottom shadow 2 px ok | 24x24 | caption size != spec |
| 20 | COSMETIC | R3 help-keys-end | 496-1004 | "Super" and "Ctrl+Escape" caps side by side with no "or"; descriptions x710 not aligned to cap column widths (fine) | add "or" | -- |
| 21 | COSMETIC | xp1x-10, xp1x-28b | 28-200,60-75 | After F1 the terminal shows "OP" and the old swarm text; cursor box 7x11 in white, not a caret style match | n/a | -- |
| 22 | COSMETIC | xp1x-02 vs xp2x | 1x vs 2x | At 2x text is crisp Tahoma-like, at 1x text has heavier hinting and menu labels are 13-14 px with 1 px different baseline per row (rows 36 px, text y445,493...) | verify baseline per row = row centre +1 | baseline offset to row centre differs across rows |
| 23 | COSMETIC | tray vs taskbar | 1118-1119 | Tray left edge: 1 px light cyan line then dark blue line 2 px wide at 1x, 1 px at 2x: border width not scaled | 1 px logical both scales | border thickness 1x vs 2x ratio != 2 |
| 24 | COSMETIC | xp1x-02 | 0-420,362 | Menu top corners radius 6 but the green hill shows through a 1 px halo at x0-4,y362-368 | clip corner pixels | -- |

## Automatable checks (distinct classes)
- Icon slot size not in {16,24,32,48} (items 7, 9, 10, 11, 13): read from the render tree, no pixels needed.
- Icon ink bbox vs slot: ink < 50 % of slot, or ink centre != slot centre +-0.5 px (1, 2, 7).
- Icon bbox not on whole pixels, or scaled non-integer (blur): compare 1x and 2x renders, 2x downscaled must equal 1x within a tolerance (1, 2, 11).
- Ring or border width unequal on four sides of one tile (1).
- Icon colour outside the look palette or saturation outlier in a set (2, 9).
- Control size != spec (caption 24x24, pager 24x24, push button 80x28) and gap != spec (3, 15, 19).
- Adjacent control tops/centres differ (14); icon centre != text centre in one row (12).
- Rect overlap between two floating surfaces (balloon vs mascot) (4); tail tip to anchor distance (5).
- Text bbox crosses its column or container edge; ellipsis present (8); text outside its group box (6).
- Text size below 14 px (15, 22 variant).
- Empty icon slot inside a list that has icons (10).
- Clipped glyph: last text column of pixels differs in colour from the rest of the glyph (clock "M").
- Border thickness ratio 2x/1x != 2 (23).
- Padding at tray ends != spec (12/8) measured from bbox of items (tray list).
