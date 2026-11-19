
## r1

Branch claude/f24-rest, head 7231b51a, from the train 74 merge a9e7084d. Gate `--jobs 3 --only pre-commit,ci-check,build,test,clippy,e2e` green (e2e 291 s).

1. Closed, nested.sh step 14 diagnostics: `wait_for` runs `WAIT_DIAG` on a timeout; step 14 sets `not_black_diag` (tests/nested-diag.sh): pixel 100,100, top colour, probe.log tail, lock lines of chora.log. Test: tests/nested-diag-selftest.sh (red a5c804dd, fix 23377de0), wired into the gate-lib step. Not run live: the flake itself did not recur.
2. Closed, GR2 voice client.rs: `regrab()` = start_grabs, grab, end_grabs; both `ungrab_all` calls and their `#[allow]` are gone. Test: `a_regrab_leaves_no_moment_without_the_grab` (FakeCore log shows no input.ungrab and no second grab; red 95c82e06, fix b66f2844). voice got the kit `testing` feature as a dev-dependency.
3. Closed, check for a bare start_grabs to end_grabs: tests/grab-gap-check.py (all tracked .rs, in the ci-check step), selftest tests/grab-gap-check-selftest.sh (red b258a451, check 7231b51a). It flags no grab call between the two; the repo is clean. It does not see a grab in a callee that skips grabbing on a branch.
Not touched: the temp-folder part of F24 (lane temp-dirs); the 300 ms cube.open watch (N3).

## PM review r1 (22:33)
Checks: red first for each item (95c82e06, a5c804dd, b258a451); client.rs regrab() = start_grabs, grab, end_grabs replaces both ungrab_all + grab sites and their two allows; test a_regrab_leaves_no_moment_without_the_grab on FakeCore; nested.sh step 14 diagnostics with selftest; grab-gap-check in ci-check. 8 files, +158 -15.
- NIT f24r-1: grab-gap-check misses a callee that skips its grab on a branch (named by the lane); backlog.
Verdict (computed): no open BLOCK or MAJOR -> MERGE-OK 7231b51a, train 78.
