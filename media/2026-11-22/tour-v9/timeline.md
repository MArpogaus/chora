# Timeline of tour v9, record-v9 r1 (candidate 6b01c976, baseline replay 733967aa)

Verdict is tests/tour-diff.py against demo/tour/intended.toml as of this commit; `check cand` is the act harness on the candidate (base: on the 733967aa replay, `-` = act not run there). Captures are `<act>.<capture>` (not committed; stills here).

| act | chapter | what | verdict | diff kinds | flipped checks | check cand | check base | capture | still |
|---|---|---|---|---|---|---|---|---|---|
| c1 | 4 construct | A watcher at a demo stands in front of the first face | regression | missing,visible,rect,meta,added,pixel | I6 | RED: I6: on VIRTUAL-1 '2' (xp-pager-1-VIRTUAL-1) has th | - | c1.Q | 04-01-use-mode |
| c5 | 4 construct | Drags a window by its title bar over the edge of the face | regression | missing,visible,rect,meta,added | I6 | RED: I6: on VIRTUAL-1 '2' (xp-pager-1-VIRTUAL-1) has th | - | c5.H50 | None |
| c5b | 4 construct | The window is a pane of the space, in front of the face | regression | missing,visible,rect,meta,added,pixel | I6 | RED: I6: on VIRTUAL-1 '2' (xp-pager-1-VIRTUAL-1) has th | - | c5b.Q | 04-11-pane-in-space |
| c2 | 4 construct | Alt+Escape: step back to see the pane in the space | regression | rect,missing,visible,meta,added,pixel | I2,I6 | RED: I2: gaps: HEADLESS-1: construct-mode-step-in and c | - | c2.Q | 04-12-pane-stepped-back |
| c5c | 4 construct | Drags the pane back over the face with a plain drag (v6 held | regression | rect,missing,visible,meta,added | I2,I6 | RED: I2: gaps: HEADLESS-1: construct-mode-step-in and c | - | c5c.H50 | None |
| c5d | 4 construct | The window is back on its face | regression | rect,missing,visible,meta,added,pixel | I2,I6 | RED: I2: gaps: HEADLESS-1: construct-mode-step-in and c | - | c5d.Q | 04-13-back-on-face |
| c3 | 4 construct | Ctrl+Alt+Right: turns the cube by key | regression | rect,missing,visible,meta,added | I2,I6 | RED: I2: gaps: HEADLESS-1: construct-mode-step-in and c | - | c3.H50 | None |
| c3b | 4 construct | Ctrl+Alt+Right again: the third face | regression | rect,missing,visible,meta,added,pixel | I2,I6 | RED: I2: gaps: HEADLESS-1: construct-mode-step-in and c | - | c3b.Q | 04-06-face-3 |
| c3c | 4 construct | Clicks the TURN LEFT chip | regression | rect,missing,visible,meta,added,pixel | I2,I6 | RED: I2: gaps: HEADLESS-1: construct-mode-step-in and c | - | c3c.Q | 04-07-chip |
| c3d | 4 construct | Ctrl+Alt+Left: back to the first face | regression | rect,missing,visible,meta,added,pixel | I2,I6 | RED: I2: gaps: HEADLESS-1: construct-mode-step-in and c | - | c3d.Q | 04-08-face-1 |
| c4 | 4 construct | Super+F1: the key overlay | regression | missing,rect,visible,meta,added,pixel | I2,I6 | RED: I2: gaps: HEADLESS-1: construct-mode-step-in and c | - | c4.Q | 04-09-keys |
| c4b | 4 construct | Escape closes the overlay | regression | rect,missing,visible,meta,added,pixel | I2,I6 | RED: I2: gaps: HEADLESS-1: construct-mode-step-in and c | - | c4b.Q | None |
| c5e | 4 construct | Return: steps into the face | regression | rect,visible,missing,meta,added,pixel | I6 | RED: I6: on VIRTUAL-1 '2' (xp-pager-1-VIRTUAL-1) has th | - | c5e.Q | 04-14-step-in |
| c2b | 4 construct | Alt+Escape: steps back out | regression | rect,missing,visible,meta,added,pixel | I2,I6 | RED: I2: gaps: HEADLESS-1: construct-mode-step-in and c | - | c2b.Q | 04-15-out |
| c3e | 4 construct | Drags the cube itself to turn it by hand | regression |  | I2,I6 | RED: I2: gaps: HEADLESS-1: construct-mode-step-in and c | - | c3e.H50 | 04-05-drag-turn |
| c3f | 4 construct | Stepped back, the cube at rest (0 degrees) | regression |  | I2,I6 | RED: I2: gaps: HEADLESS-1: construct-mode-step-in and c | - | c3f.Q | 04-20-seam-0 |
| c3g | 4 construct | Drags the cube half way and holds: about 45 degrees | regression |  | I2,I6 | RED: I2: gaps: HEADLESS-1: construct-mode-step-in and c | - | c3g.H50 | None |
| c3h | 4 construct | Ctrl+Alt+Right: one face turned (90 degrees) | regression |  | I2,I6 | RED: I2: gaps: HEADLESS-1: construct-mode-step-in and c | - | c3h.Q | 04-22-seam-90 |
| h1 | 5 handover | Two screens: xp on the left, tiling on the right (empty) | regression | missing,rect,meta,added | I6 | RED: I6: on HEADLESS-2 '2' (xp-pager-1-HEADLESS-2) has  | - | h1.Q | 05-01-two-screens |
| h1b | 5 handover | Click Start | regression | missing,rect,meta,added | I6 | RED: I6: on HEADLESS-2 'User' (xp-menu-frame) has the c | - | h1b.Q | 05-02-start |
| h1c | 5 handover | Click Notepad | regression | missing,rect,meta,added | I6 | RED: I6: on HEADLESS-2 '2' (xp-pager-1-HEADLESS-2) has  | - | h1c.Q | 05-03-editor-open |
| h1d | 5 handover | The nurse types the first line in the editor | regression | missing,rect,meta,added | I6 | RED: I6: on HEADLESS-2 '2' (xp-pager-1-HEADLESS-2) has  | - | h1d.Q | 05-03b-typed-on-xp |
| h2 | 5 handover | Title-bar right click, Send To, Tiling desktop | none | baseline missing |  | RED: I6: on HEADLESS-2 '2' (xp-pager-1-HEADLESS-2) has  | - | h2.Q | 05-04-send-to-tiling |
| h3 | 5 handover | The nurse types a line in the editor on its new screen | none | baseline missing |  | RED: I6: on HEADLESS-2 '2' (xp-pager-1-HEADLESS-2) has  | - | h3.Q | 05-05-typed-on-tiling |
| h3b | 5 handover | Super+Alt+3 gives the window to mc | none | baseline missing |  | RED: I6: on HEADLESS-2 '2' (xp-pager-1-HEADLESS-2) has  | - | h3b.Q | 05-06-give-key-mc |
| h4 | 5 handover | Super+Alt+4 gives the window to the construct | none | baseline missing |  | RED: I6: on HEADLESS-2 '2' (xp-pager-1-HEADLESS-2) has  | - | h4.Q | 05-07-give-key-construct |
| h5 | 5 handover | Super+drag the pane of the editor from the construct onto th | none | baseline missing |  | RED: I6: on HEADLESS-2 '2' (xp-pager-1-HEADLESS-2) has  | - | h5.H50 | 05-08-drag-construct-to-xp |
| h6 | 5 handover | xp has the window with all the lines | none | baseline missing |  | RED: I6: on HEADLESS-2 '2' (xp-pager-1-HEADLESS-2) has  | - | h6.Q | 05-09-all-lines |
| h7 | 5 handover | Super+Shift+Escape locks every screen | none | baseline missing |  | RED: I6: on HEADLESS-2 '2' (xp-pager-1-HEADLESS-2) has  | - | h7.Q | 05-10-lock |
| m7 | 3b mc | Evening on the sofa, the remote in hand: today's plan, large | regression | rect,missing,pixel |  | green | RED | m7.Q | 03-10-evening-dark |
| m7b | 3b mc | Remote: Right turns the page twice | regression | missing,rect | I10 | RED: I10: on HEADLESS-1 mc-s-agents-chapter-title says  | RED | m7b.Q | 03-11-tomorrow-dark |
| m7c | 3b mc | Remote: Home brings the plan back | regression | rect,missing,visible,pixel |  | green | RED | m7c.Q | None |
| m8 | 3b mc | Mouse path: clicks Programs in the column | regression | rect,missing,visible,pixel |  | green | RED | m8.Q | 03-12-launcher-dark |
| m8b | 3b mc | Escape closes the launcher | regression | rect,missing,visible,pixel |  | green | RED | m8b.Q | None |
| m9 | 3b mc | Mouse path: opens the chat in the column | regression | rect,missing,visible,added,pixel | I6 | RED: I6: on HEADLESS-1 'Programs' (mc-r0-right-doors-pr | RED | m9.Q | 03-13-chat-dark |
| m9b | 3b mc | Escape closes the chat | regression | rect,missing,visible,added,pixel |  | green | RED | m9b.Q | None |
| m10 | 3b mc | Volume up on the remote | regression | missing,rect,visible,added,pixel |  | green | RED | m10.Q+0.5 | 03-14-volume-dark |
| m1 | 3 mc | The cook has the recipe on the page, hands in the dough | regression | rect,missing |  | green | RED | m1.Q | 03-01-recipe-light |
| m1b | 3 mc | By voice: 'next step' (the voice agent is a stand-in) | regression | rect,missing |  | green | RED | m1b.Q | 03-02-heard-light |
| m1c | 3 mc | The helper turns the page to the water | regression | missing,rect |  | green | RED | m1c.Q | 03-03-boil-light |
| m3 | 3 mc | A timer card comes up in the column (a notification of the h | regression | rect,missing,visible,added,pixel |  | green | RED | m3.Q | 03-04-timer-light |
| m2 | 3 mc | By voice again: 'how much garlic' | regression | rect,missing,visible,added,pixel |  | green | RED | m2.Q | 03-04b-asked-light |
| m2b | 3 mc | The helper answers on the page (and aloud) | regression | rect,missing,visible,added,pixel |  | green | RED | m2b.Q+1 | 03-04c-answer-light |
| m4 | 3 mc | Mouse path: clicks Next in the footer to the sauce | regression | rect,missing,visible |  | green | RED | m4.Q | 03-05-sauce-light |
| m4b | 3 mc | Remote: Left twice goes back to the ingredients, Right once  | regression | rect,missing,visible |  | green | RED | m4b.Q | 03-06-back-light |
| m4c | 3 mc | Remote: Home goes to the first page | regression | rect,missing,visible,pixel |  | green | RED | m4c.Q | None |
| m6 | 3 mc | Super+F1 opens the key list on a page | regression | missing,rect,visible,added,pixel |  | green | RED | m6.Q | None |
| m6b | 3 mc | Escape shuts the key list | regression | rect,missing,visible,added,pixel |  | green | RED | m6b.Q | None |
| m5 | 3 mc | Super+n hides the column: the page is larger for the sofa | regression | missing,rect,visible,pixel |  | green | RED | m5.Q | 03-07-wide-light |
| m5b | 3 mc | Super+n brings the column back | regression | rect,missing,visible,pixel |  | green | RED | m5b.Q | None |
| m3b | 3 mc | The timer is up: a critical note | regression | rect,missing,visible,added,pixel |  | green | RED | m3b.Q | 03-08-alarm-light |
| t1 | 2 tiling | The hacker looks at the empty tiling desktop | regression | pixel |  | green | RED | t1.Q | 02-01-empty |
| t1b | 2 tiling | Super+Return: a shell | regression | rect,pixel |  | green | RED | t1b.Q | 02-02-shell |
| t1c | 2 tiling | Types ls and cat friday.md, one typo | regression | rect,pixel |  | green | RED | t1c.Q | 02-02a-ls |
| t1d | 2 tiling | Starts a small project: mkdir, a Python file in nano, run it | regression | rect,pixel |  | green | RED | t1d.Q | 02-02b-project |
| t2 | 2 tiling | Super+d and two more Super chords while a window of a progra | regression | rect,pixel |  | green | RED | t2.Q | 02-02d-stray |
| t3 | 2 tiling | Super+Return (code) | regression | rect,pixel |  | green | RED | t3.Q | 02-03-window-2 |
| t3b | 2 tiling | Super+Return (notes) | regression | rect,pixel |  | green | RED | t3b.Q | 02-04-window-3 |
| t3c | 2 tiling | Super+Return (page) | regression | rect,pixel |  | green | RED | t3c.Q | 02-05-window-4 |
| t3d | 2 tiling | Super+j moves focus down the stack, Super+Ctrl+Return makes  | regression | rect,pixel |  | green | RED | t3d.Q | 02-19-master |
| t3e | 2 tiling | Super+l widens the master | regression | rect,pixel |  | green | RED | t3e.Q | 02-20-wider |
| t3f | 2 tiling | Super+Shift+space (next layout) | regression | rect,pixel |  | green | RED | t3f.Q | 02-1-tile_left |
| t3g | 2 tiling | Super+Shift+space (next layout) | regression | rect,pixel |  | green | RED | t3g.Q | 02-1-tile_bottom |
| t3h | 2 tiling | Super+Shift+space (next layout) | regression | rect,pixel |  | green | RED | t3h.Q | 02-1-fair |
| t4 | 2 tiling | Super+Shift+2 sends the focused window to tag 2, Super+2 vie | regression | rect,pixel |  | green | RED | t4.Q | 02-21-tag-2 |
| t5 | 2 tiling | Moves onto a window, Super+Ctrl+space floats it | regression | rect,pixel |  | green | RED | t5.Q | 02-22-floating |
| t5b | 2 tiling | Super+drag moves the floating window | regression | rect |  | green | RED | t5b.H50 | 02-23-wobble |
| t6 | 2 tiling | Super+F1 opens the key list | regression | rect,pixel | I6 | RED: I6: on HEADLESS-1 'Super+F1 shows this list again. | RED | t6.Q | 02-24-key-list |
| t6b | 2 tiling | Escape closes it | regression | rect,pixel |  | green | RED | t6b.Q | None |
| t6c | 2 tiling | Super+slash opens the key list once more | regression | rect,pixel | I6 | RED: I6: on HEADLESS-1 'Super+F1 shows this list again. | RED | t6c.Q | None |
| t6d | 2 tiling | Escape closes it | regression | rect,pixel |  | green | RED | t6d.Q | None |
| t7 | 2 tiling | Super+Print: a screenshot of the rice | regression | rect,added,pixel | I6 | RED: I6: on HEADLESS-1 'Screenshot saved' (notify-card- | RED | t7.Q+0.5 | 02-25-screenshot |
| t8 | 2 tiling | The beginner clicks Apps in the bar, the visible door to pro | regression | rect,pixel |  | green | RED | t8.Q | 02-26-apps |
| t8b | 2 tiling | Types Foot in the launcher and starts it | regression | rect,pixel |  | green | RED | t8b.Q | 02-27-foot |
| t9 | 2 tiling | Super+a: the chat of the helper beside the tiles | regression | rect,pixel |  | green | RED | t9.Q | 02-28-chat |
| t9b | 2 tiling | Escape closes the chat | regression | rect,pixel |  | green | RED | t9b.Q | None |
| t10 | 2 tiling | Scrolls on the volume word of the bar | regression | rect,pixel | expect | RED: the mixer is at 40 percent, expected 45 | RED | t10.Q | 02-29-volume |
| t10b | 2 tiling | Clicks the power icon at the right end of the bar | regression | rect,pixel |  | green | RED | t10b.Q | 02-30-power-menu |
| t10c | 2 tiling | Clicks the row Screenshot | regression | rect,pixel |  | RED: no new notice in chora-notify | RED | t10c.Q+0.5 | 02-31-power-screenshot |
| x1 | 1 xp | Erika sits down at the xp desktop | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x1.Q | 01-01-desktop |
| x2 | 1 xp | Start, then Notepad | regression | rect,meta,added,pixel | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x2.Q | 01-02-start-menu |
| x2b | 1 xp | No text editor is installed: Notepad says so | regression | rect,meta,added,pixel | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x2b.Q | None |
| x2c | 1 xp | Start, Notepad again, now with a stand-in GUI editor on PATH | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x2c.Q | 01-02c-notepad-gui |
| x3 | 1 xp | Types a short letter, one typo corrected, saves with Ctrl+D  | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x3.Q | 01-03-letter |
| x4 | 1 xp | Drags the letter window aside by its title bar | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x4.H50 | 01-04-wobble |
| x5 | 1 xp | Clicks the paperclip on the desktop (node hud-figure): she w | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x5.Q | 01-05-chat-open |
| x5b | 1 xp | Clicks the field and types her question, sends it with Return | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x5b.Q | 01-06-question |
| x5c | 1 xp | The helper works on it | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x5c.Q | None |
| x5d | 1 xp | She reads the answer | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x5d.Q+1 | 01-07-answer |
| x5e | 1 xp | Closes the chat with its X button | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x5e.Q | None |
| x7 | 1 xp | Start, Control Panel, Sound, 75% | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x7.Q | 01-08-sound-menu |
| x7b | 1 xp | The pointer rests on the speaker: the tray says 75% | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x7b.Q | 01-10-volume-75 |
| x8 | 1 xp | Murat presses Print Screen (Super+Print) | regression | meta,added,pixel | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x8.Q+0.5 | 01-10b-screenshot |
| x9 | 1 xp | Start, Run..., types a name that is no program | regression | meta,added,pixel | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x9.Q | 01-10c-run |
| x9b | 1 xp | Escape shuts the Run box | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x9b.Q | None |
| x10 | 1 xp | Start, Help and Support | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x10.Q | 01-10d-help |
| x10b | 1 xp | Clicks the topic Keyboard shortcuts | regression | meta,added | I6 | known | RED | x10b.Q | 01-10e-keys |
| x10c | 1 xp | Escape shuts Help | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x10c.Q | None |
| x11 | 1 xp | Start, Turn Off Computer | regression | meta,added,pixel | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x11.Q | 01-10f-turn-off |
| x11b | 1 xp | Murat thinks again and clicks Cancel | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x11b.Q | None |
| x6 | 1 xp | Opens the paperclip again and asks the helper to keep the le | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x6.Q | 01-11-second-ask |
| x6b | 1 xp | The helper asks before it touches a file | regression | meta,added | I6 | RED: I6: on HEADLESS-1 'Recycle Bin' (xp-icon-2) has th | RED | x6b.Q | 01-12-question |
| x6c | 1 xp | Erika clicks Yes | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x6c.Q | 01-13-done |
| x6d | 1 xp | She reads that it is done | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x6d.Q+1 | 01-14-reply |
| x6e | 1 xp | Closes the chat again | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x6e.Q | None |
| x5f | 1 xp | Clicks the paperclip icon in the tray (xp-helper) | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x5f.Q | 01-15-tray-icon |
| x5g | 1 xp | Closes the chat with its X button | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x5g.Q | None |
| x12 | 1 xp | Ctrl+Alt+Right: turns the xp desktop cube by key | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '1' (xp-pager-0-HEADLESS-1) has  | RED | x12.H50 | None |
| x12b | 1 xp | Ctrl+Alt+Left: back to the first desktop | regression | meta,added | I6 | RED: I6: on HEADLESS-1 '2' (xp-pager-1-HEADLESS-1) has  | RED | x12b.Q | None |
| x12c | 1 xp | Super held, drags on the desktop to turn the cube | regression | rect,meta,visible,added | I6 | RED: I6: on HEADLESS-1 '1' (xp-pager-0-HEADLESS-1) has  | RED | x12c.H50 | 01-19-super-drag-mid |

## Counts per chapter

| chapter | same | change | regression | none |
|---|---|---|---|---|
| 4 construct | 0 | 0 | 18 | 0 |
| 5 handover | 0 | 0 | 4 | 7 |
| 3b mc | 0 | 0 | 8 | 0 |
| 3 mc | 0 | 0 | 14 | 0 |
| 2 tiling | 0 | 0 | 28 | 0 |
| 1 xp | 0 | 0 | 31 | 0 |
