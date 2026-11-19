# mv3 r1 (F4): moves without logic in core and chora-scene

Model: sonnet, effort medium (moves only, reviewed with --color-moved)
Lane: cloud, branch claude/mv3 from the session branch (27bd60c7, train 54). One round, stop at 250 k.
Cost about 1.5 %. Run `git fetch --unshallow` first.
Read reports/arch-plan-fable.md slice row MV2 and the F4 lines of 4.2, and reports/mv2*.md on
claude/session-log.
1. Split, as MV2 did, `impl` blocks and free functions into child modules, no logic change: chora-scene
   scene.rs (2,033), core render.rs (1,858), input.rs (1,729), ipc.rs (1,720). Each file under 1,500
   production lines counted with the command of reports/debt-rc.md; a test module moved out is not a drop.
2. Ratchet (MV1): lower the listed caps for these files to their new counts (`--update` in the same merge).
3. `git diff --color-moved=dimmed-zebra` shows only moves plus `mod`/`use`/`pub(crate)` lines; list any
   other line in the report with its reason.
4. No new feature; no behaviour change; do not touch mc, construct, xp, tiling or voice.
Done when: the four files under 1,500, ratchet lowered, merge gate once on the head (poll it in the same
turn), report names each new file and its line count. Refactor proposals ranked in the report.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/mv3.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
