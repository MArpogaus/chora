# net1 r1 (F2): kit::testing FakeCore and state.get

Model: sonnet, effort medium (move plus a small API; mc tests are the check)
Lane: cloud, branch claude/net1 from the session branch head. One round. Stop at 250 k. Cost about 2 %.
Read reports/arch-plan-fable.md (3.1 state shape, 4: NET1), design/foundation.md (F2), dev-rules.md.

1. Red first: a kit test that uses `chora_kit::testing::FakeCore` with `refuse_next` (fails to compile before).
2. Move plugins/chora-mc/src/testing.rs into `chora_kit::testing` (feature `testing`); mc tests use it;
   mc testing.rs deleted in the same merge. Moves first, one commit, then the API commits.
3. `Plugin::state` (default: none) and the core call `state.get {plugin}`; `choractl state <plugin>` prints it.
   mc fills the 3.1 shape it can (windows, focus, open); the other looks come with NET2 adapters.
4. Size ratchet: no listed file grows unless the merge extracts from it (tools/size-ratchet).
Done when: mc tests pass on FakeCore, a FakeCore selftest (planted refusal is seen), cargo test and
clippy pass, tests/merge-gate.sh once on the head (poll it in the same turn).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and as reports/net1.md on claude/session-log. Suggest refactors in the report (backlog).
Rules as briefs/flakes-r2.md, push only claude/net1 and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
