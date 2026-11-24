# p2r8 beginner (tiling), round 8 on bb4c07d9

Run: session tiling 1280x800, haiku persona, 19 actions (caps T1 5, T2 4, T4 5, T6 6; used 5, 3, 5, 6), 11 looks; PERSONA_CHECK=1, helper transcript seeded, one `begin` per goal. fuzzel 1.11.1. Contracts: T1 pass, T2 pass, T4:power pass, T6 FAIL. Breaches outside known-red: 0 (no breaches.jsonl; r7 had I6 x116 on the open key list). Waybar processes: 1 at the start, 1 at the end.
Persona says: T1, T2, T4 reached; T6 not reached.
Captures: media/2026-11-24/persona-tiling-r8-beginner (shots, actions.jsonl, calls.log), media/2026-11-24/persona-tiling-r8-checks/beg-*.txt.

## T6: the persona typed the name and never pressed Return; contract red (ti-31 known)
kind: issue
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: medium
task: T6
check: contract T6
steps: look, click Apps, look, type frobnicate, look (empty list, no word), Escape (6 of 6 actions)
expected: a program or a notice
actual: check.py end T6: `FAIL: neither a new notice, the words No helper nor a launcher row Nothing found`. The list is silent while typing (ti-31, fuzzel empty list); the `Nothing found for` row shows only after Return (r7 persona pressed Return: pass), this one stopped at the empty list and closed it. Persona path on a known cause
evidence: media/2026-11-24/persona-tiling-r8-beginner/shots/010-011.png, media/2026-11-24/persona-tiling-r8-checks/beg-T6.txt
status: new

## ux: the Apps list is empty and silent for a name it does not know
kind: ux
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: medium
task: T6
check: none
steps: as T6
expected: a word that the program is not found
actual: a blank list; the persona concluded "the program is not installed or registered" and gave up without Return. The Apps list shows nine rows with no helper row among them (Dark look .. LibreOffice Draw; the helper row of ti-50 is appended last per gate-tiling-apps, below the nine rows shown; not scrolled to)
evidence: media/2026-11-24/persona-tiling-r8-beginner/shots/010-011.png
status: new

## ux: the key list says "lock, log out, screenshot", the power menu has Restart and Shut down too
kind: ux
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: low
task: T4
check: none
steps: T2 key list, T4 power icon, Escape
expected: the list names what the power menu offers
actual: key list header: "power icon: lock, log out, screenshot"; the power menu: Lock, Log out, Restart, Shut down, Screenshot (as r7)
evidence: media/2026-11-24/persona-tiling-r8-beginner/shots/005.png, 007.png
status: new

## ux: the key list is much bigger than the Keys button suggests, and tiny
kind: ux
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: low
task: T2
check: none
steps: click Keys, look
expected: a short list
actual: an overlay with six sections and about 60 rows in small text; the persona said arrows may be misread at this size. At T4 the persona found the list already gone after Escape on the power menu (ti-27 fixed: contract T4:power pass)
evidence: media/2026-11-24/persona-tiling-r8-beginner/shots/005.png, 008.png
status: new
