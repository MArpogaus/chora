# c-novice (construct)
## Goals
- What is this screen and how do I get around: partly. F1 opened a keys panel on the empty desktop (shots/005.png) and Super+Space opened an app launcher (shots/008.png). Tries ~8. Nothing on the desktop tells a newcomer where to start; no dock, taskbar or icons. First moment of not knowing: the blank marble floor at start (shots/001.png). Super key alone did nothing visible.
- Open a program, look up close, step back: yes, partly. Foot terminal opened (shots/015.png); Alt+Escape stepped back to a 3D view with the window labelled "foot" (shots/016.png). Tries ~8. LibreOffice Writer did not open (see NC-3).
- Find a list of all keyboard shortcuts: yes. F1 on the empty desktop opened "Show these keys" (shots/005.png). Tries 2. Could not reopen it once a terminal had focus (NC-7).
- Two windows side by side: no. Tries ~15. Tiling and xp layout keys showed "did not take the window" toasts (shots/020.png, shots/024.png). Plain drag did not move a pane (shots/021.png, shots/032.png). Super+drag, listed as "Move or resize a pane", could not be tested because the harness cannot hold Super during a drag.
- Note on harness: "Super" and "Meta" were rejected as key names; Super+space and Super_L worked. Lena would have tried the Windows key first.

## Findings
| id | sev | kind | where | what I did | expected | saw | shot |
|---|---|---|---|---|---|---|---|
| NC-1 | major | ux | empty desktop | looked at start screen, pressed Super_L | a visible way to start programs | blank marble floor, no dock, taskbar or icons; launcher only via Super+space | shots/001.png |
| NC-2 | cosmetic | visual | launcher list | Super+space | clear app names and icons | ImageMagick label cut to "ImageMagick (color depth=..."; UXTerm, XTerm, ImageMagick have no icon while others do | shots/008.png |
| NC-3 | major | bug | launcher to window | typed writer, Return on LibreOffice Writer | Writer window opens | launcher closed, no window after about 25 s; Super+Tab and Tab both showed "0 WINDOWS" | shots/011.png, shots/012.png, shots/014.png |
| NC-4 | minor | bug | launcher | clicked LibreOffice Writer entry with mouse | entry launches | nothing happened; keyboard path worked | shots/009.png |
| NC-5 | minor | visual | first terminal | Super+space, foot, Return | clean shell | yellow warning line at top: "'C' is not a UTF-8 locale, using 'C.UTF-8' instead"; text very small at 1280x800 | shots/015.png |
| NC-6 | minor | ux | window layout | Super+Alt+1 with a pane focused | pane goes to tiling | toast "tiling did not take the window." with no change; Super+Alt+2 gave "xp did not take the window." | shots/020.png, shots/024.png, shots/033.png |
| NC-7 | major | ux | keyboard focus in terminal | pressed F1, Escape, Alt+Escape while foot had focus | F1 keys panel opens; Escape or Alt+Escape gives back control | F1 and Escape went to the shell; Alt+Escape changed the view but focus stayed in the terminal; keys panel did not open | shots/029.png, shots/030.png |
| NC-8 | major | ux | walking | pressed d three times, then again after Alt+Escape | view walks right (keys panel lists W A S D as working) | no movement; letters went into the terminal | shots/027.png, shots/028.png |
| NC-9 | major | ux | pane arrangement | dragged pane plain, then dragged focused pane toward centre | pane moves; side-by-side possible | pane did not move; drag only changed which pane had focus | shots/021.png, shots/032.png |
| NC-10 | minor | visual | second terminal | launched second foot, stepped back | second window readable and labelled | it filled the view, then appeared as a flat grey slab with no label and unreadable text; no visible text or title | shots/017.png, shots/018.png, shots/019.png |
| NC-11 | minor | ux | cube | Super+c, listed as "Open the cube" | cube view appears | no visible change | shots/023.png |
| NC-12 | cosmetic | visual | bottom hint bar | stepped back to 3D view | bar stays readable | bar with W A S D, Tab, F1, Super C hints shows, then auto-hides; a grey block sits under it at the bottom edge | shots/016.png, shots/019.png |
| NC-13 | cosmetic | visual | focus change | clicked the grey pane | clear focus cue | focused pane gets a thin blue edge, the other goes dark grey; hard to tell which is active at a glance | shots/022.png, shots/031.png |
| NC-14 | cosmetic | visual | empty desktop | pressed F1 on empty desktop | keys panel readable | keys panel is small and light grey on white, but readable; "Only the keys that work now are listed" line is tiny | shots/005.png |
