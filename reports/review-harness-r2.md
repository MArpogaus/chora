# review harness r2 (claude/harness f8ab718a)
Independent, read-only. Probes in /tmp/claude-0/r2a: bare origin, a
`--depth 1 --single-branch` clone and a clone with no remote, commit-msg hook
= tests/red-first.sh, no base ref. merge-gate-selftest, guards.sh and
gate-lib-selftest: PASS. Grep backstop of gate-0.8-plugin-runtime: tree clean.

## BLOCK
- B1' tests/red-first.sh:39-49 (branch_verdict fallback): without the base ref
  the hook REFUSES when the unpushed commits hold no test, even if the test is
  in a pushed commit of the same branch. Reproduced in a shallow single-branch
  clone: `test(y): red`, push, `docs(y): note`, `fix(y): green` ->
  "RED-FIRST MISSING: fix(y): green". The same commit with RED_FIRST_BASE set
  passes, and the merge gate (base..HEAD) accepts it: a valid commit blocked,
  only because of push state (workers are told to push often). Fix: in the
  fallback never refuse; pass with the note when the unpushed range has no
  test (the gate is the backstop), or judge `@{u}`'s merge-base too. Earliest
  gate: the hook cases in tests/merge-gate-selftest.sh (953aad46 has no case
  "test pushed, then an unpushed commit, then fix").
## MAJOR
- none. M1 is closed: xp.sh:652-656 checks own pids (zombie = dead),
  agent/nested use -P of their core, gate-mc-paths uses session_pids
  (gate-lib.sh:93-101, filters by XDG_RUNTIME_DIR), tree passes the new hook.

## MINOR
- m1 tests/no-foreign-kill.sh:20-22 false positives (blocks valid tests/*.sh):
  `pgrep -P"$pid" -x foot` (attached -P), the /proc environ XDG_RUNTIME_DIR
  filter loop that session_pids itself uses, `pkill -F pidfile`, an echo or
  heredoc that names pidof/killall. Reword works, the message names
  session_pids, so MINOR. False negatives: `pgrep foot` (no -x/-f still
  matches by name), `--exact`/`--full`, a `\` continuation. Scope is right:
  files ^tests/.*\.sh$ only; dist/ and flavors/ (waybar scripts) have no
  pgrep and are not hit; comment lines are skipped.
- m2 tests/xp.sh:654: `running "$(pgrep -P "$chora_pid" -x chora-xp)"` with two
  children (a restarted plugin beside its zombie) passes "1\n2" to ps/kill ->
  "chora-xp died" though one runs. Take the first non-zombie pid.
- m3 tests/red-first.sh:36: unborn HEAD (orphan branch, fresh repo) returns 1,
  so a first `feat: init` without a staged test is refused; return 0.
- m4 tests/gate-0.8-plugin-runtime.sh:24-28: an allow line opens a 40-line
  window for any later item of the file, not the allowed one; clippy stays
  exact, so only the backstop is loose.

## COSMETIC
- c1 f8ab718a: a docs commit carries the agent.sh step 18 change (M1 fix:
  `pgrep -s 0 -x foot` filtered by XDG_RUNTIME_DIR and WAYLAND_DISPLAY).
  Acceptable without a split: the change is correct (foots of choractl run
  are in session 0 since no core code calls setsid; the filter is the M1
  scoping), red-first does not judge it (no test needed for a script
  scoping fix), and a revert+recommit would add noise. Name it in the
  merge commit body. CI-only, not run beyond step 2 here.

## Checked and fine
Hook, no base ref, shallow single-branch and no-remote: test then fix, amend,
rebase reword, fixup, revert, "No test:", fix with all commits pushed (note),
fix without any test unpushed (refused, correct), detached HEAD test+fix,
merge --no-ff with a fix subject, conflict merge commit (MERGE_HEAD), squash
merge, branch behind its upstream. m2 r1: connect_at and Runtime::with in
plugins/clippy.toml, #[allow(..., reason)] on the two #[cfg(test)] sites,
grep matches conn::connect/runtime::with. m3 r1: clippy-config-sync diffs the
settings both ways (guards cases). m4 r1: 2d-guard in serial_steps
(merge-gate.sh:285), drains ci-check before it starts; selftest case.

Verdict: FIX (B1')
