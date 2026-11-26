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

## Review r1

Review of claude/kit-swallow-r1, 4d8bbe36..02e745c3 (9 files, +252 -28), opus, cloud. Verdict: **MERGE-OK** (no BLOCK, no MAJOR).

Checks and commands:

1. Red first: `cargo test -p chora-kit --lib swallow` fails on b279bd09 (`super_chords_with_every_modifier_combination_are_swallowed`: "Super+Alt+a ... the safety layer keeps only Ctrl+Alt") and passes on 37af9880; fails on 752af69f (`the_give_keys_are_no_chords_of_the_swallow`: "Super+Alt+1") and passes on 1c57c3d5. 02e745c3 has no own red commit; its red is real: `HOLDER=xp ONLY='^(xp mc|tiling mc) key$' tests/gate-handover-matrix.sh` (the xp cell of gate-give-holders) on 1c57c3d5 fails with `FAIL: the give keys were served by [tiling ], expected only xp`, and passes on 02e745c3.
2. rpt2-1: `tests/gate-swallow-chords.sh` on 02e745c3 EXIT 0 (tiling, xp, mc: the five chords type nothing into a real foot that keeps its bytes, `start::end` asserted; a plain text arrives; one bound chord acts per look); the construct cell is co-25 of gate-construct-cube.sh (keycodes 10, 45 checked at the probe). The give keys are left out of `free_chords` for a plugin with give keys, and all four looks set `give_keys` before their first `grab_swallowed` (tiling main.rs:794, xp startup.rs:336, mc lifecycle.rs:232, construct camera.rs:539). The looks' bindings are grabbed before the swallow in the same set, so a bound chord wins (tiling MOD+Ctrl+Shift+1..9 and +space, xp Super+Ctrl+Shift+Left/Right stand-ins).
3. Core: MAX_GRABS 1024 is a per-connection bound; the grab path scans a Vec (no allocation per key press in `grab_of`); the error at the limit is code 6 "a connection holds at most 1024 grabs". A list of 586 chords fits the existing `input.grab` API; a pattern grab would need a new core trigger: the list is KISS, no finding.
4. Escape hatch: `press_action` (input.rs:466) runs the safety layer (Ctrl+Alt with BackSpace, Escape, i; VT keys) before any grab and with Super held too; the core refuses those grabs (`is_safety_key`, ipc.rs:1112) and the kit's `safety()` names the same 15 keys of KEYS, so no swallow can hide them. Injected keys skip the safety layer (D23), as before.
5. Docs: api.md and architecture.md say 1024 as the code; D118 is stale (rksr1-1).

`cargo test -p chora-kit -p chora -p chora-xp`: all ok (0 failed). `cargo clippy -p chora-kit -p chora -p chora-xp --all-targets -- -D warnings`: clean. `pgrep -fa 'chora --headless'`: empty after the runs.

Findings:

- rksr1-1, MINOR, docs/decisions.md:167: D118 still says the swallow holds Super "also with Shift and Ctrl" at "about 200 grabs" and a join lets go "about 200 `input.ungrab`"; fix: amend D118 with rpt2-1 (every combination of Shift, Ctrl and Alt without the safety layer, 586 chords, the give keys left out, MAX_GRABS 1024).
- rksr1-2, MINOR, tests/gate-swallow-chords.sh:80: the bound chord shown per look (Super+2, Super+d, Super+F1) is of the class that was swallowed before; no cell shows that a bound chord of the newly swallowed class still acts (only the unit test does); fix: in cell_tiling send Super+Ctrl+Shift+2 and assert the tile gets tag 2.
- rksr1-3, MINOR, crates/chora-kit/src/runtime.rs:256: `give_keys()` after the first `grab_swallowed` silently loses the give keys (the xp bug of 02e745c3), an ordering trap for the next look; fix: let `grab_give_keys` take a give key that the plugin holds as swallowed (swap its action) or document the order on `give_keys`.
- rksr1-4, NIT, tests/gate-swallow-chords.sh:99: the fix 37af9880 changed the red cell of xp (Super+space opens the Start menu; xp binds Super+space to the launcher), so the xp cell was never red; fix: none needed, the report already says xp and mc were not run on red.
