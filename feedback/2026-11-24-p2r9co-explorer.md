# p2r9co explorer (construct), round 9 on b2fa7792

Run: construct 1280x800, 15 actions, 9 looks (caps C1 12/6, T1 14/6, T2 19/8; totals 45/20, none breached), haiku, mouse first, PERSONA_CHECK=1, helper transcript seeded. Contracts: T1 pass, T2 pass; C1 contract line **not captured** (driver error: output cut before the task line; `end` is single use). Persona: C1, T1, T2 all reached. Breaches: I2 co-44 x20 and I6 x79, nothing else.
Fix check: co-52 holds (row "A cube of desktops" showed the cube stood back, Return stepped in, Alt+Escape stepped out, as the hint bar said), co-61 holds (the hint bar names Alt Esc Step back; the persona used it), co-54 and co-62 not in this persona's tasks.
Captures: media/2026-11-24/persona-r9-explorer/ (shots, actions, breaches).

## xp face contrast rows (I6)
kind: issue
persona: explorer, affinity 1, mouse first
flavor: construct
severity: low
task: none
check: I6
steps: all 15 actions
expected: no contrast breach
actual: the known xp tray clock contrast family (3.27:1), 79 rows
evidence: breaches.jsonl
status: new

## Cube stood back: I2 gaps rows (co-44, known)
kind: issue
persona: explorer, affinity 1, mouse first
flavor: construct
severity: low
task: C1
check: I2
steps: row, Return, Alt+Escape
expected: mode chips apart enough
actual: I2 "construct-mode-step-in and construct-mode-turn-left are 9 apart" x20 (co-44)
evidence: breaches.jsonl
status: new

## C1: cube, step in, step out reached
kind: ux
persona: explorer, affinity 1, mouse first
flavor: construct
severity: low
task: C1
check: contract C1
steps: look, click row "A cube of desktops", Return, Alt+Escape
expected: a way in and out
actual: reached in 4 input actions. Persona surprise: the cube label reads "XP 1 · face 1 of 4" both in and out, so "which face did I step into" is not told; the hint bar writes "Alt Esc" while the key named is Alt+Escape (worked)
evidence: shots/001 to 004
status: new

## T1: terminal through the Programs menu
kind: ux
persona: explorer, affinity 1, mouse first
flavor: construct
severity: low
task: T1
check: contract T1
steps: click Programs, click Foot
expected: a terminal
actual: reached (contract pass). Three terminal-like entries (Foot, UXTerm, XTerm); a locale warning line in the terminal (env)
evidence: shots/005 to 007
status: new

## T2: key list found through All keys
kind: ux
persona: explorer, affinity 1, mouse first
flavor: construct
severity: low
task: T2
check: contract T2
steps: click "All keys" in the hint bar
expected: the key list
actual: reached in 3 actions; sheet readable, groups "CONSTRUCT · IN A PANE", "· EVERY LOOK" unexplained; "cube" and "face" never defined; small key labels on a 800 px capture
evidence: shots/008, 009
status: new
