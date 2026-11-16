# media r2 (rc 11, 1b): tour v4, one usage scenario per look, short

Model: sonnet, effort medium (scripted capture work; no product code)
Lane: cloud, branch claude/media (merge the session branch first). One round. Stop at 250 k.
Read briefs/media-r1.md (rules, licensing), reports/media-r1.md, demo/tour/, vision-card.md.
Owner 11-16 09:01: "Screen cast way to slow and repetitively. Design special usages scenarios
for every look and make brakes shorter."

1. Write design/tour-v4.md: one real usage scenario per look, each for its users (vision card),
   each showing what only that look does; no step repeated across chapters (doors were shown in
   every chapter of v3: show each shared door once). Seeds, change them if better:
   xp: Erika writes a letter in the editor, asks the paperclip how to print, finds it in Start;
   tiling: a hacker builds a project in tiled terminals, floats a window (wobble), screenshots
   the rice; mc: hands-busy kitchen, recipe by voice and remote, timer card, page turn;
   construct: a demo watcher steps out, turns the cube, pulls a window from the xp face to tiling.
   Plus one handover chapter (a window through all four looks, 30 s).
2. Pace: pauses at most 0.6 s between acts, typing about 12 cps, no idle shot over 1.5 s; target
   60 to 75 s per look, about 5 min in all. Cuts instead of waits.
3. Scripts in demo/tour/ (v4), recorded on the session branch head; mp4 per chapter and
   tour-all.mp4 in media/<date>/tour-v4/, timeline.md. Findings seen on the way go into the
   report as in r1 (time, severity, text); do not fix product code.
Done when: tour-all.mp4 under 6 min, each chapter its own scenario. Report to the PM session
session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines) and as reports/media-r2.md
on claude/session-log. Push only claude/media and the log files named here.
