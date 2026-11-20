## r1
Lane: cloud, branch claude/persona-check from fee9c695. Step 0 green (`cargo check --workspace`). No product code; a red case first, then the fix, in tests/persona (selftest: tests/persona/test_check.py, run by tests/persona-selftest.sh).

| Item | red sha | fix sha |
|---|---|---|
| 1 T4:clock | 610ee6f7 | e924eb52 |
| 2 I1/I3 on the xp child | 84705542 | aafc2501 |
| 3 begin per goal | 003b24bd | f6b21bc2 |
| 4 scene state in the line | b8e7d6ab | e3763f69 |

1. T4:clock: the checker looked for a node id with "clock"; xp draws the clock as the text of `xp-tray-OUTPUT` (`meta.text` "3:27 PM"), which is what the cell clock-door reads. The contract now takes a time in the text of that tray (a node id with "clock" still counts). Green on an xp tray, red on a tray without a time and on no node.
2. I1/I3: they apply to the xp-owned window. The construct lists such a window as a *view* with owner xp (state.rs: `Content::Manager`) and the core focuses it for xp; the checker read the state of the session's looks only (construct), so xp was "no look" and the window was in no `windows` list. Counted right, not excluded: `read_states` adds the state of xp to a construct session while the child is connected, and the invariants are unchanged. Pinned: construct alone breaks I1/I3 on the window (the old read); with xp's state both hold; a window missing from xp's `windows` stays a breach. Live: a construct session with `cube.open`, two actions, `check.py end T7` gives 0 breaches (I did not drive a pull-out or step into the xp window; the lead of r1b/r2 is covered by the pinned case, not re-seen live).
3. begin: round.sh gets `begin NAME`, the README says once per goal. A driver that forgets cannot leak: `end` leaves its end state as the begin of the next goal (pinned: a window left by goal 1 is no new window for goal 2). Cost: a second `end` of the same goal is held against the first one's end state.
4. Scene state: each `task` line ends with `[scene: cube stood back|in a face|closed, face N, pane room yes|no, walk yes|no]` for the construct, read from `cube.list` (mode navigate = stood back), a visible `construct-pane-*` node and `busy` "walk"; other looks print nothing. Whether "pane room" is exactly the node set that shows shot 012 of r2 is not checked on a capture; the `construct-pane-*` read is my reading of the scene ids.

Gate: `tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,gate-lib,doors` all PASS (doors 365 s); `tests/persona/smoke.sh` PASS; `tests/persona-selftest.sh` all ok. No red cell. Head e3763f69.

## PM review r1 (2026-11-20 15:59 UTC)
Base fee9c695..e3763f69, 5 files, +387/-12 (282 selftest). Four red-first pairs (610ee6f7, 84705542, 003b24bd, b8e7d6ab first). Not a merge-gate step; persona-selftest runs test_check.py (19 cases).
- Item 1 clock: clock_nodes reads xp-tray-* text with a time, as clock-door does. Closed.
- Item 2 I1/I3: read_states adds the xp child's state in a construct session. Closed by planted cases; live only cube.open + 2 moves (lane caveat): co persona r3 is the live check.
- Item 3 begin: round.sh begin; end saves its state as the next begin. Closed.
- Item 4 scene: [scene: ...] from cube.list and construct-pane-* nodes. Closed.
- pr1-1 NIT: scene_state counts a pane node without `visible` as visible (n.get("visible", True)). Known issue.
Verdict (computed): no open MAJOR -> MERGE-OK. Joins t91.
