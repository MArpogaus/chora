
## r1

Branch claude/f24-rest, head 7231b51a, from the train 74 merge a9e7084d. Gate `--jobs 3 --only pre-commit,ci-check,build,test,clippy,e2e` green (e2e 291 s).

1. Closed, nested.sh step 14 diagnostics: `wait_for` runs `WAIT_DIAG` on a timeout; step 14 sets `not_black_diag` (tests/nested-diag.sh): pixel 100,100, top colour, probe.log tail, lock lines of chora.log. Test: tests/nested-diag-selftest.sh (red a5c804dd, fix 23377de0), wired into the gate-lib step. Not run live: the flake itself did not recur.
2. Closed, GR2 voice client.rs: `regrab()` = start_grabs, grab, end_grabs; both `ungrab_all` calls and their `#[allow]` are gone. Test: `a_regrab_leaves_no_moment_without_the_grab` (FakeCore log shows no input.ungrab and no second grab; red 95c82e06, fix b66f2844). voice got the kit `testing` feature as a dev-dependency.
3. Closed, check for a bare start_grabs to end_grabs: tests/grab-gap-check.py (all tracked .rs, in the ci-check step), selftest tests/grab-gap-check-selftest.sh (red b258a451, check 7231b51a). It flags no grab call between the two; the repo is clean. It does not see a grab in a callee that skips grabbing on a branch.
Not touched: the temp-folder part of F24 (lane temp-dirs); the 300 ms cube.open watch (N3).
