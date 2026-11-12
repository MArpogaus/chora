# Pixel review: xp round 4, branch claude/xp-pixel @ a836cbd6 (fallback and assets)

Method: fallback captures media/2026-11-12/xp-pixel-r4/{1x,2x}/ viewed and cropped 3-4x, measured with PIL. Assets: cargo build --release, tests/xp-assets.sh
into ~/.cache/chora/xp-assets, then tests/xp.sh with CHORA_XP_ASSETS at 1x and with XP_SCALE=2 (both PASS); captures live in the session scratchpad only,
nothing of them is committed. 1x vs 2x-downscaled diffs (pixels differing >90): fallback and assets menu, chat, balloon, desktop, help, hover,
maximized 0.0 %, turn-off 0.07 %, cascade/run 1.5-1.7 % (cursor, hover state). Scale is clean in both modes. Asset captures are described in words and numbers only.

## Verdicts

- **Fallback: MERGE-OK** -- 0 MAJOR, 2 MINOR, 5 COSMETIC (r3 MINORs N1, N2, N3, N5 FIXED; N4 OPEN).
- **Assets: FIX** -- 0 MAJOR in layout, 3 MAJOR against "pixel perfect" (A1 Start menu header 65 px vs 54, A2 title face, A3 Run box), 5 MINOR, 3 COSMETIC.
  (If the PM counts only layout defects, assets are MERGE-OK; A1-A3 are the visible gaps against real XP.)

## Part A: r3 findings

| # | Status | Evidence |
|---|---|---|
| N1 rows flush on header rule | FIXED | menu 1x: header y427-491 (65), orange rule y492-493, first right row (My Documents) ink starts y502 = 9 px under the rule, folder tab whole; left Terminal row ink y509 (16 px). Same at 2x |
| N2 ImageMagick icon | FIXED | menu x8-36, y~625-655: a whole icon (tan box with pencils and ruler), no fragment. NB the worker says "generic"; it is a distinct art, fine. See B6 for the label |
| N3 chat band 40 px | FIXED | chat-question 1x: card bottom y577, input top y591 = 14 px (r3: 40); balloon grew 410 -> ~382 and still fits |
| N4 Run is fuzzel | OPEN | run.png: beige #ECE9D8 box with 2 px blue border, no title bar, no OK/Cancel/Browse, "Open:" prompt, list of apps; not the XP Run dialog |
| N5 "Windows Update" | FIXED | balloon title "Updates", text "Your updates are ready to install." |
| N6-N11 cosmetic | OPEN/unchanged | turn-off not dimmed; submenu flat rows; Send vs input height; tray clip thin; Help window close box only. All unchanged, COSMETIC |
| r2 N7, N11 | OPEN, COSMETIC | as r3 |

No REGRESSED item. Normal metrics re-measured and unchanged: taskbar y770-799 (30), Start 99x30 (edge x99), task buttons x106-265 and x267-426 (160, 2 px gap), caption y154-178 (25), 21 px buttons, Start menu x0-379, left column 190, footer y729-769.

## Part A: new fallback findings

| # | Rate | Capture | Where | What | Fix hint |
|---|---|---|---|---|---|
| F1 | MINOR | menu, run | x42-186, y640 | the ImageMagick row shows the raw name "ImageMagick (color depth=...", truncated; Run lists "ImageMagick (color depth=q16)" | strip a parenthesised suffix from Name, or use GenericName |
| F2 | MINOR | menu | header y427-491 | the header is 65 px; XP's is 54 (48 px tile + 3 px margins); the tile (x6-53) is 48 px and a flat person glyph, not XP's picture frame | shrink header to 54, rule stays 2 px, rows move up 11 px |
| F3 | COSMETIC | menu | user tile | white rounded square with a blue person: acceptable own art | |
| F4 | COSMETIC | menu | Vim, LibreOffice | non-Luna app icons (their own); fine in fallback | |
| F5 | COSMETIC | menu | Terminal subtitle "Type commands", "Chat with a helper": ok, XP has two-line pinned items | |
| F6 | COSMETIC | desktop | fallback icons pale | as r2 N7 | |
| F7 | COSMETIC | cascade | submenu | no separators | |

## Part B: assets benchmark (against XP SP3 Luna Blue)

Match (measured): taskbar body #2459D5-ish (34,87,213) vs XP #245EDC, 1 px light top edge (49,104,213 / 56,136,233 / 73,147,230) like XP's highlight, bottom darkening to (25,65,165) like XP; Start button green 99x30, flag
imported (x10-25, y776-791), "start" Franklin-Gothic-style italic with shadow; task buttons 160x24, active one darker and pressed, inactive light, both with 1 px border; caption 25 px, gradient (61,147,255)
top to (0,82,209) middle to (2,96,244) bottom, 21 px buttons with white 2 px frames, red close; Start menu 380 wide, right column #D3E5FA (211,229,250), the exact XP value, bold dark-blue top group, Control Panel with arrow,
Help and Support, Run; orange rule; blue footer with Log Off and Turn Off Computer; tray lighter blue gradient with clip, speaker, clock "8:05 AM"; Bliss as wallpaper (sky, cloud and hill correct, hill crest at y~430 of 800); Luna icons
for My Documents/Pictures/Music/Computer, Control Panel, Help, Run, cmd (C:\ in title, task and menu), recycle bin; Tahoma for menus and tasks. 2x: icons stay sharp, logical sizes identical.

| # | Rate | Capture | Where | What | Fix hint |
|---|---|---|---|---|---|
| A1 | MAJOR | menu | header y427-491 | 65 px header vs XP 54: the whole menu is 11 px too tall and the proportion differs; the 48 px tile is a flat glyph, not a Luna user picture frame (rounded white 2 px frame with the duck/flower picture; the imported set has user icons under icons/luna/res) | header 54; load the Luna user icon for the tile when present; keep the glyph as fallback |
| A2 | MAJOR | desktop, any title | title text "foot" x51-80 | title face is not Trebuchet MS Bold 13: "foot" is 27 px wide vs 24 px for trebucbd.ttf at 13 px rendered in PIL, rounder and heavier, with a smeared shadow (looks like faux-bold over bold, or 14 px). trebucbd.ttf is in the import (fonts/WindowsXP) | use trebucbd.ttf at 13 px with a 1 px #0A246A-ish shadow, no synthetic bold |
| A3 | MAJOR | run | x406-874, y241-559 | still the fuzzel box in assets mode; the central XP Run dialog (title bar "Run", icon, text, Open: combo, OK/Cancel/Browse) is a real XP feature the primary users know | own Run dialog node; or accept fuzzel and downgrade |
| A4 | MINOR | desktop | Start flag x10-25 | the imported flag renders 16x16; XP's is ~21x20 at x~7-8, so the button looks emptier; "start" label starts at x33 (XP ~x34) | scale the flag to its native size, check the 99 px layout |
| A5 | MINOR | menu | left column | Vim and LibreOffice icons are the apps' own (non-Luna), as are Log Off (orange key) and Turn Off (red power) tiles; XP shows the 24 px key and shutdown tiles of Luna | map to icons/luna/res (system-log-out, system-shutdown) when the import has them |
| A6 | MINOR | menu | x42-186, y640 | label "ImageMagick (color depth=..." (see F1) | as F1 |
| A7 | MINOR | desktop, tray | x1052-1146 | workspace buttons 1-4 are own flat blue squares; XP has no such controls, but they sit in the taskbar centre-right and read as foreign; clip and speaker glyphs are own thin white outlines while XP tray icons are 16 px colour icons | use Luna 16 px tray-style icons for clip and speaker, dim the workspace buttons to the tray tone |
| A8 | MINOR | turn-off | whole dialog | own power-button art and a ring logo instead of the XP flag logo (not importable from the sparse set: acceptable), desktop not dimmed behind (XP greys it), Cancel 76 wide vs 73 | dim backdrop |
| A9 | COSMETIC | menu | user name "User" | white 14 px with shadow; XP uses Tahoma bold 13 | |
| A10 | COSMETIC | all | Bliss | wallpaper is upscaled from the import's jpg (visibly soft, no film grain on the hill) | note only; use the largest wallpaper in the set |
| A11 | COSMETIC | help | window | no title icon, only a close box (as r2 N11) | |

Worker's notes, judged: Vim/LibreOffice non-Luna -- acceptable (apps without a Luna equivalent), A5; ImageMagick generic -- the import gives a yellow folder with a window: it reads as a generic, acceptable;
user tile, chat ball, workspace buttons, tray clip and speaker own art -- tile A1 MAJOR (size and style), the rest MINOR A7; caption font not Trebuchet -- confirmed, A2 MAJOR.

## Counts

- Fallback: MAJOR 0, MINOR 2 (F1, F2), COSMETIC 5 (F3-F7) plus the open r3 N4 (MINOR) and N6-N11 carried. Verdict MERGE-OK.
- Assets: MAJOR 3 (A1-A3), MINOR 5 (A4-A8), COSMETIC 3 (A9-A11). Verdict FIX.
- r3: N1, N2, N3, N5 FIXED; N4 OPEN; no REGRESSED.

Top 3 (assets): A1 header 65 vs 54 and the user tile, A2 title face is not Trebuchet Bold 13, A3 Run dialog is fuzzel.
