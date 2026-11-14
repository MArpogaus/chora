# tiling findings (train 4, tiling f9dabd84) | id | severity | source:line | state | text | helper door, lock, power, volume, key list: see shared.md
ti-1 | BLOCK | fb-swarm5-t5m:8 | verify | Apps menu: click and hover leave the selection on Screenshot, no program starts by mouse (fuzzel 1.9.2 suspected); RC: start a program by click
ti-2 | MAJOR | ux-deep-tiling:57 | open | Apps launcher: Screenshot first and pre-selected (Apps then Enter takes a screenshot); no sort; empty search shows an empty panel with no word
ti-3 | MAJOR | swarm-6:25 | verify | Bar Keys and Chat buttons do nothing on click (haiku lead); Keys click worked in ux-deep, Chat button no longer exists: recheck on train 4
ti-4 | MAJOR | ux-deep-tiling:51 | verify | Screenshot gives no notice: screenshot.sh needs a notification daemon; mako unit now ships (chora-tiling-mako), confirm the notice shows
ti-5 | MINOR | pixel-tiling-r4:32 | open | A toast whose mako starts while the chat is open is fully hidden under the chat panel (x896-1255 under 868-1268); read chat state at start
ti-6 | MINOR | ux-deep-tiling:66 | open | Tooltips Apps "Run a program" and Keys "Every key" stay after click or Esc (3 s late over fullscreen); set tooltip false on Apps, Keys, power
ti-7 | MINOR | ux-deep-tiling:73 | open | Fullscreen: the bar floats over the window and covers the first terminal line; put the bar under a fullscreen window
ti-8 | MINOR | ux-deep-tiling:78 | verify | New windows do not take the focus (Super+Return twice leaves typing in the first); focus new windows or say it in the key list
ti-9 | MINOR | ux-deep-tiling:82 | verify | Key list 692 px high (runs out on 720); "Window that asks" unexplained, say "Window with a question" (4f227619 reworked the grid)
ti-10 | MINOR | ux-deep-tiling:86,91 | verify | Key list translucent over seams; layout chip sat on the seam between tiles (e23209b5 moved chips beside the chat)
ti-11 | MINOR | ux-deep-tiling:95; backlog:60 | open | After settings.set look.theme, running programs and assistant surfaces keep the old palette (Ink and Paper side by side)
ti-12 | MINOR | swarm-6:26; fb-kai6:37 | open | No focus-left/right key; Super+Right switches the tag and Super+l widens the master (awesome keymap, by design): list it or decide
ti-13 | MINOR | fb-erika-r2:1; fb-swarm2:214 | verify | Bar network shows the address, not "Connected"/"Not connected"; Tile and sound words unlabelled and inert
ti-14 | MINOR | fb-swarm5-t5m:30 | open | Key list shows only keyboard chords, no mouse basics (low)
ti-15 | COSMETIC | pixel-tiling-r4:12-20 | open | 17 open pixel items: toast LCD fringes, orb ring, toast type, chat header, launcher size, tooltip shadow, 1.25 hairlines, island width jumps
ti-16 | COSMETIC | pixel-tiling-r4:33,34 | open | Question card one size (title bold not semibold, path not mono); empty chat panel while a question waits
ti-17 | COSMETIC | ux-deep-tiling:102,104,106 | open | Ink shadows invisible; unfocused border lighter and uneven; left island fixed 448 px with empty space
ti-18 | COSMETIC | ux-deep-tiling:109,111,115 | verify | Listening chip on the window border; three font families in one frame; key list says Super+Print while the note says Print
