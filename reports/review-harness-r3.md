# review harness r3 (claude/harness 4321f0e8)

Independent and read-only. The probes are in /tmp/claude-0/rev-harness3.
probe.sh builds a bare origin, a full clone, a `--depth 1 --single-branch`
clone, `--depth 1` and `--depth 2` clones of all branches (so the base
resolves), a clone with no remote, an orphan branch, a detached HEAD and a
worktree. Its commit-msg hook is tests/red-first.sh. The nfk/ and rt/
directories hold small files for tests/no-foreign-kill.sh and
tests/gate-0.8-plugin-runtime.sh. clippy-a7.log is a clippy run at a7a3150c.

## BLOCK
- none.

## MAJOR
- M1 tests/no-foreign-kill.sh:41 (`gsub(/"([^"\\]|\\.)*"/ ...)`) blanks every
  double-quoted string before it looks at the line. That also blanks a
  command substitution inside quotes, which is the usual shell form. These
  lines pass the new hook (rc 0):
  `running "$(pgrep -x chora-xp)" || fail x`,
  `kill "$(pidof chora)"` and `pids="$(pgrep chora)"`
  (probes nfk/q1, q4, q3). The hook at 58792403 refused q1 and q4, so this
  is a regression. q1 is the form xp.sh:654 used before this round. The
  guards cases (tests/guards.sh:245-267) test only the unquoted `$(pgrep ...)`.
  So the hook now misses the M1 incident class in the form people most often
  write. Fix: take the text of `$(...)` and backticks out of a string before
  blanking it, or blank only strings that hold no `$(`. Then add a guards
  case `"$(pgrep -x chora)"` that must fail. Earliest gate: tests/guards.sh,
  which has no case for a quoted command substitution.
- M2 tests/red-first.sh:35-43 refuses a valid commit made on the base branch
  itself. On claude/autonomous-dev-session-3w-i33pg7, base..HEAD holds only
  the commits that are not pushed yet. Probe "sess" in probe.sh: commit
  `test(z): red`, push to the session branch, fetch, then commit
  `fix(z): green`. The hook prints "RED-FIRST MISSING: fix(z): green" and
  refuses. The test is in an earlier commit of the same branch, so this
  is the B1' failure again, caused only by push state. The merge gate never
  judges first-parent commits of the session branch, so the gate does not
  back up this refusal. It can happen in practice: the owner commits
  straight onto the session branch (21d4a477, 6b1efd0c, 58fbe616 are on its
  first-parent line). It breaks the PM rule "never block a valid commit".
  `No test:` gets past it, but the same was true of B1'. Fix: when HEAD is
  on the base branch (`git rev-parse --abbrev-ref @{u}` equals the base, or
  base is an ancestor of HEAD with the same branch name), print the note and
  pass. Earliest gate: the hook cases of tests/merge-gate-selftest.sh, which
  have no case for a commit on the base branch.

## MINOR
- m1 tests/no-foreign-kill.sh still refuses valid code, and the brief says to
  warn in that case. Probes: `for t in pgrep pkill jq; do command -v "$t" ...`
  (f1), `need pgrep pkill` (f2), `pgrep $scope -x foot` with the scope in a
  variable (f3), and a multi-line single-quoted awk program that matches
  `/pkill chora/` (f8; quotes are only stripped within one line). Other false
  negatives: `kill -s KILL $(pgrep -x chora)` and
  `timeout -s KILL 5 pkill chora` count kill's or timeout's `-s` as scope
  (q2, q9). `echo $(pkill chora)` is skipped as an echo (q7). An arithmetic
  `<< shift` opens a fake here-document and hides the rest of the file
  (q5). The /proc environ test at line 56 accepts any line naming
  /proc/.../environ within 3 lines, even a comment (q8). The tree passes
  today. The lane said "no rule needed the warn-only fallback", and f1, f2,
  f3 and f8 show that claim is wrong.
- m2 tests/gate-0.8-plugin-runtime.sh:19 opens an allow on any line that
  contains `clippy::disallowed_methods`, including comments and strings.
  Probe rt/a: the comment `// Tests here may not use
  clippy::disallowed_methods.` comes before
  `fn bad(core) { core.call("events.subscribe"); }`, and the grep passes.
  Clippy does not check method-name strings, so for `"events.subscribe"`,
  `"output.list"` and `"methods.export"` this grep is the only check, and the
  comment hides a real violation. Inside a correctly allowed item, those
  string calls are hidden too (rt/b), because an allow for disallowed_methods
  also switches off the string rule. Fix: open the allow only on an
  attribute (`#[allow(`/`#[expect(` and the lines up to its `)]`), and keep
  the string patterns checked under an allow. `attribute` is not reset per
  file (line 18, `FNR == 1` resets only `allowed`), so an inner `#![allow]`
  takes the indent from the previous file. Today that can only make the
  window narrower (rt/c behaves like clippy).
- m3 The lane report says "the session branch is red on ci-check". That is
  false. be205f89 (the session tip, train 13 gate: clippy PASS, ci-check
  PASS) has no plugins/clippy.toml, and its runtime grep has no
  `conn::connect` or `runtime::with` pattern. Both come from this branch
  (b9eb75ca and r2). The red comes from the merge 58792403, which joins the
  session's new on_fake_core (67f1c1f2) with this branch's stricter rules.
  Clippy is the earliest gate, not ci-check: at a7a3150c,
  `cargo clippy -p chora-xp --all-targets -- -D warnings` gives two
  disallowed_methods errors at wm.rs:4314-4315, rc 101, 30 s
  (clippy-a7.log). So 58792403 through a7a3150c (6 commits) fail clippy. The
  allow belonged in the merge commit, which would keep bisect clean. The
  commit message of 4321f0e8 is accurate. Only the report is wrong.

## Checked and fine
- Never-block, red-first (probe.sh, all as expected): full clone test, docs,
  fix after a pushed test, amend; fix without a test with the base
  resolving is refused (correct); single-branch `--depth 1`, no base: fix
  passes with the note (B1' closed); `--depth 1` and `--depth 2` clones of
  all branches with the base resolving: fix after a test beyond the shallow
  boundary passes, because the boundary commit diffs as a root commit, so
  shallow history over-accepts and never hides a test; no remote: unborn
  feat and fix pass with the note (m3 r2 closed); orphan branch first feat
  passes; detached HEAD; worktree; revert; rebase -i; --fixup;
  --autosquash; merge --no-ff with a fix subject. The `--remotes` fallback
  is gone, and no fallback can reach the refusal. A stale base ref only adds
  commits to base..HEAD, so it over-accepts. A test that was already merged
  into the base before its lane's fix is refused, but the merge gate refuses
  the same range, and a red test cannot pass a train alone.
- Red first: merge-gate-selftest fails at e751d5e1 and 9d1b4653 and passes at
  the fixes. gate-lib-selftest fails at ae47dadb (live_child 127) and passes at
  ac4da861. b822a958 puts its test and fix in one commit, and the reason
  holds: tests/guards.sh at b822a958 with the old hook gives 10 FAIL.
- proc-lib.sh: live_child returns the first non-zombie child, and a zombie
  alone counts as dead. It passes one pid to ps, so the r2 m2 error is gone.
  It uses procps syntax (`ps -o stat= -p`). Busybox ps has no -p, but that
  was already true of `running` before. Busybox was not available here to
  test.
- 4321f0e8: a fn-level `#[allow(clippy::disallowed_methods, reason = ...)]`
  is the narrowest scope. It matches the other two fake-core sites
  (chora-mc testing.rs:133, chora-tiling main.rs:2291). A code change is not
  possible, because the test must connect to its fake core. At 4321f0e8,
  clippy for chora-xp passes and the runtime grep passes.
- Speed: the commit-msg hook takes 0.05 s in the real repo. no-foreign-kill on
  all of tests/*.sh takes 0.27 s. The selftests take about 48 s, as before.
  Nothing slows the commit loop.
- The candidate flake F21 (gate-construct-pull.sh:150) is outside this lane.
  The idea to fix it with until_ok is sound.

## Gates missing
- tests/guards.sh: a quoted command substitution `"$(pgrep -x chora)"` (M1),
  and `kill -s KILL $(pgrep -x chora)`.
- tests/merge-gate-selftest.sh: a fix on the base branch itself after a
  pushed test (M2).
- runtime-grep selftest: a comment that names the lint before a
  `"events.subscribe"` hit (m2).
- Each merge from the session into a lane should run clippy before its first
  commit after the merge (m3).

## Verdict
FIX (M1, M2)
