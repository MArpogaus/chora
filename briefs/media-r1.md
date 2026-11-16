# media r1 (rc 11, 1b, 1): new pictures, README screenshots, the tour rewritten and recorded

Model: sonnet, effort medium (scripted capture work; no product code)
Lane: cloud, branch claude/media from the session branch after train 22 (merge claude/tour
48fc05b5 first). One round. Stop at 250 k context. Owner 11-15 09:20/09:27: "prune and update
screenshots and screencast"; status page pictures, screencast rewrite, README screenshots.
Read dev-rules.md, conventions.md, design/rc.md rows 1b and 11, reports/tour-handover.md,
demo/tour/README.md, vision-card.md (the four looks) and design/looks/*.

Licensing (hard): xp only on our fallback art; no Microsoft file is ever captured from an import,
committed or uploaded. Every picture names the look, the build (short sha) and the date.
1. Status pictures: one keeper per look (xp, tiling, mc light and dark, construct, construct with
   xp on the faces) plus the helper chat and the key list, 1280x720 PNG, at most 400 KB each,
   on claude/session-log under status/img/ with a list (status/img/README.md: file, look, build,
   what it shows). List the old files that are now stale; delete nothing yourself.
2. README screenshots: four PNGs (one per look, at most 300 KB) under docs/img/ on claude/media,
   and a short "Looks" section in README.md showing them. Plain words, no overstatement.
3. Tour v3: rewrite the chapter scripts after the handover's "What makes the current script
   artificial": real programs (foot, a text file, a browser if present), a person's pointer
   paths (overshoot, pauses), typing with pauses and one corrected typo, waits for animations,
   one slip and its recovery per look, discovery through the visible doors (launcher, key list,
   helper) rather than method calls, window drags between looks. Every look, its transitions.
   Record on the build of step 1 (software rendering is fine); output media/2026-11-16/tour-v3/
   on claude/session-log (WebM or MP4, at most 60 MB total) with a timeline (time, chapter,
   what the person does, what should be seen). The tour scripts stay on claude/media.
Not in scope: fixing looks; findings you see go into the report with time and severity.
Done when: steps 1-3, tests/merge-gate.sh once on claude/media. Report to the PM session
session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 10 lines) and as reports/media-r1.md on
claude/session-log. Rules as briefs/flakes-r2.md; push only claude/media and the session-log files
named here; never end a turn to wait for a long background job.
