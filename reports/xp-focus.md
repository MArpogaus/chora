# xp-focus

## r1

- Base a93bc75b, head e19b470b on claude/xp-focus-r1 (one commit, rfc1-2). Diffstat: tests/gate-construct-cube.sh, 3 insertions, 1 deletion. No product change, no red commit (see below). Sleep ratchet: sleep 778, sleep-all 929, unchanged.
- Verdict on rfc1-1: xp is not late in its own code; the lateness is the core's reply time to xp's transactions, and a fix is not in xp. So by the brief only step 4 is done.
- Measure (debug build, the gate's own, cube cell, 2 cube runs, trace in xp, temporary, not committed): time from the start of `adopt` to the return of the transaction that holds `focus::give` (decoration group, focus, in one `try_apply`), 20 runs, ms: 176 167 160 156 165 157 155 163 164 12 162 9 158 158 162 156 157 158 157 162 (24 runs: the last four 158 16 158 15). 20 of 24 take 155-176 ms; 4 take 9-16 ms (the later ones, a warm core). The part that xp builds (adopt start to focus built) is 0.1-3.9 ms in all 24 runs: the focus command is queued at once. The rest is the socket round trip of `conn.try_apply` (crates/chora-kit/src/conn.rs: `client.apply`, a plain request and answer): the core applies the transaction (decoration group, textures) before it answers, and `focus::give` is its last command.
- The same on the way out (event handlers of xp, 4 + 1 runs of the cell): `FocusChanged{None}` after a program ends (restyle, sync_tasks, one apply) 8-66 ms, `SurfaceDestroyed` (forget: refocus the topmost window, sync_tasks, one apply) 6-8 ms or 50-56 ms; xp gives the focus to the window left in that handler (`FocusChanged{Some(s1)}` follows in under 1 ms) unless a click on it is already queued. xp's loop is single-threaded: a click that comes within ~100 ms after the end of a program waits behind these 50 ms handlers (`PointerPressed s1` handled 52-55 ms after `SurfaceDestroyed` in 2 of 7 sequences; typical 2 ms). That is the natural red of flake-cube (rfc1-1): the key goes to the core's focus holder, and the click is not yet handled.
- Why no unit test: adopt already focuses in the same round (one transaction, the focus after the decoration), so a unit test of "focus set in the same round" is green on the base: there is no red to show. A gate case that clicks and types at once without waiting for `focused` is red by the loop above only when the core's apply takes about 50 ms, which a debug build under load does; it is the flake, not a pin.
- Product finding (for the PM, not fixed here; kit/core file needed): a click on a window should not depend on a round trip through xp's busy loop to give it the keyboard. Options: the core focuses the clicked client surface itself, before it tells the window manager (crates/chora), or xp splits the focus into its own small transaction ahead of the decoration and the taskbar commands (xp, but it changes the atomicity of the adopt and wants its own look). A release build was not measured (no release binaries in this session); the 155 ms is a debug-core number.
- rfc1-2 (NIT): `end_others stalled` uses the named helper `stall_for QUARTERS PID` (6 quarters = 1.5 s, through `until_ok`, so no new sleep line); no comment (the inline-comment hook forbids one). Cube cell alone: green, 2 + 1 runs with the stall.
- Gate: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --push), no step filter: all steps passed, 68 PASS, cube PASS 150 s, EXIT all passed. No chora --headless left.

Last 20 lines of the gate:

```
launcher-mc PASS    44s /tmp/merge-gate.kczhKY/launcher-mc.log
voice-acp  PASS    13s /tmp/merge-gate.kczhKY/voice-acp.log
voice-events PASS     3s /tmp/merge-gate.kczhKY/voice-events.log
agent      PASS    42s /tmp/merge-gate.kczhKY/agent.log
tiling-keys-more PASS    45s /tmp/merge-gate.kczhKY/tiling-keys-more.log
tiling-windows PASS   222s /tmp/merge-gate.kczhKY/tiling-windows.log
tiling-system PASS    17s /tmp/merge-gate.kczhKY/tiling-system.log
handover   PASS    30s /tmp/merge-gate.kczhKY/handover.log
mc-scale   PASS    18s /tmp/merge-gate.kczhKY/mc-scale.log
acts       PASS  2529s /tmp/merge-gate.kczhKY/acts.log
watchdog-starved PASS    57s /tmp/merge-gate.kczhKY/watchdog-starved.log
xp-run     PASS    15s /tmp/merge-gate.kczhKY/xp-run.log
demo       PASS   291s /tmp/merge-gate.kczhKY/demo.log
leak       PASS   488s /tmp/merge-gate.kczhKY/leak.log
tiling-theme PASS    19s /tmp/merge-gate.kczhKY/tiling-theme.log
tiling-bar PASS   208s /tmp/merge-gate.kczhKY/tiling-bar.log
tiling-disc PASS    74s /tmp/merge-gate.kczhKY/tiling-disc.log
mc-host    PASS    25s /tmp/merge-gate.kczhKY/mc-host.log
e2e        PASS   293s /tmp/merge-gate.kczhKY/e2e.log
all steps passed; no --push, nothing is pushed
```

## PM review r1 (10:18 UTC)

- Base a93bc75b, head e19b470b, +2 -1, tests/gate-construct-cube.sh only (rfc1-2: named `stall_for`, no new sleep line; ratchet 778/929 unchanged). merge-tree onto 3be27e32 clean (checked at train).
- No product change, so no red commit is owed (rules.md 8 applies to fixes). Gate EXIT 0 (68 PASS, no --push).
- rfc1-1 (MINOR, product): the measure holds (adopt queues focus in 0.1-3.9 ms; the core round trip is 155-176 ms debug). The fix is core focus on click (crates/chora) or a split focus transaction in xp; new core behaviour, not RC scope: backlog, known issue. Not a BLOCK or MAJOR (one lost key when clicking and typing within ~100 ms of a program end, debug build).
- Verdict: MERGE-OK e19b470b, train 154.
