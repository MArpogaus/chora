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
14 px text, 24 px targets and honest state words.

## Sizes: XP's Large Fonts

1. **One reading size, 14 px, line 20 px.** All text of the chrome is
   Tahoma: the copy of the theme, or XP's own when it is installed. Bold
   is for titles, names and the pinned rows only. The heading of a Help
   page is 18 px.
2. **Grid unit 4 px, paddings 8 or 12.** Taskbar 36, task button 180 by 28
   with 4 between, title bar 30, frame 4, caption buttons 24 by 24 with 2
   between, menu rows 36 (pinned 48), push buttons 80 by 28 with 8
   between, desktop cells 80 by 88, balloon 340 wide with radius 8.

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
   Escape closes it.
5. **The key table is generated.** It is made from the bindings that the
   plugin holds (`help::rows`), grouped "Start and find", "The helper" and
   "Windows", one key cap per chord: button face, 1 px `#8C8A7F`, 2 px at
   the bottom. A binding that the plugin does not hold is not listed, and
   a test fails when a bound key is missing from the table.
