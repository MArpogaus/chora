# p2r9 beginner (tiling), round 9 on 721be592

Run: session tiling 1280x800, haiku persona, 15 actions (caps T1 5, T2 4, T4 5, T6 6; used 3, 2, 4, 6), 8 looks; PERSONA_CHECK=1, helper transcript seeded, one `begin` per goal. fuzzel 1.11.1. Contracts: T1 pass, T2 pass, T4:power pass, T6 pass. Breaches outside known-red: 0 (r7: 116, r8: 0). Waybar processes: not counted for this session (driver slip); none left after `round.sh stop`.
Persona says: T1, T2, T4, T6 all reached except T6 "not reached" (it pressed Return and the row Nothing found showed: contract pass; the persona read it as a failure to start).
Captures: media/2026-11-25/persona-tiling-r9-beginner (shots, actions.jsonl, calls.log), media/2026-11-25/persona-tiling-r9-checks/beginner-*.txt.

## ux: Apps list: the helper row stands first, an unknown name gives an empty list, Nothing found only after Return
kind: ux
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: low
task: T6
check: none
steps: Super+R (the persona chose the key), look (list: Chat with a helper (atlas), Foot, Foot Client, Foot Server, ImageMagick, LibreOffice ...), type frobnicate, look (empty list), Return, look (row "Nothing found for 'frobnicate'")
expected: a word that the program is not found
actual: ti-50 holds on the first row; for the name the list is blank until Return (ti-49, ti-31 known); the persona wrote "Return did not run the typed text" and did not close the list
evidence: media/2026-11-25/persona-tiling-r9-beginner/shots/006-008.png, media/2026-11-25/persona-tiling-r9-checks/beginner-T6.txt
status: new

## ux: the power menu preselects Lock
kind: ux
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: low
task: T4
check: none
steps: Super+Shift+E (read from the key list of T2), look, Escape, look
expected: n/a
actual: Lock is highlighted: Return would lock the screen. The persona left by Escape alone; the key list of T2 was gone after it (ti-27 holds: check.py end T4:power pass, open [])
evidence: media/2026-11-25/persona-tiling-r9-beginner/shots/004.png 005.png, media/2026-11-25/persona-tiling-r9-checks/beginner-T4.txt
status: new

## ux: a mouse-first persona starts with keys
kind: ux
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: low
task: T1
check: none
steps: look, Super+Return (before any list or hint), look
expected: Apps click
actual: T1 passes at once by a key the persona guessed ("common desktop conventions"), so the Apps door was not exercised for T1; T2 was by the Keys button, T4 by the key it read there, T6 by Super+R. The persona called a documentation tool once after T1 (outside its rules; no effect on the session)
evidence: media/2026-11-25/persona-tiling-r9-beginner/actions.jsonl
status: new

## ux: the key list is large and small
kind: ux
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: low
task: T2
check: none
steps: click Keys, look, read
expected: a short list
actual: about 60 rows in small text; the persona flagged Super+. and Super+, as hard to read
evidence: media/2026-11-25/persona-tiling-r9-beginner/shots/003.png
status: new
