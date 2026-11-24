# p2r8co explorer (construct), round 8 on 8c411416

Run: construct 1280x800, 21 actions, 12 looks (caps C1 12/6, T1 14/6, T2 19/8; totals 45/20, none breached), haiku, PERSONA_CHECK=1, helper transcript seeded. Contracts: C1 pass, T1 pass, T2 pass. Persona: C1 reached, T1 reached, T2 reached. Breaches not I6 and not the known co-44 I2: none. Fix check: co-52 (explorer C1) pass: the row "A cube of desktops" shows the cube stood back (it showed a plain desktop in r7).
Captures: media/2026-11-24/persona-r8-explorer/ (shots, actions.jsonl, breaches.jsonl; evidence dropped, I6 and co-44 only).

## xp face contrast rows (I6)
kind: issue
persona: explorer, affinity 1, mouse first
flavor: construct
severity: low
task: none
check: I6
steps: all 21 actions
expected: no contrast breach
actual: 163 I6 rows, the known xp contrast family (tray clock 3.27:1)
evidence: breaches.jsonl
status: new

## Cube stood back: I2 gaps rows (co-44, known)
kind: issue
persona: explorer, affinity 1, mouse first
flavor: construct
severity: low
task: C1
check: I2
steps: click row "A cube of desktops", Return, F1, Alt+Escape, Escape
expected: no I2 row
actual: 28 I2 rows "construct-mode-step-in and construct-mode-turn-left/right are 9 apart", co-44 (known, mode chips)
evidence: breaches.jsonl
status: new

## C1: cube, step in, step out reached
kind: ux
persona: explorer, affinity 1, mouse first
flavor: construct
severity: low
task: C1
check: contract C1
steps: click row (326,292), look, Return, look, F1, look, Alt+Escape, look, Escape, look
expected: the cube, a face, back out
actual: reached. The persona stepped out by Alt+Escape only after F1 listed it; the key list then stayed open over the cube until Escape
evidence: shots 003-006; actions.jsonl 1 to 11
status: new

## T1: terminal through the Programs menu, full screen
kind: ux
persona: explorer, affinity 1, mouse first
flavor: construct
severity: low
task: T1
check: contract T1
steps: look, click Programs (98,32), look, click Foot (344,184), look
expected: a terminal window
actual: reached through the top bar menu "Programs" (not the Start card); the terminal fills the face with no title bar; the list shows Foot, Foot Client, Foot Server, UXTerm, XTerm
evidence: shots 007-009
status: new

## T2: key list found by Super+F1
kind: ux
persona: explorer, affinity 1, mouse first
flavor: construct
severity: low
task: T2
check: contract T2
steps: look, Super+F1, look, scroll, look
expected: a list of all keys
actual: reached. The list is headed "CONSTRUCT · CUBE", footer "Only the keys that now work are listed" (a person reads that as not all keys); two sections name F1 and Super+F1; a faint wireframe shows behind the overlay
evidence: shots 011-012
status: new
