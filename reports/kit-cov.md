# kit-cov

## r1

Lane: claude/kit-cov, head 619da642 (two commits on 892d035c; origin/claude/autonomous-dev-session-3w-i33pg7 is already contained). Tests only, no product change.

### Coverage (`cargo llvm-cov -p chora-kit --summary-only`, lines column)

| File | Before | After |
| --- | --- | --- |
| runtime.rs | 70.39 % (199 of 672 missed) | 89.00 % (96 of 873 missed) |
| icons.rs | 80.94 % (65 of 341 missed) | 99.32 % (3 of 439 missed) |
| launcher.rs | 85.51 % (51 of 352 missed) | unchanged, already at or above 85 %; no tests added |

Notes:
- The plan row (week-3 row 42) gives runtime.rs 72.9 %. The line figure measured on 892d035c is 70.39 %; its region figure is 74.94 %. The measured figure is used here.
- llvm-cov counts the `#[cfg(test)]` lines of a file too, so the line totals grow with the new tests.

### Tests added

- `test(kit): cover icons.rs search, cache and copies` (d09677f8): 8 tests in the existing test module. Theme search (SVG first, then the largest PNG up to 64 px, wanted size wins, symbolic skipped), the answer cache of `find` and `find_sized`, slash names, the generic program icon fallback, `first_sized`, and the eviction of old sender copies past the limit.
- `test(kit): cover runtime.rs dispatch, settings and grabs` (619da642): 12 tests in the existing test module. `from_person` and `hears_pointer`; the defaults of `Plugin` for settings, state and their errors; `settings.get`, `settings.set` and `settings.reload` on a plugin with settings and default hooks; `accept_injected`; `holds`; `announce` (subscribe and export); `regrab_give_keys` with its interval gate; the give keys kept by a new set of grabs; `dispatch` of a method call and of output events; `method` and `privileged`; `run` ending at once.
- Total: 20 new tests. The chora-kit lib test count goes from 299 to 319.

### Bugs found

None. No test is `#[ignore]`d.

### Not covered (left as is)

- runtime.rs: the INVALID_PARAMS branch of `Runtime::grab` and the parse-error branch of `end_grabs` (the fake core can only answer code 1); `serve` and `start`/`connect` (no way to send events or a real socket from a test); the retry path of `run` (it sleeps 500 ms per try); `report`.
- icons.rs: the error branch of a failed write in `private_copy_in`.

### Gate

- `cargo clippy -p chora-kit --all-targets -- -D warnings`: pass.
- `cargo test -p chora-kit`: pass (lib 319, integration suites all ok).
- `pre-commit run --all-files`: pass on the second run. The first run's cargo fmt reformatted the new tests, and the vale hook failed once with "Text file busy" in its installer; the second run passed.
- `tests/merge-gate.sh --jobs 1` (no --only) on claude/kit-cov merged with origin/claude/autonomous-dev-session-3w-i33pg7: all steps passed, 49 cells PASS, 0 FAIL, nothing pushed by the gate. The first gate run failed ci-check on a guard (GR1) for `start_grabs` followed directly by `end_grabs`; the test was changed to grab inside the new set and the gate was re-run green.

## Review r1 (PM, 2026-11-21 19:30 UTC; tests-only diff, PM not the builder)
Base..head 892d035c..619da642: d09677f8 (icons.rs +129), 619da642 (runtime.rs +258); 20 new #[test], no production line changed.
Checks: diff touches only #[cfg(test)] blocks of crates/chora-kit (git diff --stat: 2 files, 387+, 0-); file tests use crate::testing::TempDir and unique names (no shared /tmp path, no env::set_var, no sleep, no #[ignore]); coverage numbers from the lane (runtime 70.39 -> 89.00 %, icons 80.94 -> 99.32 %) meet row 42 (>= 85 %); lane gate 49 cells PASS at --jobs 1.
Findings: 0 BLOCK, 0 MAJOR. Verdict: MERGE-OK (joins t104 after t103).
