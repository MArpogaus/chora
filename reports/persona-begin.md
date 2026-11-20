
## r1

Branch claude/persona-begin, head 4f07fe6c (base dcb77733). Red commits first, then fixes.

| Item | Red | Fix | Selftest cases |
|---|---|---|---|
| 1 one begin per goal | 16639b80 | cf7eba2b | test_check BeginPerTask (no begin = no verdict; end uses the begin up; goal two never reuses goal one); persona-selftest "check.py end holds each goal to its own begin" |
| 2 L1 own begin, "closed" | 10b668e5 | 82d16ac8 | Closed (3) |
| 3 L2 no config | 10b668e5 | 82d16ac8 | NoConfig (3): "task L2 not measured (no config)" |
| 4 L3 given away | 10b668e5 | 82d16ac8 | GivenAway (3) |
| 5 T7 covered | 10b668e5 | 82d16ac8 | Covered (3): pass plus "minor T7 covered: s2 under s1" |

Items 2 to 5 share one red and one fix commit. Also touched: round.sh smoke and probe-check.sh take a begin before end; tests/test_acts.py (outside tests/persona, 4f07fe6c) updated two cases that encoded the old contract (L1 "gone", baseline falling back to start); the gate cell acts was red on them.

Gate: pre-commit, ci-check, build, gate-lib, doors PASS; acts PASS (after 4f07fe6c); tests/persona-selftest.sh PASS; tests/persona/smoke.sh PASS; probe-check.sh PASS. round.sh smoke not run.
Assumptions: L1 = exactly one window closed (open at begin or seen during the goal, gone at end); "covered" = a later toplevel in root order whose rect contains the given window's rect; L2 config = any non-empty file under PERSONA_DIR/config or xp.yaml.
