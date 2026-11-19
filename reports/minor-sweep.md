
## r1 PM review (2026-11-19 20:48 UTC)
Diff 219d5afe..dfa8ddd8, 5 files, +90 -4; builder sonnet lane, reviewer PM. Identity ok; red test 4898e387 before fix fc31fdf5.
- OK rd-9: explicit ungrab of move no longer told (ipc/grabs.rs), test an_explicit_ungrab_of_move_is_not_told.
- OK rc-6: surface_created_carries_the_parent_of_the_first_commit pins D116 (no event on a later set_parent); test helper Wayland::reparent.
- OK gate-lib queued start <= 1; rm-1 synthetic-row selftest (test_acts.py 109 ok).
- MINOR rd-9b: the move filter sits at one call site; connection close (ipc.rs ipc_close, tell_ungrabbed(dropped)) and the grab revoke (input.rs ~364) still tell a move grab. POLS: one rule in one place. Fix: filter Move inside tell_ungrabbed, with a test for the close path. Backlog.
Verdict (computed): no BLOCK or MAJOR -> MERGE-OK; dfa8ddd8 to train 73.
