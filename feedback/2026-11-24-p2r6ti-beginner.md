# p2r6 beginner (tiling), round 6 on cae62f50

Run: session tiling 1280x800, haiku persona, 19 actions (caps T1 5, T2 4, T4 5, T6 6); PERSONA_CHECK=1, helper transcript seeded, one `begin` per goal. fuzzel 1.11.1 on this host (no ti-1). Contracts: T1 pass, T2 pass, T4:power FAIL (key list still open), T6 pass. Breaches outside known-red: I6 x2 (key list 'Tags' 1.00:1).
Persona says: T1, T2, T4 reached; T6 not reached (no text before Return).
Captures: media/2026-11-24/persona-tiling-r6-beginner (shots, actions.jsonl, breaches.jsonl, evidence).

## Power menu cancelled but the key list from the goal before is still open: T4 contract red
kind: issue
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: low
task: T4:power
check: contract T4:power
steps: after goal 2 (Keys click, list opens) the persona clicked the power icon, menu opened (Lock, Log out, Restart, Shut down, Screenshot), Escape closed the menu; the list stayed (Escape closes one layer)
expected: open empty at the end
actual: check.py end T4:power: `expected open [], got ['tiling-keys']`. The power menu itself closed (shot 008) and no systemctl call ran; the red is the key list of the earlier goal, which the persona never closed (same as ti-27, verify). A second Escape was not pressed because the persona was not told to close it
evidence: media/2026-11-24/persona-tiling-r6-beginner/shots/008.png, media/2026-11-24/persona-tiling-r6-checks/beginner-T4.txt
status: new

## Key list: group title 'Tags' has the contrast 1.00:1
kind: issue
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: low
task: T2
check: I6
steps: click Keys; the checker ran after the click and after the next look
expected: title readable at 7:1
actual: I6 (2 lines of breaches.jsonl): 'Tags' (tiling-keys-body-72) 1.00:1, (26, 29, 39) on (26, 29, 39), so the title is drawn in the panel colour; the other group titles pass after tiling-disc (7:1, 14 px). Same row class as ti-30 rows in r5 (1.00:1); Kemal's Keys list produced no such row in the same build, the beginner's did, so it may depend on the page the panel is drawn over
evidence: media/2026-11-24/persona-tiling-r6-beginner/evidence/001-click.png, 002-look.png, media/2026-11-24/persona-tiling-r6-beginner/breaches.jsonl
status: new

## Launcher shows no text before Return; 'Nothing found' only after Return, and as a selectable-looking row
kind: ux
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: medium
task: T6
check: none
steps: Escape, Apps, type frobnicate, look (list empty, nothing said), Return, look
expected: a sentence that frobnicate does not exist while typing
actual: before Return the list was empty and silent; after Return the first row `Nothing found for "frobnicate"` highlighted as if selectable, the full list back under it. T6 contract pass, persona says not reached (known ti-31, fuzzel has no empty-list text; not a new finding)
evidence: media/2026-11-24/persona-tiling-r6-beginner/shots/009-010.png
status: new

## Launcher lists Foot, Foot Client and Foot Server
kind: ux
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: low
task: T1
check: none
steps: click Apps, click Foot
expected: one terminal entry
actual: three similar entries; the persona guessed the plain one. Click on the row worked (fuzzel 1.11.1)
evidence: media/2026-11-24/persona-tiling-r6-beginner/shots/003.png
status: new

## Keys panel: the top edge touches the bar, 'Super' is not explained, key names small
kind: ux
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: low
task: T2
check: none
steps: click Keys, look
expected: readable list
actual: list opened by the Keys button at once (reached in 3 actions); top edge cut by the bar, text small at the 800 px capture, 'Super' unexplained
evidence: media/2026-11-24/persona-tiling-r6-beginner/shots/005.png
status: new

## The top bar is drawn twice
kind: ux
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: low
task: none
check: none
steps: every look
expected: one bar
actual: two bars, as the Kemal file (same finding, one evidence set there)
evidence: media/2026-11-24/persona-tiling-r6-beginner/shots/002.png
status: new
