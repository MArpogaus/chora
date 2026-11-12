# t5-mouse (tiling), simulated user
goal: reached no, tries 5 (start any program with the mouse: Apps menu items did not respond to clicks, double-click or hover+click)
goal: reached no, tries 0 (start a second program: blocked by goal 1)
goal: reached no, tries 0 (close one program with the mouse: no program ever opened)
goal: reached partly, tries 2 (help: found a "Keys" cheat sheet, but it lists keyboard shortcuts only, no mouse guidance)
goal: reached yes, tries 3 (power/lock: found under the Keys panel's power icon, which opens Lock, Log out, Restart, Shut down)

## Apps menu items ignore mouse clicks
kind: issue
persona: Anna, 45, teacher, tech affinity 1, mouse only, has never used a tiling desktop.
flavor: tiling
severity: high
steps: Clicked the "Apps" button top left; the list opened. Clicked "LibreOffice Writer", then hovered and clicked it again. Clicked "Foot", double-clicked "Foot Client", clicked "Screenshot". Took a screenshot after each.
expected: The clicked program starts and the menu closes.
actual: The menu stayed open with nothing launched, every time. No program started in any attempt.
evidence: shots/003.png
status: new

## Clicking the top-bar Keys button opens a full-screen overlay that is hard to dismiss
kind: ux
persona: Anna, 45, teacher, tech affinity 1, mouse only, has never used a tiling desktop.
flavor: tiling
severity: medium
steps: Clicked the "Keys" button top right. Then clicked the power icon at the top right. Then pressed Escape, then clicked the Apps button.
expected: A help panel that closes with Escape or a click, and the other buttons still work.
actual: The Keys panel covered the screen and stayed up. The power click was swallowed by it. After Escape, the Apps button click did not open the menu. A click on empty space behind the panel was needed before the Apps button responded.
evidence: shots/007.png
status: new

## Keys panel is a keyboard shortcut list with no mouse help
kind: ux
persona: Anna, 45, teacher, tech affinity 1, mouse only, has never used a tiling desktop.
flavor: tiling
severity: medium
steps: Opened the Keys panel from the top right and read it.
expected: Plain instructions for mouse users, such as how to open a program and close a window.
actual: A dense grid of key combinations (Super+j, Super+Shift+Enter and so on) with terms like "Tags", "Layouts", "Float window" and "Tile". Nothing says how to do the basics with a mouse. The text is small at this size.
evidence: shots/010.png
status: new

## Power menu is found but Lock is pre-selected
kind: ux
persona: Anna, 45, teacher, tech affinity 1, mouse only, has never used a tiling desktop.
flavor: tiling
severity: low
steps: Opened the Keys panel, then clicked the power icon at the top right.
expected: A clear power menu, with the main actions named plainly.
actual: The menu lists Lock, Log out, Restart and Shut down. Lock is highlighted by default, so pressing Enter by accident would lock the screen. It works and is easy to find once the power icon is clicked.
evidence: shots/009.png
status: new

## Top bar elements are unlabelled and small
kind: ux
persona: Anna, 45, teacher, tech affinity 1, mouse only, has never used a tiling desktop.
flavor: tiling
severity: low
steps: Looked at the top bar on first start.
expected: Clear labels for the workspace numbers and the "Tile" field.
actual: Workspace numbers 1 to 9 sit next to an empty "Tile" box with no label or hint. "No sound" and "Connected" look like status text but may be buttons. All text is very small at this size.
evidence: shots/001.png
status: new

## Apps menu gives no hint that it filters by typing
kind: ux
persona: Anna, 45, teacher, tech affinity 1, mouse only, has never used a tiling desktop.
flavor: tiling
severity: low
steps: Opened the Apps menu and looked at it.
expected: A search box that reads as a search, or a plain list of programs.
actual: The menu shows a title "Apps" with a text cursor and a list. It is not clear whether the list can be typed into or clicked, and clicking the entries did nothing.
evidence: shots/002.png
status: new
