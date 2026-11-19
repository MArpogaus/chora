# kit2b r1 (F2): port construct to kit::actions

Model: sonnet, effort high (a port; KIT2 settled the types on mc)
Lane: cloud, branch claude/kit2b from the session branch (27bd60c7, train 54). One round, stop at 250 k.
Cost about 1.5 %. Run `git fetch --unshallow` first.
Read reports/arch-plan-fable.md slice rows KIT2, KIT2b-d and reports/kit2.md (refactor proposals 1, 2) on
claude/session-log.
1. construct's bindings load into `kit::actions` with `settings::Common`; its own `Action` enum keeps only
   construct-only actions as the look part of `Action<L>`; the system part is `System`.
2. Old spellings `volume_up`, `volume_down`, `mute`, `help` load through serde aliases on `System` for one
   release (kit2 proposal 1), with a cell each.
3. With a second user, move performing `System` into `kit::actions::perform` (kit2 proposal 2); mc and
   construct both call it; mc acts.rs loses its match.
4. Cells red first: construct configs of 29e2cd77 load to the same table; the cross-look test (one YAML
   binding gives one `System` action in mc and construct) becomes real.
5. Visible behaviour unchanged (door acts, cube, pull, construct-hud; known-red only shrinks). No new
   feature; no change to tiling or xp beyond compile fixes.
Done when: cells red then green; merge gate once on the head (poll it in the same turn); `kit::actions`
coverage with `cargo llvm-cov -p chora-kit --features chora-kit/testing` (setup.sh installs it; if it is
missing, name it a blocker, do not install). Refactor proposals ranked in the report.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/kit2b.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
