# minors-2

## r1
Branch claude/minors-2, head 8e677c1c (base 9db45f8a). Builder sonnet lane.
- rd-9b closed. Red test 7e56ae02 `ipc::tests::tell_ungrabbed_does_not_tell_a_move_binding` (failed), fix 36cb29c3: the Move filter now sits only in `tell_ungrabbed` (ipc/grabs.rs); call-site check in `ipc_ungrab` and the `!= Trigger::Move` filter in `drop_grabs_where` (ipc.rs) removed. `drop_free_grabs` still never drops a move grab (a move grab is free, not revoked by grab-any).
  - Note for PM: the review text said close (ipc_close) and revoke (input.rs) still told a move grab. They did not: `drop_grabs_where` already filtered Move, so the close-path test `a_move_grab_that_ends_with_its_connection_is_not_told` was green before the fix (a pin, not a red). The red is the direct `tell_ungrabbed` test, which pins the one-place rule for all three callers.
- rl-1 closed (8e677c1c): chora-draw theme.rs `draw_file` casts moved to helpers `tile_pixmap`, `pixmap_size`, `rect_f32`; chora-mc layout.rs `panel` casts moved to `lines_shown`, `row_top` (also used by `overview_hits`, replacing its `#[allow(cast_precision_loss)]`). Both `too_many_lines` expects removed; no new allow or expect outside the helpers' reasoned `expect`s. No behaviour change, existing tests green.
- Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy` all PASS (run on the tree equal to 8e677c1c).

## r1 PM review (2026-11-19 21:05 UTC; PM, not the builder)
Base 9db45f8a, head 8e677c1c; 4 files, +104 -42. Identity: all three commits Marcel Arpogaus (author, committer).
- rd-9b: red first (7e56ae02 tell_ungrabbed_does_not_tell_a_move_binding), fix 36cb29c3. The Move filter is now only in tell_ungrabbed; drop_grabs_where returns every dropped binding; the one caller (close path, ipc.rs:1190) passes them to tell_ungrabbed. drop_free_grabs still keeps a move grab on revoke (not dropped, so not a filter): correct. PM premise note: the close and revoke paths had already filtered move, so rd-9b was a one-place-rule cleanup, not a live bug; the lane said so. Closed.
- rl-1: both too_many_lines expects gone; the cast expects moved with their reasons into helpers (theme.rs 3, layout.rs 2); two `allow` became one `expect` (stricter). Net expects -3, allows -2. Closed.
- Gate (lane): pre-commit, ci-check, build, test, clippy PASS; the full gate runs in train 73.
Findings: none open. Verdict (computed): MERGE-OK 8e677c1c.
