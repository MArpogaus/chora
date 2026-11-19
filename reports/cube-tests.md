# Cube tests

## r1

Lane: cube-tests (F5, brief briefs/cube-tests-r1.md), branch claude/cube-tests,
model haiku, effort high. Tests only; no production code changed.

Head: `b4d6321e` on claude/cube-tests (one commit on f2b7112a, tree 0d25189e).
The merge gate ran on `9b6e9e60`, the same tree, before the committer-only
amend to `b4d6321e`: `tests/merge-gate.sh --jobs 3`, all steps PASS, GATE_EXIT=0,
nothing pushed by the gate.

Coverage of `plugins/chora-construct/src/cube/construct.rs` (lines, llvm-cov):

| state | lines | missed | cover |
|---|---|---|---|
| before (f2b7112a) | 421 | 259 | 38.48 % |
| after (b4d6321e) | 421 | 14 | 96.67 % |

The brief gave 41.81 % for before; the measured figure at f2b7112a is 38.48 %
(same 421 lines). The target of 70 % is met. The 14 missed lines were not
listed one by one.

Commands:
- before: `env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-construct --features chora-kit/testing --summary-only`
- after: the same command
- `env -u XDG_RUNTIME_DIR cargo test -p chora-kit --features chora-kit/testing --test cube`: 4 passed
- `cargo clippy --workspace --all-targets -- -D warnings` (run by the pre-commit hook): passed

Tests added:
- `plugins/chora-construct/src/cube/tests_construct.rs` (13 cells, wired by
  `#[cfg(test)] mod tests_construct;` in cube.rs): open (face count, refusals,
  outputs and nodes, refused nodes), close, check (exit and late connect),
  retry, connect and disconnect, turn (step, ignored, queued), the edge (lit,
  fade, hidden while turning), mirrors, face and edge nodes, outputs.
- `crates/chora-kit/tests/cube.rs` (4 cells): exporter found, exporter none,
  listing (faces, front, mode, request), refused turn. Needs a `[[test]]` entry
  with `required-features = ["testing"]` in crates/chora-kit/Cargo.toml, added.

Not reached in unit tests: the refusal when the manager process cannot be
spawned (`launch` in open_cube and restart); a unit test cannot make `sh` or the
log file fail. Named here, not a hidden gap.

Not done: the kit cells cover the cube IO through the fake core; no kit cell
for the wm parts (already modelled, see F5 in reports/audit-f.md r2).
