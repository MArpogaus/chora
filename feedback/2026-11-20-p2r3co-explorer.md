# p2r3co Explorer (construct), round 3 on bbaad12b

Run: session construct 1280x800, 9 actions, 5 looks, haiku persona, PERSONA_CHECK=1, helper transcript seeded, begin once. Contracts (check.py end C1 T1 T2): all pass. Breaches: I7 x4 (first click, all four outputs).
Persona says: goals 1 (cube in and out, caveat), 2 (terminal), 3 (key list) reached.
Captures: media/2026-11-20/persona-r3-explorer/.

## I7 on all four outputs at the first click
kind: issue
persona: Explorer, affinity 1, mouse first
flavor: construct
severity: low
task: C1
check: I7
steps: first click on the row "A cube of desktops"
expected: the work areas draw
actual: I7 "the work area of VIRTUAL-1..4 shows one colour" at the first click only (known: faces not drawn at the first quiet snapshot, harness, r2)
evidence: media/2026-11-20/persona-r3-explorer/breaches.jsonl, evidence/
status: new

## The row "A cube of desktops" steps into face 1; the cube shows only after Alt+Escape
kind: ux
persona: Explorer, affinity 1, mouse first
flavor: construct
severity: low
task: C1
check: none
steps: click the row, look, Alt+Escape, look
expected: a cube to look at
actual: a flat XP desktop with the hint "Alt Esc Step back"; the cube (XP 1, face 1 of 4) came after Alt+Escape (designed step-in, co-34)
evidence: shots 002, 003, 004
status: new

## The top bar is gone once a window is open
kind: ux
persona: Explorer, affinity 1, mouse first
flavor: construct
severity: low
task: T2
check: none
steps: Super+t, then looked for Keys/Programs
expected: the bar to click Keys
actual: the top bar is hidden with a terminal open; the persona used Super+F1 (sheet "Construct · Cube", sections Walk, Panes, Cube, Helper and Chat, Give a window, Everywhere)
evidence: shots 004, 005
status: new

## The key sheet has no close button in the desktop view
kind: ux
persona: Explorer, affinity 1, mouse first
flavor: construct
severity: low
task: T2
check: none
steps: read the sheet
expected: a way to close it by mouse
actual: the footer offers only "Esc Close" and "F1 Show these keys, stood back"
evidence: shot 005
status: new
