# p2r11co Explorer (affinity 1, mouse first) (construct), round 11 on be6e62ec

Run: construct 1280x800, 15 actions, 9 looks (caps C1 12/6, T1 14/6, T2 19/8; used 7/4, 5/3, 3/2; none breached), haiku, PERSONA_CHECK=1, helper transcript seeded. Contracts: C1 pass, T1 pass, T2 pass. Persona: all three reached. Breaches: I2 co-44 x18, I6 rows. Fix check: co-61 holds (row, Return, Alt+Escape), co-52, co-54 hold; co-68, rcp1-1, rcp1-2 not met by this persona.

Captures: media/2026-11-24/persona-r11-explorer/.

## xp face contrast rows (I6)
kind: issue
persona: Explorer (affinity 1, mouse first)
flavor: construct
severity: low
task: none
check: I6
steps: all actions
expected: no contrast breach
actual: the known xp tray clock family
evidence: breaches.jsonl
status: new

## Cube stood back: I2 gaps rows (co-44, known)
kind: issue
persona: Explorer (affinity 1, mouse first)
flavor: construct
severity: low
task: C1
check: I2
steps: cube stood back
expected: mode chips apart enough
actual: I2 '9 apart' (co-44)
evidence: breaches.jsonl
status: new

## C1: cube, step in, step out reached
kind: ux
persona: Explorer (affinity 1, mouse first)
flavor: construct
severity: low
task: C1
check: contract C1
steps: click the row 'A cube of desktops', Return, Alt+Escape
expected: the cube again
actual: reached in 7 actions; the hint bar names Return Step in and Alt Esc Step back. The persona typed Alt+Escape for the hint 'Alt Esc' (no mismatch seen by it)
evidence: persona-r11-explorer shots 001-004
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
actual: reached in 5 actions. Surprises: the Foot window has no title bar, tiny text, a locale warning 'C is not a UTF-8 locale' (env, not product)
evidence: persona-r11-explorer shots 005-007
status: new

## T2: key list found by the hint bar
kind: ux
persona: Explorer (affinity 1, mouse first)
flavor: construct
severity: low
task: T2
check: contract T2
steps: Super+F1 from the hint bar 'Super F1 All keys'
expected: the key list
actual: reached in 3 actions. The sheet says 'Only the keys that work now are listed' and its header reads CONSTRUCT · CUBE, so the persona doubted it was the full list
evidence: persona-r11-explorer shot 009
status: new
