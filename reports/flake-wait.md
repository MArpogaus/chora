# flake-wait

## r1

Branch claude/flake-wait, head 176938d1 (base 6b01c976, already the head of
autonomous-dev-session-3w-i33pg7, so nothing to merge).

| commit | what |
| --- | --- |
| 5e1df196 | red: tests/gate-leak-selftest.sh (gate-env files and wallpaper.sh before/after one gate script) |
| e7df5cb7 | fix: session_start's mktemp env file joins tmp_dirs (removed on a green gate); wallpaper.sh sees its socket gone for an absolute WAYLAND_DISPLAY too |
| 705f7664 | red: PULL_RACE=1 in gate-construct-pull.sh (first read of the window 300 px low) |
| 3f9921cc | fix: wait until two reads of the window place agree, then press |
| b94d86bd | red: GIVE_RACE=1 in gate-start-tap.sh (foot stopped when its surface node appears) |
| 176938d1 | fix: give cell waits on the pointer_move to the node point itself, then presses |

### Causes
- FLK-PULL-DRAG (gate-construct-pull.sh:230): `read -r surface _ _ x y <<<"$(probe)"` then
  the press at `x + 90, y - 12`. The step waits only for the title node to exist, not for
  the window to hold its place (cube.open, add-output and the wobbly effect relayout the
  face). A press at a stale place misses the title bar, so no grab, and the window never
  moves. NOT observed directly: inferred from the failure text and reproduced by a stale read.
- FLK-GIVE-POINT (gate-start-tap.sh:168 cell give): foot_of_tiling waits for the surface node and
  owner tiling, not for the node to draw (inject.rs node_point needs draws_something and
  shows_point). The inject at [100, 60] ran on a node with no box yet:
  `the node "s1" shows no point [100, 60]`.

### Red and green
- give, GIVE_RACE=1: red `choractl: the node "s1" shows no point [100, 60] (code 1)`; green after the fix (`cell give pass`, also without the switch).
- pull, PULL_RACE=1: red `FAIL: the drag of the title bar does not move the window: s1 xp VIRTUAL-1 28 49`; green after the fix with and without the switch.
- leak: red `FAIL: /tmp/gate-env.* grew from 0 to 1`; green `ok: gate-env files 0 -> 0, wallpaper.sh 0 -> 0`.

### Leaks
- gate-env: confirmed and fixed (a green gate removes its file; a failed one keeps it for logs).
- wallpaper.sh: no growth seen after one gate (tiling-empty flag); the unfixed script did
  never end on an absolute WAYLAND_DISPLAY (`/*) return 1`), which is the likely 70-process leak
  under SIGKILL; fixed, not reproduced as a leak. Counting uses ps/awk (pgrep by name is barred by the hook).
- gate-leak-selftest.sh is NOT wired into the gate-lib cell: with --jobs 2 another gate's
  env file would count as growth. Run it alone.

### Not done
cell_drag and cell_return in gate-start-tap.sh use the same inject at a node point; same wait is not applied there (no flake seen).

### 5x under load
`merge-gate.sh --only ci-check,pull,matrix --jobs 2` x5 under `cargo build --release` (target /tmp/load-target): exits 0,0,0,0,0 (5/5, 0 red).

### Full gate
`tests/merge-gate.sh --jobs 2`: 51 PASS, 1 FAIL. Foreign red: mc-scale, 22s,
`FAIL: settings.set of the write time beside 300 panels takes 5243 ms, more than 5000 ms`
(a timing threshold in a cell this lane did not touch; not rerun). EXIT 1.

## PM review r1 (176938d1)

Base 6b01c976..176938d1: 5 files +48 -5, 6 commits (PM review: under 200 lines, PM not the builder). Red before green for all three: 5e1df196 -> e7df5cb7 (leaks), 705f7664 -> 3f9921cc (pull: until_ok on a place that holds 0.3 s, then the press at the read place), b94d86bd -> 176938d1 (give: until_ok on the node point, then the click). wallpaper.sh: an absolute WAYLAND_DISPLAY now ends the loop when its socket goes. gate-lib: the mktemp env file joins tmp_dirs.
Findings:
- rw-1 MINOR tests/gate-start-tap.sh hold_foot: `until ...; do :; done` spins with no sleep and no bound (only under GIVE_RACE). Fix: until_ok with a bound.
- rw-2 MINOR tests/gate-leak-selftest.sh: no step of merge-gate runs it (D91: a gate script joins merge-gate). Fix: a step or a call from gate-tiling-empty.
- Note: the pull cause is inferred (no capture of a moving window at the press); the series in train and haiku series 5 decide.
Gate: 51 PASS, 1 foreign red mc-scale (FLK-MC-SCALE, 5243 ms; lane mc-scale running). Verdict computed: 0 BLOCK, 0 MAJOR: MERGE-OK; joins t110. Worker archived.
