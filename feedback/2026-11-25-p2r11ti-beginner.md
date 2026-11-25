# p2r11 beginner (tiling), round 11 on 79ed970f

Run: one session tiling 1280x800, haiku persona, PERSONA_CHECK=1, helper transcript seeded, one `begin` per goal. Caps T1 5, T2 4, T4 5, T6 6 (20); used 5 (T1), 2 (T2), 3 (T4), 6 (T6) = 17 actions incl. 9 looks, no failed action. Contracts: T1, T2, T4:power, T6 pass. Breaches outside known-red: 0.
Persona says: T1 reached (Apps door, click Foot), T2 reached (Keys button), T4 reached (power icon, Escape), T6 not reached (reads "Nothing found" as failed start).
Captures: media/2026-11-25/persona-tiling-r11-beginner, media/2026-11-25/persona-tiling-r11-checks/beginner-*.txt.

## ux: ti-50 holds: the Apps list has "Chat with a helper (atlas)" as its first entry
kind: ux
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: low
task: T1
check: none
steps: look, click Apps, look (list), click Foot
expected: the helper row stands first and is preselected
actual: first row "Chat with a helper (atlas)", not used
evidence: media/2026-11-25/persona-tiling-r11-beginner/shots/002.png
status: new

## ux: an unknown program name: "Nothing found" only after Return, the launcher stays open, read as a failed start
kind: ux
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: low
task: T6
check: none
steps: Super+r, look, type frobnicate, look (empty list), Return, look (6 of 6)
expected: an error or a message that the program cannot start
actual: contract pass (Nothing found row); persona "not reached", launcher left open (cap reached before Escape); ti-31, ti-49 known
evidence: media/2026-11-25/persona-tiling-r11-beginner/shots/008-009.png
status: new

## ux: the power menu preselects Lock; the key list is dense and small
kind: ux
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: low
task: T4
check: none
steps: T2: Keys button; T4: power icon, look, Escape, look
expected: a cancel is as easy as an open
actual: Lock highlighted, Return would lock; Escape closed it; the key list spellings (Super+. Super+,) are hard to read at the capture size; T4 started with the terminal showing (the key list was no longer open, ti-27 holds)
evidence: media/2026-11-25/persona-tiling-r11-beginner/shots/004-007.png
status: new

## ux: the terminal fills the screen and shows no frame (single window)
kind: ux
persona: beginner, affinity 0, mouse first
flavor: tiling
severity: low
task: T1
check: none
steps: T1 click Foot, look
expected: a window with a frame
actual: one tiled window without decoration: the persona was unsure it is a separate window; locale warning in the first line
evidence: media/2026-11-25/persona-tiling-r11-beginner/shots/003.png
status: new
