# harness r3 (rc 3): the hook never blocks a valid commit

Branch claude/harness, HEAD 4321f0e8; merges origin/claude/autonomous-dev-session-3w-i33pg7
(b5266fa4) at 58792403. Commits (test first, then the fix):
- e751d5e1 test / b0809eb0 fix(gate): refuse in the red-first hook only what it can prove (B1', m3)
- b822a958 fix(gate): read commands, not text, in the no-foreign-kill hook (m1; the guards
  cases are in the same commit, since the guards hook refuses a red guards.sh)
- ae47dadb test / ac4da861 fix(gate): take a live chora-xp beside its zombie in xp.sh (m2)
- 9d1b4653 test / a7a3150c fix(gate): let an allow cover only its item in the runtime grep (m4)
- 4321f0e8 chore(lint): allow the fake core connection of the xp keys test. FOUND: the session
  branch (67f1c1f2, fix(xp) cube keys) added on_fake_core in plugins/chora-xp/src/wm.rs:4286
  calling Conn::connect_at and Runtime::with with no allow. The runtime grep (ci-check)
  refused it with the old 40-line window as well; the session branch is red on ci-check.

## 1 red-first (PM decision 11-15)
branch_verdict refuses only when the base ref resolves and base..HEAD has no test. No base
ref or unborn HEAD: one note "...the merge gate checks the range", exit 0. The fallback over
`--not --remotes` is gone (fewer branches). Red: 6 selftest checks failed (unborn HEAD feat,
no-base fix without test, single-branch clone test/push/docs/fix, each with its note); green
after. The old check "no base: a fix without any test is still refused" now passes by design.

## 2 no-foreign-kill
Skips comments, here-document bodies, quoted strings and echo/printf commands; joins `\`
lines; scope = -P/-s/-g/-F attached or not (lowercase flag cluster, so -HUP is no scope) or
--parent/--session/--pgroup/--pidfile; a pgrep whose next 3 lines read /proc/.../environ
passes. Any unscoped pgrep is refused (plain NAME, --exact, --full). Red: 9 of 14 new guards
cases failed; green: all guards behave; the whole tests/*.sh tree passes. No rule needed the
warn-only fallback: each case of the brief is told apart. Runs on mawk here (POSIX awk only).

## 3 xp.sh:654
New tests/proc-lib.sh (running, live_child PARENT NAME: first non-zombie child). Red: the old
expression with a zombie plus a live child gives "ps: process ID list syntax error", rc 1;
selftest red (no helper), green after; a zombie alone stays dead.

## 4 gate-0.8-plugin-runtime
An allow covers the item or statement it stands on, up to the first line at the indent of
its attribute that ends with } or ; (rustfmt layout). Red: 2 of 5 selftest cases (next item,
next statement); green after; the tree passes (three fake-core sites).

## Never-block proof
Real `--depth 1 --single-branch` clone of a bare origin holding only claude/harness (no base
ref), commit-msg hook via pre-commit. Passed: test, docs after a pushed test, fix after pushed
test + docs (B1'), feat, ci, chore, amend, rebase reword, fixup, revert, merge with a fix
subject, plain fix (no base: note), fix with "No test:", unborn HEAD first feat.
Refused, correctly: fix without a test when the base ref resolves (RED_FIRST_BASE set).
Passed: fix with its test staged when the base ref resolves. pre-commit shows the note only
with -v (it hides output of passing hooks).

## Gate (HEAD 4321f0e8)
- --jobs 3 run 1 (18.2 min, cold build): every step PASS but doors: known flake F16 (mc
  screenshot-door and volume-door "missing").
- --jobs 3 runs 2 and 3 (8.7, 8.5 min): doors PASS, pull FAIL after 7 s: "the pane of the
  pulled window is not in front of the face at 1:1 or nearer: 168 (then 193) px wide for 200".
  NEW FLAKE (candidate F21): tests/gate-construct-pull.sh:150 reads the width once, `sleep 1`
  after the drop, so under the load of --jobs 3 the pane has not settled yet. pull alone: 3 of
  3 PASS. Not in this lane; not changed. Fix idea: until_ok on shown_width.
- Serial (no --jobs): "all steps passed" in one run, 27.4 min, pull PASS 54 s.
Not run: tests/ci-local.sh (the guards, selftests and runtime grep it covers ran in the gate).
