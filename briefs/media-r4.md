# media r4 (rc 11, 1, 1b): tour v6 on the four fixed lanes, UX loop round 3

Model: sonnet, effort medium (scripted capture work; no product code)
Lane: cloud, branch claude/media (merge the session branch aa8aa8c0 first). One round. Stop at 250 k.
Read briefs/media-r3.md (pace rules stay), reports/media-r3.md, reports/review-tour-v5.md
(the 7 MAJOR this tour must show fixed), design/tour-v4.md, vision-card.md.

1. Re-record the tour v5 scenarios on aa8aa8c0 as tour v6 (fixes: mc r6 train 34, construct r2
   and tiling r3 train 35, xp r6 train 36). Change scripts only where a fix changed the path.
2. Harness: start the tiling bar through packaging/tiling waybar.sh (as a session does), not
   waybar directly (debt-rc.md); set CHORA_TEST_KEEP_BUS=1 so the bus outlives the chapter.
3. For each v5 MAJOR (rv5-1, j1-5, j1-4, j1-2/N1, rv5-2, rv5-3, N2) and the v5 MINOR rows: the
   time in v6 that shows it fixed, or "still open" with a still. Check every look (xp, tiling,
   mc light and dark, construct, handover). New findings: time, severity, text; no product code.
Output: mp4 per chapter and tour-all.mp4 in media/<date>/tour-v6/, timeline.md.
Done when: tour-all.mp4 under 6 min, the v5 table filled. Report to the PM session
session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines) and as reports/media-r4.md
on claude/session-log, ending with Refactor proposals (harness). Push only claude/media and
the log files named here.
