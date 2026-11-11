# Design per look: xp (draft)

Draft for the PM and the user, 2026-11-11. Base: `docs/design-principles.md` ("P" + number) and
`shared.md`. The xp look has no style guide yet (P8 asks for one); this page is not that guide.

## People and goals

- **Nostalgic users** who knew XP: it must feel exactly right; one detail off (a stretched figure,
  a cut balloon, a wrong font) breaks the whole look.
- **Beginners** (Erika, 74): start a program, find a window, write to the helper, change the volume,
  all from what they see. They read slowly and click twice when nothing answers. **Children** talk
  to the figure and must not allow what a parent must allow.

## Usage patterns

Desk distance (50-70 cm), mouse first, slow typing, short sessions most days, 1280x720 to 1920x1080.
Start menu, taskbar and tray are the map; nobody learns keys here.

## Interaction model

The XP desktop as remembered: Start, taskbar, tray, title bars, double-click. Every task has a
visible door (P3); keys are extras. The paperclip is the one door to the helper (no "Chat ..." pill).

## The helper and the chat: one balloon

One surface: **the paperclip and its one balloon**. Nothing else shows the helper in xp.

| State | What shows |
|---|---|
| idle | the paperclip alone, idle pose; no balloon, no tray balloon |
| speaking | the balloon (collapsed) with the sentence; stays 4 s after the end, then fades |
| helpers working | the balloon (collapsed), one line "Atlas is working (2/2)"; a click expands it |
| question | the card inside the balloon, whole; no field and no keyboard taken until a click |
| chat open | the balloon expanded: header (name, state, Stop, close x), transcript, field |
| muted | the balloon stays with one line, "Microphone off. Click to turn it on."; a click on it unmutes |

- **Collapsed to expanded.** A click on the balloon or on the paperclip expands the same balloon
  into the chat, in place: same tail, same paperclip, it grows up and to the left. Esc or the close
  box collapses it. No second window, no panel, no tray balloon while it is open (6.15).
- **Replies and questions show inside it.** A reply is a line in the transcript under the helper's
  name. A question is the shared card at the bottom of the transcript, above the field.
- **Nothing sits beside it.** No sentence bubble next to an open chat, no row list top right, no
  pill, no notice over it. A notice for the helper is a line inside the balloon.
- **The paperclip is never stretched or clipped.** Its art keeps its exact aspect ratio and shows
  at its native size times a whole number (output scale 1 or 2), never a fraction. Its whole box
  stays inside the work area, 8 px above the taskbar and 8 px from the right edge. The balloon never
  covers it, the taskbar never covers it, and its pose change never moves its foot by a pixel.
- **The balloon** is the XP balloon: pale yellow fill (#FFFFE1), 1 px dark border, round corners,
  the tail from the balloon edge to the paperclip's head. Width 340, height up to 60 % of the work
  area; the transcript scrolls inside. Its field is an XP edit box (1 px #7F9DB9). It never covers
  the taskbar (U2) and never leaves the work area; on 1280x720 it shrinks its height, not its width.

## Voice

The paperclip is the voice: its pose names the state (listening, thinking, speaking, question), and
the state word shows in the tray icon's tooltip. Speech is short and friendly; the screen holds the
detail. A spoken "yes" to a physical question gets "For this I need a key press or a click."

## Motion and sound

Poses change by the art's own frames, never by scaling or sliding the figure. The balloon fades in
over 160 ms and expands in one step, no elastic grow. No sound beyond voice's earcons.

## Density and type

Tahoma 14 px for every text a person reads (balloon, card, chat), even where XP used 8 pt (11 px).

## Do

- Keep every XP control in its XP place and size (title bars, Start, taskbar); keep one balloon.
- Show the question card whole inside the balloon; scroll only the detail.
- Name every state in words (tray tooltip, balloon line).

## Don't

- Stretch, crop, rotate, scale by a fraction, or blur the paperclip.
- Show a second bubble, a tray balloon or a "Chat ..." pill while the balloon is open.
- Put the field or the balloon over the taskbar or off the screen.
- Show internal words: "wpctl not found", "Foot Client", workspace numbers without labels.

## Review checklist (xp)

1. Paperclip: aspect ratio exact (measure width/height in the 1x and 2x capture); whole box inside
   the work area; foot at the same pixel in every pose.
2. One balloon only, in every state; no pill, no second bubble, no tray balloon while expanded.
3. Balloon tail touches the paperclip's head; border unbroken; nothing cut at any edge.
4. Balloon and field above the taskbar at 1280x720 and 1920x1080.
5. Question card whole inside the balloon, Yes left, No right, key line readable.
6. Text 14 px or more in Tahoma; contrast 4.5:1 (7:1 for the key line) on #FFFFE1.
7. Start, taskbar and tray: XP spacing, no internal words, volume reachable by a click.

## Open questions

- Does a click on the paperclip open the balloon (Clippy's habit) or mute (shared F5)? This draft
  assumes "open"; then mute moves to the paperclip's right-click menu and the tray. User decides.
- Where does the idle state word live: the tray tooltip (this draft), or a short balloon on hover?
- Does the whole xp chrome scale up (for example 1.25) so that 14 px text fits XP proportions, or
  only the helper's text?
- XP system sounds (start, ding): off, on, or a setting? P6 says nothing unasked.
- Who writes `docs/xp-style.md`, and when (P8)?
