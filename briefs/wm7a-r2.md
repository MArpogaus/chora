# wm7a r2 (F3): fix the review r1 findings of the hosted-xp cube measure

Model: sonnet, effort high (named fixes in acts, docs and one refusal; the design is set)
Lane: cloud, branch claude/wm7a at 4069e38f (continue it). One round, stop at 200 k. Cost about 1 %.
Run `git fetch --unshallow` first, then `git merge origin/claude/autonomous-dev-session-3w-i33pg7` (1d1aa5fd or
later). Read reports/wm7a.md "## Review r1" on claude/session-log; ids below are its ids.
1. rw-1 MAJOR, red commit first: in tests/acts/looks/construct.py the kill act drops the `cube.turn {to: 0}` cue
   (or waits for `turning: false`), asserts the `cube.turn {by: 1, ms: 3000}` answer is true and `cube.list`
   says `turning: true` right before the SIGKILL. If the corrected act is red, fix construct so I1 holds
   mid-turn (window given back, no loss) in the next commit; name the cause in the report.
2. rw-5: start the step-back act with `cube.turn {to: 0}` so no act depends on the front the known-red pager leaves.
3. rw-2: list.rs `pull` returns `crate::modes::no_cube()`.
4. rw-3: one act: pull the probe window, remove the pane, expect owner xp again (red first if it fails).
5. rw-4: docs/api.md and docs/configuration.md name the `{pane}` answer of `cube.pull` and the full stop.
6. rw-6: report section 3 names the two key lines of `turn_key_to_window = 2`, or marks the row open; state
   the 1:1 act's tolerance (rw-7 text part).
rw-7 (ownerless window without restart) and rw-8 (no event on close) are filed by the PM as wm7a-m2, wm7a-m3
for WM7b; do not fix them here.
Done when: `cargo test -p chora-construct`, `-p chora-kit --features testing` green;
`tests/merge-gate.sh --only acts,cube,pull,construct-hud,placed` green (poll in the same turn). No full gate.
Push claude/wm7a. Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append "## r2" to reports/wm7a.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
