# Review: xp v2, round 2 (confirm M1 to M4)

Reviewer: opus, read-only. Branch claude/xp-v2 (0d92149, a463f8c, 72b0c2e, f84bb33).
Captures: media/2026-11-12/xp-v2-r3/ (1280x800, scale 1). M4 by diff (no picture possible).

## M1 Windows flag: CONFIRMED FIXED
- start-menu.png: Start button (0-118 x 766-800) shows a white ring-and-dot mark at about 15-32 x 774-792 and the word "start", one colour, no four-colour flag.
- Start-menu user tile (8-65 x 368-420): a blue user head-and-shoulders glyph on a pale tile.
- turn-off.png: dialog title mark at 757-786 x 310-338 is a white ring with a dot, no flag.
- a463f8c also deletes flavors/xp/theme/winxp/start-flag.svg.

## M2 Help table cut off: CONFIRMED FIXED
- help-keys-end.png: window 249-1031 x 33-731 grew to fit; the last row "Super+Alt+1 ... 4" is whole at y 681-711 with its closing rule at y 711, about 20 px above the frame bottom (731).
- All rows from "Super / Ctrl+Escape" (y 212) to the last are fully on screen, none half cut; Ctrl+Alt+Shift+Right (y 664) is complete.
- Not proven: behaviour on a smaller screen (72b0c2e says rows shrink); not in the captures.

## M3 Run dialog unthemed: CONFIRMED FIXED (one leftover, not a blocker)
- run.png: box 405-875 x 240-560, opaque #ECE9D8 (wallpaper no longer shows through), 2 px blue #0054E3 edge, selection bar 413-867 x 287-316 in #316AC5 with white text, sans (Tahoma/DejaVu) text, prompt "Open:".
- "ImageMagick (color depth=q16)" is shown in full (y 388).
- Leftover: first row still reads "Foot" (not "Terminal"), from the desktop entry name; cosmetic, not raised as a defect.

## M4 False launch balloons: CONFIRMED FIXED (by diff, f84bb33 is the theming; the tracker fix is 0d92149)
- launch.rs: DETACHED = ["fuzzel", "chora-chat"] are never tracked (`add` returns early), so no "Starting" and no "Cannot open" for the launcher or the chat.
- At GIVE_UP (30 s), a pid still alive gives Change::Done, which closes the Starting note (wm.rs `notifier.close`) and shows nothing; Failed only when the process is gone and no new window mapped.
- Three new tests cover all three cases (detached, alive at limit, ended without window).
- Residual: a slow live program still shows "Starting" for up to 30 s before it clears (not a false error).

## New main-path flaws visible in captures
None.

## Verdict: PASS
