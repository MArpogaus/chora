# gate-orphan

## r1

Lane: cloud, branch `claude/gate-orphan` from `fee9c695`. Written 11-20. Head: `437173e1`.

- Red `897aecd8`: selftest case "orphan": at `--jobs 2` a step fails after 1 s, a sibling starts a child that sleeps 60 s; after the gate exits a process with the marker in its command line is alive (got 1, want 0).
- Fix `437173e1`: each step starts under `setsid` (its pid is its group id); on exit `stop_children` sends TERM to each running step's group, waits up to 5 s, sends KILL to groups still alive, then waits. Only groups the gate started are signalled. The exit code of a failed run is kept (1). The selftest's restricted PATH got `setsid` and `ps`.
- Selftest: `ok: orphan: a red step fails the gate`, `ok: orphan: no process of a sibling step is alive after the gate exits`, `ok: merge-gate-selftest`.
- Gate: `tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,gate-lib,build,matrix` PASS (pre-commit 1 s, gate-lib 74 s, ci-check 89 s, build 115 s, matrix 259 s).
- By hand: steps `bad|sleep 45; exit 1` and the real matrix at `--jobs 2`: gate rc 1; 10 s after exit `pgrep -f chora-probe` 0, `pgrep -f gate-handover` 0.
- Limit: a step that starts a program under its own `setsid` leaves the group of the gate.
