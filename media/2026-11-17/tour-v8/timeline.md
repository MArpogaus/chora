# Timeline of tour-v8, 2026-11-17

Build: session branch 733967aa (train 40) merged into claude/media; scripts at claude/media 0818f569 (`demo/tour/v8_*.py`, `probe.py`). Debug build, headless, software rendering.
Times are in `tour-all.mp4` (4:04, 1920x1080). Chapters: xp 0:00, tiling 1:19, mc light 2:13, mc dark 2:37, construct 2:52, handover 3:26. Per-act table: `timeline-table.md`. Stills in `stills/`.
Harness changes this round: PATH without writers on every /usr/bin entry (asserted), construct drag aims at the pane centre and checks the owner, xp sessions keep their bus (`CHORA_TEST_KEEP_BUS`), `probe.py` prints the bus, gate `gate-construct-pull.sh` has a plain-drag cell (not run in this lane).

## v7 rows
| id | v8 result | where |
|---|---|---|
| F-1 Notepad notice without editor | PRODUCT OK, NOT VISIBLE IN THE VIDEO. Harness cause fixed: xp sessions had no CHORA_TEST_KEEP_BUS, the notifier connected to a bus nobody ran (core log: "notify Cannot open Notepad: Failed to connect"). Probe on the bus: Notify "Cannot open Notepad / No text editor installed (for example mousepad)." Probe captures show the notice bottom right at 1 s and 6 s. In the recorded chapter (`choractl record`) the 6 s wait after the click still shows no notice: open, harness or recorder, not found | 0:13; stills 01-02b-probe-bus-1s, 01-02b-probe-bus-6s (probe), 01-02b-no-editor (video, empty) |
| N-1 drag-back onto the face | FIXED in the tour. Plain drag (no Super) from the pane centre: owner of s1 is `xp` again, no pane (`PANE HIT ok`) | 2:52+; 04-12 (pane), 04-13 |
| j1-5 Print / bar Screenshot | still fixed: notice after Super+Print, power menu, click on Screenshot gives a second file | 1:19+0:44..0:50; 02-25, 02-26, 02-27 |
| mc Super+a overprint | still open: "Chat with a helper" over the Super+a text in the KEYS list (light) | 2:13+; 03-04c |
| mc wide page clock vs pill | still open: clock touches the "No helper" pill | 03-07 |
| N-3 mc light "No helper" while speaking | still open | 03-04c |
| mc dark launcher raw entries (N-2) | still open: Foot, Foot Client, Foot Server, ImageMagick (color depth=...), LibreOffice x5, Vim, UXTerm, XTerm, Screenshot of an area | 2:37+; 03-12 |
| handover tiling black, no bar (N-5) | still open: that screen is black | 3:26+; 05-01-R |
| handover Start menu left open | still open: Start menu over the Notepad window while typing | 05-10 |
| N-6 xp Start menu entries | Start menu now Terminal, Chat with a helper, Notepad, UXTerm, XTerm (no LibreOffice/Vim); UXTerm and XTerm remain | 01-02-start-menu |
| xp chat "Working"+Stop after answer | still open: header "Atlas - Working, Done, step 1 of 1" with Stop after the final answer | 1:07; 05-10 sheet, 01-14 |
| N-4 mc dark "3 AGENTS", N-7 construct use mode no HUD | not re-checked | - |
| power icon, separate pills, Helper in construct HUD, N2 step back | still fixed | tiling 1:19+, construct 2:52+ |

## Owner questions and new acts
| act | result | where |
|---|---|---|
| xp desktop cube, Ctrl+Alt+Right / Left | works: the pager goes to desktop 2 and back; the turn is over by the still (no mid-frame caught) | 1:14, 1:16; 01-16, 01-17, 01-18 |
| xp desktop cube, Super+drag | works: the desktop turns as a cube, the Notepad window skewed on its face | 1:17; 01-19 |
| xp helper by the paperclip on the desktop (hud-figure) | chat opens at the bottom right | 0:33; 01-05 |
| xp helper by the tray icon (xp-helper) | chat opens (the chat close button is in the scene) | 1:10; 01-15 |
| construct stepped back at 0, 45, 90 degrees (co-21) | 0: cube with front and right face; a lighter ghost of the cube lies below (floor reflection). 45: a 180 px drag moved the cube only a few degrees, so 45 was NOT reached; stills look like 0. 90: next face (XP 2) in front. No gap between faces seen at this scale; seams go to known issues | 2:52+0:27..0:32; 04-20, 04-21-seam-45a/b, 04-22 |

## New findings
| Time | Sev | Finding |
|---|---|---|
| 1:17 | MINOR | xp: after Super+drag on the cube the Start menu stays open over the desktop; 01-20 |
| 0:13 | MINOR (harness) | the recorded run shows no notice that a live session shows; compare `choractl record` and a plain capture (probe.py) |
| 2:52+ | note | construct: windows overhang the face edge at the lower left in the cube view, 04-20 |
| 2:52+ | note | 45 degrees is not reachable with a 180 px drag; a camera call would be needed |

## Deviations
No gate cell run (the plain-drag cell is untested). Not every still viewed. The xp chapter was recorded four times (F-1 chase); the videos are the last run.
