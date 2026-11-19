# kit2d r1 (F2): port xp to kit::actions

Model: sonnet, effort high (a port on fixed types; KIT2 set them, KIT2b did construct, KIT2c does tiling)
Lane: cloud, branch claude/kit2d from 1d1aa5fd. First `git fetch --unshallow`, then `git merge origin/claude/wm1d`
(18231aa1, MERGE-OK, train 60: xp window book). When the session branch moves (trains 59, 60),
`git merge origin/claude/autonomous-dev-session-3w-i33pg7`. One round, stop at 250 k. Cost about 2 %.
Read reports/arch-plan-fable.md rows KIT2, KIT2b-d; reports/kit2.md (refactor list 6), reports/kit2b.md (Review r1
and r2) on claude/session-log.
1. xp gains `bindings` through `kit::actions` with `settings::Common` as a `Host`; its defaults equal today's
   hard-coded keys exactly (a cell lists every key xp handles on 1d1aa5fd and checks the same System action for
   each, red first against an empty table). xp's own hard-coded key match goes in the same merge.
2. The cross-look test of kit2b gains xp. Door acts and xp-* gate steps unchanged; known-red only shrinks.
3. Keys that are not a `System` action (xp-only: Start menu, Alt+Tab order, desktop moves) stay xp actions on the
   look side of `Action<L>`; no new System variant without a second look that needs it.
4. tools/copies.toml: xp action rows to 0 (or a new row at 0). docs/api.md and the xp config docs name the
   `bindings` key and its defaults. No new feature beyond configurable bindings; no keysheet (KIT3).
Done when: cells red then green; merge gate once on the head (poll it in the same turn); kit coverage of
`actions` with `cargo llvm-cov -p chora-kit --features chora-kit/testing` (missing: blocker, do not install).
Refactor proposals ranked in the report.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/kit2d.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
