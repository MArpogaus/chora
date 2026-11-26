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

## Review r1

Review of claude/h-rf2-r1 4d8bbe36..a2b080ab (sonnet, high; one round, read-only on code). **Verdict: FIX** (one MAJOR open).

Checks:

1. Red first: `tests/red-first-selftest.sh` fails at 36a41daa (9 of 10 checks, exit 2 from the unknown option) and passes at a2b080ab (13 ok, 9 s).
2. Rule: the selftest cases pass (passing test on the parent is NOT RED, red test is green, gate cell named, `No test:` opts out, second fix of one id). Hand-made temp repos: cap prints `run capped to the last 10 of 12 fix commits` and runs 10; a mixed commit is flagged (rhrf2-1).
3. Real range 3be27e32..4d8bbe36 (no gate run): rc 0, 21 to 37 s (cold target 37 s). No valid commit flagged red. 8582a5284 and c3925fc55 red on the parent; 06c3f7cc6, f396ebe63, 369cf96fd, e6afbf977 gate cells; 142260b96 "test in the fix commit"; 07654d7c2 no test function; d590aae69, 822d5be4a opt out. Only 2 of 11 fixes are proven red.
4. Gaps: 689bff45's range (b710ead48^1..^2) rc 0, 9 s; 689bff45f is a gate cell, not run: reported honestly in the report and CONTRIBUTING (named `gate cell, not run`). After a normal run no worktree or `/tmp/red-first-run.*` is left (`git worktree list` clean); the shared target does not grow over 3 runs (503 M).
5. Shell: `bash -n` ok; shellcheck is not installed (not run).

Findings:

- rhrf2-1 MAJOR tests/red-first.sh:286-289: a test commit that adds a Rust test passing on the parent and a gate cell (the real red) is flagged `RED-FIRST NOT RED` although `gate=1` (reproduced in a temp repo: test(x) adds tests/gate-q.sh exit 1 plus a passing #[test]; fix flagged, rc 1) - a valid commit blocked. Fix: when `gate=1` and no Rust test is red, print `gate cell too, not run` and do not set bad.
- rhrf2-2 MINOR tests/red-first.sh:151-160: only added `#[test]` functions are run; a red test written as an edit of an existing test (64d9a7a42, in the real range) is invisible, and its fix 142260b96 prints the misleading `test in the fix commit, not run`. Fix: when a paired test commit changes Rust tests but adds none, print `test commit edits tests, not run`.
- rhrf2-3 MINOR tests/red-first.sh:232,265,298: no trap; an interrupt or SIGPIPE leaves the worktree and `/tmp/red-first-run.*` (seen once), the `|| return 1` after `worktree add` leaks the mktemp dir, and a failed `checkout` of the parent `continue`s silently (a fix skipped with no line). Fix: `trap` cleanup on RETURN/INT/TERM and print `red-first: <sha> checkout of the parent failed, not run`.
- rhrf2-4 MINOR tests/red-first.sh:202: any `error: could not compile` of the crate on the parent counts as red, also an unrelated breakage (missing dependency, offline) - a false green. Fix: count it only when the first error names a file of the added test, else `unproven`.
- rhrf2-5 NIT tests/red-first.sh:290-297: the unproven outcomes (gate cell, in the fix, no function) exit 0 with no total, so a range with 2 proven of 11 reads as green; CONTRIBUTING.md:116-124 does not say that these pass. Fix: print `red-first: N proven red, M not run` and one clause in CONTRIBUTING.
- rhrf2-6 NIT tests/red-first-selftest.sh:8: no case for the cap line, the mixed commit or an edited test; the red cell at 36a41daa fails on the unknown option (exit 2) for 9 of 10 checks, not on behaviour; `set -u` only, no `-e`/`pipefail` (the library is sourced). Fix: add the three cases.

## r2

Fix of review r1 (sonnet, high) on claude/h-rf2-r2: base a2b080ab (h-rf2-r1 head), head 59ba5c21e. Diffstat a2b080ab..59ba5c21e: CONTRIBUTING.md 9, tests/merge-gate.sh 4, tests/red-first-selftest.sh 123, tests/red-first.sh 101 (4 files, +218 -19). No product code.

Commits: c44aaa752 `test` (red selftest cases: mixed commit, edited test, failed checkout, unrelated build error, build error in the test file, cap line, signal, total; 11 FAIL on a2b080ab, the cap case and the positive build case already passed), 23dc8ec22 `fix` (rhrf2-1..6), 1e1d7ab7d `test` (red: an edit of an inline test) then 0e4d426ab `fix`, 59ba5c21e `test` (the signal case waits on a fifo and `tail --pid`, not `sleep`: the sleep ratchet counts them).

Findings:

- rhrf2-1 closed: a passing test beside a gate cell prints `gate cell too, not run`, no NOT RED (selftest: rc 0, 0 NOT RED).
- rhrf2-2 closed, wider than r1 asked: a paired test commit that changes `tests/*.rs` without adding a `#[test]` prints `test commit edits tests, not run`; a `test`-type commit before the fix that edits `.rs` but adds no `#[test]` line (64d9a7a42 is such: an inline test edit, not a test commit for `changes_test` at all) prints the same line. 142260b96 now prints it.
- rhrf2-3 closed: cleanup of the worktree and the mktemp dir on return, INT, TERM and PIPE (the mktemp dir is registered before `worktree add`); previous INT/TERM/PIPE traps restored; a failed checkout of the parent prints `checkout of the parent failed, not run`. Done as a wrapper (`red_first_run` sets the traps, `red_first_range` is the old body) instead of `trap RETURN`, which leaks into later returns. Selftest: SIGTERM to the process group during a (fake) cargo run leaves no `red-first-run.*` and one worktree.
- rhrf2-4 closed: a compile error is red only when its first `error` block names a file of an added test (suffix match on `/`), else `unproven <crate>: build error outside the added test, not run`.
- rhrf2-5 closed: the run ends with `red-first: N proven red, M not run` (opt-outs `No test:` are in neither); one clause in CONTRIBUTING and in the merge-gate.sh header. A not-run fix still passes: only a test that passes on the parent fails.
- rhrf2-6 closed: the cases above in tests/red-first-selftest.sh (now 9 s). The `set -u`-only note is left: the library is sourced, `-e` would change its returns.

Results:

1. Selftest: all ok, 9 s; red on a2b080ab (11 FAIL, then 1 FAIL for the inline edit on 23dc8ec22).
2. Real range 3be27e32..4d8bbe36 (functions of this head sourced in a worktree at 4d8bbe36): rc 0, 22 s (warm target); `2 proven red, 6 not run` = the 6 lines (06c3f7cc6, f396ebe63, 369cf96fd, e6afbf977 gate cells; 142260b96 edits tests; 07654d7c2 no test function); 8582a5284 and c3925fc55 red; the 2 opt-outs print no line. Nothing flagged red wrongly. On the head's own range (`--base 3be27e32`, HEAD included) rc 0 too, 38 s cold. After both no worktree and no `/tmp/red-first-run.*`; an interrupted run is the selftest case.
3. Full gate `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2`, no --push, on 59ba5c21e: all steps PASS (docs SKIP, mkdocs missing), `all steps passed`. An earlier run on 0e4d426ab stopped at ratchets: the selftest's two `sleep 0.5` raised the sleep ratchet (779 > 778); fixed by 59ba5c21e. No flakes. `pgrep -fa 'chora --headless'` empty at the end.

Open: none of rhrf2-1..6. Known limits: an edited test is named, not run; the `lint-ratchet` note `allow is 359, listed 360` is old.
