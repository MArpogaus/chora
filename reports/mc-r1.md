# mc r1 (rc 1, 11): journey 1, BLOCK and MAJOR of chora-mc

Branch claude/mc (from caf9c80d), gate tests/gate-mc-sofa.sh (new, in merge-gate step `mc-sofa`).
Captures: media/2026-11-16/mc-r1/ (light and dark, 1280x720).
New cells: 10 of 10 runs pass (5 dark, 5 light). Merge gate on c2c9c353: passed once.
Stopped at the context limit: mc-34 is open, mc-35 is half done.

| id | verdict | red | fix | gate cell, capture |
|---|---|---|---|---|
| mc-32 BLOCK | real | cb553eec, 37d0803b | 9505757c | `remote`, mc-sofa-remote-{dark,light} |
| mc-37 | real | d4938d1c | 309ae998 | `wide`, mc-sofa-wide-{column,page}-{dark,light} |
| mc-36 (overlap) | real | 34587eeb | 1eb04373 | `status`, mc-sofa-status-{dark,light} |
| mc-36 ("No helper") | not a product fault, script | none | none | see below |
| mc-33 | not a product fault, script | dda31e6a (green) | none | `door`, mc-sofa-door-{dark,light} |
| mc-35 (empty NOTES) | real | 5a34fb82 | 097aaa9c | `empty-head` + unit test, mc-sofa-empty-head-{dark,light} |
| mc-35 (card, overflow) | open | | | |
| mc-34 | open | | | |

## mc-32 (BLOCK): real
The cell first ran in a bare session and passed. A session with the content that the tour sets with
`panel.set` failed: `Right` did nothing. Cause: a panel that fills or empties builds its page anew, the page node
held the keyboard, and nothing gave it back (`focused: []` after `panel.set`). Every collector (agenda.sh,
morning.py) does this, so the remote died a few seconds after the start. Fix: `replace_panel` takes
`take_free_keyboard` after such a rebuild. Red: gate cell `remote` (FAIL "Right does not turn to page 2: agenda")
and unit test `a_page_keeps_the_keyboard_when_a_helper_fills_a_panel` (left None, right Some("mc-s-agenda-page")).
The real tour script v4_mc2.py showed the same dead keys before the fix and was the reproduction.

## mc-37: real
`side_widths` read the sides from the rules and ignored the toggle, so the text never moved. A hidden
side has no width now and a toggle of a side builds all pages anew. Red: unit test
`the_page_widens_when_the_column_hides` and cell `wide` (672 then 672). Green: the text of the morning scene
widens (light capture: 160 to 1120) and goes back to 672 when the column returns.

## mc-36
Overlap: real. With a state word of mc (`hud.set`) the word `Mute` stood flush right on the first line, under
the pill of the host (Mute 1189..1224, pill 1115..1200). New placement `Put::After`: Mute follows the state word
beside the host. Cell `status`.
"No helper" while the helper answers: not a product fault. The host shows the state of the voice agent from
`voice.state` and `voice.say`; with no helper and a resting agent the word is `No helper` by design
(docs/hud.md). The tour script drives the answer through `hud.say` of mc, which the host never sees
(v4_mc.py/v4_mc2.py: `S.call("mc", "hud.set"/"hud.say", ...)`). With `voice.say` the pill says `Speaking`
(probe capture). The script should emit `voice.state` of a named helper and `voice.say` through the fake voice.

## mc-33: not a product fault
The chat door opens the chat by mouse (cell `door`, a helper row on the page, a press at the fresh
position; needs chora-chat built). In the tour the position of the door was read right after
`L.emit("voice.state", ...)` (v4_mc2.py, the `door("mc-r1-right-chat-door")` call after the emit): the helper row
came a moment later and moved the door down by 84 px, so the press hit "Nothing new". Script fix: wait for the text
`HELPERS` before `door()`.

## mc-35
Fixed: after Super+n twice the column counted as opened and drew the head of every text without lines
(an empty NOTES), which pushed the chat door down to the rule of the keys. Red: unit test
`a_column_that_came_back_shows_no_empty_head`, cell `empty-head`.
Open (not done): the critical notice as a card instead of the truncated footer line, and an overflow guard for
a column with a helper row and two long notices (Chat door at y 406 meets the rule). Both change the
footer line that gate-mc-paths `urgent` asserts, so they need that cell changed first.

## mc-34: open
The sentence and the heard words are drawn by the host (`plain()` in plugins/chora-assistant/src/assistant.rs,
`Font::Ui`, 11 px), not by mc; mc's own footer line only shows after a direct `hud.say`. A fix needs a size for the
sentence in the hud.yaml schema (crates/chora-kit/src/hud.rs, docs/hud.md) and the host. `said` of `voice.state`
is not drawn at all.

## Gate
10 runs of tests/gate-mc-sofa.sh (alternating dark and light): all ok.
tests/merge-gate.sh on c2c9c353: all steps passed once (run directory /tmp/merge-gate.5MW39K), including mc-paths and the new mc-sofa.
