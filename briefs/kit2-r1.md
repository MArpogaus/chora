# kit2 r1 (F2): kit::actions with settings::Common; pilot mc

Model: sonnet, effort high
Lane: cloud, branch claude/kit2 from the session branch (de057ab3, train 50). One round, stop at 250 k.
Cost about 1.5 %. Run `git fetch --unshallow` first.
Read reports/arch-plan-fable.md 2.1, 2.2 (last paragraph), slice row KIT2, and ARC section 2 it
names (reports/architecture-rc.md) on claude/session-log.
1. chora-kit `actions` (the `System` enum, YAML binding to action) and `settings::Common` (C10).
   `System` gains `Desk(Step)` and `Cube(Turn)` only as values; no look acts on them yet.
2. Pilot mc: its bindings and common settings load through kit; its own copy goes.
3. Cells red first: one YAML binding gives one `System` action; the mc configs of 29e2cd77
   (configs/ and docs examples) load to the same table as before (golden test).
4. Visible behaviour unchanged (door acts, NET2 acts; known-red only shrinks). No change to
   tiling, construct, xp beyond compile fixes. No new feature.
Done when: cells red then green, merge gate once on the head (poll it in the same turn).
Refactor proposals ranked in the report.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/kit2.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
