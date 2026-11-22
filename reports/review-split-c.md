# PM review split-c r1 (11:30 UTC, 3b14c825..96e5c804, 5 files +865 -807, 2 commits)

Pure-move refactor of the core; check is mechanical (procedures: PM review for small diffs; the PM is not the builder).
- `git diff --color-moved=zebra` 3b14c825...96e5c804 -- crates: every non-moved line is a mod line, a use line, a module doc line, `impl State {`/`}` wrappers, a test module header, `pub(super)` on ipc_subscribe/ipc_emit (rustfmt rewrap of one signature), and the mesh re-exports `pub(crate) use passes::run_pass; pub use passes::{Source, apply_passes, passes}` (keep the old paths). No body changed.
- Counts: ipc.rs 1401 to 1149, mesh.rs 1390 to 1068; size, lint and cut-check ratchets not grown (lane gate 49/49 PASS, --jobs 1, full).
- Verdict (computed): 0 BLOCK, 0 MAJOR: MERGE-OK; 96e5c804 joins t109.
