# ux r5 (rc 1): mc-31 (verify, fix if real), sh-19, sh-20

Lane: new opus worker, new branch claude/ux-r5 from the session branch
claude/autonomous-dev-session-3w-i33pg7 (0e303d60 or newer; ux r4 is merged). Worktree
/home/user/wt/ux-r5. One round, small and fixed. Read dev-rules.md, conventions.md, then
findings/mc.md (mc-31), findings/shared.md (sh-19, sh-20), reports/review-ux-r4.md.
Stop at 250 k context and hand over. ti-1 is out of scope (waits for the owner).
Items, in this order (rules.md 6: severity first), a red unit or gate cell first for each:
1. mc-31: tests/battle/w16.sh step 1 reports that a voice confirm in mc shows no No button and that
   every later voice turn lags one step behind. Reproduce on the newest session branch. If real:
   rate it (MAJOR if the person cannot say or click No, or the answers go to the wrong question),
   find the cause, red cell, fix. If not real: say why (the battle script's own step, a timing),
   fix the script if it is wrong, and give mc-31 its state with file:line.
2. sh-19: the chat header always reserves two lines, so Stop, Close and the transcript do not move
   when a run starts or the step count appears (all looks; one transcript row less is accepted).
   Cell: button rects equal with no task, a short task and a 1800-character task.
3. sh-20: `capped` (view.rs:561) stops wrapping after most+1 lines; unit: a 20,000-character paste
   without spaces renders the header in under 5 ms (release) or a bound that shows the cut.
Keep KISS: no new options. Before handback merge origin/claude/autonomous-dev-session-3w-i33pg7 again
(a harness train lands meanwhile; its hooks apply at once).
Output: the full tests/merge-gate.sh once (serial, background job with a 2 h limit), green;
tests/ci-local.sh once; reports/ux-r5.md on claude/session-log (item, red commit, fix commit, cell,
1x captures). Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines).
Rules: push only claude/ux-r5 and claude/session-log; author Marcel Arpogaus
<38564291+MArpogaus@users.noreply.github.com>; no AI attribution lines; never --no-verify; never end a
turn to wait for a long background job; CARGO_TARGET_DIR inside your worktree only; install nothing.
