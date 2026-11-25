# co-close

## r1 (rc 1: co-75 BLOCK, co-76 MAJOR; findings/construct.md, reports/rc1-pass.md "## r1 construct" rpc1-1, rpc1-2)

Lane co-close-r1, opus high, cloud. Branch `claude/co-close-r1`, base 280b9dcf (co-pull r2), head 8517022f. Step 0 green:
`git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`.

### Confirmed by hand first (fresh `tests/persona/session.sh construct 1280x800`, 280b9dcf)

- co-75 (rpc1-1 R2 path): Super+t, `echo paneB`, Super+c, Return, Start > Terminal on face 0, `echo faceA`, Super+x.
  Before: `s1 construct foot` (the pane at home in front of the cube), `s2 xp foot` (parent `xp-desk-0`, core focus s2).
  After Super+x: `s2 xp foot` only; `state.get` `windows [] focus {'holder': None}`; `pgrep -c -x foot` 1. The pane out of
  view and its shell were gone, the focused face window stayed.
- co-76 (rpc1-2): the new gate step on 280b9dcf, flown into the foot pane (core focus s1), 5 s after the fly-in:
  `FAIL: co-76: after the 3 s of the hints nothing in the pane leads out: no hint Step back (core: s1  xp: - - panes: 1 ...)`.

### Commits

| sha | what |
|---|---|
| f5570063 | red co-75: `cube/tests_construct.rs` `super_x_in_a_face_closes_the_focused_face_window_and_keeps_the_pane` (left `["s1"]`), `super_x_in_a_face_without_a_focused_window_closes_nothing` (left `["s1"]`); new `tests/gate-construct-close.sh` in the construct-extra cell: `FAIL: co-75: Super+x left the focused face window s2 open: core: s2 xp: xp s2 panes: 0` |
| 96927adc | red co-76: `a_pane_flown_into_in_the_walk_keeps_its_way_out_after_the_hints_went`; gate step 0 (wait out the 3 s, the hint Step back and All keys on screen, a click on Step back steps out): red line above |
| 5e42478a | red co-75 (xp side): `a_close_that_another_client_asks_for_closes_the_window_of_xp` (`node.requested` close, D31) |
| 272940fc | fix co-75, co-76 (below) |
| 090d17cf | test only: the three tests move to `cube/tests_construct/close_tests.rs` (size ratchet: `tests_construct.rs` 1531 production lines; a `tests_*.rs` name counts as production) |
| 8517022f | test only: `gate-construct-k1.sh` compares the pane with the window alone outside the resting hint (`png-same.py --outside`) and asserts the hint rests over the pane |

### Fix (272940fc)

- co-75: `adopt.rs` `close_focused` replaces `close_ahead` (keys.rs `Host::close`). One rule: the window the person sees
  focused. In a face of the cube (Mode::Use) that is the core focus (`core_focus`, the window xp shows focused), never the
  window of a pane (out of view there); with no focused window nothing closes. Elsewhere it is the pane of the focus ring
  (`frame.rs` `focus()`, the glow: the pane flown into, else under the pointer, else ahead), not one hidden behind the
  stood-back cube (`covers_cube`). The close of a face window goes to its owner as `node.requested` close (D31); xp
  answered only `update`, so `plugins/chora-xp/src/wm/dispatch.rs` now answers `close` for its own windows, as tiling
  does (`answer_request`). Outside the briefed area (xp, 6 lines); without it Super+x in a face could close nothing.
- co-76: `hints.rs` `resting_set`: `(true, _) => Rest`, so a pane flown into keeps the resting capsule `Alt Esc Step back
  · Super F1 All keys` after its 3 s, the same as a face (co-42, co-61); both chips take a click. The bottom-edge dwell
  stays as it was: over a window the construct gets no pointer motion (co-42), so a dwell cannot work there; the lasting
  cue is the briefed alternative.
- Docs: `docs/configuration.md` (the `Super+x` paragraph, formerly :952, and the hint paragraph), `docs/construct-style.md`
  (the resting chip in a face or a pane), `plugins/chora-construct/construct.yaml` (the `close` line).

### Hand runs, 5x each, each from a fresh compositor (head 272940fc binaries)

rpc1-1: Super+t, `echo paneB`, Super+c, Return, Start > Terminal on the face, `echo faceA`, Super+x, Super+x again,
Alt+Escape. rpc1-2: start card row Terminal (a click), wait 6 s with no key, a click on what shows, no key known.

| run | rpc1-1: face window closed by Super+x | pane kept (s1) | foot of the pane alive | 2nd Super+x, no face window: nothing closed | rpc1-2: way out on screen at 6 s | click on it steps out (walk has keys) | time from fly-in |
|---|---|---|---|---|---|---|---|
| 1 | yes (s2 gone) | yes | yes (foot 2 -> 1) | yes | Step back [471 708 167 32], All keys | yes | 6.4 s |
| 2 | yes | yes | yes | yes | same | yes | 6.4 s |
| 3 | yes | yes | yes | yes | same | yes | 6.4 s |
| 4 | yes | yes | yes | yes | same | yes | 6.4 s |
| 5 | yes | yes | yes | yes | same | yes | 6.4 s |

Rates: rpc1-1 5/5 (face window closed as told, pane kept, foot alive), rpc1-2 5/5 (a way out found within 10 s, no key).
Stood back after each rpc1-1 run: `windows ['s1'] focus {'holder': 'construct'}`, the terminal beside the cube.

### Gate

Full gate on 8517022f (`tests/merge-gate.sh --jobs 2`, run 3): 64 steps PASS, `EXIT 0`, no foreign red. Last 20 lines:

```
launcher-click PASS    16s /tmp/merge-gate.M2NrPi/launcher-click.log
voice-acp  PASS    12s /tmp/merge-gate.M2NrPi/voice-acp.log
voice-events PASS     1s /tmp/merge-gate.M2NrPi/voice-events.log
agent      PASS    39s /tmp/merge-gate.M2NrPi/agent.log
tiling-keys-more PASS    42s /tmp/merge-gate.M2NrPi/tiling-keys-more.log
tiling-windows PASS   149s /tmp/merge-gate.M2NrPi/tiling-windows.log
tiling-system PASS    11s /tmp/merge-gate.M2NrPi/tiling-system.log
acts       PASS  1917s /tmp/merge-gate.M2NrPi/acts.log
mc-scale   PASS    11s /tmp/merge-gate.M2NrPi/mc-scale.log
handover   PASS    24s /tmp/merge-gate.M2NrPi/handover.log
watchdog-starved PASS    52s /tmp/merge-gate.M2NrPi/watchdog-starved.log
xp-run     PASS    12s /tmp/merge-gate.M2NrPi/xp-run.log
demo       PASS   232s /tmp/merge-gate.M2NrPi/demo.log
leak       PASS   325s /tmp/merge-gate.M2NrPi/leak.log
tiling-theme PASS    18s /tmp/merge-gate.M2NrPi/tiling-theme.log
mc-host    PASS    19s /tmp/merge-gate.M2NrPi/mc-host.log
tiling-bar PASS   129s /tmp/merge-gate.M2NrPi/tiling-bar.log
e2e        PASS   262s /tmp/merge-gate.M2NrPi/e2e.log
all steps passed; no --push, nothing is pushed
EXIT 0
```

Earlier runs, both mine and fixed: run 1 on 272940fc `FAIL: size-ratchet: plugins/chora-construct/src/cube/tests_construct.rs: 1531 production lines, over 1500 and not listed` (fixed by 090d17cf); run 2 on 090d17cf construct-extra `FAIL: the output differs from the window alone after the Step back hint faded: 18404 206 (pixels, worst channel)` (gate-construct-k1.sh: the resting hint of co-76 now stays over the pane by design; fixed by 8517022f, then mcpane, nested, k2a, windows, 3d, i3, focus and close green alone).

`tools/sleep-ratchet` (from 284ce9a5, not on this base, run with `--root` on a copy of this tree): `ok: sleep-ratchet (sleep 768,
sleep-all 918)`; the new gate adds three sleep lines (5 s that a person waits out, two waits for a close that must not
come). `pgrep -fa 'chora --headless'`: empty.

### Notes

- The close of a face window goes through the owner (D31). Another manager on a face that does not answer `node.requested`
  close would leave Super+x without effect there (never a wrong window).
- In the walk the close now follows the focus ring (pane under the pointer before the pane ahead): the same target the
  glow shows, which `close_ahead` did not follow.
- act.py clicks take positions of the last capture (800 wide at 1280): the rc1-pass repro `click 480 363` is the screen
  point 768, 581; the row Terminal is `click 300 227` on an 800x500 capture.
- Context used about 200 k.

## Review r1

Seeded review, opus high, cloud, read-only on code: `claude/co-close-r1` 280b9dcf..8517022f (14 files). Step 0 green
(`git fetch --unshallow`, `cargo check --workspace`); `cargo test -p chora-construct` 282 ok, `cargo test -p chora-xp`
348 ok; `tools/lint-ratchet` ok, `tools/size-ratchet` ok. No merge gate run (lane: EXIT 0, 64 PASS).

Checks:
1) Red first: holds. At 5e42478a the two co-75 unit tests, the co-76 pane test and the xp `node.requested` close test
   all FAIL; 272940fc turns them green; 090d17cf and 8517022f touch tests only. 8517022f does not weaken K1 beyond the
   chip: `hint_boxes` masks only drawn layers of `construct-hint*` (query rects come from the draw lists), so the resting
   capsule and its shadow; the rest of the output is still compared pixel for pixel, and the chip is asserted present.
2) co-75: the rpc1-1 path holds (face window closed, pane and its process kept, nothing closed with no face focus);
   stood back, a pane hidden in front of the cube is spared (`covers_cube`). One path left: rcc1-1.
3) xp: holds. The new arm answers `close` only for `self.index(&node).is_some()`, as tiling's `answer_request` checks
   `wm.has(node)`; other looks untouched; xp's close button and Alt+F4 paths unchanged.
4) co-76: holds by code. `Set::Rest` is the same two items (`Step back`, `All keys`) at the same place in a pane as in
   a face, so it covers no more of the pane than a face's chip; `hint_set` returns None while `self.help`, so it hides
   under the key list; both chips are hits (the gate clicks Step back; the lane's 5x clicked both).
5) Docs match the code except the rcc1-1 state; merge-gate.sh only adds the cell (rcc1-4).

| id | severity | file:line | text | fix |
|---|---|---|---|---|
| rcc1-1 | MAJOR | plugins/chora-construct/src/adopt.rs:314 | the face branch is taken whenever the cube is in use mode, also while the camera is flown into a pane from a face (the co-77 / co-79 state: Super+t or Programs in a face adopted as a pane and flown into); `core_focus` is then the pane's window, the filter drops it, and Super+x closes nothing while that pane fills the screen focused; a regression (`close_ahead` closed it) and against docs/configuration.md:953 ("the pane flown into" first). Probe on 8517022f (`in_a_face_beside_a_pane`, `fly_in(0)`, `take_flight`, focus s1, `Host::close`): `mode=Use closed=[] panes 1->1` | take the face branch only when `!self.in_pane()`, so a pane flown into always closes, and add that unit test (red first) |
| rcc1-2 | MINOR | plugins/chora-construct/src/adopt.rs:320 | with `cube_manager: chora-mc` (a documented choice, names.rs:217) the close of a face window goes to mc as `node.requested` close (D31), which mc does not answer (no `NodeRequested` arm in plugins/chora-mc), so Super+x on a focused mc face window silently does nothing, though configuration.md says it closes | answer `close` for mc's own windows as xp now does, or say in the docs and a notice that the face manager must answer it |
| rcc1-3 | NIT | tests/gate-construct-close.sh:61,95,103 | three fixed sleeps (5, 1, 2 s) where polls could stand, and no `tools/sleep-ratchet` exists in the tree to bound them; step 3 never asserts that no face window has the keys (`until_ok ... \|\| true`), so "no window focused" is assumed | poll the resting set (`rect_of construct-hint-step-back` with the 3 s passed) instead of `sleep 5`, and fail step 3 unless the core focus holds no xp window |
| rcc1-4 | NIT | tests/merge-gate.sh:24 | the help text lost the "and" between `3d (...)` and `i3 (...)` | restore the comma and "and" before the last item |

Verdict: **FIX** (open: 0 BLOCK, 1 MAJOR rcc1-1; 1 MINOR, 2 NIT). Known, not findings: co-77, co-78, co-79, co-44, co-45.

## r2 (rc 1: rcc1-1 MAJOR from review r1; co-75 BLOCK, co-76 MAJOR stay fixed)

Lane co-close-r2, sonnet high, cloud. Branch `claude/co-close-r2`, base 8517022f (co-close r1 head), head fbdcd6e2.
Step 0 green: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`.

### Commits

| sha | what |
|---|---|
| 4f994c1e | red rcc1-1: `close_tests.rs` `super_x_flown_into_a_pane_from_a_face_closes_that_pane` (face beside a pane, `fly_in(0)`, `take_flight`, focus s1, `Host::close`) |
| f0ab2938 | fix rcc1-1: `adopt.rs` `close_focused` takes the face branch only when `!self.in_pane()` |
| fbdcd6e2 | test only: rcc1-3 step 3 of `gate-construct-close.sh` fails unless no face window keeps the keys; new step 4 (rcc1-1 on the compositor); rcc1-4 the "and" of the `merge-gate.sh` help text |

Red line at 4f994c1e (the r1 tests stay green, 3 of 4 ok):

```
assertion `left == right` failed: Super+x closes the pane flown into
  left: []
 right: ["s1"]
test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 279 filtered out
```

Green at f0ab2938: `cargo test -p chora-construct` 283 ok.

### Fix

One condition, no second rule: `let in_face = !self.in_pane() && cube in Mode::Use`. A pane flown into (or on the way in)
always closes, by the same path as in the walk (`focus()` ring, `remove_pane`); a face of the cube still closes only the
window with the keyboard, never a pane out of view. docs/configuration.md `Super+x` paragraph is still true ("the pane
flown into" first, then "in a face of the cube the window of the face"); only the doc comment of `close_focused` changed.

### Hand runs, 5x each, each from a fresh compositor (`tests/persona/session.sh construct 1280x800`)

The binaries of 272940fc..8517022f first: reproduced rcc1-1 live (`pane.fly` into the pane while the cube is in use mode,
Super+x: surfaces stayed `s1`, foot 1 -> 1). Then f0ab2938 binaries, per run: rpc1-1 as r1 (Super+t, `echo paneB`, Super+c,
Return, Start > Terminal on the face, `echo faceA`, Super+x, Super+x again), then Super+t in the face, Super+x, then
`pane.fly` into the pane from the face and Super+x.

| run | face window closed, pane kept, foot 2 -> 1 | 2nd Super+x, nothing | Super+t in a face, Super+x closes it only | pane flown into from a face: Super+x closes it (foot 1 -> 0) |
|---|---|---|---|---|
| 1 | yes | yes | yes (s3 gone, s1 kept) | yes |
| 2 | yes | yes | yes | yes |
| 3 | yes | yes | yes | yes |
| 4 | yes | yes | yes | yes |
| 5 | yes | yes | yes | yes |

Rates: 5/5 on every column. Note: in a real compositor Super+t and the Programs launcher in a face give a window of the
face manager (xp), not a pane (`free` is false there), so a pane flown into while the cube stays in use mode is reached by
`pane.fly` (the method that the windows list and `choractl` use), not by Super+t; the Super+t column is the face-window
close again, kept as a check that nothing else closes.

### Test only (rcc1-3, rcc1-4)

- `gate-construct-close.sh` step 3: `until_ok '! core_focus | grep -qx "$face" && ! xp_focus | grep -q "^xp s"'` now
  fails (was `|| true`), so "no window focused" is asserted. New step 4: `pane.fly` into the pane that waits, the core
  focus is the pane, the cube is still `use`, Super+x ends the pane's program. The gate stays in the construct-extra cell.
- `sleep 5` stays: nothing on the screen tells the 3 s hint set from the resting chip (`construct-hint-step-back` and
  `construct-hint-keys` are in both), and `state.get busy` does not cover the hints; a poll on the ids would be a
  guess. The two other sleeps (1 s, 2 s) wait for a close that must not come. Not done, said so: the rest of rcc1-3.
- `tests/merge-gate.sh` help text: "3d (...), i3 (...), focus and close (...)".

### Gate

Full gate on fbdcd6e2 (`tests/merge-gate.sh --jobs 2`): 64 steps PASS, `EXIT 0`, construct-extra PASS 179 s, no foreign
red. A first run on the same tree (a second commit attempt of mine ran during it, the hook rewrote files) stopped in
pre-commit ("guard test ... files were modified by this hook"), not a code red; the rerun on the clean tree is green.
Last 20 lines:

```
launcher-click PASS    19s
voice-acp  PASS    11s
voice-events PASS     2s
agent      PASS    39s
tiling-keys-more PASS    42s
tiling-windows PASS   166s
tiling-system PASS    11s
handover   PASS    26s
mc-scale   PASS    15s
watchdog-starved PASS    59s
acts       PASS  2152s
xp-run     PASS    13s
leak       PASS   316s
demo       PASS   247s
tiling-theme PASS    18s
mc-host    PASS    20s
tiling-bar PASS   144s
e2e        PASS   267s
all steps passed; no --push, nothing is pushed
EXIT 0
```

`tools/sleep-ratchet` is not in this tree (not run). `pgrep -fa 'chora --headless'`: empty.

### Notes

- Not in scope, still open: rcc1-2 (mc face close, MINOR), co-77, co-78, co-79, co-44, co-45.
- Commit message hooks reject the Co-Authored-By / Claude-Session lines; none added, as the brief says.

## Review r2 (PM, 2026-11-24 23:30 UTC; diff 8517022f..fbdcd6e2, 4 files +44 -7, under 200 lines; PM not the builder)

1. Red first: 4f994c1e (test only, fails `left: [] right: ["s1"]`) before fix f0ab2938. Ok.
2. rcc1-1: `close_focused` (adopt.rs:314) takes the face branch only when `!self.in_pane()`; one condition, no second rule. A pane flown into closes through the `focus()` ring as in the walk. Ok.
3. rcc1-3: step 3 now fails (`|| fail`, was `|| true`); new step 4 asserts the close live (pane.fly, cube stays use, Super+x ends the pane's program). `sleep 5` kept with a reason (no signal tells the 3 s hint set from the resting chip): accepted, residue of rcc1-3 stays MINOR.
4. rcc1-4: "and" restored in the merge-gate help text. Ok.
5. Hand runs 5/5 on all columns from a fresh compositor; old binary reproduced closed=[]. Full gate 64 PASS, EXIT 0 on fbdcd6e2.
Findings: rcc2-1 COSMETIC adopt.rs:311 doc comment line no longer wrapped to the block's width. rcc2-2 MINOR (open question, no repro): while flying back out of a pane (`in_pane()` still true until the flight ends) Super+x closes that pane; check in the co-79 round.
Counts: BLOCK 0, MAJOR 0, MINOR 1, COSMETIC 1. Verdict: MERGE-OK (fbdcd6e2, train134).

## waits r1

Head 22bf5fb7 on claude/co-close-waits-r1 (7f519bc8 + merge of 84f66d43 df111a40 + one test commit).

Edits, tests/gate-construct-close.sh only (0 `sleep` left in it):
- `sleep 5` becomes a wait on the hint bar's glass: the timed set draws it at alpha 0.84, the resting set at 0.92 (`construct-hints-glass` color[3] > 0.9), so the step-back rect is read only after the 3 s are over.
- `sleep 1` and `sleep 2` (both negative checks) become `handled`: a core round trip, a construct state.get, `quiet`, a second state.get; then the pane is asserted alive.

Ratchets (head with 84f66d43 merged):
- sleep-ratchet: ok (sleep 778, sleep-all 929); list lowered from 779/930 (tree was 781/932 with the merge).
- size-ratchet: ok (0 files over 1500). lint-ratchet: ok (allow 360, no-reason 0, eprintln 46).
- The merge commit carries a `ratchet-raise:` line only because tools/sleep-ratchet.list comes in as a new file (the hook reads a new file as a raise); no count rose.

Close cell, 10 runs in a row: runs 1-10 all rc=0 (10/10 green).

Red proof, the fault put back by hand (git revert of 272940fc conflicts, f0ab2938 does not touch these checks), the cell turns red each time, then the tree was restored:
| fault | result |
|---|---|
| face branch of close_focused also removes pane 0 | FAIL co-75: Super+x in a face closed the pane out of view (panes: 0) |
| no window focused: falls through to remove pane 0 | FAIL co-75: ... with no window focused in the face closed the pane out of view (panes: 0) |
| resting set shows nothing (`(true, _) => None`) | FAIL co-76: the hints did not come to rest after their 3 s |

Full tests/merge-gate.sh (no --only): all steps passed, rc 0, run dir /tmp/merge-gate.i4Arnt (65 logs). A first run failed at clippy on a stray empty plugins/chora-construct/src/lib.rs that I made by a touch; removed, not committed, rerun green.
