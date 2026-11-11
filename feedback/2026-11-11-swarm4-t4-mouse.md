# t4-mouse (tiling), simulated user
goal: reached no, tries 11 (clicks on top-left icon, right-click, double-click, hover+click, Connected button, clock)
goal: reached no, tries 0 (no program could be started, so nothing to resize)
goal: reached no, tries 0 (no program was open to close)
goal: reached no, tries 0 (no help or menu was visible to try)

## Top-left icon does not respond to clicks
kind: issue
persona: Anna, 45, teacher, tech affinity 1, mouse only, has never used a tiling desktop.
flavor: tiling
severity: high
steps: Clicked the only small icon at the top-left of the top bar: single click, double click, hover then click, 4 tries.
expected: A menu or app launcher opens so I can start a program.
actual: Nothing opened. The icon is only a few pixels wide and has no label or tooltip.
evidence: shots/002.png
status: duplicate of Tiling bar: Connected and workspace 1 are unlabelled and inert; the 1 vanishes (the "icon" at top-left is the workspace tag label, flavors/tiling/waybar/config.jsonc:11 custom/tags, which has no on-click; shot 002 read)

## Right-click on the empty desktop shows no menu
kind: issue
persona: Anna, 45, teacher, tech affinity 1, mouse only, has never used a tiling desktop.
flavor: tiling
severity: high
steps: Right-clicked in the middle of the empty desktop.
expected: A context menu with a way to start programs or get help.
actual: No menu appeared. Only the mouse pointer (a small yellow square) is visible.
evidence: shots/003.png
status: duplicate of No visible way to open a terminal from the empty desktop (tiling has no root menu or launcher in the bar, config.jsonc:10-13; shot 003 not needed, same as 001)

## Empty desktop gives no visible way to start anything
kind: ux
persona: Anna, 45, teacher, tech affinity 1, mouse only, has never used a tiling desktop.
flavor: tiling
severity: high
steps: Looked at the screen after startup and after each click attempt.
expected: Some visible icon or hint that says how to start a program.
actual: Blank dark grey desktop with a thin top bar. No icons, windows, dock or text hint.
evidence: shots/001.png
status: duplicate of No visible way to open a terminal from the empty desktop (shot 001: blank desktop and thin bar, read)

## Top-bar status items look clickable but do nothing
kind: ux
persona: Anna, 45, teacher, tech affinity 1, mouse only, has never used a tiling desktop.
flavor: tiling
severity: medium
steps: Clicked the blue "Connected" button at the top right, then the clock in the centre of the top bar.
expected: A status panel or calendar opens, or a hint explains the item.
actual: No change on screen. "Sound unavailable" and "Connected" are shown with no explanation.
evidence: shots/006.png
status: duplicate of Tiling bar: Connected and workspace 1 are unlabelled and inert; the 1 vanishes (shot 006 read; clock calendar is a hover tooltip, config.jsonc:21, and Connected is a network module without on-click)

## No help found anywhere on the desktop
kind: ux
persona: Anna, 45, teacher, tech affinity 1, mouse only, has never used a tiling desktop.
flavor: tiling
severity: high
steps: Looked for help in the top bar, the top-left icon, the desktop and the right-click menu, and tried the status button.
expected: A help entry, a guide link, or a "how do I start a program" hint.
actual: No help was visible in any place I tried. I could not reach help with the mouse alone.
evidence: shots/007.png
status: duplicate of Key list and help exist only in construct

ux: none further seen, because the desktop showed no windows, menus or help to read or reach.
