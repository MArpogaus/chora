# Deep UX review: mc, merged look (head 59df16e)

Reviewer run, 2026-11-12. 1280x800 headless session, `session.sh mc`, fake voice helper "Atlas" as in
`tests/gate-mc-host.sh`. About 32 looks, dark first, then light (first screen, chat, card, terminal);
keyboard and mouse. Captures: `media/2026-11-12/ux-deep-mc/NN-*.png` (cited as [NN]). Judged against
`design/looks/mc-v2.md`, `shared.md`, mocks `media/2026-11-12/design/mc/`, `product-briefs.md`,
`decisions-taste.md`. Pixels measured from the PNGs (contrast = WCAG, sRGB values from the pixels).

Not blamed on mc (environment): TeX Gyre Pagella missing (serif fallback), foot rejects the
`[colors-dark]` section (red `error:` lines in the Terminal scene, black terminal on the light page),
empty collector data (no agenda rows, "No weather yet."), the fake helper never sends a state after a
question or a Stop (the helper row stays "has a question" / "working"). One of my own errors: a
non-numeric question id (`q1`) is silently dropped by `Question::from_event` (`id.parse().ok()?`); real
ids are numbers, so no finding, but a refusal in the log would help.

What is right (measured): column edges 888/1224, hairline x 860 y 28 to 771; running head, clock and state
word share one baseline (y 32); key rows pitch 28; hover bar 2 x 20 at x 874, 12 px left of the item, 11.5:1;
question card 336 wide (x 888 to 1223), 1 px `text` rule, page fill, Yes/No 88 x 32 at x 902 and 998 (8 apart),
card stays still; Yes by click and No by Alt+n both answer and take the card away; Esc closes the chat and
keeps the draft; a click on the chat door opens the chat straight to the one helper; notifications fold to
"NOTIFICATIONS 3 / Show" when the chat opens; contrast of text/muted/accent/amber on both themes is
13.8/11.5, 5.0/4.9, 8.1/5.2, 4.7 (all >= 4.5, card buttons and key line 11.5:1 dark, 13.8:1 light).

## MAJOR (main paths first)

**M1. The Terminal scene loses the whole column, and a chat opened there is a different, boxed UI.**
[06] [07] [20]. On scene 5 there is no hairline, no notifications, no chat door, no key list and no
"next scene" footer (the only exits are keys nobody is shown); the helper jumps into the header between
running head and clock ("Atlas" at x 444). Super+a there opens a full-height *boxed* panel x 880 to 1280, y 0
to 800 (1 px `#424240` frame, 0 px gutter at the right and bottom), bold sans title "Chat with a helper",
a boxed "Close" button, sans list and placeholder: breaks mc-style rule 11, the Pagella chat voice (Q3 A)
and the 56 px gutter. Cause: no `mc.chat.slot` on that scene. If the chat is open when the user turns the
page to the terminal (Super+Right works while the field has the keyboard), the column's chat leftovers
(Stop, messages, the question card) stay on a bare page with no head, no hairline, no field [20].
Fix: keep the column (hairline, state, chat, keys; the terminal is 672 px and already ends at x 832) on
scene 5 and always emit the slot; the header helper row never replaces the column row; close the chat when
the scene changes, but leave a pending question card in place.

**M2. No visible Mute (shared invariant, mc-v2 "Mute flush right").** [01] [18]. The state line is
"Listening" + 12 px dot at x 1137 to 1219, right-aligned, no "Mute" word; `hud.yaml` `action: mute` makes a
click on the state word mute with no label or hover mark. A sofa user cannot find or trust it, and muted
looks like another state word. Fix: add "Mute" (muted, 14+ px) flush right at 1224 and put the state sign and
word left at 888 on the header baseline as the mock does (sign before word); muted shows "Microphone off"
plus "Turn on". The dot now ends at x 1219, 4 px short of the column edge 1223.

**M3. "All keys" (Super+F1) overflows the column and strikes through the chat door.** [17]. With three
notifications the list starts at y 504, its hairline (y 476) runs through "Chat with a helper ..." (text
y 244 to 261 relative: the line is 1 px at `#424240`), and the rows run to y 791, past the column foot 772
and the 28 px gutter; the rows after "Super+Down" (Esc and more) are cut off at the screen edge. Fix: the all-keys
list replaces the notifications and chat sections (as the expanded chat hides the key list), scrolls inside
the column foot, and the hairline moves with it.

**M4. F1 is not the key the mock and the first screen promise.** [01] [17]. Mock and mc-v2 say "F1 All
keys"; the build binds Super+F1 (`mc.yaml` line 181) and a plain F1 does nothing. Consistent between key list
and binding (good), but different from xp ("F1 opens the help") and the doc; an F1-only user fails on the
first try. Fix: bind plain F1 as well (or decide Super+F1 in `shared.md` and every look's help).

**M5. Terminal text is about 11 px (sofa floor 14).** [06]. foot text glyph height 10, row pitch 13 px
(about 11 px font), also in the empty state and the prompt. The scene fills 672 x 520 with unreadable-from-three-metres
text. Fix: start foot with a font size of 19 px (about 14 rows pitch 28 would be 18 rows, so use 16 px,
pitch 21, 24 rows), and let the terminal be at the grid (y 168, height 28 x n).

**M6. A critical notification is an unexplained red footer line and the column does not mark it.** [15]
[16]. The urgent text sits at the footer left of "-> 2 The Morning", uppercase tracked red 11 px cap height
(x 247 to 678, 29 px before the next-scene link, which reads as one sentence), its "x" is 7 x 7 px and
a click on it did nothing in my run (still shown after the click) [16]. In the column the same item has no
urgency sign. Fix: red sign or word "Urgent" in the column row; dismiss target at least 28 x 28 in the column
(row click or "Dismiss"); the footer line is sentence case, 14+ px, with one line of air before the footer link.

## MINOR

**m1. Chat field: boxed, small, different voice from the mock.** [08] [10] [16]. A 336 x 36 box (frame
`#424240` on `#1c1c1c` = 1.7:1, light frame `#d4cfc4` on `#f7f5ed` = 1.4:1: below 3:1 for a control edge), Inter
placeholder "Type a message, then Enter." (14 px, 4.9:1) instead of Pagella "Write to Atlas ..." with a
hairline above and "Enter" flush right; the mock has no box. The 36 px height is off the 28 grid (736 to 771).
Fix: hairline field per mock, placeholder with the helper's name; if the box stays, frame in `muted`
(4.9:1).

**m2. Question state says the same thing four times.** [12]. Row "Atlas has a question", the card, the hint "Typing
does not answer." (and in the first frames "Typing does not answer. Press Alt+Y or Alt+N, or click Yes or No."),
and the field's second line "A question is waiting." which sits inside the field and reads as placeholder
text; U3 says one place per sentence. Fix: drop the hint and the field line while the card shows (the card
holds the key line and "a spoken yes does not count"); the field keeps its own placeholder.

**m3. Card text is dense and wraps badly.** [12]. Prompt "Atlas wants to write the file / notes.txt." orphans
the file name (Pagella 19 on 336 minus 28); the mono detail wraps inside "(412 / bytes)"; line pitch
27 / 20 / 20 / 20 inside the card (not 28-based); mono is 14 px (8.4 px per char), at the floor, not
sofa-sized. No "time left" (mock "0:48 left", `shared.md` card order) showed. Fix: break detail at path
boundaries, 15 to 16 px mono, half-line (14 px) between prompt, detail, key line; show time left when the event has
one.

**m4. Answering leaves no trace.** [14]. After Yes the card disappears and the transcript shows nothing
("You allowed: write notes.txt"); the user cannot tell later what they answered (also after No and Esc).
Fix: one muted line in the transcript for each answer.

**m5. Stop is a 30 x 14 muted word on its own row.** [10] [15]. It is the only way to end a run, has no helper head
to say what it stops (no "Atlas" head, placeholder is generic), disappears without acknowledgement after a click
and leaves a 42 px gap [15]. Fix: "Atlas" head line with "Stop" flush right in `text` colour, a 28 px
row, and a "Stopped." line in the transcript.

**m6. Notifications: "NOTIFICATIONS 0" and a jumping chat door.** [08] [15]. With the chat open and no notices
the head reads "NOTIFICATIONS 0" (mock: folded only when there are some; honest empty state is "Nothing
new."). Three notifications push the chat door from y 251 to y 475 (and 5 lines for one long title), so
the door moves each time a notice arrives. Items: title and body are joined by a colon in one run
(no title/body difference), the text block begins at x 955 (67 px time column) leaving 266 px. Fix: cap
the notification block at 3 rows of 2 lines each, keep the chat door at a fixed y (or pin it to the foot above keys),
bold or `text`-coloured title and muted body.

**m7. Picker for a single helper.** [08] [09]. Chat opens with a list "1 Atlas · working" and typing "1" puts a
"1" into the field (no list reaction); only Enter picks. From the door by mouse it goes straight to Atlas
(no picker) [16], so keyboard and mouse differ. Fix: with one helper open it directly; with several,
a number key picks at once and a click on the row picks.

**m8. Scenes: empty-state copy and mixed fonts.** [03] [05]. "No briefing yet. The collector has not run."
uses a developer word ("collector"); "No data yet." in Mission Control is Inter while the other empty
lines are serif; "Journal, errors today" is a serif section head in error red (`#eb6b61`, accent only for scene
title and voice state) above nothing; the empty Journal has no "Nothing to report." line. Rows in Morning
stack at pitch 84 (3 lines) with no heads, so "No mail yet." / "Nothing overnight." float; two-column
"No weather yet." (x 160) / "Nothing planned today." (x 562; mock: divider at 490 and text at 518).
Mission Control divider at x 444 with right text at x 562 (284 px left, 118 px right: not centred).
Fix: one empty-line style (serif muted), plain words, red heads only for failures, and the mock's
28 px gap each side of the divider.

**m9. Scene 3 "Agents" breaks three rules.** [04]. Word "Agents" (shared: never "agent"; footer "-> 3 Agents",
and "AGENTS · 1 RUNNING" has no scene number, unlike "1 AGENDA", "2 THE MORNING", "4 MISSION CONTROL"), the
state dot is off-white (`#d9d4c9`) while the name is green (`#85b88f`), and "since 02:22" floats at y 541,
a 220 px jump away from "Tidy notes" (y 286 to 303). Fix: "Helpers", "3 HELPERS · 1 WORKING", dot in the state colour,
"since" under the task.

## COSMETIC

- c1. Hover on Yes/No thickens the rim to 2 px [13]; every other hover is the 2 x 20 bar. Different idiom, hardly
  visible from a sofa. Fix: same bar, 12 px left of the button, or fill the button with `text` at 12 %.
- c2. Chat body is 18 px / line pitch 27 (`hud.yaml` `size: 18`); mc-v2 sets 19 / 28. The key rows and
  notifications are on 28. Fix: size 19, line 28.
- c3. A hover bar stays on a row after the card under the pointer ends [05] (pointer did not move): correct, but
  unexplained at a glance; clear marks when the layer under the pointer changes.
- c4. Key list shows "Super+ <- ->" with a space after the plus; the others have none [01]. Fix: "Super+← →".
- c5. Footer "next scene" is flush right at x 831 (mock: flush left at 160); with a notice it nearly touches the
  red line (29 px). Either is fine but the doc and build differ.
- c6. The light terminal is a black 672 x 520 block (`#222`) on cream [20] (env: colors-dark rejected, so the
  light palette never arrives); in dark it is `#242424` on `#1c1c1c`, a visible box (rule 11).
- c7. Reduced contrast under 4.5:1: none on text. UI edges under 3:1: chat field frame, key-block hairline (1.7:1,
  intended faint).

## Consistency with the other looks

- Words and states: the state words are the shared set; "Agents" (m9) is the one word that is not. Question card
  order, Yes left / No right, 8 apart, 88 x 32 (xp/tiling minimum 32 x 24), "a spoken yes does not count": same.
- Escape closes the chat and keeps the draft: same as shared. Super+a opens the chat in every look: yes; the door and
  key row say Super+a; Super+Space launcher: listed, same key as construct, xp, tiling.
- Visible mute: xp has a tray icon, tiling a panel segment, construct a face mark; mc has none (M2).
- "A visible way in" (P3): mc has the door and a key row, better than tiling's single segment; but the Terminal scene
  loses it (M1), the only place in mc where the door is missing.
- Fallback chat panel (M1) is the generic host look; xp and tiling users get the same boxed panel only when no
  look slot exists. In mc it should never appear.
- F1: xp = help window on F1; mc = Super+F1 (M4).
- Sizes: mc minimum is meant to be above the shared 14 px; labels and notes are 14 to 15 px, the card mono and
  the terminal are not above the floor (m3, M5).

## Proposed order of work

M1, M2 (shared RC), M3, M6, M4, then m1 to m9 as one pass over `hud.yaml` and the scene copy.
