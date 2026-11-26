# flake-cube

## r1

- Base 16977e22, head b8cd91362 on claude/flake-cube-r1: 6d727741 (red pin), 26e4f1fd (fix), b8cd9136 (until_ok in place of a raw sleep, which the sleep ratchet counted). Diffstat: tests/gate-construct-cube.sh, 17 insertions, 4 deletions. No product change.
- Cause: harness, two of them, both in `end_others` of tests/gate-construct-cube.sh.
  1. Key before focus (the rfs2-2 line, "the first probe window has no keyboard"). The gate clicks the first probe window and types `a` at once, and checks only that the key arrived. The keyboard comes to the window through xp after the click; when xp is late, `a` is typed into nothing and is never sent again. Log of a red run (full cube step, 16977e22): `ended 23723 kept 23475; click 115,26; first s1 at 112,24; surfaces now: s1 xp VIRTUAL-1 112 24`, construct focus holder null, xp state `focus: {holder: xp, surface: s1}`: the focus did come, after the key. A person types after the focus; the product gave the keyboard to the window.
  2. Wrong window ended (a new red, message "a probe window of a program of the face stays after its end"). end_others finds the first probe window as the lowest pid of the session. pid_max is 32768; after a wrap a later window has the lowest pid, so the first window ends and the later one stays. Log lines: `ended 32219 kept 374; first s1`, `ended 32436 kept 354`, `ended 31798 kept 380` (the ended pid is the first window, the kept pid the wrapped one).
- Not found: a product fault. In 175 click-and-type rounds of the same code path with the click at once after the windows left the scene (the launcher loop and the Super+t window, one window at a time), the focus was on s1 before the click or within 20 ms of it, 175 of 175; under CUBE_LOAD=1, 72 of 72. What made xp late in the natural reds I did not capture (no trace of xp on those runs): the cause is shown by the pin, not by a recording of the natural stall.
- Red: `end_others stalled` stops xp (SIGSTOP) from before the end of the probe windows and lets it go 1.5 s later: red 3 of 3 on 6d727741 with the rfs2-2 line, and the same state (focus holder xp, s1) after. Fix 26e4f1fd: after the click the gate waits (until_ok on the focused flag of the first window) before it types; it keeps the pid of the first probe window instead of looking for the lowest pid. The wrap itself has no pin (a pid wrap cannot be fed); the log lines above are its evidence.
- Counts (cube step alone, `BIN=target/debug tests/gate-construct-cube.sh`, 16977e22, failure text with diagnostics only): 26 runs, 22 green, 4 red: 2 "no keyboard" (including the first run), 2 "stays" (pid wrap). Two more "stays" in about 95 shortened runs (the cell to the end of co-77). After the fix: 20 of 20 green alone, with the stalled xp in the cell.
- Gate: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --push): all steps passed, cube PASS 133 s, EXIT 0. A first full run stopped at ratchets (the raw sleep of the stall, fixed in b8cd9136). No chora --headless left.

Last 20 lines of the gate:

```
launcher-click PASS    17s /tmp/merge-gate.tq0vf9/launcher-click.log
voice-acp  PASS    12s /tmp/merge-gate.tq0vf9/voice-acp.log
voice-events PASS     2s /tmp/merge-gate.tq0vf9/voice-events.log
agent      PASS    39s /tmp/merge-gate.tq0vf9/agent.log
tiling-keys-more PASS    41s /tmp/merge-gate.tq0vf9/tiling-keys-more.log
acts       PASS  1917s /tmp/merge-gate.tq0vf9/acts.log
tiling-system PASS    16s /tmp/merge-gate.tq0vf9/tiling-system.log
handover   PASS    25s /tmp/merge-gate.tq0vf9/handover.log
mc-scale   PASS    14s /tmp/merge-gate.tq0vf9/mc-scale.log
tiling-windows PASS   178s /tmp/merge-gate.tq0vf9/tiling-windows.log
watchdog-starved PASS    49s /tmp/merge-gate.tq0vf9/watchdog-starved.log
xp-run     PASS    13s /tmp/merge-gate.tq0vf9/xp-run.log
demo       PASS   210s /tmp/merge-gate.tq0vf9/demo.log
tiling-bar PASS   171s /tmp/merge-gate.tq0vf9/tiling-bar.log
tiling-theme PASS    18s /tmp/merge-gate.tq0vf9/tiling-theme.log
leak       PASS   470s /tmp/merge-gate.tq0vf9/leak.log
mc-host    PASS    18s /tmp/merge-gate.tq0vf9/mc-host.log
tiling-disc PASS    70s /tmp/merge-gate.tq0vf9/tiling-disc.log
e2e        PASS   273s /tmp/merge-gate.tq0vf9/e2e.log
all steps passed; no --push, nothing is pushed
```
