# kit3 r1 (F2): kit::keysheet, one key list for all four looks

Model: sonnet, effort high (a port on fixed types: KIT2 set System and bindings, KIT2b-d ported the four looks)
Lane: cloud, branch claude/kit3 from claude/kit2d 888dfb60. First `git fetch --unshallow`, then
`git merge origin/claude/kit2c` (8eabeda0, tiling port). When the session branch moves (trains 60, 61),
`git merge origin/claude/autonomous-dev-session-3w-i33pg7`. One round, stop at 250 k. Cost about 2 %.
Read reports/arch-plan-fable.md row KIT3 and backlog.md item "KIT3 keysheet" (mc keys.rs `words()`, `volume_order` go);
reports/kit2.md, reports/kit2c.md, reports/kit2d.md on claude/session-log.
1. Red first: an acts cell reads the drawn rows (`texts` of state.get) of the key list in each of the four looks and
   checks one spelling of each chord and one wording per `System` action; red on the base.
2. `chora_kit::keysheet` builds the rows from the look's bindings (`kit::actions`): words from `System`, chords in
   one spelling, look-only actions named by the look. Each look draws its own costume; only rows come from kit.
3. mc keys.rs `words()` and `volume_order`, and every look's own row table, go in the same merge.
   tools/copies.toml rows to 0. docs/api.md names the rows. No new feature, no new key, no new System variant.
Done when: cells red then green; kit unit cells; merge gate once on the head (poll it in the same turn); kit coverage
with `cargo llvm-cov -p chora-kit --features chora-kit/testing` (missing: blocker, do not install).
Refactor proposals ranked in the report.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/kit3.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
