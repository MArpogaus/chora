# verify-maj

## r1

Date 2026-11-20. Tree: claude/verify-maj at 88258c87 (session 79c8d2b5 plus launcher-click 741d6bca). No change under crates/, plugins/, services/, flavors/ or tests/. Verdicts below; the PM decides the states.

Step 0: `git fetch --unshallow` done. `cargo check --workspace` exit 0 (1m 10s). `cargo build --workspace` exit 0 (2m 08s). First cargo call installed the 1.99.1 toolchain.

### ti-20 (Apps click shows no launcher within 1.5 s): partly

Not reproduced in the headless gate. Not measured: a visible launcher surface within 1.5 s.

`tests/gate-tiling-apps-click.sh`, 5 runs, all pass. The gate starts fuzzel with `ctl run --launch fuzzel` and does not click the bar's Apps door:

- run 1: `ok: a click on a row of the Apps list starts the program of that row` / EXIT=0
- runs 2 to 5: the same `ok:` line, EXIT=0; runs 3 to 5 first print `gate-tiling-apps-click.sh: waits for /tmp/chora-gate.lock`

The bar door path is `tests/gate-launcher-click.sh` (act.py clicks the Apps door). Run once, EXIT=0:

- `ok: tiling: a click by act.py on the Mid row of the launcher starts mid and not the first row`
- `ok: construct: a click by act.py on the Mid row of the launcher starts mid and not the first row`
- `ok: act.py click and drag move the pointer as a hand does, move and scroll as they say`

Timing, a scratch copy of the gate outside the repo (`launcher_up` polls `pgrep fuzzel`, so this is the process, not pixels): door click to fuzzel process, 3 runs.

- tiling: 253 ms, 265 ms, 518 ms
- construct: 341 ms, 330 ms, 320 ms

Not measured: the visible surface, the 1.5 s threshold, and the owner's real display (the gates run headless).

### co-26 (Super+F1 in a construct face shows no key list): partly

Super+F1 shows the key list in walk mode, after Alt+Escape. Not measured: Super+F1 while a pane has keyboard focus, the flown-in state the finding describes. No cell presses it there.

Construct key-list gate: `tests/gate-construct-keys.sh` (co-2), 3 runs, EXIT=0 each, 14 `ok:` lines per run. Super+F1 line: `ok: after Alt+Escape, Super+F1 shows the help, Escape hides it and d walks; the terminal took none of them` (Super+F1 at tests/gate-construct-keys.sh:119, Alt+Escape at :108).

Closest cell: `tests/gate-construct-layout.sh` help cell (Super+F1 at :478, after the step back to walk at :443, hints `fly-in keys terminal walk` checked at :445). 3 runs, EXIT=0 each, 97 `ok` lines per run. Help lines per run: `ok: help light-1280x720: layout audit: 232 nodes, no finding`, `ok: light-1280x720: Super+slash shows and hides the help`, `ok: help dark-1280x720: layout audit: 232 nodes, no finding`.

### xp-24 (text fields off the XP grid): partly

`tests/gate-xp-fields.sh`, 3 runs, EXIT=0 each. Each run prints:

- `ok: the Run box stands on the grid of 4 px with a label in the text colour`
- `ok: 1x: the edit box of the chat is 28 px high and level with Send`
- `ok: 2x: the edit box of the chat is 28 px high and level with Send`

Measured: the chat edit box, in pixels, at 1x and 2x (`tests/xp-field-rows.py` on a capture; draw.rs:670 sets `pad: Some(4)`).

Not measured:

- Run box, rendered: the gate checks only that the values in `flavors/xp/theme/fuzzel.ini:8-11` (horizontal-pad 12, vertical-pad 12, inner-pad 8, line-height 28) are multiples of 4, and that `prompt` is `000000ff` (`tests/gate-xp-fields.sh`, the `for key in` loop and the `prompt=` check). No pixel check. The Run item is `plugins/chora-xp/src/startmenu.rs:1043`; the box is `plugins/chora-xp/src/launch.rs:34-37`. The xp-polish report notes fuzzel draws no box around its input.
- Search: no text field. `plugins/chora-xp/src/context.rs:351` and `:389` are "Search..." rows, made disabled by `Entry::gray` (`context.rs:173-177`). `plugins/chora-xp/src/help.rs:56` is a help topic, not a field.
- Dialogs: no text field by construction. `plugins/chora-xp/src/dialog.rs:42-57` defines `Kind` as Push, Check, Radio, Tab and Area only. The Date, Display, Effects, Taskbar, Turn Off and Help dialogs have no input control.

Ledger: the rows carry the verdicts in the state cell (the ledger has no evidence column); the state words are unchanged.

Logs: scratchpad directories for this round (not in the repo).

## r2

Date 2026-11-20. Branch claude/verify-maj-r2, commit 5bd1bdfd (88258c87 plus tests/ only). Step 0: `cargo check --workspace` exit 0. Container: fuzzel 1.9.2 (r1 ran 1.11.1; gates assume 1.11.x).

### ti-20 (tests/gate-launcher-click.sh, tiling part): assert in, tree passes it

Assert: baseline capture, act.py click on the Apps door, captures in a row until 1.5 s; the first whose centre (400,150 to 880,650, tests/capture-diff.py) differs from the baseline gives the ms. The bar is excluded (hover repaints it). Time is when the capture finished, so it is an upper bound.
- 5 runs: 371, 379, 337, 373, 386 ms (limit 1500).
- Red shown: `LAUNCHER_MS_MAX=100` gives `FAIL: tiling: no launcher in 1 captures up to 308 ms after the click on the door`.
- Cell: launcher-click.

### xp-24 Run box (tests/gate-xp-fields.sh, tests/xp-runbox-rows.py): assert in, NOT verified green, finding

Assert: after `chora-xp --run-box`, the highlighted row is 28 px high and the first dark pixel of "Open:" is a multiple of 4 px from the inner border edge, at 1x and 2x.
- Real tree: fuzzel 1.9.2 refuses flavors/xp/theme/fuzzel.ini (`[colors].prompt` and `input` are not valid options), no Run box shows. The cell therefore SKIPs here (fuzzel < 1.11), so the gate prints SKIP and passes; in CI a skip fails. Not run on 1.11.1, so unknown there. 3 runs of the real tree: all SKIP.
- Measured with a scratch ini (those two lines dropped), 1.9.2: highlighted row 37.0 px (1x), 37.5 px (2x); text inset 11.0 px (1x), 12.0 px (2x); row left inset 4.0 / 5.0 px. Neither 28 nor on the grid at 1x. Whether this holds on 1.11.1 is open: the PM should have the owner run `tests/gate-xp-fields.sh` where fuzzel 1.11.1 is. If red there, it is a product finding (line-height 28 plus inner-pad 8 gives ~37), not fixed here.
- Red shown (scratch ini): `FAIL: 1x: the row of the Run box is 37.0 px high, not 28`; with RUNBOX_ROW=37: `FAIL: 1x: the text of the Run box stands 11.0 px from the border, off the grid of 4 px`.
- Cell: xp-fields. Not asked: the 3 runs on 1.11.1.

### Gate

`tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,launcher-click,xp-fields,tiling-look`: pre-commit, build, test, clippy, xp-fields PASS (xp-fields with the SKIP above); launcher-click FAIL; ci-check and tiling-look did not run after that. Not green.
launcher-click FAIL is pre-existing and environmental: `a click on the Mid row ... did not start mid` after my new assert printed ok. The unmodified gate (git stash) fails the same way 3 of 3 here; r1 passed it on fuzzel 1.11.1. The cell comments say 1.9.x differs in pointer handling (first motion starts the highlight).
