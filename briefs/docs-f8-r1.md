# docs-f8 r1 (F8): decision records and contributor docs for the shared-core structure

Model: sonnet, effort medium
Lane: cloud, branch claude/docs-f8 from the session branch (27bd60c7, train 54: kit1 in). One round,
stop at 250 k. Cost about 1.5 %. Run `git fetch --unshallow` first. Docs only, no code change.
Read on claude/session-log: design/foundation.md (F8 row), reports/arch-plan-fable.md, state.md Queue 5
(records 1-14, D87 revised, D92 kept), reports/net1.md, net2-*.md, kit1.md, kit2.md, kit4.md, wm1.md.
1. docs/decisions.md: one record per structural decision that landed (FakeCore and kit::testing, acts harness
   and invariants, kit::actions, kit::wm book, kit::ask, chora_scene::paths, kit::words Language, size ratchet
   and cut gate, D87 revised, D92 kept). Same format as D78-D95; status "accepted" only where merged.
2. CONTRIBUTING.md and the dev rules doc: where shared code lives (chora-scene, chora-kit modules), the rule
   "a look uses kit, no copy" with tools/copies.toml and the cut gate, how to add a copies row.
3. Every claim names the file or commit on the session branch; no plan text written as done.
Done when: lychee and vale pass (pre-commit); merge gate once on the head (poll it in the same turn).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/docs-f8.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
