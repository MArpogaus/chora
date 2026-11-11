# Feature showcase, 2026-11-11

Clips: MP4, H.264, 1280x720, 30 fps, 6 to 20 s, from `choractl record` with the stepped clock (clip 18 shows two 1280x720 screens side by side, scaled to 1280x360 and letterboxed). Stills: PNG at 1x (1280x720) from the recorded frames, and at 2x (2560x1440, `-2x`) from sessions with an output of scale 2. Light and dark where a look has both (mc, construct); xp and tiling have one theme. Every finding is in [findings.md](findings.md), with file and time.

Files without a mark come from the build of 67f93a9 (merge of claude/ux-construct, with the chat and HUD themes of 032971e). Files marked *(older build df8dadd)* come from the earlier build and do not show those fixes. No code was changed.

## 1. xp

| File | Shows, and what to check |
|---|---|
| [01-xp-start-menu-open.mp4](01-xp-start-menu-open.mp4) | xp: the Start button opens the menu, the pointer runs over Terminal to Notes, Notes opens; the menu opens again and starts Code; a click on the Notes task button raises it. Check: menu hover highlight, the new task buttons, which button looks pressed. *(older build df8dadd)* |
| [01-xp-start-menu-open-menu-2x.png](01-xp-start-menu-open-menu-2x.png) | Same menu at scale 2 (2560x1440), with Control Panel open. Check: crisp edges and text at 2x, the empty icon slot of Effects. *(older build df8dadd)* |
| [01-xp-start-menu-open-menu.png](01-xp-start-menu-open-menu.png) | xp Start menu open with Terminal and the four demo programs. Check: column alignment, the icon of each row. *(older build df8dadd)* |
| [01-xp-start-menu-open-notes.png](01-xp-start-menu-open-notes.png) | Notes open over the desktop icons, one task button. Check: the window covers My Documents. *(older build df8dadd)* |
| [01-xp-start-menu-open-two-windows.png](01-xp-start-menu-open-two-windows.png) | Notes and wobbly.rs cascaded, two task buttons. Check: active and inactive title bars, the code line cut at the right. *(older build df8dadd)* |
| [02-xp-volume-sound.mp4](02-xp-volume-sound.mp4) | xp: a click on the speaker mutes, a second unmutes; Start > Control Panel > Sound by keys, 75% chosen; the tray shows 75%. Check: the tray words, the lag of the reading, the Sound submenu. *(older build df8dadd)* |
| [02-xp-volume-sound-after.png](02-xp-volume-sound-after.png) | Tray reads 75% after the choice. Check: the tray width grows with the word. *(older build df8dadd)* |
| [02-xp-volume-sound-muted.png](02-xp-volume-sound-muted.png) | Tray reads "Muted" after the click. Check: the speaker glyph and the word. *(older build df8dadd)* |
| [02-xp-volume-sound-sound-menu.png](02-xp-volume-sound-sound-menu.png) | Control Panel > Sound open: Mute, 25%, 50%, 75%, 100%. Check: the missing Effects icon, the highlight. *(older build df8dadd)* |
| [03-xp-control-panel-keys.mp4](03-xp-control-panel-keys.mp4) | xp: the menu worked by keys only: Right, Down, Right into Keyboard Shortcuts, Down and Up over the rows, Escape out. Check: the keyboard highlight, access keys, Escape closing one level at a time. *(older build df8dadd)* |
| [03-xp-control-panel-keys-closed.png](03-xp-control-panel-keys-closed.png) | All menus closed again, the paperclip at the bottom right. Check: nothing left behind. *(older build df8dadd)* |
| [03-xp-control-panel-keys-shortcuts.png](03-xp-control-panel-keys-shortcuts.png) | Keyboard Shortcuts submenu: Super+Alt+1..4 window to tiling/xp/mc/construct and Super+a chat. Check: only the last row has an icon. *(older build df8dadd)* |
| [04-xp-wobbly-drag.mp4](04-xp-wobbly-drag.mp4) | xp: the title bar of wobbly.rs is whipped around twice and let go. Check: the bend while it moves and the shake after release; the paperclip above the window. *(older build df8dadd)* |
| [04-xp-wobbly-drag-bend.png](04-xp-wobbly-drag-bend.png) | Mid drag: the window bent by the wobble. Check: the skew of the frame. *(older build df8dadd)* |
| [04-xp-wobbly-drag-released.png](04-xp-wobbly-drag-released.png) | Just after release, settling. Check: residual shake, "^[" in the terminal (my injected Escape). *(older build df8dadd)* |
| [05-xp-paperclip-balloon-chat.mp4](05-xp-paperclip-balloon-chat.mp4) | xp: the paperclip says a sentence in a balloon, the helper Atlas starts working (row), the chat opens with its transcript, a question card asks to write notes.txt. Check: the balloon tail, the row, the chat over the taskbar, the card over the transcript. |
| [05-xp-paperclip-balloon-chat-balloon-2x.png](05-xp-paperclip-balloon-chat-balloon-2x.png) | The paperclip and balloon at scale 2. Check: crisp SVG figure and text. |
| [05-xp-paperclip-balloon-chat-balloon.png](05-xp-paperclip-balloon-chat-balloon.png) | Balloon "Good morning! Your notes for Friday are ready." Check: the word "ready." alone on its line. |
| [05-xp-paperclip-balloon-chat-chat.png](05-xp-paperclip-balloon-chat-chat.png) | The chat with header, Stop, Close and the transcript. Check: it covers the taskbar; the input line is 6 px left of the text. |
| [05-xp-paperclip-balloon-chat-helper-2x.png](05-xp-paperclip-balloon-chat-helper-2x.png) | The helper row at scale 2. Check: the middle dot and the padding. |
| [05-xp-paperclip-balloon-chat-helper.png](05-xp-paperclip-balloon-chat-helper.png) | Row "Atlas · working · Tidy notes (1/2)" beside the paperclip. Check: the raised middle dot. |
| [05-xp-paperclip-balloon-chat-question-2x.png](05-xp-paperclip-balloon-chat-question-2x.png) | The question card at scale 2. Check: the same overlap at 2x. |
| [05-xp-paperclip-balloon-chat-question.png](05-xp-paperclip-balloon-chat-question.png) | Question card "Atlas wants to write the file notes.txt." with Yes and No. Check: the cut transcript line and the cut hint. |

## 2. tiling

| File | Shows, and what to check |
|---|---|
| [06-tiling-windows-layouts-bar.mp4](06-tiling-windows-layouts-bar.mp4) | tiling: four real windows open and tile, Super+space steps through five layouts, Super+2 shows the empty tag 2 and Super+1 comes back, a click on the volume module mutes. Check: gaps, the black flash of a new tile, the bar modules, the Chat pill over the bar. *(older build df8dadd)* |
| [06-tiling-windows-layouts-bar-fair-horizontal.png](06-tiling-windows-layouts-bar-fair-horizontal.png) | Layout fair: a 2x2 grid. Check: the tag "1" at x=0, "Volume 40%" touching "Connected", the Chat pill over the bar. *(older build df8dadd)* |
| [06-tiling-windows-layouts-bar-sound-unavailable-2x.png](06-tiling-windows-layouts-bar-sound-unavailable-2x.png) | tiling at scale 2 without wpctl: the bar says "Sound unavailable". Check: the word touching "Connected", the Chat pill over the bar. *(older build df8dadd)* |
| [06-tiling-windows-layouts-bar-tag-2.png](06-tiling-windows-layouts-bar-tag-2.png) | The empty tag 2. Check: what an empty tag shows (only the wallpaper colour). *(older build df8dadd)* |
| [06-tiling-windows-layouts-bar-tile-bottom.png](06-tiling-windows-layouts-bar-tile-bottom.png) | A later layout (tile_bottom/tile_top). Check: even gaps. *(older build df8dadd)* |
| [06-tiling-windows-layouts-bar-tile.png](06-tiling-windows-layouts-bar-tile.png) | Layout tile with four windows. Check: the gaps, no focus mark. *(older build df8dadd)* |
| [06-tiling-windows-layouts-bar-volume-muted.png](06-tiling-windows-layouts-bar-volume-muted.png) | The bar reads "Muted" after the click. Check: the module width change. *(older build df8dadd)* |
| [07-tiling-chat-panel.mp4](07-tiling-chat-panel.mp4) | tiling: Super+a opens the chat side panel with the helper list; typing without a choice gets "Pick a helper first ...". Check: the panel over the tiles, the hint text. |
| [07-tiling-chat-panel-open-2x.png](07-tiling-chat-panel-open-2x.png) | The chat panel at scale 2 with Atlas picked. Check: text sharpness and margins at 2x. |
| [07-tiling-chat-panel-open.png](07-tiling-chat-panel-open.png) | Chat with a helper: "1  atlas". Check: lower case name, the tiles cut under the panel. |
| [07-tiling-chat-panel-rows.png](07-tiling-chat-panel-rows.png) | Before opening: the HUD row of Atlas and the Chat pill at the top right. Check: they lie over the window. |
| [07-tiling-chat-panel-sent.png](07-tiling-chat-panel-sent.png) | After Enter without a helper: "Pick a helper first: ...". Check: honest hint, text kept. |
| [07-tiling-chat-panel-typed.png](07-tiling-chat-panel-typed.png) | Text typed into the field. Check: the caret and the field. |

## 3. mc

| File | Shows, and what to check |
|---|---|
| [08-mc-scenes-agenda-chat-light.mp4](08-mc-scenes-agenda-chat-light.mp4) | mc light: agenda, morning, agents, mission control, overview, agenda again, then the chat drawer opens and closes. Check: the write-on effect, cross-fades, empty scenes, the dark drawer on the light page. |
| [08-mc-scenes-agenda-chat-light-agenda-2x.png](08-mc-scenes-agenda-chat-light-agenda-2x.png) | Agenda at scale 2. Check: Pagella and small caps at 2x. |
| [08-mc-scenes-agenda-chat-light-agenda.png](08-mc-scenes-agenda-chat-light-agenda.png) | Agenda with times in the margin. Check: margin alignment, old-style figures. |
| [08-mc-scenes-agenda-chat-light-agents.png](08-mc-scenes-agenda-chat-light-agents.png) | Agents scene: empty although two helpers are known. Check: BUG, no columns. |
| [08-mc-scenes-agenda-chat-light-chat-drawer.png](08-mc-scenes-agenda-chat-light-chat-drawer.png) | The chat drawer under the agenda. Check: near-black on cream, left edge off the grid, no "You" label. |
| [08-mc-scenes-agenda-chat-light-mission.png](08-mc-scenes-agenda-chat-light-mission.png) | Mission Control: headings without data. Check: silent empty panels. |
| [08-mc-scenes-agenda-chat-light-morning-2x.png](08-mc-scenes-agenda-chat-light-morning-2x.png) | The Morning at scale 2. Check: the same layout at 2x. |
| [08-mc-scenes-agenda-chat-light-morning.png](08-mc-scenes-agenda-chat-light-morning.png) | The Morning: lead, Weather, The day. Check: the missing lower row, the column offsets around the rule. |
| [08-mc-scenes-agenda-chat-light-overview.png](08-mc-scenes-agenda-chat-light-overview.png) | Overview: the six scenes as "N | Title". Check: alignment of the numbers. |
| [09-mc-scenes-agenda-chat-dark.mp4](09-mc-scenes-agenda-chat-dark.mp4) | mc dark: the same script as 08 in the dark theme. Check: contrast, the drawer now matching the page, cross-fade overlap of the heads. |
| [09-mc-scenes-agenda-chat-dark-agenda.png](09-mc-scenes-agenda-chat-dark-agenda.png) | Agenda, dark. Check: contrast of the margin times. |
| [09-mc-scenes-agenda-chat-dark-agents.png](09-mc-scenes-agenda-chat-dark-agents.png) | Agents, dark: empty. Check: BUG as in 08. |
| [09-mc-scenes-agenda-chat-dark-chat-drawer-2x.png](09-mc-scenes-agenda-chat-dark-chat-drawer-2x.png) | Chat drawer, dark, at scale 2. Check: same place and margins as at 1x. |
| [09-mc-scenes-agenda-chat-dark-chat-drawer.png](09-mc-scenes-agenda-chat-dark-chat-drawer.png) | Chat drawer, dark. Check: drawer edges against the text block. |
| [09-mc-scenes-agenda-chat-dark-mission.png](09-mc-scenes-agenda-chat-dark-mission.png) | Mission Control, dark. Check: empty panels. |
| [09-mc-scenes-agenda-chat-dark-morning.png](09-mc-scenes-agenda-chat-dark-morning.png) | The Morning, dark. Check: the rule and the columns. |
| [09-mc-scenes-agenda-chat-dark-overview.png](09-mc-scenes-agenda-chat-dark-overview.png) | Overview, dark. Check: list alignment. |

## 4. construct

| File | Shows, and what to check |
|---|---|
| [10-construct-walk-fly-list-light.mp4](10-construct-walk-fly-list-light.mp4) | construct light: walk by keys (w, e, q, s), Return flies into the sunset pane, Alt+Escape flies out, Tab opens the window list, Down twice and Return fly to the chora pane. Check: walk smoothness, the fly-in jump, the label over the pane, the list. |
| [10-construct-walk-fly-list-light-flown-in.png](10-construct-walk-fly-list-light-flown-in.png) | Flown into the sunset pane. Check: 1:1 pixels, edges. |
| [10-construct-walk-fly-list-light-in-pane.png](10-construct-walk-fly-list-light-in-pane.png) | Arrived at the chora pane. Check: the pane at 1:1 with the space around it. |
| [10-construct-walk-fly-list-light-list-choice.png](10-construct-walk-fly-list-light-list-choice.png) | Flying to the chosen pane (chora). Check: the motion, the neighbour pane at the left. |
| [10-construct-walk-fly-list-light-panes-2x.png](10-construct-walk-fly-list-light-panes-2x.png) | The light construct at scale 2. Check: pane edges, floor reflections at 2x. |
| [10-construct-walk-fly-list-light-walked.png](10-construct-walk-fly-list-light-walked.png) | After the walk: three panes ahead with mirror images on the floor. Check: OPEN CUBE now at the bottom right, the "Listening" label beside the ring. |
| [10-construct-walk-fly-list-light-window-list-2x.png](10-construct-walk-fly-list-light-window-list-2x.png) | The window list at scale 2. Check: crisp glass text. |
| [10-construct-walk-fly-list-light-window-list.png](10-construct-walk-fly-list-light-window-list.png) | The window list now lists all five windows, the hidden ones with "not in view". Check: glass contrast over the panes. |
| [11-construct-manager-panes-xp-tiling-light.mp4](11-construct-manager-panes-xp-tiling-light.mp4) | construct light: turn to the xp manager pane and fly in, open its Start menu, fly out, turn to the tiling manager pane, fly in, Super+space changes its layout. Check: input reaches the nested managers, missing bar in tiling, the near pane at the left. |
| [11-construct-manager-panes-xp-tiling-light-in-tiling.png](11-construct-manager-panes-xp-tiling-light-in-tiling.png) | Inside the tiling pane, two tiles. Check: no bar. |
| [11-construct-manager-panes-xp-tiling-light-in-xp-start-menu.png](11-construct-manager-panes-xp-tiling-light-in-xp-start-menu.png) | Inside the xp pane, its Start menu open. Check: default programs only. |
| [11-construct-manager-panes-xp-tiling-light-tiling-pane.png](11-construct-manager-panes-xp-tiling-light-tiling-pane.png) | The tiling manager pane in view (two tiles), before the fly-in. Check: its size and label against the other panes. |
| [11-construct-manager-panes-xp-tiling-light-xp-pane.png](11-construct-manager-panes-xp-tiling-light-xp-pane.png) | The xp manager pane among the panes, with wobbly.rs on its desktop. Check: the "XP" label, overlap with the next pane. |
| [12-construct-glass-chat-question-dark.mp4](12-construct-glass-chat-question-dark.mp4) | construct dark: a short glide, the glass chat opens with Atlas, a question card asks. Check: the light chat on the dark space, the white HUD pills. |
| [12-construct-glass-chat-light-2x.png](12-construct-glass-chat-light-2x.png) | The glass chat in the light theme at scale 2. Check: margins at 2x, OPEN CUBE in the middle at 2x. |
| [12-construct-glass-chat-question-dark-glass-chat-2x.png](12-construct-glass-chat-question-dark-glass-chat-2x.png) | The glass chat, dark, at scale 2. Check: the same at 2x. |
| [12-construct-glass-chat-question-dark-glass-chat.png](12-construct-glass-chat-question-dark-glass-chat.png) | The glass chat in the dark theme, now dark glass. Check: its bottom edge over the OPEN CUBE chip. |
| [12-construct-glass-chat-question-dark-panes-2x.png](12-construct-glass-chat-question-dark-panes-2x.png) | Dark construct at scale 2 with the HUD row. Check: dark pills, OPEN CUBE at the bottom middle. |
| [12-construct-glass-chat-question-dark-question.png](12-construct-glass-chat-question-dark-question.png) | The question card in the dark glass chat. Check: a white card on dark glass, overlap with the transcript. |
| [12-construct-glass-chat-question-dark-walk.png](12-construct-glass-chat-question-dark-walk.png) | The dark construct with the HUD pills, now dark. Check: pills match the construct pills. |
| [12-construct-window-list-dark-2x.png](12-construct-window-list-dark-2x.png) | The window list in the dark theme at scale 2. Check: dark glass, "not in view" column. |

## 5. The xp cube

| File | Shows, and what to check |
|---|---|
| [13-cube-open-cube.mp4](13-cube-open-cube.mp4) | construct light: the pointer goes to OPEN CUBE and clicks; the screen goes black, then the front face of the cube shows xp at 1:1. Check: the black flash, the leftover SUNSET pill, chips over the desktop labels. |
| [13-cube-open-cube-black-flash.png](13-cube-open-cube-black-flash.png) | The black frame after the click (frames 55-70). Check: FLASH. |
| [13-cube-open-cube-chip.png](13-cube-open-cube-chip.png) | The pointer on the OPEN CUBE chip (now bottom right) just before the click. Check: the chip. |
| [13-cube-open-cube-front-face-2x.png](13-cube-open-cube-front-face-2x.png) | The cube demo front face at scale 2, with two windows. Check: crisp xp at 2x. |
| [13-cube-open-cube-front-face.png](13-cube-open-cube-front-face.png) | The front face in use mode with the STEP BACK chip. Check: Chat pill over My Documents, chip over Recycle Bin. |
| [13-cube-open-cube-opening.png](13-cube-open-cube-opening.png) | The first frames of the front face. Check: SUNSET pill over the face. |
| [14-cube-step-back-turn-step-in.mp4](14-cube-step-back-turn-step-in.mp4) | cube (demo scene): the STEP BACK chip, a turn by Ctrl+Alt+Right, a turn by the TURN RIGHT chip, a turn by a drag on the face, a hover, a click on the face steps in. Check: windows floating in front, turn easing, the chips. |
| [14-cube-step-back-turn-step-in-drag-turn.png](14-cube-step-back-turn-step-in-drag-turn.png) | During the drag turn. Check: the face follows the drag. |
| [14-cube-step-back-turn-step-in-hover-edge.png](14-cube-step-back-turn-step-in-hover-edge.png) | Pointer over the face after the turns. Check: the lit edge. |
| [14-cube-step-back-turn-step-in-mid-turn-2x.png](14-cube-step-back-turn-step-in-mid-turn-2x.png) | Mid turn at scale 2: the cube corner with windows on their own planes. Check: depth order, edges. |
| [14-cube-step-back-turn-step-in-stepped-back-2x.png](14-cube-step-back-turn-step-in-stepped-back-2x.png) | Navigate mode at scale 2. Check: chips and the floating windows at 2x. |
| [14-cube-step-back-turn-step-in-stepped-back.png](14-cube-step-back-turn-step-in-stepped-back.png) | Navigate mode: the face whole, windows floating, chips TURN LEFT, TURN RIGHT, STEP IN. Check: the window that overhangs the face edge. |
| [14-cube-step-back-turn-step-in-stepped-in.png](14-cube-step-back-turn-step-in-stepped-in.png) | Back in use mode on face 4 after the click. Check: 1:1 face, STEP BACK chip. |
| [14-cube-step-back-turn-step-in-turn-by-key.png](14-cube-step-back-turn-step-in-turn-by-key.png) | During the turn by key. Check: easing, edge in view. |
| [14-cube-step-back-turn-step-in-turned.png](14-cube-step-back-turn-step-in-turned.png) | After the turn by key: face 2. Check: the chips stay. |
| [14-cube-step-back-turn-step-in-window-off-face-earlier-take.png](14-cube-step-back-turn-step-in-window-off-face-earlier-take.png) | An earlier take: a drag that started on a floating window moved it half off the face. Check: the window hanging over the edge. *(older build df8dadd)* |
| [15-cube-hover-edge-step-in.mp4](15-cube-hover-edge-step-in.mp4) | cube: a turn in navigate mode, the pointer moves onto the face (edge lights), off it and back, a click steps in. Check: the lit edge, the step-in motion. |
| [15-cube-hover-edge-step-in-edge-lit.png](15-cube-hover-edge-step-in-edge-lit.png) | Pointer on the face: the edge lit. Check: how visible the line is. |
| [15-cube-hover-edge-step-in-front-face-0.png](15-cube-hover-edge-step-in-front-face-0.png) | After the turn, navigate mode. Check: chips and the face. |
| [15-cube-hover-edge-step-in-off-face.png](15-cube-hover-edge-step-in-off-face.png) | Pointer off the face: no lit edge. Check: compare with the still before. |
| [15-cube-hover-edge-step-in-stepped-in.png](15-cube-hover-edge-step-in-stepped-in.png) | Stepped in. Check: the face at 1:1. |
| [16-cube-pull-window-out.mp4](16-cube-pull-window-out.mp4) | cube: after a turn back to face 4, the camera stands off and the wobble.rs window is dragged by its title bar off the face, swings as a pane and is dropped in the space. Check: the pull, the pane landing half off screen. |
| [16-cube-pull-window-out-pane.png](16-cube-pull-window-out-pane.png) | The new pane beside the cube. Check: cut by the right edge. |
| [16-cube-pull-window-out-pulling.png](16-cube-pull-window-out-pulling.png) | The window leaving the face as a pane. Check: the frame drops, the pane bends. |
| [16-cube-pull-window-out-stood-off.png](16-cube-pull-window-out-stood-off.png) | The face seen at an angle from the pull pose. Check: perspective of the windows. |
| [17-cube-drop-window-back.mp4](17-cube-drop-window-back.mp4) | cube: the wobble.rs pane is Super-dragged back onto the face and becomes an xp window again. Check: where it lands, the taskbar button coming back. |
| [17-cube-drop-window-back-dragging.png](17-cube-drop-window-back-dragging.png) | The pane moved over the face. Check: depth against the face. |
| [17-cube-drop-window-back-dropped.png](17-cube-drop-window-back-dropped.png) | Back on the face as an xp window with its title bar at the drop point. Check: its lower part under the taskbar. |
| [17-cube-drop-window-back-on-face.png](17-cube-drop-window-back-on-face.png) | The end state. Check: the task button "wobble.rs" is back. |

## 6. Moving windows between window managers

| File | Shows, and what to check |
|---|---|
| [18-handover-keys-drag-menu-two-screens.mp4](18-handover-keys-drag-menu-two-screens.mp4) | Two screens side by side (left xp, right tiling): Super+Alt+1 gives Notes to tiling, Super+Alt+2 back to xp, a title bar drag carries it to tiling, a Super drag brings it back, and Control Panel > Keyboard Shortcuts > window to tiling gives it again. Check: the window and its focus after each path, the HUD switching on the left screen. *(older build df8dadd)* |
| [18-handover-keys-drag-menu-two-screens-drag-to-tiling.png](18-handover-keys-drag-menu-two-screens-drag-to-tiling.png) | During the drag: the xp window crossing into the tiling screen. Check: xp frame on the tiling screen, yellow injection mark (capture). *(older build df8dadd)* |
| [18-handover-keys-drag-menu-two-screens-key-to-tiling.png](18-handover-keys-drag-menu-two-screens-key-to-tiling.png) | Notes in tiling after Super+Alt+1. Check: the neutral HUD pills now on the xp screen. *(older build df8dadd)* |
| [18-handover-keys-drag-menu-two-screens-key-to-xp.png](18-handover-keys-drag-menu-two-screens-key-to-xp.png) | Notes back in xp after Super+Alt+2. Check: the paperclip back. *(older build df8dadd)* |
| [18-handover-keys-drag-menu-two-screens-menu-to-tiling.png](18-handover-keys-drag-menu-two-screens-menu-to-tiling.png) | After the menu entry: Notes in tiling. Check: the window and the menu gone. *(older build df8dadd)* |
| [18-handover-keys-drag-menu-two-screens-menu.png](18-handover-keys-drag-menu-two-screens-menu.png) | The Keyboard Shortcuts menu open on the xp screen. Check: the rows. *(older build df8dadd)* |
| [18-handover-keys-drag-menu-two-screens-super-drag-to-xp.png](18-handover-keys-drag-menu-two-screens-super-drag-to-xp.png) | After the Super drag: Notes back in xp. Check: its place. *(older build df8dadd)* |

## 7. Voice and chat

| File | Shows, and what to check |
|---|---|
| [19-voice-chat-pick-type-answer-stop.mp4](19-voice-chat-pick-type-answer-stop.mp4) | xp: the fake voice hears "Open the chat" and says "Opening the chat.", the chat opens with the helper list, a click into the field, "1" and Enter pick Atlas, a message is typed and sent, a question card comes, Alt+Y is pressed, Stop is clicked. Check: the pick, the sent message, no reaction to Alt+Y, Stop. |
| [19-voice-chat-pick-type-answer-stop-answered-by-key.png](19-voice-chat-pick-type-answer-stop-answered-by-key.png) | After Alt+Y: the card unchanged. Check: no feedback. |
| [19-voice-chat-pick-type-answer-stop-chat-open.png](19-voice-chat-pick-type-answer-stop-chat-open.png) | The chat opened by voice: "Chat with a helper", "1  atlas". Check: no preselection. |
| [19-voice-chat-pick-type-answer-stop-helper-picked.png](19-voice-chat-pick-type-answer-stop-helper-picked.png) | Atlas picked: transcript, Stop and Close. Check: the header. |
| [19-voice-chat-pick-type-answer-stop-question.png](19-voice-chat-pick-type-answer-stop-question.png) | The question card in the chat. Check: overlap and the cut hint. |
| [19-voice-chat-pick-type-answer-stop-said.png](19-voice-chat-pick-type-answer-stop-said.png) | The balloon "Opening the chat." and the Atlas row. Check: balloon and row together. |
| [19-voice-chat-pick-type-answer-stop-stopped.png](19-voice-chat-pick-type-answer-stop-stopped.png) | After the click on Stop. Check: no visible change. |
| [19-voice-chat-pick-type-answer-stop-typed-sent.png](19-voice-chat-pick-type-answer-stop-typed-sent.png) | After sending "Please also add Monday.". Check: no trace of the sent line. |
| [19b-voice-chat-keyboard-stays-in-window.mp4](19b-voice-chat-keyboard-stays-in-window.mp4) | First take of 19 without the click into the field: the chat opened by voice does not get the keyboard, and "1", Enter, the message and Alt+Y land in the Notes terminal. Check: the typed text inside Notes. |
| [19b-voice-chat-keyboard-stays-in-window-typed.png](19b-voice-chat-keyboard-stays-in-window-typed.png) | "Please also add Monday." typed into Notes instead of the chat. Check: the chat field stays empty. |

## 8. Calm and honest states

| File | Shows, and what to check |
|---|---|
| [20-calm-refused-by-click.mp4](20-calm-refused-by-click.mp4) | xp: a click on No in the question card; the agent ends the question and reports Atlas blocked. Check: feedback right after the click, the cut header. |
| [20-calm-refused-by-click-after-done.png](20-calm-refused-by-click-after-done.png) | The card gone, the chat back. Check: no line about the refusal. |
| [20-calm-refused-by-click-blocked.png](20-calm-refused-by-click-blocked.png) | Header "Atlas · blocked · Tidy notes: writing notes.txt wa...". Check: the ellipsis. |
| [20-calm-refused-by-click-refused.png](20-calm-refused-by-click-refused.png) | Right after the click on No. Check: no "Refused." line. |
| [21-calm-sound-unavailable-no-helper.mp4](21-calm-sound-unavailable-no-helper.mp4) | xp without wpctl: the tray says "Sound unavailable", a click on it, Control Panel > Sound says "Sound unavailable" and "wpctl not found", then the chat opens. Check: honest words, gray rows. |
| [21-calm-sound-unavailable-no-helper-no-helper.png](21-calm-sound-unavailable-no-helper-no-helper.png) | The chat with "1  atlas" (the no-helper state did not show, see findings). Check: the list. |
| [21-calm-sound-unavailable-no-helper-sound-menu.png](21-calm-sound-unavailable-no-helper-sound-menu.png) | Control Panel > Sound: "Sound unavailable", "wpctl not found". Check: the reason shown. |
| [21-calm-sound-unavailable-no-helper-tray.png](21-calm-sound-unavailable-no-helper-tray.png) | The tray "Sound unavailable". Check: the word fits the tray. |
| [22-calm-chat-voice-unreachable.mp4](22-calm-chat-voice-unreachable.mp4) | xp with a voice agent that has no chat methods: the chat says "Cannot reach voice control. Trying again every 5...", typed text and Enter. Check: the cut sentence, the calm paperclip beside it. |
| [22-calm-chat-voice-unreachable-open.png](22-calm-chat-voice-unreachable-open.png) | The chat header "Cannot reach voice control. Trying again every 5...". Check: the ellipsis. |
| [22-calm-chat-voice-unreachable-typed.png](22-calm-chat-voice-unreachable-typed.png) | After typing "hello" and Enter. Check: no trace and no "Not sent". |
| [23-calm-mc-empty-agenda-dark-2x.png](23-calm-mc-empty-agenda-dark-2x.png) | The empty agenda, dark, at scale 2. Check: the same. |
| [23-calm-mc-empty-agenda-light-2x.png](23-calm-mc-empty-agenda-light-2x.png) | The same empty agenda at scale 2. Check: the same. |
| [23-calm-mc-empty-agenda-light.png](23-calm-mc-empty-agenda-light.png) | mc light with no agenda entries. Check: an empty page with only the head, no words. |
