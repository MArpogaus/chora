# Timeline of tour-v8, 2026-11-17

One row per act. `mm:ss` is the start of the act in the MP4 of its chapter; `joined` is the same moment in `tour-all.mp4`. Stills are in `stills/` (two files `-L` and `-R` for the two-screen chapter 7). Build: 733967a, cargo build, debug, software rendering. Each MP4 starts with a 2.5 s chapter slide. The clock is stepped, so the pace is the one the script gives, not that of the renderer. Waits for animations are cut (`Tour.cut`), the acts are not.

| chapter | mm:ss | joined | what the person does | what should be seen | still |
|---|---|---|---|---|---|
| 1 xp | 00:02 | 00:02 | Erika sits down at the xp desktop | Desktop picture, taskbar with Start, desktop buttons, the paperclip and the volume in the tray | 01-01-desktop |
| 1 xp | 00:03 | 00:03 | Start, then Notepad | The Start menu opens; the editor opens on an empty letter.txt in a window | 01-02-start-menu |
| 1 xp | 00:13 | 00:13 | No text editor is installed: Notepad says so | A notice names the missing editor (mousepad); no terminal editor opens |  |
| 1 xp | 00:15 | 00:15 | Start, Notepad again, now with a stand-in GUI editor on PATH | The editor opens in a window | 01-02c-notepad-gui |
| 1 xp | 00:19 | 00:19 | Types a short letter, one typo corrected, saves with Ctrl+D (stand-in) | The three lines of the letter show in the editor window; the file is written | 01-03-letter |
| 1 xp | 00:29 | 00:29 | Drags the letter window aside by its title bar | The window follows the pointer and bends like jelly, then settles | 01-04-wobble |
| 1 xp | 00:33 | 00:33 | Clicks the paperclip on the desktop (node hud-figure): she wants to know how to make the sound louder | The helper's chat opens at the bottom right with an empty transcript | 01-05-chat-open |
| 1 xp | 00:35 | 00:35 | Clicks the field and types her question, sends it with Return | Her question shows in the transcript | 01-06-question |
| 1 xp | 00:41 | 00:41 | She reads the answer | The helper answers: Start, Control Panel, Sound, 75% | 01-07-answer |
| 1 xp | 00:43 | 00:43 | Closes the chat with its X button | The chat closes; the paperclip stays |  |
| 1 xp | 00:44 | 00:44 | Start, Control Panel, Sound, 75% | The cascade opens step by step; the tray reads 75% afterwards | 01-08-sound-menu |
| 1 xp | 00:54 | 00:54 | The tray shows 75% | Volume 75% in the tray | 01-10-volume-75 |
| 1 xp | 00:55 | 00:55 | Opens the paperclip again and asks the helper to keep the letter safe | The chat opens with the first answer; her second request is sent | 01-11-second-ask |
| 1 xp | 01:03 | 01:03 | The helper asks before it touches a file | A question card with Yes and No | 01-12-question |
| 1 xp | 01:04 | 01:04 | Erika clicks Yes | The card goes; the helper reports it is done | 01-13-done |
| 1 xp | 01:07 | 01:07 | She reads that it is done | The helper says the letter is in My Documents | 01-14-reply |
| 1 xp | 01:08 | 01:08 | Closes the chat again | The chat closes |  |
| 1 xp | 01:10 | 01:10 | Clicks the paperclip icon in the tray (xp-helper) | The helper chat opens | 01-15-tray-icon |
| 1 xp | 01:14 | 01:14 | Ctrl+Alt+Right: turns the xp desktop cube by key | The desktop turns like a cube to the next desktop |  |
| 1 xp | 01:16 | 01:16 | Ctrl+Alt+Left: back to the first desktop | The first desktop |  |
| 1 xp | 01:17 | 01:17 | Super held, drags on the desktop to turn the cube | The desktop follows the pointer like a cube | 01-19-super-drag-mid |
| 2 tiling | 00:02 | 01:22 | The hacker looks at the empty tiling desktop | Dark desktop and the bar: layout, tags, clock, volume, Keys | 02-01-empty |
| 2 tiling | 00:03 | 01:23 | Super+Return: a shell. Types ls and cat friday.md, one typo | A terminal fills the screen; the listing and the file | 02-02-shell |
| 2 tiling | 00:07 | 01:26 | Starts a small project: mkdir, a Python file in nano, run it | The editor shows three lines, the run prints the result | 02-02b-project |
| 2 tiling | 00:20 | 01:40 | Clicks Apps in the bar, the visible door to programs | The launcher opens over the shell | 02-02c-apps |
| 2 tiling | 00:23 | 01:42 | Escape closes the launcher | The launcher closes |  |
| 2 tiling | 00:23 | 01:43 | Super+Return (code) | A window opens and the windows re-tile (2 windows) | 02-03-window-2 |
| 2 tiling | 00:24 | 01:44 | Super+Return (notes) | A window opens and the windows re-tile (3 windows) | 02-04-window-3 |
| 2 tiling | 00:25 | 01:45 | Super+Return (page) | A window opens and the windows re-tile (4 windows) | 02-05-window-4 |
| 2 tiling | 00:26 | 01:46 | Super+Shift+space (next layout) | The four windows re-tile: tile_left | 02-1-tile_left |
| 2 tiling | 00:28 | 01:47 | Super+Shift+space (next layout) | The four windows re-tile: tile_bottom | 02-1-tile_bottom |
| 2 tiling | 00:29 | 01:49 | Super+Shift+space (next layout) | The four windows re-tile: fair | 02-1-fair |
| 2 tiling | 00:31 | 01:51 | Super+j moves focus down the stack, Super+Ctrl+Return makes it the master | The focus mark moves; the window becomes the master | 02-19-master |
| 2 tiling | 00:34 | 01:53 | Super+l widens the master | The master area grows | 02-20-wider |
| 2 tiling | 00:35 | 01:54 | Super+Shift+2 sends the focused window to tag 2, Super+2 views it, Super+1 comes back | The window leaves, tag 2 shows it alone, tag 1 has the rest | 02-21-tag-2 |
| 2 tiling | 00:37 | 01:57 | Moves onto a window, Super+Ctrl+space floats it | The window floats on top of the tiles | 02-22-floating |
| 2 tiling | 00:40 | 01:59 | Super+drag moves the floating window | The window bends like jelly while it moves | 02-23-wobble |
| 2 tiling | 00:42 | 02:01 | Super+slash opens the key list | A panel lists every key in groups | 02-24-key-list |
| 2 tiling | 00:43 | 02:03 | Escape closes it | The list closes |  |
| 2 tiling | 00:44 | 02:03 | Super+Print: a screenshot of the rice | A notice says the screenshot is saved (or says why not) | 02-25-screenshot |
| 2 tiling | 00:47 | 02:06 | Clicks the power icon at the right end of the bar | A menu opens: Lock, Log out, Restart, Shut down, Screenshot | 02-26-power-menu |
| 2 tiling | 00:50 | 02:09 | Clicks the row Screenshot | The menu closes; a notice says the screenshot is saved | 02-27-power-screenshot |
| 3 mc (light) | 00:02 | 02:16 | The cook has the recipe on the page, hands in the dough | The recipe page: the lead, the list, a tip; the column with keys at the right | 03-01-recipe-light |
| 3 mc (light) | 00:04 | 02:17 | By voice: 'next step' (the voice agent is a stand-in) | The HUD line shows what was heard | 03-02-heard-light |
| 3 mc (light) | 00:05 | 02:19 | The helper turns the page to the water | The page changes to Step 1 | 03-03-boil-light |
| 3 mc (light) | 00:07 | 02:21 | A timer card comes up in the column (a notification of the helper) | A note with the time and the words in the notifications of the column | 03-04-timer-light |
| 3 mc (light) | 00:09 | 02:23 | By voice again: 'how much garlic' | The HUD line shows the question | 03-04b-asked-light |
| 3 mc (light) | 00:11 | 02:24 | The helper answers on the page (and aloud) | The HUD line shows the answer: four cloves, sliced thin | 03-04c-answer-light |
| 3 mc (light) | 00:12 | 02:26 | Mouse path: clicks Next in the footer to the sauce | The page turns to Step 2: the sauce | 03-05-sauce-light |
| 3 mc (light) | 00:15 | 02:29 | Remote: Left twice goes back to the ingredients, Right once forward | The page of the ingredients, then Step 1 | 03-06-back-light |
| 3 mc (light) | 00:18 | 02:32 | Super+n hides the column: the page is larger for the sofa | The column goes; the recipe page widens | 03-07-wide-light |
| 3 mc (light) | 00:21 | 02:34 | Super+n brings the column back | The column shows again |  |
| 3 mc (light) | 00:22 | 02:36 | The timer is up: a critical note | A red note: Spaghetti is ready | 03-08-alarm-light |
| 3b mc (dark) | 00:02 | 02:40 | Evening on the sofa, the remote in hand: today's plan, large print, dark | The agenda in the dark theme; the column with keys | 03-10-evening-dark |
| 3b mc (dark) | 00:04 | 02:41 | Remote: Right turns the page twice | The page of the morning briefing | 03-11-tomorrow-dark |
| 3b mc (dark) | 00:06 | 02:44 | Remote: Home brings the plan back; Right once | The plan, then the next page |  |
| 3b mc (dark) | 00:08 | 02:46 | Mouse path: clicks Programs in the column | The launcher opens over the page | 03-12-launcher-dark |
| 3b mc (dark) | 00:11 | 02:48 | Escape closes the launcher | The launcher closes |  |
| 3b mc (dark) | 00:11 | 02:49 | Mouse path: opens the chat in the column | The chat opens with the helper list | 03-13-chat-dark |
| 3b mc (dark) | 00:14 | 02:52 | Escape closes the chat | The chat closes |  |
| 4 construct | 00:02 | 02:55 | A watcher at a demo stands in front of the first face | The front face fills the screen: xp with sunset and chora windows, the chip STEP BACK | 04-01-use-mode |
| 4 construct | 00:04 | 02:56 | Drags a window by its title bar over the edge of the face | The window leaves the face and floats as a pane in front of the cube |  |
| 4 construct | 00:08 | 03:00 | The window is a pane of the space, in front of the face | A pane with the sunset picture floats in front of the xp face | 04-11-pane-in-space |
| 4 construct | 00:09 | 03:01 | Alt+Escape: step back to see the pane in the space | The cube with the pane beside it | 04-12-pane-stepped-back |
| 4 construct | 00:11 | 03:03 | Drags the pane back over the face with a plain drag (v6 held Super: a script slip) | The pane lands on the face and is a window of xp again |  |
| 4 construct | 00:14 | 03:06 | The window is back on its face | The cube with the window on the face, no pane | 04-13-back-on-face |
| 4 construct | 00:15 | 03:08 | Ctrl+Alt+Right: turns the cube by key | The cube turns to the second face |  |
| 4 construct | 00:16 | 03:09 | Ctrl+Alt+Right again: the third face | The cube turns to the third face | 04-06-face-3 |
| 4 construct | 00:18 | 03:10 | Clicks the TURN LEFT chip | The cube turns back one face | 04-07-chip |
| 4 construct | 00:20 | 03:12 | Ctrl+Alt+Left: back to the first face | The first face (sunset and chora) is in front | 04-08-face-1 |
| 4 construct | 00:21 | 03:14 | F1: the key overlay | A list of the keys that work now | 04-09-keys |
| 4 construct | 00:22 | 03:15 | Escape closes the overlay | The overlay closes |  |
| 4 construct | 00:23 | 03:16 | Return: steps into the face | The camera goes into the face: the xp desktop at 1:1 with the chip STEP BACK | 04-14-step-in |
| 4 construct | 00:24 | 03:17 | Alt+Escape: steps back out | The whole cube again | 04-15-out |
| 4 construct | 00:25 | 03:18 | Drags the cube itself to turn it by hand | The cube follows the pointer and turns to the third face | 04-05-drag-turn |
| 4 construct | 00:27 | 03:20 | Stepped back, the cube at rest (0 degrees) | The cube from outside, one face square on | 04-20-seam-0 |
| 4 construct | 00:28 | 03:21 | Drags the cube half way and holds: about 45 degrees | The cube stands between two faces, the seam between them in view |  |
| 4 construct | 00:32 | 03:24 | Ctrl+Alt+Right: one face turned (90 degrees) | The cube rests on the next face | 04-22-seam-90 |
| 5 handover | 00:02 | 03:29 | Two screens: xp on the left, tiling on the right (empty) | xp desktop with taskbar; tiling desktop empty | 05-01-two-screens |
| 5 handover | 00:03 | 03:29 | Click Start | The Start menu opens on the xp screen | 05-02-start |
| 5 handover | 00:05 | 03:31 | Click Notepad | A text editor opens on the xp screen | 05-03-editor-open |
| 5 handover | 00:07 | 03:34 | DRAG 1: pull the title bar of the editor window from xp to the tiling screen | the editor window leaves xp and tiles in the tiling screen (full height) | 05-04-drag-xp-to-tiling |
| 5 handover | 00:12 | 03:38 | The person types a line in the editor on its new screen | The text appears in the editor on the tiling screen | 05-04b-typed-on-tiling |
| 5 handover | 00:16 | 03:42 | DRAG 2: Super+drag the editor window from tiling to the construct screen | the editor window leaves tiling and becomes a pane in the construct space | 05-05-drag-tiling-to-construct |
| 5 handover | 00:20 | 03:46 | DRAG 3: Super+drag the the editor window pane from construct to the xp screen | the editor window leaves the construct and shows in xp with a title bar and a task button | 05-06-drag-construct-to-xp |
| 5 handover | 00:23 | 03:50 | DRAG 4: pull the title bar of the editor window from xp to the mc screen | the editor window becomes the window of an mc scene | 05-05-drag-xp-to-mc |
| 5 handover | 00:28 | 03:54 | DRAG 5: Super+drag the editor window out of mc to the xp screen | the editor window leaves mc and shows in xp | 05-08-drag-mc-out |
| 5 handover | 00:32 | 03:58 | GIVE BY KEY: Super+Alt+1 (give the focused window to tiling) | the editor window goes to tiling and fills its screen | 05-09-give-key-tiling |
| 5 handover | 00:33 | 03:59 | GIVE BY KEY: Super+Alt+2 (give the focused window to xp) | the editor window goes back to xp | 05-10-give-key-xp |
