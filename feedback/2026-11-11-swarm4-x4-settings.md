# x4-settings (xp), simulated user
goal: 1 change desktop background or colours: reached partly, tries 2
goal: 2 find screen or display settings: reached yes, tries 2
goal: 3 open two programs and switch with taskbar: reached no, tries 4
goal: 4 find turn off and back out without doing it: reached partly, tries 1
## Desktop folder icons fail with "Cannot open thunar"
kind: issue
persona: Murat, 52, shop owner, tech affinity 1, mouse only (no shortcuts)
flavor: xp
severity: high
steps: Double-clicked My Computer, then My Documents on the desktop.
expected: A folder window opens for each icon.
actual: Balloon "Cannot open thunar / The program is not installed." No window opened.
evidence: shots/008.png
status: new

## Terminal and Control Panel from Start menu do not open
kind: issue
persona: Murat, 52, shop owner, tech affinity 1, mouse only (no shortcuts)
flavor: xp
severity: high
steps: Start, Control Panel. Start, Terminal. Screenshot.
expected: Both windows open, each with a taskbar button.
actual: Nothing opened. Taskbar empty, desktop bare green.
evidence: shots/012.png
status: new

## Turn Off Computer click gave no dialog
kind: issue
persona: Murat, 52, shop owner, tech affinity 1, mouse only (no shortcuts)
flavor: xp
severity: medium
steps: Start, clicked Turn Off Computer.
expected: A Turn Off Computer dialog with Cancel.
actual: Menu closed, no dialog. A small yellow square appeared near bottom centre; its meaning is unclear.
evidence: shots/011.png
status: new

## Background picture options greyed out until a picture is chosen
kind: ux
persona: Murat, 52, shop owner, tech affinity 1, mouse only (no shortcuts)
flavor: xp
severity: low
steps: Right-click desktop, Properties, Desktop tab, selected (None), picked Green colour, Apply.
expected: Clear way to change picture and colour.
actual: Colour change worked. Browse and Position stayed greyed out, and the Desktop tab is small and easy to miss.
evidence: shots/006.png
status: new

## Screen resolution found under Display Properties Settings tab
kind: ux
persona: Murat, 52, shop owner, tech affinity 1, mouse only (no shortcuts)
flavor: xp
severity: low
steps: Right-click desktop, Properties, Settings tab.
expected: Screen settings with resolution visible.
actual: Found it: slider and 1280 by 800 pixels. Labels are small.
evidence: shots/007.png
status: new

## Start menu and taskbar text very small
kind: ux
persona: Murat, 52, shop owner, tech affinity 1, mouse only (no shortcuts)
flavor: xp
severity: low
steps: Opened Start and looked at the menu and taskbar.
expected: Readable labels at normal size.
actual: Start menu text and Log Off / Turn Off Computer labels are hard to read. Terminal shows a second line "root" with no explanation.
evidence: shots/010.png
status: new
