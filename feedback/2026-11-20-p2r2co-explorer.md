# p2r2co Explorer (construct), round 2 on 59efbc17

Run: session construct 1280x800, 13 actions, 7 looks, haiku persona, PERSONA_CHECK=1, helper transcript seeded, begin once before the three goals (not per task: one persona run). Contracts (check.py end C1 T1 T2): C1 pass, T1 pass, T2 pass. Breaches: I7 x4 (first click, one per output; as r1b).
Persona says: all three goals reached (row then Alt+Escape then Return and Alt+Escape; Super+T; Super+F1 showed "All keys").
Captures: media/2026-11-20/persona-r2-explorer/ (shots 001-007, actions.jsonl, breaches.jsonl).

## I7 on all four outputs at the first click
kind: issue
persona: Explorer, affinity 1, mouse first
flavor: construct
severity: low
task: C1
check: I7
steps: first click, the row "A cube of desktops"
expected: faces drawn when the checker takes its first quiet snapshot
actual: "the work area of VIRTUAL-1..4 shows one colour" once per output at the first click, then none (same as r1b; probably faces not yet drawn at the first quiet snapshot, not confirmed)
evidence: media/2026-11-20/persona-r2-explorer/breaches.jsonl
status: new

## The row "A cube of desktops" steps into face 1; the cube shows only after Alt+Escape
kind: ux
persona: Explorer, affinity 1, mouse first
flavor: construct
severity: medium
task: C1
check: none
steps: click the row; look; Alt+Escape; the cube "XP 1 face 1 of 4" appeared
expected: the cube
actual: a plain XP desktop with the hint "Step back Alt Esc"; persona was unsure it had seen the cube (same as r1b explorer)
evidence: shots 003-004
status: new

## Key list wording "stood back" and Esc against Alt+Esc
kind: ux
persona: Explorer, affinity 1, mouse first
flavor: construct
severity: low
task: T2
check: none
steps: Super+F1
expected: the same key named in the hint and in the list
actual: hint says Alt Esc to step back, the list says "Back out one step" Esc and "Close the cube, stood back"; "stood back" reads oddly; a LISTENING indicator stays on in every view and means nothing to the persona
evidence: shot 007
status: new
