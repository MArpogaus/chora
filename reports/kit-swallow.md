# kit-swallow: Super chords that reach no client

## r1

Lane kit-swallow-r1 (sonnet, cloud), branch claude/kit-swallow-r1. Finding rpt2-1 (MAJOR, rc1 bar "no stray keys into clients").

- Base: 4d8bbe36. Head: 02e745c3.
- Red commit: b279bd09 (unit test for the full modifier set in chora-kit, and the cell tests/gate-swallow-chords.sh, which typed `9xX` into foot of tiling on 4d8bbe36). A second red, 752af69f, covers a regression that the widened list caused (see below).
- Diffstat (9 files, +252 -28): chora-kit swallow.rs (list, tests), chora-kit runtime.rs (one field `pub(crate)`), core ipc.rs (grab limit and its test), docs/api.md and docs/architecture.md (the limit), chora-xp startup.rs (order of two calls), tests/gate-swallow-chords.sh (new), tests/gate-construct-cube.sh (co-25 chords), tests/merge-gate.sh (the cell in step tiling-windows).

### What changed

1. `WITH` in swallow.rs holds none and every combination of Shift, Ctrl and Alt (8 entries); with Ctrl+Alt the keys of the safety layer (Escape, BackSpace, i, F1 to F12) are left out, as the core refuses to grab them. `chords("Super")` is 586 chords.
2. The core allowed 256 grabs for a connection: 586 swallowed chords beside the bindings of a look do not fit, and a refused swallow is silent. `MAX_GRABS` is 1024 (ipc.rs, docs). Scope note: this touches the core and the docs, beyond the kit; the fix needs it.
3. The give keys (Super+Alt+1 to 4) were swallowed by the look that holds them: the swallow of the widened list took them before `regrab_give_keys` could. `free_chords` skips them for a plugin with give keys (red 752af69f, fix 1c57c3d5). xp set its give keys after its first swallow, so xp lost them (the matrix gate, holder xp, found it: "the give keys were served by [tiling]"); xp sets them first now (02e745c3).

### Cell

Super+Alt+9, Super+Alt+x, Super+Ctrl+Shift+x, Super+Alt+Shift+x and Super+Ctrl+Alt+x, sent as injected shortcuts to a foot that keeps what it gets, between the text `start:` and `:end`:

| look | red (4d8bbe36) | green (02e745c3) | bound chord that still acts |
|---|---|---|---|
| tiling | foot typed `start:9xX:end` | `start::end` | Super+2 views tag 2 |
| xp | not run on red | `start::end` | Super+d shows the desktop and back |
| mc | not run on red | `start::end` | Super+F1 shows the key list |
| construct | co-25 of gate-construct-cube.sh, same chords to the probe window (step cube PASS) | same | Super+d, Super+k of the face manager (existing co-25 cells) |

The give keys of every look act in the existing cells: gate-join-keys (xp-window: Super+Alt+1 gives the foot to tiling), the matrix and give-holders gates (all four holders). Unit tests: `a_bound_chord_keeps_its_action_and_the_rest_is_swallowed` (Super+Alt+2, Super+Ctrl+Shift+Left stay bound), `the_give_keys_are_no_chords_of_the_swallow`.

### Gate

`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 --allow-skip docs` (no --push) on 02e745c3, in four runs because the gate stops at the first red and cut the steps that ran beside it:

- Run 1 (all steps from the start): red-first to hud PASS (matrix PASS, test 339 in chora-kit), then `tiling-audit` FAIL: `FAIL: fullscreen paper-2560x1600@2: the bar does not hide over the fullscreen window within 20 s` (3 of 4 fullscreen cells ok, foot's fullscreen flag null; not a chord matter). `layout` was cut.
- Run 2 (`--only` tiling-audit to e2e): tiling-audit PASS on its own; then `launcher-mc` FAIL: `mc dark, old fuzzel: no launcher after mouse` (3 of 4 launcher cells ok; a click on Programs). `acts` was cut.
- Run 3 (`--only` launcher-mc to e2e): every step PASS, EXIT 0.
- Run 4 (`--only layout,acts`): both PASS, EXIT 0.

The two reds are not from this change (a bar flag and a mouse click, neither near a Super chord) and passed alone. Earlier head 1af09df3 also failed the matrix step (holder xp), which was caused here and fixed above.

Environment note: the box has no mkdocs and no PIL; `docs` and `ci-check` (clock selftest) need them. They went into a venv under /tmp (`pip install -r docs/requirements.txt pillow numpy`) and ran first on PATH; no system package.

`pgrep -fa 'chora --headless'`: empty after the last run.
