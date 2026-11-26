# p2r15co Explorer (construct), round 15 on 57ce1480, one run

17 actions, 10 looks (caps C1 7/4, T1 5/3, T2 5/3 used; none breached). Contracts C1, T1, T2 pass. Persona: C1 reached, T1 reached, T2 reached.
Captures: media/2026-11-26/persona-r15/captures/explorer; contract lines media/2026-11-26/persona-r15/ends/explorer-*.

## xp face contrast rows (I6, known)
kind: issue
persona: Explorer (affinity 1, mouse first)
flavor: construct
severity: low
task: none
check: I6
steps: all actions
expected: no contrast breach
actual: the known xp tray clock family (3.27:1)
evidence: media/2026-11-26/persona-r15/captures/explorer/breaches.jsonl
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
actual: I2 'gaps: construct-mode-step-in and construct-mode-turn-left are 9 apart' (14 lines; co-44, open)
evidence: media/2026-11-26/persona-r15/captures/explorer/breaches.jsonl
status: new

## C1: cube row, Step in by mouse, Step back by clicking the hint: reached in 7 actions
kind: ux
persona: Explorer (affinity 1, mouse first)
flavor: construct
severity: low
task: C1
check: none
steps: click "A cube of desktops" row, click Step in, click the Step back hint
expected: cube, in, out
actual: reached; first click on the row opened the cube this time (r14 only highlighted)
evidence: media/2026-11-26/persona-r15/captures/explorer/shots/004.png
status: new

## T1: terminal is called Foot in Programs, no plain Terminal entry
kind: ux
persona: Explorer (affinity 1, mouse first)
flavor: construct
severity: low
task: T1
check: none
steps: top bar Programs, Foot; the list also holds Foot Client, UXTerm, XTerm
expected: an entry called Terminal
actual: reached by choosing Foot as the first of four terminals; the persona does not know the difference (the "Terminal" row on the empty card was not used)
evidence: media/2026-11-26/persona-r15/captures/explorer/shots/007.png
status: new

## T2: key list found by the hint bar All keys and closed by Close
kind: ux
persona: Explorer (affinity 1, mouse first)
flavor: construct
severity: low
task: T2
check: none
steps: click All keys in the hint bar of the terminal, read, click Close
expected: list, then back
actual: reached (5 actions). The list text is small at the 800 px look
evidence: media/2026-11-26/persona-r15/captures/explorer/shots/009.png, 010.png
status: new
