
## r1

Commits on claude/clock-width: red 33cf911d (tests/clock-width.py bound helper + selftest), fix 527dc847 (gate-tiling-bar-spec.sh uses it). No product code.

Red line (Sun 22 Nov 00:44 UTC, tiling-look bar-spec on 1c9e64f7): `FAIL: ink-1280x800 bar-font: the clock is 121.0 wide, not Inter 14 px weight 500 (100-119)`

Fix: the clock text is "%a %d %b %H:%M", so the bound is the Inter 14 px weight 500 width of that text (107-124 px by date) +-3 px, for the capture minute and the one before. The mono face is 134.75 px for every text and stays red (python3 tests/clock-width.py --selftest covers 366 days x 3 hours: Inter passes, mono fails, and the 120.0 of 11-22 00:36 passes). Cell now: `ok: ink-1280x800 ... clock 120.0`.

Gate: ci-check,tiling-look PASS (93 s, 579 s). Full `merge-gate.sh --jobs 1` (merged with origin/claude/autonomous-dev-session-3w-i33pg7, already up to date): 26 cells PASS incl. tiling-look 584 s; acts FAIL, not caused by this change: `FAIL: construct I6 passes but is listed in known-red.toml (guide-checks r2: construct-mode-step-in-text: the contrast); remove the row` (acts: 231 pass, 5 known red, 1 blocking). Not rerun.

## Review r1 (PM; 1c9e64f7..527dc847, 2 files +71 -5)
- Red first: 33cf911d (helper and selftest) before 527dc847.
- The bound follows the clock text ("%a %d %b %H:%M", Inter 14 px 500, +-3 px) for the capture minute and the one before; the mono face (134.75 px) stays red on every date (selftest 366 days x 3 hours). Cell ok at clock 120.0.
- cw-1 MINOR tests/clock-width.py:18: the Inter path is fixed; a host with Inter elsewhere fails with a Python error, not a named cell line.
- cw-2 NIT: bounds() uses the test's local time; correct only while the compositor and the test share TZ (they do in the gate).
- Open, not this diff: acts `construct I6 passes but is listed in known-red.toml (guide-checks r2 ...)`; sh-32 and gate-waits saw a green or other acts run on the same head, so the row is intermittent. Train 105 shows it again or not.
Counts: BLOCK 0, MAJOR 0, MINOR 1. Verdict: MERGE-OK.
