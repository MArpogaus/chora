# p2r12 beginner (tiling, mouse first), round 12 on 3be27e32

Run: one tiling session 1280x800, haiku persona, PERSONA_CHECK=1, helper transcript seeded, one `begin` per goal. Caps T1 5, T2 4, T4 5, T6 6 (20); used 17 actions, 9 looks, no failed action. Contracts: T1 pass, T2 pass, T4:power pass, T6 pass. Breaches outside known-red: 0. Persona says: T1 reached (Apps, Foot), T2 reached (Keys button, 3 actions), T4 reached (power icon, Escape), T6 not reached (reads "Nothing found for frobnicate" as failure).
Captures: media/2026-11-26/persona-tiling-r12-beginner, media/2026-11-26/persona-tiling-r12-checks/beginner-*.txt.

## ux: ti-50 holds, the Apps list has "Chat with a helper" as its first entry
kind: ux
persona: beginner, affinity 1, mouse first
flavor: tiling
severity: low
task: T1
check: none
steps: click Apps, look, click Foot
expected: a visible door to a program
actual: the list shows Chat with a helper, Foot, Foot Client, Foot Server, ImageMagick, LibreOffice ...; the persona names the helper entry and does not click it
evidence: media/2026-11-26/persona-tiling-r12-beginner/shots/002.png
status: new

## ux: an unknown program name: "Nothing found" only after Return, the launcher stays open, the box looks empty
kind: ux
persona: beginner, affinity 1, mouse first
flavor: tiling
severity: low
task: T6
check: none
steps: click Apps, type frobnicate, look, Return, look
expected: a program, or a clear error
actual: contract T6 pass (Nothing found row); persona: "the box and the list disagree" (search box back to "Apps |" while the row says Nothing found for "frobnicate"), Return only searched again, the launcher stays open (ti-31, ti-49 known)
evidence: media/2026-11-26/persona-tiling-r12-beginner/shots/008-009.png
status: new

## ux: the key list is dense and small at 800 px; "Tag" and "Tile" are not explained; the power menu preselects Lock
kind: ux
persona: beginner, affinity 1, mouse first
flavor: tiling
severity: low
task: T2
check: none
steps: click Keys, look, look --max-side 1280; power icon, look, Escape
expected: a short readable list; a safe default in the power menu
actual: the persona needed a 1280 px picture to read the key names; "Window before" and "Tag before" look alike; Lock is the highlighted entry, Return would lock. The tool output of the persona's first look was followed by a block of "MCP Server Instructions" text (env noise of the subagent runner, not the product)
evidence: media/2026-11-26/persona-tiling-r12-beginner/shots/004-007.png
status: new
