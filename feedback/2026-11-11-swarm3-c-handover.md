# c-handover (construct), simulated user
goal: (1) two terminals side by side: reached no, tries 6
goal: (2) find cube and turn to another face: reached partly, tries 4
goal: (3) move a terminal to another face and type echo hi: reached no, tries 0
goal: (4) get back to start: reached partly, tries 2

## No visible way to open a terminal from the empty desktop
kind: ux
persona: Kai, 30, ricer, tech affinity 4, keyboard and mouse, wants to show off the desktop
flavor: construct
severity: high
steps: Tried ctrl+alt+t, super+Return, right-click. Found Super T only via F1.
expected: A visible launcher or menu for starting a terminal.
actual: Empty marble floor, no panel. Right-click did nothing. Hint bar vanished after one key.
evidence: shots/001.png
status: verified

## Second Super+T gave no visible second terminal
kind: issue
persona: Kai, 30, ricer, tech affinity 4, keyboard and mouse, wants to show off the desktop
flavor: construct
severity: high
steps: Pressed Super+T twice, then Alt+Esc. Tab (show windows) also did nothing.
expected: Two terminal panes side by side.
actual: One fullscreen terminal, then one pane on the cube. No second pane in any shot.
evidence: shots/006.png
status: duplicate of Construct: after a window opens, walk and help keys do nothing or type into the terminal

## Cube hard to read; pane label floats over empty space
kind: ux
persona: Kai, 30, ricer, tech affinity 4, keyboard and mouse, wants to show off the desktop
flavor: construct
severity: medium
steps: Super+C, then E to turn, then Alt+Esc. D to walk dropped me into a terminal.
expected: A clear cube with the terminal on a face.
actual: Super+C gave a walk view, not a cube. E turned to an XP-style face with taskbar; a "foot" label floated with no window drawn.
evidence: shots/013.png
status: duplicate of A closed window leaves its foot pill on the cube face

## D in walk view drops into a fullscreen terminal
kind: issue
persona: Kai, 30, ricer, tech affinity 4, keyboard and mouse, wants to show off the desktop
flavor: construct
severity: medium
steps: Super+T, Alt+Esc, Super+T, then D to walk.
expected: Walking sideways.
actual: The D went into the fullscreen terminal and appeared at the prompt; the Step back hint was the only sign.
evidence: media/2026-11-12/swarm-3/c-handover/shots/011.png
status: verified
