
## r1

gate-alive r1 (F6, D91; B4, H3, H4, T2, T3). Branch claude/gate-alive at a7ffdf22 (one commit a7ffdf22 on the session head 1c9e64f7; origin/claude/autonomous-dev-session-3w-i33pg7 was still 1c9e64f7 when merged, nothing to merge). Step 0: unshallow, cargo check and build green.

Note: the five scripts of the brief (k2a, 3d, windows, voice-acp, voice-events) are already steps of the head (construct-extra, voice-acp, voice-events). The real list was 32 scripts, mostly tiling gates.

### List before (command: tests/*.sh whose name no non-comment line of tests/, scripts/, tools/, .github calls)

    for f in tests/*.sh; do b=$(basename $f); grep -rhF "$b" tests scripts tools .github Makefile --exclude="$b" | grep -vE '^\s*#' | grep -q . || echo $b; done

capture-tiling, gate-0.7-5, gate-0.7-7, gate-0.8-11, gate-0.9-daily, gate-0.9-hardware, gate-demo, gate-handover, gate-mc-host, gate-mc-scale, gate-tiling-{bar-keys,bar,chip,doors,focus,follow,gaps,keycaps,keys,motion,notice,programs,reload,spacing,swallow,system,theme}, gate-watchdog-starved, gate-xp-run, mc-capture, no-agent-trailers, xp-assets.

### List after

capture-tiling, mc-capture, xp-assets (capture tools), no-agent-trailers (commit-msg hook), gate-0.9-daily, gate-0.9-hardware (hardware, by hand), gate-0.7-5 (needs podman and an installed unit: SKIP in the sandbox), gate-0.8-11 (a chain of the earlier suites, not a check). None is a gate step; named and left.

### Runs (3 each on the head; the gate lock serialises them) and verdict

- green 3/3, joined: gate-demo (step demo), gate-handover (handover), gate-mc-scale (mc-scale), gate-watchdog-starved (watchdog-starved), gate-xp-run (xp-run), tiling chip, doors, focus, follow, keycaps, reload, swallow, system (steps tiling-keys-more, tiling-windows, tiling-system).
- gate-0.7-7: green 3/3 but it runs the unit test of process.rs, which the step test runs: deleted (git rm).
- red 3/3, harness fault, fixed: gate-tiling-programs (start without --no-companions: the bar took the terminal's place; fixed by --no-companions), gate-tiling-gaps (default shadows light the pixel left of the border: config with shadows off, plus --no-companions), gate-tiling-motion (focus glow tints the red channel, the shadow is lighter at 12 px gap: green channel, spots 10 and 1270 with 6), gate-tiling-keys (keycap 'Super+<thin space>→' is 58.0 wide, the gate counted the thin space as a full glyph: 0.56 of a glyph).
- 1 red of 3, harness fault, fixed: gate-tiling-bar-keys (Escape not taken right after the click: a second wait and up to three injects; green 3/3 after).
- red 3/3, not fixed in 30 min, filed, step known-red-NAME (a full run leaves it out; --list shows it; --only runs it): gate-mc-host (mc-57), gate-tiling-theme (ti-39), gate-tiling-bar (ti-40, stale gate or design), gate-tiling-spacing (ti-41, same), gate-tiling-notice (ti-42; its screenshot-notice half is green).
- The known-red mechanism is new in tests/merge-gate.sh (a step named known-red-*); merge-gate-selftest green.

### T2, T3

T2: gate-construct-hud.sh has no pinned row count since 2c5d810a; green in construct-hud. T3: gate-construct-windows is green (construct-extra, 162 s with k1, mcpane, nested, k2a and 3d); the 11-09 and 536aaee reds are gone. Both marked DONE in backlog.md.

### Gate

tests/merge-gate.sh --jobs 1 (no --only) on a7ffdf22: every step PASS up to tiling-audit, then tiling-look FAIL (foreign, not caused: tests/gate-tiling-bar-spec.sh, 4 lines such as "FAIL: ink-1280x800 bar-font: the clock is 120.0 wide, not Inter 14 px weight 500 (100-119)" in /tmp/merge-gate.bDvQm6/tiling-look.log; this is ti-37, seen by tour-fix r2 on 87fdbeb9; not rerun). The gate stops at the first red, so the remaining 34 steps ran as a second run with --only (tiling-empty to e2e, run dir /tmp/merge-gate.fiFgB0): all PASS, including the eight new ones.
Added wall time, from that run: tiling-keys-more 50 s, tiling-windows 153 s, tiling-system 15 s, handover 25 s, mc-scale 18 s, watchdog-starved 53 s, xp-run 14 s, demo 218 s = 546 s (9 min), at --jobs 1.
