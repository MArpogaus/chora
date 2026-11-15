# ux r2 (rc 1): close the review of ux r1

Lane: new cloud worker (Fable; the one new Fable lane after the reset, owner 11-15 01:22), branch
claude/ux-r1 (continue it; worktree /home/user/wt/ux-r2). One round, fixed. Read dev-rules.md and
conventions.md (with the skills it names) first, then reports/review-ux-r1.md and reports/ux-r1.md
(handover: the held draft is `Session::held` in plugins/chora-assistant/src/chat.rs, set in said(),
sent in show(), reset only in open_list()). Stop at 250 k context and hand over.
Start: merge origin/claude/autonomous-dev-session-3w-i33pg7 into claude/ux-r1 once it holds train 11
(63ab1724 tiling-audit fix; `git merge-base --is-ancestor 63ab1724 origin/claude/autonomous-dev-session-3w-i33pg7`).
Until it does, work on the items and fetch again before the gate.
Items, red first (a unit test or a gate cell commit before each fix):
1. M1 (MAJOR): a held draft must never reach a helper the person did not pick. Make it impossible
   by type: the held text lives inside the list state (e.g. `Mode::List(picks, Option<String>)`),
   so hide, close_view, Esc and the only-helper path drop it. Red: the reviewer's scenario
   (clicked_row, Say, hide, show(Some(..))) as a chat.rs unit test with the fake Link: 0 sends.
   Also: a question balloon (voice.rs) and chat.show RPC never send a held draft.
2. m2: after a held text only a click or a number picks; a helper name is text, not a pick. A
   number out of range says so in the list (no silent drop).
3. m3: while all keys show, only the column region gives way; Left and Drawer keep their content
   (or the docs say all side regions; pick the KISS one and say why).
4. m6: the mc header check also asserts the date is whole (not cut) at 1280x800 and 1024x768, and
   fails on any overlap frame after the layout settles (no retry that hides a one-frame overlap).
5. Cosmetic, only if cheap: a unit test for give::keys_row.
Accepted as reported (no work): m1 list shows with a project to start; m4 mc wording; m5 process.
Done when: tests/merge-gate.sh passes as one run on the merged branch (a known flake F16/F17/F18
fails: rerun once, name it in the report). Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN
(at most 20 lines) and as reports/ux-r2.md on claude/session-log.
Rules: as briefs/ux-r1.md (author, no attribution lines, never --no-verify, stage by name, no force
push, push only claude/ux-r1 and the report, no Microsoft files, never end a turn to wait for a
long background job).
