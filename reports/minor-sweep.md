## r1

Lane: cloud, sonnet, branch claude/minor-sweep from train 72 merge 219d5afe, head dfa8ddd8.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,gate-lib` green (pre-commit 3 s, gate-lib 24 s, build 117 s, test 177 s, ci-check 26 s). `selftests` is no step name; gate-lib runs gate-lib-selftest.sh and under-load-selftest.sh. `python3 -I tests/test_acts.py` (run by the acts step) green, 109 tests.
Closed:
- rd-9: ipc/grabs.rs ipc_ungrab skips tell_ungrabbed for Trigger::Move. Test ipc::tests::an_explicit_ungrab_of_move_is_not_told, red 4898e387 (told the "move" binding), fix fc31fdf5.
- rc-6: ipc::tests::surface_created_carries_the_parent_of_the_first_commit (fc4a6e5f): no parent for a plain toplevel, parent s1 for a child, no event after a later set_parent (also set_parent None). Code agrees with docs/api.md and D116; no doc change. Adds Wayland::reparent to testing/wayland.rs.
- gate-lib-selftest queued-start (672e168a): accepts gate_elapsed <= 1.
- rm-1: KnownRed.test_a_second_overlap_of_a_listed_row_blocks in tests/test_acts.py on a synthetic row syn I5 with match (dfa8ddd8).
Noted, not changed: rd-9 covers only the explicit ungrab; the drop of a connection (ipc_close, ipc.rs:1188) and the revoke of grab-any (input.rs:364) still tell a move grab, if drop_grabs returns it.

## r1 PM review (2026-11-19 20:48 UTC)
Diff 219d5afe..dfa8ddd8, 5 files, +90 -4; builder sonnet lane, reviewer PM. Identity ok; red test 4898e387 before fix fc31fdf5.
- OK rd-9: explicit ungrab of move no longer told (ipc/grabs.rs), test an_explicit_ungrab_of_move_is_not_told.
- OK rc-6: surface_created_carries_the_parent_of_the_first_commit pins D116 (no event on a later set_parent); test helper Wayland::reparent.
- OK gate-lib queued start <= 1; rm-1 synthetic-row selftest (test_acts.py 109 ok).
- MINOR rd-9b: the move filter sits at one call site; connection close (ipc.rs ipc_close, tell_ungrabbed(dropped)) and the grab revoke (input.rs ~364) still tell a move grab. POLS: one rule in one place. Fix: filter Move inside tell_ungrabbed, with a test for the close path. Backlog.
Verdict (computed): no BLOCK or MAJOR -> MERGE-OK; dfa8ddd8 to train 73.
