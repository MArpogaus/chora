# harness r2 (rc 3): close the review of harness r1

Branch claude/harness, pushed at f8ab718a. It merges c2d0a116 (the tiling-audit fix
and the ci-check step) at 80bd9bbb. Commits:
- 953aad46 test / 5f21a086 fix(gate): never block a valid commit in the red-first hook (B1, m1)
- e24935b9 fix(gate): judge the programs of a gate by pid, not by name (M1; tests in the
  same commit, because the guards hook refuses a red guards.sh)
- b9eb75ca chore(lint): refuse Conn::connect_at and Runtime::with in plugins (m2)
- 0db1f39f fix(lint): compare the two clippy configs both ways (m3)
- 155993ae test / a8b97ed8 fix(gate): run 2d-guard alone before the parallel steps (m4)
- cd302dd2, f8ab718a docs(gate): line width and the measured minutes (c1).
  DEVIATION: f8ab718a also carries the agent.sh change of M1 step 18. A refused commit
  left that file staged. The commit was pushed before I saw it, so it stays (no force push).

## B1 and m1
Without the base ref, the hook judges `git rev-list HEAD --not --remotes`. When that is
empty, the commit passes with a note and the merge gate stays the backstop.
MERGE_HEAD always passes. Red: 6 selftest checks failed on the r1 hook, all green now.
Never-block proof, run in a real `--depth 5 --single-branch` clone with the pre-commit
commit-msg hook and no base ref:
- Passed: docs, test, fix after test, feat after test, ci, chore, amend, rebase reword,
  fixup, revert, a merge with a fix subject, fix with "No test:", the first fix in a
  depth-1 clone (nothing unpushed), and a fix after a test with no remote at all.
- Refused, correctly: a fix when the unpushed commits hold no test.
- Fail-open edges: in a shallow clone without a remote, the graft commit lists every
  file, so it counts as a test. pre-commit shows the note only with -v.

## M1
- xp.sh checks its own pids: chora, chora-xp as the child of chora, notify, assistant.
  A zombie counts as dead.
- agent.sh and nested.sh find chora-tiling as the child of their own core.
- agent.sh step 18 matches XDG_RUNTIME_DIR as well as the display. Checked with a stand-in
  program; the step itself does not run here (V12: step 2 fails before and after the change).
- gate-mc-paths uses the new session_pids of gate-lib.sh (pgrep filtered by the gate's
  XDG_RUNTIME_DIR).
- The no-foreign-kill hook now fails any unscoped `pgrep -x`, `pgrep -f` or `pidof`.
  guards.sh has 4 new cases (3 red before); gate-lib-selftest tests session_pids.
- No other step script names a process machine-wide.
- nested.sh: PASS. CI-only and not covered: agent.sh beyond step 2.

## m2, m3, m4
- m2: on the two fake-core test sites, clippy was red before the #[allow] and is green
  after. The grep backstop also matches connect_at, Runtime::with, Conn::connect and
  from_stream, and lets a hit through only below an allow. Planted hits a, b and d are
  red, the allowed hit c is green, the tree is green.
- m3: tests/clippy-config-sync.sh compares the settings of the two files. guards.sh has
  4 cases. Red before: a setting removed from the root file passed.
- m4: 2d-guard is in serial_steps. A selftest case was red before.

## Gate at cd302dd2 (f8ab718a changes only CONTRIBUTING and agent.sh; red-first and
pre-commit PASS there)
- Serial: all steps PASS in 26.7 min, after one rerun. Run 1 failed the test step on F17
  (words::a_slow_backtrace_is_not_cut_by_the_alarm).
- --jobs 3: all steps PASS in 601 s (10 min), after one rerun. Run 1 failed on F16
  (tiling-audit, ink-1280x800, "the bar does not show" in 10 cells; 307 s, slower than
  the usual 274 s, so load may raise F16).
- One more --jobs 3 attempt was cut by my own 590 s timeout (doors still running).
