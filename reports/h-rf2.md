# h-rf2: red-first runs the test on the fix's parent

## r1

- Lane: claude/h-rf2-r1. Base 4d8bbe368, head a2b080ab2.
- Commits: 36a41daa6 `test(h-rf2)` (red: tests/red-first-selftest.sh fails 9 of 10 checks on the
  base, which has no `--red-first-run`), then a2b080ab2 `feat(h-rf2)`.
- Diffstat: CONTRIBUTING.md +9; tests/merge-gate.sh +17/-3; tests/red-first.sh +154;
  tests/red-first-selftest.sh +129 (4 files, 309 insertions, 3 deletions).

### Rule (as built)

`tests/merge-gate.sh --red-first-check --red-first-run` (the gate's red-first step passes the flag) runs,
for each `fix` commit of BASE..HEAD, the Rust `#[test]` functions added by its paired test commits at
the fix's first parent: `cargo test -p <crate> --tests --no-fail-fast -- <names>`, in one reused
worktree and the repository's target dir. At least one must fail there (a build failure of the crate
counts as red), else `RED-FIRST NOT RED: <sha> <subject>` and the passing tests. Matching is by the
end of the test path (`mod::name`), not `--exact`, because a unit test's full name has its module path.

Pairing: the test commits of the range that are ancestors of the fix's parent, after the previous
`fix` that names one of the same ids (a fix that names no id: after the previous fix). So a second fix
of one id does not reuse the test of the first (a first version did, and flagged f0557aa36 wrongly).

Named and not run (no failure): `gate cell, not run` (the test is in a tests/gate-*.sh, selftest or
Python script), `test in the fix commit, not run` (the parent has no such test), `no test function
found, not run`, `none of the tests ran on the parent` (unproven). `No test: <reason>` opts out. The run
covers the last 10 fix commits (`red-first: run capped ...` is printed when it cuts).

### Real ranges

- 3be27e32..4d8bbe36 (11 fix commits): passes, 35 to 43 s warm (under 5 min, no cap).
  8582a5284 (construct, 2 tests) and c3925fc55 (mc, 1 test) are red on the parent; 06c3f7cc6,
  f396ebe63, 369cf96fd, e6afbf977 are gate cells (not run); 142260b96 has its test in the fix
  commit (not run); 07654d7c2 has no test function; d590aae69 and 822d5be4a opt out with
  `No test:`.
- 689bff45's range (claude/tiling-reach, b710ead48^1..b710ead48^2, 9 commits): passes, 11 s.
  f358771c1 and 689bff45f are gate cells (not run); 64b60b8ae is red on the parent; f0557aa36 has
  its test in the fix. **The rule does not catch 689bff45**: its test is a gate script
  (tests/gate-launcher-click.sh), which the rule names and does not run (too slow, by the brief). A
  gate cell is checked by that cell's own step only.

### Gate

`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --push) on a2b080ab2: EXIT 0, all steps passed;
`skipped: docs (mkdocs is not installed (docs unchanged))`; no flakes. No `chora --headless` left.
