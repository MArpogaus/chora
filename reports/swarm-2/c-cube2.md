# c-cube2 (construct)
## Goals
- Get to the cube: yes, tries 2, steps: F1 (key list shows "Open the cube: Super+C"), Escape, Super+C. Opening the cube was not animated; the view was already on face A when I looked.
- Turn to every side and say what each side shows: partly, tries 5, steps: Ctrl+Alt+Right four times, then once more to check.
  - Face A (start): XP-style bliss wallpaper with icons My Documents, My Computer, Recycle Bin.
  - Face B: same XP wallpaper, NO icons, no windows, no label.
  - Face C and face D: looked identical to B (checked after 5 s and again after 10 s).
- Step into one side, open a program, come back out: partly, tries 6, steps: Return did nothing (x2, shots 025-030); Super+Space on face B opened the launcher (shot 031); typed Foot, Return; Foot terminal opened full-screen (shot 033); Alt+Escape stepped out to the face, terminal became a "foot" pill (shot 034).
- Go back to where I started: partly, tries 4, steps: H (Back to the start, listed in F1) did nothing, twice (shots 036, 040). Ctrl+Alt+Left once from face B returned to face A with icons (shot 043). Confirmed H again on face A: no change (shot 044).

## Findings
| id | sev | kind | where | what I did | expected | saw | shot |
|---|---|---|---|---|---|---|---|
| c-cube2-1 | major | bug | face B, cube mode | Pressed Return ("Step into the face" in F1 list), 2 tries, 5 s wait each | Camera moves into the face | No change at all; still on the flat face | shots/028.png |
| c-cube2-2 | major | bug | cube mode, after Foot pane | Pressed H ("Back to the start" in F1 list), 3 tries | Return to the starting face/position | No change; stayed on face B, then on face A | shots/036.png |
| c-cube2-3 | major | bug | faces B, C, D | Turned to each side, watched 5-10 s | Each side shows a different desktop (as the brief says) | B, C, D look identical: bare wallpaper, no icons, no windows | shots/011.png |
| c-cube2-4 | minor | ux | cube mode, Q/E and Right arrow | Pressed E, then Right arrow on the first face, before cube mode was confirmed | Turning keys work | No change in either case. Once cube mode was on, F1 still listed Q/E/arrows as working under WALK, but they did nothing; only Ctrl+Alt+arrows turn the cube | shots/005.png, shots/007.png, shots/009.png |
| c-cube2-5 | minor | ux | after closing Foot with Alt+Escape | Went back to face A and watched the centre of the screen | The terminal is gone from face B, or only shown there | A grey "foot" pill is still in the centre of face A, and it stays there after turning back (and on every turn after) | shots/034.png, shots/041.png, shots/044.png |
| c-cube2-6 | minor | visual | every cube turn | Watched frames at 0.3 s and 0.8 s during Ctrl+Alt+Right | Clean rotation | Small yellow square at the top-left corner of the screen in most turn frames, plus white strips around the cube edges and a tilted, skewed taskbar | shots/010.png, shots/014.png, shots/017.png |
| c-cube2-7 | minor | visual | face A, Recycle Bin icon label | Looked 5 s and 10 s after turns | Label appears with its icon | "Recycle Bin" text is faint and only partly drawn during the turn and still faint after 5 s; it pops in late | shots/022.png, shots/043.png |
| c-cube2-8 | minor | visual | taskbar tray, all faces | Watched the taskbar before and after the first turn | Stable tray | "Sound unavailable" tooltip is absent in the first shot, appears about 5 s later, and stays on every face | shots/003.png, shots/004.png |
| c-cube2-9 | cosmetic | visual | bottom-right, all faces | Looked at every face | Hint pill is not in the way | "Step back Alt+Esc" hint pill is always on screen and covers the Recycle Bin area | shots/004.png |
| c-cube2-10 | cosmetic | ux | start view before cube | First screen, before Super+C | Clear sign that the view is a 3D cube | Blank marble void with no hint; the cube is only found via F1 | shots/001.png |

## Notes
- Action count about 46 of 90. Session closed with stop.
- Faces B, C and D could not be told apart. Possibly only face A has content, or the other faces failed to load. Not checked further.
