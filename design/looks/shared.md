# Design per look: shared invariants (draft)

Draft for the PM and the user, 2026-11-11. `docs/design-principles.md` stays the base ("P" +
number). This page names what every look keeps identical, so a person who learned one look is not
lost in another (P1, P8), and what a look may change. The per-look pages (`xp.md`, `tiling.md`,
`mc.md`, `construct.md`) may only add to this page, never contradict it.

## Must stay identical in every look

**Words (P4, P7).** One table, `chora-kit::words`, en and de; German uses "Sie". "helper", never
"agent", "ACP" or "HUD". State words: Asleep, Listening, Thinking, Working, Speaking, Question for
you, Voice off, Microphone off. A look never rewords, abbreviates or translates on its own.

**Honest states (P4).** Every look shows every state of the table in `ux-hud-sprint.md` section 3:
idle, busy, done, failed, refused, offline, question waits, muted. The state always has a word; a
colour, a ring or a pose alone is never enough. A failure names the thing and the next step. No
look claims "offline" when voice is up (persona-3, B2) or stays silent on a refusal (ux-loop-1, 2).

**One surface per helper conversation (U3).** Each look has exactly one helper surface per output,
with a collapsed and an expanded form of the same surface. While it is expanded, the sentence, the
replies and the question live inside it; no second balloon, line or row repeats them.

**The question card (P5).** Same content and order everywhere: title "Question from Atlas", time
left, the prompt, the whole detail (scrolls, never cut), the key line, the line "A spoken 'yes' does
not count." for a physical question, then Yes left, No right, 8 apart, at least 32 x 24.
- Keys: Alt+Y allows, Alt+N refuses, from any window, also while the chat field has the keys.
- A click counts only 400 ms after a button appeared or moved; an injected press never answers
  ("Please use your own mouse or keyboard."); typing never answers; a spoken "yes" never answers a
  physical question and gets "For this I need a key press or a click."; a spoken "no" refuses.
- The card stays where it appeared until it ends; it never moves, and it shows only once.
- Only voice's `question` entries draw a card. Agent text never draws one, and nothing a third
  party supplies (folder, agent message, window) can cover, hide or imitate it.

**Escape (P1).** Escape backs out one step: closes the expanded helper surface (the draft stays),
leaves a menu or a list, steps back from a pane in construct (Alt+Escape always). Enter sends,
Shift+Enter breaks a line, Stop cancels the running prompt; the helper goes on after a close.

**Give and move keys (P1).** Super+Alt+1 to Super+Alt+4 give the focused window to tiling, xp, mc
and construct, in every look. Super+drag moves a window, and a drop on another look hands it over.
`Super+a` opens the chat wherever a keyboard exists. Every one of these keys is listed in each
look's help, and an unbound Super key never types into a window.

**Keyboard and focus (P11).** The focus moves only after a person's act (click, key, spoken
request, a program they started). The chat field takes the keyboard only with the activation token
of that act. A question, a notice, a context switch or a helper never takes the keyboard.

**Mute.** Every look shows a visible mute and a visible unmute; the host's unmute answers only a
real click or key. A muted microphone is always visible on every output.

**A visible way in (P3).** Every look shows a door to the chat without a key: the "Chat ..." row,
the segment, the line, the pill or the paperclip. The launcher of each look has "Chat with a
helper".

**Readable (P7).** Text a person must read: 14 px or more (mc: its own larger minimum at TV
distance). Contrast 4.5:1, key line and buttons 7:1. The state word may be 11 px only next to a
shape that shows the same state.

**Security (P5, P10, P11).** Agent text is sanitised (no control, bidi or escape characters, no
links, no markup). A folder carries values only, never code; `--check-hud` refuses what breaks
these rules. The chat field cannot answer a question, change grants or claim the indicator.

## What a look may vary

| May vary | May not vary |
|---|---|
| place of the helper surface and its collapsed form | that there is exactly one surface |
| fonts, colours, shapes, corners, theme (light and dark) | words, state names, key names |
| motion within 120-240 ms fades, ease out | order of the card, Yes left / No right |
| density and type scale, above the shared minimum | the minimum size and contrast |
| how much voice says (mc more, tiling less) | what counts as an answer |
| extra keys and extra doors | the shared keys, Escape, the give keys |
| whether a question opens the expanded surface | that the card stays where it appeared |

## Open questions

- German keys: Alt+Y or Alt+J (ux-hud-sprint 6.13)? The key line shows the keys of the event, so
  either works; the user decides the default.
- mc: what is the physical answer on a remote (see `mc.md`)? It may need a shared rule here.
- Is `Super+a` an invariant or a tiling key? xp and mc users may have no Super habit.
- Should a look be allowed to hide the helper surface entirely (a "no helper" setting), with the
  launcher as the only door?
