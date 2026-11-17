# media r6 report (rc 11, 1b), 2026-11-17

Build: session branch 733967aa merged into claude/media; scripts at claude/media 0818f569. Videos, stills, timeline: branch claude/media-v8, `media/2026-11-17/tour-v8/` (tour-all.mp4 4:04, 4.9 MB; six chapter MP4 under 1.2 MB; 95 stills). Sonnet worker, one round.

## Done
- Harness: `launch.py` hides the writers from every PATH entry that resolves to /usr/bin (asserts `lowriter` is gone); construct drag aims at the pane centre and checks the owner; gate `gate-construct-pull.sh` runs the drag back twice, with Super and plain (cell not run here); xp sessions keep their bus; `probe.py` prints the bus.
- Tour v8 recorded for all six chapters, plus xp cube (key and Super+drag), xp helper by paperclip and by tray icon, construct stepped back at 0/45/90.
- Videos kept on claude/media-v8 only.

## Results (full table: timeline.md on claude/media-v8)
- N-1: FIXED in the tour; a plain drag from the pane centre gives the window to xp (`PANE HIT ok`, 04-12, 04-13).
- F-1: the product sends the notice (bus: "Cannot open Notepad / No text editor installed (for example mousepad)", probe captures at 1 s and 6 s). The old miss was harness: xp sessions had no CHORA_TEST_KEEP_BUS. Still not visible in the recorded chapter (0:13): open, cause unknown (recorder?).
- xp cube: Ctrl+Alt+Right/Left switch desktops, Super+drag turns the cube; new MINOR: Start menu stays open after the Super+drag (01-20).
- Helper opens by paperclip (hud-figure) and by tray icon (xp-helper).
- Seams: 0 and 90 recorded; 45 not reached (a 180 px drag turns a few degrees). No gap seen at this scale.
- Still open from v7: mc Super+a overprint, clock vs pill, N-3, dark launcher raw entries, handover tiling black, Start menu left open in the handover, chat "Working"+Stop after answer.

## Deviations
No gate run. xp chapter recorded four times. N-4, N-7, mc dark helper row not re-checked. The branch claude/media-v8 was committed with `PRE_COMMIT_ALLOW_NO_CONFIG=1` (the branch holds no config; no --no-verify). Local claude/media was stale; I merged into a fresh branch and pushed it as claude/media.

## Refactor proposals (harness)
- One parametrised tour set (`TOUR_VERSION`) instead of v4..v8 copies (~650 lines each). M.
- `Quick.open_start()` that verifies the menu and clicks again; `act(..., expect=)` that flags a failed state in the timeline. S.
- Make `choractl record` show notices like a live session, or document why not (F-1 video). M.
- A camera call or `cube.set_yaw` in the harness for exact 45 degrees. S.
- Fold probe.py into launch.py as a `Launcher.bus_monitor()`. S.
