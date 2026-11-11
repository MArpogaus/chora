# Design per look: mc (draft)

Draft for the PM and the user, 2026-11-11. Base: `docs/design-principles.md` ("P" + number),
`shared.md` and `docs/mc-style.md` (type, colour, space).

## People and goals

- **Living-room users**: watch, listen, look something up, from the sofa; often several people.
- **Hands-busy people** (the nurse, someone cooking): voice only; hear what helpers do, know when a
  hand is needed.
- **Beginners**: one remote, no keyboard; every task reachable with arrows, OK and Back.

## Usage patterns

Distance 2-4 m from a TV, or a big screen across a room. Input: remote or gamepad (arrows, OK,
Back), voice first, a keyboard rarely. Sessions are long and passive (a film), interrupted by short
asks. Text is read at a glance, not studied.

## Interaction model

A calm book page per scene. Arrows move one focus mark between items; OK acts; Back is Escape.
Nothing needs a pointer, and nothing needs two hands. The page never scrolls on its own.

## The helper and the chat: the top-right line and the drawer

One surface: **the helper line in the top-right region, which expands into the drawer** at the
bottom of the page.

| State | What shows |
|---|---|
| idle | top right: the state word in `accent` ("Listening"), Inter, step -1 |
| speaking | the line shows the sentence under the state word, at most two lines, step 0 |
| helpers working | under the state word: "Atlas · working · Write file (2/2)", one row per helper, max 3 |
| question | the shared card under the state word, top right, width min(560, output - 64) |
| chat open | the drawer: full text-column width, 14 lines, hairline above, no box |
| muted | the fixed host badge under the state word: "Microphone off. Press OK to turn it on." |

- **Collapsed to expanded.** OK on the helper line, a click, or "show the chat" opens the drawer;
  Back closes it and keeps the draft. The drawer is mc-style: Pagella body, Inter labels.
- **Replies and questions show in the drawer** while it is open; the top-right line then shows only
  the state word (U3: mc must not repeat the sentence top right while the drawer is open).
- **Typing** happens rarely; voice is the main way to write to a helper here.

## Voice

The main input. Speak more than in other looks: a result is read out, because the screen is far
and the person may not look. A physical question says how to answer with the remote. The state word
is always visible, so a person across the room sees if it hears them.

## Motion and sound

Fades only, 160 ms in, 120 ms out; the page never jumps while a film plays. No sound beyond voice's
earcons; during playback the earcons duck the film, never stop it.

## Density and type

Few items per page, large type. Five steps of the mc scale only (mc-style rule 2). Text read from
the sofa must be large enough at 3 m; the present step 0 (18 px at 1080) is likely too small.
Contrast 4.5:1 on the page in both themes; light `muted` for helper text is [0.43, 0.41, 0.38].

## Do

- Reach every item with arrows, OK and Back; show the focus mark clearly.
- Keep the page still while media plays; let the helper line change in place.
- Read results aloud.

## Don't

- Add a box, fill, shadow or second accent for the helper (mc-style rules 7 and 11).
- Show the sentence top right and in the drawer at once.
- Ask for a key the remote does not have.
- Show a light chat card on the dark page, or a black one on the light page (U2).

## Review checklist (mc)

1. Helper line on the baseline grid, flush with the top-right region's edge.
2. One place per sentence: top right or drawer, never both.
3. Drawer on the text column's left edge; hairline above; whole lines only, no cut line.
4. Captures at 1920x1080 and 1280x720, light and dark; chat colours follow the theme.
5. Every text is a named style; no third family; Pagella never below step 0.
6. From the sofa: a 1080 capture viewed at a quarter size still reads (stand-in for 3 m).
7. Remote-only run: every helper task reachable with arrows, OK and Back.

## Open questions

- **What is the physical answer on a remote?** Alt+Y does not exist there. OK on Yes is a press,
  but focus must not rest on Yes by default. Options: a dedicated button, a long OK press, or no
  risky questions in mc. User and security view decide.
- What size is step 0 for a TV at 3 m (24 px at 1080 is a common rule)? Does mc know it is a TV?
- Does the drawer open on every scene or only from the agents scene?
- Does a question pause playback?
