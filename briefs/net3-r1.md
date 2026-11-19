# net3 r1 (F7, F5 prep): core scene.boxes and text runs

Model: opus, effort high, because (a new core read API and text runs out of chora-draw; the core stays lean
and safe, and wrong boxes would make every later invariant lie)
Lane: cloud, branch claude/net3 from dc4bc46d (the train 56 head: mv3 split scene.rs). One round,
stop at 250 k. Cost about 4.5 %. Run `git fetch --unshallow` first.
Read reports/arch-plan-fable.md 3.2 to 3.4 and slice row NET3, reports/net2-harness.md and
reports/net2-adapters.md on claude/session-log.
1. Core `scene.boxes` (new chora-scene boxes.rs, a read-only IPC call): per output the shown nodes with
   rect, layer, opacity flag and owner; cube faces with projected corners. No policy, no new write path.
2. Text runs from chora-draw into `texts` (node, rect, font size, string), reported, not re-measured.
3. Invariants in tests/acts: I2, I5, I6, I9, I10, I12, I13, each with a cell that breaks it on purpose
   (selftest). I13 is red on today's seams: list it in the known-red file with co-21; the same for any
   other look that fails today. A listed pass fails the gate.
4. net2-harness reads `scene.query` rects; switch those reads to `scene.boxes`.
5. Visible behaviour unchanged; no change to looks beyond compile fixes.
Done when: core unit tests for boxes and runs (a font check), the selftests fail as planted, merge gate
once on the head (poll it in the same turn), known-red rows each name a finding. Refactor proposals ranked.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/net3.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
