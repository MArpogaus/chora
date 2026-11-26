# p2r14co Explorer (construct), round 14 on 4d8bbe36, one run

19 actions, 11 looks (caps C1 12/6, T1 14/6, T2 19/8; used 9/5, 5/3, 5/3; none breached). Contracts C1, T1, T2 pass. Persona: C1, T1, T2 reached.
Captures: media/2026-11-26/persona-r14/captures/explorer/; contract lines media/2026-11-26/persona-r14/ends/explorer-*.txt.

## xp face contrast rows (I6, known)
kind: issue
persona: Explorer (affinity 1, mouse first)
flavor: construct
severity: low
task: none
check: I6
steps: all actions
expected: no contrast breach
actual: the known xp tray clock family
evidence: media/2026-11-26/persona-r14/captures/explorer/breaches.jsonl
status: new

## Cube stood back: I2 gaps rows (co-44, known)
kind: issue
persona: Explorer (affinity 1, mouse first)
flavor: construct
severity: low
task: none
check: I2
steps: cube stood back
expected: mode chips apart enough
actual: I2 'gaps: construct-mode-step-in and construct-mode-turn-left/right' (co-44, open)
evidence: media/2026-11-26/persona-r14/captures/explorer/breaches.jsonl
status: new

## C1: the first click on the cube row only highlights; Super+C opens it
kind: ux
persona: Explorer (affinity 1, mouse first)
flavor: construct
severity: low
task: C1
check: none
steps: looked, clicked the row A cube of desktops, looked (no cube, background plain), Super+c, Return, Alt+Escape
expected: a click on the row opens the cube
actual: the click highlighted the row; the cube opened only after Super+c, which the row names
evidence: media/2026-11-26/persona-r14/captures/explorer/shots/ (C1)
status: new

## C1: cube, step in, step out reached
kind: ux
persona: Explorer (affinity 1, mouse first)
flavor: construct
severity: low
task: C1
check: none
steps: Super+c, Return, Alt+Escape from the hint bar
expected: -
actual: reached in 9 actions
evidence: media/2026-11-26/persona-r14/ends/explorer-C1.txt
status: new

## T1: terminal is called Foot in Programs, no plain Terminal entry
kind: ux
persona: Explorer (affinity 1, mouse first)
flavor: construct
severity: low
task: T1
check: none
steps: click Programs in the top bar, click Foot
expected: an entry named Terminal
actual: reached; the name Foot is recognised only by its icon; the shell is root with a C locale warning (harness)
evidence: media/2026-11-26/persona-r14/captures/explorer/shots/008.png
status: new

## T2: closing the key list by its Close button left the view on the cube, not the terminal
kind: ux
persona: Explorer (affinity 1, mouse first)
flavor: construct
severity: medium
task: T2
check: none
steps: Super+F1 (told by the hint bar), read, click Close
expected: back in the terminal the person had
actual: list closed; the view then showed the cube overview (faces XP 1, Foot), not the terminal
evidence: media/2026-11-26/persona-r14/captures/explorer/shots/011.png
status: new
