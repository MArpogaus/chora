# p2r5 beginner (tiling), round 5 on 230ea0e5

Run: session tiling 1280x800, haiku persona, 19 own actions (caps T1 5, T2 4, T4 5, T6 5), fuzzel 1.9.2 (below 1.11: ti-1 in play); PERSONA_CHECK=1. Contracts: T1 FAIL (no new window; the Foot row click did nothing, fuzzel 1.9.2, ti-1 known), T2 verdict line not kept (I filtered the output; 44 I6 breaches printed, ti-30 key list 13 px / contrast, known), T4:power pass, T6 pass. Breaches: 256 I6 only (tiling key list, 13 px and contrast 6.50:1 / 1.00:1 rows; ti-30). 0 outside I6.
Persona says: goal 1 not reached, goal 2 reached (close not confirmed), goal 3 reached (cancel not confirmed), goal 4 not reached (Nothing found after Return).
Captures: media/2026-11-23/persona-tiling-r5-beginner/shots, evidence.

## Click on Foot in the Apps menu does not start the program (fuzzel 1.9.2)
kind: issue
persona: beginner, affinity 1, mouse first
flavor: tiling
severity: medium
task: T1
check: contract T1
steps: Apps click, click Foot row
expected: a terminal opens (ti-1 fixed for fuzzel 1.11)
actual: menu stays open, no window; contract red 'no new focused window of tiling; new: []'. Known ti-1 on fuzzel 1.9.2; environment, not product
evidence: media/2026-11-23/persona-tiling-r5-beginner/shots/002-003.png
status: new

## Key list I6 rows: 13 px keys and 1.00:1 contrast rows
kind: issue
persona: beginner, affinity 1, mouse first
flavor: tiling
severity: low
task: T2
check: I6
steps: clicked Keys while the Apps menu was still open
expected: readable list
actual: 256 I6 breaches (13 px, 6.50:1, 1.00:1 where the Apps menu overlaps); ti-30 open
evidence: media/2026-11-23/persona-tiling-r5-beginner/breaches.jsonl
status: new

## Escape closes one layer at a time; the Keys list stays under the Apps menu
kind: ux
persona: beginner, affinity 1, mouse first
flavor: tiling
severity: low
task: T2
check: none
steps: Keys click with Apps open, Escape
expected: one Escape closes the list
actual: Escape closed only the Apps menu, the list stayed
evidence: media/2026-11-23/persona-tiling-r5-beginner/shots/005.png
status: new

## Super+R opens the Apps list, not a run box; Nothing found only after Return
kind: ux
persona: beginner, affinity 1, mouse first
flavor: tiling
severity: low
task: T6
check: none
steps: Super+R, typed frobnicate, Return
expected: a message that it cannot start
actual: the Nothing found row after Return only (known xr-2 / r4)
evidence: media/2026-11-23/persona-tiling-r5-beginner/shots/009.png
status: new

## Power menu opens with Lock selected
kind: ux
persona: beginner, affinity 1, mouse first
flavor: tiling
severity: low
task: T4
check: none
steps: power icon click
expected: a neutral first row
actual: Lock highlighted (known, r4)
evidence: media/2026-11-23/persona-tiling-r5-beginner/shots/007.png
status: new
