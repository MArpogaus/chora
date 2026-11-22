# test-gaps

Plan slice 49 (TEST-GAPS, debt H13: gaps outside lane areas). Lane: claude/test-gaps, head 89dc32c4.

## r1

| Gap | Test or fix | Commit |
| --- | --- | --- |
| WD2: `waits_in`, `wait_of`, `stop_in` untested | `safety::tests::the_waits_of_the_threads_come_from_their_schedstat`, `a_malformed_schedstat_gives_no_wait_and_a_missing_folder_no_waits`, `the_wait_of_a_frame_is_the_growth_of_the_waits_between_its_reads` | e33eb269 |
| WD3: a thread new in the second read credited its whole wait | red: `safety::tests::a_thread_that_appears_mid_frame_is_not_credited_its_whole_wait` (failed alone: 40.3 ms vs 0 ns) | 5a3cbff7 |
| WD3 fix (`stop_in` skips threads missing from the first read) | the red test, and all 28 `safety::tests` pass | 92245e0e |
| secfix-b LOW: reload wiring | existing `runtime::tests::reload_settings_reads_the_file_and_applies_it` (runtime.rs:853) already covers the parse failure; added the missing assertion that the plugin applies nothing | 89dc32c4 |
| SA10: opt-out docstring | `tests/injected-optout.py` docstring now says that a plain `injected: name` is reported like `injected: _`, and a `..` after it too (checked with the script on a sample file) | 3a927a4f |

Gate on claude/test-gaps at 89dc32c4 (already contains claude/autonomous-dev-session-3w-i33pg7 at 7a6c0eb8, so no merge was needed):
- `cargo test -p chora -p chora-kit`: exit 0
- `cargo clippy --workspace -- -D warnings`: exit 0
- `tests/merge-gate.sh --jobs 1`: EXIT 0, all steps PASS, no foreign reds

Left for a later round: grab_action (crates/chora/src/input, lane ti-38) and SA9 (plugins/chora-xp).

## PM review r1 (09:06 UTC; PM is not the builder; small diff, owner 11-19 06:12)
Range 7a6c0eb8..89dc32c4, 3 files, +80 -10. Checks: WD2 tests read schedstat from a temp task tree (waits_in, wait_of, growth between reads); WD3 red first (5a3cbff7) then fix 92245e0e: a thread missing in the first read is skipped (filter_map), not credited its whole wait; secfix-b asserts a parse failure applies nothing (runtime.rs:870); SA10 docstring now says a plain `injected: name` is reported like `_`. Lane gate: merge-gate --jobs 1 EXIT 0. Findings: none MAJOR; LOW tr-1 the growth tests sleep 40 ms (timing only asserted as >=). Verdict (computed): MERGE-OK; joins t109.
