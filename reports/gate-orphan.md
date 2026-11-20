# gate-orphan

## r1

Lane: cloud, branch `claude/gate-orphan` from `fee9c695`. Written 11-20. Head: `437173e1`.

- Red `897aecd8`: selftest case "orphan": at `--jobs 2` a step fails after 1 s, a sibling starts a child that sleeps 60 s; after the gate exits a process with the marker in its command line is alive (got 1, want 0).
- Fix `437173e1`: each step starts under `setsid` (its pid is its group id); on exit `stop_children` sends TERM to each running step's group, waits up to 5 s, sends KILL to groups still alive, then waits. Only groups the gate started are signalled. The exit code of a failed run is kept (1). The selftest's restricted PATH got `setsid` and `ps`.
- Selftest: `ok: orphan: a red step fails the gate`, `ok: orphan: no process of a sibling step is alive after the gate exits`, `ok: merge-gate-selftest`.
- Gate: `tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,gate-lib,build,matrix` PASS (pre-commit 1 s, gate-lib 74 s, ci-check 89 s, build 115 s, matrix 259 s).
- By hand: steps `bad|sleep 45; exit 1` and the real matrix at `--jobs 2`: gate rc 1; 10 s after exit `pgrep -f chora-probe` 0, `pgrep -f gate-handover` 0.
- Limit: a step that starts a program under its own `setsid` leaves the group of the gate.

## PM review r1 (16:17)
Base fee9c695..437173e1, 2 files +31 -5; red 897aecd8 first (orphan selftest case).
- Each step runs as `exec setsid bash -c`; the gate runs without job control, so the step subshell is no group leader and setsid execs in place: $! is the group id. stop_children sends TERM to each group, polls 5 s, KILLs the living groups, waits. The EXIT and INT/TERM traps still call it, so a Ctrl-C on the gate still ends the steps although they left its group.
- pr1-1 NIT: under `set -m` (an interactive shell sourcing the gate) setsid would fork and $! would not be the group; not a supported use; a comment would say so.
- pr1-2 NIT: a step that calls setsid itself escapes (lane names it); none does today.
Verdict: MERGE-OK; harness branch: its own train (t91) after t90 gate-silent, then the batch.
