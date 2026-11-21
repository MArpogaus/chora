
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

## PM check r1 (23:46 UTC; PM not the builder; diff dcb77733..4f07fe6c, 7 files, +339 -35, mostly tests)
- pb-1 MINOR tests/persona/round.sh:128: the smoke path gains a `begin` but `round.sh smoke` was not run; run it in the next persona round (P2-R4-CO) before its verdict.
- pb-2 MINOR tests/persona/check.py:398 verify_closed: windows that opened and closed inside the goal count as the one closed window; correct per brief item 2, note it in tests/persona/README.md L1 next round.
- Checked: baseline() exits without a begin (no fallback to start or the goal before); end() deletes begin.json; L2 prints "not measured (no config)"; L3 skips holders of given-away windows; T7 covered is a note, not lost. tests/test_acts.py change only rewrites two cases of the old contract (acts cell red on them): in scope.
Verdict (computed): no BLOCK or MAJOR: MERGE-OK, head 4f07fe6c, joins train 95.

## r2

- merge of claude/autonomous-dev-session-3w-i33pg7 (618ae513): d72ca975; selftest red at the C2 case as briefed ("no `begin` for this goal").
- fix dad65a95: `check begin` right before the give in the construct case; no other `check end` without a begin in tests/ (probe-check.sh, round.sh already begin). check.py unchanged.
- head: dad65a95. Cells PASS: pre-commit, ci-check, build, doors, acts; tests/persona-selftest.sh twice green.
- gate-lib FAIL (3 checks of merge-gate-selftest: "leaves no log directory" got 4/6, want 0), identical on the untouched session head 618ae513, so not from this lane; not touched.
