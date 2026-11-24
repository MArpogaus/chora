# p2r12co Explorer (affinity 1, mouse first) (construct), round 12 on 280b9dcf

Run: construct 1280x800, 20 actions, 11 looks (caps C1 12/6, T1 14/6, T2 19/8; used 8/4, 5/3, 7/4; none breached), haiku, PERSONA_CHECK=1, helper transcript seeded. Contracts: C1 pass, T1 pass, T2 pass. Persona: all three reached. Breaches: I2 co-44 x26, I6 rows. Fix check: co-71 (key sheet): in T2 opened by Super+F1 and closed by Escape, passes. co-70, C2 not met by this persona.

Captures: media/2026-11-24/persona-r12-explorer/.

## xp face contrast rows (I6)
kind: issue
persona: Explorer (affinity 1, mouse first)
flavor: construct
severity: low
task: none
check: I6
steps: all actions
expected: no contrast breach
actual: the known xp tray clock family (3.21:1)
evidence: media/2026-11-24/persona-r12-explorer/breaches.jsonl
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
actual: I2 '9 apart' x26 (co-44)
evidence: media/2026-11-24/persona-r12-explorer/breaches.jsonl
status: new

## C1: cube, step in, step out reached
kind: ux
persona: Explorer (affinity 1, mouse first)
flavor: construct
severity: low
task: C1
check: contract C1
steps: click the row 'A cube of desktops', Return, Alt+Esc (rejected by act.py: key name is Escape), Alt+Escape
expected: cube, in, out
actual: reached in 8 actions; the hint bar says 'Esc' and 'Alt Esc' while act.py wants 'Escape' (harness naming, one failed attempt)
evidence: media/2026-11-24/persona-r12-explorer shots 001-004
status: new

## T1: terminal through the Programs menu
kind: ux
persona: Explorer (affinity 1, mouse first)
flavor: construct
severity: low
task: T1
check: contract T1
steps: click Programs, click Foot
expected: a terminal
actual: reached in 5 actions; Foot opens full screen with no frame
evidence: media/2026-11-24/persona-r12-explorer shots 005-007
status: new

## T2: key list found by the hint bar
kind: ux
persona: Explorer (affinity 1, mouse first)
flavor: construct
severity: low
task: T2
check: contract T2
steps: Super+F1 (named by the hint bar), read, Escape
expected: key list, closed again
actual: reached in 7 actions; Escape closed the sheet and the cube view came back (the persona wondered if Escape also stepped back)
evidence: media/2026-11-24/persona-r12-explorer shots 008-011
status: new
