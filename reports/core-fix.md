
## r1

Head: claude/core-fix 4fedd240 (on 46e98b3e). Gate merge with origin/claude/autonomous-dev-session-3w-i33pg7: a7d8ed65.

- mch-1 red: `a_resized_layer_on_two_outputs_of_the_same_scale_drops_old_sizes` (two 160x120 outputs at scale 1, radius-4 rect across the seam, 10 resizes): `old sizes stay: [(40, 40), (41, 41), ... (50, 50)]`, 11 sizes.
  Green after the fix (`Layers::slot` runs `drop_left` when the view first joins a size's users): `test result: ok. 1 passed`.
- mch-2 and mch-4: `crates/chora/src/heap.rs`, `tune()` and `trim()`, one allow with a reason, a `// SAFETY:` line per block, cfg(all(target_os = "linux", target_env = "gnu")), no-op elsewhere; main.rs and mesh.rs call it. `cargo check --workspace` green.
- mch-3: `assert_eq!(made, 2 * 10, ...)`.
- TX-CPU-SAFE: `thread_cpu()` reads fields 14 and 15 of /proc/thread-self/stat, USER_HZ 100 fixed (doc comment); no unsafe. Five runs under CHORA_TX_LOAD=1, sl-3 cpu: 550ms, 540ms, 520ms, 530ms, 500ms (bar 3 s).
- Ratchet: main.rs's print_stderr allow went (writeln! to stderr); `ok: lint-ratchet (allow 360, no-reason 0, eprintln 130)`.
- Gate: full `tests/merge-gate.sh --jobs 2`, 57 PASS, no red, `EXIT 0`.
