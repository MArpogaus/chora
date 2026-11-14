# chora-xp style guide

These rules apply to every surface of `chora-xp`: the taskbar, the Start
menu, the title bars, the dialogs, Help and Support, and the helper's
balloon. The model is the desktop that people used for years: Start at the
bottom left, a taskbar of buttons, a tray with the clock, title bars with
three buttons and a paperclip in the corner. It is faithful in shape,
color and place, never in trademarked art: no Microsoft flag, no Bliss, no
Clippit, no "Windows" name.

One question decides taste: would a person who used XP for years notice
something off? If yes, it is a finding. The second rule is chora's own:
honest state words.

## Sizes: XP Normal at 96 DPI

1. **One reading size, 11 px (8 pt Tahoma).** All text of the chrome (Start
   menu rows, taskbar, tray, menus, tooltips) is 11 px; titles are Trebuchet
   Bold 13 px and the label of the Start button 17 px. It is all Tahoma: the
   copy of the theme, or XP's own when it is installed. Bold is for titles,
   names and the pinned rows only. Text that a person must read at length
   (the helper's balloon, Help pages) stays at 14 px. XP's "Large size"
   (120 DPI) is not the default: it is the scale 125 % of Display Properties.
2. **Grid unit 2 px.** Taskbar 30, Start button 99 by 30, task button 160 by
   24 with 2 between, title bar 25, frame 4, caption buttons 21 by 21 with 2
   between, menu rows 22 (Start menu: pinned 42, programs 34, places 26),
   push buttons 73 by 21, desktop-button pager 22 by 22, Start menu 380 wide
   with a left column of 190, balloon 340 wide with radius 8.

## Colors

3. **Luna.** Taskbar `#245DD7`, Start `#6DBF63` to `#1A6A1E`, tray
   `#0F8EE9`, active title `#0A6CF5`, selection `#316AC5` with white
   text, balloon `#FFFFE1` with a black 1 px border, push button white to
   `#E3E1D7` with `#003C74`, fields white with `#7F9DB9`, names and links
   `#003C74`, grey text `#4D4D4D`.

## Help and Support

4. **An ordinary window.** The title is "Help and Support"; the blue task
   pane at the left lists the topics (Start a program, Find a window, Chat
   with a helper, Change the volume, Keyboard shortcuts), the page is
   white at the right. F1 and Super+F1 open it on Keyboard shortcuts;
   Escape closes it. It has a button on the taskbar, which raises it, and
   an entry in Alt+Tab, first while it has the focus.
5. **The key table is generated.** It is made from the bindings that the
   plugin holds (`help::rows`), grouped "Start and find", "The helper" and
   "Windows", one key cap per chord: button face, 1 px `#8C8A7F`, 2 px at
   the bottom. A binding that the plugin does not hold is not listed, and
   a test fails when a bound key is missing from the table.

## The Start menu

6. **Two pinned rows, then recent programs.** The pinned rows are 42 px
   high with a bold name and a grey line of plain words: Terminal "Type
   commands", Chat with a helper "Ask in your words". The
   rows below a rule are 34 px high, in regular weight, with no subtitle:
   a raw command is never a subtitle. The right column has Help and
   Support above Run. The height of the menu follows its rows; when the
   settings list too few programs for the left column to reach the right
   one, the first programs of All Programs fill it, so that no white band
   stands over All Programs. The menu is 380 px wide, the left column 190.
7. **The header names the person**: the full name of the account, else
   its login name.

## Keys XP users expect

8. **Super alone opens Start**, Super+d shows the desktop and brings it
   back, Ctrl+Escape stays, Super+e opens the file manager (a balloon
   says so when none is installed), Super+Space opens the launcher.
   The name of the key is "Super"; the help says that it sits between
   Ctrl and Alt.

## Title bars

9. **Caption buttons are 21 by 21 with 2 px between**, 4 px from the right
   edge of the frame and 2 px below its top, so that a finger or a pointer
   hits them (principle P7). The title is Trebuchet Bold 13 px in white with a
   1 px shadow `#0A1E50` one pixel to the bottom right; an inactive window
   has the title `#D8E4F8`.

## Taskbar and tray

10. **The tray has icons, not words.** 12 px of room at the left, a 16 px
    helper icon (a small paperclip: a click opens the chat, the tooltip is
    the state word, a red no-sign shows while the microphone is off), 8 px,
    a 16 px speaker icon, 8 px to the clock. Muted or no sound shows a red
    no-sign; the width does not change. The words and the level are in the
    tooltip, which has the pale yellow fill and a 1 px black border of every
    balloon, 11 px text, and shows after 500 ms above the part under the
    pointer. The desktop buttons are 22 by 22, 2 px apart, with the tooltip
    "Desktop N of 4".

## Desktop and helper

11. **A grid of 80 by 88 pixel cells from 8, 8.** The Recycle Bin stands
    at the right edge, 8 px from it. The paperclip stands 8 px from the right
    edge and 8 px above the taskbar, the bin 8 px above the paperclip on
    the same centre line. The tail of the balloon ends on the head of the
    paperclip, and the border of every balloon is black.

## The helper's balloon

12. **Stop is greyed, not hidden, when no prompt runs** (`idle_stop: grey` in
    the `chat:` block of the HUD folder), so that the header does not jump;
    it takes a press only while a prompt runs.
13. **The balloon has XP's form** (`form: xp`): a header of two lines (the
    name in bold with the state word after an em dash, then the task and
    the step in grey), a close box of 21 px at the top right, Stop and Send
    as push buttons, the field in white with a 1 px rule of `#7F9DB9`, and
    the question as a white group box with Yes and No as push buttons of
    80 by 28 px, 8 px apart. No button is the default one.
14. **The balloon fades in 160 ms** and never rests half transparent: a
    capture 200 ms after it went finds it gone or whole.
