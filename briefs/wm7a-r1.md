# wm7a r1 (F3): cube role API and the hosted xp session measured

Model: opus, effort high, because it fixes a cross-look API (cube role) and measures a design choice the owner
takes next (1.2); a wrong API costs every later cube slice
Lane: cloud, branch claude/wm7a from a17e7858 (train 57 head). First `git fetch --unshallow`; merge
origin/claude/autonomous-dev-session-3w-i33pg7 when it moves. One round, stop at 250 k. Cost about 3 %.
Read reports/arch-plan-fable.md section 1.2 and row WM7a on claude/session-log, design/3d.md.
1. Cube role API in core IPC and `kit::cube`: `cube.list`, `cube.turn {to}`, a turn event, `cube.pull`.
   One cube code; construct hosts it. Until the owner's choice `xp/cube.rs` stays (WM7b decides).
2. Acts red first: turn by key, by pager, by drag; back at 1:1 with no construct surface left; construct killed
   mid-turn keeps I1 (no lost window).
3. Measure the hosted xp session (numbers, commands, captures named by build): frame cost at rest; keys with the
   pointer outside the claim; the turn grab (does the smallest-claim rule ipc.rs:441-460 give construct the
   press? if not, name the core slice "grab hand-over without a window", do not build it); construct killed
   mid-turn; the return to 1:1. End with a recommendation hosted or flat, for the owner.
4. Visible behaviour unchanged; no new feature; no change to tiling or mc beyond compile fixes.
Done when: acts red then green; merge gate once on the head (poll it in the same turn); measurements table.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/wm7a.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
