# media r6 (rc 11, 1b): tour v8 now, the videos kept

Model: sonnet, effort medium (scripted capture work and harness fixes; no product code)
Lane: cloud, branch claude/media; merge the session branch (head after train 40) first. One round. Stop at 250 k.
Read briefs/media-r5.md, reports/media-r5.md, reports/review-tour-v7.md (F-1, N-1, harness rows),
media/2026-11-17/tour-v7/timeline.md, design/tour-v4.md, vision-card.md.

1. Harness fixes first (tests and scripts only): launch.py `path_without_writer` hides the writers
   from every PATH entry (/bin too), so the Notepad "no editor installed" notice shows (F-1, j1-2/N1);
   the construct drag aims at the pane's measured rectangle (N-1); a plain-drag (no Super) gate cell
   for the drag back onto a face.
2. Re-record the v7 scenarios on the new head as tour v8, plus: xp desktop cube turned with
   Ctrl+Alt+Right and with Super+drag (owner 11-17 asks if it still works); the xp helper opened by
   a click on the paperclip and by the tray icon; construct stepped back at 0, 45 and 90 degrees (the
   seams, co-21; construct r4 runs in parallel: record what this build shows).
3. Keep the videos (owner 11-17 10:35 "save it this time"): commit the mp4s (chapters and
   tour-all.mp4, each under 25 MB) and stills to a new branch claude/media-v8 that holds only
   media/2026-11-17/tour-v8/; push it before you report. Do not copy them anywhere else.
4. timeline.md as in v7: each v7 row fixed or still open with time and still; new findings with severity.
Done when: tour-all.mp4 under 6 min is pushed on claude/media-v8. Report to the PM session
session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines) and as reports/media-r6.md on
claude/session-log, ending with Refactor proposals (harness). Push only claude/media, claude/media-v8 and
the log files named here.
