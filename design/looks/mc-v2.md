# Design per look: mc v2, the page and its margin (proposal)

Proposal for the PM and the user, 2026-11-12. Builds on `mc.md`, `shared.md`, `docs/mc-style.md`
and `docs/design-principles.md` ("P" + number). Source read: build 06044cf. Mocks:
`media/2026-11-12/design/mc/` (HTML and PNG, 1280x800, labelled "mock").

User goal: "Mc shall rather include in the right margin where notifications are shown anyway
instead of stealing half the screen." So the helper, the chat and the question move into the right
margin column. Nothing in mc takes half the screen any more.

## Philosophy

mc is a printed page with marginalia. The scene is the text block; everything that comments on it
(helpers, notifications, the chat, a question) lives in one margin column on the right, as side
notes do in a well-set book. One serif to read, one sans for the frame, one accent, hairlines
instead of boxes, every line on one baseline grid. The page never moves for the margin: opening
the chat, a notification or a question changes the margin only. The one box in mc is the question
card, so that its rarity makes it a signal (P5, P6).

## What changes, in one table

| Today (06044cf, stills `03-*`) | v2 |
|---|---|
| Chat in the bottom drawer, 14 lines, up to half the output (`region.rs` `boxes`), boxed, bold | Chat in the right column, page untouched |
| "Chat ..." pill, a box top right (breaks mc-style rule 11) | the line "Chat with a helper ..." in the column |
| Helpers and state in the header, clock squeezed beside them | helpers and state at the top of the column; header holds running head and clock |
| NOTIFICATIONS label with nothing under it | "Nothing new." (honest empty state, P4) |
| First screen: "1 AGENDA", no hint, no key list, mouse does nothing | key list in the column, clickable; hover mark on every item |
| `Super+a` opens the agenda (clash with `shared.md`: chat) | `Super+a` opens the chat; agenda is `Super+Home` |
| `Super+Return` opens a terminal drawer over half the page | `Super+Return` shows scene 5 Terminal; the drawer leaves the default preset |

## Layout

Grid unit: one baseline line, 28 px (see Type). At 1280x800, from left to right:

| Part | x (px) | Width | Contents |
|---|---|---|---|
| gutter | 0 | 56 | nothing |
| margin notes | 56 | 76 | times, flush right (rule 9) |
| gap | 132 | 28 | |
| text block | 160 | 672 | header (running head left, clock right), scene head, scene, footer |
| gap, hairline | 832 | 56 | 1 px `faint` at x 860, y 28 to 772 (rule 11: a rule left of a panel) |
| margin column | 888 | 336 | helpers, notifications, chat, question, keys |
| gutter | 1224 | 56 | nothing |

- **Column width**: 12 grid lines (336 px), growing to 16 lines (448 px) at 1920 wide:
  `clamp(336, 26 % of output, 448)`, rounded to whole lines. About 38 Pagella characters a line.
- **Below 1100 px** the column takes the place of the text block's right part; under 900 px it
  becomes a layer over the page, opened with `Super+n` or a click on the state word (rule 13).
- **The page never reflows** when the column hides or opens: the text block keeps its width.
- **Column sections**, top to bottom, each starting with a running head (spaced Inter capitals):
  1. *State line* on the header baseline: sign and state word in `accent` ("Listening"), "Mute"
     flush right (shared: visible mute). Muted: "Microphone off" plus "Turn on".
  2. *Helpers*: one row each, sign in its state colour, name in `text`, state and task in `muted`;
     at most 4 rows, then "+2 more".
  3. *Notifications*: newest first, time in `margin` with tabular figures, text in Pagella;
     at most 3; "Nothing new." when empty. Urgent ones also stay in the footer as today.
  4. *Chat*: collapsed it is one line, "Chat with a helper ..." with `Super+a` flush right.
  5. *Keys* (option Q2 A): six rows, pinned to the column foot, above the bottom gutter.

**Collapsed** (mock 1): state, helpers, notifications, the chat door, keys. Nothing else.

**Expanded** (mock 2), by `Super+a`, a click on the door, OK on it or "show the chat":
- Notifications fold to their running head with the count ("NOTIFICATIONS 2", "Show" flush right).
- The key list hides (F1 still works); the chat takes all room from its head to the field.
- Messages: the speaker on its own line in `margin` ("You", "Atlas"), the text in Pagella step 0.
  Whole lines only; older lines leave at the top behind "↑ 3 earlier" (click or Page Up scrolls).
  The newest line stays above the field. A long reply wraps; it is never cut (rule 12).
- The field sits at the column foot on the footer baseline, a hairline above it, placeholder
  "Write to Atlas ...", "Enter" flush right. Escape closes the chat, the draft stays (shared).
- One place per sentence (U3): while the chat is open the state line shows only the state word.

**The question card** (mock 2) appears pinned directly above the field, full column width
(336 px), and stays there until it ends; history scrolls above it, never under it. Order as in
`shared.md`: "Question from Atlas", time left flush right, the prompt (Pagella), the detail
(`mono`, scrolls inside, max 6 lines), the key line, "A spoken 'yes' does not count.", then Yes
and No, 88x32, 8 apart. It is the only box in mc: 1 px rule in `text`, filled with the page
`background` (dark card on a dark page, U2). With the chat collapsed, a question opens the column's
chat section by itself (the card needs the room) but never takes the keyboard (P11).

## Visual system

**Palette.** Today's `mc.yaml` values, two fixes for contrast on the paper (measured, WCAG).

| Role | Light | Dark | Use |
|---|---|---|---|
| background | `#f7f5ed` | `#1c1c1c` | page |
| tint (Q1 B only) | `#efece3` | `#232322` | column ground |
| text | `#292624` (13.8:1) | `#d9d4c9` (11.5:1) | content, keys, card rule and buttons |
| muted | `#6e6961` (5.0:1, was `#807a73` 3.9:1) | `#8c8a82` (4.9:1) | labels, times, names |
| faint | `#d4cfc4` | `#424240` | hairlines only |
| accent | `#8c241f` (8.1:1) | `#cc7866` (5.2:1) | scene title, voice state; nothing else |
| green | `#407554` | `#85b88f` | working, done |
| amber | `#966214` (4.8:1, was `#b87814` 3.4:1) | `#e0b361` | question waits |
| blue | `#386194` | `#8cb0db` | listening, thinking |
| red | `#b3211f` | `#eb6b61` | failed, refused, urgent |

On the dark tint `#232322` muted needs `#9a978f` (5.4:1); one more reason to prefer Q1 A.

**Type.** TeX Gyre Pagella to read, Inter for the frame, `mono` for paths; one weight (Inter 500
only for region labels and key names). Scale base **19 px** (was 18), ratio 1.333: step -1 14.25,
0 19, 1 25.3, 2 33.8, 3 45. Reason: step -1 was 13.5 px, under the shared 14 px floor, and the
column's notes and names are step -1. Baseline grid **28 px** (was 27).

**Spacing.** Everything in grid lines of 28; inner padding of the card is half a line (14). One
blank line between column sections. Left edges: the text block at 160, the column at 888; no
other edges.

**Focus and hover.** One focus mark for keyboard, remote and mouse: a 2 x 20 px bar in `text`,
12 px left of the item (mock 1, "Coffee with Anna"). Arrows move it; the pointer moves it on
hover; OK, Enter or a click acts. No underline, no fill, no colour change.

**Motion.** Fades only, ease out: 160 ms in, 120 ms out. Chat open: notifications fold and chat
lines fade in, 160 ms, no slide, no height animation of the page. New chat lines fade in place;
the lines above step up by whole lines at once. The card fades in once and never moves. No motion
while a film plays except the column's own fades.

## First screen for a first-time user (mock 1)

Shows: scene 1 Agenda, the column with "Listening", the helpers, "Nothing new.", the chat door and
the key list. Footer: "→ 2 The Morning", clickable.

- **Keyboard**: the key list is on screen, no key to learn first: F1 all keys, `Super+← →` turn
  the page, `Super+Space` start a program, `Super+a` chat with a helper, `Super+n` hide this column,
  Esc back one step. F1 opens the full list in the column (and closes it again).
- **Mouse**: every agenda row, the next-scene line, each helper, the chat door, "Mute", "Show" and
  every key row show the focus mark on hover and act on a click (a key row does what its key does).
- **Remote**: arrows reach the same items; the column is one more column for Left and Right.
- **Voice**: the state word says it listens; "show the chat" opens the chat.

The key list stays until the user hides it (`Super+n` hides the whole column; "Hide keys" in the
full F1 list hides only the block). It comes back with F1.

## Buildable: what the source does today

- Regions: `mc.yaml` `regions.right {size: 13}` in em, drawn at `x + width + gap`
  (`region.rs` `boxes`); `side_widths` drops both sides when the text gets too narrow.
- Chat: the HUD host draws it into a box that mc names with `mc.chat.slot {output, rect}`
  (`main.rs` `chat_drawer`, today the drawer box); the host moves its view into any rect
  (`chat/host.rs` `slot_event`), and the card follows the view width (`chat/view.rs` `card_width`).
- HUD folder `plugins/chora-mc/hud/hud.yaml`: state panel `anchor: top_right, margin [32, 16]`,
  agents panel under it; chat `style: plain`, Pagella 18, colours per theme.
- Card: `card.rs` has a fixed light fill (`FILL 250`) and a 560 px cap; `card_glass.rs` has a
  dark variant; neither is the mc plain card yet.
- Renderer: Group, Rect (colour, radius, opacity), Text, Image, Shader; `pointer.entered` and
  `pointer.left` arrive over a rect, so a transparent hit rect per item gives hover.

## Build list (ranked, small steps)

1. **Key list in the column, F1.** Rule `{content: keys, region: right, slot: 9}`, a text content
   from the bindings, F1 binding. *Check:* fresh session at 1280x800 shows the six rows; F1 toggles
   the full list; finding 3 of `reports/swarm-2/findings.md` no longer reproduces.
2. **Clickable items and hover mark.** Transparent hit rect per agenda row, overview row, next
   line, column row; focus bar on `pointer.entered`. *Check:* mouse-only run reaches Agenda →
   Overview → Terminal and opens the chat without a key; the bar shows within one frame.
3. **Column geometry.** Right width `clamp(336, 26 %, 448)` on whole lines; hairline at its left;
   header limited to the text block. *Check:* 1280x720, 1280x800, 1920x1080 captures: column edges
   at 888/1224 (1280), nothing clipped, checklist sections 2, 4, 5.
4. **Move helpers and state into the column.** Rules for `agents` and `state` to `right`; hud.yaml
   state panel margin `[56, 20]`. *Check:* one state word per output (checklist 3); header shows
   only running head and clock.
5. **Honest empty notifications.** Word "Nothing new." / "Nichts Neues." in `chora-kit::words`.
   *Check:* empty and three-item captures, light and dark.
6. **Chat slot = column.** `chat_drawer` emits the column's chat box instead of `drawer_box`.
   *Check:* pixel diff of the text block before and after opening the chat is zero.
7. **Fold notifications, hide keys while the chat is open.** *Check:* expanded capture matches
   mock 2 layout to the line; Escape restores the collapsed column, draft kept.
8. **Remove the "Chat ..." pill in mc** (host draws no row when the folder names a slot; the
   column door replaces it). *Check:* no box on screen except a question card (rule 11).
9. **mc plain question card.** Card style from the folder: page fill, 1 px `text` rule, width =
   column. *Check:* card 336 wide, Yes/No 88x32 8 apart, 7:1 on both themes, does not move while
   ten replies arrive (clip at 30 fps).
10. **Scale 19/28 and the two colour fixes.** *Check:* contrast test passes; no text under 14 px.
11. **Keys:** `Super+a` chat, `Super+Home` agenda, `Super+Return` the Terminal scene, drawer out of
    `desk`. *Check:* help lists every key; no Super key types into a window.

## Taste questions for the user

**Q1. How does the column stand apart from the page?**
- A: a hairline only, same paper (mocks 1 and 2). Strictest to the book rule, calmest.
- B: a slightly darker paper tint, no line (mock 3). Reads as a separate zone from across a room,
  but adds the first fill to mc and needs a lighter muted on dark.

**Q2. Where does the key list live?**
- A: a "Keys" block at the column foot, clickable, until hidden (mock 1). Always findable.
- B: one line in the page footer, "F1 All keys · Super+← → Turn the page · Super+Space Programs"
  (mock 3). Quieter; the full list only on F1.
- C: only on the very first start and on F1; nothing on screen afterwards.

**Q3. What voice does the chat have in the margin?**
- A: Pagella step 0, speaker on its own line (mock 2). A book's side note; fewer words fit.
- B: Inter step -1, speaker in a name column (mock 3). Denser (no name lines), reads as a log.

## Mocks

- `media/2026-11-12/design/mc/mock-1-first-screen-light.png`: first screen, Q1 A, Q2 A, hover mark.
- `media/2026-11-12/design/mc/mock-2-chat-question-dark.png`: chat open with a question, Q1 A, Q3 A.
- `media/2026-11-12/design/mc/mock-3-options-b-light.png`: chat open, speaking, Q1 B, Q2 B, Q3 B.

Sources next to them (`*.html`, `mc-v2.css`); Pagella loads from a local install or `pagella.otf`
beside the HTML (not committed).
