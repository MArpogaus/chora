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
