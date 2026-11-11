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
status: rejected: env, no audio device and no wpctl in the container; the tray and the Sound submenu both say Sound unavailable (shot 004)

## Terminal window opens late and hidden behind Start menu
kind: issue
persona: Gerda, 71, retired, tech affinity 1, mouse only, knows Windows XP from years ago.
flavor: xp
severity: medium
steps: Clicked Terminal in Start menu, looked (no window), reopened Start, looked again.
expected: Program window appears right after clicking its name.
actual: No window on first look; on second, a black "foot" window with a C-locale warning and root prompt sat behind the Start menu.
evidence: shots/011.png
status: rejected: harness timing, shot 010 was taken 0.07 s after the click and the menu was closed; in shot 011 the tester reopened Start, which draws over the window by design

## Start menu lists only one program
kind: ux
persona: Gerda, 71, retired, tech affinity 1, mouse only, knows Windows XP from years ago.
flavor: xp
severity: medium
steps: Opened Start menu to find a program.
expected: Familiar programs (Internet, E-mail, Paint, Notepad) in the left column.
actual: Left column shows only "Terminal foot"; the right column has folders, My Computer, Control Panel, Run.
evidence: shots/006.png
status: rejected: env, the container offers one native desktop entry (foot); Internet, E-mail, Paint and Notepad do not exist on this system (shot 006)

## Control Panel submenu opens only from its arrow
kind: ux
persona: Gerda, 71, retired, tech affinity 1, mouse only, knows Windows XP from years ago.
flavor: xp
severity: low
steps: Clicked the Control Panel label in the Start menu, then its arrow.
expected: The submenu opens on the label.
actual: The label only highlights (shot 007); the submenu opens after a click on the arrow (shot 008).
evidence: media/2026-11-12/swarm-3/x-everyday/shots/007.png, media/2026-11-12/swarm-3/x-everyday/shots/008.png
status: verified
