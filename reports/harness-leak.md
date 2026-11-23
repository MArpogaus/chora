# harness-leak

## r1

Branch claude/harness-leak, head 7982d549 (red 8a8f400c, fix 8cdbb31a, refusal 7982d549); base 6b01c976, already equal to fresh origin/claude/autonomous-dev-session-3w-i33pg7 (merge was a no-op).

### Leaking exits
The compositor (`chora --headless`) leads its own session (`setsid` in tests/headless-session.sh:70ish), so a kill of the group of the cell or step never reaches it; the only thing that ended it was the TERM trap of headless-session.sh (cleanup). A SIGKILL runs no trap:
- cell killed with SIGKILL (timeout -k, OOM): tests/gate-lib.sh:121 starts headless-session.sh in the background; it survives its parent and the core with it. ps: `29253 29253 /bin/bash .../ghost/chora --headless 800x600 --`
- group SIGKILL (merge-gate.sh stop_children, the KILL after its 5 s grace, line ~451): kills headless-session.sh, not the core's own session. ps: `29346 29346 /bin/bash .../ghost/chora --headless 800x600 --`
Not leaking (checked): a failing cell (`fail`), `fail` inside a subshell, a timeout TERM (the group TERM reaches headless-session.sh).

### Red
`bash tests/gate-lib-selftest.sh` on 8a8f400c: `leak cases: 3/5 leave no compositor` with the two FAIL lines above (kill, group).

### Fix
- tests/headless-session.sh: a watcher in a session of its own (`tail --pid` on the script and on its parent, no poll) ends the core's group (TERM, then KILL) and removes the temp dir when either is gone without a trap.
- tests/gate-lib.sh: session_start also traps INT and TERM (exit 130/143, so the EXIT trap runs).
- tests/merge-gate.sh: after the gate lock it refuses to start, naming pid and command line, while a `chora --headless` of an earlier run lives (scan of /proc; skipped under MERGE_GATE_NESTED).
Refusal checked by hand with a stand-in process: `merge-gate: a chora --headless of an earlier run lives ...  pid 30448: /x/chora --headless 800x600 100`.

### Green
`bash tests/gate-lib-selftest.sh` on 8cdbb31a: `leak cases: 5/5 leave no compositor`, `ok: gate-lib-selftest`.

### Gate
Full `tests/merge-gate.sh --jobs 2` (no --only) on 7982d549: `all steps passed; no --push, nothing is pushed`, `EXIT 0`; 57 PASS, 0 FAIL, 0 SKIP. `pgrep -fa 'chora --headless'` after it: empty. cube cell PASS (119 s).
