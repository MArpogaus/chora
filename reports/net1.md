# net1 r1 (F2): kit::testing FakeCore and state.get

Branch claude/net1 on 87d8aae2 (4 commits, pushed): 515879c ratchet update, f5d9305 move,
ad478bc FakeCore refuse_next, 4ac7186 Plugin::state. tests/merge-gate.sh --jobs 3 on 4ac7186:
all steps PASS, GATE_EXIT=0 (log /tmp/claude-0/gate/net1b.log; the first run failed only in
2d-guard because the clone was shallow and the pinned base 8742dee was missing; fixed by
`git fetch --unshallow`).

## Done
- `chora_kit::testing` (feature `testing`, also under cfg(test)): `FakeCore::start()` (one socket
  per core, no global state), `connect(name)`, `model()`, `refuse_next(method)`. mc keeps its
  builder `testing::mc` in an inline `#[cfg(test)] mod testing` of main.rs; mc testing.rs gone.
  The old `refuse_calls` flag (refuse every call) became `Core::refusals` (a planted refusal is used up).
- Test `crates/chora-kit/tests/fake_core.rs` (required-features testing): a planted refusal is seen
  once; a refusal for another method is not. The selftest of the brief is the first one.
  Red proof: it did not compile before (`no method refuse_next`, seen locally); a red commit cannot
  be committed because the clippy hook refuses a non-compiling test, so test and API are one commit.
- `Plugin::state` (default none), `state.get` answered in the runtime (`answer_call`),
  `runtime::state_method()` export, `choractl state <plugin>`, docs/api.md paragraph. mc fills
  windows (scene, dock, output, shown), focus {holder, surface}, open [] and busy (write animations,
  typing) in plugins/chora-mc/src/state.rs.
- Size ratchet: --update lowered mc main.rs, tiling main.rs, xp wm.rs; `--base 87d8aae2` ok.

## Deviation
The brief says the core calls `state.get {plugin}`. I made `choractl state <plugin>` call the
exported `state.get` of the plugin through the existing `methods.call`; the core has no new method.
Joining core facts (focus, outputs, toplevels with owner) is left to NET2.

## Backlog
- size-ratchet counts lines before the first `#[cfg(test)]` followed by `mod`; a file with a
  `#[cfg(test)] mod x;` near the top (mc main.rs had one at line 32) counts only that far.
- mc `open` is always empty and windows lack title and app_id (core facts): NET2 adapter.
- The red-first rule and the commit hooks cannot hold a non-compiling red commit.
- merge-gate needs a full clone; fail early with a clear message when the 2d-guard base is missing.
