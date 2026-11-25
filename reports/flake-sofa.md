
## r1

Base 62a04be0, head 3d555e4f (claude/flake-sofa-r1, 2 commits: red c8107a4b, fix 3d555e4f). Diffstat: tests/gate-mc-sofa.sh, 4 insertions.

Race: the core reports the text "Volume 50%" visible with rect [0,0,87,28] before the layout has placed the door (hit node at [988,672,87,28]); the cell moves the pointer to that rect, so the pointer is off the door and the wheel reaches no volume door (wpctl calls: two get-volume, no set-volume). Log evidence from a red run: `door=[0.0,0.0,87.0,28.0] now=[... "HEADLESS-1":[988,672,87,28]]`. Fix: the cell waits until the door text equals its hit rect and stands off the origin (until_ok on a query, no sleep). The check is as strict.

Counts: before (62a04be0, CELLS='volume-wheel|keylist', 10 runs): volume-wheel 2 red, keylist 0 red (plus 1 more volume-wheel red in a 25-run debug series, same cause). After: volume-wheel 0/20 red, keylist 0/20 red; whole mc-sofa step 3x green dark. Red commit feeds the unplaced rect: red every time.

keylist red (rpx1-9) did not reproduce in 55 runs here; not a gate-script race seen.

Light theme, full mc-sofa run: `rooms-pill` red 3 of 3 ("the host does not draw the state word at the new place: [1120.0,29.0,80.0,26.0]"). Same on the base 62a04be0 (full light run red on rooms-pill), green when the cell runs alone; so pre-existing and order-dependent, not caused by this change, out of scope. Needs its own lane.

Gate: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2`: all steps PASS ("all steps passed; no --push"), mc-sofa PASS 79s, EXIT 0. pgrep 'chora --headless' empty.

## PM review r1 (2026-11-25 18:33 UTC; diff 62a04be0..3d555e4f, 1 file, +4)

- Red first: c8107a4b feeds the cell the unplaced rect [0,0,w,h] (red every time), 3d555e4f replaces it with the wait. A pinned race, not a red on unchanged code; accepted for a harness flake (the race is shown in the logged rect).
- Fix: until_ok on text rect == hit rect and off origin; no sleep, no product change; check as strict as before. Counts 2/10 red -> 0/20.
- rfs1-1 (MINOR, core): the core reports a text visible at [0,0,w,h] before layout places it. If a frame draws it there, a person sees a flash at the corner. Backlog: check one frame capture after a door appears.
- rfs1-2 (MAJOR for rc 3, harness): mc-sofa light, rooms-pill red 3/3 in the full run (also on 62a04be0), green alone: order-dependent. FLAKE-SOFA r2.
- Verdict: MERGE-OK 3d555e4f (train144).
