# p2r5co explorer (construct), round 5 on 892d035c

Run: construct 1280x800, 21 actions, 12 looks, haiku, PERSONA_CHECK=1, helper transcript seeded, one begin per goal (C1, T1, T2). Contracts: C1 pass, T1 FAIL, T2 FAIL. Breaches outside known-red: I3 x6 (T1 2, T2 4). Persona: C1 not reached (the click on "A cube of desktops" went straight into face 0, no overview, Super+C did nothing inside the face), T1 reached (Start, Terminal in the xp face), T2 reached (Start, Help and Support, Keyboard shortcuts: an xp help page, not the construct key list).
Captures: media/2026-11-21/persona-r5-explorer/.

## T1 red: the terminal opened by the xp Start menu is owned by xp, not by construct
kind: issue
persona: explorer, affinity 1, mouse first
flavor: construct
severity: medium
task: T1
check: contract T1
steps: after C1 the persona stood in face 0 (xp). Start (30,490), Terminal (40,322); a foot window opened
expected: a new focused toplevel
actual: contract T1 "no new focused window of construct; new: ['s1']": the window belongs to the xp child inside the face, which construct does not count (r4 passed T1 by the Programs door of the walk room). Whether that is a contract gap or a bug is for triage; I3 below is the product side
evidence: media/2026-11-21/persona-r5-explorer/evidence/011-look.png, actions.jsonl lines 11 to 14
status: new

## I3 on every click inside the xp face: construct and xp each say they hold the focus
kind: issue
persona: explorer, affinity 1, mouse first
flavor: construct
severity: high
task: T1
check: I3
steps: C1 click into face 0, then Start, Terminal, Start, Help and Support, Keyboard shortcuts
expected: one holder of the focus
actual: I3 "construct and xp each say they hold the focus" after each click in the xp face (6 breaches in T1 and T2, 2 per click and look). r4 had 0 I1/I3 on the xp child in 24 actions (demo); the demo of r5 gets it from its first click in the face (see demo file), so no window is needed
evidence: media/2026-11-21/persona-r5-explorer/breaches.jsonl (I3 rows), evidence/011, 016, 018, 020
status: new

## T2 red: the key list construct-help was never open (mouse, in a face)
kind: issue
persona: explorer, affinity 1, mouse first
flavor: construct
severity: high
task: T2
check: contract T2
steps: Start (28,490), Help and Support (168,408), Keyboard shortcuts (209,151). The persona found a list of shortcuts in the xp help window
expected: the key list of the desktop opens by a visible door
actual: contract T2 "expected open [], got ['xp-dlg-help']; the key list construct-help was never open". The persona never pressed F1 or Super+F1 and the face shows no Keys door (top bar gone in a face, as r3 and r4). The xp help page lists Super+Ctrl+left/right (turn the cube) with the key labels cut off, but not the construct keys
evidence: media/2026-11-21/persona-r5-explorer/shots/012.png to 021.png, actions.jsonl lines 16 to 21
status: new

## C1: the row "A cube of desktops" enters face 0 without an overview
kind: ux
persona: explorer, affinity 1, mouse first
flavor: construct
severity: medium
task: C1
check: contract C1
steps: click on "A cube of desktops" (326,292); Super+C twice; click pager "1"
expected: the cube of all desktops, then in and out
actual: contract C1 pass (a cube is open, in face 0), but the persona sees a plain XP desktop with pager 1 to 4, Super+C and the pager change nothing it can see, and it does not get out to the overview. C1 stays "reached by contract, not by the persona" (as the r4 demo)
evidence: media/2026-11-21/persona-r5-explorer/shots/002.png to 005.png
status: new

## ux: "LISTENING" pill top right with no explanation; Foot window small; locale line
kind: ux
persona: explorer, affinity 1, mouse first
flavor: construct
severity: low
task: T1
check: none
steps: first look of T1, Terminal window
expected: nothing confusing
actual: a LISTENING pill that the persona did not understand; the new terminal is small (440x330 at the capture) and prints a locale warning first (env)
evidence: media/2026-11-21/persona-r5-explorer/shots/008.png
status: new
