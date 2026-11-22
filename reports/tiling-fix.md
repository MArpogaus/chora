
## r1

Head claude/tiling-fix `011db749` (base 4e46028b; tests 4774e302, 011db749). No core change; no Rust change.

- ti-37 (clock width): cause named. The clock text `%a %d %b %H:%M` is 107 to 128.7 px wide in Inter 14/500 (widest "Wed 06 May 04:44"), "Sun 22 Nov 00:36" is 120.0; the first gate range 100-119 was a fixed guess. The head already had the text-following bound (33cf911d, tests/clock-width.py). Red: the old fixed bound as the bound in the new selftest: `FAIL: the widest text Wed 06 May 04:44 (128.7) is outside 100.0-119.0`, `FAIL: 120.0 is outside 100.0-119.0 for Sun 22 Nov 00:36`, 345 failures. Green: `ok: clock-width bound`. The selftest now holds the widest text of the year in its bound, asserts the old range misses it, and runs in tests/tiling-scripts-selftest.sh (merge-gate ci-check). gate-tiling-bar-spec.sh 4 of 4 cells ok; 5/5 green.
- ti-27 (key list after power menu): not reproduced as "no way to close". Red probe (one Escape after the bar power icon over the key list): the list stays open (the menu takes the keyboard, the core sends tiling no event for it, so tiling cannot dismiss the list itself; I tried a focus-event hook, it never fired, reverted). A second Escape closes the list (POLS: Escape closes the top layer each time; the list says "Esc closes it"). New cell in tests/gate-tiling-power.sh: `ok: Escape closes the power menu, the next Escape the key list under it (ti-27)`, 5/5 green. Left to the PM: a visible close or a dismiss on the bar click needs a core event for layer-surface focus.
- ti-25c (floated window half off-screen): absent on this head. Float via Super+Ctrl+space, Super+drag to (1279,799) on 1280x800: rect [1080, 600, 200, 200], inside the output. The clamp is in wm.rs float() and its test a_floating_window_never_ends_off_screen. No code.
- Counts: clock selftest 5/5, tiling-scripts-selftest 5/5, bar-spec 5/5, power gate 5/5.
- Gate: full tests/merge-gate.sh --jobs 2 on the branch merged with fresh origin/claude/autonomous-dev-session-3w-i33pg7 (already up to date): all steps passed, EXIT 0, no foreign reds.
- Capture: reports/tiling-fix-r1-bar.png (bar, ink 1280x800).
