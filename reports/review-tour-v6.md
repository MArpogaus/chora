# Review of tour v6 (independent, read only), 2026-11-17

Build aa8aa8c0; media/2026-11-17/tour-v6/tour-all.mp4 (3:28). Method: 1 fps over the whole tour (209 frames, in
9-frame contact sheets), 2 fps around the Print key (1:40-1:46), 1 fps strips of construct (2:29-2:50) and handover
(2:50-3:28) at larger scale, plus stills 04-12 and 04-13. Frames are in /tmp/claude-0/rv6 (not in the repo).
Times are tour-all.mp4. media-r4 was read first for its claims and checked against the frames; two claims are wrong
(N2 "back on the face", Notepad "stub" is not a fix).

VERDICT: FIX (2 MAJOR open: j1-5, j1-2/N1; 5 of 7 v5 MAJOR fixed, as media-r4 says).

## v5 MAJOR rows
| id | result | evidence |
|---|---|---|
| rv5-1 lock/power by mouse | FIXED | 0:59-1:41: bar right end "Keys" plus a power glyph (f060, f100); key list line "power icon: lock, log out". The click itself is not shown, only the visible path |
| j1-5 Print notice | OPEN | 1:40 Print: list closes, 1:41-1:42 (f101-f102 and 2 fps strip) show the same tiles, no notice, no sign; no bar button for screenshot either. Key list names Super+Print "Screenshot" |
| j1-4 "No helperVolume 40%" | FIXED | 0:59 and every tiling frame: "No helper", "Volume 40%", "Connected" are separate, spaced items (f060) |
| j1-2/N1 Notepad | OPEN | 0:03-0:56 and 3:00-3:28: window titled Notepad holds GNU nano 7.2 with its ^ key footer; saved by Ctrl+O and Return (f008-f011, f012). Erika (mouse only) cannot use this; the launch failure path (honest notice) is not shown either |
| rv5-2 mc leg | FIXED | 3:13-3:21: the window is dragged over the xp edge (f193), mc shows the window over the agenda (f194-f195), then the full page "7 NOTEPAD" with the letter (f196-f200). No empty gap longer than 1 s in my strip. The mc page is bare nano (see M-new 3) |
| rv5-3 helper in construct HUD | FIXED | 2:26-2:50: HUD "06:09 Programs Helper Keys Lock Power Screenshot Volume 40%" (f146 on) |
| N2 step back after pull | FIXED (narrowly) | 2:31: stepped back shows the cube, the xp face small at the left and the pane "Sunset" in front (f151). The window is no longer invisible. But see N-1: the pane never goes back to the face |

## MINOR rows of media-r4
| id | result | evidence |
|---|---|---|
| xp chat "Working" + Stop after answer | OPEN | 0:30-0:56: header "Atlas - Working", "Done, step 1 of 1", Stop live, with the answer and the Yes/No card finished (f031-f058) |
| mc dark "No helper" with helper row | FIXED | 2:20: "Helpers: Atlas working - Reminder" and chat row "Working" (f140-f141); pill at the top of the dark page not visible in this layout |
| mc Super+a row overprint | OPEN | 1:53-2:09, 2:52: "Chat with a helper" printed over the Super+a description (f114-f119, f131) |
| mc dark launcher | OPEN | 2:15: cream box with "UXTerm, XTerm, Foot Server, ImageMagick (color depth=...)" (f136) |
| handover bands / tiling black / Start open | OPEN | 2:52-3:08 tiling screen black, no bar; Start menu stays open 3:13-3:28 over the window (f176, f190-f205); dark bands around the two screens |
| mc wide page clock vs pill | OPEN (media-r4 "new") | 1:57-2:01: "06:07" touches the "No helper" pill, top right (f122-f124) |

## New findings
| id | time | sev | what a user sees / why |
|---|---|---|---|
| N-1 | 2:29-2:44 | MINOR (lean MAJOR if it cannot be done) | The Sunset pane stays in front of the cube through every frame after the "drag back" act; the face at the left never gets the window, no taskbar entry (f149-f161; still 04-13 is identical to 04-12 and shows the pane). media-r4 "back on the face works / no pane" is false for this still. The window is not lost (pane visible, handover audit is fine), so not stuck as far as shown; a retest of the drag-back is needed |
| N-2 | 1:20 and 2:15 | MINOR | Launchers list raw entries ("ImageMagick (color depth=q16)", "Foot Client", "Foot Server") in tiling and mc, also in the xp Start menu. Looks unfinished for Erika/Murat; same as j1-6 family |
| N-3 | 1:47-2:09 | MINOR | mc light pill says "No helper" while "listening/speaking" and the voice answer ("Four cloves, sliced thin.") show; the dark page was fixed, the light one was not (honest state) |
| N-4 | 2:13 | COSMETIC | mc dark page "3 AGENTS" is empty for 2 s with only a heading; next "Mission Control" |
| N-5 | 3:22-3:28 | MINOR | The give-by-key leg puts the window on the tiling screen, which has no bar: a tiling user there has no visible Apps/Keys/power. Part of the "handover tiling black without bar" row; media-r4 left multi.py unchanged |
| N-6 | 0:03-0:20 | COSMETIC | xp Start menu lists "Vim", "ImageMagick" under pinned items next to Notepad; Notepad is the first thing the tour starts, so Erika sees the mismatch right away |
| N-7 | 2:24-2:30 | note | Use-mode frames of the construct (xp face 1:1) show no HUD; lock/power there come only from the xp Start menu. Fine for the bar, worth a line in the docs |

Checks that passed: xp key paths (Start, paperclip, Control Panel > Sound 75%, tray volume and clock, Yes/No card
before the helper touches a file); tiling bar (Apps launcher, clock, tags, Keys list on a mouse click, layouts);
mc remote path (Left/Right, Home), mouse Programs click, Super+n wide page, critical red note; construct keys (F1
list shows give-a-window keys and Super+Print), chip turning, step in and out, drag-turn; handover audit "one
window, none lost" at 3:28 with owner xp.

## Why FIX
Open MAJOR: j1-5 (screenshot gives no sign and has no mouse path in the tiling look) and j1-2/N1 (Notepad is a
terminal editor; the RC bar needs a start-a-program path a mouse-only user can finish). j1-2 may become
"known issue with a reason" only if the PM accepts the container having no GUI editor; the review cannot
decide that from the tour. Suggested lanes: tiling (Print notice, or a bar Screenshot pill), harness (a GUI
editor stub so the xp Notepad is a real window), construct (retest drag-back, N-1).

## Frames viewed
Contact sheets sheet01-sheet21 (f001-f189 at 1 fps, 640 px tiles, every 1 s); d1/print.png (1:40-1:46, 2 fps);
d1/c1.png and d1/c2.png (2:29-2:50 at 1 fps); h/s01-s07 (2:50-3:28, 960 px, 1 fps); stills 04-12 and 04-13.
Every chapter was covered at 1 s steps; no stretch longer than 1 s was skipped. Not viewed: the other stills,
the chapter MP4s on their own (tour-all was used), and the audio (none).
