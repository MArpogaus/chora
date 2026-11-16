# Timeline of tour-v4, 2026-11-16

One row per act. `mm:ss` is the start of the act in the MP4 of its chapter; `joined` is the same moment in `tour-all.mp4`. Stills are in `stills/` (two files `-L` and `-R` for the two-screen chapter 7). Build: 2ef891d, cargo build, debug, software rendering. Each MP4 starts with a 2.5 s chapter slide. The clock is stepped, so the pace is the one the script gives, not that of the renderer. Waits for animations are cut (`Tour.cut`), the acts are not.

| chapter | mm:ss | joined | what the person does | what should be seen | still |
|---|---|---|---|---|---|
| 1 xp | 00:02 | 00:02 | Erika sits down at the xp desktop | Desktop picture, taskbar with Start, desktop buttons, the paperclip and the volume in the tray | 01-01-desktop |
| 1 xp | 00:03 | 00:03 | Start, then Editor | The Start menu opens; the editor opens on an empty letter.txt in a window | 01-02-start-menu |
| 1 xp | 00:07 | 00:07 | Types a short letter, one typo corrected, saves with Ctrl+O and Return | The three lines of the letter show in the editor window; the file is written | 01-03-letter |
| 1 xp | 00:18 | 00:18 | Drags the letter window aside by its title bar | The window follows the pointer and bends like jelly, then settles | 01-04-wobble |
| 1 xp | 00:22 | 00:22 | Clicks the paperclip in the tray: she wants to know how to make the sound louder | The helper's chat opens at the bottom right with an empty transcript | 01-05-chat-open |
| 1 xp | 00:25 | 00:25 | Clicks the field and types her question, sends it with Return | Her question shows in the transcript | 01-06-question |
| 1 xp | 00:31 | 00:31 | She reads the answer | The helper answers: Start, Control Panel, Sound, 75% | 01-07-answer |
| 1 xp | 00:32 | 00:32 | Closes the chat with Escape | The chat closes; the paperclip stays |  |
| 1 xp | 00:33 | 00:33 | Start, Control Panel, Sound, 75% | The cascade opens step by step; the tray reads 75% afterwards | 01-08-sound-menu |
| 1 xp | 00:40 | 00:40 | The tray shows 75% | Volume 75% in the tray | 01-10-volume-75 |
| 1 xp | 00:41 | 00:41 | Opens the paperclip again and asks the helper to keep the letter safe | The chat opens with the first answer; her second request is sent | 01-11-second-ask |
| 1 xp | 00:50 | 00:50 | The helper asks before it touches a file | A question card with Yes and No | 01-12-question |
| 1 xp | 00:51 | 00:51 | Erika clicks Yes | The card goes; the helper reports it is done | 01-13-done |
| 1 xp | 00:54 | 00:54 | She reads that it is done | The helper says the letter is in My Documents | 01-14-reply |
| 2 tiling | 00:02 | 00:57 | The hacker looks at the empty tiling desktop | Dark desktop and the bar: layout, tags, clock, volume, Keys | 02-01-empty |
| 2 tiling | 00:03 | 00:59 | Super+Return: a shell. Types ls and cat friday.md, one typo | A terminal fills the screen; the listing and the file | 02-02-shell |
| 2 tiling | 00:07 | 01:02 | Starts a small project: mkdir, a Python file in nano, run it | The editor shows three lines, the run prints the result | 02-02b-project |
| 2 tiling | 00:20 | 01:15 | Clicks Apps in the bar, the visible door to programs | The launcher opens over the shell | 02-02c-apps |
| 2 tiling | 00:23 | 01:18 | Escape closes the launcher | The launcher closes |  |
| 2 tiling | 00:24 | 01:19 | Super+Return (code) | A window opens and the windows re-tile (2 windows) | 02-03-window-2 |
| 2 tiling | 00:25 | 01:20 | Super+Return (notes) | A window opens and the windows re-tile (3 windows) | 02-04-window-3 |
| 2 tiling | 00:26 | 01:21 | Super+Return (page) | A window opens and the windows re-tile (4 windows) | 02-05-window-4 |
| 2 tiling | 00:27 | 01:22 | Super+Shift+space (next layout) | The four windows re-tile: tile_left | 02-1-tile_left |
| 2 tiling | 00:28 | 01:23 | Super+Shift+space (next layout) | The four windows re-tile: tile_bottom | 02-1-tile_bottom |
| 2 tiling | 00:29 | 01:25 | Super+Shift+space (next layout) | The four windows re-tile: fair | 02-1-fair |
| 2 tiling | 00:31 | 01:27 | Super+j moves focus down the stack, Super+Ctrl+Return makes it the master | The focus mark moves; the window becomes the master | 02-19-master |
| 2 tiling | 00:34 | 01:29 | Super+l widens the master | The master area grows | 02-20-wider |
| 2 tiling | 00:35 | 01:30 | Super+Shift+2 sends the focused window to tag 2, Super+2 views it, Super+1 comes back | The window leaves, tag 2 shows it alone, tag 1 has the rest | 02-21-tag-2 |
| 2 tiling | 00:38 | 01:33 | Moves onto a window, Super+Ctrl+space floats it | The window floats on top of the tiles | 02-22-floating |
| 2 tiling | 00:40 | 01:35 | Super+drag moves the floating window | The window bends like jelly while it moves | 02-23-wobble |
| 2 tiling | 00:43 | 01:38 | Super+slash opens the key list | A panel lists every key in groups | 02-24-key-list |
| 2 tiling | 00:44 | 01:39 | Escape closes it | The list closes |  |
| 2 tiling | 00:45 | 01:40 | Super+Print: a screenshot of the rice | A notice says the screenshot is saved (or says why not) | 02-25-screenshot |
| 3 mc (light) | 00:02 | 01:44 | The cook has the recipe on the page, hands in the dough | The recipe page: the lead, the list, a tip; the column with keys at the right | 03-01-recipe-light |
| 3 mc (light) | 00:04 | 01:45 | By voice: 'next step' (the voice agent is a stand-in) | The HUD line shows what was heard | 03-02-heard-light |
| 3 mc (light) | 00:05 | 01:47 | The helper turns the page to the water | The page changes to Step 1 | 03-03-boil-light |
| 3 mc (light) | 00:07 | 01:49 | A timer card comes up in the column (a notification of the helper) | A note with the time and the words in the notifications of the column | 03-04-timer-light |
| 3 mc (light) | 00:09 | 01:51 | By voice again: 'how much garlic' | The HUD line shows the question | 03-04b-asked-light |
| 3 mc (light) | 00:11 | 01:52 | The helper answers on the page (and aloud) | The HUD line shows the answer: four cloves, sliced thin | 03-04c-answer-light |
| 3 mc (light) | 00:12 | 01:54 | Mouse path: clicks Next in the footer to the sauce | The page turns to Step 2: the sauce | 03-05-sauce-light |
| 3 mc (light) | 00:15 | 01:57 | Remote: Left twice goes back to the ingredients, Right once forward | The page of the ingredients, then Step 1 | 03-06-back-light |
| 3 mc (light) | 00:19 | 02:01 | Super+n hides the column: the page is larger for the sofa | The column goes; the recipe page widens | 03-07-wide-light |
| 3 mc (light) | 00:21 | 02:03 | Super+n brings the column back | The column shows again |  |
| 3 mc (light) | 00:22 | 02:04 | The timer is up: a critical note | A red note: Spaghetti is ready | 03-08-alarm-light |
| 3b mc (dark) | 00:02 | 02:08 | Evening on the sofa, the remote in hand: today's plan, large print, dark | The agenda in the dark theme; the column with keys | 03-10-evening-dark |
| 3b mc (dark) | 00:04 | 02:10 | Remote: Right turns the page twice | The page of the morning briefing | 03-11-tomorrow-dark |
| 3b mc (dark) | 00:06 | 02:13 | Remote: Home brings the plan back; Right once | The plan, then the next page |  |
| 3b mc (dark) | 00:08 | 02:14 | Mouse path: clicks Programs in the column | The launcher opens over the page | 03-12-launcher-dark |
| 3b mc (dark) | 00:11 | 02:17 | Escape closes the launcher | The launcher closes |  |
| 3b mc (dark) | 00:12 | 02:18 | Mouse path: opens the chat in the column | The chat opens with the helper list | 03-13-chat-dark |
| 3b mc (dark) | 00:14 | 02:21 | Escape closes the chat | The chat closes |  |
| 4 construct | 00:02 | 02:24 | A watcher at a demo stands in front of the first face | The front face fills the screen: xp with sunset and chora windows, the chip STEP BACK | 04-01-use-mode |
| 4 construct | 00:04 | 02:25 | Alt+Escape: steps back to see the whole cube | The camera steps back; chips TURN LEFT, TURN RIGHT, STEP IN | 04-02-stepped-back |
| 4 construct | 00:05 | 02:27 | Ctrl+Alt+Right: turns the cube by key | The cube turns to the second face |  |
| 4 construct | 00:07 | 02:28 | Ctrl+Alt+Right again: the third face | The cube turns to the third face | 04-06-face-3 |
| 4 construct | 00:08 | 02:29 | Clicks the TURN LEFT chip | The cube turns back one face | 04-07-chip |
| 4 construct | 00:10 | 02:32 | Ctrl+Alt+Left: back to the first face | The first face (sunset and chora) is in front | 04-08-face-1 |
| 4 construct | 00:11 | 02:33 | F1: the key overlay | A list of the keys that work now | 04-09-keys |
| 4 construct | 00:13 | 02:34 | Escape closes the overlay | The overlay closes |  |
| 4 construct | 00:13 | 02:35 | Return: steps into the face | The camera goes into the face: the xp desktop at 1:1 with the chip STEP BACK | 04-14-step-in |
| 4 construct | 00:15 | 02:36 | Alt+Escape: steps back out | The whole cube again | 04-15-out |
| 4 construct | 00:16 | 02:37 | Drags the cube itself to turn it by hand | The cube follows the pointer and turns to the third face | 04-05-drag-turn |
| 5 handover | 00:02 | 02:43 | Two screens: xp on the left, tiling on the right (empty) | xp desktop with taskbar; tiling desktop empty | 05-01-two-screens |
| 5 handover | 00:03 | 02:43 | Click Start | The Start menu opens on the xp screen | 05-02-start |
| 5 handover | 00:05 | 02:45 | Click Editor | A text editor opens on the xp screen | 05-03-editor-open |
| 5 handover | 00:08 | 02:48 | DRAG 1: pull the title bar of the editor window from xp to the tiling screen | the editor window leaves xp and tiles in the tiling screen (full height) | 05-04-drag-xp-to-tiling |
| 5 handover | 00:13 | 02:53 | The person types a line in the editor on its new screen | The text appears in the editor on the tiling screen | 05-04b-typed-on-tiling |
| 5 handover | 00:17 | 02:57 | DRAG 2: Super+drag the editor window from tiling to the construct screen | the editor window leaves tiling and becomes a pane in the construct space | 05-05-drag-tiling-to-construct |
| 5 handover | 00:21 | 03:01 | DRAG 3: Super+drag the the editor window pane from construct to the xp screen | the editor window leaves the construct and shows in xp with a title bar and a task button | 05-06-drag-construct-to-xp |
| 5 handover | 00:25 | 03:05 | DRAG 4: pull the title bar of the editor window from xp to the mc screen | the editor window becomes the window of an mc scene | 05-05-drag-xp-to-mc |
| 5 handover | 00:30 | 03:10 | DRAG 5: Super+drag the editor window out of mc to the xp screen | the editor window leaves mc and shows in xp | 05-08-drag-mc-out |
| 5 handover | 00:34 | 03:14 | GIVE BY KEY: Super+Alt+1 (give the focused window to tiling) | the editor window goes to tiling and fills its screen | 05-09-give-key-tiling |
| 5 handover | 00:36 | 03:16 | GIVE BY KEY: Super+Alt+2 (give the focused window to xp) | the editor window goes back to xp | 05-10-give-key-xp |
