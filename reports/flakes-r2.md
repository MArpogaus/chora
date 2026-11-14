# flakes r2 (rc 3, rc 9): report

Lane: cloud, branch `claude/flakes`, head `55c527d4` (pushed), on the session branch at
`498d9dea`; the rebase was a no-op. Written 11-14 20:15 UTC. Load means 8 busy shell loops on
4 cores (load average 9): `stress-ng` is not installed in the container and a lane installs no
packages. All logs are in the container under `/home/user/logs` and do not outlive it.

## Items

| Item | Result | Commits |
|---|---|---|
| 1. F14 construct deaf while xp is stopped | fixed | red `1156cbb2`, fix `41c1424b` |
| 2. F13 click during a step of the cube | fixed | red `4971476a`, fix `57b1846a` |
| 3. cube flick under load | fixed | red `52bebe0a`, fix `55c527d4` |
| 3. "the window does not float during a turn" | not reproduced (0 of 28 loaded runs) | none |
| 4. F10/F11 | fixed in r1 on this branch (`df768e4b`, `7399d792`), not touched | none |
| 4. F2, F3, C2 | already in the session branch (`cdaa648b`, `9e0842e0`, `wm::child_command`) | none |
| 4. F7, A5, A17, F4 to F6 | not started: no context left after the gates | none |

**F14.** Cause: after the step in, `Construct::land` asked every floating face in one round
(two faces, 400 ms wait each), and the next round was due 500 ms after the *start* of the
round, so it began at once. The construct handled one event per 800 ms. Evidence: with xp
stopped, six `construct.list` calls took 2462 to 2553 ms each; the gate cell failed in 4 of 9
runs without load. Fix: the first face without an answer ends the round of the landing too,
and the next round starts 500 ms after the wait. No timeout was raised; the cell now makes
three timed calls. Green: 16 to 485 ms per call in 19 of 19 loaded runs that reached the cell.

**F13.** Cause: `Host::press` took a press only while the cube rested, so a click during the
380 ms of the step back did nothing. Red: unit test `a_click_during_a_step_of_the_cube_counts`
and the gate cell "a click during the step back is lost". Fix: a press counts unless the cube
turns or the hand holds it. Green: 22 of 22 loaded runs.

**Flick.** Cause: the cube timed the hand by the moment at which the construct handled each
move. Under load the three injected moves were handled 60 to 90 ms apart; the speed came out as
0 or 1.49 widths a second (a flick needs 1.5). Red: the cell failed in 3 of 10 and 3 of 6
loaded runs. Fix: `pointer.moved` and `pointer.released` carry `time`, the time of the input in
ms, and the cube takes the speed from it (docs/api.md updated). Green: 12 of 12 loaded runs of
the whole cube gate. **Deviation:** this adds a field to two events of the API.

## Gates (full `tests/merge-gate.sh` on `55c527d4`, run directories under `/home/user/logs/gates`)

| Run | Result | Directory |
|---|---|---|
| 1 | FAIL at `xp` | `merge-gate.V2WSlC` |
| 2 | PASS, 23 steps | `merge-gate.fkH9mo` |
| 3 | FAIL at `layout` | `merge-gate.9tbKBs` |
| 4 | PASS, 23 steps | `merge-gate.uqG3RY` |
| 5 | PASS, 23 steps | `merge-gate.7KiCRO` |
| 6 | PASS, 23 steps | `merge-gate.zyf5sc` |

Runs 4, 5 and 6 are three passes in a row. Four of six runs passed, so the gate is not yet
free of flakes: the two failures are new findings, neither in the cube step.

## New findings (not fixed, for backlog.md)

1. `tests/xp.sh` step 3, "timeout: pixel_is 408 57" (gate 1; 3 of 3 passes alone). The two
   foot windows take their cascade place in the order of their first commit
   (`Wm::resized`), but they stack and take the focus in the order of their creation. When s2
   commits first, it sits at 24,24 on top and covers the title bar of s1 at 54,54. This is a
   race in xp, not only in the test. Evidence: `/home/user/logs/xp-step3-failed/windows.png`.
2. `tests/gate-construct-layout.sh`, cell `light-2560x1440@2`: "Alt+Escape during the fly-in
   did not step back", then "a key typed in use mode did not reach the terminal: d" (gate 3;
   passed in gates 2, 4, 5, 6). Panes only, no cube. Not analysed. Log:
   `/home/user/logs/layout-failed-gate3.log`.

## Open

- The construct still waits up to 400 ms in one call for a window manager that does not
  answer, once per round. A float request without a wait would remove that; it is a larger
  change than the release scope allows.
- I edited the gate script once while a loaded loop ran it; that loop was stopped and its
  results discarded, and the series above were run again.
- No independent review of `claude/flakes` yet (conventions.md 5).
