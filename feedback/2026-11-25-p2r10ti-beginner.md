# p2r10 beginner (tiling), round 10 on e39448ad

Run: session tiling 1280x800, haiku persona, PERSONA_CHECK=1, helper transcript seeded, one `begin` per goal, waybar 1. fuzzel 1.11.1. 19 actions (caps T1 5, T2 4, T4 5, T6 6; used 5, 3, 5, 6), 9 looks. Contracts: T1 pass, T2 pass, T4:power pass (open [], no power call), T6 pass. Breaches outside known-red: 0.
Persona says: T1, T2, T4 reached; T6 "not reached" (Return pressed, row "Nothing found for 'frobnicate'": contract pass, the persona read it as a failure to start, as r9).
Captures: media/2026-11-25/persona-tiling-r10-beginner (shots, actions.jsonl, calls.log), -checks/beginner-*.txt.

## ux: T1 by the Apps door this time (mouse); the helper row stands first and is preselected
kind: ux
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: low
task: T1
check: none
steps: look, click Apps, look (list: "Chat with a helper (atlas)" highlighted, Foot, Foot Client, Foot Server, ImageMagick, LibreOffice ...), click Foot, look
expected: a terminal
actual: T1 by the door in 5 actions (r9: by a key); ti-50 holds (helper row first). The persona did not know what "Looks", "Tile" and "Listening" mean and left them alone
evidence: media/2026-11-25/persona-tiling-r10-beginner/shots/001-003.png
status: new

## ux: the key list stays open until Escape (ti-27 known); the power menu preselects Lock
kind: ux
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: low
task: T4
check: none
steps: T2 Keys button (list opened); T4: Escape (list closed), click the power icon, look (Lock highlighted), Escape, look
expected: menu opens and closes
actual: contract pass; the persona found the list still open after T2 ("did not close on its own") and closed it with Escape because the list says so; Lock preselected
evidence: media/2026-11-25/persona-tiling-r10-beginner/shots/004-007.png, -checks/beginner-T4:power.txt
status: new

## ux: an unknown program name: "Nothing found" only after Return, read as a failed start
kind: ux
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: low
task: T6
check: none
steps: Super+R, type frobnicate, Return, look (row "Nothing found for 'frobnicate'" first, the list below), Escape
expected: a clear failure notice
actual: contract T6 pass; persona "not reached", "Return did nothing"; the launcher is titled Apps (ti-49, ti-31 known)
evidence: media/2026-11-25/persona-tiling-r10-beginner/shots/008-009.png, -checks/beginner-T6.txt
status: new

## ux: the key list is dense and small at 800x500
kind: ux
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: low
task: T2
check: none
steps: click Keys, look
expected: a readable list
actual: the persona read "Super+." and "Super+," with doubt (glyphs hard to read at the capture size)
evidence: media/2026-11-25/persona-tiling-r10-beginner/shots/005.png
status: new
