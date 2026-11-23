# p2r6co explorer (construct), round 6 on 86e619f3

Run: construct 1280x800, 17 actions, 10 looks (caps C1 12/6, T1 14/6, T2 19/8; totals 45/20), haiku, PERSONA_CHECK=1, helper transcript seeded, one begin per goal. Contracts: C1 pass, T1 FAIL, T2 pass. Persona: C1 not reached, T1 reached (Start, Terminal), T2 reached (clicked the "Super F1 All keys" pill in the face; the key list opened). Breaches that are not I6: none (no I3, no I1, no I2). I6 rows on the xp face (tray clock, window title, taskbar button, Start menu frame) are the known xp contrast rows.
Captures: media/2026-11-23/persona-r6-explorer/.

## C1: the row "A cube of desktops" enters face 0 with no overview (again)
kind: ux
persona: explorer, affinity 1, mouse first
flavor: construct
severity: medium
task: C1
check: none
steps: look, click the row "A cube of desktops" (326,292), look, Super+C, look, Super+C, look (three attempts, stop)
expected: the whole cube
actual: a plain xp desktop (face 0); Super+C twice nothing visible. Contract C1 is green (cube in a face, face 0), so the cube exists but the person never sees it
evidence: shots/004.png, 006.png, 007.png; actions.jsonl 1 to 7
status: new

## T1 red: the terminal started from Start in the xp face is owned by xp, not by construct
kind: issue
persona: explorer, affinity 1, mouse first
flavor: construct
severity: medium
task: T1
check: contract T1
steps: Super+Return (nothing), Start (30,490), Terminal (40,325); a foot window opened with a taskbar entry
expected: a new focused toplevel of the look
actual: contract T1 "no new focused window of construct; new: ['s1']" (same as r5). The persona reached its goal; the red is the contract gap or the ownership of a window opened in a face. I3 of r5 did not recur in this run
evidence: shots/008.png (foot window), actions.jsonl 8 to 14
status: new

## Super+Return does nothing on the face
kind: ux
persona: explorer, affinity 1, mouse first
flavor: construct
severity: low
task: T1
check: none
steps: Super+Return on the xp face, then look
expected: a terminal (the hint "Super F1 All keys" suggests Super keys work)
actual: nothing opened; Start menu route worked
evidence: actions.jsonl 9 to 10
status: new

## xp face contrast rows (I6)
kind: issue
persona: explorer, affinity 1, mouse first
flavor: construct
severity: low
task: none
check: I6
steps: any look with the xp face and a window
expected: text contrast of at least 4.5:1
actual: tray clock 3.27:1 (xp-tray-VIRTUAL-n, exempt floor 3.0), title 'foot' xp-title-s2 2.15:1, taskbar xp-task-s2-VIRTUAL-2 3.86:1, xp-menu-frame; the same family as the known xp rows, named for VIRTUAL-2 now
evidence: breaches.jsonl (282 I6 rows)
status: new

## ux: key list was found
kind: ux
persona: explorer, affinity 1, mouse first
flavor: construct
severity: low
task: T2
check: none
steps: look, click the pill "Super F1 All keys" (430,452), look
expected: the key list by mouse
actual: reached in 3 actions; the panel lists the keys as chips; the labels "CONSTRUCT · CUBE", "EVERY LOOK" mean nothing to a newcomer (known co-45 I6 sizes aside)
evidence: shots/010.png
status: new
