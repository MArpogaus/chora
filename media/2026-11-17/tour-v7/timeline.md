# Timeline of tour-v7, 2026-11-17

Build 7e06669 (session branch merged into claude/media; cargo build --locked, debug, headless). Times are in `tour-all.mp4` (4:16, 1920x1080).
Chapters: xp 0:00, tiling 1:19, mc light 2:17, mc dark 2:42, construct 2:58, handover 3:30. Stills (13 cited) are in `stills/`.
Harness: tiling bar through `flavors/tiling/waybar/waybar.sh`, `CHORA_TEST_KEEP_BUS=1`. This container has LibreOffice (`lowriter`), which opens no window in the headless session, so `launch.py` runs the sessions with a PATH that lacks `lowriter` (no GUI editor installed). A GUI editor for the xp letter and the handover is a stand-in script `mousepad` (a plain white foot window running `cat >> letter.txt`, saved with Ctrl+D): a stand-in, not a product claim.

## v6 MAJOR rows
| id | v7 result | where |
|---|---|---|
| j1-5 Print notice / bar Screenshot | FIXED | 2:05 Super+Print: notice "Screenshot saved /root/Pictures/Screenshots/..." bottom right (still 02-25). 2:08 click on the power icon: menu Lock, Log out, Restart, Shut down, Screenshot (02-26); 2:12 click on Screenshot: the notice shows (02-27) |
| j1-2/N1 Notepad | PARTLY: no nano any more, but the honest notice is not seen | 0:03-0:17: Notepad click gives the paperclip bubble "Starting Notepad / Please wait.", no window, nano nowhere. Then nothing: no "Cannot open Notepad / No text editor installed (for example mousepad)" in 14 s of frames (1 fps strip 0:08-0:17; still 01-02b); no Notify seen on the bus either. The notice named in the brief is NOT shown here (see F-1). With the stand-in mousepad on PATH and the setting `editor: mousepad` the window opens (0:17, 01-02c) |

## v6 MINOR rows and N-1..N-7
| id | v7 result | where |
|---|---|---|
| xp chat "Working" + Stop after answer | not re-checked in detail (xp script unchanged; xp chat path not in this build's fixes) | 0:30-1:19 |
| mc Super+a row overprint | still open | 2:17-2:42, 2:58 handover mc page 3:32+: "Chat with a helper" printed over the Super+a text (03-04c-answer-light) |
| mc dark launcher (j1-6) / N-2 raw entries | still open | 2:50: cream box, "Foot Client", "Foot Server", "ImageMagick (color depth=...)", LibreOffice entries (03-12-launcher-dark) |
| handover tiling black, no bar (N-5) | still open | 3:32-4:15: tiling screen black (multi.py starts no bar), 05-01/05-08 stills |
| handover Start menu stays open | still open | 4:10 end: Start menu over the window (05-10-give-key-xp-R) |
| handover bands | still open | stills 05-xx |
| mc wide page clock vs pill | still open | 2:30: "09:2x" touches the "No helper" pill (03-07-wide-light) |
| N-1 drag-back of the pane onto the face | STILL OPEN, retested with a plain drag (v6 held Super: script slip, the code says a plain drag over a face gives the window back; `modes.rs` press_on_a_pane) | 3:10-3:14: after the plain drag the owner of s1 is still `construct` (pane `construct-pane-s1`), the pane Sunset still in front of the face (04-13 equals 04-12). Construct r3 not on this build |
| N-3 mc light "No helper" while listening | still open | 2:17-2:42 (03-04c) |
| N-4 mc dark "3 AGENTS" empty 2 s | not re-checked | 2:42 |
| N-6 xp Start menu Vim/ImageMagick next to Notepad | still open, now also "LibreOffice" entries next to a Notepad that says no editor | 0:03 (01-02-start-menu) |
| N-7 construct use mode shows no HUD | unchanged (note) | 3:01 |
| rv5/v6 FIXED rows (power icon, separate pills, Helper in construct HUD, N2 step back, mc dark helper row, rv5-2 mc leg) | still fixed | tiling 1:21+; construct 3:01-3:08; handover 3:58-4:04 |

## New findings
| Time | Sev | Finding |
|---|---|---|
| 0:03-0:17 | MAJOR (open) | F-1: no text editor installed: Notepad shows "Starting Notepad" and then nothing visible; the notice that says no editor is installed does not appear in the tour session (chora-notify and a session bus run). The gate `gate-xp-notepad-gui-only` passes with its own bus monitor; the difference is the tour environment (companions: chora-notify). Needs a look by the xp lane |
| 0:03 | MINOR | F-2: this container has `lowriter` (LibreOffice). Notepad tries it first; in the headless session it opens no window and, within 48 s, no notice and no fallback was seen (probe, not in the video). A person with LibreOffice but no window gets a silent Notepad |
| 3:35 and 3:45 | note | the handover editor is the stand-in window; the mc leg 3:58-4:04 shows a bare page "7 NOTEPAD" without a frame (no longer a nano footer) |
