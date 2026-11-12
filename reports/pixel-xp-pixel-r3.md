# Pixel review: xp round 3, branch claude/xp-pixel @ 2e33b071 (fallback captures)

Method: all 1x (1280x800) and 2x (2560x1600) fallback captures in media/2026-11-12/xp-pixel-r3/ viewed; menu, cascade, turn-off, run,
chat-question, help, tray, taskbar, caption, title icons cropped at 4-10x nearest-neighbour and measured with PIL (scratch in
the session scratchpad, xprev3/). 1x vs 2x-downscaled diffs (pixels differing >90): menu 0.3 %, chat 0.25 %, turn-off 0.2 %, desktop,
hover, tooltip, balloon, paperclip 0.1 % or less; help 1.4 %, run 1.9 %, cascade 3 % (cursor/hover state only). Scale is clean.
**The asset benchmark (Bliss, Luna art, imported icons) is still owed**: Microsoft art stays off the repo, no asset captures exist for r3;
nothing about the asset look is confirmed here.

## Verdict: MERGE-OK (fallback) -- 0 MAJOR, 5 MINOR, 6 COSMETIC. Assets: not reviewed, benchmark owed.

## Normal-metric measurements (96 DPI)

| Metric | Spec | Measured 1x | 2x | Result |
|---|---|---|---|---|
| Taskbar | 30 | y770-799 = 30 | top y1540 = 2 x 770 | OK |
| Start button | 99x30 | x0-98 (edge at x99), y770-799 | x0-197 | OK |
| Task button | 160x24 | 2nd button x267-426 = 160, y773-796 = 24, 2 px gap to the first | 2x of that | OK |
| Caption bar | 25 | active window y155-179 = 25 | 2x | OK |
| Caption buttons | 21, gap 2 | boxes 21x21 (x892-912, 915-935, 939-959 incl. close), 2 px gaps, top 2 px below bar top, 4 px from the frame | clean 2x | OK |
| Chrome text | 11 px Tahoma-like | menu, task, tray, tooltip are about 11 px (x-height and cap height match the preview) | | OK (DejaVu-like face, N12 of r2 stays) |
| Title | 13 bold | bold white with a 1 px shadow, cap height about 9 | | OK |
| Start menu | 380 wide | x0-379 = 380, header y440-503 (64), footer y729-769 (41), left column 190 | 2x | OK |
| Balloon | 340 wide | x933-1270 = 338 incl. border, 410 tall at both scales, Yes/No 80x28, 8 px gap | same logical size at 2x | OK |
| Title / task icon | 16 | title icon x30-45, task icon x111-126 = 16 wide, cmd-style terminal glyph both | | OK |
| Stop vs close box | both 21 high, same top | Stop x1173-1228 y239-259, close x1237-1257 y239-259: 21 high each, same top | | OK |
| Desktop icon cell | 80x88 from 8,8 | label centred at x48 (cell 8-88), second icon 88 px below the first (labels y55 and y143), bin label centre x1232 = 8 px grid from the right edge | | OK |

## Worker's check list

- Green seam above the footer (y720): FIXED. The left column is plain white from y716 to y728, the footer starts at y729, no green line.
- First right-column row under the header: OPEN (N1). My Documents icon spans y503-529 and starts flush on the orange line (y504-505):
  the folder tab is clipped and the label (y512) sits 7 px under the rule while every later row has air. Left column Terminal row likewise touches the rule.
- Desktop icon cells 80x88: OK (above). Art is 28-32 px, label y50-62 inside the 88 cell. Icons are hidden by windows in most captures; only the
  fragment "My" leaks left of the windows at x12-24 (normal).
- Start flag position: fallback shows a ring glyph at x11-25, y777-792, baseline-aligned with "start" (italic bold), 12 px from the left edge. Fine for a
  fallback; the flag position is an asset question.
- Tray paddings: separator x1152; clip icon x1166-1177 (13 left, spec 12); speaker x1189-1203 with the badge; 12 px gap clip to speaker; clock text x1229-1268
  (11 right, spec 12). Within 1 px. Speaker: cone now reads as a cone with a red no-sign badge at the lower right (the glyph is 14x12 plus the badge): FIXED.

## r2 findings

| # | Status | Evidence |
|---|---|---|
| N1 chat balloon at 2x | FIXED | 2x balloon is 338x410 logical, same as 1x; header "Atlas - Working", Stop and close box present, card inside the balloon, tail on the clip head |
| N2 default scale | FIXED | all Normal metrics above hold at 1x and 2x |
| N3 speaker cone | FIXED | see above |
| N4 ImageMagick icon | OPEN | menu x8-36, y573-590 still a dotted red fragment (now N2 below) |
| N5 title/task icon | FIXED | 16 px, terminal icon, same in title and task button |
| N6 chat band and header controls | PARTIAL | Stop/close aligned (FIXED, finding 14); the empty band between card bottom (y551) and input (y591) is still 40 px (N3 below) |
| N7 fallback desktop icons | OPEN, COSMETIC | pale flat folder and computer on pale sky; cloud still sits to the right of the second icon (x85-292), no longer behind its label |
| N8 help capture at 1x | FIXED | help.png shows the Help and Support window at 1x |
| N9 stray pixel at column boundary | FIXED | none at x213 |
| N10 tray paddings | FIXED | 13/11 vs 12/12 |
| N11 help window buttons | OPEN, COSMETIC | Help has the close box only, no title icon, no min and max |
| N12 fallback Start font and glyph | accepted | |
| N13 Send enabled at 2x | FIXED | greyed at 1x and 2x with an empty field |
| 10 control-panel submenu | FIXED | cascade.png: flyout at x380-553, y608-765, top aligned to the highlighted row (#316AC5, white text), above the taskbar, 6 rows with icons and arrows |
| 14 Stop vs close | FIXED | both 21 high, same top y239 |
| 15 dialog Cancel | FIXED | turn-off.png Cancel x707-782 (76) by y467-487 (21), white to light grey push button |
| 16 run dialog | PARTIAL | run.png shows the external launcher (fuzzel), see N4 |
| 17 green seam | FIXED | see above |
| 20 "or" between Super and Ctrl+Escape | FIXED | help.png row 1: "Super  or  Ctrl+Escape" |
| 11, 12, 13, 18, 19, scale-2 taskbar | FIXED/unchanged | |

## New findings

| # | Rate | Capture | Where | What | Fix hint |
|---|---|---|---|---|---|
| N1 | MINOR | menu, cascade 1x/2x | right column x196-224, y503-529; left column Terminal row y505 | first rows of both columns touch the header rule; My Documents folder tab is clipped, 0 px air above the row | 4-6 px top padding under the header in both columns (shift all rows) |
| N2 | MINOR | menu | x8-36, y573-590 | ImageMagick still has no usable icon (fragment) | generic application icon when the art has under 24 px of ink |
| N3 | MINOR | chat-question | card bottom y551 to input top y591 | a 40 px empty band between the question card and the input (the transcript area keeps its minimum height) | let the transcript shrink, or put the band above the card |
| N4 | MINOR | run | x406-874, y241-559 | Run is the external launcher (fuzzel): beige #ECE9D8 panel with blue border, non-Tahoma, no title bar, no OK/Cancel, no XP "Run" look; accepted by the worker as external but it is visible to the person | theme fuzzel (font Tahoma 11, 1 px #0A246A border, #316AC5 selection already ok), or keep and note |
| N5 | MINOR | balloon | x1012-1192, y708-760 | the notification text says "Windows Update": a trademarked name, against the style guide ("no Windows name") | use "Updates" |
| N6 | COSMETIC | turn-off | whole dialog | desktop is not dimmed behind the dialog (XP greys it); Cancel is 76 wide, spec 73 | |
| N7 | COSMETIC | menu | orange rule y504-505 | the rule fades out at x272 (gradient), XP's runs full width then fades at the right: fine, noted only | |
| N8 | COSMETIC | cascade | submenu x380-553 | submenu rows have no separators and the "Keyboard Shortcuts" icon is a small grey glyph | |
| N9 | COSMETIC | chat | input x946-1186, Send x1194-1257 | input and Send differ by a few px in height (Send 28, input about 34) | match the heights |
| N10 | COSMETIC | paperclip, balloon | bin label bottom y603 to clip top y660 | the bin is 24 px above the clip art top (spec: 8 px above the clip window); not decisive without the clip window rect | |
| N11 | COSMETIC | tray | clip icon 11x15 | the tray clip reads as a thin outline, lighter than the speaker | |

## Counts

- Fallback: MAJOR 0, MINOR 5 (N1-N5), COSMETIC 6 (N6-N11, plus open r2 N7 and N11 as before). Verdict MERGE-OK.
- Assets: benchmark owed. No verdict.
- Previous r2 MAJORs (2x balloon, Large-vs-Normal scale) are both FIXED; no REGRESSED item.

Top 3: N1 rows flush on the header rule, N2 ImageMagick icon hole, N3 40 px band in the chat balloon.
