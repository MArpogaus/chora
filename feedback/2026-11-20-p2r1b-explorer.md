# p2r1b Explorer (construct), round 1b on 141dfd97 (product as b97369d1; harness fixes only)

Run: session construct 1280x800, 15 actions, 8 looks, haiku persona, PERSONA_CHECK=1, helper transcript seeded. Persona's tasks: C1, T1, T2.
Contracts (check.py end C1 T1 T2): C1 pass, T1 FAIL, T2 FAIL (both "expected open [], got ['construct-help']": the three tasks were checked at once and the key list of goal 3 was still open at the end; harness caveat as in r1, not a defect). Breaches: 4 lines of I7, all in the first click (one per output), then none (r1: 100).
Persona says: all three goals reached (cube after a click and Alt+Escape, Super+T terminal, key list from the Keys item of the top bar after two failed tries).
Captures: media/2026-11-20/persona-r1b-explorer/.

## The cube row steps into the XP face; the cube shows only after Alt+Escape
kind: ux
persona: Explorer, affinity 2, mouse first
flavor: construct
severity: medium
task: C1
check: none
steps: click row "A cube of desktops", looks, Alt+Escape
expected: the cube of desktops shows
actual: with xp running the row opens the cube and steps straight into face 1 (XP desktop, no cube view); the hint "Alt Esc Step back" brings the cube. The XP start failure of r1 is gone
evidence: shots of the run in the media folder
status: new

## Super+F1 does nothing visible with a terminal focused in a face
kind: issue
persona: Explorer, affinity 2, mouse first
flavor: construct
severity: medium
task: T2
check: none
steps: Super+T, Super+F1, click Keys in the top bar (the click went into the terminal: the bar is hidden inside a face), Alt+Escape, click Keys
expected: Super+F1 shows the key list
actual: no visible change in the face; the key list came only from the top bar after stepping back. The demo persona's Super+F1 inside a face opened XP's "Help and Support" instead: the key goes to xp in use mode (to confirm with the contract T2 on a run that starts in use mode)
evidence: shots of the run in the media folder
status: new

## The top bar is hidden inside a face, so Keys and Programs cannot be clicked
kind: ux
persona: Explorer, affinity 2, mouse first
flavor: construct
severity: low
task: T2
check: none
steps: see above
expected: a click on a top-bar item works or the bar says it is not there
actual: the click lands in the app under it; the bar returns after Alt+Escape
evidence: shots of the run in the media folder
status: new

## I7 once on all four outputs at the first click
kind: issue
persona: Explorer, affinity 2, mouse first
flavor: construct
severity: low
task: C1
check: I7
steps: first click (the cube row)
expected: no output shows one colour
actual: I7 on VIRTUAL-1..4 after that one action while the faces start (black, then the XP desktop); no later breach. Probably a face that has not drawn yet at the quiet snapshot: verify before the ledger
evidence: evidence/001-click.png
status: new
