# xp polish r2 (rc 1): editor, volume feedback, question card

Model: sonnet, effort medium (three bounded UX fixes in one look)
Lane: cloud, branch claude/xp-polish (r1 head 1df6d8ad, MERGE-OK); merge the session branch first. One round.
Stop at 250 k context. Read findings/xp.md (xp-25..27), reports/journey-1.md (X1-X3),
reports/review-xp-polish-r1.md, design/looks/xp.md and xp-v2.md, dev-rules.md.
Licensing (hard): never a Microsoft file; xp on fallback art.

1. xp-25 MAJOR: the xp editor launcher opens a GUI editor when one is installed (Writer,
   mousepad, gedit, in that order), nano in a terminal only as fallback. Red cell first.
2. xp-26 MAJOR: after setting the volume (Control Panel or tray), the tray shows the level and
   the menu ticks the current level. Red cell first, pixel assert at 1x.
3. xp-27 MAJOR: the question card uses the xp font and balloon style, does not cover the
   transcript, offers Yes/No buttons for the mouse (keys as secondary hint), says
   "Typing does not answer" at most once. Red cell first, capture at 1x and 2x.
4. Evidence from r1 review (no new work): captures of the Run box and of the no-helper
   idle paperclip, named with look, build and date.
No MINOR work (rules.md 6). Done when: the cells pass, tests/merge-gate.sh once on the head.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and as reports/xp-polish-r2.md on claude/session-log. Rules as briefs/flakes-r2.md, push only
claude/xp-polish and that report; never end a turn to wait for a long background job.
