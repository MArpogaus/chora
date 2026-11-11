# Design per look: xp v2 (proposal)

Proposal for the PM, 2026-11-11, build 06044cf. Builds on `xp.md`, `shared.md` and
`review-checklist.md`; where this page differs from `xp.md` it says so and wins once accepted.
Mocks (labelled MOCK, 1280x800): `media/2026-11-12/design/xp/xp-v2-mock-{1-start-menu,
2-chat-question,3-help-keys}.png`, HTML and `xp-v2.css` beside them. Stills: `media/2026-11-12/tour/stills/`.

## Philosophy

xp is the desktop people already know: Start bottom left, a taskbar of buttons, a tray with the
clock, title bars with three buttons, a friendly paperclip in the corner. Mouse first; every door is
on screen and keys are extras. Faithful in shape, colour and place, never in trademarked art (no
Microsoft flag, Bliss, Clippit or "Windows" name). One rule decides taste questions: *would a person
who used XP for years notice something off?* If yes, it is a finding. The second rule is chora's
own: 14 px text, 24 px targets, honest state words, one helper surface.

## Polish list: what is off in the stills today

Measured at 1x on the 1280x720 stills; path prefix `media/2026-11-12/tour/stills/`.

| # | Still | What is off | Fix |
|---|---|---|---|
| 1 | 01-02-start-menu | Program subtitles are raw commands: "foot", "chora-notes", "chora-wobbly" | Subtitle only on the two pinned rows, plain words ("Type commands"); recent rows have none |
| 2 | 01-02 | Every program shows the terminal glyph, also Notes, Chart, Page | Icon from the desktop file; fallback a neutral window glyph, never the terminal |
| 3 | 01-02 | 120 px of empty white between "Page" and the All Programs rule (y 500-612) | Menu height follows its rows; no filler |
| 4 | 01-02 | Header says "User" | The account's full name (GECOS), else the login name |
| 5 | 01-02 | Right column mixes pale line-art folders with a saturated My Computer | One icon set, 24 px, full colour (mock 1) |
| 6 | 01-02 | Menu text 11-12 px; Control Panel opens only from its arrow (swarm 3, #5) | 14 px; the whole row opens the submenu (D9) |
| 7 | 01-02 | No "Help and Support", no "Chat with a helper" in the menu | Both added (mock 1, D7) |
| 8 | 01-01-desktop | "My Documents" label starts at x 2, icon top y 5; Recycle Bin label ends 13 px from the edge | Icon grid of 80 x 88 cells from 8,8; right column 8 px from the edge |
| 9 | 01-01 | Tray padding uneven: speaker 6 px from the tray edge, clock 21 px from the screen edge | 12 px both ends, 8 px between items |
| 10 | 01-08-muted | "40%" becomes "Muted": the tray text changes width, the speaker glyph stays the same | Speaker icon only, red no-sign when muted; level and "Muted" in the tooltip (D12) |
| 11 | 01-01 | Pager "1 2 3 4" has no words (xp.md "Don't") | Keep (D14), tooltip "Desktop 2 of 4", 24 x 24 buttons |
| 12 | 01-06-two-windows | Caption buttons 20 x 20 with 4 px gaps (x 687-754); swarm: "small window buttons" | 24 x 24, 2 px gaps, 4 px from the frame edge (D2) |
| 13 | 01-06 | Title text about 12 px | 14 px Tahoma Bold, white, 1 px shadow #0A1E50 at 1,1 |
| 14 | 01-10-paperclip-balloon | Balloon tail ends about 15 px above the paperclip's head | Tail tip on the head (xp.md rule) |
| 15 | 01-10 | Balloon text 12 px; padding 10 left, 12 top; widow "ready." on line 2 | 14 px, 12 px padding all round, width fits the text up to 340 |
| 16 | 01-10 | Paperclip 2 px above the taskbar (`hud.yaml` margin [96, 2]) and 40 px left of the Recycle Bin: two objects compete | Paperclip 8 px above the taskbar, 8 px from the right; Recycle Bin above it, same centre line (D15) |
| 17 | 01-11-chat-open | Border #7F9DB9 (an edit box colour) while the notify balloon has #000: two balloons, two borders | #000 1 px for every balloon (D8) |
| 18 | 01-11 | Field fill #FFFFE1: it does not read as a field | White field, 1 px #7F9DB9, 28 px high |
| 19 | 01-11 | Row "1 Atlas · working": an index number and a middle dot | Header "Atlas — Working", task on line 2 in grey |
| 20 | 01-11, 01-14 | Close is a 88 px text button beside Stop, both the same weight | XP balloon close box (18 px, top right); Stop a push button left of it |
| 21 | 01-14-helper-picked | Replies cut with "..." mid-sentence, then a stray "summary." line | Wrap every line whole; the transcript scrolls, it never truncates |
| 22 | 01-14 | "Good morning! ..." pinned bold at the bottom, out of order | Spoken lines go into the transcript at their time, regular weight |
| 23 | 01-16-typed, 01-17-sent | "Please also add Monday" is sent but never appears | Own message shows under "You" in the frame after Enter |
| 24 | 01-18-question | Card #FAFAFA, radius 8, grey #E8E8E8 buttons with #3C3C3C border: not XP | White group box, 1 px #ACA899, radius 3; XP push buttons (mock 2) |
| 25 | 01-18 | Title "Question for you", no time left (shared order) | "Question from Atlas", time left right-aligned in grey |
| 26 | 01-18 | 40 px empty below Yes/No; "Typing does not answer." outside the card; field shows two lines | Card padding 12; the field's placeholder alone says "A question is waiting above." |
| 27 | 01-19-answered, 01-19b | Click on Yes leaves no trace; the transcript does not say what was answered | Line under "You": "Allowed: write notes.txt" (or "Refused: ...") |
| 28 | 01-19b | Stop stays enabled when no prompt runs (swarm "Stop state") | Stop enabled only while a prompt runs, greyed otherwise (D10) |
| 29 | 01-20-f1, 01-21 | F1 types `^[OP` into Notes; no key list anywhere | F1 opens Help and Support (D3, mock 3) |
| 30 | 01-11..19b | Chat glyphs look wider than the menu's Tahoma (compare "y", "t" with 01-02) | One font file: the HUD folder's Tahoma; verify with a glyph diff |
| 31 | 01-12-chat-collapsed | Balloon caught at about 50 % alpha, text near 2:1 | Fade 160 ms, never rests translucent; capture after 200 ms must be opaque |

Behaviour found by the swarms, no still: Super alone and Super+D do nothing; after a minimize the
taskbar keeps the minimized window's button pressed. Both are in the build list.

## Missing affordances, in the xp idiom

**Help and Support (F1).** An ordinary XP window (mock 3), not an overlay: title "Help and Support",
blue task pane on the left (topics: Start a program, Find a window, Chat with a helper, Change the
volume, Keyboard shortcuts), the page on the right. F1 and Super+F1 open it on "Keyboard shortcuts";
Start > Help and Support opens it on the first topic. It is a window: taskbar button, Alt+Tab,
move, close, and Esc closes it. The key table is generated from the live bindings, grouped "Start
and find", "The helper", "Windows", one key cap per chord (XP button face, 1 px #8C8A7F, 2 px bottom).

**Start menu doors.** Pinned (top left, bold name and grey subtitle): Terminal / "Type commands",
Chat with a helper / "Ask Atlas in your words" (paperclip-in-balloon icon). A rule, then recent
programs without subtitles. Right column gets "Help and Support" above "Run...", as in XP.

**The helper.** Three doors to the one balloon: the paperclip, the Start entry, and a 16 px tray
icon (mini paperclip) whose tooltip is the state word; Super+A as the key. The tray icon shows a red
no-sign while the microphone is off, so a muted mic is visible even with the balloon closed.

**The chat (mock 2).** The balloon expanded: header (name, em dash, state word; task and step on
line 2; Stop button; close box), a 1 px #D8D5B0 rule, the transcript (speaker names bold #003C74,
tool steps grey with a small page glyph, indented 20), the card when a question waits, then the
field and a Send button. Enter sends too. Grows up from the paperclip; height up to 60 % of the
work area, transcript scrolls; never over the taskbar.

**Keys XP users expect.** Super alone (pressed and released with no other key) opens or closes
Start. Super+D shows the desktop; pressed again, the same windows come back. Ctrl+Esc stays.

## Palette (Luna homage, sampled from the stills)

| Token | Value | Use |
|---|---|---|
| taskbar | #245DD7 to #2662DF, top line #3F8CF3, bottom #1941A5 | taskbar, 36 px |
| start | #6DBF63 to #1A6A1E | Start button |
| tray | #0F8EE9 to #0D9DF0, left edge #0B5BB8 + #3DB7F9 | notification area |
| title | #0A6CF5, #0052D1 at 12 px, #0062FC at 27 px | active title bar |
| title-off | #A8C2F0 to #7B97E0, text #D8E4F8 | inactive title bar |
| menu | header #0F61CB to #3F8BE9, orange rule #E8913A, right #D3E5FA, foot #438BE3 to #0F60CD | Start menu |
| select | #316AC5 on white text | menu hover and selection |
| balloon | #FFFFE1, border #000 | balloon, both forms |
| button | #FFFFFF to #E3E1D7, border #003C74, hover ring #F8B330 | push buttons |
| edit | #FFFFFF, border #7F9DB9 | fields |
| name | #003C74 | speaker names, links |
| ink-2 | #4D4D4D (8.1:1 on #FFFFE1, 8.5:1 on white) | subtitles, grey lines |
| pane | #7BA2E7 to #6375D6, groups #D6DFF7, links #215DC6 | Help task pane |

## Type and sizes

- One family: Tahoma (the Wine copy; XP's own Tahoma when the user installed it). Bold for titles,
  names and pinned rows only; no italic except the Start label (Libre Franklin Bold, 21 px).
- One reading size, 14 px, line 20 px; the Help page heading 18 px. That is all.
- Grid unit 4 px; paddings 8 or 12. Taskbar 36; task button 180 x 28, 4 apart; title bar 30;
  frame 4; caption buttons 24 x 24, 2 apart; menu rows 36 (pinned 48), icons 32 left / 24 right;
  push buttons 80 x 28, 8 apart; desktop cells 80 x 88; balloon 340 wide, radius 8, tail 23 x 17.

## Taste decisions (for the PM)

- **D1 Scale: XP "Large Fonts".** All chrome text 14 px and the bars grow about 1.2x (taskbar 36).
  Why: shared 14 px minimum and P7; XP itself offered this. Rejected: native 11 px (unreadable
  for Erika, breaks shared.md); a 1.25 output scale (fractional, blurs the paperclip).
- **D2 Caption buttons 24 x 24.** Why: P7 24 px targets, swarm finding. Rejected: XP's 21 px.
- **D3 F1 is global.** Why: P1, same key as construct; the swarm pressed it. Rejected: XP's
  per-app F1 (the terminal swallowed it as `^[OP`).
- **D4 Help is a window.** Why: least surprise for XP users, works with taskbar and Alt+Tab.
  Rejected: a full-screen key sheet (construct's idiom, foreign here).
- **D5 Key name "Super".** Why: shared key names; "Windows logo key" is a trademark. The help
  says where it sits ("between Ctrl and Alt").
- **D6 Super alone opens Start; Super+D toggles the desktop.** Why: XP habit, swarm finding.
- **D7 Chat pinned second, Help in the right column.** Why: P3, a visible door where XP users
  look. Rejected: chat only under All Programs (hidden).
- **D8 Black balloon border, white field.** Why: XP balloons are black-edged; a field must read
  as a field. Rejected: #7F9DB9 (looks like an edit box). Changes `hud.yaml` `border`.
- **D9 Control Panel opens from the whole row** on hover after 400 ms (XP's menu delay) or a click.
- **D10 Stop greyed, not hidden, when idle.** Why: no layout jump; XP greys. Rejected: hiding.
- **D11 Send button.** Why: mouse first. Rejected: Enter only.
- **D12 Tray volume as icon only.** Why: XP, and no width jump. Rejected: "40%" text.
- **D13 No default button in the card.** Why: Enter must not answer (shared). Rejected: XP's
  blue default ring on Yes.
- **D14 Keep the pager.** Why: XP's Virtual Desktop PowerToy had it; the cube needs a mouse door.
  Rejected: removing it.
- **D15 Recycle Bin above the paperclip** on one centre line. Rejected: side by side (crowded).

## Build list (ranked, small steps)

1. **F1 and Super+F1 open Help and Support** on Keyboard shortcuts, generated from the bindings.
   Check: F1 in Notes opens the window, Notes receives nothing; every bound key is in the table.
2. **Chat transcript is whole.** No truncation, own messages appear, spoken lines in order, the
   answer to a question is a line. Check: 01-14..19b replayed; every sent line visible, no "...".
3. **Stop reflects a running prompt.** Check: greyed when idle, enabled within one frame of a send.
4. **Question card in XP form** (title, time left, group box, push buttons, no default button).
   Check: matches mock 2 within 1 px at 1x; Yes and No 80 x 28, 8 apart; Enter does not answer.
5. **Start menu doors and subtitles.** Chat pinned, Help and Support, no raw commands, real icons,
   height follows rows. Check: no subtitle equals an Exec name; no empty band over 12 px.
6. **Control Panel row opens its submenu.** Check: hover on the label 400 ms opens it; click too.
7. **Super alone opens Start, Super+D toggles desktop.** Check: Super+key chords do not open Start.
8. **Taskbar state after minimize.** Check: minimized button raised; the newly focused one pressed.
9. **Caption buttons 24 x 24, title 14 px.** Check: measure in a 1x capture; 2 px gaps.
10. **Balloon: black border, white field, Send, close box, header words.** Check: one border colour
    for chat and notify balloons; field white; header has no digit or middle dot.
11. **Paperclip and icons placement.** Clip 8/8 from taskbar and edge; tail on the head; icon grid
    80 x 88 from 8,8; bin above clip. Check: 1x capture, centres equal to the pixel.
12. **Tray: padding 12/8, icon-only volume, helper tray icon.** Check: tray width equal muted
    and unmuted; tooltips show "Volume 40%" / "Muted" and the helper's state word.
13. **14 px everywhere, one Tahoma.** Check: glyph diff of chat vs menu text; no text under 14 px.
14. **Fade never rests translucent.** Check: capture 200 ms after show is fully opaque.
15. **Pager tooltips and 24 px buttons.** Check: hover shows "Desktop 2 of 4".

## Open questions

- Super+E opens the terminal today; XP users expect a file manager. Keep (the terminal is the
  pinned program) or follow XP when a file manager is installed? This draft keeps it.
