# kit2c r1 (F2): port tiling to kit::actions; kit6 rw-1, rw-2

Model: sonnet, effort high (a port on fixed types; KIT2 set them, KIT2b did construct)
Lane: cloud, branch claude/kit2c from 1d1aa5fd (train 58 candidate: kit2b, wm1c). First `git fetch --unshallow`,
then `git merge origin/claude/kit6` (ad88ebaf, MERGE-OK, train 59). When the session branch moves (trains 58, 59),
`git merge origin/claude/autonomous-dev-session-3w-i33pg7`. One round, stop at 250 k. Cost about 2 %.
Read reports/arch-plan-fable.md rows KIT2, KIT2b-d; reports/kit2.md, reports/kit2b.md (Review r1 and r2, refactor
list), reports/kit6.md "## Review r1" on claude/session-log.
1. tiling's bindings run through `kit::actions` with `settings::Common` as a `Host`; tiling's own action table and
   parser go in the same merge. Configs of 29e2cd77 load to the same table (a cell with today's default and a
   user file).
2. The cross-look test of kit2b gains tiling. Door acts and tiling gate steps unchanged; known-red only shrinks.
3. kit6 rw-1, red cell first: the launch wrapper runs the command in a `( )` subshell, so `exit N` and a failed
   `exec` write the status; put `exit 1` back in the red cells where kit6 weakened them to `false`.
4. kit6 rw-2, red cell first: when the start cannot be followed (pidfd_open fails, poll EINTR, unit active with
   pid null), poll alive(pid) or the unit state each second instead of counting it as running.
5. kit6 rw-3 NIT: status file prefix chora-launch-exit-. Then update D104 in docs/decisions.md: the open point
   (rw-1) is closed by this branch.
6. tools/copies.toml: tiling action rows to 0. No new feature; no xp `bindings` (KIT2d).
Done when: cells red then green; merge gate once on the head (poll it in the same turn); kit coverage of
`actions` and `launch` with `cargo llvm-cov -p chora-kit --features chora-kit/testing` (missing: blocker, do not
install). Refactor proposals ranked in the report.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/kit2c.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
