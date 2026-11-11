# Design per look: tiling (draft)

Draft for the PM and the user, 2026-11-11. Base: `docs/design-principles.md` ("P" + number) and
`shared.md`. The tiling look has no style guide yet (P8).

## People and goals

- **Power users** (Jonas, 31): keyboard first; open a chat, follow three helpers and leave again
  without the mouse; never lose the keyboard to something they did not start.
- **Ricers** (r/unixporn): change every colour and font with plain files, see errors at once,
  share the result.
- **Beginners who landed here**: must still find a visible way in (A3: "Chat ..." in the bar).

## Usage patterns

Desk distance, keyboard first, the mouse sometimes; long sessions every day, many windows, often
two screens. Captures and screenshots get posted, so every pixel is public.

## Interaction model

Keys first, all of them shown somewhere (P3: the help, `Super+?`). Tiles fill the work area; the
bar is the only chrome. A key does one thing and never types into a window when unbound (M5).

## The helper and the chat: the bar segment and its panel

One surface: **a segment at the right of the bar that expands into a side panel**.

| State | What shows |
|---|---|
| idle | the bar segment: a dot and the state word, "Listening" |
| speaking | the segment shows the sentence, one line, cut at its start with an ellipsis; 2 s after |
| helpers working | the segment adds "Atlas · working (2/2)"; more helpers: "3 helpers working" |
| question | the shared card hangs under the segment, top right; it takes no keyboard (P11) |
| chat open | the side panel: header (name, state, Stop, Close), transcript, field |
| muted | the segment shows "Microphone off" with the crossed mic; a click turns it on |

- **Collapsed to expanded.** `Super+a` or a click on the segment opens the panel under it at the
  right edge, 400 wide, full work-area height; the field has the keyboard (with the token of that
  act). Esc closes it and gives the keyboard back to the window of before.
- **Replies and questions** show in the panel. A question while the panel is closed shows as the
  shared card hanging from the segment, top right, and stays there until it ends.
- **Nothing beside it.** No dot at the bottom right, no rows top right, no floating sentence: the
  segment and the panel are the whole helper.

## Voice

Brief: one sentence, the detail on screen. Results of a helper with an open chat are not spoken
(only questions). A keyboard mute exists (`MOD+Shift+a`, 6.12) and the segment shows it.

## Motion and sound

The panel appears in one step or slides in over at most 120 ms, no bounce; tiles never move
because of it. No sounds beyond voice's earcons.

## Density and type

Desk distance, dense is fine: one sans family plus one mono, 14 px body, line 20. The bar is 24 high.
Contrast 4.5:1 for text, 7:1 for the key line. The bar shows tasks only: no IP address, cpu or
memory by default (ux-loop-1, 12); a ricer adds them.

## Do

- Bind every act to a key, and list every key in the help.
- Keep the panel's width fixed so the transcript never reflows while it is read.
- Let a ricer theme the bar and the panel from one folder, values only.

## Don't

- Take the keyboard without a person's act; leave a typed key in the window after `Super+a` (B4).
- Let an unbound Super key type text into a window (M5).
- Draw the bar of tiling on an output that shows another look (M4).
- Show internal words in the bar ("vol: wpctl not found", "1[tile]").

## Review checklist (tiling)

1. The segment sits on the bar baseline, aligned with the other modules; its dot is centred.
2. The panel's top edge meets the bar's bottom edge with no gap and no overlap.
3. Keyboard: `Super+a` gives the field the keys at once; Esc gives them back; nothing leaks.
4. The card under the segment never covers the segment, and stays put while the panel opens.
5. Text 14 px or more; one sans, one mono; widths and gaps on one grid.
6. Bar: no internal words, every module labelled; the tag reads "1", not "1[tile]".
7. Two screens: each output shows only its own look's bar and helper segment.

## Open questions

- Does the panel float over the tiles (today) or shrink the work area so nothing is covered?
- Does a question with a closed panel open the panel (more context) or only hang the card?
- Which key opens the help (`Super+?` here is a proposal), and is it the same in xp and mc?
- Should tiling get a light theme, as mc and construct have?
