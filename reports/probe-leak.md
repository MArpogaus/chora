# probe-leak (sh-30, F5)

## r1

Branch `claude/probe-leak` from d81c8293, head f5ac3500, pushed. Gate (`--jobs 2 --only pre-commit,gate-lib,ci-check,build,test,clippy,keys,walk,placed,out,agent`): all 11 cells PASS, one run, no rerun. `pgrep -f chora-probe` is empty after it.

| item | red | fix |
|---|---|---|
| 1 probe exits when its compositor goes away | 8e2f730d `tests/gate-probe-hangup.sh` (in the `keys` cell): headless chora + tiling, `window --report`, the session ends; the probe still ran 2 s later, 312 CPU ticks in the next second | 567464b5 `tests/probe/src/main.rs`: `dispatch_until` records a dispatch, flush or read error in `Probe.lost`; the `window` and `clip` loops return an error instead of spinning. A `WouldBlock` read is no error (the first try treated it as one and ended the probe at start) |
| 2 the gate leaves no probe | ac681d1a `tests/gate-lib-selftest.sh`: a stub cell starts a direct and an orphan chora-probe and ends with `exit 0` or `fail`; all 4 probes outlived it, and the EXIT trap hung in `wait` until the 15 s timeout; a probe of another runtime dir must survive | f5ac3500 `tests/gate-lib.sh`: each gate exports `CHORA_GATE_PID`; `gate_cleanup` kills the chora-probe processes that carry it and those of the session runtime dir (`gate_probes`), before its `wait` |

### Item 3: leaked probes in this container

- Step 0 (before any work): 0.
- Base d81c8293, cells keys, walk, placed, out run one by one (scratch worktree, base probe binary and base tests/): all pass, `pgrep -x chora-probe` is 0 after each. `agent` at base: pass, 1 probe seen right after, gone a moment later.
- So no leak here from passing cells; the PM-host leaks (4 probes, 1.5-5.3 h) came from trains that failed or were killed mid-cell, with the worktree removed. The red tests above are the reproduction: with the base code 2 of 2 direct and 2 of 2 orphan probes outlived a cell, and a probe spun at 312 ticks/s once its compositor was gone.
- After the fix: 0 after the gate.

### Notes

- Other scripts that start probes without gate-lib (agent.sh, nested.sh, xp.sh, 2d-guard.sh, doors/*, persona/*) are not touched; with item 1 a probe in `window` or `clip` mode ends when the sway or chora host dies, and the other modes are bounded by their own timeouts. agent.sh has its own trap and does not source gate-lib.
- `merge-gate.sh` `keys` cell now also runs `tests/gate-probe-hangup.sh`.
