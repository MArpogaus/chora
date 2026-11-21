# p2r4co explorer (construct), round 4 on a5b77488

Run: construct 1280x800, 17 actions, 10 looks, haiku, PERSONA_CHECK=1, helper transcript seeded, one begin per goal (C1, T1, T2). Contracts: C1 pass, T1 pass, T2 FAIL. Breaches: I7 x4 at the first click (known, harness, r2). Persona: C1 reached, T1 reached (Programs, Foot), T2 not reached (F1 and ? in a face: nothing).
Captures: media/2026-11-21/persona-r4-explorer/.

## T2 red: the key list construct-help was never open
kind: issue
persona: explorer, affinity 1, mouse first
flavor: construct
severity: high
task: T2
check: contract T2
steps: in the face with a Foot terminal: click on the "XP 1" pill (357,451), F1, Shift+slash, three looks
expected: a key list opens by a visible door
actual: contract T2 "the key list construct-help was never open"; no sheet in any shot. The persona did not press Super+F1 (r3 explorer did) and the bottom bar names no key for the list
evidence: media/2026-11-21/persona-r4-explorer/shots 007 to 010, actions.jsonl lines 11 to 17
status: new

## I7 at the first click on all four outputs
kind: issue
persona: explorer, affinity 1, mouse first
flavor: construct
severity: low
task: C1
check: I7
steps: first click on "A cube of desktops"
expected: no one-colour work area
actual: 4 breaches I7 at the first click (known: before the faces draw, harness r2, not a finding)
evidence: media/2026-11-21/persona-r4-explorer/breaches.jsonl
status: new

## No visible door to the key list in a face or stood back
kind: ux
persona: explorer, affinity 1, mouse first
flavor: construct
severity: medium
task: T2
check: contract T2
steps: as above
expected: a Keys item or a hint with the key (the top bar has Keys, but it is hidden once a window is open)
actual: bottom bar shows turn and step-in hints only; the top bar was gone (also r3 ux)
evidence: media/2026-11-21/persona-r4-explorer/shots/010.png
status: new

## ux: Foot text is very small at the capture scale
kind: ux
persona: explorer, affinity 1, mouse first
flavor: construct
severity: low
task: T1
check: none
steps: Programs, Foot
expected: readable prompt
actual: the prompt is hard to read at 800 px; locale warning first line (env)
evidence: media/2026-11-21/persona-r4-explorer/shots/006.png
status: new
