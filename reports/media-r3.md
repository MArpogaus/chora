# media r3 report (rc 11, 1, 1b), 2026-11-17

Build 4105f8d1 (cargo build --locked, debug, headless, software rendering). Scripts on claude/media
0f941e2 (`demo/tour/v5_*.py`, `tour5.py`, `assemble5.py`). Output `media/2026-11-17/tour-v5/`: five chapter
MP4 plus the handover, `tour-all.mp4` 3:48 (1920x1080), `timeline.md`. Sonnet worker, one round.

## Done
- Tour v4 scenarios re-recorded on the fixed build. Path changes: xp Notepad from the Start menu (the stub
  "Editor" is gone), chat closed by its X, Control Panel cascade waits for hover; handover frame 1920x1080
  (screens at 0.75, was 0.5) plus a 3 s window audit card at its end; chapter cards carry today's date.
- Added (backlog 11-16 10:05): mc hides and shows the column by Super+n, the page widens (mc light 0:19);
  construct pulls a window off the face over the edge into the space (construct 0:04, over a second
  output as tests/gate-construct-pull.sh does; a single output cannot).
- Chapters: xp 1:07, tiling 0:48, mc light 0:25, mc dark 0:16, construct 0:26, handover 0:46.

## Journey-1 table (times in tour-all.mp4)
| id | sev | result | where |
|---|---|---|---|
| j1-1 paperclip | MAJOR | fixed: paperclip in the tray and at the bubble | xp 0:24-0:35 (1:24 on), still 01-10 |
| j1-2 Erika's editor | MAJOR | still open: Notepad opens, but as nano in a terminal frame; Writer cannot open here (see N1) | xp 0:03-0:19, still 01-03 |
| j1-3 handover layout | MAJOR | fixed: two screens at 0.75 in 1920x1080, text readable; dark bands top and bottom remain | 3:03-3:49, still 05-04 L/R |
| j1-4 tiling bar | MAJOR | still open: "No helperVolume 40%" runs together; the bar has the Connected pill and Keys | 1:07-1:09, still 02-01 |
| j1-5 Print notice | MAJOR | still open: Super+Print shows nothing | 1:53, still 02-25 |
| j1-6 mc dark launcher | MINOR | half: dark now; raw entries (UXTerm, XTerm, Foot Server, ImageMagick) | 2:28, still 03-12 |
| j1-7 ticker over footer | MINOR | not seen in the light chapter | 2:11-2:19 |
| j1-8 window pull | MINOR | fixed in the tour: window leaves the face as a pane | 2:40-2:46, still 04-11 |
| j1-9 card dates | COSMETIC | fixed (2026-11-17) | each chapter 0:00 |
| BLOCK co-18 lost window | BLOCK | fixed in the tour: window s1 through xp, tiling, construct, xp, mc, xp and two key gives; audit: one window, owner xp | 3:03-3:49 |
| BLOCK mc-32 dead remote | BLOCK | fixed in the tour: Left/Right turn the page | mc light 2:11, mc dark 2:23 |
Reviewer ledger ids (xp-25..28, ti-22..25, mc-32..37, co-18..19, sd-h1..h2) are not in the brief: only the
named rows are judged. The tour pace: freezedetect (-80 dB, 1.5 s) finds only the 2.5 s chapter cards, plus
the audit card (3 s) and 6 holds of 1.5 to 4.2 s in the handover (screen waits while the window is moved).

## New findings (not fixed)
| Time | Sev | Finding |
|---|---|---|
| N1 xp 0:03 | MAJOR | Notepad with Writer installed: lowriter does not open (only the gen VCL plugin, no gtk3/Wayland), the only sign is a notify error "Cannot open lowriter"; no fallback to nano or another editor. The tour overrides `editor:` with a nano window. Unverified on a real display. |
| N2 construct 2:46 | MAJOR | After the pull, stepped back (Alt+Escape) shows neither the pane nor the window on the face; "back over a face" could not be recorded (no pane node to drag). Still 04-12. |
| N3 xp 0:27 | MINOR | The first key of a question typed right after the click on the chat field ("H") went into the editor behind (stray key to a client). Fixed in the script by a 0.4 s wait; unverified as a product race. |
| N4 xp 0:35 | MINOR | Escape did not close the chat while it showed "Working" in the recording (worked live in a probe): unverified. |
| N5 xp 0:38 | MINOR | Start menu lists raw "Vim" and "ImageMagick (color depth=q16)" under Notepad. |
| N6 xp 0:41 | note | The Control Panel cascade closes if the pointer leaves it on the way to Sound; the script goes straight down inside it. |
| N7 mc 2:11 | note | The column says "No helper" while the stand-in agent runs (the tour does not announce it). |
| N8 xp 0:19 | note | No GUI editor is installed (no mousepad/gedit); the tour needs the nano stub for any editor window. |

## Deviations
- Stills reviewed one by one: xp, tiling, mc light, construct pull; the dark mc and the handover only by
  sampled frames. Only cited stills are in the log branch (19 MB in all).
- Tour 3:48 total, under 6 min; looks are 16 to 67 s (xp now longer: the Control Panel steps wait for hover).
