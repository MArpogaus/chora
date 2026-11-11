# m4-mouse (mc), simulated user
goal: reached partly, tries 1 (screen identified only as an AGENDA page; purpose not clear)
goal: reached no, tries 6 (no clickable thing found to open a program)
goal: reached no, tries 3 (no way to another screen found; scroll and right-click did nothing)
goal: reached no, tries 0 (no notes or calendar reachable, so no entry added)

## No visible way to do anything on the only screen
kind: issue
persona: Ruth, 66, retired nurse, tech affinity 1, mouse only
flavor: mc
severity: high
steps: Looked at the screen, then clicked the AGENDA heading, the small "1 AGENDA" label at top left, empty middle, bottom edge, and the date at top right. Double-clicked the big "1". Right-clicked empty right side. Scrolled down 5.
expected: Something on the screen looks like a button, link, menu or tab that opens a program or another screen.
actual: Nothing changed after any of these actions. No buttons, menus, icons or context menu appeared. The screen stayed identical.
evidence: shots/001.png
status: duplicate of mc first screen shows no hint and no key reacts (shot 001 read: AGENDA page only)

## Screen purpose is not explained
kind: ux
persona: Ruth, 66, retired nurse, tech affinity 1, mouse only
flavor: mc
severity: medium
steps: Looked at the first screen and every later screen after clicking.
expected: Some label or hint saying what this screen is for and what can be done here.
actual: The only content is a small "1 AGENDA" label, a large "1 | AGENDA" heading, and a date and time at top right. The rest of the screen is empty dark background.
evidence: shots/001.png
status: duplicate of mc first screen shows no hint and no key reacts

## Clock appeared frozen at 17:33
kind: issue
persona: Ruth, 66, retired nurse, tech affinity 1, mouse only
flavor: mc
severity: low
steps: Compared the top-right date and time across the screenshots taken over the session (shots 001 to 006).
expected: The time advances between screenshots taken minutes apart.
actual: It read "WEDNESDAY 11 NOVEMBER 17:33" in every screenshot. I cannot tell from here whether the session was that short.
evidence: shots/006.png
status: rejected: env, not a bug. The whole m4 session lasted 25 s (actions.jsonl 8401 to 8426) and the clock has minute resolution, so 17:33 cannot change; chora-mc ticks the clock via CLOCK timer, plugins/chora-mc/src/main.rs:55 and :1278-1283 (shot 006 read)

## Nothing signals that the screen is usable by mouse
kind: ux
persona: Ruth, 66, retired nurse, tech affinity 1, mouse only
flavor: mc
severity: medium
steps: Tried left click, double click, right click and scroll on the page, with no keyboard use.
expected: At least a hover change, a cursor change or a response to a click, so a mouse user knows the screen responds.
actual: No hover or click feedback was visible in any screenshot, so I could not tell if the screen responds to the mouse at all.
evidence: shots/006.png
status: duplicate of mc first screen shows no hint and no key reacts (mc is keyboard driven; no pointer feedback in shot 006)
