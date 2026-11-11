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
status: rejected: env, thunar is not installed here (same as swarm2 Thunar is not installed); balloon in shot 008 is the documented answer

## Terminal and Control Panel from Start menu do not open
kind: issue
persona: Murat, 52, shop owner, tech affinity 1, mouse only (no shortcuts)
flavor: xp
severity: high
steps: Start, Control Panel. Start, Terminal. Screenshot.
expected: Both windows open, each with a taskbar button.
actual: Nothing opened. Taskbar empty, desktop bare green.
evidence: shots/012.png
status: rejected: tester error plus known duplicate. actions.jsonl: Start, click (128,273) on the Control Panel label (opens only from its arrow, duplicate of Control Panel submenu opens only from its arrow), then Start again at (24,392) which toggles the menu shut, then (60,208) hit the bare desktop. Terminal entry exists at plugins/chora-xp/src/startmenu.rs:834 and runs via Action::Run (startmenu.rs:847, wm.rs:2674); shot 012 shows no menu open

## Turn Off Computer click gave no dialog
kind: issue
persona: Murat, 52, shop owner, tech affinity 1, mouse only (no shortcuts)
flavor: xp
severity: medium
steps: Start, clicked Turn Off Computer.
expected: A Turn Off Computer dialog with Cancel.
actual: Menu closed, no dialog. A small yellow square appeared near bottom centre; its meaning is unclear.
evidence: shots/011.png
status: rejected: tester clicked (270,375), right of the 190 px wide menu, so the click closed it (shot 010 shows the buttons at x 90 and 127; yellow square is the harness pointer). Turn Off Computer opens the dialog: startmenu.rs:972 Action::TurnOff, wm.rs:2679 open_dialog

## Background picture options greyed out until a picture is chosen
kind: ux
persona: Murat, 52, shop owner, tech affinity 1, mouse only (no shortcuts)
flavor: xp
severity: low
steps: Right-click desktop, Properties, Desktop tab, selected (None), picked Green colour, Apply.
expected: Clear way to change picture and colour.
actual: Colour change worked. Browse and Position stayed greyed out, and the Desktop tab is small and easy to miss.
evidence: shots/006.png
status: rejected: by design, Browse is a disabled stub (plugins/chora-xp/src/display.rs:12 and :302); colour change worked (shot 006 read)

## Screen resolution found under Display Properties Settings tab
kind: ux
persona: Murat, 52, shop owner, tech affinity 1, mouse only (no shortcuts)
flavor: xp
severity: low
steps: Right-click desktop, Properties, Settings tab.
expected: Screen settings with resolution visible.
actual: Found it: slider and 1280 by 800 pixels. Labels are small.
evidence: shots/007.png
status: rejected: no defect, the tester found it at once (shot 007 read: slider and 1280 by 800); only the small labels remain, see Small, light-grey text

## Start menu and taskbar text very small
kind: ux
persona: Murat, 52, shop owner, tech affinity 1, mouse only (no shortcuts)
flavor: xp
severity: low
steps: Opened Start and looked at the menu and taskbar.
expected: Readable labels at normal size.
actual: Start menu text and Log Off / Turn Off Computer labels are hard to read. Terminal shows a second line "root" with no explanation.
evidence: shots/010.png
status: duplicate of Small, light-grey text on the key sheet, tray, mc header and desktop labels (shot 010 read)
