# Deep UX review: xp, merged look (head 8f358fa)

Reviewer run, 2026-11-12. `tests/persona/session.sh xp` on a build of 8f358fa (debug), driven with `act.py`
(clicks and keys only) as a first-time user. Scale 1: 1280x800; scale 2: 2560x1600@2 (logical 1280x800).
Fake helper "atlas" as in `tests/gate-mc-host.sh` (`tests/fake-voice.py`, Alt+y/Alt+n). Captures:
`media/2026-11-12/ux-deep-xp/xp1x-*` (scale 1) and `xp2x-*` (scale 2); cited as [name]. Coordinates are
logical pixels at scale 1 (scale 2 captures are twice as large). Judged against `design/looks/xp-v2.md`,
`shared.md`, `product-briefs.md`, `decisions-taste.md`.

Test caveats (not blamed on xp): (a) the first run had no `--test-injected-press-is-real`, so injected
clicks did not give the chat field the keyboard and typed text went to the terminal [xp1x-36]; every
helper capture after that comes from a second run with the flag. (b) The injector releases Alt at once, so
the Alt+Tab switcher, if it shows only while Alt is held, cannot be seen by me. (c) No sound card here
(tray says "Sound unavailable"), `choractl quit` and `systemctl poweroff` were not run to the end
(Turn Off was cancelled). (d) The fake helper sends no deadline and no state after an answer.

What is right (measured or seen): Start menu now matches the polish list: 14 px rows, "Terminal / Type
commands" and "Chat with a helper / Ask in your words" pinned, Help and Support and Run in the right
column, no filler band, whole Control Panel row opens its submenu on hover and click [xp1x-02, 04]; start
mark and user picture are own art; Super tap (Super_L) opens Start [xp1x-14]; Super+D hides all windows and a
second press brings them back, buttons raised/pressed correctly [xp1x-12]; minimize leaves the button
raised [xp1x-50]; maximize fills the work area down to the taskbar, paperclip stays above [xp1x-51];
Turn Off dialog opens, dims the desktop, Esc cancels [xp1x-23]; double click on the clock opens a working
Date and Time Properties window [xp1x-29b]; the helper chat balloon follows the mock (header "Atlas —
Working", task line 2, Stop left of the close box, Send button, transcript with "You"/"Atlas") [xp1x-43];
the question card is a white group box with 80x28 push buttons 8 apart and no default ring, Yes by click and
Alt+n by key both answer [xp1x-44, 46]; a notify balloon has a black 1 px border and closes on a click
[xp1x-48]; scale 2 is crisp, no blur on icons or the paperclip [xp2x-02, 05].

## MAJOR (ranked, main paths first)

| # | Where | Flaw | Evidence |
|---|---|---|---|
| 1 | Lock | No way to lock the screen: Start footer has only Log Off and Turn Off Computer; Turn Off offers Stand By, Turn Off, Restart; the Help key table has no lock key. RC (c) fails for xp. | [xp1x-02] footer y 742; [xp1x-23] dialog x 484-797, y 301-499; [xp1x-17] |
| 2 | Log Off | A click on "Log Off" (180, 742) closes the menu and nothing else happens: no dialog, no balloon, session stays; focus is lost (both windows draw as inactive, no button pressed) and stays lost. RC (d). | [xp1x-25], [xp1x-26] (2.5 s later) |
| 3 | Run (Start > Run..., Super+Space) | A bare launcher panel x 406-874, y 241-559 in the xp palette, no title bar, no OK / Cancel / Browse, no "Open:" combobox look. Type a word with no match: the list vanishes, an empty 470x320 box stays, and Enter closes it with no message at all. First-time user cannot tell what failed. | [xp1x-05], [xp1x-07], [xp2x-09] |
| 4 | F1 | The first F1 I pressed (after Alt+Tab, two Super+D, Esc) went to the terminal as `OP` (the `^[OP` of the swarm) instead of opening Help; later F1 presses opened Help (toggle: second press closes it). Cause unclear, so intermittent: the bound key must never leak. Also the table says "Super+F1 Open this help" and does not list plain F1. | [xp1x-15] text at (184, 73); [xp1x-17] row y 308 |
| 5 | Helper without voice | Click on the tray clip with no helper: a 340 x 455 balloon (x 941-1280, y 307-763), nearly empty, title cut "Cannot reach voice control. Tryin...", no paperclip drawn, no Send or Stop, field at y 728-763 narrower than the balloon. The honest state is half there and unreadable. | [xp1x-31] |
| 6 | Notification vs helper | A notify balloon (x 927-1234, y 709-759) sits on top of the paperclip's head; text is about 12 px, below the 14 px floor of D1/shared.md. | [xp1x-48] |

## MINOR

| # | Where | Flaw | Evidence |
|---|---|---|---|
| 7 | Question card | No time left (shared order: title, time, prompt ...); "Typing does not answer." stands outside the card at y 530 (polish 26); the field shows two lines (placeholder plus "A question is waiting."); card text (DejaVu-like) and field placeholder (Arial-like) are not Tahoma. | [xp1x-44], [xp2x-05] |
| 8 | After an answer | The transcript has no line for the answer ("Allowed: ..."), polish 27. Header stays "Question for you" only because the fake sends no state. | [xp1x-46] |
| 9 | Help window | Esc does not close it (doc: Esc closes); only a close box (x 1005-1027, y 37-57), no minimize or maximize; first topic leaves the right pane empty below y 230; key table ends at y 712 and a scroll shows no further rows, so Print and volume keys (if any) are unreachable. | [xp1x-16], [xp1x-18], [xp1x-19] |
| 10 | Caption buttons | 22 x 22, 4 px apart, 6 px from the right edge (spec 24 x 24, 2 apart, 4 from edge). Measured on row y 14: buttons x 1201-1222, 1228-1248, 1254-1274 (maximized). | [xp1x-51], [xp1x-10] |
| 11 | Turn Off dialog | Cancel is about 56 x 16 at (728-784, 470-486), under the 24 px target; icon captions about 13 px. | [xp1x-23] |
| 12 | Date and Time | Text and calendar about 12 px, OK / Cancel / Apply 72 x 20; single click on the clock does nothing; the dialog has no taskbar button (Help has one). | [xp1x-29b] |
| 13 | Chat reopen after no-helper | After the no-helper balloon, the next balloon with the helper list kept the field at y 728-763, detached from the balloon (y 471-621), over the clip. Fixed after a restart. | [xp1x-33], [xp1x-36] |
| 14 | Recycle Bin | Hidden under the chat balloon while it is open; label ends at x 1267 (13 px from the edge, polish 8 asked for 8). | [xp1x-40], [xp1x-01] |
| 15 | Desktop labels | Icon labels and title text about 12 px ("My / Documents" two lines 13 px apart). | [xp1x-01] |

## COSMETIC

- Start menu: "ImageMagick (color depth..." cut and its icon is tiny and offset; "Chat with a helper" icon is a
  green dot, not the paperclip-in-balloon of the mock [xp2x-02].
- Control Panel submenu: "Effects" and "Keyboard Shortcuts" rows mix icon and no-icon rows (x 427-443)
  [xp1x-04]; header says "User" (login name of the harness).
- Both windows (foot and Vim) are titled "foot"; program name is not shown [xp1x-10].
- Alt+Tab gave a focus change with no visible switcher in my 600 ms capture [xp1x-11]; please verify by a
  test that holds Alt (brief: "Alt+Tab shows a switcher").
- Pager tooltips ("Desktop 2 of 4") not checked.
- "Sound unavailable" tooltip is honest and well placed [xp1x-28b]..

## Brief check (RC list of chora-xp)

Start menu, taskbar, tray and caption buttons work: yes. F1 opens a key and help sheet in xp style: yes,
via Help window, but one leak (4). Helper from tray and Start: yes. Super opens Start: yes. Super+D: yes.
Alt+Tab switcher: not verified. Taskbar follows focus and minimize: yes. Turn Off dialog: yes. Menu labels
open submenus: yes. Shared RC (a) key list and chat door: yes; (c) lock: no (1), screenshot: no door (no key
in the table, no Start row), volume: tray only and unavailable here; (d) honest failures: Log Off (2), Run
no-match (3), no-helper (5) fail.

## Suggested order for next week

1 Lock door (Start footer and Turn Off) and a Log Off that answers; 2 Run window with title bar,
OK / Cancel and a "no match" word; 3 swallow every bound key whatever the focus state (F1); 4 no-helper
balloon text and size; 5 notify balloon 14 px and clear of the clip; 6 card details (time, Typing line,
Tahoma, answer line); 7 sizes: caption buttons 24, Cancel 24 high, 14 px in dialogs.
