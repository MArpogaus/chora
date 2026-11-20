# kit-tests

## r1

Brief: briefs/kit-tests-r1.md. Branch claude/kit-tests from 79c8d2b5 (train 81). Tests only; no product code changed, no test is ignored, no bug found.

Step 0: `git fetch --unshallow` and `cargo check --workspace` pass. The branch claude/autonomous-dev-session-3w-i33pg7 is already contained in the base (merge is a no-op).

1. Harness, sha a09fa5a3 (`test(chora-kit): gate the perform test on the testing feature`): tests/perform.rs imports the feature-gated `chora_kit::testing`, so it did not compile. Added `[[test]] name = "perform", required-features = ["testing"]`. `cargo test -p chora-kit --all-features` and `cargo test -p chora-kit` both pass.

2. Unit tests through FakeCore, sha 40b61f28 (`test(chora-kit): cover give, kept, conn and state edges`). The new files have `[[test]]` entries with `required-features = ["testing"]`.
   - wm::give, tests/wm_give.rs: `a_given_window_is_forgotten_once_the_core_took_it`, `a_refused_give_keeps_the_window`, `a_give_of_a_foreign_window_or_to_oneself_is_refused`, `a_window_that_was_given_cannot_be_given_again`, `a_pass_on_leaves_the_surface_free_for_any_manager` (round trip through the fake core), `a_created_window_given_away_opens_then_closes_in_order`.
   - wm::kept, tests/wm_kept.rs: `what_the_look_writes_is_read_back_from_the_core`, `a_kept_window_reads_back_and_a_group_is_no_window`, `keys_of_before_are_read_when_nothing_is_versioned`, `a_versioned_key_wins_over_a_key_of_before`, `a_write_retires_the_keys_of_before`, `a_retired_key_is_gone_from_the_core`, `only_the_versioned_keys_are_reserved`.
   - conn, tests/conn.rs: `a_connection_is_named_for_its_log_lines`, `a_token_is_minted_only_when_the_core_mints_one`, `a_query_finds_the_node_of_its_id`, `a_rejected_transaction_is_returned_or_logged_not_failed`, `a_call_answers_within_its_time`.
   - wm::state, in src/wm/state.rs: `one_window_has_its_keys_and_leaves_out_the_unknown_ones`, `a_removed_focused_window_leaves_no_holder`.
   - Stability: `cargo test -p chora-kit --all-features` run 3 times in a row, 387 passing tests each time, no failure.
   - One assumption of mine was wrong and was corrected in the test, not the product: `Remove` on a surface detaches it (the surface stays on the core with no owner); the test now asserts that.

3. Coverage (cargo-llvm-cov 0.9.1 is installed, so it was run; nothing installed). Line coverage of chora-kit, `--all-features --summary-only`:
   - before (79c8d2b5): 18198 lines, 1452 missed, 92.02%
   - after (40b61f28): 18266 lines, 1438 missed, 92.13%
   - conn.rs: 82.52% to 83.74%
   - wm/give.rs: 100% to 100% (already covered by the scripted book tests)
   - wm/kept.rs: 98.77% to 98.77% (already covered; one line missed in both)
   - wm/state.rs: 100% to 100% (82 to 150 lines counted; the new edges are covered)

Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,gate-lib` on claude/kit-tests at 40b61f28 (run dir /tmp/merge-gate.XJu7SA): pre-commit PASS, gate-lib PASS, build PASS, test PASS, clippy PASS (`-D warnings` also clean on all targets), ci-check PASS. No red cell.

## PM review r1 (2026-11-20 03:30 UTC)
Range 40b61f28 on claude/kit-tests (tests plus the Cargo.toml `required-features = ["testing"]` fix a09fa5a3 for `perform`); +467 lines, no production code. PM is not the builder.
- Harness: a09fa5a3 lets `cargo test -p chora-kit` build `perform` without `--features testing`; the lane gate cells (pre-commit, ci-check, build, test, clippy, gate-lib) PASS; 387 tests pass 3 runs in a row.
- Tests: wm/give, kept, conn, state edges; kit coverage 92.02 % to 92.13 % (llvm-cov). Small gain: give and kept were already covered; noted, not a finding.
Verdict (computed): no open BLOCK or MAJOR -> MERGE-OK 40b61f28, train 84.
