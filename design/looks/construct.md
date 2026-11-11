# Design per look: construct (draft)

Draft for the PM and the user, 2026-11-11. Base: `docs/design-principles.md` ("P" + number),
`shared.md`, `docs/construct-style.md` (glass, sizes, colour, motion) and `docs/cube-modes.md`.

## People and goals

- **Explorers and demo watchers**: walk a 3D space, fly into a pane, show it to others; delight
  matters, but a lost window or a silent key spoils it at once.
- **Ricers** (Max): push it, break it, ship their own look.
- **xp users inside the cube**: work on a face at 1:1 as if it were xp, step back to turn it.

## Usage patterns

Desk distance, keyboard and mouse together (walk keys plus pointer), sometimes voice for
"step back" or "turn left". Medium sessions, often for a demo or a recording; 1280x720 and
1920x1080, light and dark. The view moves, so anything fixed to the screen must stay still.

## Interaction model

Two modes: walk (navigate) and use (flown into a pane or a cube face at 1:1). Walk keys and `t`
act only in walk mode; in use mode every key goes to the pane. Every chip carries its key
(`STEP BACK  Alt+Esc`). Text only on glass (style rule 11); nothing is drawn on the bare space.

## The helper and the chat: one glass panel

One surface: **a glass panel at the top right, 32 from the edges**, screen-fixed (style rule 9).

| State | What shows |
|---|---|
| idle | a glass pill: the state ring and the state word as a label, "LISTENING" |
| speaking | the pill grows into a panel with the sentence, `text` 16, at most two lines; fades 2 s after |
| helpers working | the pill adds a `muted` row "Atlas · working (2/2)"; a click expands it |
| question | the shared card, glass, under the pill; it stays until it ends |
| chat open | the panel expanded: 480 wide, down to 32 above the bottom; header, transcript, field |
| muted | the fixed host badge inside the pill: "Microphone off. Click to turn it on." |

- **Collapsed to expanded.** A click on the pill, `Super+a` or "show the chat" expands the same
  panel downward; the ring and state word stay at its top. Esc collapses it to the pill.
- **Replies and questions show inside it** while it is open (U3: no floating sentence balloon apart
  from the open chat).
- **In use mode** (flown into a pane) the pill stays, at reduced opacity, and every other chip
  hides (ux-loop-1, 6); the expanded panel covers the pane only while the person has it open.
- Notices are one line of text on glass, bottom centre, whatever `voice_hud` says (ux-loop-1, 1).

## Voice

Voice moves the view: "step back", "step in", "turn left", "show the windows". It answers in one
sentence. The state word always names the state (style rule 14); the ring alone is never enough.

## Motion and sound

Construct-style rule 12: fades 160 ms in, 120 ms out, ease out; ring turns 1.2 s and 2.4 s; the
question pulse is the only loop. Screen-fixed glass never moves with the camera. Fly-in drops no
typed key. No sounds beyond voice's earcons.

## Density and type

Three sizes (11 label, 12 data, 16 text). Every text a person must read is `text` 16 or the 14 px
of the shared card; the 11 px label only names a state that also has a shape. A new pane opens at
the 1:1 distance so its text is readable (ux-loop-1, 10).

## Do

- Put every chip in its own corner, 32 from the edges, never two in one corner.
- Hide chips that do not apply (cube rows without a cube, OPEN CUBE in a pane).
- List every window in the window list, marking hidden ones "not in view".

## Don't

- Draw text on the bare space or on a pane; let a label sit on another pane or a face's taskbar.
- Show two voice indicators (the LISTENING card and a green dot).
- Use a light chat on the dark theme, or a plain box instead of glass.
- Let a walk key spawn a pane silently when the person meant to type.

## Review checklist (construct)

1. Glass: radius 12, hairline 1, glow fades within 16; the text stays inside the padding of 16.
2. Every chip and panel 32 from the edges; nothing touches or crosses an edge (OPEN CUBE).
3. One helper surface; the sentence never shows twice.
4. Over a white and a black pane, light and dark: contrast per style rule 5.
5. Clip of a walk and a fly-in: screen-fixed glass does not jitter by a pixel; labels hide while
   a pane moves; no frame flashes at the end of a fly-in.
6. Pane labels use the window title, sit 8 below the pane, never on a face's taskbar.
7. Help lists only rows that apply now, each once, none cut.

## Open questions

- Top right (this draft) or top centre (today's voice HUD) for the one glass panel?
- In the xp cube in use mode, which helper shows: the face's paperclip or the construct's pill?
  See `combinations.md`.
- Should the expanded chat be a pane in the space (walkable) instead of screen-fixed glass?
- Does the cube need a visible way in (an OPEN CUBE chip) for every person, or only a setting (M7)?
