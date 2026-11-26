# mc-boil

## r1

Branch claude/mc-boil-r1, base 329a13e5 (mc-resume r1), head f10be960 (red c3afca51, fix c3925fc5, gate-cell follow-up f10be960). Step 0 green: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`.
Diffstat 329a13e5..f10be960: 3 files, +142 -3 (chora-mc layout.rs, rooms.rs; tests/gate-mc-sofa.sh).

**The state found (step 1).** It is the timer card at the foot, as m2, m2b and m3 of the tour show it (the card of `notify.push`, chora-notify at the bottom left, on the boil page). With the card the page ends at y 559 (page rect [0,56,1280,503]); the side panel stands at y 528, so only 31 px of it are over the edge. Its lines node `mc-s-boil-panels-side-lines` is [160,530,185,29]: one line of 28 px and a 1 px strip of the next. The cause: the clip of a panel group counted whole lines from the top of the group (`whole_lines`), but the lines start a lift below it (2 px here), and the page edge lies on the baseline grid plus a descent, not on a line end. The same cut also stood with no card (sauce side 113 of 140 px, from the edge at 643) and, with no page edge involved, the lead of the boil page was 273 of 280 px (5 lines of 56, clip 7 px short).

**Red** c3afca51, a unit test in rooms.rs and the mc-sofa cell `boil-lines` (saves the boil page, fills it as the tour does, pushes the timer card, then asks every `-lines` node of the page for a whole number of lines inside the page). Both fail on the base:
- unit: `lines cut through, no card: sauce/side 113 of 140 px; a card: boil/side 29 of 84 px, ingredients/list 29 of 112 px, sauce/side 29 of 140 px`
- gate: `FAIL: a lines node of the boil page shows part of a line at the edge of the page [0.0,56.0,1280.0,503.0] (rtr3-1): mc-s-boil-panels-lead-lines [160.0,175.0,664.0,273.0]` and `mc-s-boil-panels-side-lines [160.0,530.0,185.0,29.0]`.

**Fix** c3925fc5. `clip_height` gives the clip of a panel with lines: it counts whole lines from where the lines start (`lift`) and from the room the page leaves the panel (edge minus panel top minus scroll). A panel with no lines (empty words, rows of times) keeps `whole_lines`. `clip_panels` sets the clip again on every scroll, so the clip follows the edge. No option, no timer, no kit helper. The page still says that it goes on (the more mark is unchanged). f10be960 drops the cell's own `sleep` (sleep ratchet 778).
After: boil side lines [160,530,185,28] (1 of 3 lines whole, the mark shows); the lead is whole. 216 chora-mc tests pass, clippy and fmt clean, pre-commit hooks pass.

Rect before and after (boil side lines, the m2 state): [160,530,185,29] then [160,530,185,28].

**Captures** `media/2026-11-26/mc-boil-r1/m2-boil-before.png`, `m2-boil-after.png` (1280x720, 146 KB each, boil page with the timer card). They are pixel-identical (same md5): the cut strip is the empty top of the next line, so the fix changes the box, not the ink; the gate cell and the unit test measure the box.

**Gate**, no step filter, `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` on f10be960 (no --push): every step PASS (red-first, identity, ratchets .. mc-sofa 121 s, mc-column, mc-scale, mc-host .. e2e 264 s), last line `all steps passed; no --push, nothing is pushed`. A first run on c3925fc5 stopped at ratchets (sleep 779 > list 778 from the cell's own sleep); fixed by f10be960 and rerun in full. No foreign flake. `pgrep -fa 'chora --headless'` is empty.
