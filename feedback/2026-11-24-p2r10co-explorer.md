# p2r10co Explorer (affinity 1, mouse first) (construct), round 10 on f1457035

Run: construct 1280x800, 15 actions, 9 looks (caps C1 12/6, T1 14/6, T2 19/8; none breached), haiku, PERSONA_CHECK=1, helper transcript seeded. Contracts: C1 pass, T1 pass, T2 pass. Persona: C1 reached, T1 reached, T2 reached. Breaches: I2 co-44 x18, I6 x76 (xp tray clock), nothing else. Fix check: co-61 holds (Alt Esc Step back named), co-52 holds (row shows the cube stood back), co-54 holds (Foot pane beside the cube).
Captures: media/2026-11-24/persona-r10-explorer/.

## xp face contrast rows (I6)
kind: issue
persona: Explorer, affinity 1, mouse first
flavor: construct
severity: low
task: none
check: I6
steps: all 15 actions
expected: no contrast breach
actual: the known xp tray clock family, 76 rows
evidence: breaches.jsonl
status: new

## Cube stood back: I2 gaps rows (co-44, known)
kind: issue
persona: Explorer, affinity 1, mouse first
flavor: construct
severity: low
task: C1
check: I2
steps: click "A cube of desktops", Return, Alt+Escape
expected: mode chips apart enough
actual: I2 "9 apart" x18 (co-44)
evidence: breaches.jsonl
status: new

## C1: cube, step in, step out reached
kind: ux
persona: Explorer, affinity 1, mouse first
flavor: construct
severity: low
task: C1
check: contract C1
steps: look, click the row "A cube of desktops", look, Return (Step in), look, Alt+Escape (hint "Alt Esc Step back"), look
expected: a way out is named
actual: reached in 7 actions; the hint bar names the key (co-61 fix holds)
evidence: persona-r10-explorer shots 002-004
status: new

## T1: terminal through the Programs menu
kind: ux
persona: Explorer, affinity 1, mouse first
flavor: construct
severity: low
task: T1
check: contract T1
steps: click Programs in the top bar, click Foot
expected: a terminal
actual: reached in 5 actions. Note: three terminals in the list (Foot, UXTerm, XTerm), none named default
evidence: persona-r10-explorer shots 005-007
status: new

## T2: key list found by the hint bar
kind: ux
persona: Explorer, affinity 1, mouse first
flavor: construct
severity: low
task: T2
check: contract T2
steps: read the hint bar "Super F1 All keys", Super+F1, look
expected: the key list
actual: reached in 3 actions; the list is searchable and says only keys that work now are listed
evidence: persona-r10-explorer shot 009
status: new
