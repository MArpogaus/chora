# Pixel review: xp round 2, branch claude/xp-pixel @ 449b8326

Method: every asset and fallback capture at 1x (1280x800) and 2x (2560x1600) viewed; taskbar, tray, caption buttons, task buttons,
start-menu tile and columns, chat balloon cropped at 3-8x nearest-neighbour and measured with PIL. 1x vs 2x-downscaled diffs on
taskbar, menu and caption are within 1 % of pixels (scale is clean). Reference: kde-plasma-winxp-launcher preview-classic.png and
the winxp look-and-feel fullscreenpreview.jpg. No image is embedded or copied here; asset captures stay in the scratchpad.
Asset captures in words: the Bliss wallpaper fills the screen, the Luna start flag sits in a green Start button, the menu and
desktop show the imported full-colour icons (folders, computer, recycle bin, control panel, help), text is Tahoma-like.

## Verdicts

- Assets: **FIX** (1 MAJOR, 4 MINOR, 4 COSMETIC).
- Fallback: **FIX** (same MAJOR, which is scale-independent, 4 MINOR, 3 COSMETIC). Without the 2x balloon bug it would be MERGE-OK.

## Round-1 findings

| # | Status | Evidence |
|---|---|---|
| 1 user tile | FIXED | assets 1x tile x9-57, y370-418, 48x48, equal 2 px ring, avatar centred, no lilac band, no shadow |
| 2 speaker | PARTIAL (now MINOR N3) | glyph and no-sign concentric, red (about 200,50,40) on palette; the cone is still an unreadable white wedge cut by the slash |
| 3 caption buttons | FIXED | outline boxes 24x24 (x882-905, y157-180), gap 2 px between outlines, close red when inactive (x936-955) |
| 4 notify balloon over clip | FIXED | 1x balloon at 1011-1192,708-760, tail at x1160 on the tray clip icon; the mascot is uncovered |
| 5 chat tail | FIXED | tail tip about x1232,y650, over the clip head (x1215-1250,y655) |
| 6 chat field | FIXED at 1x, REGRESSED at 2x (N1) | 1x: one-line field, path strip (236,233,216) = #ECE9D8, no grey strip, "A spoken yes does not count." inside the card, tail on head. "Typing does not answer." is gone. |
| 7 menu icon sizes | PARTIAL | Terminal, Vim, LibreOffice, Calc now 32 px and clean; the ImageMagick icon (x8-36,y573-590) is still a dotted red fragment; chat dot is a 16 px ball in a 32 slot |
| 8 text truncation | FIXED, COSMETIC left | "ImageMagick (color depth..." ends x208, column edge is 210: only 1 px air |
| 9 right-column icon styles | FIXED with assets; fallback consistent flat blue set, acceptable |
| 10 control-panel submenu | UNVERIFIED | cascade.png and cannot-open.png show Control Panel highlighted with no flyout; no capture of the submenu |
| 11 title/task icon size | OPEN | task icon 12x12 at x135-146,y776-787 (320-331 second button); title icon about 13 px, a dark grey-bordered ">_" box, not the cmd icon used for Terminal in the menu. Spec 16 |
| 12 task button icon centre | FIXED | button y768-795, icon y776-787 and text y777-786 both centre 781.5; button 180x28 |
| 13 desktop icons | FIXED with assets (32 px art, label centred at x48, 8 px origin). Fallback: pale flat folder low contrast on sky (N7) |
| 14 chat header | OPEN | Stop (about 28 tall) and close box (about 18 tall) still different heights, tops differ |
| 15 dialog Cancel 80x28 | UNVERIFIED | no turn-off or run dialog in this capture set |
| 16 run dialog | UNVERIFIED | not captured |
| 17 green seam above footer | OPEN, COSMETIC | light green line at y720 under the left column, x0-210 |
| 18 All Programs arrow | FIXED | gradient green arrow |
| 19-21 | 19 help close 24 FIXED (help window only has close); 20 OPEN (no "or" between Super and Ctrl+Escape); 21 n/a |
| 22 | n/a | rows regular at 1x and 2x |
| 23, 24 | UNVERIFIED | not cropped this round |
| scale 2 black taskbar | FIXED | taskbar blue at 2x, top at y1528 = 2 x 764 |
| icon-size and clip audit | audit.log "33 nodes, no finding" yet N5 (ImageMagick, 12 px title icon) passes: audit gaps |

## New findings

| # | Rate | Capture | Where | What | Fix hint |
|---|---|---|---|---|---|
| N1 | MAJOR | chat-question 2x, assets and fallback | balloon 1460-1990,795-1265 (2x px) | At scale 2 the helper balloon is about 234 logical px tall instead of 409. The question card sticks out above the balloon (card top y755, balloon top y795), the header "Atlas - Working", Stop, close box and the transcript are gone: the Stop button is unreachable at 2x. Fallback same. 1x correct. | The balloon height is computed in a unit that is not multiplied by scale (or clamped to a physical-px limit); size by logical px, then scale |
| N2 | MAJOR | all, 1x and 2x | taskbar y764-799, Start 0-118 x 36, menu 420 wide, labels 14 px | The default chrome is XP "Large size" (about 1.2x: taskbar 36, Start 118x36, 14 px text, 36 px menu rows) while brief and reference say Normal 96 DPI (taskbar 30, Start 99x30, 8 pt Tahoma = 11 px, caption 25). The reference preview is visibly denser. If D1 was meant to stay, the brief needs a note; otherwise set scale 1.0 and make 1.2 the option | one logical unit table at 96 DPI; Large = x1.25 |
| N3 | MINOR | tray, 1x | 1180-1196,774-790 | Speaker cone still reads as a white wedge, not a speaker | draw the cone as a 16 px glyph, no-sign only as a corner badge |
| N4 | MINOR | menu | x8-36,y573-590 | ImageMagick has no icon art (raw fragment) | substitute the generic application icon when art is under 24 px ink |
| N5 | MINOR | title and taskbar | 61-73,62-75; 135-146,776-787 | 12-13 px grey-bordered terminal icon in both, 16 px spec, different from the 32 px cmd icon in the menu | use the same cmd icon at 16 |
| N6 | MINOR | chat 1x | card 946-1258,378-546 to field y585 | About 40 px empty band between card and input; header controls mismatched (finding 14) | tighten, align Stop and close tops |
| N7 | MINOR | fallback desktop | 8-80,8-100 | Pale flat folder and computer icons on a pale sky: label legible, art nearly invisible; a cloud of the fallback wallpaper sits behind the second label | darker outline or fill, move the cloud |
| N8 | MINOR | help capture 1x | help.png | The "help" capture shows no Help window (2x does): capture script timing, not a build bug | recapture |
| N9 | COSMETIC | menu right column | x213,y592 | stray dark pixel at the column boundary beside Control Panel | |
| N10 | COSMETIC | tray | 1158-1270 | tray paddings 14 left, 10 right of the clock, spec 12/12 | |
| N11 | COSMETIC | help window | title bar | no title icon, no min and max (XP Help has both) | |
| N12 | COSMETIC | fallback | Start button | circle glyph instead of the flag, intended; font is a DejaVu-like sans (no Tahoma) | acceptable |
| N13 | COSMETIC | 2x chat fallback | Send | looks enabled while the input is empty (1x is greyed) | |

## Counts

- Assets: MAJOR 2 (N1, N2), MINOR 5 (N3-N6, plus open 11 as N5), COSMETIC 4. Verdict FIX (N1 is a functional break at 2x; if N2 is a PM decision the verdict after N1 is MERGE-OK).
- Fallback: MAJOR 2 (N1, N2), MINOR 5, COSMETIC 4. Verdict FIX likewise.
- Unverified: 10, 15, 16, 23, 24 (not in the capture set; add the submenu, turn-off and run dialogs to the next round).

Top 3: N1 balloon height at 2x, N2 default scale 1.2 vs 96 DPI, N5/N4 icon holes (title/task icon, ImageMagick).
