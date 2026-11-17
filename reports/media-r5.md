# media r5 report (rc 11, 1), 2026-11-17

Build 7e06669 (session branch merged into claude/media; debug, headless). Scripts on claude/media e48ae83 (`demo/tour/v7_*.py`, `tour7.py`,
`assemble7.py`; `launch.py`, `multi.py` changed). Capture in the media worker's container `media/2026-11-17/tour-v7/`: six chapter MP4, `tour-all.mp4`
4:16, 1920x1080 (the repo hook refuses media over 500 KB and any media in the repo; only timeline.md and the small stills are here). Sonnet worker, one round.

## Done
- Tour v6 scenarios re-recorded as v7. Path changes: xp (Notepad notice, stand-in editor, `settings.set editor`), tiling (power-menu Screenshot click), construct
  (plain drag), handover (stand-in editor). Tiling bar through waybar.sh, `CHORA_TEST_KEEP_BUS=1`.
- This container has LibreOffice: `lowriter` opens no window in the headless session. `launch.py` now runs sessions with a PATH without `lowriter` so "no GUI editor".

## v6 MAJOR
| id | result | where |
|---|---|---|
| j1-5 Print / bar Screenshot | FIXED. Super+Print gives the notice "Screenshot saved ..."; the power icon opens Lock, Log out, Restart, Shut down, Screenshot; a mouse click on Screenshot gives the notice | 2:05, 2:08, 2:12; stills 02-25, 02-26, 02-27 |
| j1-2/N1 Notepad | PARTLY. No nano, no terminal. With no GUI editor: "Starting Notepad / Please wait." and then nothing: the honest notice naming mousepad is not seen in 14 s (F-1). With a stand-in `mousepad` and the setting set: a window opens | 0:03-0:17, still 01-02b, 01-02c |

## v6 MINOR and N-1..N-7
Full table: media/2026-11-17/tour-v7/timeline.md. Summary: still open: mc Super+a overprint (2:17+), mc dark launcher raw entries (2:50), clock touches
"No helper" pill (2:30), handover tiling black with no bar and Start menu left open (3:32-4:15), mc light "No helper" while listening (N-3), Start menu entries
Vim/ImageMagick/LibreOffice (N-2, N-6). Not re-checked: xp chat "Working"+Stop, N-4.
**N-1 drag-back: still open, retested with a plain drag.** v6 held Super: a script slip (code: a plain drag over a face gives the window back, modes.rs
press_on_a_pane). Plain drag: owner of s1 stays `construct` (`construct-pane-s1`), still 04-13 equals 04-12 (3:10-3:14). Construct r3 is not on this build.

## New findings
| Time | Sev | Finding |
|---|---|---|
| 0:03-0:17 | MAJOR | F-1: Notepad with no text editor installed shows "Starting Notepad" and then nothing; the notice "No text editor installed (for example mousepad)" never appears in the tour session (chora-notify and a bus run; no Notify seen on the bus). Gate `gate-xp-notepad-gui-only` passes; differs from the tour in having no chora-notify |
| 0:03 | MINOR | F-2: with `lowriter` on PATH and no window opening (this container), Notepad stayed silent for 48 s (probe, not in the video) |
| 3:58-4:04 | note | handover mc leg: a bare page "7 NOTEPAD" with the stand-in editor text, no frame |

## Verdict hint
Open MAJOR in the tour: F-1 (silent failure of Notepad without an editor). j1-5 closed. N-1 stays MINOR, lean MAJOR if not fixed in construct.

## Deviations
- Contact sheets and 1 fps strips of the xp and handover chapters and stills of tiling, mc, construct; not every still. xp chat Stop row and N-4 not re-checked.
- The stand-in `mousepad` is a plain foot window with `cat`; the letter is saved with Ctrl+D. Not a product claim.
- The handover chapter took ~25 min (1 fps recording); one background run was killed at its 30-min limit and re-run alone.

## Refactor proposals (harness)
- demo/tour/v4_* to v7_*: another full copy (~650 lines). One parametrised set (`TOUR_VERSION`) and one assemble.py. M.
- launch.py: add `no_gui_editor` / stand-in `mousepad` helpers, so xp and multi.py share it (now two copies of the stub). S.
- Probe helper: `demo/tour/probe.py` that runs a session and prints notices from the bus; this round needed ad-hoc probes to see F-1. S.
- assemble: still takes the still at the act start; for acts that need a wait, a `t.still(name)` after the wait (used in v7_xp). Make `act(..., still_after=)` standard. S.
- multi.py: tiling screen without a bar (open since r4); start it through waybar.sh. S.
