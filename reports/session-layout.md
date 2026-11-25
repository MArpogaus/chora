# session-layout

## r1

Lane `claude/session-layout-r1`, brief rtd4-1 (tiling-doors PM review r4).

- Base: `62a04be03`. Head: `691f0c8e2`.
- Commits: red `d08aa02a3` (persona-selftest: the claims of tiling and xp
  must not overlap; fails on the base), fix `691f0c8e2` (session.sh).
- Diffstat: `tests/persona-selftest.sh` +9, `tests/persona/README.md` +5 -2,
  `tests/persona/session.sh` +17 -5 (3 files, 31 insertions, 7 deletions).

Layout before (`media/2026-11-25/session-layout-r1/before-tiling-xp.png`):
one output 1280x800. Claims: tiling `[0,40,1280,730]`, xp `[0,40,1280,760]`.
xp's desktop and taskbar cover tag 1 and every empty tag of tiling.

Layout after: two outputs, `HEADLESS-1` (tiling, `--output HEADLESS-1`,
claim `[0,40,1280,760]`) and `HEADLESS-2` at `+1280+0` (xp,
`--output HEADLESS-2`, claim `[1280,0,1280,800]`), as D55 step 2 and
`docs/configuration.md` ("each window manager takes its own outputs").
Tiling tag 1 is its own empty tag with its bar (`after-tiling-tag1.png`);
xp is on its own output (`after-xp.png`, `choractl capture --output
HEADLESS-2`). Single-look sessions are unchanged (`look_output` is empty).

Smoke: `round.sh start/stop` for tiling and tiling+xp end clean, with
calls.log, evidence and shots kept. `round.sh smoke` fails at `check.py end
T7` for tiling ("no window changed look") with a single look, on a path this
change leaves untouched: the smoke predates the T7 meaning, not caused here.
Its first run left one headless session behind; stopped by its env file.

Gate: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2`, no step filter, no
red cell: `e2e PASS 266s` ... `all steps passed; no --push, nothing is pushed`.
