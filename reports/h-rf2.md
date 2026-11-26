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

## Review r2

Review of claude/h-rf2-r2 a2b080ab..59ba5c21e (sonnet, high; one round, read-only on code). **Verdict: FIX** (one MAJOR open).

Checks:

1. Red first: `tests/red-first-selftest.sh` at c44aaa752 (parent's script) fails 11 checks (rc 1, 26 s: total line, mixed commit x4, edited test, failed checkout, build error x2, signal x2); at 23dc8ec22 passes; at 1e1d7ab7d fails 1 (inline edit); at 0e4d426ab and 59ba5c21e passes (10 s). `cargo check --workspace` ok.
2. rhrf2-1..6: one selftest case each, all closed as reported. Reproduced rhrf2-1 (mixed commit rc 0, `gate cell too, not run`) and rhrf2-4 (broken module outside the test is `unproven`, not red) through the selftest. But the rhrf2-1 rule is not general: see rhrf2r2-1.
3. False reds: functions of the head sourced at 4d8bbe36, range 3be27e32..4d8bbe36: rc 0, `2 proven red, 6 not run` in 50 s (target shared); each of the 6 lines has its named reason (4 gate cells, 142260b96 edits tests, 07654d7c2 no test function); 8582a5284 and c3925fc55 red. No valid commit of the real range flagged.
4. Cleanup: after the real range `git worktree list` has no extra entry and `/tmp/red-first-run.*` is absent. Traps INT/TERM restored after return (`mine`/`t` survive; none stay when none were set). Group SIGTERM: selftest case passes. Lead-pid SIGTERM: see rhrf2r2-3. Range time 50 s, far under 5 min.
5. Shell: `bash -n` ok; shellcheck not installed (not run); `set -u` fine, arrays declared; sleep ratchet unchanged (`ok: sleep-ratchet (sleep 778, sleep-all 929)`).

Findings:

- rhrf2r2-1 MAJOR tests/red-first.sh:347-352: a passing added test beside an edited test (the real red, not run) is flagged `RED-FIRST NOT RED`, a valid commit blocked. Reproduced in a temp repo: test(x) edits `tests/it.rs` (red on the parent), test(x) adds a passing `#[test]`, fix(x): `RED-FIRST NOT RED`, rc 1. Same class as rhrf2-1, closed only for the gate cell. Fix: treat `passes = 1` with `edits = 1` like `gate = 1` (print `edited test too, not run`, count not run, do not set bad), and add the selftest case.
- rhrf2r2-2 MINOR tests/red-first.sh:347-352 (by reading, not reproduced): a crate whose test passes beside another crate whose result is `unproven` (build error outside the test, none ran) also reaches `RED-FIRST NOT RED`, although the unproven crate may be the red. Fix: record any `unproven` verdict in a flag and let it excuse a pass like the gate and edit cases.
- rhrf2r2-3 NIT tests/red-first.sh:251-262, tests/merge-gate.sh:239: SIGTERM to the gate pid alone (not its group) kills the main shell at once (the trap at merge-gate.sh:562 is not set on the `--red-first-check` path), the `( ... red_first_run )` subshell is orphaned and keeps running; its trap runs only when the foreground cargo ends, so the worktree and `red-first-run.*` stay until then (reproduced: 10 files, 2 worktrees until the fake cargo ended, 20 s; a real cargo test runs minutes). The selftest kills the group only. Fix: run red_first_run without the subshell (or trap TERM in the check path and kill the group) and add a lead-pid case.

Not findings: shellcheck absent; merge-gate.sh:52-57 comment runs the red-first paragraph into the `pre-commit` sentence (old, now longer).

## r3

Branch claude/h-rf2-r3, base 59ba5c21e, head 4b0d327a0 (2 commits, 2 files, +44 -3: tests/red-first.sh +6 -3, tests/red-first-selftest.sh +38).

- Red first, 0270d2bc4: two selftest groups, 7 checks fail on 59ba5c21e (rc 1, `NOT RED`): a passing added test beside an edited test (rhrf2r2-1), a passing crate beside an unproven crate (rhrf2r2-2; second crate `rfy` with a broken module in a workspace).
- Fix, 4b0d327a0: one rule. `unp` flag set on any `unproven` verdict; a pass with `gate`, `edits` or `unp` set prints `gate cell too, not run` and/or `test commit edits tests too, not run` (the unproven crate prints its own line), counts not run, does not set bad. The gate-cell special case is replaced, not joined by a third.
- rhrf2r2-1, rhrf2r2-2 closed. rhrf2r2-3 NIT and the merge-gate.sh:52-57 comment not in scope (backlog).

Results:

1. Selftest on 4b0d327a0: all ok. On 59ba5c21e with the new cases: 7 FAIL.
2. Real range `--base 3be27e32` (head's own range, HEAD included): rc 0, 47 s, `2 proven red, 6 not run` (8582a5284, c3925fc55 red; 07654d7c2 no test function; 369cf96fd, e6afbf977, 23dc8ec22, 0e4d426ab gate cell; 142260b96 edits tests).
3. Full gate `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2`, no --push, on 4b0d327a0: every step PASS (docs SKIP, mkdocs missing), last lines: `e2e PASS 278s`, `skipped: docs (mkdocs is not installed (docs unchanged))`, `all steps passed; no --push, nothing is pushed`. No flakes. `pgrep -fa 'chora --headless'` empty.

## Review r3 (PM review, 2026-11-26; diff 59ba5c21e..4b0d327a0, 2 files +44 -3, under 200 lines)
1. Red first: `tests/red-first-selftest.sh` on 0270d2bc4 (script of 59ba5c21e): 8 FAIL lines; on 4b0d327a0: rc 0, "ok: red-first-selftest".
2. rhrf2r2-1 MAJOR closed: a pass beside an edited test prints "test commit edits tests too, not run", counted not run (selftest case 1). rhrf2r2-2 MINOR closed: an `unproven` crate excuses a pass (selftest case 2). One rule `gate$edits$unp != 000`, KISS.
3. rhrf2r2-3 NIT stays (backlog).
Verdict (computed): no BLOCK, no MAJOR: MERGE-OK. Joins train 158 first (harness).
