# co-pull

## r1

Lane: cloud, claude/co-pull from f1457035 (co-reach r4 head). Step 0: unshallow, `cargo check --workspace` and `cargo build --workspace` green. One area: plugins/chora-construct (hint bar), docs/cube-modes.md. MINOR rows untouched.

**Head: be6e62eca5fe203cbddb76b4f34cc6b6d06c6832** (red 9f257e39, fix be6e62ec).

### co-68 reproduced: yes
Fresh compositor per run (persona round.sh start, construct 1280x800, f1457035 binaries; script: Super+c, Foot from Programs, then (a) Alt+Escape so the pane stands beside the cube, or (b) stay in the pane full screen; Super+Alt+4; state dump before and after). Runs where Foot did not start (the fuzzel click of the script missed; mode use, no pane) are not counted.

| State | Valid runs | Before the key | After Super+Alt+4 |
|---|---|---|---|
| (a) pane beside the cube | 6 (of 9 started) | mode navigate, front 0, camera [0,0], pane foot at [-1562,3136], s1 construct visible at [17,283,383,233], holder construct surface s1; hint bar `Click Fly in ... Super Alt 4 Pull out` 6/6 | identical dump 6/6: no change |
| (b) in the pane, full screen | 5 (of 5) | mode navigate, camera [-1191,2391], s1 at [0,0,1280,800], holder construct s1; hint bar `Super F1 All keys, Alt Esc Step back` (no pull named) 5/5 | identical dump 5/5: no change |

### Decision (docs/cube-modes.md "Flows" row pull and the stood-back paragraph; vision card POLS: a told way must work or not be told)
- (a) The hint names a key that does nothing: the hint is not told. It is told only where the key acts.
- (b) The hint bar never named the pull in a pane (Flown set), so nothing to unname; the key stays silent there. (Max's "full screen" report was the pane beside the cube; the hint is only in the stood-back set.)

### Cause
`plugins/chora-construct/src/input.rs:183` (`give_key`): `Some(node) if node.owner.as_deref() == Some(to) => return Ok(())`: the key gives the window with the keyboard to the construct, and a window of a pane is the construct's already, so it returns silently. The hint bar told the key unconditionally while a pane stood beside the cube: `overlay.rs` `pane_hints` (was lines 619-641, `pull-out` item always pushed). With the keyboard in the pane (co-67: it stays with s1) or in no window, the key can never act; it acts only with a window of a face that has the keyboard or had it at the step back (`noted_focus`).

### Fix
`overlay.rs`: `pull_acts()` (the window with the keyboard from `core_focus`, else `noted_focus`, and it is no pane window of the construct) decides whether `pane_hints` adds `Super Alt 4 Pull out`; `hint_stamp()` includes it so the bar redraws when it changes (also pulled out of `draw_hints`, which was over the clippy line limit). No new mode, no sleep, no known row, no raised bar. docs/cube-modes.md follows.

### Red then green
- Red 9f257e39: `the_pull_key_is_named_only_where_it_acts` (cube/tests_construct.rs) fails on f1457035 (pull-out hint shown with no window to pull); the old test of co-64 now asserts only the click hint (`a_pane_beside_the_cube_names_its_click`).
- Green be6e62ec: `cargo test -p chora-construct` 276 passed, `cargo clippy -p chora-construct --all-targets` clean.

### After the fix (be6e62ec binaries)
| State | Valid runs | Result |
|---|---|---|
| (a) pane beside the cube | 12 (of 16 started; the 4 others: Foot did not start, as above) | hint bar `Click Fly in ... All keys`, no `Pull out` 12/12; key silent (no change) |
| (b) in the pane | 11 (of 12 started) | hint bar `Step back, All keys`, no `Pull out` 11/11 |
| tests/gate-construct-i3.sh | 10 of 10 green | co-61, co-62, co-64, co-65, co-67 hold (steps in gate cells cube, pull, construct-hud, construct-extra, pixels) |

Not shown by a persona or by hand: the positive path (a face window with the keyboard, then Pull out told and the key pulling it). It is covered by the unit test (hint shown with a window of a face in focus or noted) and by the gate cell `pull` (the drag; the give key itself is cell handover); no hand run of the key with a face window beside a pane was made here.

### Gate
`tests/merge-gate.sh --jobs 2` (no --only), detached, on claude/co-pull be6e62ec merged with fresh origin/claude/autonomous-dev-session-3w-i33pg7 2235551f (merge 17539f17, not pushed): **all 65 steps PASS, EXIT 0**; no foreign red. `pgrep -fa 'chora --headless'` empty after.

## Review r1 (PM, 2026-11-24 12:40 UTC; diff 81+/20- under 200 lines, PM not the builder)
Base f1457035, head be6e62ec: docs/cube-modes.md, plugins/chora-construct/src/overlay.rs, cube/tests_construct.rs.
1) Red first: 9f257e39 (test only) before be6e62ec; `the_pull_key_is_named_only_where_it_acts` asserts no "pull-out" hint with no focus, which the old `pane_hints` (always pushed) fails. Holds.
2) Fix: `pull_acts()` = core focus or the focus noted at step back, not the construct's own window (overlay.rs:648-652); the hint stamp includes it (`hint_stamp`), and `draw_hints` runs after every event (scene.rs:100), so a focus change redraws the bar. Holds.
3) POLS, KISS: the hint names only a key that acts; no new mode or setting; docs/cube-modes.md follows. Holds.
4) Gate: lane full gate EXIT 0, 65/65, on the merge with 2235551f.
Findings:
- rcp1-1 | MINOR | tests_construct.rs:1250-1293 | the told-and-pulls path (a face window with the keyboard, the hint shown, Super+Alt+4 pulls it) is checked by a unit test of the hint only, no gate cell or hand run of the pull | Fix: persona r11 C2 and the co-64 5x run check it on screen.
- rcp1-2 | MINOR | overlay.rs:634-641 | in demo state (pane beside, keyboard in the pane) the bar no longer names any way to pull; C2 now rests on the title drag or a step in first; co-64 "the way is told on screen" narrows | Fix: persona r11 measures C2 reach; a told title-drag hint only if it is not reached.
Verdict (computed): 0 BLOCK, 0 MAJOR, 2 MINOR -> MERGE-OK be6e62ec.

## r2

Lane: cloud, claude/co-pull-r2 from be6e62ec (co-pull r1 head). Step 0: unshallow, `cargo check --workspace`, `cargo build --workspace` green. Area: plugins/chora-construct (overlay.rs, command.rs, input.rs), cube/tests_construct.rs, tests/gate-construct-cube.sh, docs/cube-modes.md.

**Head: 280b9dcf5821e41784aeeb7cfe0b36041e201989** (red 5316338c co-70, red f9e6cfbe co-71, fix 280b9dcf). Pushed to claude/co-pull-r2.

### co-70 (pull key not told in a face window with no pane beside): fixed
Red 5316338c `the_pull_key_is_named_in_a_face_window_stood_back_with_no_pane_beside` fails on be6e62ec:
```
tests_construct.rs:1319:5: the keyboard is in a window of a face: the key pulls it out, so it is named
```
Cause: `pane_hints` returned nothing without a pane beside the cube, so `pull_acts()` was never asked. Fix 280b9dcf: `pane_hints` adds the click hint by `pane_beside()` and the pull hint by `pull_acts()` alone, the predicate `give_key` follows (no second rule); the hint stamp already held both. co-68 holds: pane beside with the keyboard in the pane stays silent (unit test of r1 green, hand run below).

### co-71 (key sheet): reproduced by hand, cause found, fixed
Not reproduced as written (Max's three failures at once): in the states of the r11 path (row Terminal, Super+c, Ctrl+Alt+Left, drag the pane to the face, Programs, Foot, the Keys door) Escape closed the sheet 4 of 4. What did fail, on be6e62ec (script scratchpad hand/co71b.sh, capture pixels as the persona clicks):
- **Close**: a click on Close left the sheet open (6 of 6 runs: with the probe pane, without it, and in a plain cube with no window). `choractl events` at the click: `pointer.pressed node construct-look position [1035.2,755.2]`: the press does not reach the Close hit. State after: `open: [construct-help sheet modal], focus holder construct`.
- **Click outside**: no handler at all; the same press went to `construct-look`.
- **Escape after a click outside on a pane beside the cube** (probe pane s3): the press gave the keyboard to the pane window (`focus holder construct surface s3`, the sheet still open), then Escape and Close did nothing: `after click: open [help], focus s3; after Escape: open [help]; after Close: open [help]`. This is Max's T5 (click on the grey slab first).
Cause: the root `construct-help` and the body were `input: pass` groups; the core hands a mode other than `accept` down the subtree (chora-scene `descend`: `input: if input == Accept {node.props.input} else {input}`), so the Close hit (accept) was pass, and the scrim let the click through. The unit test of the Close (event sent straight to the plugin) never saw it.
Red f9e6cfbe: `command::tests::the_close_and_the_scrim_of_the_key_sheet_can_be_pressed` (`help-body would hand pass or block down to the Close`), `a_press_outside_the_key_sheet_closes_it` (`a press outside the sheet closes it`), and the gate step in tests/gate-construct-cube.sh on the old binaries: `FAIL: co-71: a click on the Close of the key list opened from a face keeps it`.
Fix 280b9dcf: root and body accept; the scrim accepts and a press on it closes the sheet; a block panel over the sheet keeps a press on the sheet from the scrim; no press reaches the space or a pane behind the open sheet. Gate cell cube after the fix: `ok: co-71: the key list opened from a face closes by its Close, by a click outside and by Escape`.
By hand after the fix (the same path with the probe pane beside): click outside closes (1 of 1), Escape closes (1 of 1), Close closes 6 of 8 runs (1 plain cube, 7 with the probe pane); the two misses: one with `pointer.pressed node hud-chat-stop` at the same point (1035,755), the chat HUD of chora-assistant, an overlay of another client, over the Close in that run; the other (3 sessions at once on 4 CPUs, no event capture) not attributed, not repeated. **Not fixed here**: the overlay is above any node of the scene; a finding for the PM (the sheet's Close and the HUD Stop button overlap at 1280x800 while the chat HUD shows).

### Hand runs, 5x each from a fresh compositor (fixed binaries 280b9dcf, scripts of r11: co68.sh, reach.sh)
| Row | Steps | told | acts |
|---|---|---|---|
| co-68 / co-70 (no pane beside) | row, Return, Super+t, Alt+Escape, Super+Alt+4 | **5/5** (`... Super Alt 4 Pull out`; r11: 0/5) | **5/5** (s1 xp -> construct, rect 290,150,700,500; r11: 5/5) |
| rcp1-1 (pane beside) | row Terminal, Super+c, Return, Super+t, Alt+Escape, Super+Alt+4 | 5/5 | 5/5 (s2 xp -> construct) |
| co-64 (pane beside, keyboard in the pane) | row Terminal, Super+c, hint, drag on the pane | click named 5/5; pull not named 5/5 (as designed, co-68); the script's "click and pull named" flag 0/5 for that reason | the drag moves the pane 5/5 (s1 at [1125 400], cube front 0) |
Non-I6 breaches: 0 in every co68 run (the co-44 gap rows are filtered as r11 did).

### Gate
`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --only), detached, on 280b9dcf: **all steps PASS, EXIT 0**, no foreign red. Last 20 lines:
```
launcher-click PASS    18s launcher-click.log
voice-acp  PASS    12s voice-acp.log
voice-events PASS     2s voice-events.log
agent      PASS    39s agent.log
tiling-keys-more PASS    42s tiling-keys-more.log
tiling-windows PASS   160s tiling-windows.log
tiling-system PASS    11s tiling-system.log
handover   PASS    26s handover.log
acts       PASS  1997s acts.log
mc-scale   PASS    12s mc-scale.log
watchdog-starved PASS    49s watchdog-starved.log
xp-run     PASS    13s xp-run.log
demo       PASS   258s demo.log
leak       PASS   327s leak.log
tiling-theme PASS    18s tiling-theme.log
mc-host    PASS    20s mc-host.log
tiling-bar PASS   138s tiling-bar.log
e2e        PASS   268s e2e.log
all steps passed; no --push, nothing is pushed
EXIT 0
```
`pgrep -fa 'chora --headless'` empty after. `tools/sleep-ratchet` is not on this branch (it lives on claude/autonomous-dev-session-3w-i33pg7); the diff adds 0 `sleep` lines to tests/ (`git diff be6e62ec..HEAD -- tests tools | grep -c sleep` = 0), so the count cannot rise.

### Open for the PM
1. **Pull out in a face (use mode)**: Super+Alt+4 also acts with the keyboard in a window of a face (stepped in); the bar there (`Step back`, `All keys`) does not name it. The brief's state list was the stood-back one; I named it only there. Naming it in the face set is a design call (the face hints are timed and short).
2. **hud-chat-stop over the sheet's Close** (above).
3. Repository hooks reject a commit message with Co-Authored-By or Claude-Session lines (hook `no-agent-trailers`), so none are in these commits, as the brief says.

## Review r2 (review lane, 2026-11-24; claude/co-pull-r2 280b9dcf vs be6e62ec, 6 files +164 -19; reviewer not the builder)

**Verdict: MERGE-OK** (0 BLOCK, 0 MAJOR, 2 MINOR, 1 NIT). Step 0: unshallow, `cargo check --workspace` green. No merge gate run (lane: EXIT 0).

Checks:
1. Red first: 5316338c (co-70) and f9e6cfbe (co-71) are tests only and come before the fix 280b9dcf. On f9e6cfbe (pre-fix code) all 3 fail for the stated reasons: `tests_construct.rs:1340` (keyboard in a face window, no pane beside: the hint is not named), `command.rs:993` (pass/block ancestor of the Close), `tests_construct.rs:1022` (a press on the scrim does not close). Ok.
2. co-70: `pane_hints` adds the click hint on `pane_beside()` and the pull hint on `pull_acts()` alone; `pull_acts` follows `give_key`'s order (core focus, else the window noted at the step back; a pane window is the construct's own, so a no-op), no second rule; `hint_stamp` already holds `pull_acts`, so a focus change redraws. Silent with the keyboard in the pane (co-68 unit test of r1 green). `cargo test -p chora-construct`: 279 passed, 0 failed. Ok.
3. co-71: root, body and scrim accept; the block panel sits under the rows, the search and the footer (pushed before them), so a press on the body background goes to nobody, a press on a row reaches the plugin and is ignored (rows have no handler, no `-hit`), the Close hit is above the panel and closes. A press cannot reach the look, a face or a pane behind the open sheet: the hint capsule is hidden while `help` (`hint_set` busy), and the sheet is re-created on top under `construct` at each open and redraw. No dead layer: the close fades to `visible: false`, and the core's walk skips invisible nodes (but see rcp2-1 for the 120 ms of the fade). Escape unchanged (`close_help_by_escape`), gate step green. Core: `descend` and `found` hand a non-accept mode down the subtree, as docs/api.md:69 states (`pass` and `block` apply to all descendants); the fix is in the plugin only (no diff under crates/). Ok.
4. Gate step uses `until_ok` only, 0 added `sleep` lines; docs/cube-modes.md matches the code (Close, click outside, Escape; the hint with or without a pane beside). Ok.
5. Other looks: no change in chora-kit or any crate; xp, mc, tiling unaffected.

Findings:
- rcp2-1 | MINOR | plugins/chora-construct/src/overlay.rs:309-312 | on close `help` turns false and the sheet fades 120 ms (`hud::FADE_OUT`) with its accept scrim still visible; `help_input` then ignores a press on the scrim and nothing else takes it, so a click in the 120 ms after Escape or Close is lost (before r2 the pass root let it through) | fix: in the close path set the scrim (or the root) to `input: pass` at once, before the fade.
- rcp2-2 | MINOR | plugins/chora-construct/src/input.rs:733 (footer at command.rs:224-236) | with a search typed the footer button reads `Esc Clear`, but a press on it toggles the whole sheet closed while Escape only clears; the hit was unreachable before r2, so the told word and the act now differ live | fix: route the footer press through `close_help_by_escape` so the click clears first, as its label and Escape do.
- rcp2-3 | NIT | plugins/chora-construct/src/cube/tests_construct.rs:1019 | the half "a press on the sheet keeps it open" sends a press on `construct-help-body-panel`, a block node the core never delivers, so it checks only the name filter; no unit test holds the Close above the block panel (the gate step does) | fix: in the command test assert the panel's index precedes the foot hit, and drop the vacuous half.
