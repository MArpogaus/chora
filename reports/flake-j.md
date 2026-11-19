# flake-j (F29, tiling-audit fullscreen bar)

## r1

Lane: cloud, sonnet, branch claude/flake-j at 219d5afe (train 72 merge), no commits. Host 4 CPUs.

Finding: F29 is already fixed on the base. flakes-f 5955b3a9 made the fullscreen act wait (<= 20 s)
until the window covers the output and `work_area[1] == 0` (the bar reserve is gone), and c227d433
made that wait a SECONDS deadline. Both are in 219d5afe (tests/gate-tiling-audit.sh:76-92, :203).
The two hits (train 26, train 54 00:3x) were before 5955b3a9 (08:56 the same day); none since.
No code change was needed, so none was made (no speculative edit, no red to prove).

Reproduction counts (paper-1280x800 fullscreen cell, `tests/under-load.sh --runs 30`, 4 busy loops):

| script | natural load | BAR_STALL=6 (waybar stopped 6 s) |
|---|---|---|
| before the fix (5955b3a9^, the act captures 1 s after `foot -F`) | 0 of 30 fail | 2 of 2 fail: "the bar shows over the window at x 66" |
| base 219d5afe (waits for the bar to hide) | 0 of 30 fail | 0 of 2 fail (pass) |

The busy loops do not slow waybar enough here (a run is 30 s loaded or not); only BAR_STALL reproduces
the race, deterministically. So "0 of 30 after" holds, but "before" is 0 of 30 without BAR_STALL.
The core shows no bar over a fullscreen window for real: with the bar stopped, the capture shows it
until waybar gets the CPU, then the work area drops and the wait ends. No core finding.

Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,gate-lib,tiling-audit` green
(pre-commit 51 s, gate-lib 25 s, build, test 212 s, ci-check 94 s, tiling-audit 298 s; partial run, nothing
pushed by it). tiling-audit (all 4 rounds, 8 cells each) twice more: exit 0, no FAIL line, both times.

Head: 219d5afe (claude/flake-j, unchanged). Suggest closing F29 in the backlog as fixed by flakes-f.
