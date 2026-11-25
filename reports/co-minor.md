## r1

Lane: cloud, branch claude/co-minor-r1 from 6f37120b. Step 0: `git fetch --unshallow`, `cargo check --workspace` green. Area: plugins/chora-construct (camera.rs, adopt.rs, plugin.rs, overlay.rs, main.rs), unit tests, gate-construct-cube. 8 files +227 -10.

**Head: dd1ac7e4** (red 6c040c09, fix 7becf38f, then dd1ac7e4 drops a sleep that the sleep-ratchet counted). Pushed to claude/co-minor-r1.

### Cause (co-78 and rcr1-1, one chain)
With the key sheet open over a face the construct holds the keyboard (a node focus on its look). A new window goes to the holder of the keyboard (core rule: "to the focus, else the smallest claim under the pointer"), so the window of Super+t was reserved for the construct, which adopted it as a pane (`terminal_beside`, flown in); the face stayed empty (rcr1-1). The window then took the keyboard from the sheet, so Escape went to it, not to the sheet (co-78). Hand repro on 6f37120b: after Super+t `s1 owner construct`, rect 0,0,1280,800, Escape leaves `open: construct-help` (as co-repro r1).

### Fixes
- rcr1-1: in a face (`in_a_face()`: cube in use mode, camera in no pane) `terminal_beside` starts the terminal for the face (`start_terminal_in_face`): the token is kept apart from the pane waits, no pane is made; when the window comes (`created`) the construct takes what the core gave it and lets it go to the manager of the faces (`Remove {to: xp}`); xp shows it on the face.
- co-78: a focus on a non-pane window while the key sheet is open in a face (`focus_heard`) is noted for the close and the keyboard goes back to the sheet (`help_retakes_keyboard`); Escape closes the sheet (existing path) and the close gives the noted window the keyboard (`activate_in_face`).

### Red then fix (pasted red lines)
| Item | Red | Fix | Red line |
|---|---|---|---|
| rcr1-1 | 6c040c09 | 7becf38f | unit `face_start_tests.rs:19`: `the terminal of Super+t over a face is no pane of the construct: 1 waits for a pane`; cell: `FAIL: rcr1-1: Super+t with the key sheet open over a face started a window of construct, not of the face manager xp: s2 construct HEADLESS-1 566 326 148 148` |
| co-78 | 6c040c09 | 7becf38f | unit `face_start_tests.rs:37`: `the keyboard goes back to the sheet: []`; cell (owner check skipped to reach it): `FAIL: rcr1-1: the window of Super+t is on HEADLESS-1, not on the face VIRTUAL-1` (the pane; the Escape line behind it is hidden until the window is in the face) |
| co-77 | none (pin, see below) | - | - |

The two items share one red commit and one fix commit: the gate cell is one scenario (sheet, Super+t, Escape) in one file. Green after: `ok: rcr1-1: Super+t with the key sheet open over a face starts a window of the face (xp, VIRTUAL-1)`, `ok: co-78: Escape closes the key sheet after Super+t opened a window of the face, and that window has the keyboard`. The cell asserts nodes, not a look: owner `xp` and output of the window by `scene.query`, `shown` in `state xp`, the sheet node `construct-help` opacity back to 0, and the keys 'a' of the probe window log after Escape.

### co-77: no fault; contract right, brief reading differs (one line for the PM)
A program from Programs (the launcher) in a face that holds a window is held by **xp**, as the faces of a hosted cube are (D106: xp holds the windows of its faces, the construct holds the cube and the turn; WM7b), and that is what the contract C3 already accepts (`focus_with_flavor`: the manager of the front face on a window that shows): hand run in a face with a Super+t window, then Programs, Foot: `xp focus s2`, `construct holder null`, both windows owner xp, windowed 700x500 inside the face, `check.py end C3`: `task C3 pass [scene: cube in a face, face 0]` (5 of 5 in co-repro r1 and in the cell below). I did not make construct hold it: that would break D106 and the faces' window manager (taskbar, title, pull). The r12 C3 FAIL (stood back, xp s3, "shown" probably false) did not reproduce (stood back with no face window Foot is a construct pane and C3 passes; stood back with a face window the launcher started nothing in my run). **Decision for the PM:** if a stood-back cube should also give Programs windows to the construct, that is a product call; the contract is not wrong and needs no change.
Cell (green, a pin, no red): `ok: co-77: a program of the launcher in a face that holds a window is shown on that face, held by the face manager xp, and takes keys, 5 of 5 runs` (gate-construct-cube; launcher is a stand-in that starts a chora-probe window, no child of the launcher, ends at once as fuzzel does).

### Captures (media/2026-11-25/co-minor-r1/)
- rcr1-1-super-t-sheet-open-window-in-face.png: sheet open, the terminal window behind it on the xp face (not a pane).
- co-78-escape-closes-sheet-window-has-keys.png: sheet closed by Escape, the window focused (blue frame), typed `abc` in it, hint bar names Pull out.
- co-77-foot-from-programs-in-face.png: Foot from Programs in the face that holds a terminal, two xp windows.
- hand.sh: the hand run (cube row, Return, Super+F1, Super+t, Escape; and Super+t, Super+space, foot).

### Gate
`tests/merge-gate.sh --jobs 2` (no step filter), detached, polled in the turn, on dd1ac7e4: **EXIT 0, 65 PASS, 0 FAIL**. A first run on 7becf38f stopped at `ci-check` (sleep-ratchet: sleep 779 vs 778, my `sleep 3` in the stand-in launcher), fixed in dd1ac7e4 (no new sleep in the tree). Unit: chora-construct 288 passed, clippy clean. `pgrep -fa 'chora --headless'` empty after. Last 20 lines:
```
voice-acp  PASS    13s /tmp/merge-gate.lV6cwD/voice-acp.log
voice-events PASS     2s /tmp/merge-gate.lV6cwD/voice-events.log
agent      PASS    42s /tmp/merge-gate.lV6cwD/agent.log
tiling-keys-more PASS    44s /tmp/merge-gate.lV6cwD/tiling-keys-more.log
tiling-windows PASS   210s /tmp/merge-gate.lV6cwD/tiling-windows.log
tiling-system PASS    16s /tmp/merge-gate.lV6cwD/tiling-system.log
handover   PASS    28s /tmp/merge-gate.lV6cwD/handover.log
mc-scale   PASS    18s /tmp/merge-gate.lV6cwD/mc-scale.log
watchdog-starved PASS    58s /tmp/merge-gate.lV6cwD/watchdog-starved.log
acts       PASS  2351s /tmp/merge-gate.lV6cwD/acts.log
xp-run     PASS    15s /tmp/merge-gate.lV6cwD/xp-run.log
demo       PASS   279s /tmp/merge-gate.lV6cwD/demo.log
leak       PASS   483s /tmp/merge-gate.lV6cwD/leak.log
tiling-theme PASS    19s /tmp/merge-gate.lV6cwD/tiling-theme.log
tiling-disc PASS    71s /tmp/merge-gate.lV6cwD/tiling-disc.log
mc-host    PASS    20s /tmp/merge-gate.lV6cwD/mc-host.log
tiling-bar PASS   201s /tmp/merge-gate.lV6cwD/tiling-bar.log
e2e        PASS   287s /tmp/merge-gate.lV6cwD/e2e.log
all steps passed; no --push, nothing is pushed
EXIT 0
```

### Not done / open
- co-77 owner stays xp by design (above); the r12 stood-back FAIL is not reproduced and not explained (the xp start menu click path of Max was not replayed exactly).
- Stood back, Super+t with the key sheet open still starts a pane (K-mode navigate, unchanged by design: the fix is for a face only).

## PM decision r1 (2026-11-25 10:20 UTC)
- co-77: no fault. D106 and WM7b hosted (owner 11-19 10:52) hold: a Programs window in a face is held by xp; C3 pin cell passes 5/5. Closed as by design after the review confirms the pin cell.
- Diff 227 lines, construct focus path: seeded opus review (briefs/review-co-minor-r1.md).
