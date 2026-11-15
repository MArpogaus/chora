# review harness r1 (claude/harness f3bab766, base 8bcbdc73)
Independent, read-only. Probes: scratch clone with the pre-commit 4.7.0
commit-msg hook installed; clippy on chora-notify with planted calls;
tests/merge-gate-selftest.sh PASS (shellcheck is not installed here).

## BLOCK
- B1 tests/red-first.sh:32-34 (branch_has_test): when RED_FIRST_BASE (default
  origin/claude/autonomous-dev-session-3w-i33pg7, hard-coded) does not
  resolve, only staged files count, so the very workflow the rule asks for
  (`test(x): red`, then `fix(x): green`) is REFUSED. Reproduced without the
  ref: "RED-FIRST MISSING: fix(z): green2"; `commit --amend -m "fix..."` and a
  rebase `reword` of such a fix fail too. The ref is missing in a shallow or
  single-branch clone (create_session clones depth 50, which implies
  --single-branch), a fork, the owner's clone, and everywhere once the session
  branch is renamed. Breaks friction 44. Fix: fail open with a note when the
  base does not resolve (the gate is the backstop), or fall back to
  `git rev-list HEAD --not --remotes`. Earliest gate: the hook checks in
  tests/merge-gate-selftest.sh:264-299 (all pass RED_FIRST_BASE=base; one
  case without the ref shows it).
## MAJOR
- M1 tests/merge-gate.sh:282-294 with tests/xp.sh:651-653: xp.sh tests its
  plugins alive with global `pgrep -x chora-xp|chora-notify|chora-assistant`.
  Under --jobs 3, xp runs beside matrix, virtual and xp-faces, which start the
  same programs, so a crashed plugin of the xp step reads as alive: a check
  that lies (friction 47). The comment at merge-gate.sh:40-44 says each gate
  session is separate; this check is not. Fix: kill -0 the pids xp.sh
  started (or pgrep -P), or put xp in serial_steps. Earliest gate: the
  builder's shared-resource audit of the parallel steps (brief item 3,
  "separate sockets"); the no-process-by-name hook matches only `chora`.

## MINOR
- m1 tests/red-first.sh:43-46: merges are judged by subject; the gate skips
  them. `git merge --no-ff side -m "fix(m): x"` is refused (reproduced). Train
  merges say "Merge ..." so rare; return 0 when MERGE_HEAD exists.
- m2 plugins/clippy.toml:13-18: `Conn::connect_at` (chora-kit conn.rs:40) and
  `Runtime::with` stay open, and the grep backstop misses connect_at too.
  Planted alias (`use Conn as C`), fn pointer and local macro: all 3 caught;
  connect_at: not. Two test sites use it (chora-tiling main.rs:2316, chora-mc
  testing.rs:129): disallow with `#[allow]` there, or name the gap.
- m3 .pre-commit-config.yaml:20 clippy-config-sync is one way: a setting
  removed from clippy.toml stays in plugins/clippy.toml, hook green
  (reproduced). Changed values and one-file changes are caught.
- m4 tests/merge-gate.sh:155: 2d-guard runs a cold `cargo build` of its base
  (2d-guard.sh:286-290) beside timing-sensitive gates under --jobs (own target
  dir, so no write clash); the CPU load can feed the tiling-audit flake.

## COSMETIC
- c1 CONTRIBUTING.md:239 over 72 columns; "about half" vs 27.5 to ~11 min.
## Checked and fine
Hook: docs/ci/chore/test/refactor/Revert/fixup! pass; "No test:" passes; a
test commit earlier but not the parent passes (with the ref); `commit -a` and
`commit PATH` see the staged test; `-v` and comment lines keep the subject;
plain rebase and cherry-pick do not run it; no network or cargo, 0.8 s for
all commit-msg hooks. claude/session-log has its own config: hook not there.
Missing red-first.sh fails closed, but config and script share a tree.
--jobs: one log per step, a failure exits 1 and stops the others (their EXIT
traps ran), no push; push only after the drain; one lock, inherited by the
children; default 1; cargo steps serial; mktemp runtime dirs and sockets.

Verdict: FIX (B1, M1)
