# p2r7co explorer (construct), round 7 on bc153929

Run: construct 1280x800, 14 actions, 8 looks (caps C1 12/6, T1 14/6, T2 19/8; totals 45/20, none breached), haiku, PERSONA_CHECK=1, helper transcript seeded. Contracts: C1 pass, T1 pass, T2 pass. Persona: C1 not reached, T1 reached, T2 reached. Breaches that are not I6: none. Fix check co-48 (explorer T1): pass.
Captures: media/2026-11-23/persona-r7-explorer/.

## xp face contrast rows (I6)
kind: issue
persona: explorer, affinity 1, mouse first
flavor: construct
severity: low
task: none
check: I6
steps: all 14 actions (looks and clicks in the xp face)
expected: no contrast breach
actual: 234 I6 rows (tray clock, title, taskbar button, menu frame), the known xp contrast family
evidence: breaches.jsonl, evidence/
status: new

## C1: the row "A cube of desktops" enters face 0 with no overview (fourth time)
kind: ux
persona: explorer, affinity 1, mouse first
flavor: construct
severity: medium
task: C1
check: none
steps: look; click row "A cube of desktops" (326,292); Super+C; Super (act.py refused the name)
expected: a cube of desktops to step into and out of
actual: the launcher closed and a plain desktop with a pager 1 to 4 showed; Super+C changed nothing visible; contract C1 pass ("cube in a face")
evidence: shots/002.png-005.png; actions.jsonl 2 to 6
status: new

## T1 reached: Start, Terminal
kind: ux
persona: explorer, affinity 1, mouse first
flavor: construct
severity: low
task: T1
check: contract T1
steps: look, click Start (30,490), click Terminal (40,322), look
expected: a terminal
actual: terminal "foot" open, contract T1 pass (co-48 holds for the click path); the persona saw a locale line "'C' is not a UTF-8 locale" (env noise, rejected: env)
evidence: shots/006.png
status: new

## ux: key list was found
kind: ux
persona: explorer, affinity 1, mouse first
flavor: construct
severity: low
task: T2
check: contract T2
steps: look, click the pill "Super F1 All keys" (400,452), look
expected: a key list
actual: the list opened (Walk, Panes, Cube, Helper, Give a Window, ...); the persona noted the "Walk" section (W A S D) looks like cube mode while a terminal has the focus
evidence: shots/008.png
status: new
