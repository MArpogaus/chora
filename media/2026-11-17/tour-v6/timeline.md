# Timeline of tour-v6, 2026-11-17

One row per act. `mm:ss` is the start of the act in the MP4 of its chapter; `joined` is the same moment in `tour-all.mp4`. Stills are in `stills/` (two files `-L` and `-R` for the two-screen chapter 7). Build: aa8aa8c0, cargo build, debug, software rendering. Each MP4 starts with a 2.5 s chapter slide. The clock is stepped, so the pace is the one the script gives, not that of the renderer. Waits for animations are cut (`Tour.cut`), the acts are not.

| chapter | mm:ss | joined | what the person does | what should be seen | still |
|---|---|---|---|---|---|
| 1 xp | 00:02 | 00:02 | Erika sits down at the xp desktop | Desktop picture, taskbar with Start, desktop buttons, the paperclip and the volume in the tray | 01-01-desktop |
| 1 xp | 00:03 | 00:03 | Start, then Notepad | The Start menu opens; the editor opens on an empty letter.txt in a window | 01-02-start-menu |
| 1 xp | 00:07 | 00:07 | Types a short letter, one typo corrected, saves with Ctrl+O and Return | The three lines of the letter show in the editor window; the file is written | 01-03-letter |
| 1 xp | 00:18 | 00:18 | Drags the letter window aside by its title bar | The window follows the pointer and bends like jelly, then settles | 01-04-wobble |
| 1 xp | 00:21 | 00:21 | Clicks the paperclip in the tray: she wants to know how to make the sound louder | The helper's chat opens at the bottom right with an empty transcript | 01-05-chat-open |
| 1 xp | 00:24 | 00:24 | Clicks the field and types her question, sends it with Return | Her question shows in the transcript | 01-06-question |
| 1 xp | 00:30 | 00:30 | She reads the answer | The helper answers: Start, Control Panel, Sound, 75% | 01-07-answer |
| 1 xp | 00:31 | 00:31 | Closes the chat with its X button | The chat closes; the paperclip stays |  |
| 1 xp | 00:33 | 00:33 | Start, Control Panel, Sound, 75% | The cascade opens step by step; the tray reads 75% afterwards | 01-08-sound-menu |
| 1 xp | 00:42 | 00:42 | The tray shows 75% | Volume 75% in the tray | 01-10-volume-75 |
| 1 xp | 00:43 | 00:43 | Opens the paperclip again and asks the helper to keep the letter safe | The chat opens with the first answer; her second request is sent | 01-11-second-ask |
| 1 xp | 00:52 | 00:52 | The helper asks before it touches a file | A question card with Yes and No | 01-12-question |
| 1 xp | 00:53 | 00:53 | Erika clicks Yes | The card goes; the helper reports it is done | 01-13-done |
| 1 xp | 00:56 | 00:56 | She reads that it is done | The helper says the letter is in My Documents | 01-14-reply |
| 2 tiling | 00:02 | 00:59 | The hacker looks at the empty tiling desktop | Dark desktop and the bar: layout, tags, clock, volume, Keys | 02-01-empty |
| 2 tiling | 00:03 | 01:01 | Super+Return: a shell. Types ls and cat friday.md, one typo | A terminal fills the screen; the listing and the file | 02-02-shell |
| 2 tiling | 00:07 | 01:04 | Starts a small project: mkdir, a Python file in nano, run it | The editor shows three lines, the run prints the result | 02-02b-project |
| 2 tiling | 00:20 | 01:17 | Clicks Apps in the bar, the visible door to programs | The launcher opens over the shell | 02-02c-apps |
| 2 tiling | 00:23 | 01:20 | Escape closes the launcher | The launcher closes |  |
| 2 tiling | 00:23 | 01:21 | Super+Return (code) | A window opens and the windows re-tile (2 windows) | 02-03-window-2 |
| 2 tiling | 00:24 | 01:22 | Super+Return (notes) | A window opens and the windows re-tile (3 windows) | 02-04-window-3 |
| 2 tiling | 00:25 | 01:23 | Super+Return (page) | A window opens and the windows re-tile (4 windows) | 02-05-window-4 |
| 2 tiling | 00:26 | 01:24 | Super+Shift+space (next layout) | The four windows re-tile: tile_left | 02-1-tile_left |
| 2 tiling | 00:28 | 01:25 | Super+Shift+space (next layout) | The four windows re-tile: tile_bottom | 02-1-tile_bottom |
| 2 tiling | 00:29 | 01:26 | Super+Shift+space (next layout) | The four windows re-tile: fair | 02-1-fair |
| 2 tiling | 00:31 | 01:28 | Super+j moves focus down the stack, Super+Ctrl+Return makes it the master | The focus mark moves; the window becomes the master | 02-19-master |
| 2 tiling | 00:34 | 01:31 | Super+l widens the master | The master area grows | 02-20-wider |
| 2 tiling | 00:35 | 01:32 | Super+Shift+2 sends the focused window to tag 2, Super+2 views it, Super+1 comes back | The window leaves, tag 2 shows it alone, tag 1 has the rest | 02-21-tag-2 |
| 2 tiling | 00:37 | 01:35 | Moves onto a window, Super+Ctrl+space floats it | The window floats on top of the tiles | 02-22-floating |
| 2 tiling | 00:40 | 01:37 | Super+drag moves the floating window | The window bends like jelly while it moves | 02-23-wobble |
| 2 tiling | 00:42 | 01:39 | Super+slash opens the key list | A panel lists every key in groups | 02-24-key-list |
| 2 tiling | 00:43 | 01:40 | Escape closes it | The list closes |  |
| 2 tiling | 00:44 | 01:41 | Super+Print: a screenshot of the rice | A notice says the screenshot is saved (or says why not) | 02-25-screenshot |
| 3 mc (light) | 00:02 | 01:45 | The cook has the recipe on the page, hands in the dough | The recipe page: the lead, the list, a tip; the column with keys at the right | 03-01-recipe-light |
| 3 mc (light) | 00:04 | 01:47 | By voice: 'next step' (the voice agent is a stand-in) | The HUD line shows what was heard | 03-02-heard-light |
| 3 mc (light) | 00:05 | 01:48 | The helper turns the page to the water | The page changes to Step 1 | 03-03-boil-light |
| 3 mc (light) | 00:07 | 01:50 | A timer card comes up in the column (a notification of the helper) | A note with the time and the words in the notifications of the column | 03-04-timer-light |
| 3 mc (light) | 00:09 | 01:52 | By voice again: 'how much garlic' | The HUD line shows the question | 03-04b-asked-light |
| 3 mc (light) | 00:11 | 01:53 | The helper answers on the page (and aloud) | The HUD line shows the answer: four cloves, sliced thin | 03-04c-answer-light |
| 3 mc (light) | 00:12 | 01:55 | Mouse path: clicks Next in the footer to the sauce | The page turns to Step 2: the sauce | 03-05-sauce-light |
| 3 mc (light) | 00:15 | 01:58 | Remote: Left twice goes back to the ingredients, Right once forward | The page of the ingredients, then Step 1 | 03-06-back-light |
| 3 mc (light) | 00:18 | 02:01 | Super+n hides the column: the page is larger for the sofa | The column goes; the recipe page widens | 03-07-wide-light |
| 3 mc (light) | 00:21 | 02:03 | Super+n brings the column back | The column shows again |  |
| 3 mc (light) | 00:22 | 02:05 | The timer is up: a critical note | A red note: Spaghetti is ready | 03-08-alarm-light |
| 3b mc (dark) | 00:02 | 02:09 | Evening on the sofa, the remote in hand: today's plan, large print, dark | The agenda in the dark theme; the column with keys | 03-10-evening-dark |
| 3b mc (dark) | 00:04 | 02:11 | Remote: Right turns the page twice | The page of the morning briefing | 03-11-tomorrow-dark |
| 3b mc (dark) | 00:06 | 02:13 | Remote: Home brings the plan back; Right once | The plan, then the next page |  |
| 3b mc (dark) | 00:08 | 02:15 | Mouse path: clicks Programs in the column | The launcher opens over the page | 03-12-launcher-dark |
| 3b mc (dark) | 00:11 | 02:18 | Escape closes the launcher | The launcher closes |  |
| 3b mc (dark) | 00:11 | 02:18 | Mouse path: opens the chat in the column | The chat opens with the helper list | 03-13-chat-dark |
| 3b mc (dark) | 00:14 | 02:21 | Escape closes the chat | The chat closes |  |
| 4 construct | 00:02 | 02:24 | A watcher at a demo stands in front of the first face | The front face fills the screen: xp with sunset and chora windows, the chip STEP BACK | 04-01-use-mode |
| 4 construct | 00:04 | 02:26 | Drags a window by its title bar over the edge of the face | The window leaves the face and floats as a pane in front of the cube |  |
| 4 construct | 00:08 | 02:30 | The window is a pane of the space, in front of the face | A pane with the sunset picture floats in front of the xp face | 04-11-pane-in-space |
| 4 construct | 00:09 | 02:31 | Alt+Escape: step back to see the pane in the space | The cube with the pane beside it | 04-12-pane-stepped-back |
| 4 construct | 00:11 | 02:33 | Drags the pane back over the face | The pane lands on the face and is a window of xp again |  |
| 4 construct | 00:14 | 02:36 | The window is back on its face | The cube with the window on the face, no pane | 04-13-back-on-face |
| 4 construct | 00:15 | 02:37 | Ctrl+Alt+Right: turns the cube by key | The cube turns to the second face |  |
| 4 construct | 00:17 | 02:38 | Ctrl+Alt+Right again: the third face | The cube turns to the third face | 04-06-face-3 |
| 4 construct | 00:18 | 02:40 | Clicks the TURN LEFT chip | The cube turns back one face | 04-07-chip |
| 4 construct | 00:20 | 02:42 | Ctrl+Alt+Left: back to the first face | The first face (sunset and chora) is in front | 04-08-face-1 |
| 4 construct | 00:21 | 02:43 | F1: the key overlay | A list of the keys that work now | 04-09-keys |
| 4 construct | 00:22 | 02:44 | Escape closes the overlay | The overlay closes |  |
| 4 construct | 00:23 | 02:45 | Return: steps into the face | The camera goes into the face: the xp desktop at 1:1 with the chip STEP BACK | 04-14-step-in |
| 4 construct | 00:24 | 02:46 | Alt+Escape: steps back out | The whole cube again | 04-15-out |
| 4 construct | 00:25 | 02:47 | Drags the cube itself to turn it by hand | The cube follows the pointer and turns to the third face | 04-05-drag-turn |
| 5 handover | 00:02 | 02:52 | Two screens: xp on the left, tiling on the right (empty) | xp desktop with taskbar; tiling desktop empty | 05-01-two-screens |
| 5 handover | 00:03 | 02:53 | Click Start | The Start menu opens on the xp screen | 05-02-start |
| 5 handover | 00:05 | 02:54 | Click Notepad | A text editor opens on the xp screen | 05-03-editor-open |
| 5 handover | 00:07 | 02:57 | DRAG 1: pull the title bar of the editor window from xp to the tiling screen | the editor window leaves xp and tiles in the tiling screen (full height) | 05-04-drag-xp-to-tiling |
| 5 handover | 00:12 | 03:02 | The person types a line in the editor on its new screen | The text appears in the editor on the tiling screen | 05-04b-typed-on-tiling |
| 5 handover | 00:16 | 03:06 | DRAG 2: Super+drag the editor window from tiling to the construct screen | the editor window leaves tiling and becomes a pane in the construct space | 05-05-drag-tiling-to-construct |
| 5 handover | 00:20 | 03:10 | DRAG 3: Super+drag the the editor window pane from construct to the xp screen | the editor window leaves the construct and shows in xp with a title bar and a task button | 05-06-drag-construct-to-xp |
| 5 handover | 00:23 | 03:13 | DRAG 4: pull the title bar of the editor window from xp to the mc screen | the editor window becomes the window of an mc scene | 05-05-drag-xp-to-mc |
| 5 handover | 00:28 | 03:18 | DRAG 5: Super+drag the editor window out of mc to the xp screen | the editor window leaves mc and shows in xp | 05-08-drag-mc-out |
| 5 handover | 00:32 | 03:22 | GIVE BY KEY: Super+Alt+1 (give the focused window to tiling) | the editor window goes to tiling and fills its screen | 05-09-give-key-tiling |
| 5 handover | 00:33 | 03:23 | GIVE BY KEY: Super+Alt+2 (give the focused window to xp) | the editor window goes back to xp | 05-10-give-key-xp |

## Tour v5 findings: state in v6 (times in tour-all.mp4, build aa8aa8c0)

Chapter starts: xp 0:00, tiling 0:57, mc light 1:43, mc dark 2:07, construct 2:22, handover 2:50. Total 3:28.
The tiling bar starts through flavors/tiling/waybar/waybar.sh (in dbus-run-session), CHORA_TEST_KEEP_BUS=1 is set.

| id | sev | result | where / still |
|---|---|---|---|
| rv5-1 lock/power by mouse | MAJOR | fixed: power icon in the bar, the key list names "power icon: lock, log out" | 0:59-1:41, stills 02-01, 02-24 |
| j1-5 Super+Print notice | MAJOR | still open: no notice or sign after the key | 1:41, still 02-25 |
| j1-4 "No helperVolume 40%" | MAJOR | fixed: "No helper", "Volume 40%", "Connected" are separate pills | 0:59, still 02-01 |
| j1-2 / N1 Notepad is nano | MAJOR | still open: Notepad is nano in a terminal frame (stub); no GUI editor in the container | 0:03-0:20, still 01-03 |
| rv5-2 mc leg of the handover | MAJOR | fixed in the still: mc draws the window ("7 NOTEPAD", full page) about 3 s after the drop; the first ~3 s of the drag show the empty agenda | 3:13-3:21, still 05-05-drag-xp-to-mc-R |
| rv5-3 helper button in construct HUD | MAJOR | fixed: the HUD has Helper beside Programs, Keys, Lock, Power | 2:26, still 04-11 |
| N2 construct stepped back after pull | MAJOR | fixed: stepped back shows the pane beside the xp face; dragging it back leaves the window on the face | 2:31-2:36, stills 04-12, 04-13 |
| xp chat "Working, step 1 of 1" + Stop after the answer | MINOR | still open: header reads "Working / Done, step 1 of 1" with Stop 30 s after the answer | 0:30, still 01-07; also 0:56, 01-14 |
| mc dark "No helper" with a Helpers row | MINOR | fixed: chat row reads "Working" while Atlas works | 2:20, still 03-13 |
| mc light Super+a row overprints | MINOR | still open: "Chat with a helper" overprints the Super+a description while "listening/speaking" shows | 1:53, still 03-04c; dark 2:09, still 03-10 |
| mc dark launcher cream box, raw entries | MINOR | still open: cream box, UXTerm, XTerm, Foot Server, ImageMagick (color depth=) | 2:15, still 03-12 |
| handover dark bands / tiling black without bar / Start stays open | MINOR | still open: tiling screen is black at the start; the Start menu stays open through the mc legs and the give by key | 2:52, 3:13-3:28, stills 05-01-R, 05-08-L |

## New findings

| Time | Sev | Finding |
|---|---|---|
| 1:57 | MINOR | mc wide page (Super+n): the clock "06:07" runs into the "No helper" pill at the top right (still 03-07) |
| 1:41 | note | j1-5 shows no Print notice and no file sign anywhere; the key does nothing visible (same as before) |
| 3:20 | note | mc leg: the window is a bare nano page with "7 NOTEPAD" as heading, no frame; the audit at 3:28 says one window, owner xp |
