
## r1

Commits on claude/clock-width: red 33cf911d (tests/clock-width.py bound helper + selftest), fix 527dc847 (gate-tiling-bar-spec.sh uses it). No product code.

Red line (Sun 22 Nov 00:44 UTC, tiling-look bar-spec on 1c9e64f7): `FAIL: ink-1280x800 bar-font: the clock is 121.0 wide, not Inter 14 px weight 500 (100-119)`

Fix: the clock text is "%a %d %b %H:%M", so the bound is the Inter 14 px weight 500 width of that text (107-124 px by date) +-3 px, for the capture minute and the one before. The mono face is 134.75 px for every text and stays red (python3 tests/clock-width.py --selftest covers 366 days x 3 hours: Inter passes, mono fails, and the 120.0 of 11-22 00:36 passes). Cell now: `ok: ink-1280x800 ... clock 120.0`.

Gate: ci-check,tiling-look PASS (93 s, 579 s). Full `merge-gate.sh --jobs 1` (merged with origin/claude/autonomous-dev-session-3w-i33pg7, already up to date): 26 cells PASS incl. tiling-look 584 s; acts FAIL, not caused by this change: `FAIL: construct I6 passes but is listed in known-red.toml (guide-checks r2: construct-mode-step-in-text: the contrast); remove the row` (acts: 231 pass, 5 known red, 1 blocking). Not rerun.
