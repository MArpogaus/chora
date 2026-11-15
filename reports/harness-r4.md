# harness r4 (claude/harness 7ef5fe1f): close review-harness-r3 (M1, M2, m1-m3)

## Commits (on 4321f0e8)
- 2e56d0d9 merge origin/claude/autonomous-dev-session-3w-i33pg7 (be205f89). Conflicts in
  tests/agent.sh (kept `pgrep -P "$chora_pid"` and the session's `ctl_name`) and
  tests/gate-lib-selftest.sh (kept both blocks). clippy --workspace --all-targets ran clean
  on the merge before the next commit (review m3 lesson).
- df4118b6 test / 7fdac11c fix(hook): M2. On a branch of the base's name the hook prints
  a note and passes; the merge gate's red-first step stays the enforcing check. A lane
  branch made with `checkout -b claude/lane origin/<base>` still refuses (selftest case);
  that is why @{upstream} is not used: such a lane tracks the base.
- 0209e5ba test / bb818ceb fix(gate): m2. The runtime grep opens an allow only on a
  `#[allow(`/`#[expect(` (also `#![...]`, also over several lines) that names
  clippy::disallowed_methods; the attribute indent resets per file.
- 7ef5fe1f fix(gate): M1 + m1 in tests/no-foreign-kill.sh, with its guards cases in the same
  commit (the guards hook refuses a red guards.sh, as in b822a958). The reader walks
  quotes with a stack: literal text is blanked (also multi-line), `$(...)` and backticks
  inside strings are kept. Options are read from the tool word on (kill/timeout -s),
  `((...))` is blanked (no fake here-document), echo/printf/for skip only without a
  substitution, a tool followed by a tool name is no call, a first argument in a
  variable only warns (rc 0), a comment line naming environ is no filter.
  CONTRIBUTING.md updated for both hooks.

## Red / green
- merge-gate-selftest at 0209e5ba (fixes absent): 4 FAIL (base branch x2, plugin runtime x2);
  green after 7fdac11c + bb818ceb.
- tests/guards.sh with the r3 hook and the new cases: 14 FAIL (13 kill cases + the nested
  run); green at 7ef5fe1f ("all guards behave"). Hook on all tracked *.sh: rc 0, 0.5 s.
- Full serial tests/merge-gate.sh at 7ef5fe1f: all steps passed, 0 SKIP.
- tests/ci-local.sh: msrv, docs, pre-commit, deny, machete, packaging PASS.

## m3
reports/harness-r3.md corrected (session branch not red; the red came from the merge
58792403; clippy was the earliest gate). The 4321f0e8 allow stays.

## Open
- Review rt/b: string method calls inside a correctly allowed item stay hidden; not in the
  brief, and fake-core tests may name those methods as strings.
- `kill $(pgrep $scope x)` warns rather than fails; by design (scope unreadable).
