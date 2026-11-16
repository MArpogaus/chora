# Timeline of tour-v3, 2026-11-16

One row per act. `mm:ss` is the start of the act in the MP4 of its chapter; `joined` is the same moment in `tour-all.mp4`. Stills are in `stills/` (two files `-L` and `-R` for the two-screen chapter 7). Build: 7d90f049, cargo build, debug, software rendering. Each MP4 starts with a 2.5 s chapter slide. The clock is stepped, so the pace is the one the script gives, not that of the renderer. Nothing is cut.

| chapter | mm:ss | joined | what the person does | what should be seen | still |
|---|---|---|---|---|---|
| 1 xp | 00:04 | 00:04 | The person sits down at the xp desktop and looks around | Desktop picture, taskbar with Start, four desktop buttons, volume, clock | 01-01-desktop |
| 1 xp | 00:08 | 00:08 | Move to Start and click it | The Start menu opens: Terminal, Chat with a helper, four programs | 01-02-start-menu |
| 1 xp | 00:12 | 00:12 | Pick Terminal | A terminal window opens, a task button appears | 01-03-terminal-open |
| 1 xp | 00:19 | 00:19 | Type ls with a slip (lss), correct it, then cat friday.md | ls lists the notes (the slip is deleted before Return); cat prints the Friday list | 01-04-shell |
| 1 xp | 00:29 | 00:29 | Slip: open Start and click Chat with a helper by mistake, wanting the Editor | Start menu opens, then the chat of the helper opens instead of an editor | 01-05-slip-chat |
| 1 xp | 00:37 | 00:37 | Notice it is the chat, press Escape | The chat closes | 01-06-slip-closed |
| 1 xp | 00:41 | 00:41 | Start again and pick the Editor | A text editor opens on friday.md in its own window | 01-07-editor-open |
| 1 xp | 00:50 | 00:50 | Type a line in the editor, then save with Ctrl+O and Return | The new line is in the text; the editor asks for the file name and keeps it | 01-08-editor-typed |
| 1 xp | 01:01 | 01:01 | Drag the title bar of the editor to the right | The window follows and bends like jelly, then settles | 01-09-wobbly-drag |
| 1 xp | 01:09 | 01:09 | Click the terminal button on the taskbar | The terminal comes to the front | 01-10-taskbar-terminal |
| 1 xp | 01:14 | 01:14 | Open the key list: Super+F1 | A help window lists the keys in groups | 01-11-key-list |
| 1 xp | 01:17 | 01:17 | Close the help with Escape | The help closes, the windows stay | 01-12-key-list-closed |
| 1 xp | 01:25 | 01:25 | Press Super+A, the key the list names for the helper | The chat opens with the helper's transcript | 01-13-chat-open |
| 1 xp | 01:29 | 01:29 | Click into the message field and type | The field takes the keyboard; the text shows | 01-15-typed |
| 1 xp | 01:39 | 01:39 | Press Return | The message is sent: the field empties | 01-16-sent |
| 1 xp | 01:44 | 01:44 | The helper asks a question; the person reads it | A question card with Yes and No shows in the chat | 01-17-question |
| 1 xp | 01:48 | 01:48 | Click Yes | The click answers the question | 01-18-answered |
| 1 xp | 01:53 | 01:53 | Click Close | The chat closes; only the paperclip stays | 01-19-chat-closed |
| 2 tiling | 00:04 | 02:02 | The person looks at the empty tiling desktop | Dark ink desktop, the bar with Apps, tags and layout at the left, clock, volume, Connected and Keys | 02-01-empty |
| 2 tiling | 00:08 | 02:06 | Click Keys in the bar, the visible door to the key list | The key list opens | 02-02-keys-door |
| 2 tiling | 00:16 | 02:14 | Escape closes the list | The key list closes | 02-03-keys-closed |
| 2 tiling | 00:20 | 02:18 | Slip: press Super+s, which is not a key here | Nothing happens and nothing is typed into a window | 02-04-slip |
| 2 tiling | 00:23 | 02:21 | Open the key list with Super+slash and read it | A panel lists every key in groups | 02-05-key-list |
| 2 tiling | 00:29 | 02:28 | Escape closes the key list | The key list closes | 02-06-key-list-closed |
| 2 tiling | 00:33 | 02:31 | Press Super+Return (terminal) | A terminal opens and fills the screen | 02-07-terminal |
| 2 tiling | 00:37 | 02:35 | Type ls and cat friday.md, with a slip corrected | The shell lists the notes and prints the Friday list | 02-08-cat |
| 2 tiling | 00:47 | 02:45 | Open the file in a text editor: nano friday.md, add a line, save and leave | The editor shows the file, the new line, then the shell returns | 02-09-editor |
| 2 tiling | 01:03 | 03:01 | Press Super+Return (second window) | A new window opens, 2 windows tile | 02-12-window-2 |
| 2 tiling | 01:07 | 03:05 | Press Super+Return (third window) | A new window opens, 3 windows tile | 02-13-window-3 |
| 2 tiling | 01:12 | 03:10 | Press Super+Return (fourth window) | A new window opens, 4 windows tile | 02-14-window-4 |
| 2 tiling | 01:16 | 03:14 | Press Super+Shift+space (next layout) | The windows re-tile in tile_left | 02-20-layout-tile_left |
| 2 tiling | 01:20 | 03:18 | Press Super+Shift+space (next layout) | The windows re-tile in tile_bottom | 02-21-layout-tile_bottom |
| 2 tiling | 01:23 | 03:21 | Press Super+Shift+space (next layout) | After two more presses the windows are in a 2x2 grid | 02-22-layout-fair |
| 2 tiling | 01:28 | 03:26 | Move the pointer over another window | Focus follows the pointer; the focused window has the gradient border | 02-30-focus-pointer |
| 2 tiling | 01:33 | 03:31 | Press Super+l (wider master) | The master area grows | 02-31-master-wider |
| 2 tiling | 01:36 | 03:34 | Press Super+Shift+c (close the focused window) | The window closes and the others fill the space | 02-32-closed |
| 2 tiling | 01:40 | 03:38 | Press Super+Shift+2 (send the window to tag 2) | The focused window leaves tag 1 for tag 2 | 02-33-to-tag-2 |
| 2 tiling | 01:43 | 03:41 | Press Super+2 (view tag 2) | Tag 2 shows the moved window alone | 02-34-tag-2 |
| 2 tiling | 01:46 | 03:44 | Press Super+1 (back to tag 1) | The other windows are back | 02-35-tag-1 |
| 2 tiling | 01:49 | 03:47 | Click Chat ... at the top right, the visible door | The chat opens as a side panel with the helper list | 02-37-chat-panel |
| 2 tiling | 01:55 | 03:53 | Type 1 and press Return (pick Atlas) | Atlas is picked: header and transcript | 02-38-picked |
| 2 tiling | 01:59 | 03:57 | Type 'Please also add Monday.' and press Return | The message is sent and the field empties | 02-39-sent |
| 2 tiling | 02:08 | 04:06 | Press Escape | The chat panel closes | 02-40-chat-closed |
| 3 mc (light) | 00:04 | 04:13 | The person reads the first scene | Scene 1 AGENDA: times in the margin, the right column with Notifications, Chat, the keys and the doors | 03-01-start-light |
| 3 mc (light) | 00:09 | 04:18 | Click Next in the footer | The scene fades to number 2 | 03-02-next-scene-2-light |
| 3 mc (light) | 00:15 | 04:24 | Click Next in the footer | The scene fades to number 3 | 03-03-next-scene-3-light |
| 3 mc (light) | 00:21 | 04:30 | Click Next in the footer | The scene fades to number 4 | 03-04-next-scene-4-light |
| 3 mc (light) | 00:26 | 04:35 | Click Next in the footer | The scene fades to number 5 | 03-05-next-scene-5-light |
| 3 mc (light) | 00:32 | 04:41 | Slip: press Super+Left twice, one scene too far back | The scenes go back; the person goes forward again with the footer | 03-07-back-light |
| 3 mc (light) | 00:36 | 04:45 | Press Super+o (overview) | The overview lists the scenes | 03-08-overview-light |
| 3 mc (light) | 00:41 | 04:50 | Press Super+Home (agenda) | The agenda again | 03-09-agenda-light |
| 3 mc (light) | 00:45 | 04:54 | Click Programs in the column, the visible door | The launcher opens | 03-10-launcher-light |
| 3 mc (light) | 00:50 | 04:59 | Escape closes the launcher; the person opens the terminal scene with Super+Return | Scene Terminal with a shell | 03-11-terminal-light |
| 3 mc (light) | 00:56 | 05:05 | Type ls and cat friday.md with a slip, then open the file in nano | The shell prints the Friday list; nano shows the file | 03-12-nano-light |
| 3 mc (light) | 01:13 | 05:22 | Press Super+Home | Back to the agenda | 03-13-agenda-back-light |
| 3 mc (light) | 01:16 | 05:25 | Press Super+n (hide the column) | The right column hides | 03-14-right-light |
| 3 mc (light) | 01:21 | 05:30 | Press Super+n again | The column shows again | 03-14b-right-back-light |
| 3 mc (light) | 01:27 | 05:36 | Click the chat door in the column | The chat opens in the right column with the helper list | 03-15-chat-drawer-light |
| 3 mc (light) | 01:33 | 05:42 | Press Escape | The chat closes | 03-17-chat-closed-light |
| 3 mc (dark) | 00:04 | 05:49 | The person reads the first scene | Scene 1 AGENDA: times in the margin, the right column with Notifications, Chat, the keys and the doors | 03-01-start-dark |
| 3 mc (dark) | 00:09 | 05:54 | Click Next in the footer | The scene fades to number 2 | 03-02-next-scene-2-dark |
| 3 mc (dark) | 00:15 | 06:01 | Click Next in the footer | The scene fades to number 3 | 03-03-next-scene-3-dark |
| 3 mc (dark) | 00:21 | 06:06 | Slip: press Super+Left twice, one scene too far back | The scenes go back; the person goes forward again with the footer | 03-07-back-dark |
| 3 mc (dark) | 00:25 | 06:11 | Press Super+Home (agenda) | The agenda again | 03-09-agenda-dark |
| 3 mc (dark) | 00:29 | 06:14 | Click Programs in the column, the visible door | The launcher opens | 03-10-launcher-dark |
| 3 mc (dark) | 00:34 | 06:20 | Escape closes the launcher; the person opens the terminal scene with Super+Return | Scene Terminal with a shell | 03-11-terminal-dark |
| 3 mc (dark) | 00:40 | 06:26 | Type ls and cat friday.md with a slip, then open the file in nano | The shell prints the Friday list; nano shows the file | 03-12-nano-dark |
| 3 mc (dark) | 00:56 | 06:42 | Press Super+Home | Back to the agenda | 03-13-agenda-back-dark |
| 3 mc (dark) | 01:02 | 06:48 | Click the chat door in the column | The chat opens in the right column with the helper list | 03-15-chat-drawer-dark |
| 3 mc (dark) | 01:08 | 06:54 | Press Escape | The chat closes | 03-17-chat-closed-dark |
| 4 construct | 00:05 | 07:02 | The person looks at the empty construct | A start card stands on the floor: Nothing is open yet, Terminal, All programs, A cube of desktops | 04-01-start-card |
| 4 construct | 00:07 | 07:04 | Click Terminal on the start card, the visible door | The card fades and a terminal opens flown in, with the hint Alt Esc Step back | 04-02-terminal-flown-in |
| 4 construct | 00:13 | 07:10 | Type ls, then cat friday.md | Typed keys reach the terminal at once | 04-03-shell |
| 4 construct | 00:22 | 07:19 | Press Alt+Escape (step back) | The camera steps back: the terminal is a pane, hints W A S D Walk, Tab Windows, Super T Terminal, F1 All keys | 04-04-step-back |
| 4 construct | 00:27 | 07:23 | Press Super+t again (second terminal) | A second terminal opens beside the first and flies in | 04-05-second-terminal |
| 4 construct | 00:32 | 07:29 | Open friday.md in the editor, add a line and save | The editor shows the file and the new line | 04-06-picture |
| 4 construct | 00:48 | 07:45 | Press Alt+Escape (step back) | Two panes side by side with their labels | 04-07-two-panes |
| 4 construct | 00:53 | 07:50 | Press Super+t a third time | A third terminal opens | 04-08-third |
| 4 construct | 00:58 | 07:55 | Type a page command | The terminal shows a page | 04-09-page |
| 4 construct | 01:13 | 08:10 | Press Alt+Escape (step back) | Three panes | 04-10-three-panes |
| 4 construct | 01:17 | 08:14 | Hold w for 1.2 s (walk forward) | The camera glides forward by pane steps | 04-11-walk |
| 4 construct | 01:21 | 08:18 | Hold e for 1 s (turn right) | The view turns | 04-12-turn |
| 4 construct | 01:24 | 08:21 | Press Tab (window list) | A glass list shows the three windows with thumbnails | 04-13-window-list |
| 4 construct | 01:27 | 08:24 | Press Down, then Return | The camera flies to the chosen window | 04-14-list-choice |
| 4 construct | 01:33 | 08:30 | Press Alt+Escape (fly out) | The camera flies back out | 04-15-fly-out |
| 4 construct | 01:37 | 08:34 | Press F1 (key overlay) | A searchable list of the keys that work now | 04-16-keys |
| 4 construct | 01:41 | 08:38 | Type 'turn' into the search | The list narrows to the turn keys | 04-17-keys-search |
| 4 construct | 01:44 | 08:41 | Press Escape twice | The first Escape clears the search, the second closes the overlay | 04-18-keys-closed |
| 4 construct | 01:48 | 08:45 | Slip: press z, which does nothing here | A notice says what the keys do here | 04-19-notice |
| 4 construct | 01:51 | 08:48 | The fake voice agent says: Good morning! Your notes for Friday are ready. | The HUD island at the top right shows the sentence | 04-20-hud-sentence |
| 4 construct | 01:56 | 08:53 | Click the HUD island | The chat opens as a glass panel with the field focused | 04-21-chat-open |
| 4 construct | 02:02 | 08:59 | Type 'Please also add Monday.' and press Return | The message is sent | 04-23-sent |
| 4 construct | 02:13 | 09:10 | The helper asks: Atlas wants to write the file notes.txt | A question card with Yes and No | 04-24-question |
| 4 construct | 02:16 | 09:13 | Click Yes | The click answers the question | 04-25-yes |
| 4 construct | 02:20 | 09:17 | The fake agent closes the question | The card goes; the helper goes on | 04-26-card-closed |
| 4 construct | 02:23 | 09:20 | Press Ctrl+. (stop the helper) | The helper is stopped | 04-27-stop |
| 4 construct | 02:27 | 09:24 | Press Escape (close the chat) | The chat closes | 04-28-chat-closed |
| 5 construct with manager panes | 00:05 | 09:33 | Look at the construct with two manager panes | xp is the pane ahead, tiling a pane to the right (black, empty), labels XP and TILING | 05-01-start |
| 5 construct with manager panes | 00:07 | 09:35 | Hold w to walk toward the xp pane | The xp desktop shows large in the middle of the view | 05-02-xp-view |
| 5 construct with manager panes | 00:13 | 09:40 | Move to the Start button in the xp pane and click | The Start menu opens inside the pane | 05-03-xp-start-menu |
| 5 construct with manager panes | 00:18 | 09:46 | Click Terminal in the menu | A terminal window opens inside the xp pane | 05-04-xp-terminal |
| 5 construct with manager panes | 00:25 | 09:53 | Type 'echo hello from xp' and press Return | The text appears in the xp terminal and the shell answers hello from xp | 05-05-xp-typed |
| 5 construct with manager panes | 00:33 | 10:01 | Turn right with e and walk on to the tiling pane | The tiling pane (black) fills the middle of the view | 05-06-tiling-view |
| 5 construct with manager panes | 00:39 | 10:07 | Click into the tiling pane | The pane takes the keyboard (its edge lights up) | 05-07-tiling-click |
| 5 construct with manager panes | 00:44 | 10:12 | Press Super+Return in the tiling pane | tiling opens a terminal that fills its pane | 05-08-tiling-terminal |
| 5 construct with manager panes | 00:50 | 10:18 | Type 'echo hello from tiling' and press Return | The text appears in the tiling terminal and the shell answers | 05-09-tiling-typed |
| 5 construct with manager panes | 00:58 | 10:25 | Step back out with Alt+Escape | Both panes in view again, both with a terminal | 05-10-both |
| 6 xp cube | 00:05 | 10:36 | Look at the cube in use mode | The front face fills the screen: xp with sunset and chora windows, a STEP BACK Alt+Esc chip | 06-01-use-mode |
| 6 xp cube | 00:08 | 10:39 | Press Alt+Escape (step back) | The camera steps back: the whole face shows with chips TURN LEFT, TURN RIGHT, STEP IN | 06-02-stepped-back |
| 6 xp cube | 00:13 | 10:44 | Look at the floating windows | The windows float in front of the face on their own planes | 06-03-floating |
| 6 xp cube | 00:17 | 10:48 | Press Ctrl+Alt+Right (turn by key), look mid-turn | The cube starts to turn to the next face | 06-04-turn-key-mid |
| 6 xp cube | 00:19 | 10:50 | Wait for the turn to end | The next face (desktop 2: wobbly.rs and load) stands in front | 06-05-turned-key |
| 6 xp cube | 00:25 | 10:56 | Drag that starts on a floating window (load), from right to left | The cube turns (or the window moves) | 06-06-drag-on-window-mid |
| 6 xp cube | 00:31 | 11:02 | Look at the face after that drag | Either the face turned or the window moved on the face | 06-07-after-drag-on-window |
| 6 xp cube | 00:35 | 11:06 | Drag that starts on the empty desktop (top right), right to left | The cube follows the drag and turns to the next face | 06-06b-drag-turn-mid |
| 6 xp cube | 00:40 | 11:11 | Wait for the turn to settle | Face 3 (cube.rs and chart) stands in front | 06-07b-turned-drag |
| 6 xp cube | 00:47 | 11:18 | Move the pointer off the face, then onto it | The edge of the face lights when the pointer is over it | 06-08-hover-off |
| 6 xp cube | 00:52 | 11:23 | Move onto the face | The edge of the face is lit | 06-09-hover-edge |
| 6 xp cube | 00:57 | 11:28 | Click the TURN LEFT chip | The cube turns back one face | 06-10-turn-chip |
| 6 xp cube | 01:03 | 11:34 | Press Ctrl+Alt+Left (turn back) | The cube is on the first face again (sunset, chora) | 06-11-turn-back |
| 6 xp cube | 01:08 | 11:39 | Click the face to step in | The camera goes into the face: the xp desktop at 1:1 with the STEP BACK chip | 06-12-step-in |
| 6 xp cube | 01:15 | 11:46 | Press Alt+Escape (step back again) | The camera steps back from the face | 06-13-step-back-again |
| 6 xp cube | 01:20 | 11:51 | Drag the title bar of the sunset window off the face | The window leaves the face and becomes a pane in the space, with the label SUNSET | 06-15-pulling |
| 6 xp cube | 01:26 | 11:56 | Wait for the pane to settle | The pane floats beside the face, the taskbar button of sunset is gone | 06-16-pane |
| 6 xp cube | 01:32 | 12:02 | Super-drag the pane back onto the face | While dragged the pane moves over the face | 06-17-drag-back-mid |
| 6 xp cube | 01:37 | 12:08 | Wait after the drop | The pane is an xp window again on the face, its task button is back | 06-18-dropped |
| 7 window drags | 00:05 | 12:20 | Two screens: xp on the left, tiling on the right (empty) | xp desktop with taskbar; tiling desktop empty | 07-01-two-screens |
| 7 window drags | 00:08 | 12:22 | Click Start | The Start menu opens on the xp screen | 07-02-start |
| 7 window drags | 00:12 | 12:27 | Click Editor | A text editor opens on the xp screen | 07-03-editor-open |
| 7 window drags | 00:19 | 12:33 | DRAG 1: pull the title bar of the editor window from xp to the tiling screen | the editor window leaves xp and tiles in the tiling screen (full height) | 07-04-drag-xp-to-tiling |
| 7 window drags | 00:29 | 12:44 | The person types a line in the editor on its new screen | The text appears in the editor on the tiling screen | 07-04b-typed-on-tiling |
| 7 window drags | 00:38 | 12:52 | DRAG 2: Super+drag the editor window from tiling to the construct screen | the editor window leaves tiling and becomes a pane in the construct space | 07-05-drag-tiling-to-construct |
| 7 window drags | 00:46 | 13:00 | DRAG 3: Super+drag the the editor window pane from construct to the xp screen | the editor window leaves the construct and shows in xp with a title bar and a task button | 07-06-drag-construct-to-xp |
| 7 window drags | 00:54 | 13:08 | DRAG 4: pull the title bar of the editor window from xp to the mc screen | the editor window becomes the window of an mc scene | 07-07-drag-xp-to-mc |
| 7 window drags | 01:04 | 13:19 | DRAG 5: Super+drag the editor window out of mc to the xp screen | the editor window leaves mc and shows in xp | 07-08-drag-mc-out |
| 7 window drags | 01:12 | 13:27 | GIVE BY KEY: Super+Alt+1 (give the focused window to tiling) | the editor window goes to tiling and fills its screen | 07-09-give-key-tiling |
| 7 window drags | 01:17 | 13:32 | GIVE BY KEY: Super+Alt+2 (give the focused window to xp) | the editor window goes back to xp | 07-10-give-key-xp |
