# c4-mouse (construct), simulated user
goal: open a program with the mouse only: reached no, tries 6
goal: move or turn the 3D view with the mouse: reached no, tries 1
goal: drag a window to a new place: reached no, tries 0
goal: close the window: reached no, tries 0

## Desktop shows nothing to open
kind: ux
persona: Ben, 13, pupil, tech affinity 3, mouse first, plays 3D games
flavor: construct
severity: high
steps: Started the session and looked at the screen. Right-clicked the middle of the desktop, moved to the bottom edge, double-clicked the middle, right-clicked the top-left corner, and scrolled the wheel on the desktop.
expected: Some icon, dock, taskbar or menu that opens a program, or a context menu after a right-click.
actual: The screen is an empty grey-white marble background. No icons, no taskbar, no dock, no menu. Nothing changed after any of the six tries. The only visible thing is a small yellow dot for the pointer.
evidence: shots/001.png
status: new

## Right-click on the desktop gives no menu
kind: issue
persona: Ben, 13, pupil, tech affinity 3, mouse first, plays 3D games
flavor: construct
severity: medium
steps: Right-clicked at the middle of the desktop, then took a screenshot.
expected: A context menu with a way to open something.
actual: Nothing appeared. The screen looked the same as before.
evidence: shots/002.png
status: new

## Double-click and scroll on the desktop do nothing
kind: issue
persona: Ben, 13, pupil, tech affinity 3, mouse first, plays 3D games
flavor: construct
severity: low
steps: Moved the pointer to the bottom edge, then double-clicked the middle of the desktop, then scrolled the wheel.
expected: Some visible response, such as a launcher, a panel sliding in, or the view moving.
actual: No visible change in any screenshot. The pointer moved but nothing else did.
evidence: shots/003.png
status: new

## Pointer is a small yellow dot that is easy to lose
kind: ux
persona: Ben, 13, pupil, tech affinity 3, mouse first, plays 3D games
flavor: construct
severity: low
steps: Moved and clicked the pointer around the screen, checking where it was in each screenshot.
expected: A pointer that is easy to see and follow.
actual: The pointer is a tiny yellow square. It was hard to find, and when it sat at the bottom edge it was not visible at all.
evidence: shots/003.png
status: new

## Nothing to test for window goals
kind: ux
persona: Ben, 13, pupil, tech affinity 3, mouse first, plays 3D games
flavor: construct
severity: high
steps: Could not open any program, so no window or 3D view appeared to move, drag or close.
expected: A way to reach a window from the desktop, so that drag and close could be tried.
actual: Goals 2 to 4 could not be tried at all, because no window or 3D view was ever on screen.
evidence: shots/004.png
status: new
