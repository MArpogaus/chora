# m5-mouse (mc), simulated user
goal: reached partly, tries 6 (1: clicked headings and page links; saw Agenda, The Morning and Agents pages; purpose still unclear, reads like a daily briefing board with columns, but no label says so)
goal: reached partly, tries 6 (2: forward to The Morning and Agents by clicking the bottom links worked; no way back found by mouse)
goal: reached no, tries 4 (3: single and double click on "Chat with a helper" only put a caret mark beside it; chat did not open)
goal: reached no, tries 2 (4: "Nothing new." under NOTIFICATIONS is not clickable and shows no list)

## No mouse way back from page 2 (The Morning)
kind: issue
persona: Ruth, 66, retired nurse, tech affinity 1, mouse only
flavor: mc
severity: high
steps: clicked "2 The Morning" link at bottom right to go to page 2; then clicked the small "2 THE MORNING" label at top left, the left margin, the big "2" numeral, and dragged left to right across the page
expected: a visible control (arrow or "1 Agenda" link) to return to the previous page
actual: page 2 has no back control. Bottom bar shows only "→ 3 Agents". Top-left label, margin, numeral and drag do nothing. Only the keyboard (Esc, Super+arrows) seems to go back, and the keys list is not reachable by mouse.
evidence: shots/005.png
status: new

## Chat with a helper does not open by mouse
kind: issue
persona: Ruth, 66, retired nurse, tech affinity 1, mouse only
flavor: mc
severity: high
steps: single-clicked "Chat with a helper ..." in the CHAT area of the right column, then double-clicked it
expected: the chat with the helper opens
actual: a thin caret mark appears to the left of the entry and nothing else happens. The only chat shortcut is the Super+a hint, which needs a keyboard.
evidence: shots/009.png
status: new

## Chat entry has no mouse hint or button
kind: ux
persona: Ruth, 66, retired nurse, tech affinity 1, mouse only
flavor: mc
severity: medium
steps: looked for a button or link for the helper chat on the first page
expected: a visible button or clear link labelled for the chat
actual: the only way shown is a key hint "Super+a" in small grey text at the right of the entry. It looks like a label, not a button, and I could not tell how to use it without a keyboard.
evidence: shots/001.png
status: new

## Notifications area is not clickable and says nothing
kind: ux
persona: Ruth, 66, retired nurse, tech affinity 1, mouse only
flavor: mc
severity: medium
steps: clicked the NOTIFICATIONS heading and the "Nothing new." text on page 1
expected: a list of notifications, or a way to open them
actual: nothing changed. "Nothing new." is the only text, with no link, count, or badge. I could not find notifications by mouse.
evidence: shots/011.png
status: new

## Purpose of the screen is not stated
kind: ux
persona: Ruth, 66, retired nurse, tech affinity 1, mouse only
flavor: mc
severity: medium
steps: clicked the AGENDA heading, then the page links at the bottom
expected: a short sentence or heading that says what the screen is for, for example a daily plan or a helper
actual: page 1 shows AGENDA, "Nothing planned today.", NOTIFICATIONS, CHAT and a keys list. The keys list is the only explanation. The page sequence (Agenda, The Morning, Agents, Mission Control) is shown only as small grey text at the bottom, so I could not tell what the pages hold.
evidence: shots/001.png
status: new

## Clicking the AGENDA heading does nothing
kind: ux
persona: Ruth, 66, retired nurse, tech affinity 1, mouse only
flavor: mc
severity: low
steps: clicked the large AGENDA heading on page 1
expected: the heading opens something, or nothing looks clickable
actual: no visible change. The heading is coloured like a link but does nothing.
evidence: shots/002.png
status: new

## Page-turn animation leaves text overlapping
kind: issue
persona: Ruth, 66, retired nurse, tech affinity 1, mouse only
flavor: mc
severity: low
steps: clicked "2 The Morning" at the bottom and took a screenshot straight after
expected: a clean page change
actual: for a moment the old heading "AGENDA" and the new "MORNING" overlap, and the top labels overlap too. It settles after about two seconds.
evidence: shots/003.png
status: new

## Agents page is empty
kind: ux
persona: Ruth, 66, retired nurse, tech affinity 1, mouse only
flavor: mc
severity: low
steps: clicked "→ 3 Agents" at the bottom of page 2
expected: some content or an explanation of what agents are
actual: the page shows only the heading "3 AGENTS" and nothing else. The bottom bar shows "→ 4 Mission Control".
evidence: shots/012.png
status: new
