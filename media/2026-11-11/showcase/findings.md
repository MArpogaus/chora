# Findings of the feature showcase, 2026-11-11

Every visual surprise, inaccuracy or unaesthetic detail that I saw in my own frames, by clip. Each line names the file and the time (mm:ss in the clip, or frame numbers at 30 fps) or the still. Lines marked **BUG** are behaviour that contradicts the docs; **NOT SHOWN** marks a scene I could not show; *capture artefact* marks something that comes from my recording setup, not from chora. Nothing is cut or retouched in the clips.

## Setup notes that affect what you see

- All sessions are headless (`tests/headless-session.sh`) with the software renderer; clips come from `choractl record` with the stepped clock and `--pointer arrow` (none for some construct and mc clips). The core runs with `--test-injected-press-is-real` and `test_accept_injected_keys: true`, so every click and key is injected.
- The voice agent is `tests/fake-voice.py` fed from a FIFO; its transcript is a fixed test file (`You: Please tidy the notes for Friday ... Done. Friday has three items now.`), so the transcript does not change when I type, and it already holds a question line and "Done." before any card asks.
- Sound comes from a stand-in `wpctl` in my scratch directory (state file, 40 % at start); the cube clips 14-17 ran without it and so show "Sound unavailable" in the face tray, and the 2x cube stills show "Muted" from an earlier state file.
- Waybar of the tiling flavor ran with a private D-Bus session bus; TeX Gyre Pagella was installed from the Debian package `fonts-texgyre` (without it mc falls back to a sans font).
- Real-time parts (waybar polling, the wpctl thread, fake voice events) run on the wall clock while the record runs at about a third of real time, so their delays look about three times longer in the clips than on a screen.
- xp and tiling have one theme each, so they show in their only theme; mc and construct show light and dark.
- Builds: clips 01-09 and 11, 13-23 were first recorded on df8dadd; the files that index.md lists without a mark were recorded again on 67f93a9 (construct UX fixes, chat and HUD themes). A finding marked (new build) was seen on 67f93a9; unmarked findings of a re-recorded file were checked again there; findings of a file that index.md marks as older build were seen on df8dadd only.

## 01 xp: Start menu, open a program, taskbar

| Where | Finding |
|---|---|
| 01-xp-start-menu-open.mp4 0:00-0:15 | desktop icon label "My Documents" starts at x=2, touching the left edge of the screen (also in every xp still) |
| 01-xp-start-menu-open.mp4 0:05-0:15 / 01-xp-start-menu-open-notes.png | a window that opens is placed at the top left (25,25) over the desktop icons My Documents and My Computer, hiding them |
| 01-xp-start-menu-open.mp4 0:12-0:15 / 01-xp-start-menu-open-two-windows.png | the code content line 106 is cut at the right edge of the terminal ("// cl") |

## 02 xp: volume tray and Sound

| Where | Finding |
|---|---|
| 02-xp-volume-sound.mp4 0:07-0:09 / 02-xp-volume-sound-sound-menu.png, 03-xp-control-panel-keys-shortcuts.png | Control Panel submenu: "Effects" has no icon while every other row has one (gap at x=390-405) |

## 03 xp: Control Panel keys

| Where | Finding |
|---|---|
| 03-xp-control-panel-keys.mp4 0:06 / 03-xp-control-panel-keys-shortcuts.png | Keyboard Shortcuts submenu: only the last row "Super+a: chat" has an icon; the four "Super+Alt+N" rows have none, so the text column starts at the icon column for one row only and looks ragged |

## 04 xp: wobbly window drag

| Where | Finding |
|---|---|
| 04-xp-wobbly-drag.mp4 0:01-0:06 | capture artefact, not chora: "^[" at the bottom left of the terminal, typed by Escape keys I injected while the terminal had the focus |
| 04-xp-wobbly-drag.mp4 0:01-0:04 | the wobble is subtle: at drag speeds under about 1000 px/s the window stays visibly rectangular; only fast whips bend it |
| 04-xp-wobbly-drag.mp4 0:01.7 (frame 52) / 04-xp-wobbly-drag-bend.png | the paperclip draws above the window (frame 52: it covers the lower right corner of the code window) |

## 05 xp: paperclip, balloon, helper, chat, question card

| Where | Finding |
|---|---|
| 05-xp-paperclip-balloon-chat.mp4 0:00-0:04 / 05-xp-paperclip-balloon-chat-balloon.png | the balloon text wraps with one word ("ready.") alone on the second line |
| 05-xp-paperclip-balloon-chat.mp4 0:04-0:09 / 05-xp-paperclip-balloon-chat-helper.png | the middle dot of "Atlas · working · Tidy notes (1/2)" renders as a small raised mark, high above the x-height (also in the chat header) |
| 05-xp-paperclip-balloon-chat.mp4 0:07-0:15 / 05-xp-paperclip-balloon-chat-chat.png | the input line "Type a message, then Enter." starts at x=604 while the header and transcript start at x=610; its second line "A question is waiting." sits 8 px above the taskbar |
| 05-xp-paperclip-balloon-chat.mp4 0:10-0:15 / 05-xp-paperclip-balloon-chat-question.png | the question card is drawn over the transcript and cuts the line "Please tidy the notes for Friday" in half (y=392) |
| 05-xp-paperclip-balloon-chat.mp4 0:10-0:15 / 05-xp-paperclip-balloon-chat-question.png | the hint "Typing does not answer. Press Alt+Y or Alt+N, o..." is cut with an ellipsis; the full sentence never shows in the panel |
| 05-xp-paperclip-balloon-chat.mp4 0:10-0:15 / 05-xp-paperclip-balloon-chat-question.png | the card has about 40 px of empty space below Yes and No, but 10 px above "Question for you": uneven padding |
| 05-xp-paperclip-balloon-chat.mp4 0:10-0:15 | the transcript of the fake helper already shows "Write notes.txt?" and "Done." before the question card asks; this is my test transcript, not a chora fault |
| 05-xp-paperclip-balloon-chat.mp4 0:07-0:15 / 05-xp-paperclip-balloon-chat-chat.png | (new build) the chat now ends above the taskbar (y=690) and leaves the task buttons free |
| 05-xp-paperclip-balloon-chat.mp4 0:01-0:04 / 05-xp-paperclip-balloon-chat-balloon.png | (new build) the balloon text is set in about 10 px type, smaller than every other text of the xp HUD and of the chat, and "ready." still stands alone on the second line |
| 05-xp-paperclip-balloon-chat.mp4 0:04-0:09 / 05-xp-paperclip-balloon-chat-helper.png | (new build) the helper row now sits at the bottom left of the paperclip, 8 px from its edge and 4 px above the taskbar |
| 05-xp-paperclip-balloon-chat.mp4 0:10-0:15 / 05-xp-paperclip-balloon-chat-question.png | capture artefact, not chora: the arrow pointer of my script rests on the card over "(412 bytes)" |

## 06 tiling: windows, layouts, bar

| Where | Finding |
|---|---|
| 06-tiling-windows-layouts-bar.mp4 all / all tiling stills | the tag "1" of the bar sits at x=0, flush against the left edge of the screen, with no padding |
| 06-tiling-windows-layouts-bar.mp4 0:00-0:14 | the HUD pill "Chat ..." (y=18-47) overlaps the bottom of the bar (height 26) and the "Connected" module; it does not move below the bar when the bar reserves its space |
| 06-tiling-windows-layouts-bar.mp4 0:00-0:14 | "Volume 40%" touches the blue "Connected" box: no gap between the two modules |
| 06-tiling-windows-layouts-bar.mp4 0:00-0:14 | the focused window has no border or other mark: with four windows one cannot tell which has the keyboard |
| 06-tiling-windows-layouts-bar.mp4 0:04-0:06 (frames 140-150) | a new window takes its tile and shows as a black rectangle for about 10 frames before foot draws (flash) |
| 06-tiling-windows-layouts-bar.mp4 0:13-0:14 | the volume module changes about 2 s after the click: waybar polls once a second in wall time, and the stepped record runs slower than real time, so the delay in the clip is longer than on a real screen |

## 07 tiling: chat side panel

| Where | Finding |
|---|---|
| 07-tiling-chat-panel.mp4 0:01-0:10 / 07-tiling-chat-panel-open.png | the chat side panel lies over the right column of tiles without a relayout: the "load" chart and the notes are cut off under it |
| 07-tiling-chat-panel.mp4 0:01-0:10 | the helper list says "1  atlas" in lower case, while the row, the header and the transcript say "Atlas" |
| 07-tiling-chat-panel.mp4 0:05-0:10 / 07-tiling-chat-panel-sent.png | Enter with no helper chosen keeps the text and says "Pick a helper first: type its number and press Enter, or click it." (honest), but the chat opened by Super+a does not preselect the only helper |
| 07-tiling-chat-panel.mp4 0:00-0:01 / 07-tiling-chat-panel-rows.png | (new build) the pill "Chat ..." and the Atlas row now sit below the bar; they still lie over the "load" window, with 4 px between pill and row |
| 07-tiling-chat-panel.mp4 all | (new build) the bar of the tiling flavor says "Muted" here: capture artefact, my restarted waybar read another state file of the stand-in wpctl |
| 07-tiling-chat-panel.mp4 0:02-0:08 / 07-tiling-chat-panel-open.png | (new build) the neutral chat is an opaque white panel with a 1 px gray line; the transcript area stays empty white down to the field (about 450 px of blank paper) while no helper is picked |
| 07-tiling-chat-panel.mp4 0:09-0:10 | (new build) after Escape the chat closes and the "Listening" pill shows at the bottom right over the corner of the notes tile |

## 08 mc: scenes, agenda, chat drawer (light)

| Where | Finding |
|---|---|
| 08-mc-scenes-agenda-chat-light.mp4 0:05-0:07 / 08-mc-scenes-agenda-chat-light-agents.png | BUG (old and new build): the scene Agents is empty although the fake voice agent named two helpers (Atlas working, Coder thinking) with voice.state; the HUD rows show both, but `choractl call mc agent.list` answers `{"agents": []}` |
| 08-mc-scenes-agenda-chat-light.mp4 0:07-0:12 / 08-mc-scenes-agenda-chat-light-mission.png | Mission Control shows the headings LOAD, BUSY and "Journal, errors today" with nothing under them and no word that there is no data yet (silent empty state) |
| 08-mc-scenes-agenda-chat-light.mp4 0:02-0:04 / 08-mc-scenes-agenda-chat-light-morning.png | at 1280x720 the morning page shows only the lead, Weather and The day; the panels Mail, Overnight and Reading of the scene file fall below the screen and never show |
| 08-mc-scenes-agenda-chat-light.mp4 0:02-0:04 / 08-mc-scenes-agenda-chat-light-morning.png | "THE DAY" starts 110 px right of the rule that separates it from Weather, while Weather starts at the text block edge: the two columns sit unevenly around the rule |
| 08-mc-scenes-agenda-chat-light.mp4 0:00-0:15 | the HUD rows (Atlas, Coder) and the pill "Chat ..." lie in the right margin at y=56-163, so a page whose text block is wider would run under them; nothing reserves that space |
| 08-mc-scenes-agenda-chat-light.mp4 0:15-0:18 / 08-mc-scenes-agenda-chat-light-chat-drawer.png | the drawer starts at x=54: neither on the margin column (times at x=87) nor on the text block (x=153); its own header is at x=68 and the input text at x=62 |
| 08-mc-scenes-agenda-chat-light.mp4 0:15-0:18 / 08-mc-scenes-agenda-chat-light-chat-drawer.png | the transcript names "Atlas" above each agent line but gives the person's line no "You" label (xp and tiling show "You") |
| 08-mc-scenes-agenda-chat-light.mp4 0:15-0:18 / 08-mc-scenes-agenda-chat-light-chat-drawer.png | (new build) the chat drawer now follows the light theme: cream with a 1 px rule; its sans text (Inter) sits under the Pagella page, so the drawer reads as another document |

## 09 mc: scenes, agenda, chat drawer (dark)

| Where | Finding |
|---|---|
| 09-mc-scenes-agenda-chat-dark.mp4 0:02.5-0:02.8 (frames 76-84) | in the cross-fade of two scenes the old and the new chapter heads lie on top of each other in the same place ("1 / AGENDA" over "2 / THE MORNING"), unreadable for about 8 frames; the same at every scene change in 08 and 09 |

## 10 construct: walk, fly in and out, window list (light)

| Where | Finding |
|---|---|
| 10-construct-walk-fly-list-light.mp4 (not in the clip) | NOT SHOWN: (old and new build) the notice "The keys walk here. F1 shows them all." I typed z (as a key and as text, injected); the node construct-hud-say stayed " " and construct-hud has opacity 0 with voice_hud: host, so a construct notice may never be visible under the host HUD (needs a check with real keys) |
| 10-construct-walk-fly-list-light.mp4 all / all construct stills | (new build) the chip OPEN CUBE moved to the bottom right (x 1139-1263 at 1x), but at scale 2 it sits at the bottom middle (10-construct-walk-fly-list-light-panes-2x.png, x about 1140-1260 of 2560): its place changes with the scale |
| 10-construct-walk-fly-list-light.mp4 all / 10-construct-walk-fly-list-light-window-list.png | (new build) the new label "Listening" left of the ring is a white box whose right edge stops 6 px before the ring; label and ring read as two separate things, and the box is 2 px lower than the ring centre |
| 10-construct-walk-fly-list-light.mp4 0:09.9-0:10.1 (frames 298-302) | the fly-in still starts with a jump: the pane grows by about half between frames 298 and 302, then eases; the SUNSET label now just vanishes at frame 300 (no longer drawn over the face) |
| 10-construct-walk-fly-list-light.mp4 0:14 / 10-construct-walk-fly-list-light-window-list.png | (new build) the window list lists all five windows; the shown ones have an empty place column, the hidden ones say "not in view": the column is right aligned at x=787 while the names start at x=376, leaving a wide empty gap |

## 11 construct: manager panes, xp and tiling in panes

| Where | Finding |
|---|---|
| 11-construct-manager-panes-xp-tiling-light.mp4 setup | `pane.manager` put the xp pane at x=2253 and the tiling pane at x=-2253, out of the view (the docs say "a pane ahead"); I had to turn the camera to find them |
| 11-construct-manager-panes-xp-tiling-light.mp4 (setup before 0:00) | `window.give` on an own window without a pane answers "s6 is no pane (code 1)"; the windows Notes, Chart and wobbly.rs-2 had pane "none" because the view was full, so they could not be given until window.show made panes of them |
| 11-construct-manager-panes-xp-tiling-light.mp4 0:05-0:09 / 11-construct-manager-panes-xp-tiling-light-in-xp-start-menu.png | the xp in the pane shows its default start menu (Terminal only): the manager starts with its own settings, not the ones of the session |
| 11-construct-manager-panes-xp-tiling-light.mp4 0:09-0:11 | the first Alt+Escape after Escape did not fly out of the xp pane at once; the Escape reached the terminal and printed "^[" in it (frame 323); the fly-out came about 1.5 s later |
| 11-construct-manager-panes-xp-tiling-light.mp4 0:12-0:14 (frames 400-430) | at yaw -0.85 the pane "load" stands so near the camera that it fills the left third of the view, dark and blurred |
| 11-construct-manager-panes-xp-tiling-light.mp4 0:15-0:18 / 11-construct-manager-panes-xp-tiling-light-in-tiling.png | the tiling in the pane has no bar: the panel of the tiling flavor (waybar) does not start for a manager pane, so tags, volume and network are missing there |
| 11-construct-manager-panes-xp-tiling-light.mp4 0:15-0:18 | the clip ends inside the tiling pane: the last Alt+Escape arrives after the last frame (my timing) |
| 11-construct-manager-panes-xp-tiling-light.mp4 (setup before 0:00) | (new build) two programs started with choractl run after the camera had flown to a pane of an xp window (wobbly.rs-2) became xp windows, not construct panes; I gave them to tiling through xp |
| 11-construct-manager-panes-xp-tiling-light.mp4 0:03-0:11 / 11-construct-manager-panes-xp-tiling-light-in-xp-start-menu.png | (new build) while flown into the xp pane, the chip OPEN CUBE stays at the bottom right, above the xp tray and clock of the pane, and the "Listening" label and ring sit over the top edge of the pane |

## 12 construct: glass chat, question, dark theme

| Where | Finding |
|---|---|
| 12-construct-glass-chat-question-dark.mp4 all / 12-construct-glass-chat-question-dark-walk.png | (new build) the HUD pills and the chat now follow the dark theme |
| 12-construct-glass-chat-question-dark.mp4 0:09-0:13 / 12-construct-glass-chat-question-dark-question.png | (new build) the question card stays white (#f5f5f5) inside the dark glass chat: a bright block in the dark theme |
| 12-construct-glass-chat-question-dark.mp4 0:09-0:13 / 12-construct-glass-chat-question-dark-question.png | the card still covers the lower transcript, with 40 px empty under Yes and No and an empty band of 60 px above the input |
| 12-construct-glass-chat-question-dark.mp4 0:05-0:13 / 12-construct-glass-chat-question-dark-glass-chat.png | (new build) the chat reaches y=687 and the chip OPEN CUBE (y 680-711, x 1139-1263) peeks out under its bottom right corner: chip and chat overlap |

## 13 cube: OPEN CUBE

| Where | Finding |
|---|---|
| 13-cube-open-cube.mp4 0:01.8-0:02.4 (frames 56-71) / 13-cube-open-cube-black-flash.png | FLASH (old and new build): after the click on OPEN CUBE the whole screen is black for 16 frames (0.53 s) before the front face shows; there is no transition from the space to the cube |
| 13-cube-open-cube.mp4 0:02.4-0:11 / 13-cube-open-cube-front-face.png | the pill "SUNSET" of the construct pane stays on screen over the xp face of the cube (x 597-683, y 494-524), also on the new build |
| 13-cube-open-cube.mp4 0:02.4-0:11 / 13-cube-open-cube-front-face.png | the pill "Chat ..." covers the desktop icon label "My Documents" of the face |
| 13-cube-open-cube.mp4 0:02.4-0:11 / 13-cube-open-cube-front-face.png | the chip "STEP BACK Alt+Esc" covers the label "Recycle Bin"; one letter of the label peeks out right of the chip (x=1262, y=652) |

## 14 cube: step back, turn by key, chip and drag

| Where | Finding |
|---|---|
| 14-cube-step-back-turn-step-in-window-off-face-earlier-take.png (take 1, frame 340) | in navigate mode a drag that starts on a floating window does not turn the cube: it moves that window on the face, and the window ends half off the face (left and top edges hang out over the sky); the docs say a drag on a face turns the cube |

## 15 cube: hover edge, step in

| Where | Finding |
|---|---|
| 15-cube-hover-edge-step-in.mp4 0:03-0:07 / 15-cube-hover-edge-step-in-edge-lit.png | the lit edge of the face under the pointer is a 1 px darker blue line on the blue sky of the face; it is barely visible at 1x |

## 16 cube: pull a window out

| Where | Finding |
|---|---|
| 16-cube-pull-window-out.mp4 0:00-0:01 | a cube.turn in use mode steps back and turns, but the chip STEP BACK Alt+Esc stays on screen through the turn, although stepping back is already happening |
| 16-cube-pull-window-out.mp4 0:06-0:09 / 16-cube-pull-window-out-pane.png | (new build) the pulled pane still lands with its right part cut by the right edge of the screen |

## 17 cube: drop it back

| Where | Finding |
|---|---|
| 17-cube-drop-window-back.mp4 0:03-0:08 / 17-cube-drop-window-back-dropped.png | the window dropped back onto the face lands with its title bar at the drop point, so its lower part lies under the taskbar of the face and is cut off |
| 17-cube-drop-window-back.mp4 0:07-0:08 / 17-cube-drop-window-back-on-face.png | `view.step_in` while the camera stands off at the pull pose in use mode does nothing: the camera stays off, the still shows the face at an angle |

## 18 moving windows between window managers, two screens

| Where | Finding |
|---|---|
| 18-handover-keys-drag-menu-two-screens.mp4 all | capture artefact, not chora: the right half (HEADLESS-2) is captured with `choractl capture` once per recorded frame, so it shows the yellow mark of injected input instead of the arrow, and it may lag the left half by one frame |
| 18-handover-keys-drag-menu-two-screens.mp4 0:07-0:09 / 18-handover-keys-drag-menu-two-screens-drag-to-tiling.png | capture artefact, not chora: "^[^[" after "effects" in Notes, typed by Escape keys I injected before the take |
| 18-handover-keys-drag-menu-two-screens.mp4 0:01-0:04 / 18-handover-keys-drag-menu-two-screens-key-to-tiling.png | when Notes moves to tiling on the right screen, the HUD on the LEFT screen switches from the paperclip to the neutral pills "Chat ..." (top right) and "Listening" (bottom right, over the clock of the xp taskbar); the HUD follows the focused window manager but stays on the other screen |
| 18-handover-keys-drag-menu-two-screens.mp4 0:01-0:04 | the "Listening" pill of the neutral HUD lies over the tray and the clock of the xp taskbar (bottom right of HEADLESS-1) |
| 18-handover-keys-drag-menu-two-screens.mp4 0:09-0:12 | the window given to tiling by a drop fills the whole tiling screen (one window, layout tile): there is no visual hint where it came from, and the jump from a small xp window to full screen is abrupt |

## 19 voice and chat

| Where | Finding |
|---|---|
| 19b-voice-chat-keyboard-stays-in-window.mp4 0:03-0:16 / 19b-voice-chat-keyboard-stays-in-window-typed.png | the chat opened by the voice path (`assistant chat.show {}`, as the voice agent calls it) does not take the keyboard: "1", Enter, "Please also add Monday." and Alt+y all went into the focused Notes terminal (they show there), and the chat field stayed empty |
| 19b-voice-chat-keyboard-stays-in-window.mp4 0:14-0:16 | a click on Close closes the chat but the question card stays alone at the right edge, without the chat around it |
| 19-voice-chat-pick-type-answer-stop.mp4 0:11-0:12 / 19-voice-chat-pick-type-answer-stop-answered-by-key.png | the injected Alt+Y does nothing and says nothing: the card stays, no answer reaches the fake voice agent (claims.answers stays empty) and no line tells why (the words "Please use your own mouse or keyboard." exist but do not show); in a real session a person's key would answer, so this needs a check with real keys |
| 19-voice-chat-pick-type-answer-stop.mp4 0:08 / 19-voice-chat-pick-type-answer-stop-typed-sent.png | after Enter the sent line "Please also add Monday." does not show in the transcript; the fake agent does not echo it, but the chat itself gives no sign (no "You" line, no "sending") that the message left |
| 19-voice-chat-pick-type-answer-stop.mp4 0:13-0:16 / 19-voice-chat-pick-type-answer-stop-stopped.png | Stop sends agent.cancel (the fake agent got it) but the chat shows no change: the header still says "question for you", no "Stopped" line; with a real agent the state event would follow, so this is partly the fake |
| 19-voice-chat-pick-type-answer-stop.mp4 0:10-0:16 | as in clip 05, the card covers the transcript and the hint is cut with "o..."  |

## 20 calm states: refusal

| Where | Finding |
|---|---|
| 20-calm-refused-by-click.mp4 0:01-0:03 / 20-calm-refused-by-click-refused.png | a click on No reaches the agent at once (confirm.answer yes=false) but the card shows no feedback: no "Refused." line, no pressed state; it stays unchanged for about 2 s until the agent ends the question |
| 20-calm-refused-by-click.mp4 0:03-0:06 / 20-calm-refused-by-click-blocked.png | the header "Atlas · blocked · Tidy notes: writing notes.txt wa..." is cut with an ellipsis; the reason never shows in full anywhere in the chat |
| 20-calm-refused-by-click.mp4 0:03-0:06 | after the question ends, the chat shows no line that the write was refused: only the header state word changes to "blocked" |
| 20-calm-refused-by-click.mp4 0:00-0:06 | (new build) re-checked: no feedback right after the click on No, and the header ellipsis stays |

## 21 calm states: Sound unavailable

| Where | Finding |
|---|---|
| 21-calm-sound-unavailable-no-helper.mp4 0:01-0:03 / 21-calm-sound-unavailable-no-helper-tray.png | honest: the tray says "Sound unavailable" and a click on it changes nothing and says nothing more (no tooltip, no balloon with the reason; the reason "wpctl not found" is only in Control Panel > Sound) |
| 21-calm-sound-unavailable-no-helper.mp4 0:07-0:09 / 21-calm-sound-unavailable-no-helper-sound-menu.png | honest: Control Panel > Sound shows "Sound unavailable" and "wpctl not found" as gray rows |
| 21-calm-sound-unavailable-no-helper.mp4 0:09 | three Escape keys close only the innermost menus; the start menu stays open until the chat opens over it |
| 21-calm-sound-unavailable-no-helper.mp4 0:10-0:12 / 21-calm-sound-unavailable-no-helper-no-helper.png | NOT SHOWN: the empty state "No helper is set up..."; the fake voice agent with the transcript env var still lists atlas, so the list shows "1  atlas" (my setup) |
| 21-calm-sound-unavailable-no-helper.mp4 0:10-0:12 / 21-calm-sound-unavailable-no-helper-no-helper.png | (new build) the chat list "1  atlas" also reaches y=720 over the taskbar, as in 22 |

## 22 calm states: voice unreachable

| Where | Finding |
|---|---|
| 22-calm-chat-voice-unreachable.mp4 0:01-0:06 / 22-calm-chat-voice-unreachable-open.png | with a voice agent that exports no chat methods, the chat says "Cannot reach voice control. Trying again every 5..." in its header: the sentence is cut with an ellipsis, so "5 s." never shows |
| 22-calm-chat-voice-unreachable.mp4 0:01-0:06 | the paperclip keeps its calm listening look while the chat says that voice control cannot be reached: the two disagree |
| 22-calm-chat-voice-unreachable.mp4 0:04-0:06 / 22-calm-chat-voice-unreachable-typed.png | the typed "hello" and Enter leave no trace: the field shows the placeholder again and no "Not sent" line appears |
| 22-calm-chat-voice-unreachable.mp4 0:01-0:06 / 22-calm-chat-voice-unreachable-open.png | (new build) without a reachable helper the xp chat still reaches down to y=720 and covers the task buttons, while the chat with a helper (clips 05, 19) now ends above the taskbar at y=690: the panel has two heights |

## 23 calm states: empty agenda

| Where | Finding |
|---|---|
| 23-calm-mc-empty-agenda-*.png | the agenda without entries shows only the chapter head "1 / AGENDA" on an empty page: no word such as "Nothing today" says that the empty page is the true state and not a failure to load |
