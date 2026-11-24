# p2r7 beginner (tiling), round 7 on ad0093ba

Run: session tiling 1280x800, haiku persona, 17 actions (caps T1 5, T2 4, T4 5, T6 6), 9 looks; PERSONA_CHECK=1, helper transcript seeded, one `begin` per goal. fuzzel 1.11.1. Contracts: T1 pass, T2 pass, T4:power FAIL, T6 pass. Breaches outside known-red: I6 x116 lines, all the key list (rows of the open list under later popups). Waybar processes: 1 at the start, 1 at the end.
Persona says: T1, T2, T4 reached; T6 not reached.
Captures: media/2026-11-24/persona-tiling-r7-beginner (shots, actions.jsonl, breaches.jsonl, calls.log, evidence), media/2026-11-24/persona-tiling-r7-checks/beginner-*.txt.

## Power menu opened and left open, key list of the goal before still open: T4 contract red
kind: issue
persona: beginner, affinity 1, mouse first
flavor: tiling
severity: medium
task: T4
check: contract T4:power
steps: click the power icon, look (menu: Lock, Log out, Restart, Shut down, Screenshot); no Escape; the key list from T2 was still open behind it
expected: the menu is left, `open` empty
actual: check.py end T4:power: `expected open [], got ['tiling-keys']`. No systemctl or loginctl call ran (calls.log). The persona read "leave without choosing" as "do not touch" and left the menu up; the key list of T2 was never closed (Escape closes one layer; ti-27, verify, same as r6). The power menu was closed by the Escape of the next goal
evidence: media/2026-11-24/persona-tiling-r7-beginner/shots/006-007.png, media/2026-11-24/persona-tiling-r7-checks/beginner-T4:power.txt
status: new

## Key list rows under a popup read 1.00:1 (I6, many rows, up from 2 in r6)
kind: issue
persona: beginner, affinity 1, mouse first
flavor: tiling
severity: low
task: T4
check: I6
steps: the key list stayed open through T4 and T6; the launcher and the power menu opened over it
expected: contrast of the guide (7:1)
actual: 116 I6 lines on the key list (tiling-keys-body-*): 'Tags' and most row labels and keys 1.00:1 (text colour equals the panel), 'Move focus' 1.34:1, 'Window before' 3.77:1. r6 had 2 ('Tags'). Cause not checked: the rows may be read while a popup dims or covers them (look at the checker, then ti-30). The open list is only a state a persona left; no persona saw unreadable rows
evidence: media/2026-11-24/persona-tiling-r7-beginner/breaches.jsonl, media/2026-11-24/persona-tiling-r7-checks/beginner-T6.txt
status: new

## Launcher: "Nothing found" appears only after Return and the persona read it as no answer
kind: ux
persona: beginner, affinity 1, mouse first
flavor: tiling
severity: medium
task: T6
check: none
steps: Escape, Super+R, type frobnicate, Return, look
expected: the desktop says it cannot start frobnicate
actual: after Return the first row reads `Nothing found for 'frobnicate'` above the app list; contract T6 pass. The persona answered "not reached": it expected a program or a notice and the row looks like a list entry that does nothing on Return. While typing, the list is empty and silent (ti-31, known)
evidence: media/2026-11-24/persona-tiling-r7-beginner/shots/009.png
status: new

## ux: the key list says "lock, log out, screenshot", the power menu has Restart and Shut down too
kind: ux
persona: beginner, affinity 1, mouse first
flavor: tiling
severity: low
task: T4
check: none
steps: read the key list (T2), opened the power menu (T4)
expected: the same words
actual: Mouse line of the list: power icon gives lock, log out, screenshot; the menu lists Lock, Log out, Restart, Shut down, Screenshot, Lock pre-selected (Return would lock)
evidence: media/2026-11-24/persona-tiling-r7-beginner/shots/005.png, 007.png
status: new
