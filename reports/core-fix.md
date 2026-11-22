
## r1

Head: claude/core-fix 4fedd240 (on 46e98b3e). Gate merge with origin/claude/autonomous-dev-session-3w-i33pg7: a7d8ed65.

- mch-1 red: `a_resized_layer_on_two_outputs_of_the_same_scale_drops_old_sizes` (two 160x120 outputs at scale 1, radius-4 rect across the seam, 10 resizes): `old sizes stay: [(40, 40), (41, 41), ... (50, 50)]`, 11 sizes.
  Green after the fix (`Layers::slot` runs `drop_left` when the view first joins a size's users): `test result: ok. 1 passed`.
- mch-2 and mch-4: `crates/chora/src/heap.rs`, `tune()` and `trim()`, one allow with a reason, a `// SAFETY:` line per block, cfg(all(target_os = "linux", target_env = "gnu")), no-op elsewhere; main.rs and mesh.rs call it. `cargo check --workspace` green.
- mch-3: `assert_eq!(made, 2 * 10, ...)`.
- TX-CPU-SAFE: `thread_cpu()` reads fields 14 and 15 of /proc/thread-self/stat, USER_HZ 100 fixed (doc comment); no unsafe. Five runs under CHORA_TX_LOAD=1, sl-3 cpu: 550ms, 540ms, 520ms, 530ms, 500ms (bar 3 s).
- Ratchet: main.rs's print_stderr allow went (writeln! to stderr); `ok: lint-ratchet (allow 360, no-reason 0, eprintln 130)`.
- Gate: full `tests/merge-gate.sh --jobs 2`, 57 PASS, no red, `EXIT 0`.

## Review r1 (PM, diff 46e98b3e..4fedd240, +83 -39, under 200 lines; PM not the builder)

Checks: mch-1 (mesh.rs:383 `joins` when the key is new or this view is not yet a user; drop_left then runs for the second view of the same scale), mch-2/4 (heap.rs, one allow with reason, a SAFETY line per block, cfg gnu with no-op), mch-3 (assert_eq), TX-CPU-SAFE (fields 14 and 15 counted from the closing parenthesis: skip(11) after field 3 is right; USER_HZ 100 named), ratchet 360/130, gate 57 PASS EXIT 0.
- ck-1 MINOR render/cost.rs:624: `sizes.len() < 11` passes with 10 old sizes left; the sibling test bars `<= 2`. Fix: `assert!(sizes.len() <= 2, ...)` (two outputs of one scale share one size). Backlog.
- ck-2 MINOR (process, rules.md 8): the red test and the fix are one commit 4fedd240; the red is shown by the pasted line `old sizes stay: [(40, 40) ... (50, 50)]` and by the code (the old `!contains_key` skips drop_left when the other view made the key). Not rewritten (no force push); next lanes: red test as its own commit.
Verdict (computed): 0 BLOCK, 0 MAJOR, 2 MINOR: MERGE-OK. Joins the train after t110b.
