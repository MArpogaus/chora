# flakes r5 (rc 3)

Lane: opus, cloud, new branch claude/flakes-r5 from the session branch
claude/autonomous-dev-session-3w-i33pg7 (98291088 or newer). One round. Stop at 250 k context and
hand over. Read dev-rules.md and conventions.md (with the skills it names) first, then
reports/flakes-r4.md and reports/review-flakes-r4.md. Setup as briefs/flakes-r2.md. Items
(backlog), red first:
1. F19: chora-xp taskbar::tests::the_speaker_has_ink_and_the_no_sign_is_the_palette_red, "a PNG
   file: NotFound" at plugins/chora-xp/src/sheets.rs:92 (theme cache picture gone when loaded;
   1 of 4 test steps under load). Find the cause (a shared cache dir, a cleanup, a race between
   render and load); fix it, no retry that hides it.
2. F18: tests/nested.sh step 14 "timeout: not_black" at 100,100 after the lock takeover (1 of 7).
3. F16: the tiling bar does not show (train 8 gate 1, train 11 gate 1: all ten ink-1280x800 cells,
   the first config of the audit). The doors log now names the cause; reproduce with the first
   config cold, read waybar's lines; fix the start order or the wait with a reason.
4. F20: the review minors of flakes r4 (time only `call cube.open` against 300 ms; at_rest under
   heavy load; a red test on sync_cube_keys itself; the stale clippy allow wm.rs:688).
5. If context remains: F7, A5, A17 (backlog).
For each: name the earliest gate (compile time first) that would have caught it (routines.md).
Done when: tests/merge-gate.sh passes three times in a row under load (say the load) and once
after a cold start. Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 20 lines) and as
reports/flakes-r5.md on claude/session-log. Rules as briefs/flakes-r2.md, with push only
claude/flakes-r5; never end a turn to wait for a long background job.
