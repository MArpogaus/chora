# Review: mc v2 "page and margin", pixels

Independent review of branch `claude/mc-v2` (head f1450d1, built in
`/home/user/wt/mc-v2/target/debug`), 2026-11-11. No code changed.
Captures: `media/2026-11-12/review-mc-v2/` (`NN-*` full frames, `16-20`
at scale 2 = `2560x1600@2`, `21` with TeX Gyre Pagella available,
`cNN-*` zoomed crops with the flaw boxed in magenta). Compared against
`design/looks/mc-v2.md`, mocks 1 and 2, `design/decisions-taste.md` (mc:
Q1 A hairline, Q2 A keys at the column foot, Q3 A Pagella chat, nothing
under 14 px) and the chora-mc brief in `design/product-briefs.md`.

Area tags: **[mc]** = `plugins/chora-mc`, **[host]** = the HUD host
`chora-assistant` (and its mc folder `plugins/chora-mc/hud/hud.yaml`).

## How the captures were made

- `tests/persona/session.sh mc 1280x800` (and `2560x1600@2`) with
  `CHORA_BIN=/home/user/wt/mc-v2/target/debug`, `PERSONA_DIR` and
  `PERSONA_KEEP` in a scratch dir, a private config
  (`chora-mc.yaml`: `theme: light|dark`, `voice_hud: host` = the shipped
  default; `chora-assistant.yaml`: `hud: mc`; the mc HUD folder copied to
  `XDG_DATA_HOME`). The session starts `chora-assistant` as its companion.
- `act.py look --max-side 1280` (2560 at scale 2), `key`, `move`, `click`,
  `type`. Data the headless box does not have was set through the admin
  socket: agenda lines (`panel.set`, as `collectors/agenda.sh` does),
  notifications (`notify.push`) and a helper with a transcript and a
  question (`tests/fake-voice.py` with `FAKE_VOICE_ENTRIES`, `voice.state`,
  `voice.confirm`, as the gates do).
- The dark and scale-2 runs used `CHORA_TEST_FLAGS=--test-injected-press-is-real`;
  without it injected clicks and keys do not reach the host's chat (the
  light run: a click on "1 Atlas" and the key `1` did nothing, Escape did
  not close the chat). That is the test rig, not a finding.
- The yellow 12 px square in some frames (`05`, `02`) is the core's
  injection mark (`chora-scene` `show_injection`, D23), not a product flaw.
- Ended with `session.sh --stop ENV`; nothing killed by name.
- Also ran `ONLY='mc-host-hud|mc-chat-drawer' tests/gate-hud.sh`:
  `mc-host-hud pass`, `mc-chat-drawer missing` ("timeout 5s:
  chat_in_slot"). mc emits `mc.chat.slot {rect: [888, 98, 336, 674]}`;
  the host does not put the chat there.

## First look (before reading the design)

The builder's captures (`media/2026-11-12/mc-v2/*.png`): a cream (light) or
charcoal (dark) page, a small spaced running head "1 AGENDA" and a date
line at the top, a big red "1 | AGENDA" chapter head, and an empty page
under it. A thin vertical hairline at x 860 separates a right column:
"Listening" in red, three helper rows with coloured signs, a NOTIFICATIONS
head with "18:51 Update ready", a CHAT head with "Chat with a helper ..."
and "Super+a" flush right, and a KEYS block of six rows pinned to the
foot. The column is calm and even. What jumps out: the reading text
("Update ready", "Chat with a helper ...", "AGENDA") is a wide sans
(DejaVu Sans) next to Inter labels, so the page has two sans faces; the
running head and the date are tiny; the notifications list
(`mc-column-notifications.png`) spaces its rows two lines apart; with the
chat open (`mc-column-chat-*.png`) the column holds a boxed panel with
an error line "Cannot reach voice control. Trying again e..." cut by an
ellipsis, bold Stop/Close buttons and a field in a third, Arial-like
face, and the "dark" chat is a light box on the dark page. The page
itself never shows content, so the agenda look cannot be judged from
them.

## What I saw in my own run (short)

The first screen of the shipped default (`voice_hud: host`) does not look
like the builder's captures: the host draws its own boxed "Chat ..." pill,
an 11 px "Listening" with a dot and a boxed "Atlas · working · Tidy notes
(1/2)" row on top of the column, over mc's own text (`02`, `10`, `16`,
`19`). Super+a opens the chat as a box over the page at x 364, not in the
column (`06`, `07`, `12`, `20`); a question moves it to x 325 and leaves
the field behind (`08`, `13`). The question card is the stock light card
with rounded corners, in the dark theme too. mc's own column (keys,
hover bar, Super+F1, fold of notifications, the door, Close) works by key
and mouse and is close to the mock.

## Findings

Ranked; inside a rank, main path first. Measurements are logical pixels
at 1280x800 unless noted.

### MAJOR

**M1 [host] The chat opens over the page, not in the column.**
`06-chat-picker-light.png`, `07-chat-messages-light.png`,
`12-chat-dark.png`, `20-s2-chat-dark.png`, crop `c06-chat-over-page.png`.
mc announces the slot `[888, 98, 336, 674]` (seen on the event bus); the
host draws `hud-chat` at `[364, 98, 336, 674]` (dark) or
`[364, 154, 336, 618]` (light, with a state line): y, width and height are
the slot's, x is not. The box covers 336 px of the agenda text block
(rows cut at "Stand-up with the pla|"), the chapter head ("AGEN|DA" cut,
`12`) and the footer "11:30" (`c15`), while the column below "CHAT  Esc
Close" stays empty. Breaks the user goal ("rather the right margin ...
instead of stealing half the screen"), the design rule "the page never
moves for the margin", and gate cell `mc-chat-drawer`. This is the
builder's "chat placed at x=364" open item; rated MAJOR because it is the
main chat path in both themes and both scales. Fix: in
`chat/host.rs`, place the view at the slot rect as given (x too) and stop
re-deriving x from the view/card width; add an assertion of all four
numbers in `mc-chat-drawer`.

**M2 [host] Host rows drawn on top of mc's column text.**
`02-first-screen-filled-light.png`, `09-notifications-under-pill-light.png`,
`10-first-screen-dark.png`, `16-s2-first-light.png`, crops
`c02-host-pills-over-column.png`, `c03-dark-pill-over-chat-door.png`.
`hud-row-Atlas` is an opaque image at `[982, 102, 242, 32]`. In the
default first screen it lands on the CHAT head row (`CHAT` text rect
`[888, 98, 51, 28]`) and its bottom edge (y 134) cuts the ascenders of
"helper" (ink from y 132) and touches "Super+a" (ink 134 to 147); at scale
2 the border visibly runs through "helper". With a state line it hides
"Nothing new." down to "Nothing n" (`02`) and a notification down to
"18:59 Atl" (`09`). The builder lists this as "helper rows drawn flush
right by the host". Fix: with `slot: mc` the host must draw no `agents`
panel (mc draws helpers in the column, design §Column sections 2), or the
folder drops `panels: agents`; never paint an opaque image over another
client's text.

**M3 [host] The boxed "Chat ..." pill is still drawn.**
`01-first-screen-light.png`, crop `c01-host-chat-pill.png`; also `02`, `10`,
`16`, `19`. `hud-row-chat` at `[1149, 64, 75, 32]`, a 1 px rounded box in
Inter 14 between "Nothing new." (ink bottom 65) and the CHAT head (ink top
106): it duplicates the door "Chat with a helper ... Super+a" 2 lines
below and is the one box mc-style rule 11 forbids (build list step 8).
Known open item, confirmed. Fix: host draws no chat row when the folder
names a `slot`.

**M4 [host] Host state word 11 px, a second state line, and it covers
mc's "Show".** `10-first-screen-dark.png`, `20-s2-chat-dark.png`, crop
`c04-s2-host-word-over-show.png`. `hud-word` `[1144, 18, 56, 21]` plus
`hud-figure` `[1206, 20, 16, 16]`: cap height of "L" 8 px = Inter ~11 px,
under the 14 px floor and unreadable from the sofa. It sits on the
NOTIFICATIONS line, so when the chat is open mc's "Show" (flush right at
x 1186 to 1224) is drawn under the word and the dot: it reads "Listening
1C●V" (`c04`). In host mode mc's own state line is empty, so the state is
only this 11 px word; with mc's own state set the screen shows two state
words, and during a question they disagree ("Listening" in mc, "Question
for you" in the host, `08`). Design: one state word in `accent` at 888 on
the header baseline, "Mute" flush right. Fix: with `slot: mc` the host
sends state to mc (or mc reads `voice.state`) and draws no `state` panel;
otherwise move it to the column's first line at 14.25 px and reserve
that line in mc's layout.

**M5 [host] The question card is the stock card, light on dark, and not
in the column.** `08-question-light.png`, `13-question-dark.png`,
`17-s2-question-light.png`, crops `c07-card-and-field-offset.png`,
`c08-dark-light-card.png`. `hud-card` `[325, 486, 336, 212]`: fill
`#fafafa` (not the paper `#f7f5ed`, and in dark not `#1c1c1c`: a white
card on a charcoal page, U2 says dark card on a dark page), border
`#3c3c3c` with ~8 px radius (design: 1 px `text` rule, square), buttons
`#e8e8e8` fill (off palette), text in an Arial-like face (third sans),
the prompt at body size instead of Pagella, the path in the proportional
face and wrapped "(5 new / folders)" (design: `mono`), no "Question from
Atlas", no time left, the key line reworded ("Press Alt+Y to allow or
Alt+N to refuse." vs "Alt+Y allow   Alt+N refuse"). Padding is uneven:
18 px from the top edge to the title ink, 40 px from the buttons (bottom
657) to the card edge (697). Yes and No are 88x32, 8 apart as specified.
The builder lists "question card style not built (host)"; confirmed, and
the dark case makes it the loudest thing on the screen. Fix: build step 9
(card style from the folder: page fill, 1 px `text` rule, no radius,
column width, `mono` detail, title line with time left), and pin it above
the field inside the slot.

**M6 [host] The chat jumps when a question comes and the field stays
behind.** `08`, `13`, `14-after-answer-dark.png`, crop `c07`. On
`voice.confirm` the chat box moves from x 364 to x 325 (39 px) and stays
there after the question ends; the field stays at `[364, 736, 336, 36]`,
so the frame shows two misaligned boxes, the field sticking 39 px out of
the chat's right edge. Design: "the card fades in once and never moves",
the field at the column foot. Fix: one rect for view, card and field (the
slot); do not recompute x from the card width.

**M7 [host] Chat type is Inter 14, not Pagella; the reply is cut with an
ellipsis mid-wrap.** `07`, `12`, crop `c09-chat-ellipsis-mid-wrap.png`.
`hud.yaml` asks `font: TeX Gyre Pagella, size: 18`; the transcript renders
as a 14 px sans (cap height 10 px), speakers bold. Q3 A (PM decision) is
the book serif with the speaker on its own line. The reply "I read 14
notes and made 5 days. Moving them needs your yes." shows as "...Moving
the..." / "needs your yes.": a word is cut by "..." in the middle of a
wrapped message (design rule 12: a long reply wraps, never cut). The
builder's capture has the same cut ("Trying again e..."). Fix: honour the
folder font and size (fall back to the same serif fallback as mc when
Pagella is missing), wrap on word boundaries without ellipsis.

**M8 [mc] The running head and the date are 11.6 px.**
`01`, crop `c05-running-head-11px.png`. `mc-r1-header-head` and
`mc-r1-header-clock` have `font_size` 11.6128 (`running_head` `em:
0.6112` x 19): cap height 8 to 9 px (ink y 23 to 31). Nothing may be
under 14 px at 1280 (brief: readable from three metres; decision "watch"),
and the gate only checks the column (x >= 860). The date and time are the
one thing a sofa user reads in the header. Fix: `running_head` em 0.75
(14.25 px, the same as the column labels `label`), and extend the gate's
14 px check to every mc text node.

**M9 [mc] An empty scene is a blank page.** `01`, `05-next-scene-light.png`,
`19-s2-first-dark.png`. Without `~/Notes/agenda/gtd.org` (any new user,
and the headless runs) "1 AGENDA" has nothing under it, and "2 THE
MORNING" is blank too; no line says why or what to do, and the footer
"→ 2 The Morning" of the mock is absent. The first screen must say what
it is (brief RC); the column now has "Nothing new." but the page that
fills two thirds of the screen does not. Fix: per panel an empty text
("Nothing planned today." / "No weather yet: the collector did not run")
in `muted`, and the `next` footer line falling back to the next scene's
name when there is no next agenda entry.

### MINOR

**N1 [mc, packaging] Without TeX Gyre Pagella the reading text falls
back to DejaVu Sans next to Inter.** `01` vs `21-pagella-first-light.png`.
Every builder capture and my default runs show "AGENDA", agenda rows,
"Nothing new.", notifications and the door in DejaVu Sans (wider, heavier
at scale 1), so the page has two sans faces and none of the mock's
character. With `CHORA_FONT_DIRS` pointing at `pagella.otf` the page
matches mock 1 closely (`21`). The RPM ships the fontconfig snippet, so
this is the dev and headless setup; but a missing Pagella should fall
back to a serif (DejaVu Serif is installed) rather than a sans, and the
review captures should be made with Pagella. Fix: `body` family list
`[TeX Gyre Pagella, DejaVu Serif]` or a serif generic; add Pagella to the
gate and persona environment.

**N2 [host] Hover passes through the chat to the page.** `07`, `12`,
crop `c10-hover-leaks-under-chat.png`. With the pointer over the chat at
(430, 238) the agenda row "11:30 Coffee with Anna" under it shows its
focus bar `[146, 228, 2, 20]`; with the pointer at (430, 182) row 08:30
does. A click there would act on the hidden row. Fix: the chat image
takes pointer input (InputMode block) over its rect; disappears with M1
for the page, but the same holds for the column rows.

**N3 [host] Field in a third font and of the wrong words.** crop
`c15-field-font-and-footer-clip.png`. "Type a message, then Enter." is an
Arial-like face (not Inter, not Pagella); design: "Write to Atlas ...",
"Enter" flush right, on the footer baseline under a hairline. The box
`[364, 736, 336, 36]` also covers the footer "11:30" (ink x 668 to 706,
y 766 to 776). The caret touches the first glyph (1 px gap) at scale 1
and 2. Fix: field in the chat font, the design words, no box (hairline
only).

**N4 [host] No hover mark on the card buttons, Stop and Close.**
`13` (pointer on Yes, frame `run-d1/011`): Yes keeps `#e8e8e8`, no bar,
no change. Design: one focus mark on every item. Fix: the 2x20 bar 12 px
left of the hovered button, or the button rule in `text`.

**N5 [host] Duplicate Close and bold buttons in the chat.** `07`, `12`.
The host's chat has its own "Stop" and "Close" boxed bold buttons
(`hud-chat-stop`, `hud-chat-close` 88x28) beside mc's "Esc  Close" on the
CHAT head; the picker shows a third "Close". Design: no boxes but the
card; the column head owns Close. Fix: with `slot: mc` the host omits its
title line and Close, keeps Stop as a plain text item.

**N6 [host] After a question, the field loses the caret and Esc no
longer closes the chat.** `14-after-answer-dark.png`. After a click on Yes
the field shows no caret; Escape then did nothing (chat stayed, `hud-chat`
still at x 325); only a click on "Close" closed it. The key list says
"Esc  Back one step". Fix: give the keyboard back to the field when the
card ends (P11), or let mc's Esc binding call `chat.hide`.

**N7 [mc] Notification rows are two lines apart and the time is not the
margin style.** `09`, `15-notifications-wrap-dark.png`, crops
`c14-notifications-56px-rows.png`, `c13-notification-wrap-no-hang.png`.
Rows step 56 px (y 111, 167; builder's `mc-column-notifications.png`
55, 111, 167): a blank line between one-line notes, so three notes take
six lines while helpers step 28. The time "18:59" is the reading face in
`text` colour; design: time in `margin` (Inter, `muted`, tabular). A
wrapped note starts its second line at 888 under the time instead of
under the text at 955. Fix: one line per note, time in `margin`, hanging
indent to the text column.

**N8 [mc] Super+F1 list order and words.** `04-all-keys-light.png`,
crop `c11-all-keys-order.png`. Sorted by key name bytes: Super+Down, F1,
F12, Home, ←, Return, →, Up, a, n, o, Space; Super+← and Super+→ are split
by Super+Return. Words differ from the short list ("Show or hide the
right" vs "Hide this column", "Previous page"/"Next page" vs "Turn the
page"); "Show or hide the right" is unfinished. Fix: order by task (page,
chat, programs, column, overview, header/footer, Esc last) and reuse the
short list's words.

**N9 [mc] Key list says Super+F1, the design says F1.** `01` vs mock 1.
The mock and the design text name "F1 All keys"; the build binds and
shows "Super+F1". Either is defensible (F1 may belong to a program in a
scene), but design, mock and build disagree. Fix: decide, then align
`mc-v2.md` or the binding.

**N10 [mc] "Nothing new." and notifications in full `text` colour.**
`01` vs mock 1. The mock sets "Nothing new." in `muted` so the empty
state stays quiet; the build draws it at full contrast at 19 px, the
loudest line in the column on an empty screen. Fix: `muted` for the
empty text only.

**N11 [host] Helper state in `accent`.** crop `c02`, scale 2
(`16`). "working" is red (`accent` `[140, 36, 31]` in `hud.yaml`).
Design: accent for the scene title and the voice state, nothing else;
working is `green`. Fix: map the state colours in the folder to mc's
`states` palette (green working, blue thinking, amber question).

**N12 [host] Builder's "dark chat renders light in headless": not
reproduced.** In my dark runs (`12`, `20`) the chat fill is `#1c1c1c`
with `#424240` border as `hud.yaml` `dark.chat` says; in the builder's
`mc-column-chat-dark.png` it is light. Likely the host started before mc
sent the theme (the gate's `chat_session dark` restarts mc after a light
one). Rated MINOR because it depends on start order: the host should ask
mc for the theme on connect, not wait for an event.

### COSMETIC

**C1 [mc] "Super+← →" crowds the plus.** crop `c12-super-plus-arrow.png`:
1 px between "+" and "←", a full space between "←" and "→". Fix: "Super+←/→"
or a thin space after "+", as in mock 1.

**C2 [mc] Header and state baselines 1 px apart.** `21`, crop `c16`:
running head "1" baseline y 31, "Listening" (column) y 32 and
NOTIFICATIONS y 32 at scale 1. Mock 1 has both on 34. Fix: put the
column's first line on the header baseline (same text rect y and line
height as `mc-r1-header-head`).

**C3 [mc] Door and its key 1 to 2 px apart.** `21`, crop `c16`: "Chat
with a helper ..." (Pagella 19) baseline y 198 to 199, "Super+a" (Inter
14.25) y 200. Same in the key column. Fix: align the margin style to the
body baseline in a mixed row (shift by the ascent difference).

**C4 [mc] Folded NOTIFICATIONS with nothing new shows no count and no
"Show".** `12` (dark, chat open): the head reads "NOTIFICATIONS" alone;
with notes it reads "NOTIFICATIONS  2   Show". Fix: "NOTIFICATIONS  0" or
hide the head while the chat is open and there is nothing.

**C5 [mc] Chapter number and title tops 1 px apart in the fallback
face.** `01`: "1" ink top 94, "AGENDA" 93 (bottoms both 126). Gone with
Pagella (`21`, lining figures). No fix beyond N1.

**C6 [host] Picker row is a bare "1  Atlas · working".** `06`: no
hover mark, no Pagella, the digit key is not explained ("1" reads as a
count). Fix: "Atlas  working" as a column row with the focus bar, "1" in
`muted` as a key hint flush right.

## Passes (checked, no finding)

- Column geometry at 1280x800: hairline `[860, 28, 1, 744]`, column 888
  to 1224, keys rule `[888, 562, 336, 1]`, key list foot y 772 (28 from
  the bottom), clock right ink 828 vs text block 832.
- Hover bar: `[874, 664, 2, 20]` on the key row 660 to 688 (centred,
  12 px left of the text at 888, `text` colour), on agenda rows
  `[146, 228, 2, 20]`, on the door; it goes when the pointer leaves.
- Super+F1 toggles the full list in place; Super+Right turns the page,
  the column does not move; Super+Home back; Super+a and a click on the
  door ask the host for the chat; "Close" in the column closes it; the
  notifications fold to "NOTIFICATIONS  2  Show" and the keys hide while
  the chat is open, and come back after.
- Mouse-only path: door click, helper pick (with real presses), Yes on
  the card (`confirm.answer {"yes": true}` recorded), Close: all by mouse.
- Palette contrast on the paper (computed from `mc.yaml`): light muted
  4.99, accent 8.05, green 4.93, blue 5.82, amber 4.75; dark muted 4.92,
  accent 5.22, text 11.5. All at or over 4.5:1.
- Scale 2: mc text and the host images are sharp (host renders at 2x);
  no blur, no half-pixel hairlines (hairline 2 px wide at scale 2).
- No flicker seen on page turn other than the core's injection mark.
