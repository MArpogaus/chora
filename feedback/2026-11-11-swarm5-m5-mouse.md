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
status: verified (shot 005 read: footer holds only the next entry, mc.yaml:267 content next; no previous link; mouse-only user cannot go back)

## Chat with a helper does not open by mouse
kind: issue
persona: Ruth, 66, retired nurse, tech affinity 1, mouse only
flavor: mc
severity: high
steps: single-clicked "Chat with a helper ..." in the CHAT area of the right column, then double-clicked it
expected: the chat with the helper opens
actual: a thin caret mark appears to the left of the entry and nothing else happens. The only chat shortcut is the Super+a hint, which needs a keyboard.
evidence: shots/009.png
status: rejected: env, same cause as Super+a: no voice backend in the persona session; the row click is implemented (chat.rs:1599 a_click_on_the_chat_row_opens_the_chat_with_a_token)

## Chat entry has no mouse hint or button
kind: ux
persona: Ruth, 66, retired nurse, tech affinity 1, mouse only
flavor: mc
severity: medium
steps: looked for a button or link for the helper chat on the first page
expected: a visible button or clear link labelled for the chat
actual: the only way shown is a key hint "Super+a" in small grey text at the right of the entry. It looks like a label, not a button, and I could not tell how to use it without a keyboard.
evidence: shots/001.png
status: rejected: not shown, the row is the button and its click works with a backend (chat.rs:1599); the tester could not see it only because no chat backend ran

## Notifications area is not clickable and says nothing
kind: ux
persona: Ruth, 66, retired nurse, tech affinity 1, mouse only
flavor: mc
severity: medium
steps: clicked the NOTIFICATIONS heading and the "Nothing new." text on page 1
expected: a list of notifications, or a way to open them
actual: nothing changed. "Nothing new." is the only text, with no link, count, or badge. I could not find notifications by mouse.
evidence: shots/011.png
status: duplicate of Notifications panel shows labels but no content

## Purpose of the screen is not stated
kind: ux
persona: Ruth, 66, retired nurse, tech affinity 1, mouse only
flavor: mc
severity: medium
steps: clicked the AGENDA heading, then the page links at the bottom
expected: a short sentence or heading that says what the screen is for, for example a daily plan or a helper
actual: page 1 shows AGENDA, "Nothing planned today.", NOTIFICATIONS, CHAT and a keys list. The keys list is the only explanation. The page sequence (Agenda, The Morning, Agents, Mission Control) is shown only as small grey text at the bottom, so I could not tell what the pages hold.
evidence: shots/001.png
status: duplicate of mc first screen shows no hint and no key reacts (the key list now exists; the missing purpose line is the part left)

## Clicking the AGENDA heading does nothing
kind: ux
persona: Ruth, 66, retired nurse, tech affinity 1, mouse only
flavor: mc
severity: low
steps: clicked the large AGENDA heading on page 1
expected: the heading opens something, or nothing looks clickable
actual: no visible change. The heading is coloured like a link but does nothing.
evidence: shots/002.png
status: verified (shot 002 read: the heading is in the accent colour like a link and has no action; low)

## Page-turn animation leaves text overlapping
kind: issue
persona: Ruth, 66, retired nurse, tech affinity 1, mouse only
flavor: mc
severity: low
steps: clicked "2 The Morning" at the bottom and took a screenshot straight after
expected: a clean page change
actual: for a moment the old heading "AGENDA" and the new "MORNING" overlap, and the top labels overlap too. It settles after about two seconds.
evidence: shots/003.png
status: rejected: harness timing, the shot is 60 ms into the page turn (003), the settled page is clean (005); no defect at rest

## Agents page is empty
kind: ux
persona: Ruth, 66, retired nurse, tech affinity 1, mouse only
flavor: mc
severity: low
steps: clicked "→ 3 Agents" at the bottom of page 2
expected: some content or an explanation of what agents are
actual: the page shows only the heading "3 AGENTS" and nothing else. The bottom bar shows "→ 4 Mission Control".
evidence: shots/012.png
status: rejected: env, the agents scene is empty without an agent (agent.rs:465); no voice backend ran
