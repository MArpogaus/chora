# flake-hint

## r1

Branch claude/flake-hint, head d2634fe0 (red e046fce1, fix 6b5bec5c, merge of origin/claude/autonomous-dev-session-3w-i33pg7 at 6b01c976). Test code only (tests/acts/looks/construct.py); no product fault.

### Before (3b14c825, construct look of the acts cell, 8 runs)
3 of 8 red (runs 3, 6, 8; the loop was stopped after run 8 because I edited the test file while it ran, so runs 6 and 8 may overlap short edit windows; the failure lines are the ones below):
- "the hint Keys of a face opens the key list by a click": 2 of 8 (`expected open ['construct-help'], got []`, I4 `construct-help was declared open and is not`)
- "the hint Keys of a face names a key that opens the key list": 1 of 8 (`RuntimeError: the hint keys names no key`)
A quick runner (cube open, one window, keys_acts) saw the names-no-key red in 1 of 4.

### Cause
1. The hint is drawn in two layouts. Stepped in, it is Flown (Step back + Keys, rect [647,708,162,32] in the default session, [676,708,104,32] with the F1 grant) for 3 s, then Rest (Keys alone, centred, [559,708,162,32] / [588,708,104,32]). Trace lines: `0.58 [647,708,162,32]` ... `3.18 [559,708,162,32]`. The failure still of the click act shows the capsule `F1 All keys` centred at x 579..701 while the act clicked the centre of the earlier rect (x 728). The act read the rect inside the 3 s and clicked after they ran out (slow host: the hint appears 0.3 to 1 s after the pointer rests low), so the click missed.
2. After `Return` the cube still reports mode navigate for a moment and still draws the stood-back hint `construct-hint-keys` at [829,732,162,32]. `hint_shown` took that rect (`hint_shown ok 0.04 navigate False [829,...]`), the step in then removed it, and `press_hinted` read no keycap node: "names no key". A click read in that window lands where no hint is.

### Fix (test only)
- `cube_uses`: wait for mode use and no turn after the Return, before the hint cues.
- `stable_rect`: the rect counts when two reads 0.25 s apart agree.
- `hint_click`: aims at the rect, moves onto it, reads the stable rect again, aims again if it moved (3 tries, else RuntimeError "moves away from the pointer"), then clicks.
- `press_hinted`: waits up to 5 s for the keycap nodes instead of reading once.
- Red first: e046fce1 adds `dwell=3.5` between the read and the click of the act; red with `expected open ['construct-help'], got []` (deterministic, 1 of 1). 6b5bec5c keeps the dwell as the regression and is green. The "names no key" red has no deterministic variant (it needs the 0.04 s window after the Return); it is covered by `cube_uses` and the wait for the keycaps.
- The product moves the hint under a pointer that leaves the bottom 64 px when the 3 s have run out; not changed (a design question for the owner, not a fault of the act).

### After
20 runs of the construct look (`tests/acts/run.py --no-selftest construct`, 800 to 840 s each): 20 of 20 pass, `acts: 83 pass, 2 known red, 0 blocking` each time. (Run directly, not through gate-acts.sh, which adds ~100 s of selftests the act does not touch.)

### Gate
`tests/merge-gate.sh --jobs 1` on d2634fe0: all steps passed, 0 FAIL, 0 SKIP (acts PASS 1380 s).
