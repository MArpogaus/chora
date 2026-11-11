# x-everyday (xp), simulated user
goal: find time and date, reached yes, tries 2
goal: change volume, reached no, tries 6
goal: open Start program, enlarge window, close it, reached yes, tries 9
goal: turn computer off, reached no, tries 0 (step cap hit)

## Volume cannot be changed
kind: issue
persona: Gerda, 71, retired, tech affinity 1, mouse only, knows Windows XP from years ago.
flavor: xp
severity: high
steps: Clicked and double-clicked the tray speaker; opened Start, Control Panel, Sound.
expected: A volume slider or sound settings I can move with the mouse.
actual: Tray says "Sound unavailable"; speaker click does nothing; Sound submenu says "Sound unavailable" and a line like "wjcd1 not found".
evidence: shots/004.png
status: new

## Terminal window opens late and hidden behind Start menu
kind: issue
persona: Gerda, 71, retired, tech affinity 1, mouse only, knows Windows XP from years ago.
flavor: xp
severity: medium
steps: Clicked Terminal in Start menu, looked (no window), reopened Start, looked again.
expected: Program window appears right after clicking its name.
actual: No window on first look; on second, a black "foot" window with a C-locale warning and root prompt sat behind the Start menu.
evidence: shots/011.png
status: new

## Start menu lists only one program
kind: ux
persona: Gerda, 71, retired, tech affinity 1, mouse only, knows Windows XP from years ago.
flavor: xp
severity: medium
steps: Opened Start menu to find a program.
expected: Familiar programs (Internet, E-mail, Paint, Notepad) in the left column.
actual: Left column shows only "Terminal foot"; the right column has folders, My Computer, Control Panel, Run.
evidence: shots/006.png
status: new
