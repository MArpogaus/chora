# xp polish r1 (rc 1): the paperclip back, text fields aligned

Model: sonnet, effort medium (one cause hunt, one bounded pixel round)
Lane: cloud, branch claude/xp-polish (session branch 66588880); merge the session branch first. One round.
Stop at 250 k context. Read findings/xp.md (xp-23, xp-24), design/looks/xp.md and xp-v2.md, dev-rules.md.
Owner 11-16 09:01: "Xp: why is clippy gone? Bring it back. Textfields poorly aligned we need an
other ux/ui polish there." Licensing (hard): our own paperclip art only, never Clippit or any
Microsoft file; xp on fallback art.

1. xp-23 MAJOR: the paperclip (xp.md: the one helper door) did not show in tour v3. Find why
   (config, helper state, hud, art path, regression; git log -S), red cell first
   (tests/gate-*.sh: the paperclip is at its xp.md place on a fresh xp session, with and without
   a helper), then the fix. A missing helper shows the paperclip in its honest "no helper" pose.
2. xp-24 MAJOR: text fields (Run box, search, rename, dialogs, chat input): label baseline,
   field height, inner padding and border on the XP grid; one pixel assert per field kind,
   captures at 1x and 2x in the report.
Done when: both cells pass, tests/merge-gate.sh once on the head. Report to the PM session
session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines) and as reports/xp-polish-r1.md
on claude/session-log, captures named with look, build and date. Rules as briefs/flakes-r2.md,
push only claude/xp-polish and that report; never end a turn to wait for a long background job.
