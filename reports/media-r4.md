# media r4 report (rc 11, 1, 1b), 2026-11-17

Build aa8aa8c0 (cargo build --locked, debug, headless). Scripts on claude/media 8ae77dd (`demo/tour/v6_*.py`,
`tour6.py`, `assemble6.py`; copies of v5 with the tiling companion changed). Output
`media/2026-11-17/tour-v6/`: six chapter MP4, `tour-all.mp4` 3:28 (1920x1080), `timeline.md` (rows plus the v5 table),
17 cited stills (the full set of 81 is not in the log branch). Sonnet worker, one round.

## Done
- Tour v5 scenarios re-recorded unchanged in path (no fix changed a path; all scripts ran first time).
- Harness: the tiling bar starts as `dbus-run-session -- sh flavors/tiling/waybar/waybar.sh flavors/tiling/waybar`
  (choractl on PATH, jq present), not waybar directly; `CHORA_TEST_KEEP_BUS=1` is set for every chapter.
  The handover session still has no bar on the tiling screen (multi.py starts only chora-tiling).
- Chapters (tour-all): xp 0:00, tiling 0:57, mc light 1:43, mc dark 2:07, construct 2:22, handover 2:50.

## v5 MAJOR and MINOR table (times in tour-all.mp4)
| id | sev | result | where |
|---|---|---|---|
| rv5-1 lock/power by mouse | MAJOR | fixed: power icon in the bar | 0:59, still 02-01 |
| j1-5 Print notice | MAJOR | open: Super+Print shows nothing | 1:41, still 02-25 |
| j1-4 "No helperVolume 40%" | MAJOR | fixed: separate pills | 0:59, still 02-01 |
| j1-2/N1 Notepad | MAJOR | open: nano in a terminal frame (stub; no GUI editor here) | 0:03, still 01-03 |
| rv5-2 mc leg of handover | MAJOR | fixed in the still: window drawn in mc ("7 NOTEPAD") after the drop; ~3 s of empty agenda while it is dragged in | 3:13-3:21, still 05-05-drag-xp-to-mc-R |
| rv5-3 helper in construct HUD | MAJOR | fixed: Helper in the HUD | 2:26, still 04-11 |
| N2 step back after pull | MAJOR | fixed: pane shown beside the face; back on the face works | 2:31-2:36, stills 04-12, 04-13 |
| xp chat "Working" with Stop | MINOR | open: "Working / Done, step 1 of 1" + Stop after the answer | 0:30, 0:56, stills 01-07, 01-14 |
| mc dark "No helper" with helper row | MINOR | fixed: chat row says "Working" | 2:20, still 03-13 |
| mc Super+a row overprint | MINOR | open | 1:53, 2:09, stills 03-04c, 03-10 |
| mc dark launcher (j1-6) | MINOR | open: cream box, raw entries | 2:15, still 03-12 |
| handover bands/black tiling/Start open | MINOR | open | 2:52, 3:13-3:28, stills 05-01-R, 05-08-L |
Result: 5 of 7 MAJOR fixed (rv5-1, j1-4, rv5-2, rv5-3, N2); j1-5 and j1-2/N1 open.

## New findings
| Time | Sev | Finding |
|---|---|---|
| 1:57 | MINOR | mc wide page: clock "06:07" runs into the "No helper" pill (still 03-07) |
| 3:20 | note | mc leg shows a bare nano page headed "7 NOTEPAD", no frame |

## Deviations
- Looks checked on stills (xp, tiling, mc light and dark, construct, handover) and a 1 fps strip of the mc leg;
  not every still. rv5-2 judged on the strip: not followed second by second.
- The tour pace was not re-checked with freezedetect. Chapter lengths 14-56 s, total 3:28 (under 6 min).
- Only the scripts are on claude/media (the media hook limits files to 500 KB); media is on this branch.

## Refactor proposals (harness)
- demo/tour/v4_*, v5_*, v6_* and tour/tour5/tour6, assemble4-6: each round copies every script. One parametrised
  set (`TOUR_VERSION`, one assemble.py) removes ~900 lines of copies. M.
- demo/tour/multi.py: the handover session has no bar on the tiling screen; start it through waybar.sh there too, and
  one helper `start_tiling_bar(L)` in launch.py for v6_tiling.py:21 and multi.py. S.
- launch.py: needs /usr/share/chora/flavors linked by hand; make Launcher set CHORA_FLAVOR_DIR or link it. S.
- assemble: the timeline header has a hard-coded date and "expected" text from the script, not what was seen; add
  an "observed" column filled by the pixel asserts. M.
- Notepad needs a GUI editor stub; the tour's `editor:` override (nano in foot) lives in each script. Move to launch.py. S.
- Cut check (plugins): not examined in this lane; no plugin code was read.
