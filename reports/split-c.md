# split-c

## r1

F4 headroom, compositor core: two pure moves, one commit per file. Branch
`claude/split-c`, head 96e5c804 (base 3b14c825, equal to
origin/claude/autonomous-dev-session-3w-i33pg7, so the merge is a no-op).

| file | commit | production lines before | after |
| --- | --- | --- | --- |
| crates/chora/src/ipc.rs | 932fad2c | 1,401 | 1,149 |
| crates/chora/src/mesh.rs | 96e5c804 | 1,390 | 1,068 |

### Moved groups

ipc.rs (to the new modules `ipc/emit.rs`, `ipc/closing.rs`; both with
`mod` lines in ipc.rs):

- events group, from ipc.rs lines 72-79 and 82 (`MAX_EMIT`, `MAX_EMITS`,
  `LOOK_EVENTS`, `EMIT_WINDOW`), 732-767 (`core_namespace`, `subscribable`,
  `stamped_look`) and 1079-1156 (`State::ipc_subscribe`, `State::ipc_emit`),
  to `ipc/emit.rs` (143 production lines). Their tests, 2504-2725
  (`subscribe_wants_known_event_names` to
  `a_subscription_names_its_events_or_takes_all`, 222 lines), moved with them.
  `ipc_subscribe` and `ipc_emit` are `pub(super)`; the rest stays private.
- close group, from ipc.rs lines 1158-1286 (`drop_nodes`, `end_graces`,
  `ipc_end`, `ipc_close`, `ipc_flush`, `ipc_write`), to `ipc/closing.rs`
  (141 production lines). `GRACE` and `MAX_OUTPUT` stay in ipc.rs and are
  imported with `use super::`; the tests of the grace time stay in ipc.rs.
  Visibility is as before (`ipc_write` stays private).

mesh.rs (to the new module `mesh/passes.rs`):

- shader passes group, from mesh.rs lines 1065-1389 (`Pass`, `passes`,
  `Source`, `apply_passes`, `run_passes`, `hash_of`, `signature`, `Compiled`,
  `program_of`, `PassInputs`, `run_pass`, `draw_pass`), to `mesh/passes.rs`
  (341 production lines). `float_size` stays in mesh.rs (the mesh element
  also uses it). The paths `mesh::passes`, `mesh::apply_passes`,
  `mesh::Source` and `mesh::run_pass` stay by re-export in mesh.rs
  (`pub use`, and `pub(crate) use` for `run_pass`). No test of mesh.rs
  touches this group.

### Proof of a pure move

The sorted line multiset (leading blanks cut) of the old file against the
sum of the new files differs only by: the new `mod` and `use` lines,
the module doc line, the `impl State {`, `}`, `#[cfg(test)]`, `mod tests {`
wrappers, the rustfmt wrap of two moved signatures (`ipc_subscribe`) after
`pub(super)` was added, and the `use` lists. The stray first two lines of the
doc comment above `core_namespace` ("The line of an event notification
with the sequence number...") belong to `stamped`; they stay in ipc.rs on
`stamped`, so no comment is left on the wrong item. No body changed.

### Other checks

- `cargo clippy --workspace --tests`: no warning; `cargo test -p chora`
  green (ipc:: 152 tests, mesh:: 4 tests, then the full gate `test` step).
- `tools/size-ratchet`: ok, 0 files over 1500, list unchanged (it was empty).
- `tools/lint-ratchet`: ok (allow 360, no-reason 0, eprintln 131), the
  counts did not move. `tools/cut-check`: ok (8 crates, 18 reported); the
  two "lower max" reports are old and not from this change.
- The commit hook refused my first message (subject over 68 characters);
  the commit was made again with a shorter one. No `--no-verify`.

### Gate

`tests/merge-gate.sh --jobs 1` (no --only) on claude/split-c at 96e5c804, merged
with origin/claude/autonomous-dev-session-3w-i33pg7 (already contained, so
no merge commit): all 49 steps PASS (red-first, identity, pre-commit,
gate-lib, ci-check, build, test, clippy, 2d-guard, package-deps,
package-lists, xp through hud, layout 1347 s, tiling-audit, tiling-look,
tiling-empty, chat-keys, doors, acts 1044 s, last-words, cube, pull, placed,
construct-hud, construct-extra, out, winit, wm-pane, pane-keys, walk, keys,
pixels, mc-paths, mc-sofa, mc-column, mc-handover, launcher-click, voice-acp,
voice-events, agent, e2e). No SKIP, no FAIL, no red cell, no rerun. Final line:
"all steps passed; no --push, nothing is pushed". Logs in
/tmp/merge-gate.wo7kPY of the lane container.
