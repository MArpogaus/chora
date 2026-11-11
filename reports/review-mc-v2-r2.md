# Review: mc v2, round 2 (pixels, host fixes)

Second independent review of branch `claude/mc-v2` at 12a241d (binaries in
`/home/user/wt/mc-v2/target/debug`), 2026-11-11. No code changed. Judged
against `design/looks/mc-v2.md`, mocks `media/2026-11-12/design/mc/`,
`design/decisions-taste.md` (mc: Q1 A, Q2 A, Q3 A, nothing under 14 px)
and `design/product-briefs.md` (chora-mc, chora-assistant). First round:
`reports/review-mc-v2.md` (M1 to M9, N1 to N12, C1 to C6).

Captures: `media/2026-11-12/review-mc-v2-r2/`. `s1-light-*`, `s1-dark-*`
at 1280x800, `s2-*` at 2560x1600@2, `tiling-*` and `xp-*` the other looks
with the same binaries, `tiling-base-*` and `xp-base-*` the same steps with
the older binaries of `/home/user/wt/tiling-v2` (5e23625) as a baseline,
`cNN-*` zoomed crops, flaw boxed in magenta. Coordinates are logical
pixels at 1280x800 unless noted.

Area tags: **[mc]** `plugins/chora-mc`, **[host]** `chora-assistant` (and
the mc folder `plugins/chora-mc/hud`), **[other look]** tiling or xp.

## Verdict: FIX

One MAJOR in mc (M9's fix stacks six empty texts on top of each other on
scene 2, the first page a user turns to). Every first-round MAJOR in the
host is fixed or reduced to MINOR. Counts below: 1 MAJOR, 12 MINOR,
6 COSMETIC (new and remaining together).

## How the captures were made

- `tests/persona/session.sh mc 1280x800` (and `2560x1600@2`) with
  `CHORA_BIN=/home/user/wt/mc-v2/target/debug`,
  `CHORA_TEST_FLAGS=--test-injected-press-is-real`, `PERSONA_DIR` and
  `PERSONA_KEEP` in a scratch dir; private config as the gate
  `tests/gate-mc-host.sh` does (`chora-mc.yaml`: `theme`, `voice_hud:
  host`, `test_accept_injected_keys`; `chora-assistant.yaml`: `hud: mc`;
  the mc HUD folder in `XDG_DATA_HOME`). Driving only by `act.py`
  (`look`, `move`, `click`, `key`, `type`).
- Data the headless box lacks, through the admin socket: agenda lines
  (`panel.set`, as `collectors/agenda.sh`), three notifications
  (`notify.push`), a helper with a transcript and questions
  (`tests/fake-voice.py`, `FAKE_VOICE_ENTRIES`, `FAKE_VOICE_KEYS=1`).
  The fake agent emits `voice.confirm {done}` only for Alt+y/Alt+n; for a
  click I emitted `done` 0.3 s later, as the real agent
  (`services/voice/src/mc.rs`) does.
- Steps per run: first screen; hover an agenda row and a key row;
  Super+F1 on and off; Super+Right three times (scenes 2 to 4),
  Super+Home; notifications; Super+a, pick `1` + Return, send "hello
  there"; question 71, hover Yes, click Yes, Esc; Super+a, question 72
  Alt+y, 73 Alt+n, 74 click No; hover and click Stop; Esc. Mouse only:
  hover and click the door, click the helper row, question 75 click Yes,
  hover and click Close, click "→ 2 The Morning". All five answers reached
  the agent (`71 yes, 72 yes, 73 no, 74 no, 75 yes`) in all four runs;
  Stop called `agent.cancel`.
- Pagella is not installed here, so mc and the host both fall back to
  DejaVu Serif (the builder's own captures had Pagella in mc, which is why
  their page looks different from mine).
- Each session ended with `session.sh --stop ENV`; nothing killed by name.

## First look (builder's captures `media/2026-11-12/mc-v2-r2/`, before notes)

Calm cream or charcoal page, the column at 888 with a hairline, the chat now
lives in the column in a serif with the speaker on its own line, the card
is a square 1 px box in the page fill, light card on light, dark on dark.
What I saw: the field below is a boxed Arial-like face and its placeholder
is in full text colour; the card text is the sans with the path wrapped
"5 new / folders"; "Stop" floats alone on a line under "Esc Close"; the
state dot next to "Listening" is a pale salmon in light; in `07-after` the
card is gone but the hint "Typing does not answer. Press Alt+Y or Alt+N,
or..." is cut by an ellipsis and the field still says "A question is
waiting."; `06-hover` is byte-identical to `05-question` (no hover on
Yes/No); in `08-esc` the state still reads "Question for you" after the
question was answered; the running head and date are now readable.

## Findings, ranked

### MAJOR

**R1 [mc] Scene 2 "The Morning" stacks six empty texts on one spot.**
`s1-light-05-scene-2.png`, `s1-dark-05-scene-2.png`,
`s1-light-31-mouse-next-scene.png` (mouse path), `s2-light-05-scene-2.png`,
crop `c01-morning-empty-texts-stacked.png`. The M9 fix gave each panel of
`scenes/morning.yaml` an `empty:` text; all six are drawn at the same rect
origin `[160, 168]` instead of in their cells: `lead-empty` (33.8 px,
`[160, 168, 641, 112]`) over `day-empty`, `mail-empty`,
`overnight-empty`, `reading-empty`, `weather-empty` (19 px, all
`[160, 168, w, 28]`). Result: an unreadable smear "Nothing planned
today./No mail yet./Nothing overnight./..." with "No briefing yet. The
collector has not run." printed through it; "run." wraps alone. This is
the scene every first user reaches by "→ 2 The Morning" or Super+→, in
both themes and scales. Fix: place an empty text in its panel's cell (the
same rect the panel's lines would get), or show one empty line for the
whole scene ("No briefing yet. The collector has not run.") and hide the
other empties while the lead is empty; add a gate check that no two
visible mc texts overlap.

### MINOR

**R2 [host] The card is placed for a one-line field; when the field shows
two lines the card can cover the hint.** Seen once at 1280x800 light
(card `[888, 508, 336, 190]`, bottom rule y 697 drawn through "Typing does
not answer.", ink 688 to 702, while the field showed "A question is
waiting." on a second line; that frame was overwritten by a re-run).
Reproduced the mechanism: with the picker open and a one-line field the
card is at y 508 (`s1-light-32-question-after-reopen.png`); with the
transcript and a two-line field it is at y 488 (`s1-light-12-question.png`);
the y is chosen from the field's height, and when the card is laid out
before the field grows, the hint lands under the card. Same overlap,
always, in tiling (`tiling-04-question.png`, crop
`c02-tiling-card-over-hint.png`: card bottom 725, hint ink from 716), and
also in the baseline (`tiling-base-04-question.png`), so not a regression
there. Fix: lay out field, hint and card from one stack bottom-up each
frame (field height first), never from a cached field height.

**R3 [host] Light theme state dot uses the dark accent, 2.98:1.**
`s1-light-01-first.png`, crop `c03-light-dot-dark-accent.png`. The
figure next to "Listening" is `#cc7866` (the dark theme's accent) on
`#f7f5ed`: 2.98:1, below the 3:1 for a meaningful graphic, and off the
light palette (`accent #8c241f`). The word itself is `#8c241f` (8.06:1).
Dark is right (`#cc7866`, 5.23:1). Fix: take the figure colour from the
folder's `light` block, or tint `state.svg` with the theme accent.

**R4 [host] M4 remainder: state word flush right, no "Mute".**
`s1-*-01-first.png`. One state word now (mc's own state line is a blank
`[888, 14, 4, 28]`), 14 px, on the header baseline (word cap top y 22,
baseline 32, same as the running head), so M4's defects are gone. But the
design puts the state word at x 888 with "Mute" flush right (mc-v2.md
column section 1, shared: visible mute); the build draws "Listening" at
`[1132, 17, 68, 21]` flush right with a dot and has no Mute anywhere.
Fix: anchor the word at the slot's left edge, draw "Mute" (or "Microphone
off" / "Turn on") flush right, hover and click like every column item.

**R5 [host] M5 remainder: card content not the mc card.** crop
`c05-card-content.png`, `s1-*-12-question.png`. Fixed: page fill
(`#f7f5ed` / `#1c1c1c`), 1 px `text` rule, square, 336 wide, Yes/No 88x32
8 apart, 18 px padding top and bottom (title ink 506, buttons 628 to 660,
edge 678). Left: title "Question for you" instead of "Question from Atlas"
with time left flush right; prompt in the sans at 14 px, not the serif;
detail in the sans and wrapped mid-phrase "(412 bytes, 5 new / folders)",
not `mono`; key line "Press Alt+Y to allow or Alt+N to refuse." vs "Alt+Y
allow   Alt+N refuse"; no 14 px gap above the buttons (612 to 628 is the
line gap only). Fix: card style per mock 2 (title row, Pagella prompt,
mono detail that scrolls, key line in Inter 500).

**R6 [host] Reopening the chat returns to the picker.**
`s1-light-16-chat-reopen.png`, crop `c06-reopen-picker.png`. After Esc and
Super+a the column shows "1  Atlas · working" again, though there is only
one helper and the conversation was open a second ago; a sofa user must
pick again (type 1, Enter or click). Design: Super+a opens the chat, the
draft stays. Fix: reopen on the last helper; show the picker only when
there are two or more helpers or none was chosen.

**R7 [host] The hint is cut with an ellipsis while the question waits
without its card.** Builder's `mc-v2-r2/07-after-light-1280_800.png`, crop
`c11-hint-ellipsis.png`: "Typing does not answer. Press Alt+Y or Alt+N,
or..." (ink to x 1221) after a click on Yes, until the agent sends `done`;
the field still says "A question is waiting." and the card is gone. With
the real agent this lasts until `done` arrives, but rule 12 says text is
never cut, and a hint naming keys for a card that is not shown misleads.
Fix: when the card is answered locally, drop the hint and the field note
at once; wrap the hint instead of cutting it.

**R8 [host, chat] N3 remainder plus blur at scale 2.** crop
`c04-s2-field-blurry.png`, `s2-light-12-question.png`,
`s2-dark-11-chat-sent.png`. Known: the field is a boxed Arial-like face,
"Type a message, then Enter." in full `text` colour (13.77:1, a
placeholder should be `muted`), not "Write to Atlas ..." with Enter
flush right. New: at scale 2 the field text is rendered at 1x and scaled
up (soft edges, compare the crisp transcript above it); the second line
"A question is waiting." starts 4 px left of the first (x 896 vs 900, the
caret pushes line 1). Known not done (services/voice/chat); listed so the
blur is not lost.

**R9 [mc] No hover mark on "Esc  Close".** `s1-light-29-mouse-hover-close.png`
vs `s1-light-28-mouse-yes.png`, crop `c09-close-no-hover.png`: pointer at
the centre of `mc-r1-right-chat-close` `[1153, 182, 71, 28]`, the frame is
pixel-identical to the one before (all four runs). A click closes the chat,
so the item works, but the design wants the 2x20 bar on every item. Agenda
rows (`[146, 228, 2, 20]`), key rows (`[874, 636, 2, 20]`), the door
(`[874, 298, 2, 20]`) and the footer show it. Fix: give the head's Close
(and "Show") the same hit rect and bar as the other column rows.

**R10 [host] N4: no hover on Yes, No, Stop or the picker row (known).**
`s1-*-13-hover-yes.png` identical to `12-question`, `22`/`20` (Stop) and
`27`/`25` (picker) likewise, in all four runs. Known not done; still
MINOR.

**R11 [host, xp mostly] Scrolling drops the speaker line.**
`xp-04-question.png`, crop `c08-xp-orphan-reply.png`. When the question
card takes room in the xp balloon the oldest lines leave at the top, and
the first visible line is "I read 14 notes and made 5 days. Moving them"
without its "Atlas" label above (the baseline cut the reply with "..."
instead, `xp-base-04-question.png`, so wrapping is better now, but the
orphan is new). The mc design wants whole messages and a "↑ 3 earlier"
line; nothing marks the cut in any look. Fix: scroll by whole messages,
or repeat the speaker on the first visible line, and show the
"earlier" marker.

**R12 [mc, packaging] N1 remainder: Pagella missing on the host.**
All my captures. mc and the host now fall back to the same serif
(DejaVu Serif; families list in mc, `chat.font` fallback in the host), so
the page has one reading face; Pagella itself is not found by the host
(backlog: core fontconfig). Rated MINOR as agreed.

**R13 [other look, xp] "Starting chora-chat / Please wait." balloon never
ends.** `xp-03-chat-sent.png` to the last frame, over the Recycle Bin
label, from the first Super+a on. Same in the baseline
(`xp-base-04-question.png`), so not caused by this branch; noted because
it stands next to the host's balloon in every chat capture. Fix (other
branch): end the notice when chora-chat's field is up, or time it out.

### COSMETIC

**R14 [host] "Stop" sits alone on a line.** crop `c07-stop-own-line.png`:
`hud-chat-stop` `[1193, 224, 31, 20]`, a whole 28 px line between the
CHAT head (y 182 to 210) and "You" (ink 258), in `text` colour while
"Esc  Close" on the head is `muted`. N5 is otherwise fixed (no boxes, no
second Close). Fix: put Stop on the CHAT head row left of "Esc  Close"
in the same style, or on the state line while working.

**R15 [mc] C1 remainder: "Super+ ← →" spacing uneven.** crop
`c10-super-plus-arrows.png`: "+" ink 925 to 936, "←" 941 to 951, "→"
959 to 969: gap 4 px after the plus, 7 px between the arrows; the text
is "Super+ ← →" with a full space after "+", which reads as two keys.
Fix: "Super+←/→" or a thin space on both sides.

**R16 [mc] C3 remainder: door and its key 1 px apart.** `s1-light-01-first.png`:
"Chat with a helper ..." baseline 257 ("C" ink 244 to 257), "Super+a"
256 ("S" 245 to 256). Within the round shape overshoot of "C", so
nearly fixed; keep an eye with Pagella.

**R17 [mc] C5 remainder: chapter "1" top 93, "AGENDA" 94.**
crop `c12-chapter-number-tops.png`; bottoms both 126. Fallback face only.

**R18 [host] C6: picker row "1  Atlas · working" (known).**
`s1-light-09-chat-picker.png`: Inter 14 at `[888, 224, 336, 20]`, a
20 px row off the 28 px grid, no hover bar, the digit unexplained. Known
not done.

**R19 [design doc] N9: Super+F1 vs F1 decided in the repo, not in the
design.** `docs/mc-style.md` line 110 explains Super+F1 ("a plain F1
belongs to a program"); `design/looks/mc-v2.md` and mock 1 still say "F1
All keys". Fix (PM): update the design text.

## First-round findings, re-checked

| # | Area | Status | Evidence |
|---|---|---|---|
| M1 chat over the page | host | fixed | `hud-chat [888, 210, 336, 562]` = slot from `mc.chat.slot`; page untouched (`s1-*-11`) |
| M2 host rows over mc text | host | fixed | no `hud-row-*` node; no host image over an mc text in any frame |
| M3 "Chat ..." pill | host | fixed | gone in all mc frames |
| M4 11 px word, two state words | host | fixed; position left as R4 | word 14 px, cap 22 to 32 on the header baseline; one word |
| M5 stock card | host | partly, MINOR R5 | page fill, square 1 px rule, 336 wide; content and type still not the mc card |
| M6 chat jumps, field behind | host | fixed | chat rect unchanged with a question; card does not move while hovered; R2 is the remaining layout race |
| M7 Inter chat, ellipsis | host | fixed | serif 19 px, speaker on its own line, reply wraps; hint ellipsis is R7 |
| M8 11.6 px head | mc | fixed | `mc-r1-header-head`/`clock` 14.25 px; smallest mc text in all frames 14.25 |
| M9 blank scenes | mc | partly; regression R1 | agenda footer "→ 2 The Morning" and empty texts exist; on scene 2 they stack |
| N1 serif fallback | mc | fixed; Pagella R12 | DejaVu Serif, not Sans |
| N2 hover through chat | host | fixed (moot) | chat in the column; no mc item under it |
| N3 field | host/chat | not (known), R8 | |
| N4 card hover | host | not (known), R10 | |
| N5 duplicate Close, bold buttons | host | partly, R14 | one Close (mc's), Stop plain text |
| N6 Esc after a question | host | fixed | `s1-*-15-esc-after-yes`: chat closed by Esc after a click on Yes |
| N7 notification rows | mc | fixed | rows 28 apart (y 166, 194, 222), time Inter `muted` |
| N8 Super+F1 order | mc | fixed | by task, words of the short list |
| N9 F1 vs Super+F1 | mc/docs | decided in repo, design doc open (R19) | |
| N10 "Nothing new." colour | mc | fixed | `#6e6961`, 4.99:1 |
| N11 helper state in accent | host/mc | fixed | sign green `#407554`, "working" `muted` |
| N12 dark chat light | host | fixed | dark chat on `#1c1c1c` in both dark runs |
| C1 Super+← → | mc | partly, R15 | |
| C2 header baselines | mc | fixed | running head and state word baseline 32 |
| C3 door/key baseline | mc | nearly, R16 | 1 px |
| C4 NOTIFICATIONS without count | mc | fixed | "NOTIFICATIONS  3  Show" / "0" |
| C5 chapter tops | mc | not (fallback face), R17 | |
| C6 picker row | host | not (known), R18 | |

## Passes (checked, no finding)

- Column geometry at 1280x800 and at scale 2: hairline x 860 (1720 at
  2x), column 888 to 1224, keys rule at y 562, key list foot 772; the
  page does not move when the chat opens (text block identical in
  `s1-light-08` and `11`).
- Contrast, light / dark: text 13.77 / 11.53, muted 4.98 / 4.93, accent
  word 8.06 / 5.23, amber "?" 4.70 / 8.67, card rule 13.77 / 11.53. Only
  R3 (dot) is under its threshold.
- Keys and clicks: Super+F1 toggles the full list; Super+→ and Super+Home
  turn pages; Super+a opens the chat in the column; a sent message shows
  at once; Alt+Y and Alt+N answer and take the card away; a click on Yes
  and on No answers (recorded); Stop calls `agent.cancel`; Esc closes the
  chat after a question; mouse-only path door → helper → Yes → Close →
  next scene works.
- Scale 2: mc text, transcript, card and hairlines sharp (except R8).
- Other looks with this build: tiling opens the chat under the bar
  (`tiling-02-chat-open.png`, `hud-chat [880, 26, 400, 774]`, same as the
  baseline), replies now wrap instead of "..." (baseline cut "needs your
  y..."), speakers regular weight; xp balloon with question works
  (`xp-04-question.png`), card 312 wide inside the balloon, reply wraps
  (baseline cut it). No regression found besides R11's orphaned label.
