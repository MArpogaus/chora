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
